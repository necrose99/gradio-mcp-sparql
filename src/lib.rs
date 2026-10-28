"""
gradio-mcp-sparql: Gradio dashboard + MCP server over the native Kolibrie SPARQL engine.

    python main_app.py                      # dashboard on http://127.0.0.1:8181
    python main_app.py --mcp-only           # headless MCP server on stdio (for Claude Desktop etc.)
    python main_app.py --mcp-only --allow-load   # also let the MCP client load Turtle text

Safe defaults: read-only queries (SELECT/ASK/CONSTRUCT/DESCRIBE), no SERVICE (federation),
row cap, localhost-only dashboard. The graph store is one shared in-process store; it is NOT
per-user, so do not expose the dashboard to untrusted users as-is.
"""
import argparse
import asyncio
import re
import sys
import threading

try:
    import kolibrie  # native extension built by `just compile`
except ImportError:
    print("Error: native extension not found. Run 'just compile' first.", file=sys.stderr)
    sys.exit(1)

import pandas as pd

MAX_ROWS = 1000                 # rows returned to the UI / MCP client
MAX_TTL_BYTES = 50 * 1024 * 1024  # largest Turtle upload accepted
READ_FORMS = ("SELECT", "ASK", "CONSTRUCT", "DESCRIBE")


def _reraise_if_fatal(e: BaseException) -> None:
    """PyO3 turns Rust panics into pyo3_runtime.PanicException, which derives from BaseException,
    not Exception. Catch those like errors, but never swallow Ctrl-C / interpreter exit."""
    if isinstance(e, (KeyboardInterrupt, SystemExit)):
        raise e


class Policy:
    allow_service = False   # SPARQL SERVICE = outbound federated queries
    allow_load = False      # MCP clients may load Turtle text


POLICY = Policy()


# ---------------------------------------------------------------------------
# Query helpers
# ---------------------------------------------------------------------------
def _strip_comments(q: str) -> str:
    """Remove '#' comments but not '#' inside <IRIs> or string literals."""
    out, in_iri, quote = [], False, None
    i = 0
    while i < len(q):
        ch = q[i]
        if quote:
            out.append(ch)
            if ch == "\\" and i + 1 < len(q):
                out.append(q[i + 1]); i += 1
            elif ch == quote:
                quote = None
        elif in_iri:
            out.append(ch)
            if ch == ">":
                in_iri = False
        elif ch in "\"'":
            quote = ch; out.append(ch)
        elif ch == "<":
            in_iri = True; out.append(ch)
        elif ch == "#":
            while i < len(q) and q[i] != "\n":
                i += 1
            continue
        else:
            out.append(ch)
        i += 1
    return "".join(out)


_PROLOGUE = re.compile(r"\s*(?:(?:PREFIX\s+[^\s:]*:\s*<[^>]*>|BASE\s+<[^>]*>)\s*)*", re.I)
_SELECT_VARS = re.compile(r"\bSELECT\s+(?:DISTINCT\s+|REDUCED\s+)?((?:\?\w+\s*)+)", re.I)


def check_query(q: str) -> str | None:
    """Return an error message if the query is not allowed, else None."""
    body = _strip_comments(q)
    body = body[_PROLOGUE.match(body).end():].lstrip()
    word = re.match(r"[A-Za-z]+", body)
    if not word or word.group(0).upper() not in READ_FORMS:
        return "Only read queries are allowed (SELECT, ASK, CONSTRUCT, DESCRIBE)."
    if not POLICY.allow_service and re.search(r"\bSERVICE\b", body, re.I):
        return "SERVICE (federated queries) is disabled. Start with --allow-service to enable it."
    return None


def column_names(q: str, width: int) -> list[str]:
    m = _SELECT_VARS.search(_strip_comments(q))
    if m:
        names = [v.lstrip("?") for v in m.group(1).split()]
        if len(names) == width:
            return names
    return [f"col{i}" for i in range(width)]


