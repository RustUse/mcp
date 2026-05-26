# RustUse MCP Server Overview

RustUse MCP Server is the AI-facing server for RustUse. It exposes static catalog data, RustUse rules, adoption paths, documentation, reusable prompt templates, and bounded planning tools to MCP-compatible clients.

This repository sits beside `docs`, `use-math`, `use-geometry`, `use-cli`, `use-web`, and other RustUse repositories. It is not a `use-*` facade crate and is not named `use-mcp`.

Version 0.1 is intentionally read-only. It does not scan sibling repositories, write files, run shell commands, publish crates, call GitHub write APIs, or mutate RustUse workspaces.

The v0.1 server uses stdio transport and the official Rust `rmcp` SDK. MCP protocol messages go through stdout; diagnostics must go to stderr.
