# Create RustUse Child Crate: {{set_name}} / {{crate_name}}

Parent set: {{set_name}}
Child crate: {{crate_name}}
Description: {{description}}

Create a focused RustUse child crate that belongs inside the parent set.

The child crate should:

- Hold the implementation for one primitive concept.
- Use Rust edition 2024.
- Prefer no dependencies unless clearly justified.
- Include tests and docs proportional to the API.
- Expose a small, obvious public surface.
- Fit cleanly under the parent facade without duplicating another RustUse crate.

Return a plan first, then implement the smallest useful v0.1 slice.
