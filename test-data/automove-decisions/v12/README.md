# Automove decisions v12

This immutable corpus pins deterministic public `suggestMove` behavior with
`performance.now()` fixed at `0`, after one change to the packed engine: the
drainer now weighs what its trip is worth, not only how long it is.

The evaluation used to choose the drainer's plan by the shortest fused distance
— pick-up distance plus the distance from that item to a pool — over every loose
mana on the board. That minimum ignored value, although the supermana and the
other side's mana score two points against one for the drainer's own. The
evaluation now tracks a second shortest trip over two-point items only, prices
both plans by the turn bucket they land in less the steps still to walk, and
keeps whichever is worth more. `tripTwoPointScale` reads the two-point plan's
bucket at 290%; at 100 the two prices coincide and the choice collapses to the
shorter trip, which is the previous behavior.

The correction is deliberately bounded. Past roughly 330 it grows large enough
to outbid across a turn boundary, and self-play falls off a cliff there: fetching
the valuable item is right when it costs no extra turn and wrong when it does,
because the opponent gets that turn to contest the trip.

Fast searches up to 38,400 nodes, Normal up to 184,000 nodes, and Pro up to
2,000,000 nodes; the 16 ms, 75 ms, and 460 ms wall-clock budgets are unchanged.
Unsupported positions retain the canonical fallback.

The corpus retains the same thirteen source states as v11: the initial state for
every game variant and one retained regression state. Compared with v11, 0 Fast,
1 Normal, and 2 Pro observations differ as parsed decision objects; all source
metadata and FENs are unchanged. The changed observations are
`initial-OuterWedgeManaRows` at Normal and `initial-OuterEdgeManaRows` and
`initial-SplitFlankManaRows` at Pro. As in v11 that delta understates the round:
twelve of the thirteen states are initial positions where every loose mana still
sits on its own side, so the two-point choice rarely arises. The round's evidence
is the held-out self-play measurement recorded in the theory appendix.

`decisions.jsonl` is the authoritative executable contract. `manifest.json`
pins its byte size, SHA-256, record order, and counts. Do not regenerate,
normalize, reorder, or edit this v12 payload; add a new version directory for a
different selection.

Earlier payloads stay on disk unchanged as the record of previous selectors.
