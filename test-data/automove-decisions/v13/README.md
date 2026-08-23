# Automove decisions v13

This immutable corpus pins deterministic public `suggestMove` behavior with
`performance.now()` fixed at `0`, after two changes to the packed engine: a
spirit push no longer outranks every quiet mon step in move ordering, and the
attack-origin tables are reused across selection calls.

The generator gave every spirit push the ordering base `1 << 13`, while a quiet
mon step is keyed 0 — or 1,024 for the drainer — and a mana carrier's best
pool-approach step tops out at 6,144. Because the search's history bonus is
clamped to 8,191, that gap was one unit wider than history could ever close, so
a quiet push outranked every plain quiet mon step no matter how much history the
mon step had accumulated. The base is now 0 and both tactical branches add the
old base back, so the set of moves exempt from late-move reduction, move-count
pruning and futility pruning is unchanged and only the rank of a quiet push
moves.

`attackTablesFor` memoized its per-variant attack tables by the identity of the
squares array, but every selection call arrives with a fresh array, so the cache
never hit across calls and the mystic origin table was rebuilt every time. The
tables are a pure function of the squares content, so one comparison now stands
in for the rebuild. That change alters no decision at all.

Fast searches up to 38,400 nodes, Normal up to 184,000 nodes, and Pro up to
2,000,000 nodes; the 16 ms, 75 ms, and 460 ms wall-clock budgets are unchanged.
Unsupported positions retain the canonical fallback.

The corpus retains the same thirteen source states as v12: the initial state for
every game variant and one retained regression state. Compared with v12, 0 Fast,
0 Normal, and 1 Pro observation differ as parsed decision objects; all source
metadata and FENs are unchanged. The changed observation is `retained-release`
at Pro. As in v12 that delta understates the round: twelve of the thirteen
states are initial positions, where no loose mana is yet within a spirit's reach
of a pool and the reordered class is nearly empty. The round's evidence is the
held-out self-play measurement recorded in the theory appendix.

`decisions.jsonl` is the authoritative executable contract. `manifest.json`
pins its byte size, SHA-256, record order, and counts. Do not regenerate,
normalize, reorder, or edit this v13 payload; add a new version directory for a
different selection.

Earlier payloads stay on disk unchanged as the record of previous selectors.
