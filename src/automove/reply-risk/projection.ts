import { Color } from "../../engine/domain.js";
import type { MonsGame } from "../../engine/game.js";
import type { AutomoveExecutionContext } from "../execution-context.js";
import type { RootCandidate } from "../root-candidates.js";
import type { EvaluatedRoot } from "../search.js";
import {
  hasProgressSurface,
  isPlainSpiritDevelopmentRoot,
  productionEnabled,
  rootIsUnsafe as isUnsafe,
  type AutomoveConfig,
} from "../selector-types.js";
import {
  TURN_PLAN_FAMILY_CACHE_TAG,
  TurnPlanFamily,
  compareTurnUtilities,
  turnEngineCandidatePlan,
  turnEngineEvaluatePlanWithReplies,
  turnEngineEvaluateStateUtility,
  type TurnEngineConfig,
  type TurnPlan,
  type TurnUtility,
} from "../turn-engine.js";
import {
  cachedSelectedOverrideUtility,
  cachedSpiritFollowupFloor,
  replyRiskCacheKey,
  selectedOverrideConfigKey,
  storeSelectedOverrideUtility,
  storeSpiritFollowupFloor,
} from "./cache.js";
import {
  fullTurnEngineConfig,
  productionSecondaryAnalysisLive,
  projectionTurnEngineConfig,
  rerankTurnEngineConfig,
  SMART_TERMINAL_SCORE,
} from "./config.js";
import {
  compareRankedReplyRiskEvaluations,
  safePlainSpiritCompetition,
} from "./ranking.js";
import { evaluateReplyRiskGame, rootReplyRiskSnapshot } from "./snapshot.js";
import {
  NO_REPLY_RISK_HOOKS,
  type ReplyRiskHooks,
  type RootReplyRiskSnapshot,
  type TurnEngineRootProjection,
} from "./types.js";

export function canTurnEngineProjectReplyRiskRoot(
  root: RootCandidate,
  perspective: Color,
): boolean {
  return (
    root.game.activeColor === perspective &&
    root.game.winnerColor() === undefined
  );
}

export function isTacticalTurnEngineFamily(family: TurnPlanFamily): boolean {
  return (
    family === TurnPlanFamily.ImmediateScore ||
    family === TurnPlanFamily.DenyOpponentWindow ||
    family === TurnPlanFamily.DrainerKill
  );
}

function isInformativeReplyRiskProjectionFamily(
  family: TurnPlanFamily,
): boolean {
  return (
    isTacticalTurnEngineFamily(family) ||
    family === TurnPlanFamily.SpiritImpact ||
    family === TurnPlanFamily.DrainerSafetyRecovery
  );
}

export function shouldUseReplyRiskProjectionForRoot(
  root: EvaluatedRoot,
  projection: TurnEngineRootProjection,
  perspective: Color,
  config: AutomoveConfig,
): boolean {
  if (
    !config.planner.enabled ||
    !productionEnabled(config) ||
    !canTurnEngineProjectReplyRiskRoot(root, perspective) ||
    isPlainSpiritDevelopmentRoot(root)
  ) {
    return false;
  }

  if (
    projection.plan.headFamily === TurnPlanFamily.SpiritImpact &&
    !root.spiritDevelopment &&
    root.ownDrainerVulnerable
  ) {
    return false;
  }

  if (isInformativeReplyRiskProjectionFamily(projection.plan.headFamily)) {
    return true;
  }

  return (
    (projection.plan.headFamily === TurnPlanFamily.SafeSupermanaProgress ||
      projection.plan.headFamily === TurnPlanFamily.SafeOpponentManaProgress) &&
    hasProgressSurface(root)
  );
}

export function rootReplyRiskSnapshotWithProjection(
  execution: AutomoveExecutionContext,
  root: EvaluatedRoot,
  projection: TurnEngineRootProjection | undefined,
  perspective: Color,
  config: AutomoveConfig,
  replyLimit: number,
): RootReplyRiskSnapshot {
  const state =
    projection !== undefined &&
    shouldUseReplyRiskProjectionForRoot(root, projection, perspective, config)
      ? projection.plan.endGame
      : root.game;
  return rootReplyRiskSnapshot(
    execution,
    state,
    perspective,
    config,
    replyLimit,
  );
}

