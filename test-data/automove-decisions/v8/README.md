# Automove decisions v8

This immutable corpus pins deterministic public `suggestMove` behavior with
`performance.now()` fixed at `0`, after the packed searcher changed for every
preference: selectively pruned nodes are now stored in the transposition table
with their real depth and a lower or upper bound instead of a depth-0 move hint,
commuting mon sub-moves of one turn are searched in canonical ascending order
only, and the published bundle targets ES2022 so class private members are no
longer lowered to WeakMap helpers. Fast searches up to TBD nodes,
Normal up to TBD nodes, and Pro up to TBD nodes; the
wall-clock budgets are unchanged. Unsupported positions retain the canonical
fallback.

The corpus retains the same thirteen source states as v7: the initial state for
every game variant and one retained regression state. Compared with v7,
0 Fast, 1 Normal, and 2 Pro observations differ as parsed
decision objects; all source metadata and FENs are unchanged.

The changed Fast observations are none. The changed Normal
observations are `initial-OuterEdgeManaRows`. The changed Pro observations are
`initial-OuterEdgeManaRows`, `retained-release`.

`decisions.jsonl` is the authoritative executable contract. `manifest.json`
pins its byte size, SHA-256, record order, and counts. Do not regenerate,
normalize, reorder, or edit this v8 payload; add a new version directory for a
different selection.

Earlier payloads stay on disk unchanged as the record of previous selectors.
