import { Color } from "../../engine/domain.js";
import type { MonsGame } from "../../engine/game.js";
import type { AutomoveExecutionContext } from "../execution-context.js";
import { spiritScoreChallengeOrder } from "../root-selector.js";
import { saturatingScoreAdd } from "../score-math.js";
import type { EvaluatedRoot } from "../search.js";
import {
  isPlainSpiritDevelopmentRoot,
  productionEnabled,
  rootIsUnsafe as isUnsafe,
} from "../selector-types.js";
import type { AutomoveConfig } from "../selector-types.js";
import {
  TURN_PLAN_FAMILY_PRIORITY_ORDER,
  TurnPlanFamily,
  compareUtilityPrimaryAxes,
  familyRank,
  turnEngineComparePlans,
  utilityPassesOverrideGuard,
  utilitySupportsFamilyFallback,
  utilitySupportsTemporaryRiskRecovery,
} from "../turn-engine.js";
import { productionSecondaryAnalysisLive } from "./config.js";
import { rankedRootOrder } from "./ranking.js";
import type {
  RootReplyRiskSnapshot,
  TurnEngineRootProjection,
} from "./types.js";
import {
  earlyBlackPlainSpiritSiblingOrder,
  spiritFollowupFloorOrder,
} from "./followup-ordering.js";

export { spiritScoreChallengeOrder };

function closeSpiritGoalFamilyPriority(family: TurnPlanFamily): number {
  return TURN_PLAN_FAMILY_PRIORITY_ORDER.length - familyRank(family) - 1;
}

function compareCloseSpiritGoalFamily(
  candidate: TurnEngineRootProjection,
  incumbent: TurnEngineRootProjection,
): number {
  if (
    candidate.plan.headFamily !== TurnPlanFamily.SpiritImpact ||
    incumbent.plan.headFamily !== TurnPlanFamily.SpiritImpact
  ) {
    return 0;
  }
  return (
    closeSpiritGoalFamilyPriority(candidate.plan.goalFamily) -
    closeSpiritGoalFamilyPriority(incumbent.plan.goalFamily)
  );
}

export function compareSpiritProjectionPlans(
  candidate: TurnEngineRootProjection,
  incumbent: TurnEngineRootProjection,
  closeSpiritRoots: boolean,
): number {
  if (closeSpiritRoots) {
    let order = compareUtilityPrimaryAxes(
      candidate.plan.headUtility,
      incumbent.plan.headUtility,
    );
    if (order !== 0) return order;
    order = compareUtilityPrimaryAxes(
      candidate.plan.utility,
      incumbent.plan.utility,
    );
    if (order !== 0) return order;
    const candidateImpact =
      candidate.plan.headFamily === TurnPlanFamily.SpiritImpact;
    const incumbentImpact =
      incumbent.plan.headFamily === TurnPlanFamily.SpiritImpact;
    if (candidateImpact !== incumbentImpact) return candidateImpact ? 1 : -1;
    order = compareCloseSpiritGoalFamily(candidate, incumbent);
    if (order !== 0) return order;
  }
  return turnEngineComparePlans(candidate.plan, incumbent.plan);
}

export function mixedPlainSpiritReplyFloorOrder(
  candidateSnapshot: RootReplyRiskSnapshot,
  candidateProjection: TurnEngineRootProjection,
  incumbentSnapshot: RootReplyRiskSnapshot,
  incumbentProjection: TurnEngineRootProjection,
  config: AutomoveConfig,
): number | undefined {
  if (!productionEnabled(config)) return undefined;
  const candidateImpact =
    candidateProjection.plan.headFamily === TurnPlanFamily.SpiritImpact;
  const incumbentImpact =
    incumbentProjection.plan.headFamily === TurnPlanFamily.SpiritImpact;
  if (candidateImpact === incumbentImpact) return undefined;
  const nonSpiritSnapshot = candidateImpact
    ? incumbentSnapshot
    : candidateSnapshot;
  const nonSpiritProjection = candidateImpact
    ? incumbentProjection
    : candidateProjection;
  const spiritSnapshot = candidateImpact
    ? candidateSnapshot
    : incumbentSnapshot;
  const spiritProjection = candidateImpact
    ? candidateProjection
    : incumbentProjection;
  const spiritPriority = closeSpiritGoalFamilyPriority(
    spiritProjection.plan.goalFamily,
  );
  const nonSpiritPriority = closeSpiritGoalFamilyPriority(
    nonSpiritProjection.plan.goalFamily,
  );
  if (
    spiritPriority >
      closeSpiritGoalFamilyPriority(TurnPlanFamily.SpiritImpact) ||
    nonSpiritPriority < spiritPriority ||
    nonSpiritSnapshot.worstReplyScore <
      saturatingScoreAdd(spiritSnapshot.worstReplyScore, 128)
  ) {
    return undefined;
  }
  const utilityOrder = compareUtilityPrimaryAxes(
    nonSpiritProjection.plan.utility,
    spiritProjection.plan.utility,
  );
  if (
    utilityOrder < 0 &&
    !utilitySupportsFamilyFallback(
      nonSpiritProjection.plan.utility,
      spiritProjection.plan.utility,
    )
  ) {
    return undefined;
  }
  return candidateImpact ? -1 : 1;
}

