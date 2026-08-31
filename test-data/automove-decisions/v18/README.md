# Automove decisions v18

This immutable corpus pins deterministic public `suggestMove` behavior with
`performance.now()` fixed at `0` after promoting stable top-two move
ordering and the verified Pro root challenger. It retains the exact thirteen
v17 source states.

Compared with v17, Fast changes: none. Normal changes:
none. Pro changes: none.

The production public bundle is byte-identical to the independently audited
candidate bundle pinned in `manifest.json`.

`decisions.jsonl` is authoritative. Do not edit this v18 payload; add a new
version directory for a different selection.