function maxTurnUtility(left: TurnUtility, right: TurnUtility): TurnUtility {
  return compareTurnUtilities(left, right) >= 0 ? left : right;
}

function turnEngineRootPlanUtilityWithConfig(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  evaluation: RootCandidate,
  perspective: Color,
  engineConfig: TurnEngineConfig,
  family: TurnPlanFamily,
): TurnUtility {
  const headUtility = turnEngineEvaluateStateUtility(
    execution,
    evaluation.game,
    game,
    perspective,
    engineConfig,
  );
  const plan: TurnPlan = {
    actions: [],
    compiledChunks: [evaluation.inputs],
    endGame: evaluation.game.fork(),
    utility: headUtility,
    headUtility,
    headFamily: family,
    goalFamily: family,
    packageMeta: {
      scoreGain: 0,
      denyGain: 0,
      drainerSafetyDelta: 0,
      spiritOnlySetup: false,
      endsNonnegativeDrainerSafety: true,
      opponentImmediateWindowAfter: 0,
    },
  };
  return turnEngineEvaluatePlanWithReplies(
    execution,
    game,
    plan,
    perspective,
    engineConfig,
  );
}

export function turnEngineRootPlanUtility(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  evaluation: RootCandidate,
  perspective: Color,
  config: AutomoveConfig,
  family: TurnPlanFamily,
  hooks: ReplyRiskHooks = NO_REPLY_RISK_HOOKS,
): TurnUtility {
  const injected = hooks.evaluateTurnEngineRootUtility?.(
    game,
    evaluation,
    perspective,
    family,
  );
  return (
    injected ??
    turnEngineRootPlanUtilityWithConfig(
      execution,
      game,
      evaluation,
      perspective,
      fullTurnEngineConfig(config),
      family,
    )
  );
}

export function turnEngineSelectedOverrideUtility(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  evaluation: RootCandidate,
  perspective: Color,
  config: AutomoveConfig,
  family: TurnPlanFamily,
  hooks: ReplyRiskHooks = NO_REPLY_RISK_HOOKS,
): TurnUtility {
  const injected = hooks.evaluateTurnEngineRootUtility?.(
    game,
    evaluation,
    perspective,
    family,
  );
  if (injected !== undefined) return injected;
  const engineConfig = projectionTurnEngineConfig(evaluation.game, config);
  const baseCacheKey = replyRiskCacheKey(
    evaluation.game,
    perspective,
    TURN_PLAN_FAMILY_CACHE_TAG[family],
    config,
  );
  const configKey = selectedOverrideConfigKey(
    engineConfig,
    config.planner.selectedFollowupProjection,
    config.planner.secondaryAnalysis,
  );
  const cacheKey =
    baseCacheKey === undefined || configKey === undefined
      ? undefined
      : { ...baseCacheKey, secondary: configKey };
  if (cacheKey !== undefined) {
    const cached = cachedSelectedOverrideUtility(execution, cacheKey);
    if (cached !== undefined) return cached;
  }
  const baseStateUtility = turnEngineEvaluateStateUtility(
    execution,
    evaluation.game,
    game,
    perspective,
    engineConfig,
  );
  let result: TurnUtility;
  if (!productionSecondaryAnalysisLive(config)) {
    result = baseStateUtility;
  } else {
    const baseUtility = turnEngineRootPlanUtilityWithConfig(
      execution,
      game,
      evaluation,
      perspective,
      engineConfig,
      family,
    );
    if (
      !config.planner.selectedFollowupProjection ||
      !canTurnEngineProjectReplyRiskRoot(evaluation, perspective)
    ) {
      result = maxTurnUtility(baseUtility, baseStateUtility);
    } else {
      const hasFollowupSurface =
        evaluation.ownDrainerVulnerable ||
        evaluation.spiritDevelopment ||
        evaluation.spiritSameTurnScoreSetupNow ||
        evaluation.spiritOwnManaSetupNow ||
        hasProgressSurface(evaluation);
      const safeBlackManaTempoProjection =
        evaluation.game.activeColor === Color.Black &&
        (family === TurnPlanFamily.ManaTempo ||
          family === TurnPlanFamily.DrainerSafetyRecovery) &&
        !hasFollowupSurface &&
        !isUnsafe(evaluation) &&
        !evaluation.manaHandoffToOpponent &&
        !evaluation.hasRoundtrip &&
        !evaluation.winsImmediately &&
        !evaluation.attacksOpponentDrainer;
      if (!hasFollowupSurface && !safeBlackManaTempoProjection) {
        result = baseUtility;
      } else {
        const projectedPlan = turnEngineCandidatePlan(
          execution,
          evaluation.game,
          perspective,
          engineConfig,
        );
        result =
          projectedPlan === undefined
            ? baseUtility
            : maxTurnUtility(projectedPlan.utility, baseUtility);
      }
    }
  }
  if (execution.session.cacheWriteAllowed && cacheKey !== undefined) {
    storeSelectedOverrideUtility(execution, cacheKey, result);
  }
  return result;
}

