import type { Color } from "../../api/types.js";
import type { MonsGame } from "../../engine/game/mons-game.js";
import type { AutomoveConfig } from "../config/types.js";
import type { EvaluatedRoot } from "./types.js";

type RootReplyRiskSnapshot = {
  readonly allowsImmediateOpponentWin: boolean;
};

export type RootSelectionContext = {
  readonly game: MonsGame;
  readonly roots: readonly EvaluatedRoot[];
  readonly candidateIndices: readonly number[];
  readonly perspective: Color;
  readonly config: AutomoveConfig;
};

export const ProductionCompetitionKind = Object.freeze({
  SafeProgress: "safe-progress",
  FollowupProgress: "followup-progress",
  RiskyScore: "risky-score",
  NegativeDeny: "negative-deny",
  Score: "score",
  Projection: "projection",
  RiskyRecovery: "risky-recovery",
} as const);

export type ProductionCompetitionKind =
  (typeof ProductionCompetitionKind)[keyof typeof ProductionCompetitionKind];

export const PRODUCTION_COMPETITION_KIND_ORDER = Object.freeze([
  ProductionCompetitionKind.SafeProgress,
  ProductionCompetitionKind.FollowupProgress,
  ProductionCompetitionKind.RiskyScore,
  ProductionCompetitionKind.NegativeDeny,
  ProductionCompetitionKind.Score,
  ProductionCompetitionKind.Projection,
  ProductionCompetitionKind.RiskyRecovery,
] as const satisfies readonly ProductionCompetitionKind[]);

export const ProductionComparisonPhase = Object.freeze({
  SpiritSetup: "spirit-setup",
  ProjectionChallenge: "projection-challenge",
  Projection: "projection",
  FollowupFloor: "followup-floor",
} as const);

export type ProductionComparisonPhase =
  (typeof ProductionComparisonPhase)[keyof typeof ProductionComparisonPhase];

type ProductionRootRuleId =
  | `competition.${string}`
  | `safety-reentry.${string}`
  | `final-reentry.${string}`
  | `comparison.${string}`
  | `root-picker.${string}`;

type ProductionContinueResult = {
  readonly kind: "continue";
};

export type ProductionCompetitionResult =
  ProductionContinueResult | { readonly kind: "select" };

export type ProductionIndexSelectionResult =
  | ProductionContinueResult
  | {
      readonly kind: "select";
      readonly indices: readonly number[];
    };

type ProductionRootSelectionResult =
  | ProductionContinueResult
  | {
      readonly kind: "select";
      readonly index: number;
    };

export type ProductionComparisonResult =
  | ProductionContinueResult
  | {
      readonly kind: "compare";
      /** Positive means candidate wins, negative means incumbent wins. */
      readonly order: number;
    };

export type ProductionRootComparisonContext = RootSelectionContext & {
  readonly candidateIndex: number;
  readonly incumbentIndex: number;
};

export type ProductionRootReentryContext = RootSelectionContext & {
  readonly selectedIndices: readonly number[];
};

type ProductionCompetitionRule = {
  readonly id: ProductionRootRuleId;
  readonly kind: ProductionCompetitionKind;
  readonly evaluate: (context: RootSelectionContext) => ProductionCompetitionResult;
};

type ProductionReentryRule = {
  readonly id: ProductionRootRuleId;
  readonly select: (
    context: ProductionRootReentryContext,
  ) => ProductionIndexSelectionResult;
};

type ProductionComparisonRule = {
  readonly id: ProductionRootRuleId;
  readonly phase: ProductionComparisonPhase;
  readonly compare: (
    context: ProductionRootComparisonContext,
  ) => ProductionComparisonResult;
};

export type ProductionRootPicker = {
  readonly id: ProductionRootRuleId;
  readonly select: (context: RootSelectionContext) => ProductionRootSelectionResult;
};

export type ProductionRootPolicy = {
  readonly competitionRules: readonly ProductionCompetitionRule[];
  readonly safetyReentryRules: readonly ProductionReentryRule[];
  readonly finalReentryRules: readonly ProductionReentryRule[];
  readonly comparisonRules: readonly ProductionComparisonRule[];
  readonly rootPicker?: ProductionRootPicker;
};

export type RootSelectorOptions = {
  readonly rootReplyRiskSnapshot?: (
    stateAfterMove: MonsGame,
    perspective: Color,
    config: AutomoveConfig,
    replyLimit: number,
    rootIndex: number,
  ) => RootReplyRiskSnapshot;
  readonly pickReplyRiskGuardedIndex?: (
    context: RootSelectionContext,
  ) => number | undefined;
  readonly productionPolicy?: ProductionRootPolicy;
  readonly checkpoint?: () => boolean;
  readonly cancelled?: () => boolean;
};
