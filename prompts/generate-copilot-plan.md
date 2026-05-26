# GitHub Copilot Plan Mode Prompt: {{set_name}}

We need to create or update the RustUse set {{set_name}}.

Description: {{description}}
Minimum child crates: {{minimum_children}}
Proposed children: {{proposed_children}}
Include docs: {{include_docs}}
Include CI: {{include_ci}}

RustUse constraints:

- This is a RustUse facade set, not a monolithic framework.
- Rust edition 2024.
- Child crates hold implementation; facade crates re-export and organize.
- Prefer primitive utilities and few dependencies.
- Dual license MIT OR Apache-2.0.
- Include README, CHANGELOG, tests, docs, and examples where appropriate.
- Avoid duplicate or crisscrossing taxonomy.
- Do not publish, mutate unrelated repositories, or add broad automation unless requested.

Start by inspecting the target folder and nearby RustUse workspaces. Produce a plan before broad edits. Then implement the smallest complete v0.1 that passes fmt, clippy, and tests.
