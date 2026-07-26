import type { AutomoveExecutionContext } from "../execution-context.js";
import {
  buildSpiritRootProjections,
  compareSpiritProjectionPlans,
  spiritFollowupFloorOrder,
  spiritProjectionChallengeOrder,
} from "../reply-risk.js";
import {
  ProductionComparisonPhase,
  ProductionCompetitionKind,
  compareRankedEvaluatedRootIndices,
  type ProductionComparisonResult,
  type ProductionCompetitionResult,
  type ProductionIndexSelectionResult,
  type ProductionRootPolicy,
  type ProductionRootReentryContext,
  type RootSelectionContext,
} from "../root-selector.js";
import { MIN_SCORE, saturatingScoreSubtract } from "../score-math.js";
import type { EvaluatedRoot } from "../search.js";
import { hasProgressSurface, productionEnabled } from "../selector-types.js";
import { compareTurnUtilities } from "../turn-engine.js";
import {
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
  spiritSetupCompetesWithRoot,
} from "./competition-policy.js";
import type { SpiritProjectionMap } from "./competition-policy.js";
import {
  plainSpiritClusterProgressReentry,
  riskyRecoveryReentry,
} from "./reentries.js";
import { rootUtility } from "./support.js";

const CONTINUE = Object.freeze({ kind: "continue" } as const);
const SELECT = Object.freeze({ kind: "select" } as const);

function competitionResult(selected: boolean): ProductionCompetitionResult {
  return selected ? SELECT : CONTINUE;
}

function comparisonResult(
  order: number | undefined,
): ProductionComparisonResult {
  return order === undefined ? CONTINUE : { kind: "compare", order };
}

function indexSelectionResult(
  indices: readonly number[],
): ProductionIndexSelectionResult {
  return indices.length === 0 ? CONTINUE : { kind: "select", indices };
}

function freezeRules<Rule extends object>(
  rules: readonly Rule[],
): readonly Readonly<Rule>[] {
  return Object.freeze(rules.map((rule) => Object.freeze(rule)));
}