# ---------------------------------------------------------------------------
# Graph controller
# ---------------------------------------------------------------------------
class GenericNodeController:
    def __init__(self):
        self.node = kolibrie.LocalSparqlNode()
        self._lock = threading.Lock()  # Gradio runs handlers in threads; serialize engine access

    def query_graph(self, sparql_str: str) -> pd.DataFrame:
        err = check_query(sparql_str or "")
        if err:
            return pd.DataFrame([{"Error": err}])
        try:
            with self._lock:
                raw_rows = self.node.query(sparql_str)
            if not raw_rows:
                return pd.DataFrame(columns=["Result"])
            truncated = len(raw_rows) > MAX_ROWS
            rows = raw_rows[:MAX_ROWS]
            df = pd.DataFrame(rows, columns=column_names(sparql_str, len(rows[0])))
            if truncated:
                df.attrs["note"] = f"truncated to {MAX_ROWS} rows"
            return df
        except BaseException as e:  # includes Rust panics (PanicException)
            _reraise_if_fatal(e)
            return pd.DataFrame([{"Error": f"{type(e).__name__}: {e}"}])

    def query_text(self, sparql_str: str) -> str:
        df = self.query_graph(sparql_str)
        text = df.to_csv(index=False)
        if df.attrs.get("note"):
            text += f"\n# {df.attrs['note']}"
        return text

    def load_ttl(self, file_path) -> str:
        """Load a Turtle file from disk into the store. Returns a status message."""
        if not file_path:
            return "No file selected."
        try:
            import os
            if os.path.getsize(file_path) > MAX_TTL_BYTES:
                return f"Refused: file is larger than {MAX_TTL_BYTES / (1024 * 1024):g} MB."
            with open(file_path, "r", encoding="utf-8") as f:
                return self.load_ttl_string(f.read())
        except Exception as e:
            return f"Ingestion failure: {e}"

    def load_ttl_string(self, ttl_content: str) -> str:
        if len(ttl_content.encode("utf-8")) > MAX_TTL_BYTES:
            return "Refused: Turtle text is too large."
        try:
            with self._lock:
                return str(self.node.load_turtle_string(ttl_content))
        except BaseException as e:  # includes Rust panics (PanicException)
            _reraise_if_fatal(e)
            return f"Ingestion failure ({type(e).__name__}): {e}. The graph may be in a partial state."


graph_driver = GenericNodeController()


# ---------------------------------------------------------------------------
# Gradio dashboard
# ---------------------------------------------------------------------------
def build_dashboard():
    import gradio as gr

    def execution_router(query):
        return graph_driver.query_graph(query)

    with gr.Blocks(title="Gradio MCP SPARQL Extension Node") as dashboard:
        gr.Markdown("## Gradio-MCP-SPARQL Unified Node")
        with gr.Row():
            with gr.Column(scale=1):
                query_box = gr.Textbox(
                    label="SPARQL Statement", lines=6,
                    value="SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 10",
                )
                submit_btn = gr.Button("Query Node Engine", variant="primary")
                upload_zone = gr.File(label="Hot-Load Turtle Data (.ttl)", file_types=[".ttl"])
                status_box = gr.Textbox(label="Load status", interactive=False)
            with gr.Column(scale=2):
                output_table = gr.DataFrame(label="SPARQL Output Results Matrix")

        submit_btn.click(fn=execution_router, inputs=query_box, outputs=output_table)
        # .upload (not .change) so clearing the file doesn't trigger a load; status goes to its
        # own box instead of overwriting the user's query
        upload_zone.upload(fn=graph_driver.load_ttl, inputs=upload_zone, outputs=status_box)
    return dashboard


# ---------------------------------------------------------------------------
# MCP server (stdio). Works with mcp 1.x (FastMCP) and 2.x (MCPServer).
# ---------------------------------------------------------------------------
def build_mcp_server():
    try:
        from mcp.server.mcpserver import MCPServer as Server   # mcp >= 2
    except ImportError:
        from mcp.server.fastmcp import FastMCP as Server       # mcp 1.x

    server = Server("gradio-mcp-sparql")

    @server.tool()
    def sparql_query(query: str) -> str:
        """Run a read-only SPARQL query (SELECT/ASK/CONSTRUCT/DESCRIBE) on the local graph; returns CSV."""
        return graph_driver.query_text(query)

    if POLICY.allow_load:
        @server.tool()
        def load_turtle(turtle: str) -> str:
            """Load Turtle (.ttl) text into the local graph."""
            return graph_driver.load_ttl_string(turtle)

    return server


def main(argv=None):
    ap = argparse.ArgumentParser()
    ap.add_argument("--mcp-only", action="store_true", help="headless MCP server on stdio")
    ap.add_argument("--host", default="127.0.0.1")
    ap.add_argument("--port", type=int, default=8181)
    ap.add_argument("--allow-load", action="store_true", help="MCP clients may load Turtle text")
    ap.add_argument("--allow-service", action="store_true", help="allow SPARQL SERVICE (outbound federation)")
    args = ap.parse_args(argv)
    POLICY.allow_load = args.allow_load
    POLICY.allow_service = args.allow_service

    if args.mcp_only:
        # stdout carries only MCP protocol here; everything else goes to stderr
        build_mcp_server().run("stdio")
    else:
        build_dashboard().launch(server_name=args.host, server_port=args.port)


if __name__ == "__main__":
    main()
