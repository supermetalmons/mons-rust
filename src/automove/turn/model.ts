import type { MonsGame } from "../../engine/game/mons-game.js";
import type { Input } from "../../engine/model/domain.js";
import type { Mana } from "../../api/types.js";
import type { Location } from "../../engine/board/geometry.js";
import type { ExactColorSummary, ExactOpportunityContext } from "../exact/types.js";
import { hash64, type Hash64 } from "../core/hash64.js";
import type { ScoringWeights } from "../scoring/profile-schema.js";

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

export type TurnEngineMode = (typeof TurnEngineMode)[keyof typeof TurnEngineMode];

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

type TurnSnapshot = {
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

export type TurnPlanFamily = (typeof TurnPlanFamily)[keyof typeof TurnPlanFamily];

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

export function createTurnUtility(values: Partial<TurnUtility> = {}): TurnUtility {
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

type TurnPackageMeta = {
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

export type OpportunityKind = (typeof OpportunityKind)[keyof typeof OpportunityKind];

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

export type PlanBuildStatus = (typeof PlanBuildStatus)[keyof typeof PlanBuildStatus];

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
