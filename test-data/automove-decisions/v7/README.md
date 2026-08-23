# Automove decisions v7

This immutable corpus pins deterministic public `suggestMove` behavior with
`performance.now()` fixed at `0`, after public Fast and Normal retuned the
shared search knobs they pass to the packed searcher. Their audited profiles
still use 30,000 nodes for Fast and 150,000 nodes for Normal; the retune moves
late-move reduction to start at index 2 with the deeper step at index 6, and
widens move-count pruning to `7 + 7 * depth`. Unsupported positions retain the
canonical fallback. Pro carries no tuning object and keeps the v6 packed
selector and evaluation unchanged.

The corpus retains the same thirteen source states as v6: the initial state for
every game variant and one retained regression state. Compared with v6, one Fast
observation and one Normal observation differ. All thirteen Pro observations are
byte-for-byte equivalent as parsed decision objects, and all source metadata and
FENs are unchanged.

The changed Fast observation is `initial-OuterEdgeManaRows`. The changed Normal
observation is `initial-OuterEdgeManaRows`.

`decisions.jsonl` is the authoritative executable contract. `manifest.json`
pins its byte size, SHA-256, record order, and counts. Do not regenerate,
normalize, reorder, or edit this v7 payload; add a new version directory for a
different selection.

Earlier payloads stay on disk unchanged as the record of previous selectors.
