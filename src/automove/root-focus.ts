import { BOARD_SIZE, MONS_MOVES_PER_TURN } from "../engine/config.js";
import { inputChainsEqual, type Color, type Input } from "../engine/domain.js";
import {
  MIN_SCORE,
  saturatingScoreAdd,
  saturatingScoreSubtract,
} from "./score-math.js";
import { patchAutomoveConfig } from "./selector-config.js";
import {
  AUTOMOVE_TURN_ENGINE_MODE,
  type AutomoveConfig,
} from "./selector-types.js";

export const ROOT_FOCUS_CONSTANTS = Object.freeze({
  scoutDepth: 2,
  scoutMinNodes: 96,
  focusCount: 3,
  focusBudgetShareBp: 7_000,
  focusScoreMargin: 2_000,
  narrowSpreadFallback: 700,
  volatilityMargin: 600,
  volatilityKeep: 2,
});

export type RootFocusMoveClassFlags = {
  readonly immediateScore: boolean;
  readonly drainerAttack: boolean;
  readonly drainerSafetyRecover: boolean;
  readonly carrierProgress: boolean;
  readonly material: boolean;
  readonly quiet: boolean;
};

/** Structural subset of `RootCandidate` used by root focusing. */
export type RootFocusCandidate = {
  readonly inputs: readonly Input[];
  readonly game: { readonly activeColor: Color };
  readonly heuristic: number;
  readonly efficiency: number;
  readonly winsImmediately: boolean;
  readonly attacksOpponentDrainer: boolean;
  readonly ownDrainerVulnerable: boolean;
  readonly ownDrainerWalkVulnerable: boolean;
  readonly spiritDevelopment: boolean;
  readonly manaHandoffToOpponent: boolean;
  readonly hasRoundtrip: boolean;
  readonly scoresSupermanaThisTurn: boolean;
  readonly scoresOpponentManaThisTurn: boolean;
  readonly safeSupermanaPickupNow: boolean;
  readonly safeOpponentManaPickupNow: boolean;
  readonly safeSupermanaProgressSteps: number;
  readonly safeOpponentManaProgressSteps: number;
  readonly scorePathBestSteps: number;
  readonly sameTurnScoreWindowValue: number;
  readonly spiritSameTurnScoreSetupNow: boolean;
  readonly spiritOwnManaSetupNow: boolean;
  readonly supermanaProgress: boolean;
  readonly opponentManaProgress: boolean;
  readonly classes: RootFocusMoveClassFlags;
};

export type RootFocusConfig = AutomoveConfig;

export type RootFocusScoutContext<Candidate extends RootFocusCandidate> = {
  readonly candidate: Candidate;
  readonly candidateIndex: number;
  readonly perspective: Color;
  readonly depth: number;
  readonly alpha: number;
  readonly visitedNodes: number;
  readonly config: RootFocusConfig;
  readonly useTranspositionTable: boolean;
};

export type RootFocusScoutEvaluation = {
  readonly score: number;
  /** Absolute cumulative count, including the root node already charged. */
  readonly visitedNodes: number;
};

export type RootFocusOptions<Candidate extends RootFocusCandidate> = {
  readonly rootMoves: readonly Candidate[];
  readonly perspective: Color;
  readonly config: RootFocusConfig;
  readonly useTranspositionTable: boolean;
  readonly priorityInputs?: readonly (readonly Input[])[];
  readonly forcedInputs?: readonly Input[];
  readonly evaluateDeeperScout: (
    context: RootFocusScoutContext<Candidate>,
  ) => RootFocusScoutEvaluation;
  /** Mirrors the Production SpiritImpact plan plus nonnegative deny-gain gate. */
  readonly qualifiesPlainSpiritPlan?: (candidate: Candidate) => boolean;
  /** Mirrors the Production DrainerSafetyRecovery plan-family gate. */
  readonly qualifiesDrainerSafetyRecoveryPlan?: (
    candidate: Candidate,
  ) => boolean;
  readonly checkpoint?: () => boolean;
  readonly cancelled?: () => boolean;
};

