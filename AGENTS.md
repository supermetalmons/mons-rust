# Agent Instructions

- Do not index, scan, or read files under `rules-tests/` unless explicitly requested.
- For automove experimentation, read `docs/automove-experiments.md` — start with the Quick Reference section for the 3-command pipeline (fast screen → progressive duel → full ladder).
- All experiments use `#[cfg(test)]` harness; run via `cargo test --release --lib <test_name> -- --ignored --nocapture`.