export function buildRootPolicy(
  execution: AutomoveExecutionContext,
): ProductionRootPolicy {
  const projectionCache = new Map<string, SpiritProjectionMap>();
  const followupScores = new Map<number, number>();
  const projections = (context: RootSelectionContext) => {
    const key = `${context.perspective}:${context.candidateIndices.join(",")}`;
    const cached = projectionCache.get(key);
    if (cached !== undefined) return cached;
    const built = buildSpiritRootProjections(
      execution,
      context.roots,
      context.candidateIndices,
      context.perspective,
      context.config,
    );
    projectionCache.set(key, built);
    return built;
  };

  const safetyReentries = (
    context: ProductionRootReentryContext,
  ): readonly number[] => {
    const bestScore = Math.max(
      ...context.candidateIndices.map(
        (index) => context.roots[index]?.score ?? MIN_SCORE,
      ),
    );
    const margin = Math.max(context.config.policy.drainerSafetyScoreMargin, 0);
    const recoverySetupIndices =
      context.config.planner.enabled && productionEnabled(context.config)
        ? context.candidateIndices.filter((index) => {
            const root = context.roots[index];
            return (
              root !== undefined &&
              root.ownDrainerVulnerable &&
              !root.manaHandoffToOpponent &&
              !root.hasRoundtrip &&
              root.score + margin >= bestScore &&
              canChallengeSpiritPreferenceRootWithRecovery(
                root,
                context.perspective,
              )
            );
          })
        : [];
    const bestSafeUtility =
      productionEnabled(context.config) &&
      context.selectedIndices.length > 0 &&
      recoverySetupIndices.length > 0
        ? context.selectedIndices
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
            )
            .sort((left, right) => compareTurnUtilities(right, left))[0]
        : undefined;
    const bestSafeScore = Math.max(
      ...context.selectedIndices.map(
        (index) => context.roots[index]?.score ?? MIN_SCORE,
      ),
    );
    const result = [...context.selectedIndices];
    for (const index of recoverySetupIndices) {
      const root = context.roots[index];
      if (root === undefined) continue;
      const recoveryHeadSignal =
        root.attacksOpponentDrainer ||
        root.classes.drainerSafetyRecover ||
        root.sameTurnScoreWindowValue > 0 ||
        root.spiritSameTurnScoreSetupNow ||
        root.spiritOwnManaSetupNow ||
        hasProgressSurface(root);
      const recoveryScoreGap = saturatingScoreSubtract(
        bestSafeScore,
        root.score,
      );
      if (recoveryScoreGap > (recoveryHeadSignal ? 48 : 24)) continue;
      const utility = projectedRiskyRecoveryUtility(
        execution,
        context.game,
        root,
        context.perspective,
        context.config,
      );
      if (
        utility === undefined ||
        (bestSafeUtility !== undefined &&
          !projectedRecoveryUtilityCompetes(utility, bestSafeUtility))
      ) {
        continue;
      }
      if (!result.includes(index)) result.push(index);
    }
    const progress = safeProgressReentryAfterSafetyPrefilter(
      execution,
      context,
      result,
    );
    if (progress !== undefined && !result.includes(progress)) {
      result.push(progress);
      result.sort((left, right) =>
        compareRankedEvaluatedRootIndices(context.roots, left, right),
      );
    }
    return result.filter((index) => !context.selectedIndices.includes(index));
  };

  const competitionRules = freezeRules([
    {
      id: "competition.safe-progress",
      kind: ProductionCompetitionKind.SafeProgress,
      evaluate: (context: RootSelectionContext) =>
        competitionResult(safeProgressCompetition(context)),
    },
    {
      id: "competition.followup-progress",
      kind: ProductionCompetitionKind.FollowupProgress,
      evaluate: (context: RootSelectionContext) =>
        competitionResult(followupProgressCompetition(execution, context)),
    },
    {
      id: "competition.risky-score",
      kind: ProductionCompetitionKind.RiskyScore,
      evaluate: (context: RootSelectionContext) =>
        competitionResult(riskyScoreCompetition(context)),
    },
    {
      id: "competition.negative-deny",
      kind: ProductionCompetitionKind.NegativeDeny,
      evaluate: (context: RootSelectionContext) =>
        competitionResult(
          negativeDenyCompetition(context, projections(context)),
        ),
    },
    {
      id: "competition.score",
      kind: ProductionCompetitionKind.Score,
      evaluate: (context: RootSelectionContext) =>
        competitionResult(scoreCompetition(context)),
    },
    {
      id: "competition.projection",
      kind: ProductionCompetitionKind.Projection,
      evaluate: (context: RootSelectionContext) =>
        competitionResult(projectionCompetition(context, projections(context))),
    },
    {
      id: "competition.risky-recovery",
      kind: ProductionCompetitionKind.RiskyRecovery,
      evaluate: (context: RootSelectionContext) =>
        competitionResult(riskyRecoveryCompetition(execution, context)),
    },
  ] as const satisfies ProductionRootPolicy["competitionRules"]);

  const safetyReentryRules = freezeRules([
    {
      id: "safety-reentry.recovery-and-progress",
      select: (context: ProductionRootReentryContext) =>
        indexSelectionResult(safetyReentries(context)),
    },
  ] as const satisfies ProductionRootPolicy["safetyReentryRules"]);

  const finalReentryRules = freezeRules([
    {
      id: "final-reentry.plain-spirit-progress",
      select(context: ProductionRootReentryContext) {
        const index = plainSpiritClusterProgressReentry(
          execution,
          context.roots,
          context.candidateIndices,
          context.perspective,
          context.config,
        );
        return indexSelectionResult(index === undefined ? [] : [index]);
      },
    },
    {
      id: "final-reentry.risky-recovery",
      select(context: ProductionRootReentryContext) {
        const index = riskyRecoveryReentry(
          execution,
          context.game,
          context.roots,
          context.candidateIndices,
          context.perspective,
          context.config,
        );
        return indexSelectionResult(index === undefined ? [] : [index]);
      },
    },
  ] as const satisfies ProductionRootPolicy["finalReentryRules"]);

  const comparisonRules = freezeRules([
    {
      id: "comparison.spirit-setup",
      phase: ProductionComparisonPhase.SpiritSetup,
      compare(context) {
        return comparisonResult(
          spiritSetupCompetesWithRoot(
            execution,
            context,
            context.candidateIndex,
            context.incumbentIndex,
          )
            ? 1
            : -1,
        );
      },
    },
    {
      id: "comparison.projection-challenge",
      phase: ProductionComparisonPhase.ProjectionChallenge,
      compare(context) {
        const candidate = context.roots[context.candidateIndex];
        const incumbent = context.roots[context.incumbentIndex];
        if (candidate === undefined || incumbent === undefined) return CONTINUE;
        const projected = projections(context);
        return comparisonResult(
          spiritProjectionChallengeOrder(
            candidate,
            projected.get(context.candidateIndex),
            incumbent,
            projected.get(context.incumbentIndex),
          ),
        );
      },
    },
    {
      id: "comparison.projection",
      phase: ProductionComparisonPhase.Projection,
      compare(context) {
        const projected = projections(context);
        const candidate = projected.get(context.candidateIndex);
        const incumbent = projected.get(context.incumbentIndex);
        if (candidate === undefined && incumbent === undefined) return CONTINUE;
        if (candidate === undefined) return comparisonResult(-1);
        if (incumbent === undefined) return comparisonResult(1);
        const candidateRoot = context.roots[context.candidateIndex];
        const incumbentRoot = context.roots[context.incumbentIndex];
        return comparisonResult(
          compareSpiritProjectionPlans(
            candidate,
            incumbent,
            candidateRoot !== undefined &&
              incumbentRoot !== undefined &&
              Math.abs(candidateRoot.score - incumbentRoot.score) <= 192,
          ),
        );
      },
    },
    {
      id: "comparison.followup-floor",
      phase: ProductionComparisonPhase.FollowupFloor,
      compare(context) {
        return comparisonResult(
          spiritFollowupFloorOrder(
            execution,
            context.game,
            context.roots,
            context.candidateIndex,
            context.incumbentIndex,
            context.perspective,
            context.config,
            followupScores,
          ),
        );
      },
    },
  ] as const satisfies ProductionRootPolicy["comparisonRules"]);

  return Object.freeze({
    competitionRules,
    safetyReentryRules,
    finalReentryRules,
    comparisonRules,
  });
}
