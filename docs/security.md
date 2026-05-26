# Security

RustUse MCP Server v0.1 is read-only.

It does not:

- Write files.
- Execute shell commands.
- Commit to git.
- Create repositories.
- Publish to crates.io.
- Write to GitHub APIs.
- Mutate sibling RustUse repositories.
- Implement Streamable HTTP transport.
- Request sampling or logging capabilities from clients.

It may:

- Read embedded static catalog, rule, adoption, prompt, and doc data.
- Search and validate that static data.
- Generate prompt text.
- Return bounded planning recommendations.

For stdio transport, stdout must contain only valid MCP protocol messages. Logs and diagnostics must go to stderr.
