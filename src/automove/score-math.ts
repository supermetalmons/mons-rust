export const MIN_SCORE = -0x8000_0000;
export const MAX_SCORE = 0x7fff_ffff;
export const TERMINAL_SEARCH_SCORE = 0x0fff_ffff;

// Static evaluations leave ample room for ordering and policy bonuses while
// remaining strictly less important than a proven win or loss.
export const MAX_HEURISTIC_SCORE = Math.trunc(TERMINAL_SEARCH_SCORE / 4);
export const MIN_HEURISTIC_SCORE = -MAX_HEURISTIC_SCORE;

/** Keep a calculated search score within the range used by score sentinels. */
export function clampScore(value: number): number {
  if (Number.isNaN(value)) return 0;
  if (value <= MIN_SCORE) return MIN_SCORE;
  if (value >= MAX_SCORE) return MAX_SCORE;
  return Math.trunc(value);
}

/** Keep a nonterminal evaluation inside the dedicated heuristic band. */
export function clampHeuristicScore(value: number): number {
  if (Number.isNaN(value)) return 0;
  if (value <= MIN_HEURISTIC_SCORE) return MIN_HEURISTIC_SCORE;
  if (value >= MAX_HEURISTIC_SCORE) return MAX_HEURISTIC_SCORE;
  return Math.trunc(value);
}

export function saturatingScoreAdd(left: number, right: number): number {
  return clampScore(left + right);
}

export function saturatingScoreSubtract(left: number, right: number): number {
  return clampScore(left - right);
}

export function saturatingScoreMultiply(left: number, right: number): number {
  return clampScore(left * right);
}
