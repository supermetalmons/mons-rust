import { BOARD_SIZE } from "../../engine/config.js";
import type { AutomoveExecutionContext } from "../execution-context.js";
import {
  rootProgressStepsBetter as progressStepsBetter,
  rootScorePathStepsBetter as scorePathStepsBetter,
} from "../root-focus.js";
import { compareTacticalEvaluatedRoots as compareTacticalRoots } from "../root-selector.js";
import {
  MAX_SCORE,
  MIN_SCORE,
  saturatingScoreAdd,
  saturatingScoreSubtract,
} from "../score-math.js";
import type { EvaluatedRoot } from "../search.js";
import {
  hasProgressSurface,
  isPlainSpiritDevelopmentRoot,
  productionEnabled,
  rootIsUnsafe as isUnsafe,
  type AutomoveConfig,
} from "../selector-types.js";
import { SMART_ROOT_REPLY_RISK_WINNER_SPREAD_SKIP } from "./config.js";

export { compareTacticalRoots, progressStepsBetter, scorePathStepsBetter };

export function compareRankedReplyRiskEvaluations(
  evaluations: readonly EvaluatedRoot[],
  leftIndex: number,
  rightIndex: number,
): number {
  const left = evaluations[leftIndex];
  const right = evaluations[rightIndex];
  if (left === undefined || right === undefined) return leftIndex - rightIndex;
  return (
    right.score - left.score ||
    compareTacticalRoots(left, right) ||
    leftIndex - rightIndex
  );
}

export function rootProgressOrSetupBetter(
  candidate: EvaluatedRoot,
  incumbent: EvaluatedRoot,
): boolean {
  return (
    progressStepsBetter(
      candidate.safeSupermanaProgressSteps,
      incumbent.safeSupermanaProgressSteps,
    ) ||
    progressStepsBetter(
      candidate.safeOpponentManaProgressSteps,
      incumbent.safeOpponentManaProgressSteps,
    ) ||
    candidate.spiritSetupGain > incumbent.spiritSetupGain
  );
}

export function isTacticalPriorityRoot(root: EvaluatedRoot): boolean {
  return (
    root.classes.immediateScore ||
    root.classes.drainerAttack ||
    root.classes.drainerSafetyRecover
  );
}

export function rankedRootOrder(
  evaluations: readonly EvaluatedRoot[],
  candidateIndex: number,
  incumbentIndex: number,
): number {
  const candidate = evaluations[candidateIndex];
  const incumbent = evaluations[incumbentIndex];
  if (candidate === undefined || incumbent === undefined) {
    return incumbentIndex - candidateIndex;
  }
  if (candidate.score !== incumbent.score) {
    return candidate.score > incumbent.score ? 1 : -1;
  }
  const tactical = compareTacticalRoots(candidate, incumbent);
  if (tactical !== 0) return tactical < 0 ? 1 : -1;
  return candidateIndex === incumbentIndex
    ? 0
    : candidateIndex < incumbentIndex
      ? 1
      : -1;
}

