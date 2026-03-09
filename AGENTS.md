# Agent Instructions

- Do not index, scan, or read files under `rules-tests/` unless explicitly requested.
- For automove experimentation, read `HOW_TO_ITERATE_ON_AUTOMOVE.md` first — start with the Quick Reference section and use `AUTOMOVE_IDEAS.md` as the backlog for the next iteration idea.
- All experiments use `#[cfg(test)]` harness; run via `cargo test --release --lib <test_name> -- --ignored --nocapture`.
