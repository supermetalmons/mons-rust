import type { Input } from "../../engine/model/domain.js";
import { locationIndex, type Location } from "../../engine/board/geometry.js";
import { compareInputChains } from "../transitions/order.js";
import {
  TURN_PLAN_FAMILY_PRIORITY_ORDER,
  TurnPlanFamily,
  type TurnAction,
  type TurnPlan,
  type TurnUtility,
} from "./model.js";

export function utilityHasNonnegativeDenyGain(utility: TurnUtility): boolean {
  return utility.denyGain >= 0;
}

export function utilitySupportsTemporaryRiskRecovery(utility: TurnUtility): boolean {
  return utility.drainerSafety > 0 || utility.avoidImmediateLoss > 0;
}

export function utilityStrictlyDominatesOverrideAxes(
  candidate: TurnUtility,
  incumbent: TurnUtility,
): boolean {
  const notWorse =
    candidate.winState >= incumbent.winState &&
    candidate.avoidImmediateLoss >= incumbent.avoidImmediateLoss &&
    candidate.scoreDelta >= incumbent.scoreDelta &&
    candidate.denyGain >= incumbent.denyGain &&
    candidate.drainerAttack >= incumbent.drainerAttack &&
    candidate.drainerSafety >= incumbent.drainerSafety;
  const strictlyBetter =
    candidate.winState > incumbent.winState ||
    candidate.avoidImmediateLoss > incumbent.avoidImmediateLoss ||
    candidate.scoreDelta > incumbent.scoreDelta ||
    candidate.denyGain > incumbent.denyGain ||
    candidate.drainerAttack > incumbent.drainerAttack ||
    candidate.drainerSafety > incumbent.drainerSafety;
  return notWorse && strictlyBetter;
}

export function utilityPassesOverrideGuard(
  candidate: TurnUtility,
  incumbent: TurnUtility,
): boolean {
  if (!utilityStrictlyDominatesOverrideAxes(candidate, incumbent)) return false;
  const strategicAxisGain =
    candidate.winState > incumbent.winState ||
    candidate.avoidImmediateLoss > incumbent.avoidImmediateLoss ||
    candidate.denyGain > incumbent.denyGain ||
    candidate.drainerAttack > incumbent.drainerAttack ||
    candidate.drainerSafety > incumbent.drainerSafety;
  const scoreDeltaForce = candidate.scoreDelta >= incumbent.scoreDelta + 220;
  return (
    candidate.evalScore + 192 >= incumbent.evalScore ||
    strategicAxisGain ||
    scoreDeltaForce
  );
}

export function utilitySupportsFamilyFallback(
  candidate: TurnUtility,
  incumbent: TurnUtility,
): boolean {
  return (
    compareTurnUtilities(candidate, incumbent) >= 0 &&
    candidate.evalScore + 192 >= incumbent.evalScore
  );
}

export function utilityImprovesNonScoreOverrideAxes(
  candidate: TurnUtility,
  incumbent: TurnUtility,
): boolean {
  return (
    candidate.winState > incumbent.winState ||
    candidate.avoidImmediateLoss > incumbent.avoidImmediateLoss ||
    candidate.denyGain > incumbent.denyGain ||
    candidate.drainerAttack > incumbent.drainerAttack ||
    candidate.drainerSafety > incumbent.drainerSafety
  );
}

export function utilityHasScoreDeltaForce(
  candidate: TurnUtility,
  incumbent: TurnUtility,
  minGain: number,
): boolean {
  return candidate.scoreDelta >= incumbent.scoreDelta + minGain;
}

export function utilitySupportsPrimaryAxesEvalTolerance(
  candidate: TurnUtility,
  incumbent: TurnUtility,
  evalDropMax: number,
): boolean {
  return (
    compareUtilityPrimaryAxes(candidate, incumbent) >= 0 &&
    candidate.evalScore + evalDropMax >= incumbent.evalScore
  );
}

export function compareNumber(left: number, right: number): number {
  return left < right ? -1 : left > right ? 1 : 0;
}

export function compareTuples(
  left: readonly number[],
  right: readonly number[],
): number {
  const length = Math.min(left.length, right.length);
  for (let index = 0; index < length; index += 1) {
    const order = compareNumber(left[index] ?? 0, right[index] ?? 0);
    if (order !== 0) return order;
  }
  return compareNumber(left.length, right.length);
}

export function copyPlan(plan: TurnPlan): TurnPlan {
  return {
    actions: [...plan.actions],
    compiledChunks: plan.compiledChunks.map((chunk) => chunk.slice()),
    endGame: plan.endGame.fork(),
    utility: plan.utility,
    headUtility: plan.headUtility,
    headFamily: plan.headFamily,
    goalFamily: plan.goalFamily,
    packageMeta: plan.packageMeta,
  };
}

export function compareChunks(
  left: readonly (readonly Input[])[],
  right: readonly (readonly Input[])[],
): number {
  const lengthOrder = compareNumber(left.length, right.length);
  if (lengthOrder !== 0) return lengthOrder;
  for (let index = 0; index < left.length; index += 1) {
    const order = compareInputChains(left[index] ?? [], right[index] ?? []);
    if (order !== 0) return order;
  }
  return 0;
}

export function actionKeyTuple(
  action: TurnAction,
): readonly [number, Location, Location | undefined, Location | undefined] {
  switch (action.kind) {
    case "walk":
      return [0, action.actor, action.to, undefined];
    case "attack":
      return [1, action.actor, action.target, undefined];
    case "spirit-shift":
      return [2, action.actor, action.target, action.destination];
    case "bomb":
      return [3, action.actor, action.target, undefined];
    case "move-mana":
      return [4, action.from, action.to, undefined];
    case "score-carry":
      return [5, action.actor, action.step, undefined];
    case "safety-retreat":
      return [6, action.actor, action.to, undefined];
  }
}

