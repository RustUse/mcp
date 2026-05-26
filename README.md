# RustUse MCP Server

RustUse MCP Server is the AI-facing Model Context Protocol server for RustUse. It exposes RustUse catalog data, rules, adoption paths, docs, reusable prompt templates, and bounded planning tools to MCP-compatible clients.

This repository lives beside `docs`, `use-math`, `use-geometry`, `use-cli`, `use-web`, and the other RustUse repositories. It is not a `use-*` facade crate and should not be named `use-mcp`.

Version 0.1 is deliberately small and read-only. It uses stdio transport through the official Rust `rmcp` SDK. It does not write files, run commands, publish crates, call GitHub write APIs, or mutate sibling RustUse repositories.

## Why It Exists

RustUse has many small primitive Rust crates, facade sets, documentation surfaces, and adoption paths. AI clients need a stable way to ask about that structure without guessing from the filesystem or making broad changes. This MCP server gives those clients a bounded, protocol-compliant interface for catalog lookup, rule validation, overlap checks, and prompt generation.

## Local Development

Build the server:

```powershell
cargo build --release -p rustuse-mcp-server
```

Run it over stdio:

```powershell
cargo run -p rustuse-mcp-server
```

Validate the workspace:

```powershell
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Client Configuration

Use stdio transport and point your MCP client at the release binary:

```text
/path/to/RustUse/git_local/mcp/target/release/rustuse-mcp-server
```

Example client configs live in `examples/`.

## Resources

- `rustuse://catalog`
- `rustuse://rules`
- `rustuse://adoption-paths`
- `rustuse://sets`
- `rustuse://sets/use-math`
- `rustuse://sets/use-geometry`
- `rustuse://sets/use-cli`
- `rustuse://sets/use-web`
- `rustuse://sets/use-data`
- `rustuse://prompts`
- `rustuse://docs/overview`
- `rustuse://docs/security`

Resource templates are available for sets, crates, and docs where the SDK supports them.

## Tools

- `rustuse_catalog_search`
- `rustuse_get_set`
- `rustuse_get_crate`
- `rustuse_list_children`
- `rustuse_check_name_collision`
- `rustuse_find_overlap`
- `rustuse_validate_set_plan`
- `rustuse_generate_copilot_prompt`

Tools return readable text and structured JSON where practical. Domain validation failures are returned as tool execution errors, not process crashes.

## Prompts

- `rustuse_brainstorm_set`
- `rustuse_create_set_v0_1`
- `rustuse_create_child_crate`
- `rustuse_validate_facade`
- `rustuse_audit_overlap`
- `rustuse_generate_copilot_plan`
- `rustuse_generate_docs_page`

Prompt templates live in `prompts/*.md` and can be read directly outside MCP.

## Security Posture

Version 0.1 is read-only. It does not implement Streamable HTTP, file writes, shell execution, GitHub writes, crates.io publishing, git commits, repository creation, or sibling repository mutation.

Stdout is reserved for valid MCP protocol messages. Diagnostics must go to stderr.

## Roadmap

- v0.2 local repo scanner and catalog verification.
- v0.3 dry-run scaffold plans.
- v0.4 guarded file-writing behind explicit `--allow-write`.
- v0.5 integration with `rustuse` CLI / `cargo-rustuse`.
- v0.6 GitHub and crates.io metadata awareness.
- Possible future `use-mcp` facade only if reusable MCP primitives emerge.
