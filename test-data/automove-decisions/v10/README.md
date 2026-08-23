# Automove decisions v10

This immutable corpus pins deterministic public `suggestMove` behavior with
`performance.now()` fixed at `0`, after three changes to the packed engine. The
evaluation adds a six-cell antisymmetric correction over the score pair, because
a first-to-five race does not price a lead by the score difference: the marginal
value of a point rises as the need falls. It also reprices a carried bomb from
220 to 1,200 and a carried supermana from 1,200 to 4,000. The searcher no longer
grants the tactical exemption from late-move reduction, move-count pruning and
futility pruning to every loose-mana spirit push; a push keeps it only when it
carries the mana strictly closer to a pool or hands it to an own awake drainer.

Fast searches up to 42,000 nodes, Normal up to 200,000 nodes, and Pro up to
2,000,000 nodes; the 16 ms, 75 ms, and 460 ms wall-clock budgets are unchanged.
Unsupported positions retain the canonical fallback.

The corpus retains the same thirteen source states as v9: the initial state for
every game variant and one retained regression state. Compared with v9, 0 Fast,
0 Normal, and 1 Pro observation differ as parsed decision objects; all source
metadata and FENs are unchanged. The single changed observation is
`retained-release`. That small delta is expected rather than reassuring: twelve
of the thirteen states are initial positions at 0-0, where the score-pair
correction is identically zero and the repriced items are not yet carried, so
this corpus exercises little of what the round changed. The round's evidence is
the held-out self-play measurement recorded in the theory appendix.

`decisions.jsonl` is the authoritative executable contract. `manifest.json`
pins its byte size, SHA-256, record order, and counts. Do not regenerate,
normalize, reorder, or edit this v10 payload; add a new version directory for a
different selection.

Earlier payloads stay on disk unchanged as the record of previous selectors.
