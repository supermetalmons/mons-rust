import type { EvaluatedRoot } from "../../root/types.js";
import { isPlainSpiritDevelopmentRoot, productionEnabled } from "../../config/types.js";
import type { AutomoveConfig } from "../../config/types.js";
import {
  turnEngineComparePlans,
  utilityHasNonnegativeDenyGain,
} from "../../turn/ordering.js";
import { rootProgressStepsBetter, rootScorePathStepsBetter } from "../../root/focus.js";
import { isTacticalTurnEngineFamily } from "./projection.js";
import { sameOpeningSafeSetupPair } from "./sibling-ordering.js";
import {
  compareSpiritProjectionPlans,
  spiritProjectionChallengeOrder,
} from "./spirit-ordering.js";
import { spiritScoreChallengeOrder } from "../../root/evaluated-ordering.js";
import type { ReplyRiskComparisonContext, RootReplyRiskSnapshot } from "./types.js";

export function finalReplyRiskDecision(
  candidate: EvaluatedRoot,
  candidateSnapshot: RootReplyRiskSnapshot,
  incumbent: EvaluatedRoot,
  incumbentSnapshot: RootReplyRiskSnapshot,
  config: AutomoveConfig,
  context: ReplyRiskComparisonContext,
): boolean {
  if (
    productionEnabled(config) &&
    Math.abs(candidateSnapshot.worstReplyScore - incumbentSnapshot.worstReplyScore) >
      240
  ) {
    return candidateSnapshot.worstReplyScore > incumbentSnapshot.worstReplyScore;
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
      Math.abs(candidateSnapshot.worstReplyScore - incumbentSnapshot.worstReplyScore) <=
        120
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
    candidate.safeSupermanaProgressSteps !== incumbent.safeSupermanaProgressSteps
  ) {
    return rootProgressStepsBetter(
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
    candidate.safeOpponentManaProgressSteps !== incumbent.safeOpponentManaProgressSteps
  ) {
    return rootProgressStepsBetter(
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
    return rootScorePathStepsBetter(
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
      Math.abs(candidateSnapshot.worstReplyScore - incumbentSnapshot.worstReplyScore) <=
        120
    ) {
      return candidate.spiritDevelopment;
    }
  }
  if (
    context.candidateProjection !== undefined &&
    context.incumbentProjection !== undefined
  ) {
    const replyFloorClose =
      Math.abs(candidateSnapshot.worstReplyScore - incumbentSnapshot.worstReplyScore) <=
      100;
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
    if (tacticalProjection && noTerminalRisk && replyFloorClose && !spiritPhase) {
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
    return candidateSnapshot.worstReplyScore > incumbentSnapshot.worstReplyScore;
  }
  if (candidate.score !== incumbent.score) return candidate.score > incumbent.score;
  if (candidate.efficiency !== incumbent.efficiency) {
    return candidate.efficiency > incumbent.efficiency;
  }
  return false;
}
