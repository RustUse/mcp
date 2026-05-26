# Local Setup

Build the server from the `mcp` repository:

```powershell
cargo build --release -p rustuse-mcp-server
```

Run the server over stdio:

```powershell
cargo run -p rustuse-mcp-server
```

MCP clients should launch the release binary with stdio transport:

```text
/path/to/RustUse/git_local/mcp/target/release/rustuse-mcp-server
```

Validation commands:

```powershell
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Use the MCP Inspector or a client that supports local stdio servers to test initialization, resource listing, tool calls, and prompt rendering.
