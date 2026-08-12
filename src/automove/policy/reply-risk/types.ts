import type { Color } from "../../../api/types.js";
import type { MonsGame } from "../../../engine/game/mons-game.js";
import type { RootCandidate } from "../../root/types.js";
import type { EvaluatedRoot } from "../../root/types.js";
import type { TurnPlan, TurnPlanFamily, TurnUtility } from "../../turn/model.js";

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

export type FullReplyRiskComparisonContext = ReplyRiskComparisonContext & {
  readonly game: MonsGame;
  readonly evaluations: readonly EvaluatedRoot[];
  readonly candidateIndex: number;
  readonly incumbentIndex: number;
  readonly perspective: Color;
};

export const NO_REPLY_RISK_HOOKS: ReplyRiskHooks = Object.freeze({});