export function sameNonTacticalProgressLane(
  candidate: EvaluatedRoot,
  anchor: EvaluatedRoot,
): boolean {
  const sameProgressSteps =
    candidate.safeSupermanaProgressSteps ===
      anchor.safeSupermanaProgressSteps &&
    candidate.safeOpponentManaProgressSteps ===
      anchor.safeOpponentManaProgressSteps;
  const meaningfulProgressRoute =
    sameProgressSteps &&
    (candidate.safeSupermanaProgressSteps < BOARD_SIZE + 4 ||
      candidate.safeOpponentManaProgressSteps < BOARD_SIZE + 4);
  return (
    meaningfulProgressRoute &&
    candidate.safeSupermanaPickupNow === anchor.safeSupermanaPickupNow &&
    candidate.safeOpponentManaPickupNow === anchor.safeOpponentManaPickupNow &&
    candidate.supermanaProgress === anchor.supermanaProgress &&
    candidate.opponentManaProgress === anchor.opponentManaProgress &&
    !candidate.classes.immediateScore &&
    !candidate.classes.drainerAttack &&
    !candidate.classes.drainerSafetyRecover &&
    !anchor.classes.immediateScore &&
    !anchor.classes.drainerAttack &&
    !anchor.classes.drainerSafetyRecover &&
    !candidate.spiritDevelopment &&
    !anchor.spiritDevelopment &&
    !candidate.spiritSameTurnScoreSetupNow &&
    !anchor.spiritSameTurnScoreSetupNow &&
    !candidate.spiritOwnManaSetupNow &&
    !anchor.spiritOwnManaSetupNow &&
    candidate.sameTurnScoreWindowValue === 0 &&
    anchor.sameTurnScoreWindowValue === 0 &&
    !candidate.manaHandoffToOpponent &&
    !anchor.manaHandoffToOpponent &&
    !candidate.hasRoundtrip &&
    !anchor.hasRoundtrip
  );
}

export function replyRiskGuardShortlistIndices(
  execution: AutomoveExecutionContext,
  evaluations: readonly EvaluatedRoot[],
  candidateIndices: readonly number[],
  config: AutomoveConfig,
): number[] {
  if (candidateIndices.length === 0 || execution.session.checkpoint())
    return [];
  let bestScore = MIN_SCORE;
  let worstScore = MAX_SCORE;
  let hasWinningCandidate = false;
  for (const index of candidateIndices) {
    const root = evaluations[index];
    if (root === undefined) continue;
    bestScore = Math.max(bestScore, root.score);
    worstScore = Math.min(worstScore, root.score);
    hasWinningCandidate ||= root.winsImmediately;
  }
  if (
    hasWinningCandidate &&
    saturatingScoreSubtract(bestScore, worstScore) >
      SMART_ROOT_REPLY_RISK_WINNER_SPREAD_SKIP
  ) {
    return [];
  }
  const scoreMargin = Math.max(0, config.replyRisk.scoreMargin);
  let shortlist = candidateIndices.filter((index) => {
    const root = evaluations[index];
    return (
      root !== undefined &&
      saturatingScoreAdd(root.score, scoreMargin) >= bestScore
    );
  });
  shortlist.sort((left, right) =>
    compareRankedReplyRiskEvaluations(evaluations, left, right),
  );
  if (shortlist.length === 0) return shortlist;
  const shortlistLimit = Math.max(1, config.replyRisk.shortlistLimit);
  if (shortlist.length > shortlistLimit) {
    if (productionEnabled(config)) {
      const retained = shortlist.slice(0, shortlistLimit);
      const retainedHasSpirit = retained.some((index) => {
        const root = evaluations[index];
        return (
          root !== undefined &&
          (root.spiritDevelopment ||
            root.spiritSameTurnScoreSetupNow ||
            root.spiritOwnManaSetupNow)
        );
      });
      const bestShortlistScore =
        evaluations[shortlist[0] ?? -1]?.score ?? bestScore;
      const extras: number[] = [];
      if (!retainedHasSpirit) {
        const spirit = shortlist.slice(shortlistLimit).find((index) => {
          const root = evaluations[index];
          return (
            root !== undefined &&
            saturatingScoreSubtract(bestShortlistScore, root.score) <= 64 &&
            (root.spiritDevelopment ||
              root.spiritSameTurnScoreSetupNow ||
              root.spiritOwnManaSetupNow)
          );
        });
        if (spirit !== undefined) extras.push(spirit);
      }
      const hasPlainSpiritAnchor = [...retained, ...extras].some((index) => {
        const root = evaluations[index];
        return root !== undefined && isPlainSpiritDevelopmentRoot(root);
      });
      if (hasPlainSpiritAnchor) {
        let siblingsAdded = 0;
        for (const index of shortlist.slice(shortlistLimit)) {
          const root = evaluations[index];
          if (
            root !== undefined &&
            isPlainSpiritDevelopmentRoot(root) &&
            saturatingScoreSubtract(bestShortlistScore, root.score) <= 64 &&
            !extras.includes(index)
          ) {
            extras.push(index);
            siblingsAdded += 1;
            if (siblingsAdded >= 2) break;
          }
        }
      }
      shortlist = [...retained, ...extras];
    } else {
      shortlist = shortlist.slice(0, shortlistLimit);
    }
  }

  const anchorIndex = shortlist[0];
  const anchor =
    anchorIndex === undefined ? undefined : evaluations[anchorIndex];
  if (
    productionEnabled(config) &&
    anchor !== undefined &&
    isUnsafe(anchor) &&
    hasProgressSurface(anchor)
  ) {
    const extension = candidateIndices
      .filter((index) => !shortlist.includes(index))
      .filter((index) => {
        const root = evaluations[index];
        return (
          root !== undefined &&
          !isUnsafe(root) &&
          sameNonTacticalProgressLane(root, anchor) &&
          saturatingScoreSubtract(anchor.score, root.score) <= 320
        );
      })
      .sort((left, right) =>
        compareRankedReplyRiskEvaluations(evaluations, left, right),
      )[0];
    if (extension !== undefined) shortlist.push(extension);
  }
  shortlist.sort((left, right) =>
    compareRankedReplyRiskEvaluations(evaluations, left, right),
  );
  return shortlist;
}

