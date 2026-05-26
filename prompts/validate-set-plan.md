# Validate RustUse Facade Plan: {{set_name}}

Set name: {{set_name}}
Description: {{description}}
Proposed children: {{proposed_children}}

Validate this set plan against RustUse rules:

- Name is unique and ASCII-safe.
- Scope is primitive and composable.
- Facade responsibility is re-export and organization.
- Child crates hold implementation.
- Proposed children are distinct and focused.
- Taxonomy does not duplicate existing sets or crates.
- Docs, tests, and examples are accounted for.
- Adoption paths are clear for crates.io, copy-and-own, and CLI-assisted use.

Return findings by severity and recommend whether to proceed, revise, or reject.
