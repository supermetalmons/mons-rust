import { TARGET_SCORE } from "../../engine/config.js";
import { Color } from "../../engine/domain.js";
import type { MonsGame } from "../../engine/game.js";
import { scoreForColor } from "../../engine/legality.js";
import type { AutomoveExecutionContext } from "../execution-context.js";
import { MAX_SCORE, saturatingScoreSubtract } from "../score-math.js";
import { evaluateSearchScore } from "../search.js";
import type { EvaluatedRoot } from "../search.js";
import { patchAutomoveConfig } from "../selector-config.js";
import { hasProgressSurface, productionEnabled } from "../selector-types.js";
import type { AutomoveConfig } from "../selector-types.js";
import { enumerateLegalTransitions } from "../transitions.js";
import { SMART_TERMINAL_SCORE } from "./config.js";
import {
  isTacticalPriorityRoot,
  sameNonTacticalProgressLane,
} from "./ranking.js";
import { evaluateReplyRiskGame } from "./snapshot.js";
import type { RootReplyRiskSnapshot } from "./types.js";
import { sameOpeningSafeSetupPair } from "./sibling-ordering.js";

function isFlatLateManaOnlyReplyRoot(root: EvaluatedRoot): boolean {
  return (
    !root.winsImmediately &&
    !root.attacksOpponentDrainer &&
    !isTacticalPriorityRoot(root) &&
    !root.spiritDevelopment &&
    !root.spiritSameTurnScoreSetupNow &&
    !root.spiritOwnManaSetupNow &&
    !root.supermanaProgress &&
    !root.opponentManaProgress &&
    !root.scoresSupermanaThisTurn &&
    !root.scoresOpponentManaThisTurn &&
    !root.safeSupermanaPickupNow &&
    !root.safeOpponentManaPickupNow &&
    root.sameTurnScoreWindowValue <= 0 &&
    !root.manaHandoffToOpponent &&
    !root.hasRoundtrip
  );
}

export function lateSafeManaRootOrder(
  game: MonsGame,
  candidate: EvaluatedRoot,
  candidateSnapshot: RootReplyRiskSnapshot,
  incumbent: EvaluatedRoot,
  incumbentSnapshot: RootReplyRiskSnapshot,
  config: AutomoveConfig,
): number | undefined {
  if (
    !productionEnabled(config) ||
    !config.replyRisk.lateSafeManaRootPreference ||
    game.activeColor !== Color.White ||
    game.turnNumber < 6 ||
    game.monsMovesCount !== 0 ||
    game.playerCanUseAction() ||
    !game.playerCanMoveMana() ||
    candidateSnapshot.allowsImmediateOpponentWin ||
    incumbentSnapshot.allowsImmediateOpponentWin ||
    candidateSnapshot.opponentReachesMatchPoint ||
    incumbentSnapshot.opponentReachesMatchPoint ||
    !isFlatLateManaOnlyReplyRoot(candidate) ||
    !isFlatLateManaOnlyReplyRoot(incumbent)
  ) {
    return undefined;
  }
  let safe: EvaluatedRoot;
  let vulnerable: EvaluatedRoot;
  let candidateIsSafe: boolean;
  if (!candidate.ownDrainerVulnerable && incumbent.ownDrainerVulnerable) {
    safe = candidate;
    vulnerable = incumbent;
    candidateIsSafe = true;
  } else if (
    candidate.ownDrainerVulnerable &&
    !incumbent.ownDrainerVulnerable
  ) {
    safe = incumbent;
    vulnerable = candidate;
    candidateIsSafe = false;
  } else {
    return undefined;
  }
  if (
    vulnerable.score <= safe.score ||
    saturatingScoreSubtract(vulnerable.score, safe.score) > 16 ||
    Math.abs(vulnerable.rootRank - safe.rootRank) > 4 ||
    vulnerable.efficiency <= safe.efficiency
  ) {
    return undefined;
  }
  return candidateIsSafe ? 1 : -1;
}

type NormalRootSafetySnapshot = {
  readonly allowsImmediateOpponentWin: boolean;
  readonly opponentReachesMatchPoint: boolean;
  readonly opponentMaxScoreGain: number;
  readonly myScoreGain: number;
  readonly worstReplyScore: number;
};

