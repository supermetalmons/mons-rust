import type { Color } from "../../../api/types.js";
import type { Input } from "../../../engine/model/domain.js";
import type { MonsGame } from "../../../engine/game/mons-game.js";
import type { RootCandidate } from "../../root/types.js";
import type { AutomoveConfig } from "../../config/types.js";
import type { TurnPlan, TurnPlanFamily } from "../../turn/model.js";

export const ProductionRootAdvisorReasonCode = Object.freeze({
  RankedRoot: "ranked-root",
  ReplyRiskShortlist: "reply-risk-shortlist",
  PreserveSpiritRepresentative: "preserve-spirit-representative",
  PreserveSafeProgressRepresentative: "preserve-safe-progress-representative",
  PreserveManaTempoRepresentative: "preserve-mana-tempo-representative",
  OmittedRootReentry: "omitted-root-reentry",
  AdmitInjectedMacroRoot: "admit-injected-macro-root",
  RejectInjectedMacroRoot: "reject-injected-macro-root",
  ApprovedReplyRiskGuard: "approved-reply-risk-guard",
  ApprovedBaselineSelector: "approved-baseline-selector",
  ApprovedFamilyCompetition: "approved-family-competition",
} as const);

export type ProductionRootAdvisorReasonCode =
  (typeof ProductionRootAdvisorReasonCode)[keyof typeof ProductionRootAdvisorReasonCode];

export type ProductionRootAdvisorEntry = {
  readonly inputs: readonly Input[];
  readonly family: TurnPlanFamily;
  readonly rootRank: number;
  readonly reason: ProductionRootAdvisorReasonCode;
};

export type ProductionInjectedRootAdvisorDecision = {
  readonly inputs: readonly Input[];
  readonly family: TurnPlanFamily;
  readonly admitted: boolean;
  readonly reason: ProductionRootAdvisorReasonCode;
};

export type ProductionRootAdvisorDecision = {
  readonly orderedShortlist: readonly ProductionRootAdvisorEntry[];
  readonly preservedFamilyRepresentatives: readonly ProductionRootAdvisorEntry[];
  readonly approvedRoot: ProductionRootAdvisorEntry | undefined;
  readonly injectedRoot: ProductionInjectedRootAdvisorDecision | undefined;
};

export type ProductionAdvisorOptions = {
  /** Builds the exact scored first-chunk root when the turn-engine head was not enumerated. */
  readonly buildInjectedRootCandidate?: (
    game: MonsGame,
    perspective: Color,
    config: AutomoveConfig,
    inputs: readonly Input[],
    plan: TurnPlan,
  ) => RootCandidate | undefined;
};

export type ProductionRootAdvisorPostsearchResult = {
  readonly index: number;
  readonly decision: ProductionRootAdvisorDecision;
};