export function mixedPlainSpiritProjectionOrder(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  evaluations: readonly EvaluatedRoot[],
  candidateIndex: number,
  candidateProjection: TurnEngineRootProjection,
  incumbentIndex: number,
  incumbentProjection: TurnEngineRootProjection,
  perspective: Color,
  config: AutomoveConfig,
  followupScores: Map<number, number>,
): number | undefined {
  if (!productionSecondaryAnalysisLive(config)) return undefined;
  const candidateImpact =
    candidateProjection.plan.headFamily === TurnPlanFamily.SpiritImpact;
  const incumbentImpact =
    incumbentProjection.plan.headFamily === TurnPlanFamily.SpiritImpact;
  if (candidateImpact === incumbentImpact) return undefined;

  const candidate = evaluations[candidateIndex];
  const incumbent = evaluations[incumbentIndex];
  if (candidate === undefined || incumbent === undefined) return undefined;
  const nonSpiritRoot = candidateImpact ? incumbent : candidate;
  const nonSpiritProjection = candidateImpact
    ? incumbentProjection
    : candidateProjection;
  const spiritRoot = candidateImpact ? candidate : incumbent;
  const spiritProjection = candidateImpact
    ? candidateProjection
    : incumbentProjection;
  const candidateIsNonSpirit = !candidateImpact;
  const followupOrder = spiritFollowupFloorOrder(
    execution,
    game,
    evaluations,
    candidateIndex,
    incumbentIndex,
    perspective,
    config,
    followupScores,
  );
  const nonSpiritIndex = candidateIsNonSpirit ? candidateIndex : incumbentIndex;
  const spiritIndex = candidateIsNonSpirit ? incumbentIndex : candidateIndex;
  const nonSpiritGoalPriority = closeSpiritGoalFamilyPriority(
    nonSpiritProjection.plan.goalFamily,
  );
  const spiritGoalPriority = closeSpiritGoalFamilyPriority(
    spiritProjection.plan.goalFamily,
  );
  const nonSpiritRootCompetes =
    nonSpiritGoalPriority >= spiritGoalPriority &&
    spiritGoalPriority <=
      closeSpiritGoalFamilyPriority(TurnPlanFamily.SpiritImpact) &&
    nonSpiritRoot.score >= spiritRoot.score &&
    rankedRootOrder(evaluations, nonSpiritIndex, spiritIndex) > 0 &&
    compareUtilityPrimaryAxes(
      nonSpiritProjection.plan.utility,
      spiritProjection.plan.utility,
    ) >= 0;
  if (nonSpiritRootCompetes) return candidateIsNonSpirit ? 1 : -1;

  const spiritGoalCompetes =
    spiritGoalPriority > nonSpiritGoalPriority &&
    compareUtilityPrimaryAxes(
      spiritProjection.plan.utility,
      nonSpiritProjection.plan.utility,
    ) >= 0 &&
    compareUtilityPrimaryAxes(
      spiritProjection.plan.headUtility,
      nonSpiritProjection.plan.headUtility,
    ) >= 0;
  if (spiritGoalCompetes) return candidateIsNonSpirit ? -1 : 1;

  const nonSpiritCompetes =
    (nonSpiritProjection.plan.headFamily ===
      TurnPlanFamily.SafeSupermanaProgress ||
      nonSpiritProjection.plan.headFamily ===
        TurnPlanFamily.SafeOpponentManaProgress ||
      nonSpiritProjection.plan.headFamily ===
        TurnPlanFamily.DrainerSafetyRecovery) &&
    (followupOrder === 0 ||
      utilitySupportsFamilyFallback(
        nonSpiritProjection.plan.utility,
        spiritProjection.plan.utility,
      )) &&
    nonSpiritRoot.score >= spiritRoot.score &&
    turnEngineComparePlans(nonSpiritProjection.plan, spiritProjection.plan) > 0;
  if (nonSpiritCompetes) return candidateIsNonSpirit ? 1 : -1;
  return candidateImpact ? 1 : -1;
}