export type FocusedRootCandidatesResult<Candidate> = {
  readonly candidates: readonly Candidate[];
  readonly scoutVisitedNodes: number;
};

function rootProgressStepSoftBonus(
  steps: number,
  perStepBonus: number,
): number {
  const unknownSteps = BOARD_SIZE + 4;
  if (steps >= unknownSteps || perStepBonus <= 0) return 0;
  const clampedSteps = Math.min(
    MONS_MOVES_PER_TURN,
    Math.max(0, Math.trunc(steps)),
  );
  return (MONS_MOVES_PER_TURN - clampedSteps) * Math.trunc(perStepBonus);
}

export function rootScoutProgressBonus(candidate: RootFocusCandidate): number {
  let bonus = 0;
  if (
    candidate.supermanaProgress &&
    !candidate.scoresSupermanaThisTurn &&
    !candidate.safeSupermanaPickupNow &&
    !candidate.spiritSameTurnScoreSetupNow &&
    !candidate.spiritOwnManaSetupNow
  ) {
    bonus = saturatingScoreAdd(
      saturatingScoreAdd(bonus, 520),
      rootProgressStepSoftBonus(candidate.safeSupermanaProgressSteps, 48),
    );
  }
  if (
    candidate.opponentManaProgress &&
    !candidate.scoresOpponentManaThisTurn &&
    !candidate.safeOpponentManaPickupNow &&
    !candidate.spiritSameTurnScoreSetupNow &&
    !candidate.spiritOwnManaSetupNow
  ) {
    bonus = saturatingScoreAdd(
      saturatingScoreAdd(bonus, 480),
      rootProgressStepSoftBonus(candidate.safeOpponentManaProgressSteps, 40),
    );
  }
  return bonus;
}

export function rootFocusScoutScore(candidate: RootFocusCandidate): number {
  return saturatingScoreAdd(
    saturatingScoreAdd(
      Math.trunc(candidate.heuristic),
      Math.trunc(Math.trunc(candidate.efficiency) / 2),
    ),
    rootScoutProgressBonus(candidate),
  );
}

export function rootVolatilityScore(candidate: RootFocusCandidate): number {
  let score = 0;
  if (candidate.winsImmediately) score = score + 5_000;
  if (candidate.attacksOpponentDrainer || candidate.classes.drainerAttack) {
    score = score + 2_800;
  }
  if (candidate.ownDrainerVulnerable) score = score + 2_200;
  if (candidate.classes.immediateScore) score = score + 1_700;
  if (candidate.classes.drainerSafetyRecover) {
    score = score + 1_500;
  }
  if (candidate.manaHandoffToOpponent) score = score + 900;
  if (candidate.hasRoundtrip) score = score + 700;
  if (candidate.classes.material) score = score + 240;
  if (candidate.efficiency < 0) {
    score = score + Math.min(0 - Math.trunc(candidate.efficiency), 400);
  }
  return score;
}

export function rootProgressStepsBetter(
  candidateSteps: number,
  incumbentSteps: number,
): boolean {
  const unknownSteps = BOARD_SIZE + 4;
  const candidateKnown = candidateSteps < unknownSteps;
  const incumbentKnown = incumbentSteps < unknownSteps;
  return candidateKnown
    ? !incumbentKnown || candidateSteps < incumbentSteps
    : false;
}

export function rootScorePathStepsBetter(
  candidateSteps: number,
  incumbentSteps: number,
): boolean {
  const unknownSteps = BOARD_SIZE * 3;
  const candidateKnown = candidateSteps < unknownSteps;
  const incumbentKnown = incumbentSteps < unknownSteps;
  return candidateKnown
    ? !incumbentKnown || candidateSteps < incumbentSteps
    : false;
}

