import { Color } from "../../engine/domain.js";
import { MonsGame } from "../../engine/game.js";
import type { AutomoveExecutionContext } from "../execution-context.js";
import {
  buildSpiritRootProjections,
  canChallengeSpiritPreferenceRoot,
  canChallengeSpiritPreferenceRootWithRecovery,
  spiritFollowupFloorScore,
  spiritProjectionChallengeOrder,
} from "../reply-risk.js";
import { rootFamily as advisorRootFamily } from "../root-family.js";
import {
  compareRankedEvaluatedRootIndices,
  spiritScoreChallengeOrder,
} from "../root-selector.js";
import type { RootSelectionContext } from "../root-selector.js";
import { MIN_SCORE, saturatingScoreAdd } from "../score-math.js";
import type { EvaluatedRoot } from "../search.js";
import {
  hasConcreteScoreSurface,
  isPlainSpiritDevelopmentRoot,
  productionEnabled,
  rootIsUnsafe as advisorRootIsUnsafe,
} from "../selector-types.js";
import type { AutomoveConfig } from "../selector-types.js";
import {
  TurnPlanFamily,
  compareUtilityPrimaryAxes,
  utilityHasNonnegativeDenyGain,
  utilitySupportsFamilyFallback,
  utilitySupportsTemporaryRiskRecovery,
} from "../turn-engine.js";
import { productionSecondaryAnalysisLive } from "../turn-engine-config.js";
import { advisorRootIsSafe, memoizedByIndex, rootUtility } from "./support.js";

function spiritSetupCompetes(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  candidate: EvaluatedRoot,
  incumbent: EvaluatedRoot,
  perspective: Color,
  config: AutomoveConfig,
): boolean {
  if (
    incumbent.spiritDevelopment ||
    !(candidate.spiritOwnManaSetupNow || candidate.spiritSameTurnScoreSetupNow)
  ) {
    return true;
  }
  if (
    candidate.sameTurnScoreWindowValue > incumbent.sameTurnScoreWindowValue ||
    candidate.score >= incumbent.score
  ) {
    return true;
  }
  if (
    game.activeColor === Color.Black &&
    game.turnNumber <= 2 &&
    candidate.spiritOwnManaSetupNow &&
    !candidate.spiritSameTurnScoreSetupNow &&
    advisorRootFamily(incumbent) === TurnPlanFamily.ManaTempo &&
    !hasConcreteScoreSurface(candidate) &&
    !hasConcreteScoreSurface(incumbent) &&
    !candidate.attacksOpponentDrainer &&
    !incumbent.attacksOpponentDrainer &&
    advisorRootIsSafe(candidate) &&
    advisorRootIsSafe(incumbent) &&
    saturatingScoreAdd(candidate.score, 64) >= incumbent.score &&
    candidate.spiritSetupGain >=
      saturatingScoreAdd(incumbent.spiritSetupGain, 48) &&
    candidate.rootRank <= incumbent.rootRank
  ) {
    return true;
  }
  return (
    compareUtilityPrimaryAxes(
      rootUtility(execution, game, candidate, perspective, config),
      rootUtility(execution, game, incumbent, perspective, config),
    ) >= 0
  );
}

function spiritSetupCompetesWithRoot(
  execution: AutomoveExecutionContext,
  context: RootSelectionContext,
  candidateIndex: number,
  incumbentIndex: number,
): boolean {
  const candidate = context.roots[candidateIndex];
  const incumbent = context.roots[incumbentIndex];
  return (
    candidate !== undefined &&
    incumbent !== undefined &&
    spiritSetupCompetes(
      execution,
      context.game,
      candidate,
      incumbent,
      context.perspective,
      context.config,
    )
  );
}

function riskyScoreCompetition(context: RootSelectionContext): boolean {
  const spiritScores = context.candidateIndices
    .map((index) => context.roots[index])
    .filter(
      (root): root is EvaluatedRoot =>
        root !== undefined &&
        (root.spiritDevelopment ||
          root.spiritSameTurnScoreSetupNow ||
          root.spiritOwnManaSetupNow),
    )
    .map((root) => root.score);
  if (spiritScores.length === 0) return false;
  const bestSpiritScore = Math.max(...spiritScores);
  return context.candidateIndices.some((index) => {
    const root = context.roots[index];
    return (
      root !== undefined &&
      !root.spiritDevelopment &&
      !root.spiritSameTurnScoreSetupNow &&
      !root.spiritOwnManaSetupNow &&
      root.ownDrainerVulnerable &&
      !root.ownDrainerWalkVulnerable &&
      !root.manaHandoffToOpponent &&
      !root.hasRoundtrip &&
      (hasConcreteScoreSurface(root) ||
        root.attacksOpponentDrainer ||
        root.sameTurnScoreWindowValue > 0) &&
      root.score >= bestSpiritScore
    );
  });
}

