# Automove decisions v11

This immutable corpus pins deterministic public `suggestMove` behavior with
`performance.now()` fixed at `0`, after one change to the packed engine: the
evaluation's model of danger to a drainer. A threat is now read through the
share of its own turn the threatened side still holds. With three or more
sub-moves in hand the side to move can usually deliver, step away, or block
before the attack lands, so the threat is priced at a quarter; with one or two
it is priced at 55%; with none of its turn left, or when the other side moves
next, it is priced in full. Only once that separation exists is the underlying
threat worth pricing properly, so the immediate threat rises from 700 to 2,100
and the walking threat from 240 to 720. The evaluation also grades the drainer's
fused pick-up-and-deliver distance inside its turn bucket, which is otherwise
flat, and prices the tempo lead in half turns once either side is within two
points of the target.

Fast searches up to 38,400 nodes, Normal up to 184,000 nodes, and Pro up to
2,000,000 nodes; the 16 ms, 75 ms, and 460 ms wall-clock budgets are unchanged.
Unsupported positions retain the canonical fallback.

The corpus retains the same thirteen source states as v10: the initial state for
every game variant and one retained regression state. Compared with v10, 2 Fast,
2 Normal, and 1 Pro observation differ as parsed decision objects; all source
metadata and FENs are unchanged. The changed observations are
`initial-OuterEdgeManaRows` and `initial-CornerChainManaRows` at Fast and Normal
and `retained-release` at Pro. As in v10 that delta understates the round:
twelve of the thirteen states are initial positions at 0-0, where the tempo-lead
term is gated off and no drainer is yet under threat. The round's evidence is the
held-out self-play measurement recorded in the theory appendix.

`decisions.jsonl` is the authoritative executable contract. `manifest.json`
pins its byte size, SHA-256, record order, and counts. Do not regenerate,
normalize, reorder, or edit this v11 payload; add a new version directory for a
different selection.

Earlier payloads stay on disk unchanged as the record of previous selectors.
