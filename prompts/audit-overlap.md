# Audit RustUse Overlap: {{set_name}}

Proposed name: {{set_name}}
Description: {{description}}
Proposed children: {{proposed_children}}

Audit the proposal for overlap with existing RustUse sets and crates.

Check:

- Exact name collision.
- Hyphen and underscore normalized collision.
- Similar child crate names.
- Concept overlap with existing set boundaries.
- Crisscrossing taxonomy where a crate could belong to multiple sets.

Return likely overlaps, severity, and a recommendation.