function projectedRiskyRecoveryUtility(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  root: EvaluatedRoot,
  perspective: Color,
  config: AutomoveConfig,
): ReturnType<typeof rootUtility> | undefined {
  if (
    !config.planner.enabled ||
    !productionEnabled(config) ||
    !root.ownDrainerVulnerable ||
    !canChallengeSpiritPreferenceRootWithRecovery(root, perspective)
  ) {
    return undefined;
  }
  const utility = rootUtility(execution, game, root, perspective, config);
  return utilitySupportsTemporaryRiskRecovery(utility) ? utility : undefined;
}

function projectedRecoveryUtilityCompetes(
  candidate: ReturnType<typeof rootUtility>,
  incumbent: ReturnType<typeof rootUtility>,
): boolean {
  return (
    utilitySupportsTemporaryRiskRecovery(candidate) &&
    compareUtilityPrimaryAxes(candidate, incumbent) >= 0 &&
    utilitySupportsFamilyFallback(candidate, incumbent)
  );
}

function safeProgressCompetition(context: RootSelectionContext): boolean {
  if (!productionEnabled(context.config)) return false;
  const spiritEfficiencies = context.candidateIndices
    .map((index) => context.roots[index])
    .filter(
      (root): root is EvaluatedRoot =>
        root !== undefined &&
        root.spiritDevelopment &&
        !root.manaHandoffToOpponent,
    )
    .map((root) => root.efficiency);
  if (spiritEfficiencies.length === 0) return false;
  const bestSpiritEfficiency = Math.max(...spiritEfficiencies);
  return context.candidateIndices.some((index) => {
    const root = context.roots[index];
    return (
      root !== undefined &&
      !root.ownDrainerVulnerable &&
      !root.manaHandoffToOpponent &&
      !root.hasRoundtrip &&
      (root.supermanaProgress || root.opponentManaProgress) &&
      root.policyPriority > 0 &&
      root.efficiency >= bestSpiritEfficiency
    );
  });
}

function followupProgressCompetition(
  execution: AutomoveExecutionContext,
  context: RootSelectionContext,
): boolean {
  if (
    !productionSecondaryAnalysisLive(context.config) ||
    context.candidateIndices.length < 2
  ) {
    return false;
  }
  const spiritIndices = context.candidateIndices
    .filter((index) => {
      const root = context.roots[index];
      return (
        root !== undefined &&
        isPlainSpiritDevelopmentRoot(root) &&
        !advisorRootIsUnsafe(root) &&
        !root.manaHandoffToOpponent &&
        !root.hasRoundtrip
      );
    })
    .sort((left, right) =>
      compareRankedEvaluatedRootIndices(context.roots, left, right),
    )
    .slice(0, 3);
  if (spiritIndices.length === 0) return false;
  const bestSpiritRank = Math.min(
    ...spiritIndices.map(
      (index) => context.roots[index]?.rootRank ?? Number.MAX_SAFE_INTEGER,
    ),
  );
  const bestSpiritScore = Math.max(
    ...spiritIndices.map((index) => context.roots[index]?.score ?? MIN_SCORE),
  );
  const followup = memoizedByIndex((index): number => {
    const root = context.roots[index];
    return root === undefined
      ? MIN_SCORE
      : spiritFollowupFloorScore(
          execution,
          root.game,
          context.perspective,
          context.config,
        );
  });
  const bestSpiritFollowup = Math.max(...spiritIndices.map(followup));
  return context.candidateIndices.some((index) => {
    if (spiritIndices.includes(index)) return false;
    const root = context.roots[index];
    return (
      root !== undefined &&
      !advisorRootIsUnsafe(root) &&
      !root.spiritDevelopment &&
      !root.spiritSameTurnScoreSetupNow &&
      !root.spiritOwnManaSetupNow &&
      !root.ownDrainerVulnerable &&
      !root.manaHandoffToOpponent &&
      !root.hasRoundtrip &&
      root.rootRank <= bestSpiritRank + 2 &&
      saturatingScoreAdd(root.score, 32) >= bestSpiritScore &&
      root.policyPriority > 0 &&
      followup(index) >= saturatingScoreAdd(bestSpiritFollowup, 32)
    );
  });
}

function hasNonSpiritScoreCompetitionSurface(root: EvaluatedRoot): boolean {
  return (
    root.winsImmediately ||
    root.attacksOpponentDrainer ||
    root.scoresSupermanaThisTurn ||
    root.scoresOpponentManaThisTurn ||
    root.safeSupermanaPickupNow ||
    root.safeOpponentManaPickupNow ||
    root.sameTurnScoreWindowValue > 0
  );
}

