import { MAX_MOVES } from "./moves.js";
import { WIN_VALUE } from "./evaluation.js";

export const MAX_PLY = 48;
export const MAX_SEARCH_DEPTH = MAX_PLY - 2;
export const MAX_NONTERMINAL_SCORE = WIN_VALUE - MAX_PLY - 1;

export type SearchTuning = {
  readonly lateMoveReduction: boolean;
  readonly lateMoveIndex: number;
  readonly lateMoveDeepIndex: number;
  readonly moveCountPruning: boolean;
  readonly moveCountDepth: number;
  readonly moveCountBase: number;
  readonly moveCountFactor: number;
  readonly futilityMargin: number;
  readonly aspirationDelta: number;
  readonly aspirationMinDepth: number;
  readonly winsNextTurnThreat: number;
};

export type NormalizedSearchTuning = SearchTuning;

export const DEFAULT_TUNING: NormalizedSearchTuning = Object.freeze({
  lateMoveReduction: true,
  lateMoveIndex: 3,
  lateMoveDeepIndex: 8,
  moveCountPruning: true,
  moveCountDepth: 3,
  moveCountBase: 4,
  moveCountFactor: 5,
  futilityMargin: 900,
  aspirationDelta: 0,
  aspirationMinDepth: 3,
  winsNextTurnThreat: 0,
});

export type SearchLimits = {
  readonly maxDepth: number;
  readonly maxNodes: number;
  readonly tuning?: SearchTuning;
};

export type NormalizedSearchLimits = {
  readonly maxDepth: number;
  readonly maxNodes: number;
  readonly tuning: NormalizedSearchTuning;
};

const NORMALIZED_LIMITS_MEMO = new WeakMap<object, NormalizedSearchLimits>();

export function memoizedSearchLimits(limits: SearchLimits): NormalizedSearchLimits {
  const cacheable =
    Object.isFrozen(limits) &&
    (limits.tuning === undefined || Object.isFrozen(limits.tuning));
  if (!cacheable) return normalizeSearchLimits(limits);
  let normalized = NORMALIZED_LIMITS_MEMO.get(limits);
  if (normalized === undefined) {
    normalized = normalizeSearchLimits(limits);
    NORMALIZED_LIMITS_MEMO.set(limits, normalized);
  }
  return normalized;
}

export function normalizeSearchLimits(limits: unknown): NormalizedSearchLimits {
  if (typeof limits !== "object" || limits === null || Array.isArray(limits)) {
    throw new TypeError("fast search limits must be an object");
  }
  const values = limits as Readonly<Record<string, unknown>>;
  const maxDepth = values["maxDepth"];
  if (
    !Number.isSafeInteger(maxDepth) ||
    (maxDepth as number) < 0 ||
    (maxDepth as number) > MAX_SEARCH_DEPTH
  ) {
    throw new RangeError(
      `maxDepth must be a safe integer from 0 through ${MAX_SEARCH_DEPTH}`,
    );
  }
  const maxNodes = values["maxNodes"];
  if (!Number.isSafeInteger(maxNodes) || (maxNodes as number) < 0) {
    throw new RangeError("maxNodes must be a nonnegative safe integer");
  }
  const tuning = values["tuning"];
  return Object.freeze({
    maxDepth: maxDepth as number,
    maxNodes: maxNodes as number,
    tuning: tuning === undefined ? DEFAULT_TUNING : normalizeSearchTuning(tuning),
  });
}

function normalizeSearchTuning(tuning: unknown): NormalizedSearchTuning {
  if (typeof tuning !== "object" || tuning === null || Array.isArray(tuning)) {
    throw new TypeError("fast search tuning must be an object");
  }
  const values = tuning as Readonly<Record<string, unknown>>;
  return Object.freeze({
    lateMoveReduction: normalizedTuningBoolean(values, "lateMoveReduction"),
    lateMoveIndex: normalizedTuningInteger(values, "lateMoveIndex", MAX_MOVES),
    lateMoveDeepIndex: normalizedTuningInteger(values, "lateMoveDeepIndex", MAX_MOVES),
    moveCountPruning: normalizedTuningBoolean(values, "moveCountPruning"),
    moveCountDepth: normalizedTuningInteger(values, "moveCountDepth", MAX_SEARCH_DEPTH),
    moveCountBase: normalizedTuningInteger(values, "moveCountBase", MAX_MOVES),
    moveCountFactor: normalizedTuningInteger(values, "moveCountFactor", MAX_MOVES),
    futilityMargin: normalizedTuningInteger(
      values,
      "futilityMargin",
      MAX_NONTERMINAL_SCORE,
    ),
    aspirationDelta: normalizedTuningInteger(
      values,
      "aspirationDelta",
      MAX_NONTERMINAL_SCORE,
    ),
    aspirationMinDepth: normalizedTuningInteger(
      values,
      "aspirationMinDepth",
      MAX_SEARCH_DEPTH,
    ),
    winsNextTurnThreat: normalizedTuningInteger(
      values,
      "winsNextTurnThreat",
      MAX_NONTERMINAL_SCORE,
    ),
  });
}

function normalizedTuningBoolean(
  tuning: Readonly<Record<string, unknown>>,
  key: keyof SearchTuning,
): boolean {
  const value = tuning[key];
  if (typeof value !== "boolean") {
    throw new TypeError(`tuning.${key} must be a boolean`);
  }
  return value;
}

function normalizedTuningInteger(
  tuning: Readonly<Record<string, unknown>>,
  key: keyof SearchTuning,
  maximum: number,
): number {
  const value = tuning[key];
  if (typeof value !== "number") {
    throw new TypeError(`tuning.${key} must be a number`);
  }
  if (!Number.isSafeInteger(value) || value < 0 || value > maximum) {
    throw new RangeError(
      `tuning.${key} must be a safe integer from 0 through ${maximum}`,
    );
  }
  return value;
}
