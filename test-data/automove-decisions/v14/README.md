# Automove decisions v14

This immutable corpus pins deterministic public `suggestMove` behavior with
`performance.now()` fixed at `0` after correcting packed-search transposition
bounds and identity.

Selectively pruned fail-low nodes now store the assumed alpha bound rather than
the best searched score. Positions whose legal subtree is reduced by commuting
mon-move canonicalization include the preceding move in their transposition
identity.

The corpus retains the thirteen v13 source states. Compared with v13, 0 Fast,
1 Normal, and 2 Pro observations differ: `initial-OuterWedgeManaRows` at Normal,
plus `initial-SplitFlankManaRows` and `retained-release` at Pro.

`decisions.jsonl` is authoritative. Do not edit this v14 payload; add a new
version directory for a different selection.
