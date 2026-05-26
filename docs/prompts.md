# Prompts

Version 0.1 exposes reusable RustUse prompt templates:

- `rustuse_brainstorm_set`
- `rustuse_create_set_v0_1`
- `rustuse_create_child_crate`
- `rustuse_validate_facade`
- `rustuse_audit_overlap`
- `rustuse_generate_copilot_plan`
- `rustuse_generate_docs_page`

Templates live in `prompts/*.md` and are useful outside MCP. The server renders them with explicit arguments such as `set_name`, `description`, `minimum_children`, `proposed_children`, `parent_set`, `include_docs`, `include_ci`, and `include_examples`.

The server does not advertise prompt list-change notifications or completions in v0.1.