function scoreCompetition(context: RootSelectionContext): boolean {
  if (!productionEnabled(context.config)) return false;
  const spiritIndices = context.candidateIndices
    .filter((index) => {
      const root = context.roots[index];
      return root !== undefined && isPlainSpiritDevelopmentRoot(root);
    })
    .sort((left, right) =>
      compareRankedEvaluatedRootIndices(context.roots, left, right),
    )
    .slice(0, 3);
  if (spiritIndices.length === 0) return false;
  return context.candidateIndices.some((index) => {
    if (spiritIndices.includes(index)) return false;
    const root = context.roots[index];
    return (
      root !== undefined &&
      hasNonSpiritScoreCompetitionSurface(root) &&
      spiritIndices.every((spiritIndex) => {
        const spirit = context.roots[spiritIndex];
        return (
          spirit !== undefined &&
          (spiritScoreChallengeOrder(root, spirit) ?? 0) > 0
        );
      })
    );
  });
}

type SpiritProjectionMap = ReturnType<typeof buildSpiritRootProjections>;

function projectedSpiritIndices(
  context: RootSelectionContext,
  projections: SpiritProjectionMap,
): number[] {
  return context.candidateIndices
    .filter((index) => {
      const root = context.roots[index];
      return (
        root !== undefined &&
        isPlainSpiritDevelopmentRoot(root) &&
        projections.has(index)
      );
    })
    .sort((left, right) =>
      compareRankedEvaluatedRootIndices(context.roots, left, right),
    );
}

function projectionCompetition(
  context: RootSelectionContext,
  projections: SpiritProjectionMap,
): boolean {
  if (
    !context.config.planner.enabled ||
    !productionEnabled(context.config) ||
    projections.size < 2
  ) {
    return false;
  }
  const bestSpiritIndex = projectedSpiritIndices(context, projections)[0];
  const bestSpirit =
    bestSpiritIndex === undefined ? undefined : context.roots[bestSpiritIndex];
  const bestProjection =
    bestSpiritIndex === undefined
      ? undefined
      : projections.get(bestSpiritIndex);
  if (
    bestSpiritIndex === undefined ||
    bestSpirit === undefined ||
    bestProjection === undefined
  ) {
    return false;
  }
  return context.candidateIndices.some((index) => {
    if (index === bestSpiritIndex) return false;
    const root = context.roots[index];
    const projection = projections.get(index);
    if (root === undefined || projection === undefined) return false;
    const safeChallenger = canChallengeSpiritPreferenceRoot(
      root,
      context.perspective,
    );
    const riskyRecoveryChallenger =
      canChallengeSpiritPreferenceRootWithRecovery(root, context.perspective) &&
      projection.plan.headFamily === TurnPlanFamily.DrainerSafetyRecovery;
    return (
      (safeChallenger || riskyRecoveryChallenger) &&
      Math.abs(root.score - bestSpirit.score) <= 320 &&
      (spiritProjectionChallengeOrder(
        root,
        projection,
        bestSpirit,
        bestProjection,
      ) ?? 0) > 0
    );
  });
}

function negativeDenyCompetition(
  context: RootSelectionContext,
  projections: SpiritProjectionMap,
): boolean {
  if (!context.config.planner.enabled || !productionEnabled(context.config)) {
    return false;
  }
  const spiritIndices = projectedSpiritIndices(context, projections).slice(
    0,
    3,
  );
  if (
    spiritIndices.length === 0 ||
    spiritIndices.some((index) => {
      const projection = projections.get(index);
      return (
        projection !== undefined &&
        utilityHasNonnegativeDenyGain(projection.plan.utility)
      );
    })
  ) {
    return false;
  }
  const bestSpiritScore = Math.max(
    ...spiritIndices.map((index) => context.roots[index]?.score ?? MIN_SCORE),
  );
  return context.candidateIndices.some((index) => {
    if (spiritIndices.includes(index)) return false;
    const root = context.roots[index];
    return (
      root !== undefined &&
      !root.spiritDevelopment &&
      !root.spiritSameTurnScoreSetupNow &&
      !root.spiritOwnManaSetupNow &&
      !root.ownDrainerVulnerable &&
      !root.manaHandoffToOpponent &&
      !root.hasRoundtrip &&
      root.score >= bestSpiritScore
    );
  });
}