export function turnEngineReplyRiskProjections(
  execution: AutomoveExecutionContext,
  evaluations: readonly EvaluatedRoot[],
  shortlist: readonly number[],
  perspective: Color,
  config: AutomoveConfig,
): ReadonlyMap<number, TurnEngineRootProjection> {
  if (!productionSecondaryAnalysisLive(config) || shortlist.length < 2) {
    return new Map();
  }
  const allowSafePlainSpiritProjection = safePlainSpiritCompetition(
    evaluations,
    shortlist,
    config,
  );
  const first = evaluations[shortlist[0] ?? -1];
  const hasTacticalWindow =
    first !== undefined &&
    (first.winsImmediately ||
      first.attacksOpponentDrainer ||
      first.scoresSupermanaThisTurn ||
      first.scoresOpponentManaThisTurn);
  const hasSpiritDevelopmentWindow = shortlist.some((index) => {
    const root = evaluations[index];
    return (
      root !== undefined &&
      isPlainSpiritDevelopmentRoot(root) &&
      canTurnEngineProjectReplyRiskRoot(root, perspective)
    );
  });
  if (!hasTacticalWindow && !hasSpiritDevelopmentWindow) return new Map();

  let projectionLimit: number;
  if (hasSpiritDevelopmentWindow) {
    projectionLimit = allowSafePlainSpiritProjection
      ? config.planner.lowBudgetGuard
        ? Math.min(shortlist.length, 4)
        : shortlist.length
      : Math.min(shortlist.length, config.planner.lowBudgetGuard ? 4 : 8);
  } else {
    projectionLimit = Math.min(
      shortlist.length,
      config.planner.lowBudgetGuard ? 3 : 6,
    );
  }

  const rerankConfig = rerankTurnEngineConfig(config);
  const fullConfig = fullTurnEngineConfig(config);
  const projections = new Map<number, TurnEngineRootProjection>();
  for (const index of shortlist.slice(0, projectionLimit)) {
    if (execution.session.checkpoint()) return new Map();
    const root = evaluations[index];
    if (
      root === undefined ||
      !canTurnEngineProjectReplyRiskRoot(root, perspective)
    ) {
      continue;
    }
    const vulnerableRecoveryProjection =
      productionEnabled(config) &&
      root.ownDrainerVulnerable &&
      !root.manaHandoffToOpponent &&
      !root.hasRoundtrip;
    const engineConfig = vulnerableRecoveryProjection
      ? fullConfig
      : rerankConfig;
    const plan = turnEngineCandidatePlan(
      execution,
      root.game,
      perspective,
      engineConfig,
    );
    if (plan !== undefined) projections.set(index, { plan });
  }

  if (
    hasSpiritDevelopmentWindow &&
    !hasTacticalWindow &&
    !allowSafePlainSpiritProjection &&
    ![...projections.values()].some(({ plan }) =>
      isInformativeReplyRiskProjectionFamily(plan.headFamily),
    )
  ) {
    return new Map();
  }
  return projections;
}

