# Automove decisions v16

This immutable corpus pins deterministic public `suggestMove` behavior with
`performance.now()` fixed at `0` after simplifying two packed-search fixes.

Every selective subtree now retains only transposition move ordering; no score
bound is reused without its direction being known. One- and two-point mana
trips are tracked as separate candidates, so the two-point scale works across
its full validated range.

The corpus retains the thirteen v15 source states. Compared with v15, 0 Fast,
5 Normal, and 1 Pro observations differ. Normal changes on
`initial-InnerWedgeManaRows`, `initial-OuterWedgeManaRows`,
`initial-BentCenterManaRows`, `initial-ForwardBridgeManaRows`, and
`retained-release`; Pro changes on `initial-AlternatingManaRows`.

`decisions.jsonl` is authoritative. Do not edit this v16 payload; add a new
version directory for a different selection.
