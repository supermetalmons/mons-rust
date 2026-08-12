import { saturatingScoreAdd } from "../core/score-math.js";
import {
  isPlainSpiritDevelopmentRoot,
  rootIsUnsafe as isUnsafe,
} from "../config/types.js";
import { rootProgressStepsBetter, rootScorePathStepsBetter } from "./focus.js";
import type { EvaluatedRoot } from "./types.js";
import type {
  ProductionComparisonPhase,
  ProductionRootComparisonContext,
  ProductionRootPolicy,
} from "./selector-model.js";

const SPIRIT_SCORE_CHALLENGE_MARGIN = 40;

function valueAt<Value>(values: readonly Value[], index: number): Value {
  const value = values[index];
  if (value === undefined) {
    throw new RangeError(`root selector index ${index} is out of bounds`);
  }
  return value;
}

export function compareTacticalEvaluatedRoots(
  candidate: EvaluatedRoot,
  incumbent: EvaluatedRoot,
): number {
  const preferBoolean = (
    candidateValue: boolean,
    incumbentValue: boolean,
    preferTrue: boolean,
  ): number | undefined => {
    if (candidateValue === incumbentValue) return undefined;
    return candidateValue === preferTrue ? -1 : 1;
  };
  let order = preferBoolean(candidate.winsImmediately, incumbent.winsImmediately, true);
  order ??= preferBoolean(
    candidate.attacksOpponentDrainer,
    incumbent.attacksOpponentDrainer,
    true,
  );
  order ??= preferBoolean(
    candidate.ownDrainerVulnerable,
    incumbent.ownDrainerVulnerable,
    false,
  );
  order ??= preferBoolean(
    candidate.classes.immediateScore,
    incumbent.classes.immediateScore,
    true,
  );
  order ??= preferBoolean(
    candidate.scoresSupermanaThisTurn,
    incumbent.scoresSupermanaThisTurn,
    true,
  );
  order ??= preferBoolean(
    candidate.scoresOpponentManaThisTurn,
    incumbent.scoresOpponentManaThisTurn,
    true,
  );
  order ??= preferBoolean(
    candidate.safeSupermanaPickupNow,
    incumbent.safeSupermanaPickupNow,
    true,
  );
  order ??= preferBoolean(
    candidate.safeOpponentManaPickupNow,
    incumbent.safeOpponentManaPickupNow,
    true,
  );
  if (order !== undefined) return order;
  if (candidate.sameTurnScoreWindowValue !== incumbent.sameTurnScoreWindowValue) {
    return candidate.sameTurnScoreWindowValue > incumbent.sameTurnScoreWindowValue
      ? -1
      : 1;
  }
  order = preferBoolean(
    candidate.spiritSameTurnScoreSetupNow,
    incumbent.spiritSameTurnScoreSetupNow,
    true,
  );
  order ??= preferBoolean(
    candidate.spiritOwnManaSetupNow,
    incumbent.spiritOwnManaSetupNow,
    true,
  );
  if (order !== undefined) return order;
  if (
    candidate.spiritOwnManaSetupNow &&
    incumbent.spiritOwnManaSetupNow &&
    candidate.supermanaProgress &&
    incumbent.supermanaProgress &&
    candidate.safeSupermanaProgressSteps !== incumbent.safeSupermanaProgressSteps
  ) {
    return rootProgressStepsBetter(
      candidate.safeSupermanaProgressSteps,
      incumbent.safeSupermanaProgressSteps,
    )
      ? -1
      : 1;
  }
  if (
    candidate.spiritOwnManaSetupNow &&
    incumbent.spiritOwnManaSetupNow &&
    candidate.opponentManaProgress &&
    incumbent.opponentManaProgress &&
    candidate.safeOpponentManaProgressSteps !== incumbent.safeOpponentManaProgressSteps
  ) {
    return rootProgressStepsBetter(
      candidate.safeOpponentManaProgressSteps,
      incumbent.safeOpponentManaProgressSteps,
    )
      ? -1
      : 1;
  }
  if (
    candidate.spiritOwnManaSetupNow &&
    incumbent.spiritOwnManaSetupNow &&
    candidate.scorePathBestSteps !== incumbent.scorePathBestSteps
  ) {
    return rootScorePathStepsBetter(
      candidate.scorePathBestSteps,
      incumbent.scorePathBestSteps,
    )
      ? -1
      : 1;
  }
  order = preferBoolean(candidate.supermanaProgress, incumbent.supermanaProgress, true);
  if (order !== undefined) return order;
  if (
    candidate.supermanaProgress &&
    incumbent.supermanaProgress &&
    candidate.safeSupermanaProgressSteps !== incumbent.safeSupermanaProgressSteps
  ) {
    return rootProgressStepsBetter(
      candidate.safeSupermanaProgressSteps,
      incumbent.safeSupermanaProgressSteps,
    )
      ? -1
      : 1;
  }
  order = preferBoolean(
    candidate.opponentManaProgress,
    incumbent.opponentManaProgress,
    true,
  );
  if (order !== undefined) return order;
  if (
    candidate.opponentManaProgress &&
    incumbent.opponentManaProgress &&
    candidate.safeOpponentManaProgressSteps !== incumbent.safeOpponentManaProgressSteps
  ) {
    return rootProgressStepsBetter(
      candidate.safeOpponentManaProgressSteps,
      incumbent.safeOpponentManaProgressSteps,
    )
      ? -1
      : 1;
  }
  order = preferBoolean(
    candidate.manaHandoffToOpponent,
    incumbent.manaHandoffToOpponent,
    false,
  );
  order ??= preferBoolean(candidate.hasRoundtrip, incumbent.hasRoundtrip, false);
  order ??= preferBoolean(
    candidate.spiritDevelopment,
    incumbent.spiritDevelopment,
    true,
  );
  if (order !== undefined) return order;
  if (candidate.policyPriority !== incumbent.policyPriority) {
    return candidate.policyPriority > incumbent.policyPriority ? -1 : 1;
  }
  if (candidate.efficiency !== incumbent.efficiency) {
    return candidate.efficiency > incumbent.efficiency ? -1 : 1;
  }
  return 0;
}