function isBetterTacticalRootCandidate(
  candidate: RootFocusCandidate,
  incumbent: RootFocusCandidate,
  compareSpiritScorePath: boolean,
): boolean {
  if (candidate.winsImmediately !== incumbent.winsImmediately) {
    return candidate.winsImmediately;
  }
  if (candidate.attacksOpponentDrainer !== incumbent.attacksOpponentDrainer) {
    return candidate.attacksOpponentDrainer;
  }
  if (candidate.ownDrainerVulnerable !== incumbent.ownDrainerVulnerable) {
    return !candidate.ownDrainerVulnerable;
  }
  if (candidate.classes.immediateScore !== incumbent.classes.immediateScore) {
    return candidate.classes.immediateScore;
  }
  if (candidate.scoresSupermanaThisTurn !== incumbent.scoresSupermanaThisTurn) {
    return candidate.scoresSupermanaThisTurn;
  }
  if (
    candidate.scoresOpponentManaThisTurn !==
    incumbent.scoresOpponentManaThisTurn
  ) {
    return candidate.scoresOpponentManaThisTurn;
  }
  if (candidate.safeSupermanaPickupNow !== incumbent.safeSupermanaPickupNow) {
    return candidate.safeSupermanaPickupNow;
  }
  if (
    candidate.safeOpponentManaPickupNow !== incumbent.safeOpponentManaPickupNow
  ) {
    return candidate.safeOpponentManaPickupNow;
  }
  if (
    candidate.sameTurnScoreWindowValue !== incumbent.sameTurnScoreWindowValue
  ) {
    return (
      candidate.sameTurnScoreWindowValue > incumbent.sameTurnScoreWindowValue
    );
  }
  if (
    candidate.spiritSameTurnScoreSetupNow !==
    incumbent.spiritSameTurnScoreSetupNow
  ) {
    return candidate.spiritSameTurnScoreSetupNow;
  }
  if (candidate.spiritOwnManaSetupNow !== incumbent.spiritOwnManaSetupNow) {
    return candidate.spiritOwnManaSetupNow;
  }
  if (
    candidate.spiritOwnManaSetupNow &&
    incumbent.spiritOwnManaSetupNow &&
    candidate.supermanaProgress &&
    incumbent.supermanaProgress &&
    candidate.safeSupermanaProgressSteps !==
      incumbent.safeSupermanaProgressSteps
  ) {
    return rootProgressStepsBetter(
      candidate.safeSupermanaProgressSteps,
      incumbent.safeSupermanaProgressSteps,
    );
  }
  if (
    candidate.spiritOwnManaSetupNow &&
    incumbent.spiritOwnManaSetupNow &&
    candidate.opponentManaProgress &&
    incumbent.opponentManaProgress &&
    candidate.safeOpponentManaProgressSteps !==
      incumbent.safeOpponentManaProgressSteps
  ) {
    return rootProgressStepsBetter(
      candidate.safeOpponentManaProgressSteps,
      incumbent.safeOpponentManaProgressSteps,
    );
  }
  if (
    compareSpiritScorePath &&
    candidate.spiritOwnManaSetupNow &&
    incumbent.spiritOwnManaSetupNow &&
    candidate.scorePathBestSteps !== incumbent.scorePathBestSteps
  ) {
    return rootScorePathStepsBetter(
      candidate.scorePathBestSteps,
      incumbent.scorePathBestSteps,
    );
  }
  if (candidate.supermanaProgress !== incumbent.supermanaProgress) {
    return candidate.supermanaProgress;
  }
  if (
    candidate.supermanaProgress &&
    incumbent.supermanaProgress &&
    candidate.safeSupermanaProgressSteps !==
      incumbent.safeSupermanaProgressSteps
  ) {
    return rootProgressStepsBetter(
      candidate.safeSupermanaProgressSteps,
      incumbent.safeSupermanaProgressSteps,
    );
  }
  if (candidate.opponentManaProgress !== incumbent.opponentManaProgress) {
    return candidate.opponentManaProgress;
  }
  if (
    candidate.opponentManaProgress &&
    incumbent.opponentManaProgress &&
    candidate.safeOpponentManaProgressSteps !==
      incumbent.safeOpponentManaProgressSteps
  ) {
    return rootProgressStepsBetter(
      candidate.safeOpponentManaProgressSteps,
      incumbent.safeOpponentManaProgressSteps,
    );
  }
  if (candidate.manaHandoffToOpponent !== incumbent.manaHandoffToOpponent) {
    return !candidate.manaHandoffToOpponent;
  }
  if (candidate.hasRoundtrip !== incumbent.hasRoundtrip) {
    return !candidate.hasRoundtrip;
  }
  if (candidate.spiritDevelopment !== incumbent.spiritDevelopment) {
    return candidate.spiritDevelopment;
  }
  if (candidate.efficiency !== incumbent.efficiency) {
    return candidate.efficiency > incumbent.efficiency;
  }
  if (candidate.heuristic !== incumbent.heuristic) {
    return candidate.heuristic > incumbent.heuristic;
  }
  return false;
}

