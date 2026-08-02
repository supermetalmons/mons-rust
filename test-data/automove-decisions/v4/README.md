# Automove decisions v4

This immutable corpus pins deterministic public `suggestMove` behavior with
`performance.now()` fixed at `0`, after the packed Pro search began verifying
reduced fail-highs at full depth and relying on its exact work-unit ceiling when
the host clock does not advance. It contains the initial state for every game
variant and one retained regression state. Every Fast, Normal, and Pro decision
also records the result of replaying its selected input.

`decisions.jsonl` is the authoritative executable contract. `manifest.json`
pins its byte size, SHA-256, record order, and counts. Do not regenerate,
normalize, reorder, or edit this v4 payload; add a new version directory for a
different selection.

The v1 payload stays on disk unchanged as the record of the previous Pro
selector.