function riskyRecoveryCompetition(
  execution: AutomoveExecutionContext,
  context: RootSelectionContext,
): boolean {
  if (!context.config.planner.enabled || !productionEnabled(context.config)) {
    return false;
  }
  const spiritIndices = context.candidateIndices
    .filter((index) => {
      const root = context.roots[index];
      return (
        root !== undefined &&
        (root.spiritDevelopment ||
          root.spiritSameTurnScoreSetupNow ||
          root.spiritOwnManaSetupNow)
      );
    })
    .sort((left, right) =>
      compareRankedEvaluatedRootIndices(context.roots, left, right),
    )
    .slice(0, 3);
  if (spiritIndices.length === 0) return false;
  const bestSpiritScore = Math.max(
    ...spiritIndices.map((index) => context.roots[index]?.score ?? MIN_SCORE),
  );
  const spiritUtilities = spiritIndices
    .map((index) => context.roots[index])
    .filter((root): root is EvaluatedRoot => root !== undefined)
    .map((root) =>
      rootUtility(
        execution,
        context.game,
        root,
        context.perspective,
        context.config,
      ),
    );
  if (spiritUtilities.length === 0) return false;
  return context.candidateIndices.some((index) => {
    const root = context.roots[index];
    if (
      root === undefined ||
      root.spiritDevelopment ||
      root.spiritSameTurnScoreSetupNow ||
      root.spiritOwnManaSetupNow ||
      !root.ownDrainerVulnerable ||
      root.manaHandoffToOpponent ||
      root.hasRoundtrip ||
      root.score < bestSpiritScore
    ) {
      return false;
    }
    const utility = projectedRiskyRecoveryUtility(
      execution,
      context.game,
      root,
      context.perspective,
      context.config,
    );
    return (
      utility !== undefined &&
      spiritUtilities.every((spiritUtility) =>
        projectedRecoveryUtilityCompetes(utility, spiritUtility),
      )
    );
  });
}

function safeProgressReentryAfterSafetyPrefilter(
  execution: AutomoveExecutionContext,
  context: RootSelectionContext,
  keptIndices: readonly number[],
): number | undefined {
  if (
    !productionSecondaryAnalysisLive(context.config) ||
    keptIndices.length === 0 ||
    !keptIndices.every((index) => {
      const root = context.roots[index];
      return (
        root !== undefined &&
        isPlainSpiritDevelopmentRoot(root) &&
        !advisorRootIsUnsafe(root) &&
        !root.manaHandoffToOpponent &&
        !root.hasRoundtrip
      );
    })
  ) {
    return undefined;
  }
  const bestKeptRank = Math.min(
    ...keptIndices.map(
      (index) => context.roots[index]?.rootRank ?? Number.MAX_SAFE_INTEGER,
    ),
  );
  const bestKeptScore = Math.max(
    ...keptIndices.map((index) => context.roots[index]?.score ?? MIN_SCORE),
  );
  const followup = memoizedByIndex((index): number => {
    const root = context.roots[index];
    return root === undefined
      ? MIN_SCORE
      : spiritFollowupFloorScore(
          execution,
          root.game,
          context.perspective,
          context.config,
        );
  });
  const bestKeptFollowup = Math.max(...keptIndices.map(followup));
  return context.candidateIndices
    .filter((index) => !keptIndices.includes(index))
    .filter((index) => {
      const root = context.roots[index];
      return (
        root !== undefined &&
        !advisorRootIsUnsafe(root) &&
        !root.ownDrainerVulnerable &&
        !root.ownDrainerWalkVulnerable &&
        !root.manaHandoffToOpponent &&
        !root.hasRoundtrip &&
        !root.spiritDevelopment &&
        !root.spiritSameTurnScoreSetupNow &&
        !root.spiritOwnManaSetupNow &&
        root.policyPriority > 0 &&
        root.rootRank <= bestKeptRank + 2 &&
        saturatingScoreAdd(root.score, 32) >= bestKeptScore &&
        followup(index) >= saturatingScoreAdd(bestKeptFollowup, 32)
      );
    })
    .sort((left, right) => {
      const leftFollowup = followup(left);
      const rightFollowup = followup(right);
      if (leftFollowup !== rightFollowup) {
        return leftFollowup > rightFollowup ? -1 : 1;
      }
      return compareRankedEvaluatedRootIndices(context.roots, left, right);
    })[0];
}

export {
  canChallengeSpiritPreferenceRootWithRecovery,
  followupProgressCompetition,
  negativeDenyCompetition,
  projectedRecoveryUtilityCompetes,
  projectedRiskyRecoveryUtility,
  projectionCompetition,
  riskyRecoveryCompetition,
  riskyScoreCompetition,
  safeProgressCompetition,
  safeProgressReentryAfterSafetyPrefilter,
  scoreCompetition,
  spiritSetupCompetes,
  spiritSetupCompetesWithRoot,
};
export type { SpiritProjectionMap };
