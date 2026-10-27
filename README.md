# gradio-mcp-sparql

[![License: MIT](https://shields.io)](https://opensource.org)
[![Rust](https://shields.io)](https://rust-lang.org)
[![Gradio](https://shields.io)](https://gradio.app)

A highly efficient, open-source **Gradio Extension and Open Model Context Protocol (MCP) Server** built on a high-performance **Rust SPARQL engine wrapper**. 

This scaffolding allows digital humanities labs, computational linguists, and semantic web researchers to drop in arbitrary W3C Semantic Web Graph files (`.ttl`, `.nt`) and instantly inherit an interactive visual data workbench *and* an AI-agent-ready tool schema out of a single zero-configuration runtime node.

---

## 🧬 Architecture Overview

`gradio-mcp-sparql` decouples your semantic graph datasets from the complex web routing, serialization, and pipeline assembly code.

```text
  [ RDF/Turtle Store (.ttl) ] 
               │
               ▼
   [ Rust Kolibrie Engine ]  <--- Compiled dynamically via PyO3 & Maturin
               │
               ▼
       [ Python Extension ]
               │
      ┌────────┴────────┐
      ▼                 ▼
[ Gradio UI ]   [ Open MCP Server ] <--- Exposes raw SPARQL capability 
(Local: 8181)   (Stdio Stream)           directly to downstream LLMs
```

* **Rust Performance Backbone:** Utilizes a native wrapper around the `StreamIntelligenceLab/Kolibrie` framework to offer memory-efficient, SIMD-accelerated SPARQL queries and rule-based datalog inference directly inside memory spaces.
* **Dual-Boot Execution Matrix:** Can be deployed simultaneously as an interactive visual web platform or as a headless command-line daemon dedicated strictly to AI agent tool processing pipelines.

---

## 🛠️ Installation & Local Build

Ensure you have **Python 3.10+**, **Rust (Cargo 1.60+)**, and **`just`** installed on your system context.

### 1. Initialize the Isolated Workspace
Clone this repository alongside your external dataset repositories and initialize your environment runner boundary:
```bash
just init
```

### 2. Compile the PyO3 Native Bindings
Run the compilation task to pull the upstream master repository source code files, resolve workspace dependencies, and construct the shared binary objects locally:
```bash
just compile
```

---

## 🚀 Execution Modes

### Mode A: Visual Dashboard & Tool Node (Default)
Launches the interactive Gradio component web panel over local system execution channels:
```bash
just run
```
* **Dashboard Portal:** Open your browser to `http://127.0.0.1:8181` to dynamically upload `.ttl` graph segments, track POS tag metadata layers, execute query statements, and audit results tables instantly.

### Mode B: Headless Open MCP Daemon Core
Launches explicitly as a stream-isolated terminal tool engine pipeline intended for agent consumption:
```bash
just run-mcp
```

---

## 🤖 Connecting to LLM Clients (Claude Desktop Configuration)

To turn your compiled local graph store into a global knowledge base tool that your AI assistant can query natively using standard SPARQL semantics, add the following node allocation matrix inside your local `claude_desktop_config.json` parameter map:

```json
{
  "mcpServers": {
    "gradio-mcp-sparql-node": {
      "command": "python",
      "args": [
        "/absolute/path/to/gradio-mcp-sparql/src/gradio_mcp_sparql/main_app.py",
        "--mcp-only"
      ]
    }
  }
}
```

Once connected, you can converse with your model using intuitive instructions:
> *"Query my local node for all lexical entries reconstructed as Proto-Algonquian verb structures, and group them by semantic sound-change properties."*

---

## 📂 Structural Layout

```text
gradio-mcp-sparql/
├── pyproject.toml              # Unified Hatch & Maturin optimization layout
├── Cargo.toml                  # Rust binding pipeline and upstream pointers
├── LICENSE                     # MIT License manifest
├── README.md                   # System configuration and user reference documentation
├── justfile                    # Local toolchain automation registry
└── src/
    ├── lib.rs                  # Native Rust PyO3 classes mapping core engine attributes
    └── gradio_mcp_sparql/
        ├── __init__.py
        └── main_app.py         # Gradio component wrappers and Open MCP server loop
```

---

## 🤝 Contributing

Contributions to improve this scaffolding are welcome! 
* Optimize the underlying string parsing vectors inside `src/lib.rs`.
* Build out a more sophisticated `networkx` topological graphing canvas interface block inside the web panel.
* Add support for streaming sliding windows.

All submitted pull request allocations are assumed to be published natively under the open-source **MIT License** terms.

---

## ⚖️ License

Distributed under the **MIT License**. See the `LICENSE` file for more comprehensive information metrics.
