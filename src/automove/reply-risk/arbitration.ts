import type { AutomoveExecutionContext } from "../execution-context.js";
import type { EvaluatedRoot } from "../search.js";
import {
  isPlainSpiritDevelopmentRoot,
  productionEnabled,
} from "../selector-types.js";
import type { AutomoveConfig } from "../selector-types.js";
import {
  TurnPlanFamily,
  turnEngineComparePlans,
  utilityHasNonnegativeDenyGain,
} from "../turn-engine.js";
import { isTacticalTurnEngineFamily } from "./projection.js";
import {
  compareTacticalRoots,
  progressStepsBetter,
  rankedRootOrder,
  scorePathStepsBetter,
} from "./ranking.js";
import type {
  ReplyRiskComparisonContext,
  RootReplyRiskSnapshot,
} from "./types.js";
import {
  blackPlainSpiritFollowupReplyOrder,
  earlyBlackManaProgressReplyOrder,
  earlyBlackPlainSpiritManaReplyOrder,
  earlyBlackPlainSpiritSiblingOrder,
  safeNonSpiritFollowupOrder,
  spiritFollowupFloorOrder,
  whiteSpiritFollowupSetupReplyOrder,
} from "./followup-ordering.js";
import {
  lateSafeManaRootOrder,
  normalSafetyReplyOrder,
} from "./safety-ordering.js";
import {
  riskyRecoveryProgressSiblingOrder,
  safePickupOrder,
  safePlainSpiritReplyRiskPair,
  safeProgressSiblingOrder,
  sameOpeningSafeSetupPair,
} from "./sibling-ordering.js";
import {
  compareSpiritProjectionPlans,
  mixedPlainSpiritProjectionOrder,
  mixedPlainSpiritReplyFloorOrder,
  spiritProjectionChallengeOrder,
  spiritScoreChallengeOrder,
} from "./spirit-ordering.js";

