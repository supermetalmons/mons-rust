import type { MonsGame } from "../engine/game.js";
import { type Input, type Mana } from "../engine/domain.js";
import { locationIndex, type Location } from "../engine/geometry.js";
import {
  exactSearchStateHash,
  type ExactColorSummary,
  type ExactOpportunityContext,
} from "./exact.js";
import { hash64, type Hash64 } from "./hash64.js";
import type { ScoringWeights } from "./scoring.js";
import { compareInputChains } from "./transitions.js";

export const TURN_ENGINE_CACHE_MAX_ENTRIES = 4_096;
export const TURN_ENGINE_COMPILE_LIMIT_MAX = 256;
export const LOCAL_HASH_COLLECTION_CAPACITY = Number.MAX_SAFE_INTEGER;
export const FNV_OFFSET_BASIS = hash64(0x1465_0fb0, 0x739d_0383);
export const FNV_PRIME = hash64(0x0000_0100, 0x0000_01b3);
export const HASH64_ALL_ONES = hash64(0xffff_ffff, 0xffff_ffff);
export const HASH64_ALL_ONES_EXCEPT_LOW_BIT = hash64(0xffff_ffff, 0xffff_fffe);

export const TurnEngineMode = Object.freeze({
  Baseline: "baseline",
  Production: "production",
} as const);

export type TurnEngineMode =
  (typeof TurnEngineMode)[keyof typeof TurnEngineMode];

export const TURN_ENGINE_MODE_CACHE_TAG = Object.freeze({
  [TurnEngineMode.Baseline]: 0,
  [TurnEngineMode.Production]: 1,
} satisfies Readonly<Record<TurnEngineMode, number>>);

export type TurnEngineConfig = {
  readonly mode: TurnEngineMode;
  readonly ownSeedCap: number;
  readonly ownBeam: number;
  readonly perNodeFamilyCap: number;
  readonly stepCap: number;
  readonly opponentSeedCap: number;
  readonly opponentBeam: number;
  readonly replySeedCap: number;
  readonly replyBeam: number;
  readonly expansionCap: number;
  readonly enableSpiritFamily: boolean;
  readonly scoringWeights: ScoringWeights;
  readonly enableLazyOracleScoreWindowProjection: boolean;
};

export type TurnSnapshot = {
  readonly stateHash: Hash64;
};

export type TurnAction =
  | { readonly kind: "walk"; readonly actor: Location; readonly to: Location }
  | {
      readonly kind: "attack";
      readonly actor: Location;
      readonly target: Location;
    }
  | {
      readonly kind: "spirit-shift";
      readonly actor: Location;
      readonly target: Location;
      readonly destination: Location;
    }
  | {
      readonly kind: "bomb";
      readonly actor: Location;
      readonly target: Location;
    }
  | {
      readonly kind: "move-mana";
      readonly from: Location;
      readonly to: Location;
    }
  | {
      readonly kind: "score-carry";
      readonly actor: Location;
      readonly wanted: Mana;
      readonly step: Location;
    }
  | {
      readonly kind: "safety-retreat";
      readonly actor: Location;
      readonly to: Location;
    };

export const TurnPlanFamily = Object.freeze({
  ImmediateScore: "immediate-score",
  DenyOpponentWindow: "deny-opponent-window",
  DrainerKill: "drainer-kill",
  SafeSupermanaProgress: "safe-supermana-progress",
  SafeOpponentManaProgress: "safe-opponent-mana-progress",
  DrainerSafetyRecovery: "drainer-safety-recovery",
  SpiritImpact: "spirit-impact",
  ManaTempo: "mana-tempo",
} as const);

export type TurnPlanFamily =
  (typeof TurnPlanFamily)[keyof typeof TurnPlanFamily];

export const TURN_PLAN_FAMILY_PRIORITY_ORDER = Object.freeze([
  TurnPlanFamily.ImmediateScore,
  TurnPlanFamily.DenyOpponentWindow,
  TurnPlanFamily.DrainerKill,
  TurnPlanFamily.DrainerSafetyRecovery,
  TurnPlanFamily.SpiritImpact,
  TurnPlanFamily.SafeSupermanaProgress,
  TurnPlanFamily.SafeOpponentManaProgress,
  TurnPlanFamily.ManaTempo,
] as const satisfies readonly TurnPlanFamily[]);

