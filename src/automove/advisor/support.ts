import {
  Color,
  inputChainsEqual,
  inputChainsShareFirstInput,
  inputEquals,
} from "../../engine/domain.js";
import type { Input } from "../../engine/domain.js";
import { MonsGame } from "../../engine/game.js";
import { exactOpportunityContext } from "../exact.js";
import type { AutomoveExecutionContext } from "../execution-context.js";
import {
  turnEngineRootPlanUtility,
  turnEngineSelectedOverrideUtility,
} from "../reply-risk.js";
import type { RootCandidate } from "../root-candidates.js";
import { rootFamily as advisorRootFamily } from "../root-family.js";
import {
  compareRankedRootIndices,
  compareTacticalRootCandidates,
} from "../root-focus.js";
import { compareRankedEvaluatedRootIndices } from "../root-selector.js";
import type { EvaluatedRoot } from "../search.js";
import { patchAutomoveConfig } from "../selector-config.js";
import { rootIsUnsafe as advisorRootIsUnsafe } from "../selector-types.js";
import type { AutomoveConfig } from "../selector-types.js";
import { compareInputChains } from "../transitions.js";
import {
  TurnPlanFamily,
  compareUtilityPrimaryAxes,
  utilitySupportsFamilyFallback,
} from "../turn-engine.js";
import type {
  ProductionRootAdvisorEntry,
  ProductionRootAdvisorReasonCode,
} from "./types.js";

function isTurnPlanFamilyOneOf(
  family: TurnPlanFamily,
  ...families: readonly TurnPlanFamily[]
): boolean {
  return families.includes(family);
}

function sameInputAt(
  left: readonly Input[],
  right: readonly Input[],
  index: number,
): boolean {
  const leftInput = left[index];
  const rightInput = right[index];
  return (
    leftInput !== undefined &&
    rightInput !== undefined &&
    inputEquals(leftInput, rightInput)
  );
}

function advisorRootIsSafe(root: RootCandidate): boolean {
  return !advisorRootIsUnsafe(root) && !root.hasRoundtrip;
}

function rootUtility(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  root: RootCandidate,
  perspective: Color,
  config: AutomoveConfig,
) {
  return turnEngineSelectedOverrideUtility(
    execution,
    game,
    root,
    perspective,
    config,
    advisorRootFamily(root),
  );
}

function rootMoveUtility(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  root: RootCandidate,
  perspective: Color,
  config: AutomoveConfig,
) {
  return turnEngineRootPlanUtility(
    execution,
    game,
    root,
    perspective,
    config,
    advisorRootFamily(root),
  );
}

function utilityCompetes(
  candidate: ReturnType<typeof rootUtility>,
  incumbent: ReturnType<typeof rootUtility>,
): boolean {
  return (
    compareUtilityPrimaryAxes(candidate, incumbent) >= 0 ||
    utilitySupportsFamilyFallback(candidate, incumbent)
  );
}

function utilitiesEqual(
  left: ReturnType<typeof rootUtility>,
  right: ReturnType<typeof rootUtility>,
): boolean {
  return (
    left.winState === right.winState &&
    left.avoidImmediateLoss === right.avoidImmediateLoss &&
    left.scoreDelta === right.scoreDelta &&
    left.denyGain === right.denyGain &&
    left.drainerAttack === right.drainerAttack &&
    left.drainerSafety === right.drainerSafety &&
    left.evalScore === right.evalScore
  );
}

function entry(
  root: RootCandidate,
  reason: ProductionRootAdvisorReasonCode,
): ProductionRootAdvisorEntry {
  return {
    inputs: [...root.inputs],
    family: advisorRootFamily(root),
    rootRank: root.rootRank,
    reason,
  };
}

function pushUnique(
  entries: ProductionRootAdvisorEntry[],
  value: ProductionRootAdvisorEntry,
): void {
  if (
    !entries.some((existing) => inputChainsEqual(existing.inputs, value.inputs))
  ) {
    entries.push(value);
  }
}

function compareRankedRootMoveIndices(
  roots: readonly RootCandidate[],
  left: number,
  right: number,
): number {
  return compareRankedRootIndices(roots, [left, 0], [right, 0]);
}

function compareRootMoveSearchPriority(
  left: RootCandidate,
  right: RootCandidate,
): number {
  if (left.heuristic !== right.heuristic) {
    return left.heuristic > right.heuristic ? -1 : 1;
  }
  return (
    compareTacticalRootCandidates(left, right) ||
    compareInputChains(left.inputs, right.inputs)
  );
}

function withPlannerMode(
  config: AutomoveConfig,
  mode: AutomoveConfig["planner"]["mode"],
): AutomoveConfig {
  return patchAutomoveConfig(config, { planner: { mode } });
}

function withoutReplyRiskGuard(
  config: AutomoveConfig,
  mode: AutomoveConfig["planner"]["mode"],
): AutomoveConfig {
  return patchAutomoveConfig(withPlannerMode(config, mode), {
    replyRisk: { enabled: false },
  });
}

function bestOverrideIndex(
  roots: readonly EvaluatedRoot[],
  indices: readonly number[],
  predicate: (root: EvaluatedRoot, index: number) => boolean,
  compare: (left: number, right: number) => number,
): number | undefined {
  return indices
    .filter((index) => {
      const root = roots[index];
      return root !== undefined && predicate(root, index);
    })
    .sort(compare)[0];
}

function compareRootRankThenRanked(
  roots: readonly EvaluatedRoot[],
  left: number,
  right: number,
): number {
  const leftRoot = roots[left];
  const rightRoot = roots[right];
  if (leftRoot === undefined || rightRoot === undefined) return left - right;
  if (leftRoot.rootRank !== rightRoot.rootRank) {
    return leftRoot.rootRank < rightRoot.rootRank ? -1 : 1;
  }
  return compareRankedEvaluatedRootIndices(roots, left, right);
}

function compareRootRankThenScoreThenRanked(
  roots: readonly EvaluatedRoot[],
  left: number,
  right: number,
): number {
  const leftRoot = roots[left];
  const rightRoot = roots[right];
  if (leftRoot === undefined || rightRoot === undefined) return left - right;
  if (leftRoot.rootRank !== rightRoot.rootRank) {
    return leftRoot.rootRank < rightRoot.rootRank ? -1 : 1;
  }
  if (leftRoot.score !== rightRoot.score) {
    return leftRoot.score > rightRoot.score ? -1 : 1;
  }
  return compareRankedEvaluatedRootIndices(roots, left, right);
}

function exactContextIsQuiet(
  execution: AutomoveExecutionContext,
  game: MonsGame,
): boolean {
  const exact = exactOpportunityContext(execution, game, game.activeColor);
  return (
    exact.delta.sameTurnScoreWindowValue === 0 &&
    exact.delta.opponentWindowDenyGain === 0 &&
    !exact.delta.drainerAttackAvailable
  );
}

export {
  advisorRootIsSafe,
  bestOverrideIndex,
  compareRankedRootMoveIndices,
  compareRootMoveSearchPriority,
  compareRootRankThenRanked,
  compareRootRankThenScoreThenRanked,
  entry,
  exactContextIsQuiet,
  isTurnPlanFamilyOneOf,
  pushUnique,
  rootMoveUtility,
  rootUtility,
  inputChainsShareFirstInput as sameFirstInput,
  sameInputAt,
  utilitiesEqual,
  utilityCompetes,
  withPlannerMode,
  withoutReplyRiskGuard,
};