function compareTacticalRootCandidateOrder(
  candidate: RootFocusCandidate,
  incumbent: RootFocusCandidate,
  compareSpiritScorePath: boolean,
): number {
  if (
    isBetterTacticalRootCandidate(candidate, incumbent, compareSpiritScorePath)
  ) {
    return -1;
  }
  if (
    isBetterTacticalRootCandidate(incumbent, candidate, compareSpiritScorePath)
  ) {
    return 1;
  }
  return 0;
}

export function compareTacticalRootCandidates(
  candidate: RootFocusCandidate,
  incumbent: RootFocusCandidate,
): number {
  return compareTacticalRootCandidateOrder(candidate, incumbent, true);
}

export function compareTacticalRootCandidatesIgnoringScorePath(
  candidate: RootFocusCandidate,
  incumbent: RootFocusCandidate,
): number {
  return compareTacticalRootCandidateOrder(candidate, incumbent, false);
}

function compareScoresDescending(left: number, right: number): number {
  if (left === right) return 0;
  return left > right ? -1 : 1;
}

function valueAt<Value>(values: readonly Value[], index: number): Value {
  const value = values[index];
  if (value === undefined) {
    throw new RangeError(`root focus index ${index} is out of bounds`);
  }
  return value;
}

export function compareRankedRootIndices(
  rootMoves: readonly RootFocusCandidate[],
  left: readonly [number, number],
  right: readonly [number, number],
): number {
  return (
    compareScoresDescending(left[1], right[1]) ||
    compareTacticalRootCandidates(
      valueAt(rootMoves, left[0]),
      valueAt(rootMoves, right[0]),
    ) ||
    left[0] - right[0]
  );
}

export function prioritizeRootInputs<Candidate extends RootFocusCandidate>(
  rootMoves: readonly Candidate[],
  priorityInputs?: readonly (readonly Input[])[],
  forcedInputs?: readonly Input[],
): Candidate[] {
  const orderedInputs: (readonly Input[])[] = [];
  if (forcedInputs !== undefined) orderedInputs.push(forcedInputs);
  if (priorityInputs !== undefined) {
    for (const inputs of priorityInputs) {
      if (
        !orderedInputs.some((existing) => inputChainsEqual(existing, inputs))
      ) {
        orderedInputs.push(inputs);
      }
    }
  }
  const orderedRoots = [...rootMoves];
  let insertAt = 0;
  for (const inputs of orderedInputs) {
    const index = orderedRoots.findIndex((candidate) =>
      inputChainsEqual(candidate.inputs, inputs),
    );
    if (index < 0) continue;
    if (index === insertAt) {
      insertAt += 1;
      continue;
    }
    const [prioritized] = orderedRoots.splice(index, 1);
    if (prioritized !== undefined) {
      orderedRoots.splice(insertAt, 0, prioritized);
      insertAt += 1;
    }
  }
  return orderedRoots;
}

function nonnegativeInteger(value: number, fallback: number): number {
  return Number.isFinite(value) ? Math.max(0, Math.trunc(value)) : fallback;
}

function clampInteger(value: number, minimum: number, maximum: number): number {
  if (minimum > maximum) {
    throw new RangeError("root scout budget has invalid clamp bounds");
  }
  return Math.min(maximum, Math.max(minimum, Math.trunc(value)));
}

function isProductionMode(config: RootFocusConfig): boolean {
  return (
    config.planner.mode === AUTOMOVE_TURN_ENGINE_MODE.Production &&
    config.planner.enabled
  );
}

