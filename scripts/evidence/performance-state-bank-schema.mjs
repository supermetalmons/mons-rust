export const performanceStateBankColors = Object.freeze(["white", "black"]);
export const performanceStatesPerColor = 32;
export const performanceStateBankAlgorithm = Object.freeze({
  id: "complete-games-v1-midpoint-balanced-v1",
  midpointTurnIndex: "floor(turns.length / 2)",
  tooShort: "midpointTurnIndex === 0 || midpointTurnIndex >= turns.length",
  terminal: "winner is defined after replaying turns before the midpoint",
  candidateSha256: "SHA-256 of the UTF-8 public FEN",
  variantSeed: "lowest candidate SHA-256 per variant, with source line as tie-break",
  colorFill: "variant seeds followed by candidates in SHA-256 and source-line order",
  outputOrder: "candidate SHA-256, then source line",
  statesPerColor: performanceStatesPerColor,
});
export const performanceStateBankSource = Object.freeze({
  path: "test-data/complete-games/v1/complete-games.jsonl",
  bytes: 2_273_026,
  sha256: "5bc194f15516a9c275807415910c95b2e62ce63df9e575ac93e1dd93013197eb",
  records: 1_527,
});
