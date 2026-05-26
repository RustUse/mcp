# Resources

The server exposes stable read-only `rustuse://` resources.

JSON resources:

- `rustuse://catalog`
- `rustuse://rules`
- `rustuse://adoption-paths`
- `rustuse://sets`
- `rustuse://sets/use-math`
- `rustuse://sets/use-geometry`
- `rustuse://sets/use-cli`
- `rustuse://sets/use-web`
- `rustuse://sets/use-data`

Markdown resources:

- `rustuse://prompts`
- `rustuse://docs/overview`
- `rustuse://docs/security`

Resource templates:

- `rustuse://sets/{set_name}`
- `rustuse://crates/{crate_name}`
- `rustuse://docs/{doc_slug}`

Unknown resources return MCP resource-not-found protocol errors.
