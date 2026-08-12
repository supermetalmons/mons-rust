import { compareInputChains } from "../transitions/order.js";
import {
  UNKNOWN_PROGRESS_STEPS,
  UNKNOWN_SCORE_PATH_STEPS,
  type RootCandidate,
} from "./types.js";

function compareBooleanPreferred(left: boolean, right: boolean): number {
  return left === right ? 0 : left ? -1 : 1;
}

function progressStepsOrder(left: number, right: number): number {
  const leftKnown = left < UNKNOWN_PROGRESS_STEPS;
  const rightKnown = right < UNKNOWN_PROGRESS_STEPS;
  if (leftKnown !== rightKnown) return leftKnown ? -1 : 1;
  return leftKnown ? left - right : 0;
}

function scorePathStepsOrder(left: number, right: number): number {
  const leftKnown = left < UNKNOWN_SCORE_PATH_STEPS;
  const rightKnown = right < UNKNOWN_SCORE_PATH_STEPS;
  if (leftKnown !== rightKnown) return leftKnown ? -1 : 1;
  return leftKnown ? left - right : 0;
}

function tacticalRootOrder(left: RootCandidate, right: RootCandidate): number {
  let order = compareBooleanPreferred(left.winsImmediately, right.winsImmediately);
  if (order !== 0) return order;
  order = compareBooleanPreferred(
    left.attacksOpponentDrainer,
    right.attacksOpponentDrainer,
  );
  if (order !== 0) return order;
  order = compareBooleanPreferred(
    !left.ownDrainerVulnerable,
    !right.ownDrainerVulnerable,
  );
  if (order !== 0) return order;
  order = compareBooleanPreferred(
    left.classes.immediateScore,
    right.classes.immediateScore,
  );
  if (order !== 0) return order;
  for (const pair of [
    [left.scoresSupermanaThisTurn, right.scoresSupermanaThisTurn],
    [left.scoresOpponentManaThisTurn, right.scoresOpponentManaThisTurn],
    [left.safeSupermanaPickupNow, right.safeSupermanaPickupNow],
    [left.safeOpponentManaPickupNow, right.safeOpponentManaPickupNow],
  ] as const) {
    order = compareBooleanPreferred(pair[0], pair[1]);
    if (order !== 0) return order;
  }
  if (left.sameTurnScoreWindowValue !== right.sameTurnScoreWindowValue) {
    return right.sameTurnScoreWindowValue - left.sameTurnScoreWindowValue;
  }
  order = compareBooleanPreferred(
    left.spiritSameTurnScoreSetupNow,
    right.spiritSameTurnScoreSetupNow,
  );
  if (order !== 0) return order;
  order = compareBooleanPreferred(
    left.spiritOwnManaSetupNow,
    right.spiritOwnManaSetupNow,
  );
  if (order !== 0) return order;
  if (
    left.spiritOwnManaSetupNow &&
    right.spiritOwnManaSetupNow &&
    left.supermanaProgress &&
    right.supermanaProgress
  ) {
    order = progressStepsOrder(
      left.safeSupermanaProgressSteps,
      right.safeSupermanaProgressSteps,
    );
    if (order !== 0) return order;
  }
  if (
    left.spiritOwnManaSetupNow &&
    right.spiritOwnManaSetupNow &&
    left.opponentManaProgress &&
    right.opponentManaProgress
  ) {
    order = progressStepsOrder(
      left.safeOpponentManaProgressSteps,
      right.safeOpponentManaProgressSteps,
    );
    if (order !== 0) return order;
  }
  if (left.spiritOwnManaSetupNow && right.spiritOwnManaSetupNow) {
    order = scorePathStepsOrder(left.scorePathBestSteps, right.scorePathBestSteps);
    if (order !== 0) return order;
  }
  order = compareBooleanPreferred(left.supermanaProgress, right.supermanaProgress);
  if (order !== 0) return order;
  if (left.supermanaProgress && right.supermanaProgress) {
    order = progressStepsOrder(
      left.safeSupermanaProgressSteps,
      right.safeSupermanaProgressSteps,
    );
    if (order !== 0) return order;
  }
  order = compareBooleanPreferred(
    left.opponentManaProgress,
    right.opponentManaProgress,
  );
  if (order !== 0) return order;
  if (left.opponentManaProgress && right.opponentManaProgress) {
    order = progressStepsOrder(
      left.safeOpponentManaProgressSteps,
      right.safeOpponentManaProgressSteps,
    );
    if (order !== 0) return order;
  }
  order = compareBooleanPreferred(
    !left.manaHandoffToOpponent,
    !right.manaHandoffToOpponent,
  );
  if (order !== 0) return order;
  order = compareBooleanPreferred(!left.hasRoundtrip, !right.hasRoundtrip);
  if (order !== 0) return order;
  order = compareBooleanPreferred(left.spiritDevelopment, right.spiritDevelopment);
  if (order !== 0) return order;
  if (left.efficiency !== right.efficiency) {
    return right.efficiency - left.efficiency;
  }
  return right.heuristic - left.heuristic;
}

