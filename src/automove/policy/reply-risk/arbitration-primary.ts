import type { AutomoveExecutionContext } from "../../core/execution-context.js";
import type { EvaluatedRoot } from "../../root/types.js";
import type { AutomoveConfig } from "../../config/types.js";
import {
  blackPlainSpiritFollowupReplyOrder,
  earlyBlackManaProgressReplyOrder,
  earlyBlackPlainSpiritManaReplyOrder,
  earlyBlackPlainSpiritSiblingOrder,
  safeNonSpiritFollowupOrder,
  whiteSpiritFollowupSetupReplyOrder,
} from "./followup-ordering.js";
import { lateSafeManaRootOrder, normalSafetyReplyOrder } from "./safety-ordering.js";
import {
  riskyRecoveryProgressSiblingOrder,
  safePickupOrder,
  safeProgressSiblingOrder,
} from "./sibling-ordering.js";
import type {
  FullReplyRiskComparisonContext,
  ReplyRiskComparisonContext,
  RootReplyRiskSnapshot,
} from "./types.js";

export function immediateReplyRiskDecision(
  candidate: EvaluatedRoot,
  candidateSnapshot: RootReplyRiskSnapshot,
  incumbent: EvaluatedRoot,
  incumbentSnapshot: RootReplyRiskSnapshot,
): boolean | undefined {
  if (candidate.winsImmediately !== incumbent.winsImmediately) {
    return candidate.winsImmediately;
  }
  if (candidate.attacksOpponentDrainer !== incumbent.attacksOpponentDrainer) {
    return candidate.attacksOpponentDrainer;
  }
  if (
    candidateSnapshot.allowsImmediateOpponentWin !==
    incumbentSnapshot.allowsImmediateOpponentWin
  ) {
    return !candidateSnapshot.allowsImmediateOpponentWin;
  }
  if (
    candidateSnapshot.opponentReachesMatchPoint !==
    incumbentSnapshot.opponentReachesMatchPoint
  ) {
    return !candidateSnapshot.opponentReachesMatchPoint;
  }
  return undefined;
}

export function contextualReplyRiskDecision(
  execution: AutomoveExecutionContext,
  candidate: EvaluatedRoot,
  candidateSnapshot: RootReplyRiskSnapshot,
  incumbent: EvaluatedRoot,
  incumbentSnapshot: RootReplyRiskSnapshot,
  config: AutomoveConfig,
  context: ReplyRiskComparisonContext,
  fullContext: FullReplyRiskComparisonContext | undefined,
  followupScores: Map<number, number>,
): boolean | undefined {
  if (context.game !== undefined) {
    const lateManaOrder = lateSafeManaRootOrder(
      context.game,
      candidate,
      candidateSnapshot,
      incumbent,
      incumbentSnapshot,
      config,
    );
    if (lateManaOrder !== undefined) return lateManaOrder > 0;
  }
  const riskyRecoveryOrder = riskyRecoveryProgressSiblingOrder(
    candidate,
    candidateSnapshot,
    incumbent,
    incumbentSnapshot,
    context.candidateProjection,
    context.incumbentProjection,
    config,
  );
  if (riskyRecoveryOrder !== undefined) return riskyRecoveryOrder > 0;
  const safeProgressOrder = safeProgressSiblingOrder(
    candidate,
    candidateSnapshot,
    incumbent,
    incumbentSnapshot,
    config,
  );
  if (safeProgressOrder !== undefined) return safeProgressOrder > 0;
  if (context.game !== undefined) {
    const manaProgressOrder = earlyBlackManaProgressReplyOrder(
      context.game,
      candidate,
      candidateSnapshot,
      incumbent,
      incumbentSnapshot,
      config,
    );
    if (manaProgressOrder !== undefined) return manaProgressOrder > 0;
  }
  if (fullContext !== undefined) {
    const game = fullContext.game;
    const evaluations = fullContext.evaluations;
    const candidateIndex = fullContext.candidateIndex;
    const incumbentIndex = fullContext.incumbentIndex;
    const perspective = fullContext.perspective;
    const spiritManaOrder = earlyBlackPlainSpiritManaReplyOrder(
      execution,
      game,
      evaluations,
      candidateIndex,
      candidateSnapshot,
      incumbentIndex,
      incumbentSnapshot,
      perspective,
      config,
      followupScores,
    );
    if (spiritManaOrder !== undefined) return spiritManaOrder > 0;
    const nonSpiritOrder = safeNonSpiritFollowupOrder(
      execution,
      game,
      evaluations,
      candidateIndex,
      candidateSnapshot,
      incumbentIndex,
      incumbentSnapshot,
      perspective,
      config,
      followupScores,
    );
    if (nonSpiritOrder !== undefined) return nonSpiritOrder > 0;
    const blackSetupOrder = blackPlainSpiritFollowupReplyOrder(
      execution,
      game,
      evaluations,
      candidateIndex,
      candidateSnapshot,
      incumbentIndex,
      incumbentSnapshot,
      perspective,
      config,
      followupScores,
    );
    if (blackSetupOrder !== undefined) return blackSetupOrder > 0;
    const earlySiblingOrder = earlyBlackPlainSpiritSiblingOrder(
      execution,
      game,
      evaluations,
      candidateIndex,
      candidateSnapshot,
      incumbentIndex,
      incumbentSnapshot,
      perspective,
      config,
      followupScores,
    );
    if (earlySiblingOrder !== undefined) return earlySiblingOrder > 0;
    const whiteSetupOrder = whiteSpiritFollowupSetupReplyOrder(
      game,
      candidate,
      candidateSnapshot,
      incumbent,
      incumbentSnapshot,
      config,
    );
    if (whiteSetupOrder !== undefined) return whiteSetupOrder > 0;
  }
  const pickupOrder = safePickupOrder(
    candidate,
    candidateSnapshot,
    incumbent,
    incumbentSnapshot,
    config,
  );
  if (pickupOrder !== undefined) return pickupOrder > 0;
  if (context.game !== undefined && context.perspective !== undefined) {
    const safetyOrder = normalSafetyReplyOrder(
      execution,
      context.game,
      candidate,
      candidateSnapshot,
      incumbent,
      incumbentSnapshot,
      context.perspective,
      config,
    );
    if (safetyOrder !== undefined) return safetyOrder > 0;
  }
  const candidateProgressAdvantage =
    candidate.classes.carrierProgress && !incumbent.classes.carrierProgress;
  const incumbentProgressAdvantage =
    incumbent.classes.carrierProgress && !candidate.classes.carrierProgress;
  if (candidateProgressAdvantage || incumbentProgressAdvantage) {
    const noTacticalPriority =
      !candidate.classes.immediateScore &&
      !candidate.classes.drainerAttack &&
      !candidate.classes.drainerSafetyRecover &&
      !incumbent.classes.immediateScore &&
      !incumbent.classes.drainerAttack &&
      !incumbent.classes.drainerSafetyRecover;
    if (
      noTacticalPriority &&
      Math.abs(candidateSnapshot.worstReplyScore - incumbentSnapshot.worstReplyScore) <=
        80
    ) {
      return candidateProgressAdvantage;
    }
  }
  if (config.replyRisk.preferCleanRoots) {
    if (candidate.manaHandoffToOpponent !== incumbent.manaHandoffToOpponent) {
      return !candidate.manaHandoffToOpponent;
    }
    if (candidate.hasRoundtrip !== incumbent.hasRoundtrip) {
      return !candidate.hasRoundtrip;
    }
  }
  return undefined;
}
