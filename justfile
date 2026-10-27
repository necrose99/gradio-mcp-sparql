# Toolchain orchestrator for gradio-mcp-sparql development node

# Setup local sandboxed developer virtual environment boundaries
init:
    python3 -m venv .venv
    .venv/bin/pip install --upgrade pip hatch maturin

# Compile the Rust PyO3 bindings and inject the native module locally
compile:
    .venv/bin/maturin develop --release

# Run the complete test suite stack (Gradio Dashboard and background MCP server loop)
run:
    .venv/bin/python src/gradio_mcp_sparql/main_app.py

# Launch headless daemon mode dedicated strictly to Open MCP terminal requests
run-mcp:
    .venv/bin/python src/gradio_mcp_sparql/main_app.py --mcp-only
