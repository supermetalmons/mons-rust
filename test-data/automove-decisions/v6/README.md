# Automove decisions v6

This immutable corpus pins deterministic public `suggestMove` behavior with
`performance.now()` fixed at `0`, after public Fast and Normal adopted the
packed-state search for supported positions. Their audited profiles use 30,000
nodes for Fast and 150,000 nodes for Normal; unsupported positions retain the
canonical fallback. Pro keeps the v5 packed selector and evaluation unchanged.

The corpus retains the same thirteen source states as v5: the initial state for
every game variant and one retained regression state. Compared with v5, five
Fast observations and three Normal observations differ. All thirteen Pro
observations are byte-for-byte equivalent as parsed decision objects, and all
source metadata and FENs are unchanged.

The changed Fast observations are `initial-SwappedManaRows`,
`initial-OffsetArcManaRows`, `initial-CenterSpokeManaRows`,
`initial-SplitFlankManaRows`, and `retained-release`. The changed Normal
observations are `initial-SwappedManaRows`, `initial-CenterSpokeManaRows`, and
`retained-release`.

`decisions.jsonl` is the authoritative executable contract. `manifest.json`
pins its byte size, SHA-256, record order, and counts. Do not regenerate,
normalize, reorder, or edit this v6 payload; add a new version directory for a
different selection.

Earlier payloads stay on disk unchanged as the record of previous selectors.
