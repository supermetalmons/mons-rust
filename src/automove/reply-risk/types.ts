import type { Color } from "../../engine/domain.js";
import type { MonsGame } from "../../engine/game.js";
import type { RootCandidate } from "../root-candidates.js";
import type { EvaluatedRoot } from "../search.js";
import type { TurnPlan, TurnPlanFamily, TurnUtility } from "../turn-engine.js";

export type TurnEngineRootProjection = {
  readonly plan: TurnPlan;
};

export type ReplyRiskHooks = {
  readonly evaluateTurnEngineRootUtility?: (
    game: MonsGame,
    root: RootCandidate,
    perspective: Color,
    family: TurnPlanFamily,
  ) => TurnUtility;
};

export type RootReplyRiskSnapshot = {
  readonly allowsImmediateOpponentWin: boolean;
  readonly opponentReachesMatchPoint: boolean;
  readonly worstReplyScore: number;
};

export type ReplyRiskComparisonContext = {
  readonly candidateProjection?: TurnEngineRootProjection | undefined;
  readonly incumbentProjection?: TurnEngineRootProjection | undefined;
  readonly game?: MonsGame | undefined;
  readonly evaluations?: readonly EvaluatedRoot[] | undefined;
  readonly candidateIndex?: number | undefined;
  readonly incumbentIndex?: number | undefined;
  readonly perspective?: Color | undefined;
  readonly spiritFollowupScores?: Map<number, number> | undefined;
};

export const NO_REPLY_RISK_HOOKS: ReplyRiskHooks = Object.freeze({});
