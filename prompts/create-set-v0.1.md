# Create RustUse Set v0.1: {{set_name}}

Create a minimal RustUse facade workspace for {{set_name}}.

Description: {{description}}
Minimum child crates: {{minimum_children}}
Proposed children: {{proposed_children}}
Include docs: {{include_docs}}
Include CI: {{include_ci}}
Include examples: {{include_examples}}

Requirements:

- Rust edition 2024.
- Workspace with a facade crate and focused child crates.
- Facade primarily re-exports child crates.
- Child crates hold implementation, tests, docs, and examples where appropriate.
- Prefer few or no dependencies.
- Dual license MIT OR Apache-2.0.
- Add README and CHANGELOG surfaces.
- Keep scope narrow and avoid taxonomy overlap.

Start in plan mode. Inspect nearby RustUse workspaces before editing.
