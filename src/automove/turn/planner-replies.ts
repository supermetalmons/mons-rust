import { otherColor } from "../../engine/model/domain.js";
import type { Color } from "../../api/types.js";
import type { AutomoveExecutionContext } from "../core/execution-context.js";
import { exactSearchStateHash } from "../exact/hash.js";
import {
  createTurnUtilityEvalContextWithSearchHash,
  evaluateStateUtilityWithSearchHash,
  type TurnUtilityEvalContext,
} from "./evaluation.js";
import {
  EMPTY_TURN_UTILITY,
  type TurnEngineConfig,
  type TurnPlan,
  type TurnUtility,
} from "./model.js";
import { compareChunks, compareTurnUtilities } from "./ordering.js";
import { generatePlansForMode } from "./planner-generation.js";

function replyShortlistLength(total: number, beam: number): number {
  return Math.min(total, Math.min(Math.max(Math.max(beam, 0) * 2, 4), 8));
}

export function evaluatePlanWithReplies(
  execution: AutomoveExecutionContext,
  plan: TurnPlan,
  perspective: Color,
  config: TurnEngineConfig,
  utilityContext: TurnUtilityEvalContext,
): TurnUtility {
  if (execution.session.checkpoint()) return EMPTY_TURN_UTILITY;
  const after = plan.endGame;
  const afterHash = exactSearchStateHash(after);
  const opponent = otherColor(perspective);
  if (after.winnerColor() !== undefined || after.activeColor !== opponent) {
    return evaluateStateUtilityWithSearchHash(utilityContext, after, afterHash);
  }

  const opponentConfig: TurnEngineConfig = {
    ...config,
    ownSeedCap: Math.max(config.opponentSeedCap, 1),
    ownBeam: Math.max(config.opponentBeam, 1),
    perNodeFamilyCap: Math.max(config.perNodeFamilyCap, 1),
    stepCap: Math.min(Math.max(config.stepCap, 1), 4),
    opponentSeedCap: Math.max(config.replySeedCap, 1),
    opponentBeam: Math.max(config.replyBeam, 1),
    replySeedCap: 0,
    replyBeam: 0,
    expansionCap: Math.max(Math.trunc(config.expansionCap / 2), 24),
  };
  const opponentUtilityContext = createTurnUtilityEvalContextWithSearchHash(
    execution,
    after,
    afterHash,
    opponent,
    opponentConfig,
  );
  const opponentResult = generatePlansForMode(
    execution,
    after,
    opponent,
    opponentConfig,
    opponentUtilityContext,
    opponentConfig.ownSeedCap,
    opponentConfig.ownBeam,
    opponentConfig.stepCap,
    opponentConfig.expansionCap,
  );
  if (execution.session.checkpoint()) return EMPTY_TURN_UTILITY;
  if (opponentResult.status !== "ok")
    return evaluateStateUtilityWithSearchHash(utilityContext, after, afterHash);
  const opponentPlans = opponentResult.plans;

  const shortlist = replyShortlistLength(opponentPlans.length, opponentConfig.ownBeam);
  let bestOpponent = opponentPlans[0];
  if (bestOpponent === undefined)
    return evaluateStateUtilityWithSearchHash(utilityContext, after, afterHash);
  let bestOpponentHash = exactSearchStateHash(bestOpponent.endGame);
  let bestOpponentUtility = evaluateStateUtilityWithSearchHash(
    opponentUtilityContext,
    bestOpponent.endGame,
    bestOpponentHash,
  );
  for (const opponentPlan of opponentPlans.slice(1, shortlist)) {
    if (execution.session.checkpoint()) return EMPTY_TURN_UTILITY;
    const opponentPlanHash = exactSearchStateHash(opponentPlan.endGame);
    const utility = evaluateStateUtilityWithSearchHash(
      opponentUtilityContext,
      opponentPlan.endGame,
      opponentPlanHash,
    );
    const utilityOrder = compareTurnUtilities(utility, bestOpponentUtility);
    if (
      utilityOrder > 0 ||
      (utilityOrder === 0 &&
        compareChunks(opponentPlan.compiledChunks, bestOpponent.compiledChunks) < 0)
    ) {
      bestOpponent = opponentPlan;
      bestOpponentHash = opponentPlanHash;
      bestOpponentUtility = utility;
    }
  }

  const afterOpponent = bestOpponent.endGame;
  if (
    afterOpponent.winnerColor() !== undefined ||
    afterOpponent.activeColor !== perspective ||
    config.replySeedCap === 0
  ) {
    return evaluateStateUtilityWithSearchHash(
      utilityContext,
      afterOpponent,
      bestOpponentHash,
    );
  }

  const replyConfig: TurnEngineConfig = {
    ...config,
    ownSeedCap: Math.max(config.replySeedCap, 1),
    ownBeam: Math.max(config.replyBeam, 1),
    perNodeFamilyCap: Math.max(config.perNodeFamilyCap, 1),
    stepCap: Math.min(Math.max(config.stepCap, 1), 3),
    opponentSeedCap: 0,
    opponentBeam: 0,
    replySeedCap: 0,
    replyBeam: 0,
    expansionCap: Math.max(Math.trunc(config.expansionCap / 3), 16),
  };
  const replyUtilityContext = createTurnUtilityEvalContextWithSearchHash(
    execution,
    afterOpponent,
    bestOpponentHash,
    perspective,
    replyConfig,
  );
  const replyResult = generatePlansForMode(
    execution,
    afterOpponent,
    perspective,
    replyConfig,
    replyUtilityContext,
    replyConfig.ownSeedCap,
    replyConfig.ownBeam,
    replyConfig.stepCap,
    replyConfig.expansionCap,
  );
  if (execution.session.checkpoint()) return EMPTY_TURN_UTILITY;
  if (replyResult.status !== "ok") {
    return evaluateStateUtilityWithSearchHash(
      utilityContext,
      afterOpponent,
      bestOpponentHash,
    );
  }
  const replyPlans = replyResult.plans;
  let bestReply: TurnUtility | undefined;
  for (const reply of replyPlans.slice(
    0,
    replyShortlistLength(replyPlans.length, replyConfig.ownBeam),
  )) {
    const replyHash = exactSearchStateHash(reply.endGame);
    const utility = evaluateStateUtilityWithSearchHash(
      utilityContext,
      reply.endGame,
      replyHash,
    );
    if (bestReply === undefined || compareTurnUtilities(utility, bestReply) > 0) {
      bestReply = utility;
    }
  }
  return (
    bestReply ??
    evaluateStateUtilityWithSearchHash(utilityContext, afterOpponent, bestOpponentHash)
  );
}
