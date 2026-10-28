## /src/gradio_mcp_sparql/main_app.py
import sys
import asyncio
import pandas as pd
import gradio as gr
from mcp.server import Server
from mcp.types import Tool, TextContent

# Import the native extension compiled directly via your Cargo pointer setup
try:
    import kolibrie
except ImportError:
    print("❌ Error: Native extension not found. Run 'just compile' first.", file=sys.stderr)
    sys.exit(1)

class GenericNodeController:
    def __init__(self):
        # Initialize our PyO3 wrapper class pointing to upstream SparqlDatabase
        self.node = kolibrie.LocalSparqlNode()
        
    def query_graph(self, sparql_str: str) -> pd.DataFrame:
        """Executes query arrays directly via Kolibrie's compiled execution matrix."""
        try:
            # Query returns a raw nested Vec<Vec<String>> from the Rust layer
            raw_rows = self.node.query(sparql_str)
            if not raw_rows:
                return pd.DataFrame(columns=["Result"])
                
            # Convert raw structural arrays straight into uniform DataFrames
            return pd.DataFrame(raw_rows)
        except Exception as e:
            return pd.DataFrame([{"Error": str(e)}])

    def load_ttl(self, file_path: str) -> str:
        """Streams raw Turtle string configurations into the local Triple Store."""
        try:
            with open(file_path, "r", encoding="utf-8") as f:
                ttl_content = f.read()
            return self.node.load_turtle_string(ttl_content)
        except Exception as e:
            return f"Ingestion Failure: {str(e)}"

# Instantiate the global graph node driver
graph_driver = GenericNodeController()

# =====================================================================
# GRADIO EXTENSION LAYOUT BLOCK
# =====================================================================

def execution_router(query):
    df = graph_driver.query_graph(query)
    return df

with gr.Blocks(title="Gradio MCP SPARQL Extension Node") as extension_dashboard:
    gr.Markdown("## 🔗 Gradio-MCP-SPARQL Unified Node")
    
    with gr.Row():
        with gr.Column(scale=1):
            query_box = gr.Textbox(
                label="SPARQL Statement", 
                lines=6, 
                value="SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 10"
            )
            submit_btn = gr.Button("Query Node Engine", variant="primary")
            upload_zone = gr.File(label="Hot-Load Turtle Data (.ttl)", file_types=[".ttl"])
            
        with gr.Column(scale=2):
            output_table = gr.DataFrame(label="SPARQL Output Results Matrix")

    submit_btn.click(fn=execution_router, inputs=query_box, outputs=output_table)
    upload_zone.change(fn=graph_driver.load_ttl, inputs=upload_zone, outputs=query_box)

# =====================================================================
# SERVER RUNTIME ENGINES INVOCATION
# =====================================================================

if __name__ == "__main__":
    # Target index 1 to explicitly isolate flag evaluation inputs
    if len(sys.argv) > 1 and sys.argv[1] == "--mcp-only":
        # Run exclusively as a background Open MCP engine pipe
        # asyncio.run(mcp_server.run_stdio_async())
        pass
    else:
        # Launch standard user workspace view over http://127.0.0.1:8181
        extension_dashboard.launch(server_port=8181)