type RootScoutRun =
  | {
      readonly status: "finished";
      readonly scores: readonly (number | undefined)[];
      readonly visitedNodes: number;
    }
  | {
      readonly status: "cancelled";
      readonly visitedNodes: number;
    };

type RankedRootIndex = readonly [index: number, score: number];

function prioritizedRootResult<Candidate extends RootFocusCandidate>(
  rootMoves: readonly Candidate[],
  priorityInputs: readonly (readonly Input[])[] | undefined,
  forcedInputs: readonly Input[] | undefined,
): FocusedRootCandidatesResult<Candidate> {
  return {
    candidates: prioritizeRootInputs(rootMoves, priorityInputs, forcedInputs),
    scoutVisitedNodes: 0,
  };
}

function createRootScoutConfig(
  config: RootFocusConfig,
  rootCount: number,
): RootFocusConfig | undefined {
  const depth =
    config.search.volatilityFocus || config.budget.depth <= 3
      ? 1
      : clampInteger(config.budget.depth, 1, ROOT_FOCUS_CONSTANTS.scoutDepth);
  const scoutShareBp = clampInteger(
    10_000 - ROOT_FOCUS_CONSTANTS.focusBudgetShareBp,
    500,
    4_000,
  );
  let budget = rootCount;
  if (depth > 1) {
    const maximumBudget = Math.max(0, config.budget.maxVisitedNodes - 1);
    if (maximumBudget < ROOT_FOCUS_CONSTANTS.scoutMinNodes) return undefined;
    budget = clampInteger(
      Math.trunc((config.budget.maxVisitedNodes * scoutShareBp) / 10_000),
      ROOT_FOCUS_CONSTANTS.scoutMinNodes,
      maximumBudget,
    );
  }
  if (budget < rootCount) return undefined;
  return patchAutomoveConfig(config, {
    budget: { depth, maxVisitedNodes: budget },
    search: { selectiveExtensions: false, quietReductions: false },
  });
}

function runRootScout<Candidate extends RootFocusCandidate>(
  options: RootFocusOptions<Candidate>,
  rootMoves: readonly Candidate[],
  scoutConfig: RootFocusConfig,
): RootScoutRun {
  let visitedNodes = 0;
  let alpha = MIN_SCORE;
  const scores: (number | undefined)[] = Array.from(
    { length: rootMoves.length },
    () => undefined,
  );

  for (const [index, candidate] of rootMoves.entries()) {
    if (options.checkpoint?.() === true) {
      return { status: "cancelled", visitedNodes };
    }
    if (
      scoutConfig.budget.depth > 1 &&
      visitedNodes >= scoutConfig.budget.maxVisitedNodes
    ) {
      break;
    }
    let score: number;
    if (scoutConfig.budget.depth > 1) {
      visitedNodes += 1;
      const evaluation = options.evaluateDeeperScout({
        candidate,
        candidateIndex: index,
        perspective: options.perspective,
        depth: scoutConfig.budget.depth - 1,
        alpha,
        visitedNodes,
        config: scoutConfig,
        useTranspositionTable: options.useTranspositionTable,
      });
      score = Math.trunc(evaluation.score);
      visitedNodes = Math.min(
        scoutConfig.budget.maxVisitedNodes,
        Math.max(
          visitedNodes,
          nonnegativeInteger(evaluation.visitedNodes, visitedNodes),
        ),
      );
    } else {
      score = rootFocusScoutScore(candidate);
    }
    if (options.cancelled?.() === true) {
      return { status: "cancelled", visitedNodes };
    }
    scores[index] = score;
    alpha = Math.max(alpha, score);
  }

  return { status: "finished", scores, visitedNodes };
}

function effectiveScoutScore(
  candidate: RootFocusCandidate,
  scoutScore: number | undefined,
): number {
  return scoutScore ?? rootFocusScoutScore(candidate);
}

function rankedRootIndices(
  rootMoves: readonly RootFocusCandidate[],
  scoutScores: readonly (number | undefined)[],
): RankedRootIndex[] {
  const ranked: RankedRootIndex[] = rootMoves.map((candidate, index) => {
    return [index, effectiveScoutScore(candidate, scoutScores[index])];
  });
  ranked.sort((left, right) =>
    compareRankedRootIndices(rootMoves, left, right),
  );
  return ranked;
}

