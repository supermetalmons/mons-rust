# Automove decisions v17

This immutable corpus pins deterministic public `suggestMove` behavior with
`performance.now()` fixed at `0` after integrating the frozen learned Pro
residual evaluator.

Fast and Normal keep their previous weight identities and exact v16 decisions.
Pro alone uses a distinct scalar-weight identity whose evaluation tables carry
the frozen 3,025-entry phase numerator table.

The corpus retains the thirteen v16 source states. Compared with v16, 0 Fast,
0 Normal, and 8 Pro observations differ. Pro changes on `initial-Classic`,
`initial-AlternatingManaRows`, `initial-InnerWedgeManaRows`,
`initial-OuterWedgeManaRows`, `initial-BentCenterManaRows`,
`initial-ForwardBridgeManaRows`, `initial-CornerChainManaRows`, and
`retained-release`.

`decisions.jsonl` is authoritative. Do not edit this v17 payload; add a new
version directory for a different selection.