function normalRootSafetySnapshot(
  execution: AutomoveExecutionContext,
  stateAfterMove: MonsGame,
  perspective: Color,
  myScoreBefore: number,
  config: AutomoveConfig,
  replyLimit: number,
): NormalRootSafetySnapshot {
  const myScoreGain = Math.max(
    0,
    scoreForColor(stateAfterMove, perspective) - myScoreBefore,
  );
  const winner = stateAfterMove.winnerColor();
  if (winner !== undefined) {
    const won = winner === perspective;
    return {
      allowsImmediateOpponentWin: !won,
      opponentReachesMatchPoint: !won,
      opponentMaxScoreGain: won ? 0 : TARGET_SCORE,
      myScoreGain,
      worstReplyScore: won
        ? Math.trunc(SMART_TERMINAL_SCORE / 2)
        : -Math.trunc(SMART_TERMINAL_SCORE / 2),
    };
  }
  if (stateAfterMove.activeColor === perspective) {
    return {
      allowsImmediateOpponentWin: false,
      opponentReachesMatchPoint: false,
      opponentMaxScoreGain: 0,
      myScoreGain,
      worstReplyScore: evaluateReplyRiskGame(
        execution,
        stateAfterMove,
        perspective,
        config,
      ),
    };
  }
  const opponent = perspective === Color.White ? Color.Black : Color.White;
  const opponentScoreBefore = scoreForColor(stateAfterMove, opponent);
  const replies = enumerateLegalTransitions(
    execution,
    stateAfterMove,
    Math.max(1, Math.trunc(replyLimit)),
  );
  if (replies.length === 0) {
    return {
      allowsImmediateOpponentWin: false,
      opponentReachesMatchPoint: false,
      opponentMaxScoreGain: 0,
      myScoreGain,
      worstReplyScore: Math.trunc(SMART_TERMINAL_SCORE / 4),
    };
  }
  let allowsImmediateOpponentWin = false;
  let opponentReachesMatchPoint = false;
  let opponentMaxScoreGain = 0;
  let worstReplyScore = MAX_SCORE;
  for (const reply of replies) {
    const afterReply = reply.game;
    const opponentScoreAfter = scoreForColor(afterReply, opponent);
    opponentMaxScoreGain = Math.max(
      opponentMaxScoreGain,
      Math.max(0, opponentScoreAfter - opponentScoreBefore),
    );
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
  if (worstReplyScore === MAX_SCORE) {
    worstReplyScore = evaluateReplyRiskGame(
      execution,
      stateAfterMove,
      perspective,
      config,
    );
  }
  return {
    allowsImmediateOpponentWin,
    opponentReachesMatchPoint,
    opponentMaxScoreGain,
    myScoreGain,
    worstReplyScore,
  };
}

function betterNormalRootSafetyCandidate(
  candidate: NormalRootSafetySnapshot,
  candidateScore: number,
  incumbent: NormalRootSafetySnapshot,
  incumbentScore: number,
): boolean {
  if (
    candidate.allowsImmediateOpponentWin !==
    incumbent.allowsImmediateOpponentWin
  ) {
    return !candidate.allowsImmediateOpponentWin;
  }
  if (
    candidate.opponentReachesMatchPoint !== incumbent.opponentReachesMatchPoint
  ) {
    return !candidate.opponentReachesMatchPoint;
  }
  if (candidate.opponentMaxScoreGain !== incumbent.opponentMaxScoreGain) {
    return candidate.opponentMaxScoreGain < incumbent.opponentMaxScoreGain;
  }
  if (candidate.myScoreGain !== incumbent.myScoreGain) {
    return candidate.myScoreGain > incumbent.myScoreGain;
  }
  if (candidate.worstReplyScore !== incumbent.worstReplyScore) {
    return candidate.worstReplyScore > incumbent.worstReplyScore;
  }
  return candidateScore > incumbentScore;
}

function quietNonTacticalReplyRiskRoot(root: EvaluatedRoot): boolean {
  return (
    !root.winsImmediately &&
    !root.attacksOpponentDrainer &&
    !isTacticalPriorityRoot(root) &&
    !root.scoresSupermanaThisTurn &&
    !root.scoresOpponentManaThisTurn &&
    !root.safeSupermanaPickupNow &&
    !root.safeOpponentManaPickupNow &&
    root.sameTurnScoreWindowValue === 0
  );
}

function normalRootSafetyDeepFloorScore(
  execution: AutomoveExecutionContext,
  stateAfterMove: MonsGame,
  perspective: Color,
  config: AutomoveConfig,
  replyLimit: number,
): number {
  if (execution.session.checkpoint()) return 0;
  const winner = stateAfterMove.winnerColor();
  if (winner !== undefined) {
    return winner === perspective
      ? Math.trunc(SMART_TERMINAL_SCORE / 2)
      : -Math.trunc(SMART_TERMINAL_SCORE / 2);
  }
  if (stateAfterMove.activeColor === perspective) {
    return evaluateReplyRiskGame(
      execution,
      stateAfterMove,
      perspective,
      config,
    );
  }
  const clamp = (value: number, minimum: number, maximum: number): number =>
    Math.min(maximum, Math.max(minimum, Math.trunc(value)));
  const rootBranchLimit = clamp(config.search.nodeBranchLimit, 5, 12);
  const nodeBranchLimit = clamp(
    Math.max(0, config.search.nodeBranchLimit - 4),
    4,
    10,
  );
  const probe = patchAutomoveConfig(config, {
    budget: {
      depth: 1,
      maxVisitedNodes: clamp(config.budget.maxVisitedNodes / 18, 110, 360),
    },
    search: {
      rootBranchLimit,
      nodeBranchLimit,
      rootEnumerationLimit: clamp(rootBranchLimit * 3, rootBranchLimit, 48),
      nodeEnumerationLimit: clamp(nodeBranchLimit * 3, nodeBranchLimit, 36),
    },
  });
  const replies = enumerateLegalTransitions(
    execution,
    stateAfterMove,
    Math.max(1, Math.trunc(replyLimit)),
  );
  if (execution.session.cancelled) return 0;
  if (replies.length === 0) return Math.trunc(SMART_TERMINAL_SCORE / 4);

  let worst = MAX_SCORE;
  for (const reply of replies) {
    if (execution.session.checkpoint()) return 0;
    const replyWinner = reply.game.winnerColor();
    const score =
      replyWinner === perspective
        ? Math.trunc(SMART_TERMINAL_SCORE / 2)
        : replyWinner !== undefined
          ? -Math.trunc(SMART_TERMINAL_SCORE / 2)
          : evaluateSearchScore(execution, reply.game, perspective, 1, probe);
    if (execution.session.checkpoint()) return 0;
    worst = Math.min(worst, score);
  }
  return worst === MAX_SCORE
    ? evaluateReplyRiskGame(execution, stateAfterMove, perspective, config)
    : worst;
}

export function normalSafetyReplyOrder(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  candidate: EvaluatedRoot,
  candidateSnapshot: RootReplyRiskSnapshot,
  incumbent: EvaluatedRoot,
  incumbentSnapshot: RootReplyRiskSnapshot,
  perspective: Color,
  config: AutomoveConfig,
): number | undefined {
  if (
    !productionEnabled(config) ||
    !config.replyRisk.safetyRerank ||
    !quietNonTacticalReplyRiskRoot(candidate) ||
    !quietNonTacticalReplyRiskRoot(incumbent) ||
    Math.abs(candidate.rootRank - incumbent.rootRank) > 8 ||
    Math.abs(candidate.score - incumbent.score) > 160 ||
    Math.abs(
      candidateSnapshot.worstReplyScore - incumbentSnapshot.worstReplyScore,
    ) > 320
  ) {
    return undefined;
  }
  const progressOrSetupPair =
    hasProgressSurface(candidate) ||
    hasProgressSurface(incumbent) ||
    candidate.spiritOwnManaSetupNow ||
    incumbent.spiritOwnManaSetupNow ||
    candidate.spiritSameTurnScoreSetupNow ||
    incumbent.spiritSameTurnScoreSetupNow ||
    sameNonTacticalProgressLane(candidate, incumbent) ||
    sameOpeningSafeSetupPair(candidate, incumbent, config);
  if (!progressOrSetupPair) return undefined;
  const replyLimit = Math.min(
    36,
    Math.max(12, Math.trunc(config.search.nodeEnumerationLimit)),
  );
  const myScoreBefore = scoreForColor(game, perspective);
  const candidateNormal = normalRootSafetySnapshot(
    execution,
    candidate.game,
    perspective,
    myScoreBefore,
    config,
    replyLimit,
  );
  const incumbentNormal = normalRootSafetySnapshot(
    execution,
    incumbent.game,
    perspective,
    myScoreBefore,
    config,
    replyLimit,
  );
  const axesDiffer =
    candidateNormal.allowsImmediateOpponentWin !==
      incumbentNormal.allowsImmediateOpponentWin ||
    candidateNormal.opponentReachesMatchPoint !==
      incumbentNormal.opponentReachesMatchPoint ||
    candidateNormal.opponentMaxScoreGain !==
      incumbentNormal.opponentMaxScoreGain ||
    candidateNormal.myScoreGain !== incumbentNormal.myScoreGain ||
    candidateNormal.worstReplyScore !== incumbentNormal.worstReplyScore;
  if (axesDiffer) {
    if (
      betterNormalRootSafetyCandidate(
        candidateNormal,
        candidate.score,
        incumbentNormal,
        incumbent.score,
      )
    ) {
      return 1;
    }
    if (
      betterNormalRootSafetyCandidate(
        incumbentNormal,
        incumbent.score,
        candidateNormal,
        candidate.score,
      )
    ) {
      return -1;
    }
  }
  const mine = TARGET_SCORE - scoreForColor(game, perspective);
  const opponent =
    TARGET_SCORE -
    scoreForColor(
      game,
      perspective === Color.White ? Color.Black : Color.White,
    );
  if (config.replyRisk.deepSafetyFloor && (mine <= 3 || opponent <= 3)) {
    const floor = (state: MonsGame): number =>
      normalRootSafetyDeepFloorScore(
        execution,
        state,
        perspective,
        config,
        replyLimit,
      );
    const candidateFloor = floor(candidate.game);
    const incumbentFloor = floor(incumbent.game);
    if (candidateFloor !== incumbentFloor) {
      return candidateFloor > incumbentFloor ? 1 : -1;
    }
  }
  return undefined;
}