function hasNarrowScoutSpread(
  rankedIndices: readonly RankedRootIndex[],
  focusCount: number,
): boolean {
  if (rankedIndices.length < focusCount) return false;
  const bestScore = rankedIndices[0]?.[1] ?? MIN_SCORE;
  const kthScore = rankedIndices[focusCount - 1]?.[1] ?? MIN_SCORE;
  return (
    saturatingScoreSubtract(bestScore, kthScore) <=
    ROOT_FOCUS_CONSTANTS.narrowSpreadFallback
  );
}

function selectFocusedRootIndices<Candidate extends RootFocusCandidate>(
  options: RootFocusOptions<Candidate>,
  rootMoves: readonly Candidate[],
  rankedIndices: readonly RankedRootIndex[],
  scoutScores: readonly (number | undefined)[],
  effectiveBestScore: number,
  focusCount: number,
): boolean[] {
  const { config, forcedInputs, perspective, priorityInputs } = options;
  const selected = Array.from<boolean>({ length: rootMoves.length }).fill(
    false,
  );
  for (const [index] of rankedIndices.slice(0, focusCount)) {
    selected[index] = true;
  }

  if (isProductionMode(config) && focusCount <= 3) {
    const topFocusHasPlainSpirit = rankedIndices
      .slice(0, focusCount)
      .some(([index]) => {
        const root = valueAt(rootMoves, index);
        return (
          root.spiritDevelopment &&
          !root.spiritSameTurnScoreSetupNow &&
          !root.spiritOwnManaSetupNow
        );
      });
    if (!topFocusHasPlainSpirit) {
      const nearFocusPlainSpirit = rankedIndices
        .slice(0, focusCount + 3)
        .map(([index]) => index)
        .filter((index) => {
          const root = valueAt(rootMoves, index);
          return (
            root.spiritDevelopment &&
            !root.spiritSameTurnScoreSetupNow &&
            !root.spiritOwnManaSetupNow &&
            !root.ownDrainerVulnerable &&
            !root.manaHandoffToOpponent &&
            !root.hasRoundtrip &&
            root.game.activeColor === perspective &&
            options.qualifiesPlainSpiritPlan?.(root) === true
          );
        });
      if (nearFocusPlainSpirit.length >= 2) {
        for (const index of nearFocusPlainSpirit) selected[index] = true;
      }
    }
  }

  for (const [index, score] of rankedIndices) {
    if (score + ROOT_FOCUS_CONSTANTS.focusScoreMargin < effectiveBestScore) {
      continue;
    }
    selected[index] = true;
  }

  for (const [index, candidate] of rootMoves.entries()) {
    if (candidate.attacksOpponentDrainer) selected[index] = true;
    if (
      candidate.scoresSupermanaThisTurn ||
      candidate.scoresOpponentManaThisTurn ||
      candidate.safeSupermanaPickupNow ||
      candidate.safeOpponentManaPickupNow ||
      candidate.spiritSameTurnScoreSetupNow ||
      candidate.sameTurnScoreWindowValue > 0 ||
      candidate.spiritOwnManaSetupNow
    ) {
      selected[index] = true;
    }
    if (
      isProductionMode(config) &&
      candidate.ownDrainerVulnerable &&
      !candidate.ownDrainerWalkVulnerable &&
      !candidate.manaHandoffToOpponent &&
      !candidate.hasRoundtrip &&
      candidate.game.activeColor === perspective &&
      options.qualifiesDrainerSafetyRecoveryPlan?.(candidate) === true
    ) {
      selected[index] = true;
    }
  }

  if (priorityInputs !== undefined) {
    for (const inputs of priorityInputs) {
      const index = rootMoves.findIndex((candidate) =>
        inputChainsEqual(candidate.inputs, inputs),
      );
      if (index >= 0) selected[index] = true;
    }
  }
  if (forcedInputs !== undefined) {
    const index = rootMoves.findIndex((candidate) =>
      inputChainsEqual(candidate.inputs, forcedInputs),
    );
    if (index >= 0) selected[index] = true;
  }

  if (config.search.volatilityFocus) {
    const volatilityRanked = rootMoves
      .map((candidate, index) => {
        const scoutScore = scoutScores[index];
        return {
          index,
          volatility: rootVolatilityScore(candidate),
          scoutScore: effectiveScoutScore(candidate, scoutScore),
        };
      })
      .filter(({ volatility }) => volatility > 0)
      .sort(
        (left, right) =>
          compareScoresDescending(left.volatility, right.volatility) ||
          compareScoresDescending(left.scoutScore, right.scoutScore) ||
          left.index - right.index,
      );
    for (const { index } of volatilityRanked.slice(
      0,
      Math.max(ROOT_FOCUS_CONSTANTS.volatilityKeep, 1),
    )) {
      selected[index] = true;
    }
    const bestVolatility = volatilityRanked[0]?.volatility;
    if (bestVolatility !== undefined) {
      for (const { index, volatility, scoutScore } of volatilityRanked) {
        if (
          volatility + ROOT_FOCUS_CONSTANTS.volatilityMargin <
          bestVolatility
        ) {
          break;
        }
        if (
          scoutScore + ROOT_FOCUS_CONSTANTS.focusScoreMargin <
          effectiveBestScore
        ) {
          continue;
        }
        selected[index] = true;
      }
    }
  }

  return selected;
}

