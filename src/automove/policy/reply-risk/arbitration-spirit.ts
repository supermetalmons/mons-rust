import type { AutomoveExecutionContext } from "../../core/execution-context.js";
import type { EvaluatedRoot } from "../../root/types.js";
import type { AutomoveConfig } from "../../config/types.js";
import { TurnPlanFamily } from "../../turn/model.js";
import { compareTacticalEvaluatedRoots } from "../../root/evaluated-ordering.js";
import { spiritFollowupFloorOrder } from "./followup-ordering.js";
import { rankedRootOrder } from "./shortlist.js";
import { safePlainSpiritReplyRiskPair } from "./sibling-ordering.js";
import {
  compareSpiritProjectionPlans,
  mixedPlainSpiritProjectionOrder,
  mixedPlainSpiritReplyFloorOrder,
} from "./spirit-ordering.js";
import type {
  FullReplyRiskComparisonContext,
  ReplyRiskComparisonContext,
  RootReplyRiskSnapshot,
} from "./types.js";

export function plainSpiritReplyRiskDecision(
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
  if (
    !safePlainSpiritReplyRiskPair(
      candidate,
      candidateSnapshot,
      incumbent,
      incumbentSnapshot,
      config,
    )
  ) {
    return undefined;
  }
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
    if (fullContext !== undefined) {
      const projectionOrder = mixedPlainSpiritProjectionOrder(
        execution,
        fullContext.game,
        fullContext.evaluations,
        fullContext.candidateIndex,
        context.candidateProjection,
        fullContext.incumbentIndex,
        context.incumbentProjection,
        fullContext.perspective,
        config,
        followupScores,
      );
      if (projectionOrder !== undefined) return projectionOrder > 0;
    }
    const anySpiritImpact =
      context.candidateProjection.plan.headFamily === TurnPlanFamily.SpiritImpact ||
      context.incumbentProjection.plan.headFamily === TurnPlanFamily.SpiritImpact;
    if (anySpiritImpact) {
      const projectionOrder = compareSpiritProjectionPlans(
        context.candidateProjection,
        context.incumbentProjection,
        Math.abs(candidate.score - incumbent.score) <= 192,
      );
      if (projectionOrder !== 0) return projectionOrder > 0;
    }
  }
  if (fullContext !== undefined) {
    const followupOrder = spiritFollowupFloorOrder(
      execution,
      fullContext.game,
      fullContext.evaluations,
      fullContext.candidateIndex,
      fullContext.incumbentIndex,
      fullContext.perspective,
      config,
      followupScores,
    );
    if (followupOrder !== undefined && followupOrder !== 0) {
      return followupOrder > 0;
    }
    return (
      rankedRootOrder(
        fullContext.evaluations,
        fullContext.candidateIndex,
        fullContext.incumbentIndex,
      ) > 0
    );
  }
  if (candidate.score !== incumbent.score) {
    return candidate.score > incumbent.score;
  }
  const tacticalOrder = compareTacticalEvaluatedRoots(candidate, incumbent);
  return (
    tacticalOrder < 0 ||
    (tacticalOrder === 0 && candidate.rootRank < incumbent.rootRank)
  );
}

export function sharedSpiritFollowupDecision(
  execution: AutomoveExecutionContext,
  config: AutomoveConfig,
  fullContext: FullReplyRiskComparisonContext | undefined,
  followupScores: Map<number, number>,
): boolean | undefined {
  if (fullContext === undefined) return undefined;
  const followupOrder = spiritFollowupFloorOrder(
    execution,
    fullContext.game,
    fullContext.evaluations,
    fullContext.candidateIndex,
    fullContext.incumbentIndex,
    fullContext.perspective,
    config,
    followupScores,
  );
  return followupOrder !== undefined && followupOrder !== 0
    ? followupOrder > 0
    : undefined;
}
