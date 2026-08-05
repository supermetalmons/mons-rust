# Automove decisions v5

This immutable corpus pins deterministic public `suggestMove` behavior with
`performance.now()` fixed at `0`, after the packed Pro evaluation gained a
nearest-pool term for loose regular mana and refitted its carrier weights. It
contains the initial state for every game variant and one retained regression
state. Every Fast, Normal, and Pro decision also records the result of replaying
its selected input.

Only the Pro decision for `retained-release` differs from v4; the twelve variant
openings hold no pool-adjacent loose mana, so they select the same inputs.

`decisions.jsonl` is the authoritative executable contract. `manifest.json`
pins its byte size, SHA-256, record order, and counts. Do not regenerate,
normalize, reorder, or edit this v5 payload; add a new version directory for a
different selection.

Earlier payloads stay on disk unchanged as the record of previous selectors.