export function plainSpiritReplyRiskPick(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  evaluations: readonly EvaluatedRoot[],
  shortlist: readonly number[],
  snapshots: ReadonlyMap<number, RootReplyRiskSnapshot>,
  projections: ReadonlyMap<number, TurnEngineRootProjection>,
  perspective: Color,
  config: AutomoveConfig,
  followupScores: Map<number, number>,
): number | undefined {
  if (!productionEnabled(config) || shortlist.length < 2) return undefined;
  const spiritShortlist = shortlist.filter((index) => {
    const root = evaluations[index];
    return root !== undefined && isPlainSpiritDevelopmentRoot(root);
  });
  if (
    spiritShortlist.length < 2 ||
    spiritShortlist.some((index) => !projections.has(index))
  ) {
    return undefined;
  }

  let bestIndex = spiritShortlist[0];
  if (bestIndex === undefined) return undefined;
  for (const index of spiritShortlist.slice(1)) {
    const candidate = evaluations[index];
    const incumbent = evaluations[bestIndex];
    const candidateProjection = projections.get(index);
    const incumbentProjection = projections.get(bestIndex);
    const candidateSnapshot = snapshots.get(index);
    const incumbentSnapshot = snapshots.get(bestIndex);
    if (
      candidate === undefined ||
      incumbent === undefined ||
      candidateProjection === undefined ||
      incumbentProjection === undefined ||
      candidateSnapshot === undefined ||
      incumbentSnapshot === undefined
    ) {
      continue;
    }

    let order = earlyBlackPlainSpiritSiblingOrder(
      execution,
      game,
      evaluations,
      index,
      candidateSnapshot,
      bestIndex,
      incumbentSnapshot,
      perspective,
      config,
      followupScores,
    );
    order ??= mixedPlainSpiritReplyFloorOrder(
      candidateSnapshot,
      candidateProjection,
      incumbentSnapshot,
      incumbentProjection,
      config,
    );
    order ??= mixedPlainSpiritProjectionOrder(
      execution,
      game,
      evaluations,
      index,
      candidateProjection,
      bestIndex,
      incumbentProjection,
      perspective,
      config,
      followupScores,
    );
    if (order === undefined) {
      const bothSpiritImpact =
        candidateProjection.plan.headFamily === TurnPlanFamily.SpiritImpact &&
        incumbentProjection.plan.headFamily === TurnPlanFamily.SpiritImpact;
      if (bothSpiritImpact) {
        order = compareSpiritProjectionPlans(
          candidateProjection,
          incumbentProjection,
          Math.abs(candidate.score - incumbent.score) <= 192,
        );
      } else {
        order = spiritFollowupFloorOrder(
          execution,
          game,
          evaluations,
          index,
          bestIndex,
          perspective,
          config,
          followupScores,
        );
        order ??= rankedRootOrder(evaluations, index, bestIndex);
      }
    }
    if (order > 0) bestIndex = index;
  }
  return bestIndex;
}

export function spiritProjectionChallengeOrder(
  candidate: EvaluatedRoot,
  candidateProjection: TurnEngineRootProjection | undefined,
  incumbent: EvaluatedRoot,
  incumbentProjection: TurnEngineRootProjection | undefined,
): number | undefined {
  const candidatePlain = isPlainSpiritDevelopmentRoot(candidate);
  const incumbentPlain = isPlainSpiritDevelopmentRoot(incumbent);
  if (
    candidatePlain === incumbentPlain ||
    candidateProjection === undefined ||
    incumbentProjection === undefined
  ) {
    return undefined;
  }
  const challenger = candidatePlain ? incumbent : candidate;
  const challengerProjection = candidatePlain
    ? incumbentProjection
    : candidateProjection;
  const spirit = candidatePlain ? candidate : incumbent;
  const spiritProjection = candidatePlain
    ? candidateProjection
    : incumbentProjection;
  const challengerUnsafe = isUnsafe(challenger);
  const spiritUnsafe = isUnsafe(spirit);
  const recoveryChallenge =
    challengerUnsafe &&
    !spiritUnsafe &&
    ((utilitySupportsTemporaryRiskRecovery(challengerProjection.plan.utility) &&
      compareUtilityPrimaryAxes(
        challengerProjection.plan.utility,
        spiritProjection.plan.utility,
      ) >= 0 &&
      utilitySupportsFamilyFallback(
        challengerProjection.plan.utility,
        spiritProjection.plan.utility,
      )) ||
      (challengerProjection.plan.goalFamily === TurnPlanFamily.ImmediateScore &&
        compareUtilityPrimaryAxes(
          challengerProjection.plan.utility,
          spiritProjection.plan.utility,
        ) >= 0));
  if (challengerUnsafe && !spiritUnsafe && !recoveryChallenge) {
    return undefined;
  }
  if (recoveryChallenge) return candidatePlain ? -1 : 1;
  if (
    challengerProjection.plan.headFamily === TurnPlanFamily.SpiritImpact ||
    !utilityPassesOverrideGuard(
      challengerProjection.plan.utility,
      spiritProjection.plan.utility,
    ) ||
    compareUtilityPrimaryAxes(
      challengerProjection.plan.headUtility,
      spiritProjection.plan.headUtility,
    ) < 0 ||
    compareUtilityPrimaryAxes(
      challengerProjection.plan.utility,
      spiritProjection.plan.utility,
    ) <= 0
  ) {
    return undefined;
  }
  return candidatePlain ? -1 : 1;
}
