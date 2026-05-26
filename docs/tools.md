# Tools

Version 0.1 exposes read-only planning and catalog tools.

- `rustuse_catalog_search`
- `rustuse_get_set`
- `rustuse_get_crate`
- `rustuse_list_children`
- `rustuse_check_name_collision`
- `rustuse_find_overlap`
- `rustuse_validate_set_plan`
- `rustuse_generate_copilot_prompt`

All tools use JSON Schema-compatible input schemas. They return human-readable text content and structured JSON where practical.

Protocol errors are reserved for malformed MCP requests or unknown tool names. RustUse domain validation problems are returned as tool execution errors with structured payloads so clients and models can self-correct.