export const TURN_PLAN_FAMILY_CACHE_TAG = Object.freeze({
  [TurnPlanFamily.ImmediateScore]: 1,
  [TurnPlanFamily.DenyOpponentWindow]: 2,
  [TurnPlanFamily.DrainerKill]: 3,
  [TurnPlanFamily.SafeSupermanaProgress]: 4,
  [TurnPlanFamily.SafeOpponentManaProgress]: 5,
  [TurnPlanFamily.DrainerSafetyRecovery]: 6,
  [TurnPlanFamily.SpiritImpact]: 7,
  [TurnPlanFamily.ManaTempo]: 8,
} satisfies Readonly<Record<TurnPlanFamily, number>>);

/** Lexicographically ordered utility tuple used by the turn engine. */
export type TurnUtility = Readonly<{
  winState: number;
  avoidImmediateLoss: number;
  scoreDelta: number;
  denyGain: number;
  drainerAttack: number;
  drainerSafety: number;
  evalScore: number;
}>;

export function createTurnUtility(
  values: Partial<TurnUtility> = {},
): TurnUtility {
  return {
    winState: values.winState ?? 0,
    avoidImmediateLoss: values.avoidImmediateLoss ?? 0,
    scoreDelta: values.scoreDelta ?? 0,
    denyGain: values.denyGain ?? 0,
    drainerAttack: values.drainerAttack ?? 0,
    drainerSafety: values.drainerSafety ?? 0,
    evalScore: values.evalScore ?? 0,
  };
}

export const EMPTY_TURN_UTILITY = Object.freeze(createTurnUtility());

export function utilityHasNonnegativeDenyGain(utility: TurnUtility): boolean {
  return utility.denyGain >= 0;
}