export function actionKey(action: TurnAction): string {
  const [tag, first, second, third] = actionKeyTuple(action);
  return `${tag}:${locationIndex(first)}:${second === undefined ? -1 : locationIndex(second)}:${
    third === undefined ? -2 : locationIndex(third)
  }`;
}

function compareLocations(left: Location, right: Location): number {
  const rowOrder = compareNumber(left.i, right.i);
  return rowOrder !== 0 ? rowOrder : compareNumber(left.j, right.j);
}

function compareOptionalLocations(
  left: Location | undefined,
  right: Location | undefined,
): number {
  if (left === undefined) return right === undefined ? 0 : -1;
  return right === undefined ? 1 : compareLocations(left, right);
}

export function compareActionKeys(left: TurnAction, right: TurnAction): number {
  const leftKey = actionKeyTuple(left);
  const rightKey = actionKeyTuple(right);
  return (
    compareNumber(leftKey[0], rightKey[0]) ||
    compareLocations(leftKey[1], rightKey[1]) ||
    compareOptionalLocations(leftKey[2], rightKey[2]) ||
    compareOptionalLocations(leftKey[3], rightKey[3])
  );
}

export function compareTurnUtilities(left: TurnUtility, right: TurnUtility): number {
  return compareTuples(
    [
      left.winState,
      left.avoidImmediateLoss,
      left.scoreDelta,
      left.denyGain,
      left.drainerAttack,
      left.drainerSafety,
      left.evalScore,
    ],
    [
      right.winState,
      right.avoidImmediateLoss,
      right.scoreDelta,
      right.denyGain,
      right.drainerAttack,
      right.drainerSafety,
      right.evalScore,
    ],
  );
}

export function compareUtilityPrimaryAxes(
  left: TurnUtility,
  right: TurnUtility,
): number {
  return compareTuples(
    [
      left.winState,
      left.avoidImmediateLoss,
      left.scoreDelta,
      left.denyGain,
      left.drainerAttack,
      left.drainerSafety,
    ],
    [
      right.winState,
      right.avoidImmediateLoss,
      right.scoreDelta,
      right.denyGain,
      right.drainerAttack,
      right.drainerSafety,
    ],
  );
}

export function familyRank(family: TurnPlanFamily): number {
  return TURN_PLAN_FAMILY_PRIORITY_ORDER.indexOf(family);
}

function headOpeningRiskClass(utility: TurnUtility): number {
  if (utility.avoidImmediateLoss < 0) return 0;
  if (utility.drainerSafety < 0 || utility.scoreDelta < 0) return 1;
  return 2;
}

function shouldCompareHeadOpeningUtility(
  family: TurnPlanFamily,
  left: TurnUtility,
  right: TurnUtility,
): boolean {
  return (
    (family === TurnPlanFamily.SafeSupermanaProgress ||
      family === TurnPlanFamily.SafeOpponentManaProgress) &&
    headOpeningRiskClass(left) !== headOpeningRiskClass(right)
  );
}

function comparePlanRank(
  leftUtility: TurnUtility,
  leftHeadUtility: TurnUtility,
  leftHeadFamily: TurnPlanFamily,
  rightUtility: TurnUtility,
  rightHeadUtility: TurnUtility,
  rightHeadFamily: TurnPlanFamily,
): number {
  let order = compareUtilityPrimaryAxes(leftUtility, rightUtility);
  if (order !== 0) return order;
  if (
    leftHeadFamily === rightHeadFamily &&
    shouldCompareHeadOpeningUtility(leftHeadFamily, leftHeadUtility, rightHeadUtility)
  ) {
    order = compareUtilityPrimaryAxes(leftHeadUtility, rightHeadUtility);
    if (order !== 0) return order;
    order = compareNumber(leftHeadUtility.evalScore, rightHeadUtility.evalScore);
    if (order !== 0) return order;
  }
  return compareNumber(leftUtility.evalScore, rightUtility.evalScore);
}

function comparePackageMeta(
  left: TurnPlan["packageMeta"],
  right: TurnPlan["packageMeta"],
): number {
  return compareTuples(
    [
      Number(left.scoreGain > 0),
      left.scoreGain,
      Number(left.denyGain > 0),
      left.denyGain,
      Number(left.drainerSafetyDelta > 0),
      left.drainerSafetyDelta,
      Number(left.endsNonnegativeDrainerSafety),
      Number(!left.spiritOnlySetup),
      -left.opponentImmediateWindowAfter,
    ],
    [
      Number(right.scoreGain > 0),
      right.scoreGain,
      Number(right.denyGain > 0),
      right.denyGain,
      Number(right.drainerSafetyDelta > 0),
      right.drainerSafetyDelta,
      Number(right.endsNonnegativeDrainerSafety),
      Number(!right.spiritOnlySetup),
      -right.opponentImmediateWindowAfter,
    ],
  );
}

export function turnEngineComparePlans(left: TurnPlan, right: TurnPlan): number {
  let order = comparePlanRank(
    left.utility,
    left.headUtility,
    left.headFamily,
    right.utility,
    right.headUtility,
    right.headFamily,
  );
  if (order !== 0) return order;
  order = comparePackageMeta(left.packageMeta, right.packageMeta);
  if (order !== 0) return order;
  order = compareNumber(familyRank(right.goalFamily), familyRank(left.goalFamily));
  if (order !== 0) return order;
  order = compareNumber(familyRank(right.headFamily), familyRank(left.headFamily));
  if (order !== 0) return order;
  order = compareNumber(right.actions.length, left.actions.length);
  return order !== 0 ? order : compareChunks(left.compiledChunks, right.compiledChunks);
}
