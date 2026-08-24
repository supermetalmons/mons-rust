# Automove decisions v15

This immutable corpus pins deterministic public `suggestMove` behavior with
`performance.now()` fixed at `0` after correcting two packed-search review
findings.

Selectively pruned fail-low nodes now retain only move ordering instead of an
unproven upper score bound. Two-point mana trips use their scaled value even
when the two-point item is also the shortest or only trip.

The corpus retains the thirteen v14 source states. Compared with v14, 0 Fast,
5 Normal, and 1 Pro observations differ. Normal changes on
`initial-InnerWedgeManaRows`, `initial-OuterWedgeManaRows`,
`initial-BentCenterManaRows`, `initial-ForwardBridgeManaRows`, and
`retained-release`; Pro changes on `initial-AlternatingManaRows`.

`decisions.jsonl` is authoritative. Do not edit this v15 payload; add a new
version directory for a different selection.