export function isBetterReplyRiskCandidate(
  execution: AutomoveExecutionContext,
  candidate: EvaluatedRoot,
  candidateSnapshot: RootReplyRiskSnapshot,
  incumbent: EvaluatedRoot,
  incumbentSnapshot: RootReplyRiskSnapshot,
  config: AutomoveConfig,
  context: ReplyRiskComparisonContext = {},
): boolean {
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
  const hasFullContext =
    context.game !== undefined &&
    context.evaluations !== undefined &&
    context.candidateIndex !== undefined &&
    context.incumbentIndex !== undefined &&
    context.perspective !== undefined;
  const followupScores =
    context.spiritFollowupScores ?? new Map<number, number>();
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
  if (hasFullContext) {
    const game = context.game;
    const evaluations = context.evaluations;
    const candidateIndex = context.candidateIndex;
    const incumbentIndex = context.incumbentIndex;
    const perspective = context.perspective;
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
      Math.abs(
        candidateSnapshot.worstReplyScore - incumbentSnapshot.worstReplyScore,
      ) <= 80
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
  if (
    safePlainSpiritReplyRiskPair(
      candidate,
      candidateSnapshot,
      incumbent,
      incumbentSnapshot,
      config,
    )
  ) {
    if (
      context.candidateProjection !== undefined &&
      context.incumbentProjection !== undefined &&
      Math.abs(candidate.score - incumbent.score) <= 16 &&
      candidate.efficiency === incumbent.efficiency
    ) {
      const floorOrder = mixedPlainSpiritReplyFloorOrder(
        candidateSnapshot,
        context.candidateProjection,
        incumbentSnapshot,
        context.incumbentProjection,
        config,
      );
      if (floorOrder !== undefined) return floorOrder > 0;
      if (hasFullContext) {
        const projectionOrder = mixedPlainSpiritProjectionOrder(
          execution,
          context.game,
          context.evaluations,
          context.candidateIndex,
          context.candidateProjection,
          context.incumbentIndex,
          context.incumbentProjection,
          context.perspective,
          config,
          followupScores,
        );
        if (projectionOrder !== undefined) return projectionOrder > 0;
      }
      const anySpiritImpact =
        context.candidateProjection.plan.headFamily ===
          TurnPlanFamily.SpiritImpact ||
        context.incumbentProjection.plan.headFamily ===
          TurnPlanFamily.SpiritImpact;
      if (anySpiritImpact) {
        const projectionOrder = compareSpiritProjectionPlans(
          context.candidateProjection,
          context.incumbentProjection,
          Math.abs(candidate.score - incumbent.score) <= 192,
        );
        if (projectionOrder !== 0) return projectionOrder > 0;
      }
    }
    if (hasFullContext) {
      const followupOrder = spiritFollowupFloorOrder(
        execution,
        context.game,
        context.evaluations,
        context.candidateIndex,
        context.incumbentIndex,
        context.perspective,
        config,
        followupScores,
      );
      if (followupOrder !== undefined && followupOrder !== 0) {
        return followupOrder > 0;
      }
      return (
        rankedRootOrder(
          context.evaluations,
          context.candidateIndex,
          context.incumbentIndex,
        ) > 0
      );
    }
    if (candidate.score !== incumbent.score) {
      return candidate.score > incumbent.score;
    }
    const tacticalOrder = compareTacticalRoots(candidate, incumbent);
    return (
      tacticalOrder < 0 ||
      (tacticalOrder === 0 && candidate.rootRank < incumbent.rootRank)
    );
  }
  if (hasFullContext) {
    const followupOrder = spiritFollowupFloorOrder(
      execution,
      context.game,
      context.evaluations,
      context.candidateIndex,
      context.incumbentIndex,
      context.perspective,
      config,
      followupScores,
    );
    if (followupOrder !== undefined && followupOrder !== 0) {
      return followupOrder > 0;
    }
  }
  if (
    productionEnabled(config) &&
    Math.abs(
      candidateSnapshot.worstReplyScore - incumbentSnapshot.worstReplyScore,
    ) > 240
  ) {
    return (
      candidateSnapshot.worstReplyScore > incumbentSnapshot.worstReplyScore
    );
  }
  if (
    productionEnabled(config) &&
    candidate.policyPriority !== incumbent.policyPriority &&
    candidate.efficiency >= incumbent.efficiency
  ) {
    return candidate.policyPriority > incumbent.policyPriority;
  }
  if (
    config.replyRisk.deterministicTiebreak &&
    candidate.spiritOwnManaSetupNow !== incumbent.spiritOwnManaSetupNow
  ) {
    const sameOpening = sameOpeningSafeSetupPair(candidate, incumbent, config);
    if (
      !productionEnabled(config) ||
      sameOpening ||
      Math.abs(
        candidateSnapshot.worstReplyScore - incumbentSnapshot.worstReplyScore,
      ) <= 120
    ) {
      return candidate.spiritOwnManaSetupNow;
    }
  }
  if (
    config.replyRisk.deterministicTiebreak &&
    candidate.spiritOwnManaSetupNow &&
    incumbent.spiritOwnManaSetupNow &&
    candidate.supermanaProgress &&
    incumbent.supermanaProgress &&
    candidate.safeSupermanaProgressSteps !==
      incumbent.safeSupermanaProgressSteps
  ) {
    return progressStepsBetter(
      candidate.safeSupermanaProgressSteps,
      incumbent.safeSupermanaProgressSteps,
    );
  }
  if (
    config.replyRisk.deterministicTiebreak &&
    candidate.spiritOwnManaSetupNow &&
    incumbent.spiritOwnManaSetupNow &&
    candidate.opponentManaProgress &&
    incumbent.opponentManaProgress &&
    candidate.safeOpponentManaProgressSteps !==
      incumbent.safeOpponentManaProgressSteps
  ) {
    return progressStepsBetter(
      candidate.safeOpponentManaProgressSteps,
      incumbent.safeOpponentManaProgressSteps,
    );
  }
  if (
    config.replyRisk.deterministicTiebreak &&
    candidate.spiritOwnManaSetupNow &&
    incumbent.spiritOwnManaSetupNow &&
    candidate.scorePathBestSteps !== incumbent.scorePathBestSteps
  ) {
    return scorePathStepsBetter(
      candidate.scorePathBestSteps,
      incumbent.scorePathBestSteps,
    );
  }
  if (
    config.replyRisk.deterministicTiebreak &&
    candidate.spiritDevelopment !== incumbent.spiritDevelopment
  ) {
    const projectionOrder = spiritProjectionChallengeOrder(
      candidate,
      context.candidateProjection,
      incumbent,
      context.incumbentProjection,
    );
    if (projectionOrder !== undefined) return projectionOrder > 0;
    const scoreOrder = spiritScoreChallengeOrder(candidate, incumbent);
    if (
      scoreOrder !== undefined &&
      (!productionEnabled(config) ||
        Math.abs(
          candidateSnapshot.worstReplyScore - incumbentSnapshot.worstReplyScore,
        ) <= 120)
    ) {
      return scoreOrder > 0;
    }
    if (
      !productionEnabled(config) ||
      Math.abs(
        candidateSnapshot.worstReplyScore - incumbentSnapshot.worstReplyScore,
      ) <= 120
    ) {
      return candidate.spiritDevelopment;
    }
  }
  if (
    context.candidateProjection !== undefined &&
    context.incumbentProjection !== undefined
  ) {
    const replyFloorClose =
      Math.abs(
        candidateSnapshot.worstReplyScore - incumbentSnapshot.worstReplyScore,
      ) <= 100;
    const noTerminalRisk =
      !candidateSnapshot.allowsImmediateOpponentWin &&
      !incumbentSnapshot.allowsImmediateOpponentWin &&
      !candidateSnapshot.opponentReachesMatchPoint &&
      !incumbentSnapshot.opponentReachesMatchPoint;
    const tacticalProjection =
      isTacticalTurnEngineFamily(context.candidateProjection.plan.headFamily) ||
      isTacticalTurnEngineFamily(context.incumbentProjection.plan.headFamily);
    const spiritPhase =
      candidate.spiritOwnManaSetupNow ||
      incumbent.spiritOwnManaSetupNow ||
      candidate.spiritSameTurnScoreSetupNow ||
      incumbent.spiritSameTurnScoreSetupNow ||
      candidate.spiritDevelopment ||
      incumbent.spiritDevelopment;
    const plainSpiritProjection =
      productionEnabled(config) &&
      isPlainSpiritDevelopmentRoot(candidate) &&
      isPlainSpiritDevelopmentRoot(incumbent) &&
      replyFloorClose &&
      utilityHasNonnegativeDenyGain(context.candidateProjection.plan.utility) &&
      utilityHasNonnegativeDenyGain(context.incumbentProjection.plan.utility) &&
      noTerminalRisk;
    if (plainSpiritProjection) {
      const order = compareSpiritProjectionPlans(
        context.candidateProjection,
        context.incumbentProjection,
        Math.abs(candidate.score - incumbent.score) <= 192,
      );
      if (order !== 0) return order > 0;
    }
    const spiritDevelopmentProjection =
      productionEnabled(config) &&
      candidate.spiritDevelopment &&
      incumbent.spiritDevelopment &&
      !candidate.spiritSameTurnScoreSetupNow &&
      !incumbent.spiritSameTurnScoreSetupNow &&
      !candidate.spiritOwnManaSetupNow &&
      !incumbent.spiritOwnManaSetupNow &&
      replyFloorClose &&
      noTerminalRisk;
    if (spiritDevelopmentProjection) {
      const order = compareSpiritProjectionPlans(
        context.candidateProjection,
        context.incumbentProjection,
        Math.abs(candidate.score - incumbent.score) <= 192,
      );
      if (order !== 0) return order > 0;
    }
    if (
      tacticalProjection &&
      noTerminalRisk &&
      replyFloorClose &&
      !spiritPhase
    ) {
      const order = turnEngineComparePlans(
        context.candidateProjection.plan,
        context.incumbentProjection.plan,
      );
      if (order !== 0) return order > 0;
    }
  }
  if (candidate.policyPriority !== incumbent.policyPriority) {
    return candidate.policyPriority > incumbent.policyPriority;
  }
  if (candidateSnapshot.worstReplyScore !== incumbentSnapshot.worstReplyScore) {
    return (
      candidateSnapshot.worstReplyScore > incumbentSnapshot.worstReplyScore
    );
  }
  if (candidate.score !== incumbent.score)
    return candidate.score > incumbent.score;
  if (candidate.efficiency !== incumbent.efficiency) {
    return candidate.efficiency > incumbent.efficiency;
  }
  return false;
}