export function compareRankedEvaluatedRootIndices(
  roots: readonly EvaluatedRoot[],
  candidateIndex: number,
  incumbentIndex: number,
): number {
  const candidate = valueAt(roots, candidateIndex);
  const incumbent = valueAt(roots, incumbentIndex);
  if (candidate.score !== incumbent.score) {
    return candidate.score > incumbent.score ? -1 : 1;
  }
  return (
    compareTacticalEvaluatedRoots(candidate, incumbent) ||
    candidateIndex - incumbentIndex
  );
}

export function bestScoredRootIndex(
  roots: readonly EvaluatedRoot[],
  candidateIndices: readonly number[],
): number {
  let bestIndex = candidateIndices[0] ?? 0;
  for (const index of candidateIndices) {
    if (compareRankedEvaluatedRootIndices(roots, index, bestIndex) < 0) {
      bestIndex = index;
    }
  }
  return bestIndex;
}

/** Positive means candidate wins the challenge; negative means incumbent wins. */
export function spiritScoreChallengeOrder(
  candidate: EvaluatedRoot,
  incumbent: EvaluatedRoot,
): number | undefined {
  const candidatePlainSpirit = isPlainSpiritDevelopmentRoot(candidate);
  const incumbentPlainSpirit = isPlainSpiritDevelopmentRoot(incumbent);
  if (candidatePlainSpirit === incumbentPlainSpirit) return undefined;
  const challenger = candidatePlainSpirit ? incumbent : candidate;
  const spirit = candidatePlainSpirit ? candidate : incumbent;
  const candidateIsChallenger = !candidatePlainSpirit;
  if (
    isUnsafe(challenger) ||
    challenger.hasRoundtrip ||
    challenger.score <
      saturatingScoreAdd(spirit.score, SPIRIT_SCORE_CHALLENGE_MARGIN) ||
    challenger.sameTurnScoreWindowValue < spirit.sameTurnScoreWindowValue
  ) {
    return undefined;
  }
  return candidateIsChallenger ? 1 : -1;
}

export function compareProductionRules(
  phase: ProductionComparisonPhase,
  context: ProductionRootComparisonContext,
  policy: ProductionRootPolicy | undefined,
): number | undefined {
  for (const rule of policy?.comparisonRules ?? []) {
    if (rule.phase !== phase) continue;
    const result = rule.compare(context);
    if (result.kind === "compare") {
      return result.order === 0 ? 0 : result.order > 0 ? 1 : -1;
    }
  }
  return undefined;
}