export function focusedRootCandidates<Candidate extends RootFocusCandidate>(
  options: RootFocusOptions<Candidate>,
): FocusedRootCandidatesResult<Candidate> {
  const { config, priorityInputs, forcedInputs } = options;
  const rootMoves = [...options.rootMoves];
  if (options.checkpoint?.() === true) {
    return { candidates: [], scoutVisitedNodes: 0 };
  }
  if (
    !config.search.twoPassRootAllocation ||
    rootMoves.length <= ROOT_FOCUS_CONSTANTS.focusCount ||
    config.budget.depth <= 1
  ) {
    return prioritizedRootResult(rootMoves, priorityInputs, forcedInputs);
  }

  const scoutConfig = createRootScoutConfig(config, rootMoves.length);
  if (scoutConfig === undefined) {
    return prioritizedRootResult(rootMoves, priorityInputs, forcedInputs);
  }
  const scout = runRootScout(options, rootMoves, scoutConfig);
  if (scout.status === "cancelled") {
    return { candidates: [], scoutVisitedNodes: scout.visitedNodes };
  }

  const focusCount = Math.min(
    ROOT_FOCUS_CONSTANTS.focusCount,
    rootMoves.length,
  );
  const rankedIndices = rankedRootIndices(rootMoves, scout.scores);
  if (hasNarrowScoutSpread(rankedIndices, focusCount)) {
    const rankedRoots = rankedIndices.map(([index]) =>
      valueAt(rootMoves, index),
    );
    return prioritizedRootResult(rankedRoots, priorityInputs, forcedInputs);
  }

  const selected = selectFocusedRootIndices(
    options,
    rootMoves,
    rankedIndices,
    scout.scores,
    rankedIndices[0]?.[1] ?? MIN_SCORE,
    focusCount,
  );

  if (!selected.some(Boolean)) {
    return prioritizedRootResult(rootMoves, priorityInputs, forcedInputs);
  }

  const focusedWithScores: RankedRootIndex[] = selected.flatMap(
    (isSelected, index) => {
      if (!isSelected) return [];
      const candidate = valueAt(rootMoves, index);
      return [[index, effectiveScoutScore(candidate, scout.scores[index])]];
    },
  );
  focusedWithScores.sort((left, right) =>
    compareRankedRootIndices(rootMoves, left, right),
  );
  const focused = focusedWithScores.map(([index]) => valueAt(rootMoves, index));
  return {
    candidates: prioritizeRootInputs(focused, priorityInputs, forcedInputs),
    scoutVisitedNodes: Math.min(
      scout.visitedNodes,
      config.budget.maxVisitedNodes,
    ),
  };
}
