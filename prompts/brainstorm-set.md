# Brainstorm RustUse Set: {{set_name}}

You are planning a RustUse facade set.

Set name: {{set_name}}
Description: {{description}}
Minimum child crates: {{minimum_children}}

Work within the RustUse model:

- Use Rust edition 2024.
- Prefer small primitive utility crates over frameworks.
- Keep facade crates focused on re-exporting child crates.
- Put implementation in child crates.
- Prefer few or no dependencies.
- Use dual licensing: MIT OR Apache-2.0.
- Avoid duplicate or crisscrossing taxonomy with existing RustUse sets.

Produce:

1. A crisp set boundary.
2. Candidate child crates with one-line responsibilities.
3. Likely overlap risks.
4. A recommended first v0.1 slice.