export function compareRootCandidates(
  left: RootCandidate,
  right: RootCandidate,
): number {
  if (left.heuristic !== right.heuristic) {
    return right.heuristic - left.heuristic;
  }
  const tactical = tacticalRootOrder(left, right);
  return tactical !== 0 ? tactical : compareInputChains(left.inputs, right.inputs);
}

function hasPriorityClass(candidate: RootCandidate, classIndex: number): boolean {
  switch (classIndex) {
    case 0:
      return candidate.classes.immediateScore;
    case 1:
      return candidate.classes.drainerAttack;
    default:
      return candidate.classes.drainerSafetyRecover;
  }
}

export function truncateWithClassCoverage(
  candidates: readonly RootCandidate[],
  limit: number,
): RootCandidate[] {
  if (candidates.length <= limit) return [...candidates];
  if (limit <= 0) return [];
  const selected = new Set<number>();
  const priorityIndices: number[] = [];
  const markPriority = (index: number): void => {
    selected.add(index);
    if (!priorityIndices.includes(index)) priorityIndices.push(index);
  };
  for (let classIndex = 0; classIndex < 3; classIndex += 1) {
    const index = candidates.findIndex((candidate) =>
      hasPriorityClass(candidate, classIndex),
    );
    if (index >= 0) markPriority(index);
  }
  let scoreWindowIndex = -1;
  for (let index = 0; index < candidates.length; index += 1) {
    const candidate = candidates[index];
    if (candidate === undefined || candidate.sameTurnScoreWindowValue <= 0) {
      continue;
    }
    const incumbent = candidates[scoreWindowIndex];
    if (incumbent === undefined || tacticalRootOrder(candidate, incumbent) < 0) {
      scoreWindowIndex = index;
    }
  }
  if (scoreWindowIndex >= 0) markPriority(scoreWindowIndex);
  for (let index = 0; index < candidates.length; index += 1) {
    const candidate = candidates[index];
    if (candidate === undefined) continue;
    const directHighValue =
      candidate.scoresSupermanaThisTurn ||
      candidate.scoresOpponentManaThisTurn ||
      candidate.safeSupermanaPickupNow ||
      candidate.safeOpponentManaPickupNow ||
      candidate.spiritSameTurnScoreSetupNow ||
      candidate.sameTurnScoreWindowValue > 0 ||
      candidate.spiritOwnManaSetupNow;
    const exactProgress =
      (candidate.supermanaProgress &&
        candidate.safeSupermanaProgressSteps < UNKNOWN_PROGRESS_STEPS) ||
      (candidate.opponentManaProgress &&
        candidate.safeOpponentManaProgressSteps < UNKNOWN_PROGRESS_STEPS);
    if (directHighValue || exactProgress) selected.add(index);
  }
  const result: RootCandidate[] = [];
  const appended = new Set<number>();
  const append = (index: number): void => {
    if (result.length >= limit || appended.has(index)) return;
    const candidate = candidates[index];
    if (candidate !== undefined) {
      result.push(candidate);
      appended.add(index);
    }
  };
  for (const index of priorityIndices) append(index);
  for (let index = 0; index < candidates.length; index += 1) {
    if (selected.has(index)) append(index);
  }
  for (let index = 0; index < candidates.length; index += 1) {
    append(index);
  }
  return result;
}