export function isSafePlainSpiritPair(
  candidate: EvaluatedRoot,
  incumbent: EvaluatedRoot,
  config: AutomoveConfig,
): boolean {
  return (
    productionEnabled(config) &&
    isPlainSpiritDevelopmentRoot(candidate) &&
    isPlainSpiritDevelopmentRoot(incumbent) &&
    !isUnsafe(candidate) &&
    !isUnsafe(incumbent) &&
    !candidate.manaHandoffToOpponent &&
    !incumbent.manaHandoffToOpponent &&
    !candidate.hasRoundtrip &&
    !incumbent.hasRoundtrip &&
    !candidate.winsImmediately &&
    !incumbent.winsImmediately &&
    !candidate.attacksOpponentDrainer &&
    !incumbent.attacksOpponentDrainer &&
    !candidate.scoresSupermanaThisTurn &&
    !incumbent.scoresSupermanaThisTurn &&
    !candidate.scoresOpponentManaThisTurn &&
    !incumbent.scoresOpponentManaThisTurn &&
    !candidate.safeSupermanaPickupNow &&
    !incumbent.safeSupermanaPickupNow &&
    !candidate.safeOpponentManaPickupNow &&
    !incumbent.safeOpponentManaPickupNow &&
    !candidate.supermanaProgress &&
    !incumbent.supermanaProgress &&
    !candidate.opponentManaProgress &&
    !incumbent.opponentManaProgress &&
    candidate.sameTurnScoreWindowValue === 0 &&
    incumbent.sameTurnScoreWindowValue === 0
  );
}

export function shortlistHasPair<Value>(
  values: readonly Value[],
  shortlist: readonly number[],
  predicate: (candidate: Value, incumbent: Value) => boolean,
): boolean {
  for (let left = 0; left < shortlist.length; left += 1) {
    const candidate = values[shortlist[left] ?? -1];
    if (candidate === undefined) continue;
    for (let right = left + 1; right < shortlist.length; right += 1) {
      const incumbent = values[shortlist[right] ?? -1];
      if (incumbent !== undefined && predicate(candidate, incumbent)) {
        return true;
      }
    }
  }
  return false;
}

export function safePlainSpiritCompetition(
  evaluations: readonly EvaluatedRoot[],
  shortlist: readonly number[],
  config: AutomoveConfig,
): boolean {
  if (!productionEnabled(config) || shortlist.length < 2) return false;
  return shortlistHasPair(evaluations, shortlist, (candidate, incumbent) =>
    isSafePlainSpiritPair(candidate, incumbent, config),
  );
}
