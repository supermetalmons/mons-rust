import { describe, expect, it } from "vitest";

import {
  MAX_HEURISTIC_SCORE,
  MAX_SCORE,
  MIN_HEURISTIC_SCORE,
  MIN_SCORE,
  TERMINAL_SEARCH_SCORE,
  clampHeuristicScore,
  clampScore,
  saturatingScoreAdd,
  saturatingScoreMultiply,
  saturatingScoreSubtract,
} from "../../src/automove/score-math.js";

describe("automove score math", () => {
  it("clamps and truncates calculated scores", () => {
    expect(clampScore(Number.NaN)).toBe(0);
    expect(clampScore(12.9)).toBe(12);
    expect(clampScore(MAX_SCORE + 1)).toBe(MAX_SCORE);
    expect(clampScore(MIN_SCORE - 1)).toBe(MIN_SCORE);
  });

  it("saturates score arithmetic at the search bounds", () => {
    expect(saturatingScoreAdd(MAX_SCORE, 1)).toBe(MAX_SCORE);
    expect(saturatingScoreSubtract(MIN_SCORE, 1)).toBe(MIN_SCORE);
    expect(saturatingScoreMultiply(MAX_SCORE, 2)).toBe(MAX_SCORE);
    expect(saturatingScoreMultiply(MIN_SCORE, 2)).toBe(MIN_SCORE);
  });

  it("keeps nonterminal evaluations strictly inside terminal scores", () => {
    expect(clampHeuristicScore(Number.NaN)).toBe(0);
    expect(clampHeuristicScore(12.9)).toBe(12);
    expect(clampHeuristicScore(Number.POSITIVE_INFINITY)).toBe(
      MAX_HEURISTIC_SCORE,
    );
    expect(clampHeuristicScore(Number.NEGATIVE_INFINITY)).toBe(
      MIN_HEURISTIC_SCORE,
    );
    expect(MAX_HEURISTIC_SCORE).toBeLessThan(TERMINAL_SEARCH_SCORE);
    expect(MIN_HEURISTIC_SCORE).toBeGreaterThan(-TERMINAL_SEARCH_SCORE);
  });
});
