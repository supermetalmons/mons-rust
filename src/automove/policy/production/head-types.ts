import type { Color } from "../../../api/types.js";
import type { MonsGame } from "../../../engine/game/mons-game.js";
import { exactOpportunityContext } from "../../exact/turn-opportunity.js";
import type { EvaluatedRoot } from "../../root/types.js";
import type { AutomoveConfig } from "../../config/types.js";
import type { TurnPlan, TurnPlanFamily } from "../../turn/model.js";
import { turnEngineSelectedUtility } from "./head-projection.js";
import type { ProductionHeadGuardId } from "./head-guard-order.js";

type TurnEngineHeadUtility = ReturnType<typeof turnEngineSelectedUtility>;

export type TurnEngineHeadAcceptanceContext = {
  readonly game: MonsGame;
  readonly perspective: Color;
  readonly config: AutomoveConfig;
  readonly plan: TurnPlan;
  readonly candidateIndex: number;
  readonly candidate: EvaluatedRoot;
  readonly selected: EvaluatedRoot;
  readonly macroMode: boolean;
  readonly candidateUnsafe: boolean;
  readonly selectedUnsafe: boolean;
  readonly candidateProgress: boolean;
  readonly selectedProgress: boolean;
  readonly exactContext: ReturnType<typeof exactOpportunityContext>;
  readonly scoreGap: number;
  readonly sameTurnWindowBetter: boolean;
  readonly drainerAttackBetter: boolean;
  readonly scoresNowBetter: boolean;
  readonly safetyRecoverBetter: boolean;
  readonly spiritWindowBetter: boolean;
  readonly spiritDevelopmentBetter: boolean;
  readonly candidateSpiritTactical: boolean;
  readonly progressBetter: boolean;
  readonly selectedSpiritPhase: boolean;
  readonly candidateFamily: TurnPlanFamily;
  readonly selectedFamily: TurnPlanFamily;
  readonly selectedUtilityValue: () => TurnEngineHeadUtility;
  readonly candidateUtilityValue: () => TurnEngineHeadUtility;
  readonly blackSpiritPair: boolean;
  readonly whiteSpiritSetupGain: boolean;
  readonly blackTurnSixRouteChangePlainSpirit: boolean;
};

export type TurnEngineHeadFamilyPolicyContext = {
  readonly selectedUtility: TurnEngineHeadUtility;
  readonly pickupUpgrade: boolean;
  readonly strategicAxesBetter: boolean;
  readonly projectedDeferredRecoveryWithoutConcreteGain: boolean;
  readonly safeRootBlocksPlainSpirit: boolean;
  readonly safeRootBlocksPlainSpiritProgress: boolean;
  readonly plainSpiritSiblingRegresses: boolean;
  readonly allowNonConcreteWhiteProgress: boolean;
  readonly whiteSetupRecoveryBlocksUtilityOverride: boolean;
};

export type TurnEngineHeadInitialGuardFacts = {
  readonly narrowUnsafeBlackManaScore: boolean;
  readonly pickupUpgrade: boolean;
};

export type TurnEngineHeadOrderedFacts = TurnEngineHeadInitialGuardFacts & {
  readonly nearTieProgress: boolean;
  readonly primaryAxesOrder: number;
  readonly strategicAxesBetter: boolean;
  readonly selectedUtility: TurnEngineHeadUtility;
};

export type TurnEngineHeadInitialGuardResult =
  | { readonly kind: "reject"; readonly reason: ProductionHeadGuardId }
  | {
      readonly kind: "continue";
      readonly facts: TurnEngineHeadInitialGuardFacts;
    };

export type TurnEngineHeadOrderedDecision =
  | { readonly kind: "accept" }
  | { readonly kind: "reject"; readonly reason: ProductionHeadGuardId }
  | {
      readonly kind: "delegate";
      readonly policy: TurnEngineHeadFamilyPolicyContext;
    };
