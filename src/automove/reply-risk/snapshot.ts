import { TARGET_SCORE } from "../../engine/config.js";
import { Color } from "../../engine/domain.js";
import type { MonsGame } from "../../engine/game.js";
import type { AutomoveExecutionContext } from "../execution-context.js";
import { MAX_SCORE } from "../score-math.js";
import { evaluatePreferabilityWithWeightsAndExactPolicy } from "../scoring.js";
import type { AutomoveConfig } from "../selector-types.js";
import { enumerateLegalTransitions } from "../transitions.js";
import { SMART_TERMINAL_SCORE } from "./config.js";
import {
  cachedReplyRiskSnapshot,
  replyRiskCacheKey,
  storeReplyRiskSnapshot,
} from "./cache.js";
import type { RootReplyRiskSnapshot } from "./types.js";

function conservativeSnapshot(): RootReplyRiskSnapshot {
  return {
    allowsImmediateOpponentWin: true,
    opponentReachesMatchPoint: true,
    worstReplyScore: -Math.trunc(SMART_TERMINAL_SCORE / 2),
  };
}

export function evaluateReplyRiskGame(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: AutomoveConfig,
): number {
  return evaluatePreferabilityWithWeightsAndExactPolicy(
    execution,
    game,
    perspective,
    config.evaluation.weights,
    false,
  );
}

export function rootReplyRiskSnapshot(
  execution: AutomoveExecutionContext,
  stateAfterMove: MonsGame,
  perspective: Color,
  config: AutomoveConfig,
  replyLimit: number,
): RootReplyRiskSnapshot {
  if (execution.session.checkpoint()) return conservativeSnapshot();
  const normalizedReplyLimit = Math.max(1, Math.trunc(replyLimit));
  const key = replyRiskCacheKey(
    stateAfterMove,
    perspective,
    normalizedReplyLimit,
    config,
  );
  if (key !== undefined) {
    const cached = cachedReplyRiskSnapshot(execution, key);
    if (cached !== undefined) return cached;
  }

  const winner = stateAfterMove.winnerColor();
  let snapshot: RootReplyRiskSnapshot;
  if (winner !== undefined) {
    const perspectiveWon = winner === perspective;
    snapshot = {
      allowsImmediateOpponentWin: !perspectiveWon,
      opponentReachesMatchPoint: !perspectiveWon,
      worstReplyScore: perspectiveWon
        ? Math.trunc(SMART_TERMINAL_SCORE / 2)
        : -Math.trunc(SMART_TERMINAL_SCORE / 2),
    };
  } else if (stateAfterMove.activeColor === perspective) {
    snapshot = {
      allowsImmediateOpponentWin: false,
      opponentReachesMatchPoint: false,
      worstReplyScore: evaluateReplyRiskGame(
        execution,
        stateAfterMove,
        perspective,
        config,
      ),
    };
  } else {
    const replies = enumerateLegalTransitions(
      execution,
      stateAfterMove,
      normalizedReplyLimit,
    );
    if (execution.session.checkpoint()) return conservativeSnapshot();
    if (replies.length === 0) {
      snapshot = {
        allowsImmediateOpponentWin: false,
        opponentReachesMatchPoint: false,
        worstReplyScore: Math.trunc(SMART_TERMINAL_SCORE / 4),
      };
    } else {
      let allowsImmediateOpponentWin = false;
      let opponentReachesMatchPoint = false;
      let worstReplyScore = MAX_SCORE;
      let evaluatedReply = false;
      for (const reply of replies) {
        if (execution.session.checkpoint()) return conservativeSnapshot();
        const afterReply = reply.game;
        evaluatedReply = true;
        const opponentScoreAfter =
          perspective === Color.White
            ? afterReply.blackScore
            : afterReply.whiteScore;
        if (TARGET_SCORE - opponentScoreAfter <= 1) {
          opponentReachesMatchPoint = true;
        }
        const replyWinner = afterReply.winnerColor();
        let replyScore: number;
        if (replyWinner === perspective) {
          replyScore = Math.trunc(SMART_TERMINAL_SCORE / 2);
        } else if (replyWinner !== undefined) {
          allowsImmediateOpponentWin = true;
          opponentReachesMatchPoint = true;
          replyScore = -Math.trunc(SMART_TERMINAL_SCORE / 2);
        } else {
          replyScore = evaluateReplyRiskGame(
            execution,
            afterReply,
            perspective,
            config,
          );
        }
        worstReplyScore = Math.min(worstReplyScore, replyScore);
        if (allowsImmediateOpponentWin) break;
      }
      if (!evaluatedReply || worstReplyScore === MAX_SCORE) {
        worstReplyScore = evaluateReplyRiskGame(
          execution,
          stateAfterMove,
          perspective,
          config,
        );
      }
      snapshot = {
        allowsImmediateOpponentWin,
        opponentReachesMatchPoint,
        worstReplyScore,
      };
    }
  }

  if (execution.session.cacheWriteAllowed && key !== undefined) {
    storeReplyRiskSnapshot(execution, key, snapshot);
  }
  return snapshot;
}