export function canChallengeSpiritPreferenceRoot(
  root: EvaluatedRoot,
  perspective: Color,
): boolean {
  return (
    canTurnEngineProjectReplyRiskRoot(root, perspective) &&
    !isPlainSpiritDevelopmentRoot(root) &&
    !root.spiritSameTurnScoreSetupNow &&
    !root.spiritOwnManaSetupNow &&
    !isUnsafe(root) &&
    !root.hasRoundtrip
  );
}

export function canChallengeSpiritPreferenceRootWithRecovery(
  root: EvaluatedRoot,
  perspective: Color,
): boolean {
  return (
    canTurnEngineProjectReplyRiskRoot(root, perspective) &&
    !isPlainSpiritDevelopmentRoot(root) &&
    !root.spiritSameTurnScoreSetupNow &&
    !root.spiritOwnManaSetupNow &&
    !root.manaHandoffToOpponent &&
    !root.hasRoundtrip
  );
}

/** Build the spirit/challenger projection shortlist used by the advisor. */
export function buildSpiritRootProjections(
  execution: AutomoveExecutionContext,
  evaluations: readonly EvaluatedRoot[],
  candidateIndices: readonly number[],
  perspective: Color,
  config: AutomoveConfig,
): ReadonlyMap<number, TurnEngineRootProjection> {
  if (!productionSecondaryAnalysisLive(config)) return new Map();

  const spiritLimit = config.planner.lowBudgetGuard ? 4 : 6;
  const spiritShortlist = candidateIndices
    .filter((index) => {
      const root = evaluations[index];
      return (
        root !== undefined &&
        isPlainSpiritDevelopmentRoot(root) &&
        canTurnEngineProjectReplyRiskRoot(root, perspective)
      );
    })
    .sort((left, right) =>
      compareRankedReplyRiskEvaluations(evaluations, left, right),
    )
    .slice(0, spiritLimit);

  const challengerLimit = config.planner.lowBudgetGuard ? 2 : 4;
  const challengerShortlist = candidateIndices
    .filter((index) => {
      const root = evaluations[index];
      return (
        root !== undefined &&
        (canChallengeSpiritPreferenceRoot(root, perspective) ||
          canChallengeSpiritPreferenceRootWithRecovery(root, perspective))
      );
    })
    .sort((left, right) =>
      compareRankedReplyRiskEvaluations(evaluations, left, right),
    )
    .slice(0, challengerLimit);

  const shortlist = [...spiritShortlist];
  for (const index of challengerShortlist) {
    if (!shortlist.includes(index)) shortlist.push(index);
  }
  if (shortlist.length < 2) return new Map();

  const rerankConfig = rerankTurnEngineConfig(config);
  const fullConfig = fullTurnEngineConfig(config);
  const projections = new Map<number, TurnEngineRootProjection>();
  for (const index of shortlist) {
    const root = evaluations[index];
    if (root === undefined) continue;
    const recoveryOnly =
      canChallengeSpiritPreferenceRootWithRecovery(root, perspective) &&
      !canChallengeSpiritPreferenceRoot(root, perspective);
    const engineConfig = recoveryOnly ? fullConfig : rerankConfig;
    const plan = turnEngineCandidatePlan(
      execution,
      root.game,
      perspective,
      engineConfig,
    );
    if (plan !== undefined) projections.set(index, { plan });
  }
  return projections;
}

export function spiritFollowupFloorScore(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: AutomoveConfig,
): number {
  if (execution.session.checkpoint()) return 0;
  const winner = game.winnerColor();
  if (winner !== undefined) {
    return winner === perspective
      ? Math.trunc(SMART_TERMINAL_SCORE / 2)
      : -Math.trunc(SMART_TERMINAL_SCORE / 2);
  }
  const key = replyRiskCacheKey(game, perspective, 1, config);
  if (key !== undefined) {
    const cached = cachedSpiritFollowupFloor(execution, key);
    if (cached !== undefined) return cached;
  }
  const score = evaluateReplyRiskGame(execution, game, perspective, config);
  if (execution.session.cacheWriteAllowed && key !== undefined) {
    storeSpiritFollowupFloor(execution, key, score);
  }
  return score;
}