export function utilitySupportsTemporaryRiskRecovery(
  utility: TurnUtility,
): boolean {
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

export type TurnPackageMeta = {
  readonly scoreGain: number;
  readonly denyGain: number;
  readonly drainerSafetyDelta: number;
  readonly spiritOnlySetup: boolean;
  readonly endsNonnegativeDrainerSafety: boolean;
  readonly opponentImmediateWindowAfter: number;
};

export type TurnPlan = {
  readonly actions: readonly TurnAction[];
  readonly compiledChunks: readonly (readonly Input[])[];
  readonly endGame: MonsGame;
  utility: TurnUtility;
  readonly headUtility: TurnUtility;
  readonly headFamily: TurnPlanFamily;
  readonly goalFamily: TurnPlanFamily;
  readonly packageMeta: TurnPackageMeta;
};

export const OpportunityKind = Object.freeze({
  ImmediateScore: "immediate-score",
  TacticalDeny: "tactical-deny",
  DrainerKill: "drainer-kill",
  SafeSupermanaProgress: "safe-supermana-progress",
  SafeOpponentManaProgress: "safe-opponent-mana-progress",
  DrainerSafetyRecovery: "drainer-safety-recovery",
  SpiritImpact: "spirit-impact",
  ManaTempo: "mana-tempo",
} as const);

export type OpportunityKind =
  (typeof OpportunityKind)[keyof typeof OpportunityKind];

export type OpportunityBudget = {
  readonly monMovesNeeded: number;
  readonly needsAction: boolean;
  readonly needsManaMove: boolean;
};

export type OpportunityDelta = {
  readonly sameTurnScoreWindowGain: number;
  readonly spiritGain: number;
  readonly opponentWindowDenyGain: number;
  readonly drainerAttack: boolean;
  readonly drainerSafetyDelta: number;
  readonly supermanaProgressGain: number;
  readonly opponentManaProgressGain: number;
};

export type TurnOpportunity = {
  readonly kind: OpportunityKind;
  readonly family: TurnPlanFamily;
  readonly action: TurnAction;
  readonly priority: number;
  readonly budget: OpportunityBudget;
  readonly delta: OpportunityDelta;
};

export type ActionSeed = {
  readonly family: TurnPlanFamily;
  readonly action: TurnAction;
  readonly priority: number;
};

export type PlanNode = {
  readonly game: MonsGame;
  readonly stateHash: Hash64;
  readonly actions: readonly TurnAction[];
  readonly compiledChunks: readonly (readonly Input[])[];
  readonly headUtility: TurnUtility;
  readonly headFamily: TurnPlanFamily;
  readonly goalFamily: TurnPlanFamily;
};

export type MacroOpportunity = {
  readonly headFamily: TurnPlanFamily;
  readonly goalFamily: TurnPlanFamily;
  readonly priority: number;
  readonly delta: OpportunityDelta;
  readonly actions: readonly TurnAction[];
  readonly compiledChunks: readonly (readonly Input[])[];
  readonly endGame: MonsGame;
  readonly endSnapshot: TurnSnapshot;
  readonly headUtility: TurnUtility;
  readonly signature: Hash64;
};

export type MacroPlanNode = PlanNode & {
  readonly signature: Hash64;
};

export const PlanBuildStatus = Object.freeze({
  NoPlan: "no-plan",
  BudgetExceeded: "budget-exceeded",
} as const);

export type PlanBuildStatus =
  (typeof PlanBuildStatus)[keyof typeof PlanBuildStatus];

export type PlanGenerationResult =
  | { readonly status: "ok"; readonly plans: TurnPlan[] }
  | { readonly status: PlanBuildStatus };

export type PlanBuildResult =
  | { readonly status: "ok"; readonly plan: TurnPlan }
  | { readonly status: PlanBuildStatus };

export type TurnOracleContext = {
  readonly opportunity: ExactOpportunityContext;
  readonly strategic: ExactColorSummary;
  readonly opponentImmediateWindow: number;
};

export const EMPTY_PACKAGE_META: TurnPackageMeta = Object.freeze({
  scoreGain: 0,
  denyGain: 0,
  drainerSafetyDelta: 0,
  spiritOnlySetup: false,
  endsNonnegativeDrainerSafety: false,
  opponentImmediateWindowAfter: 0,
});

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

export function compareLocations(left: Location, right: Location): number {
  const rowOrder = compareNumber(left.i, right.i);
  return rowOrder !== 0 ? rowOrder : compareNumber(left.j, right.j);
}

export function compareOptionalLocations(
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

export function turnSnapshotFromGame(game: MonsGame): TurnSnapshot {
  return { stateHash: exactSearchStateHash(game) };
}

export function compareTurnUtilities(
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

export function headOpeningRiskClass(utility: TurnUtility): number {
  if (utility.avoidImmediateLoss < 0) return 0;
  if (utility.drainerSafety < 0 || utility.scoreDelta < 0) return 1;
  return 2;
}

export function shouldCompareHeadOpeningUtility(
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

export function comparePlanRank(
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
    shouldCompareHeadOpeningUtility(
      leftHeadFamily,
      leftHeadUtility,
      rightHeadUtility,
    )
  ) {
    order = compareUtilityPrimaryAxes(leftHeadUtility, rightHeadUtility);
    if (order !== 0) return order;
    order = compareNumber(
      leftHeadUtility.evalScore,
      rightHeadUtility.evalScore,
    );
    if (order !== 0) return order;
  }
  return compareNumber(leftUtility.evalScore, rightUtility.evalScore);
}

export function comparePackageMeta(
  left: TurnPackageMeta,
  right: TurnPackageMeta,
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

export function turnEngineComparePlans(
  left: TurnPlan,
  right: TurnPlan,
): number {
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
  order = compareNumber(
    familyRank(right.goalFamily),
    familyRank(left.goalFamily),
  );
  if (order !== 0) return order;
  order = compareNumber(
    familyRank(right.headFamily),
    familyRank(left.headFamily),
  );
  if (order !== 0) return order;
  order = compareNumber(right.actions.length, left.actions.length);
  return order !== 0
    ? order
    : compareChunks(left.compiledChunks, right.compiledChunks);
}
