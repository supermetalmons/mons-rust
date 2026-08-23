# Automove decisions v9

This immutable corpus pins deterministic public `suggestMove` behavior with
`performance.now()` fixed at `0`, after the packed evaluation changed shape for
every preference and the packed generator gained one turn-model move. The
evaluation now prices own loose regular mana by its mana-step queue position
with a cliff inside the opponent's reply horizon instead of a `weight/(d+1)`
gradient, scores the fused drainer pick-up-and-deliver distance in whole turns
instead of two separate distance terms, separates a carried supermana from an
enemy regular mana of the same point value, and adds a threshold term for a side
that needs one point and already owns mana within mana-step reach of a pool. The
generator additionally offers the mana move at any point in the turn when that
move wins the game, which a mandatory mon move could otherwise deny.

Fast searches up to 42,000 nodes, Normal up to 200,000 nodes, and Pro up to
2,000,000 nodes; the 16 ms, 75 ms, and 460 ms wall-clock budgets are unchanged.
Unsupported positions retain the canonical fallback.

The corpus retains the same thirteen source states as v8: the initial state for
every game variant and one retained regression state. Compared with v8, 2 Fast,
1 Normal, and 3 Pro observations differ as parsed decision objects; all source
metadata and FENs are unchanged.

The changed Fast observations are `initial-OuterEdgeManaRows`,
`initial-CornerChainManaRows`. The changed Normal observation is
`initial-CornerChainManaRows`. The changed Pro observations are
`initial-OuterEdgeManaRows`, `initial-SplitFlankManaRows`, `retained-release`.

`decisions.jsonl` is the authoritative executable contract. `manifest.json`
pins its byte size, SHA-256, record order, and counts. Do not regenerate,
normalize, reorder, or edit this v9 payload; add a new version directory for a
different selection.

Earlier payloads stay on disk unchanged as the record of previous selectors.
