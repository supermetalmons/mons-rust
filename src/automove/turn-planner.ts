import type { AutomoveExecutionContext } from "./execution-context.js";
import {
  inputChainKey,
  otherColor,
  type Color,
  type Input,
} from "../engine/domain.js";
import { MonsGame } from "../engine/game.js";
import { BOARD_SIZE, locationIndex } from "../engine/geometry.js";
import {
  MAX_SCORE,
  MIN_SCORE,
  saturatingScoreAdd,
  saturatingScoreMultiply,
} from "./score-math.js";
import {
  Hash64Set,
  Hash64Table,
  hash64FromLowWord,
  hash64Mul,
  hash64RotateLeft,
  hash64Xor,
  type Hash64,
} from "./hash64.js";
import { exactSearchStateHash } from "./exact.js";
import { applyInputsForSearch, compareInputChains } from "./transitions.js";
import {
  cacheKey,
  cacheKeyForMode,
  turnCacheAdd,
  turnCacheDelete,
  turnCacheGet,
  turnCacheHas,
  turnCacheSet,
  turnEngineCaches,
  type TurnCacheKey,
} from "./turn-cache.js";
import {
  TransitionCompilePool,
  actionIdentity,
  compileAction,
  compileActionFromPool,
} from "./turn-compiler.js";
import {
  createTurnUtilityEvalContext,
  createTurnUtilityEvalContextWithSearchHash,
  evaluateStateUtilityWithSearchHash,
  opponentCanWinImmediately,
  opponentCanWinImmediatelyWithSearchHash,
  ownDrainerSafetyScore,
  quickOrderScoreWithSearchHash,
  turnOracleContextWithSearchHash,
  type TurnUtilityEvalContext,
} from "./turn-evaluation.js";
import {
  discoverTurnOpportunities,
  fallbackWalkSeeds,
  generateActionSeeds,
  opportunityScore,
} from "./turn-opportunities.js";
import {
  EMPTY_TURN_UTILITY,
  EMPTY_PACKAGE_META,
  FNV_OFFSET_BASIS,
  FNV_PRIME,
  LOCAL_HASH_COLLECTION_CAPACITY,
  PlanBuildStatus,
  TURN_ENGINE_COMPILE_LIMIT_MAX,
  TURN_PLAN_FAMILY_PRIORITY_ORDER,
  HASH64_ALL_ONES,
  HASH64_ALL_ONES_EXCEPT_LOW_BIT,
  TurnEngineMode,
  TurnPlanFamily,
  actionKeyTuple,
  compareActionKeys,
  compareChunks,
  compareNumber,
  compareTuples,
  compareTurnUtilities,
  compareUtilityPrimaryAxes,
  copyPlan,
  familyRank,
  turnEngineComparePlans,
  type MacroOpportunity,
  type MacroPlanNode,
  type OpportunityDelta,
  type PlanBuildResult,
  type PlanGenerationResult,
  type PlanNode,
  type TurnAction,
  type TurnEngineConfig,
  type TurnOpportunity,
  type TurnOracleContext,
  type TurnPlan,
  type TurnUtility,
} from "./turn-types.js";

export function cachedBestPlanIfLegal(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  key: TurnCacheKey,
): TurnPlan | undefined {
  if (execution.session.checkpoint()) return undefined;
  const cached = turnCacheGet(turnEngineCaches(execution).bestPlan, key);
  if (cached === undefined) return undefined;
  const first = cached.compiledChunks[0];
  const legal =
    first !== undefined && applyInputsForSearch(game, first) !== undefined;
  if (execution.session.checkpoint()) return undefined;
  if (!legal) {
    if (execution.session.cacheWriteAllowed)
      turnCacheDelete(turnEngineCaches(execution).bestPlan, key);
    return undefined;
  }
  return copyPlan(cached);
}

export function cachedStepIfLegal(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  config: TurnEngineConfig,
): Input[] | undefined {
  if (execution.session.checkpoint()) return undefined;
  const key = cacheKey(game, config);
  const cached = turnCacheGet(turnEngineCaches(execution).continuation, key);
  if (cached === undefined) return undefined;
  const legal = applyInputsForSearch(game, cached) !== undefined;
  if (execution.session.checkpoint()) return undefined;
  if (!legal) {
    if (execution.session.cacheWriteAllowed)
      turnCacheDelete(turnEngineCaches(execution).continuation, key);
    return undefined;
  }
  return cached.slice();
}

export function turnEngineCachedStep(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  config: TurnEngineConfig,
): Input[] | undefined {
  if (execution.session.checkpoint()) return undefined;
  const result = cachedStepIfLegal(execution, game, config);
  return execution.session.checkpoint() ? undefined : result;
}

export function turnEngineStoreCachedStep(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  mode: TurnEngineMode,
  config: TurnEngineConfig,
  inputs: readonly Input[],
): void {
  if (execution.session.checkpoint() || !execution.session.cacheWriteAllowed)
    return;
  turnCacheSet(
    turnEngineCaches(execution).continuation,
    cacheKeyForMode(game, mode, config),
    inputs.slice(),
  );
}

export function turnEngineCandidatePlan(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: TurnEngineConfig,
): TurnPlan | undefined {
  if (execution.session.checkpoint() || game.activeColor !== perspective)
    return undefined;
  const key = cacheKey(game, config);
  if (turnCacheHas(turnEngineCaches(execution).noPlan, key)) return undefined;
  const cached = cachedBestPlanIfLegal(execution, game, key);
  if (cached !== undefined)
    return execution.session.checkpoint() ? undefined : cached;

  const result = buildBestPlan(execution, game, perspective, config);
  if (execution.session.checkpoint()) return undefined;
  switch (result.status) {
    case PlanBuildStatus.NoPlan:
      if (execution.session.cacheWriteAllowed)
        turnCacheAdd(turnEngineCaches(execution).noPlan, key);
      return undefined;
    case PlanBuildStatus.BudgetExceeded:
      return undefined;
    case "ok": {
      if (!execution.session.cacheWriteAllowed) return undefined;
      turnCacheSet(
        turnEngineCaches(execution).bestPlan,
        key,
        copyPlan(result.plan),
      );
      return result.plan;
    }
  }
}

export function turnEngineCandidatePlanLive(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: TurnEngineConfig,
): TurnPlan | undefined {
  if (execution.session.checkpoint() || game.activeColor !== perspective)
    return undefined;
  const result = buildBestPlan(execution, game, perspective, config);
  return execution.session.checkpoint() || result.status !== "ok"
    ? undefined
    : result.plan;
}

export function allowedRankMap(
  allowedFirstSteps: readonly (readonly Input[])[],
): ReadonlyMap<string, number> {
  const result = new Map<string, number>();
  allowedFirstSteps.forEach((inputs, rank) => {
    const key = inputChainKey(inputs);
    if (!result.has(key)) result.set(key, rank);
  });
  return result;
}

export type AllowedHeadSelectionMeta = {
  readonly rank: number;
  readonly allowedLength: number;
  readonly firstStepOpponentImmediateLoss: boolean;
  readonly firstStepDrainerSafety: number;
};

export function allowedHeadSelectionMeta(
  execution: AutomoveExecutionContext,
  root: MonsGame,
  plan: TurnPlan,
  perspective: Color,
  rank: number,
  allowedLength: number,
): AllowedHeadSelectionMeta {
  const first = plan.compiledChunks[0];
  const after =
    first === undefined ? undefined : applyInputsForSearch(root, first);
  return {
    rank,
    allowedLength,
    firstStepOpponentImmediateLoss:
      after !== undefined &&
      opponentCanWinImmediately(execution, after, perspective),
    firstStepDrainerSafety:
      after === undefined
        ? Math.trunc(MIN_SCORE / 4)
        : ownDrainerSafetyScore(execution, after.board, perspective),
  };
}

export function allowedHeadRankAdjustedEval(
  utility: TurnUtility,
  meta: AllowedHeadSelectionMeta,
): number {
  return saturatingScoreAdd(
    utility.evalScore,
    saturatingScoreMultiply(
      Math.min(Math.max(meta.allowedLength - meta.rank, 0), 96),
      12,
    ),
  );
}

export function compareAllowedHeadPlans(
  left: TurnPlan,
  leftMeta: AllowedHeadSelectionMeta,
  right: TurnPlan,
  rightMeta: AllowedHeadSelectionMeta,
): number {
  let order = compareUtilityPrimaryAxes(left.utility, right.utility);
  if (order !== 0) return order;
  order = compareTuples(
    [
      Number(!leftMeta.firstStepOpponentImmediateLoss),
      leftMeta.firstStepDrainerSafety,
    ],
    [
      Number(!rightMeta.firstStepOpponentImmediateLoss),
      rightMeta.firstStepDrainerSafety,
    ],
  );
  if (order !== 0) return order;
  order = compareNumber(
    allowedHeadRankAdjustedEval(left.utility, leftMeta),
    allowedHeadRankAdjustedEval(right.utility, rightMeta),
  );
  if (order !== 0) return order;
  order = turnEngineComparePlans(left, right);
  return order !== 0 ? order : compareNumber(rightMeta.rank, leftMeta.rank);
}

export function bestPlanFromAllowedHeads(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: TurnEngineConfig,
  utilityContext: TurnUtilityEvalContext,
  plans: readonly TurnPlan[],
  allowedFirstSteps: readonly (readonly Input[])[],
): TurnPlan | undefined {
  const ranks = allowedRankMap(allowedFirstSteps);
  let best: { plan: TurnPlan; meta: AllowedHeadSelectionMeta } | undefined;
  for (const plan of plans) {
    if (execution.session.checkpoint()) return undefined;
    const first = plan.compiledChunks[0];
    if (first === undefined) continue;
    const rank = ranks.get(inputChainKey(first));
    if (rank === undefined) continue;
    plan.utility = evaluatePlanWithReplies(
      execution,
      plan,
      perspective,
      config,
      utilityContext,
    );
    if (execution.session.cancelled) return undefined;
    const meta = allowedHeadSelectionMeta(
      execution,
      game,
      plan,
      perspective,
      rank,
      allowedFirstSteps.length,
    );
    if (
      best === undefined ||
      compareAllowedHeadPlans(plan, meta, best.plan, best.meta) > 0
    ) {
      best = { plan, meta };
    }
  }
  return best?.plan;
}

export function buildBestPlanFromAllowedHeads(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: TurnEngineConfig,
  allowedFirstSteps: readonly (readonly Input[])[],
): PlanBuildResult {
  if (execution.session.checkpoint())
    return { status: PlanBuildStatus.BudgetExceeded };
  const utilityContext = createTurnUtilityEvalContext(
    execution,
    game,
    perspective,
    config,
  );
  const generated = generatePlansForMode(
    execution,
    game,
    perspective,
    config,
    utilityContext,
    Math.max(config.ownSeedCap, 1),
    Math.max(config.ownBeam, 1),
    Math.max(config.stepCap, 1),
    Math.max(config.expansionCap, 1),
  );
  if (execution.session.checkpoint())
    return { status: PlanBuildStatus.BudgetExceeded };
  if (generated.status === PlanBuildStatus.BudgetExceeded) return generated;

  if (generated.status === "ok") {
    const selected = bestPlanFromAllowedHeads(
      execution,
      game,
      perspective,
      config,
      utilityContext,
      generated.plans,
      allowedFirstSteps,
    );
    if (execution.session.checkpoint())
      return { status: PlanBuildStatus.BudgetExceeded };
    if (selected !== undefined) return { status: "ok", plan: selected };
  }

  const fallback = fallbackSingleActionPlanFromAllowedHeads(
    execution,
    game,
    perspective,
    config,
    utilityContext,
    allowedFirstSteps,
  );
  if (execution.session.checkpoint())
    return { status: PlanBuildStatus.BudgetExceeded };
  return fallback === undefined
    ? { status: PlanBuildStatus.NoPlan }
    : { status: "ok", plan: fallback };
}

export function turnEngineCandidatePlanFromAllowedHeads(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: TurnEngineConfig,
  allowedFirstSteps: readonly (readonly Input[])[],
): TurnPlan | undefined {
  if (
    execution.session.checkpoint() ||
    game.activeColor !== perspective ||
    allowedFirstSteps.length === 0
  ) {
    return undefined;
  }
  const ranks = allowedRankMap(allowedFirstSteps);
  const cached = cachedBestPlanIfLegal(execution, game, cacheKey(game, config));
  const cachedFirst = cached?.compiledChunks[0];
  if (cachedFirst !== undefined && ranks.has(inputChainKey(cachedFirst))) {
    return cached;
  }
  const result = buildBestPlanFromAllowedHeads(
    execution,
    game,
    perspective,
    config,
    allowedFirstSteps,
  );
  return execution.session.checkpoint() || result.status !== "ok"
    ? undefined
    : result.plan;
}

export function turnEngineNextInputsFromAllowedHeads(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  mode: TurnEngineMode,
  config: TurnEngineConfig,
  allowedFirstSteps: readonly (readonly Input[])[],
): Input[] | undefined {
  if (
    execution.session.checkpoint() ||
    game.activeColor !== perspective ||
    allowedFirstSteps.length === 0
  ) {
    return undefined;
  }
  const allowed = new Set(allowedFirstSteps.map(inputChainKey));
  const cached = turnEngineCachedStep(execution, game, config);
  if (cached !== undefined && allowed.has(inputChainKey(cached))) return cached;
  const plan = turnEngineCandidatePlanFromAllowedHeads(
    execution,
    game,
    perspective,
    config,
    allowedFirstSteps,
  );
  if (plan === undefined || execution.session.checkpoint()) return undefined;
  registerPlanContinuations(execution, game, perspective, mode, plan, config);
  if (execution.session.cancelled) return undefined;
  return plan.compiledChunks[0]?.slice();
}

export function turnEngineCommitPlan(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  mode: TurnEngineMode,
  plan: TurnPlan,
  config: TurnEngineConfig,
): void {
  if (!execution.session.checkpoint())
    registerPlanContinuations(execution, game, perspective, mode, plan, config);
}

export function registerPlanContinuations(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  mode: TurnEngineMode,
  plan: TurnPlan,
  config: TurnEngineConfig,
): void {
  if (execution.session.checkpoint() || plan.compiledChunks.length === 0)
    return;
  let state = game.fork();
  const startColor = game.activeColor;
  for (let index = 0; index < plan.compiledChunks.length; index += 1) {
    if (execution.session.checkpoint()) return;
    const chunk = plan.compiledChunks[index];
    if (chunk === undefined) continue;
    if (index > 0 && mode === TurnEngineMode.Production) {
      const fresh = turnEngineCandidatePlan(
        execution,
        state,
        perspective,
        config,
      );
      if (
        fresh?.compiledChunks[0] === undefined ||
        execution.session.checkpoint()
      )
        break;
      if (compareInputChains(fresh.compiledChunks[0], chunk) !== 0) break;
    }
    if (!execution.session.cacheWriteAllowed) return;
    turnCacheSet(
      turnEngineCaches(execution).continuation,
      cacheKeyForMode(state, mode, config),
      chunk.slice(),
    );
    const next = applyInputsForSearch(state, chunk);
    if (next?.activeColor !== startColor) break;
    state = next;
  }
}

export function turnEngineEvaluatePlanWithReplies(
  execution: AutomoveExecutionContext,
  root: MonsGame,
  plan: TurnPlan,
  perspective: Color,
  config: TurnEngineConfig,
): TurnUtility {
  if (execution.session.checkpoint()) return EMPTY_TURN_UTILITY;
  const utilityContext = createTurnUtilityEvalContext(
    execution,
    root,
    perspective,
    config,
  );
  const utility = evaluatePlanWithReplies(
    execution,
    plan,
    perspective,
    config,
    utilityContext,
  );
  return execution.session.checkpoint() ? EMPTY_TURN_UTILITY : utility;
}

export function buildBestPlan(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: TurnEngineConfig,
): PlanBuildResult {
  if (execution.session.checkpoint())
    return { status: PlanBuildStatus.BudgetExceeded };
  const utilityContext = createTurnUtilityEvalContext(
    execution,
    game,
    perspective,
    config,
  );
  const generated = generatePlansForMode(
    execution,
    game,
    perspective,
    config,
    utilityContext,
    Math.max(config.ownSeedCap, 1),
    Math.max(config.ownBeam, 1),
    Math.max(config.stepCap, 1),
    Math.max(config.expansionCap, 1),
  );
  if (execution.session.checkpoint())
    return { status: PlanBuildStatus.BudgetExceeded };
  if (generated.status === PlanBuildStatus.BudgetExceeded) return generated;

  let plans: TurnPlan[];
  if (generated.status === "ok") {
    plans = generated.plans;
  } else {
    const fallback = fallbackSingleActionPlan(
      execution,
      game,
      perspective,
      config,
      utilityContext,
    );
    if (execution.session.checkpoint())
      return { status: PlanBuildStatus.BudgetExceeded };
    return fallback === undefined
      ? { status: PlanBuildStatus.NoPlan }
      : { status: "ok", plan: fallback };
  }

  if (config.mode === TurnEngineMode.Production && plans.length > 1) {
    plans.sort((left, right) => -turnEngineComparePlans(left, right));
    const shortlistLength = Math.min(
      plans.length,
      Math.min(Math.max(Math.max(config.ownBeam, 1) * 2, 6), 12),
    );
    const perSignatureCap = config.ownBeam >= 4 ? 2 : 1;
    const signatures = new Map<string, number>();
    const shortlisted: TurnPlan[] = [];
    for (const plan of plans) {
      const signature = `${inputChainKey(plan.compiledChunks[0] ?? [])}:${plan.headFamily}:${
        plan.goalFamily
      }`;
      const count = signatures.get(signature) ?? 0;
      if (count >= perSignatureCap) continue;
      signatures.set(signature, count + 1);
      shortlisted.push(plan);
      if (shortlisted.length >= shortlistLength) break;
    }
    plans = shortlisted;
  }

  let best: TurnPlan | undefined;
  for (const plan of plans) {
    if (execution.session.checkpoint())
      return { status: PlanBuildStatus.BudgetExceeded };
    plan.utility = evaluatePlanWithReplies(
      execution,
      plan,
      perspective,
      config,
      utilityContext,
    );
    if (execution.session.cancelled)
      return { status: PlanBuildStatus.BudgetExceeded };
    if (best === undefined || turnEngineComparePlans(plan, best) > 0)
      best = plan;
  }
  if (execution.session.checkpoint())
    return { status: PlanBuildStatus.BudgetExceeded };
  return best === undefined
    ? { status: PlanBuildStatus.NoPlan }
    : { status: "ok", plan: best };
}

export function generatePlansForMode(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: TurnEngineConfig,
  utilityContext: TurnUtilityEvalContext,
  seedCap: number,
  beamWidth: number,
  stepCap: number,
  expansionCap: number,
): PlanGenerationResult {
  if (execution.session.checkpoint())
    return { status: PlanBuildStatus.BudgetExceeded };
  const result =
    config.mode === TurnEngineMode.Production
      ? generateMacroPlans(
          execution,
          game,
          perspective,
          config,
          utilityContext,
          seedCap,
          beamWidth,
          Math.min(stepCap, bundlePlanCapForConfig(config)),
          expansionCap,
        )
      : generateTurnPlans(
          execution,
          game,
          perspective,
          config,
          utilityContext,
          seedCap,
          beamWidth,
          stepCap,
          expansionCap,
        );
  return execution.session.checkpoint()
    ? { status: PlanBuildStatus.BudgetExceeded }
    : result;
}

export function bundleChunkCapForConfig(config: TurnEngineConfig): number {
  return Math.min(Math.max(config.stepCap, 1), 6);
}

export function bundlePlanCapForConfig(config: TurnEngineConfig): number {
  return Math.min(Math.max(config.stepCap, 1), 4);
}

export function mergePlanFamily(
  current: TurnPlanFamily,
  next: TurnPlanFamily,
): TurnPlanFamily {
  return familyRank(next) < familyRank(current) ? next : current;
}

export function macroFollowupFamilyAllowed(
  head: TurnPlanFamily,
  goal: TurnPlanFamily,
  candidate: TurnPlanFamily,
): boolean {
  if (candidate === goal || candidate === head) return true;
  switch (head) {
    case TurnPlanFamily.ImmediateScore:
      return (
        candidate === TurnPlanFamily.ImmediateScore ||
        candidate === TurnPlanFamily.DrainerSafetyRecovery ||
        candidate === TurnPlanFamily.SafeSupermanaProgress ||
        candidate === TurnPlanFamily.SafeOpponentManaProgress
      );
    case TurnPlanFamily.DenyOpponentWindow:
    case TurnPlanFamily.DrainerKill:
      return (
        candidate === TurnPlanFamily.ImmediateScore ||
        candidate === TurnPlanFamily.DenyOpponentWindow ||
        candidate === TurnPlanFamily.DrainerKill ||
        candidate === TurnPlanFamily.DrainerSafetyRecovery ||
        candidate === TurnPlanFamily.SafeSupermanaProgress ||
        candidate === TurnPlanFamily.SafeOpponentManaProgress
      );
    case TurnPlanFamily.DrainerSafetyRecovery:
      return (
        candidate === TurnPlanFamily.ImmediateScore ||
        candidate === TurnPlanFamily.DrainerSafetyRecovery ||
        candidate === TurnPlanFamily.SafeSupermanaProgress ||
        candidate === TurnPlanFamily.SafeOpponentManaProgress ||
        candidate === TurnPlanFamily.ManaTempo
      );
    case TurnPlanFamily.SpiritImpact:
      return (
        candidate === TurnPlanFamily.ImmediateScore ||
        candidate === TurnPlanFamily.DenyOpponentWindow ||
        candidate === TurnPlanFamily.SpiritImpact ||
        candidate === TurnPlanFamily.SafeSupermanaProgress ||
        candidate === TurnPlanFamily.SafeOpponentManaProgress ||
        candidate === TurnPlanFamily.DrainerSafetyRecovery
      );
    case TurnPlanFamily.SafeSupermanaProgress:
    case TurnPlanFamily.SafeOpponentManaProgress:
      return (
        candidate === TurnPlanFamily.ImmediateScore ||
        candidate === TurnPlanFamily.DrainerSafetyRecovery ||
        candidate === TurnPlanFamily.SafeSupermanaProgress ||
        candidate === TurnPlanFamily.SafeOpponentManaProgress ||
        candidate === TurnPlanFamily.DenyOpponentWindow ||
        candidate === TurnPlanFamily.SpiritImpact
      );
    case TurnPlanFamily.ManaTempo:
      return (
        candidate === TurnPlanFamily.ImmediateScore ||
        candidate === TurnPlanFamily.DrainerSafetyRecovery ||
        candidate === TurnPlanFamily.SafeSupermanaProgress ||
        candidate === TurnPlanFamily.SafeOpponentManaProgress ||
        candidate === TurnPlanFamily.SpiritImpact ||
        candidate === TurnPlanFamily.ManaTempo
      );
  }
}

export function macroFollowupFamilyBonus(
  head: TurnPlanFamily,
  goal: TurnPlanFamily,
  candidate: TurnPlanFamily,
): number {
  let bonus = 0;
  if (candidate === goal) bonus += 420;
  if (candidate === head) bonus += 220;
  if (candidate === TurnPlanFamily.ImmediateScore) bonus += 640;
  if (
    head === TurnPlanFamily.SpiritImpact &&
    (candidate === TurnPlanFamily.SpiritImpact ||
      candidate === TurnPlanFamily.ImmediateScore ||
      candidate === TurnPlanFamily.SafeSupermanaProgress ||
      candidate === TurnPlanFamily.SafeOpponentManaProgress)
  ) {
    bonus += 180;
  }
  if (
    (head === TurnPlanFamily.SafeSupermanaProgress ||
      head === TurnPlanFamily.SafeOpponentManaProgress) &&
    (candidate === TurnPlanFamily.SafeSupermanaProgress ||
      candidate === TurnPlanFamily.SafeOpponentManaProgress ||
      candidate === TurnPlanFamily.ImmediateScore)
  ) {
    bonus += 180;
  }
  if (
    head === TurnPlanFamily.DrainerSafetyRecovery &&
    (candidate === TurnPlanFamily.DrainerSafetyRecovery ||
      candidate === TurnPlanFamily.SafeSupermanaProgress ||
      candidate === TurnPlanFamily.SafeOpponentManaProgress ||
      candidate === TurnPlanFamily.ImmediateScore)
  ) {
    bonus += 160;
  }
  return bonus;
}

export function macroFollowupFamilies(
  head: TurnPlanFamily,
  goal: TurnPlanFamily,
): TurnPlanFamily[] {
  return TURN_PLAN_FAMILY_PRIORITY_ORDER.filter((candidate) =>
    macroFollowupFamilyAllowed(head, goal, candidate),
  );
}

export function macroFollowupSeedCandidates(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  gameHash: Hash64,
  perspective: Color,
  config: TurnEngineConfig,
  head: TurnPlanFamily,
  goal: TurnPlanFamily,
  usedActions: readonly TurnAction[],
): TurnOpportunity[] {
  const oracle = turnOracleContextWithSearchHash(
    execution,
    game,
    perspective,
    gameHash,
  );
  const emergency =
    oracle.opportunity.opponentCanWinImmediately ||
    oracle.opportunity.delta.drainerSafety < 0;
  const used = new Set(usedActions.map(actionIdentity));
  const candidates = discoverTurnOpportunities(
    execution,
    game,
    perspective,
    config,
    Math.max(Math.max(config.ownSeedCap, config.perNodeFamilyCap * 3), 8),
    macroFollowupFamilies(head, goal),
  ).filter((opportunity) => !used.has(actionIdentity(opportunity.action)));
  candidates.sort((left, right) => {
    const scoreOrder = compareNumber(
      opportunityScore(right, emergency) +
        macroFollowupFamilyBonus(head, goal, right.family),
      opportunityScore(left, emergency) +
        macroFollowupFamilyBonus(head, goal, left.family),
    );
    return scoreOrder !== 0
      ? scoreOrder
      : compareActionKeys(left.action, right.action);
  });
  return candidates.slice(
    0,
    Math.max(Math.max(config.perNodeFamilyCap, 1) * 2, 4),
  );
}

export function progressStepGain(
  before: number | undefined,
  after: number | undefined,
): number {
  const unknown = BOARD_SIZE * 3;
  return Math.max((before ?? unknown) - (after ?? unknown), 0);
}

export function macroOpportunityDelta(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  endGame: MonsGame,
  endHash: Hash64,
  perspective: Color,
  startOracle: TurnOracleContext,
): OpportunityDelta {
  if (execution.session.checkpoint()) return emptyOpportunityDelta();
  const endOracle = turnOracleContextWithSearchHash(
    execution,
    endGame,
    perspective,
    endHash,
  );
  if (execution.session.cancelled) return emptyOpportunityDelta();
  return {
    sameTurnScoreWindowGain: Math.max(
      endOracle.strategic.immediateWindow.bestScore -
        startOracle.strategic.immediateWindow.bestScore,
      0,
    ),
    spiritGain: Math.max(
      endOracle.strategic.spirit.nextTurnSetupGain -
        startOracle.strategic.spirit.nextTurnSetupGain,
      endOracle.strategic.spirit.utility - startOracle.strategic.spirit.utility,
      0,
    ),
    opponentWindowDenyGain: Math.max(
      startOracle.opponentImmediateWindow - endOracle.opponentImmediateWindow,
      0,
    ),
    drainerAttack: endOracle.opportunity.delta.drainerAttackAvailable,
    drainerSafetyDelta:
      ownDrainerSafetyScore(execution, endGame.board, perspective) -
      ownDrainerSafetyScore(execution, game.board, perspective),
    supermanaProgressGain: progressStepGain(
      startOracle.opportunity.delta.safeSupermanaProgressSteps,
      endOracle.opportunity.delta.safeSupermanaProgressSteps,
    ),
    opponentManaProgressGain: progressStepGain(
      startOracle.opportunity.delta.safeOpponentManaProgressSteps,
      endOracle.opportunity.delta.safeOpponentManaProgressSteps,
    ),
  };
}

export function emptyOpportunityDelta(): OpportunityDelta {
  return {
    sameTurnScoreWindowGain: 0,
    spiritGain: 0,
    opponentWindowDenyGain: 0,
    drainerAttack: false,
    drainerSafetyDelta: 0,
    supermanaProgressGain: 0,
    opponentManaProgressGain: 0,
  };
}

export function macroPriorityFromState(
  utilityContext: TurnUtilityEvalContext,
  endGame: MonsGame,
  endHash: Hash64,
  family: TurnPlanFamily,
  chunkCount: number,
  priorityHint: number,
): number {
  return saturatingScoreAdd(
    priorityHint,
    Math.max(
      MIN_SCORE,
      Math.min(
        MAX_SCORE,
        Math.trunc(
          quickOrderScoreWithSearchHash(
            utilityContext,
            endGame,
            endHash,
            family,
            chunkCount,
          ) / 1_024,
        ),
      ),
    ),
  );
}

export function macroSignatureMix(hash: Hash64, value: Hash64): Hash64 {
  return hash64RotateLeft(hash64Mul(hash64Xor(hash, value), FNV_PRIME), 11);
}

export function macroSignatureForActions(
  actions: readonly TurnAction[],
): Hash64 {
  let hash = FNV_OFFSET_BASIS;
  for (const action of actions) {
    const [tag, first, second, third] = actionKeyTuple(action);
    hash = macroSignatureMix(hash, hash64FromLowWord(tag));
    hash = macroSignatureMix(hash, hash64FromLowWord(locationIndex(first)));
    hash = macroSignatureMix(
      hash,
      second === undefined
        ? HASH64_ALL_ONES
        : hash64FromLowWord(locationIndex(second)),
    );
    hash = macroSignatureMix(
      hash,
      third === undefined
        ? HASH64_ALL_ONES_EXCEPT_LOW_BIT
        : hash64FromLowWord(locationIndex(third)),
    );
  }
  return hash;
}

export function macroPlanSignature(
  previous: Hash64,
  opportunity: MacroOpportunity,
): Hash64 {
  return macroSignatureMix(
    macroSignatureMix(previous, opportunity.endSnapshot.stateHash),
    opportunity.signature,
  );
}

export function buildMacroFromHeadOpportunity(
  execution: AutomoveExecutionContext,
  root: MonsGame,
  perspective: Color,
  config: TurnEngineConfig,
  utilityContext: TurnUtilityEvalContext,
  opportunity: TurnOpportunity,
): MacroOpportunity | undefined {
  if (execution.session.checkpoint()) return undefined;
  const startOracle = turnOracleContextWithSearchHash(
    execution,
    root,
    perspective,
    utilityContext.startHash,
  );
  if (execution.session.cancelled) return undefined;
  const first = compileAction(
    execution,
    root,
    perspective,
    opportunity.action,
    config,
  );
  if (first === undefined || execution.session.checkpoint()) return undefined;
  let current = first[0];
  let currentHash = exactSearchStateHash(current);
  const headUtility = evaluateStateUtilityWithSearchHash(
    utilityContext,
    current,
    currentHash,
  );
  const sessionAfterHeadEvaluation = execution.session;
  if (sessionAfterHeadEvaluation.cancelled) return undefined;
  const actions: TurnAction[] = [opportunity.action];
  const compiledChunks: Input[][] = [first[1]];
  let goalFamily = opportunity.family;
  const visitedStates = new Hash64Set(LOCAL_HASH_COLLECTION_CAPACITY);
  visitedStates.add(utilityContext.startHash);
  visitedStates.add(currentHash);

  while (
    current.activeColor === perspective &&
    current.winnerColor() === undefined &&
    compiledChunks.length < bundleChunkCapForConfig(config)
  ) {
    if (execution.session.checkpoint()) return undefined;
    const currentOracle = turnOracleContextWithSearchHash(
      execution,
      current,
      perspective,
      currentHash,
    );
    const sessionAfterCurrentOracle = execution.session;
    if (sessionAfterCurrentOracle.cancelled) return undefined;
    const currentUtility = evaluateStateUtilityWithSearchHash(
      utilityContext,
      current,
      currentHash,
    );
    const sessionAfterCurrentEvaluation = execution.session;
    if (sessionAfterCurrentEvaluation.cancelled) return undefined;
    const riskyTemporaryState =
      currentOracle.opportunity.delta.drainerSafety < 0 ||
      currentUtility.drainerSafety < 0 ||
      ownDrainerSafetyScore(execution, current.board, perspective) < 0;
    let best:
      | {
          readonly score: number;
          readonly opportunity: TurnOpportunity;
          readonly after: MonsGame;
          readonly afterHash: Hash64;
          readonly chunk: Input[];
          readonly goalFamily: TurnPlanFamily;
        }
      | undefined;

    for (const followup of macroFollowupSeedCandidates(
      execution,
      current,
      currentHash,
      perspective,
      config,
      opportunity.family,
      goalFamily,
      actions,
    )) {
      if (execution.session.checkpoint()) return undefined;
      const compiled = compileAction(
        execution,
        current,
        perspective,
        followup.action,
        config,
      );
      if (compiled === undefined || execution.session.checkpoint()) continue;
      const [after, chunk] = compiled;
      const afterHash = exactSearchStateHash(after);
      if (visitedStates.has(afterHash)) continue;
      const delta = macroOpportunityDelta(
        execution,
        current,
        after,
        afterHash,
        perspective,
        currentOracle,
      );
      const nextGoalFamily = mergePlanFamily(goalFamily, followup.family);
      const nextUtility = evaluateStateUtilityWithSearchHash(
        utilityContext,
        after,
        afterHash,
      );
      const sessionAfterFollowupEvaluation = execution.session;
      if (sessionAfterFollowupEvaluation.cancelled) return undefined;
      const improvementSignal =
        delta.sameTurnScoreWindowGain +
        delta.spiritGain +
        delta.opponentWindowDenyGain +
        Math.max(delta.drainerSafetyDelta, 0) +
        delta.supermanaProgressGain +
        delta.opponentManaProgressGain +
        (delta.drainerAttack ? 2 : 0);
      const temporaryRecoveryFollowup =
        riskyTemporaryState &&
        (followup.family === TurnPlanFamily.DrainerSafetyRecovery ||
          followup.family === TurnPlanFamily.ImmediateScore ||
          followup.family === TurnPlanFamily.SafeSupermanaProgress ||
          followup.family === TurnPlanFamily.SafeOpponentManaProgress);
      if (
        improvementSignal <= 0 &&
        compareTurnUtilities(nextUtility, currentUtility) <= 0 &&
        after.activeColor === perspective &&
        !temporaryRecoveryFollowup
      ) {
        continue;
      }
      const riskyBonus = riskyTemporaryState
        ? (() => {
            switch (followup.family) {
              case TurnPlanFamily.DrainerSafetyRecovery:
                return 960;
              case TurnPlanFamily.ImmediateScore:
                return 820;
              case TurnPlanFamily.SafeSupermanaProgress:
              case TurnPlanFamily.SafeOpponentManaProgress:
                return 360;
              case TurnPlanFamily.DenyOpponentWindow:
              case TurnPlanFamily.DrainerKill:
                return 220;
              case TurnPlanFamily.SpiritImpact:
              case TurnPlanFamily.ManaTempo:
                return 0;
            }
          })()
        : 0;
      const score =
        macroPriorityFromState(
          utilityContext,
          after,
          afterHash,
          nextGoalFamily,
          compiledChunks.length + 1,
          followup.priority +
            macroFollowupFamilyBonus(
              opportunity.family,
              goalFamily,
              followup.family,
            ),
        ) +
        riskyBonus +
        delta.sameTurnScoreWindowGain * 280 +
        delta.spiritGain * 220 +
        delta.opponentWindowDenyGain * 240 +
        Math.max(delta.drainerSafetyDelta, 0) * 200 +
        delta.supermanaProgressGain * 120 +
        delta.opponentManaProgressGain * 112 +
        (delta.drainerAttack ? 820 : 0);
      if (
        best === undefined ||
        score > best.score ||
        (score === best.score &&
          familyRank(nextGoalFamily) < familyRank(best.goalFamily))
      ) {
        best = {
          score,
          opportunity: followup,
          after,
          afterHash,
          chunk,
          goalFamily: nextGoalFamily,
        };
      }
    }
    if (best === undefined) break;
    actions.push(best.opportunity.action);
    compiledChunks.push(best.chunk);
    goalFamily = best.goalFamily;
    current = best.after;
    currentHash = best.afterHash;
    visitedStates.add(currentHash);
  }

  if (execution.session.checkpoint()) return undefined;
  const endSnapshot = { stateHash: currentHash };
  const delta = macroOpportunityDelta(
    execution,
    root,
    current,
    currentHash,
    perspective,
    startOracle,
  );
  const sessionAfterMacroDelta = execution.session;
  if (sessionAfterMacroDelta.cancelled) return undefined;
  return {
    headFamily: opportunity.family,
    goalFamily,
    priority: macroPriorityFromState(
      utilityContext,
      current,
      currentHash,
      goalFamily,
      compiledChunks.length,
      opportunity.priority,
    ),
    delta,
    actions,
    compiledChunks,
    endGame: current.fork(),
    endSnapshot,
    headUtility,
    signature: macroSignatureForActions(actions),
  };
}

export function discoverMacroOpportunities(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: TurnEngineConfig,
  utilityContext: TurnUtilityEvalContext,
  opportunityCap: number,
  allowedFamilies?: readonly TurnPlanFamily[],
): MacroOpportunity[] {
  if (execution.session.checkpoint()) return [];
  const macros: MacroOpportunity[] = [];
  const seen = new Hash64Set(LOCAL_HASH_COLLECTION_CAPACITY);
  const opportunities = discoverTurnOpportunities(
    execution,
    game,
    perspective,
    config,
    Math.max(Math.max(opportunityCap, config.perNodeFamilyCap * 3), 8),
    allowedFamilies,
  );
  if (execution.session.checkpoint()) return [];
  for (const opportunity of opportunities) {
    if (execution.session.checkpoint()) return [];
    const bundle = buildMacroFromHeadOpportunity(
      execution,
      game,
      perspective,
      config,
      utilityContext,
      opportunity,
    );
    if (bundle === undefined) {
      if (execution.session.cancelled) return [];
      continue;
    }
    if (seen.has(bundle.endSnapshot.stateHash, 0, bundle.signature)) continue;
    seen.add(bundle.endSnapshot.stateHash, 0, bundle.signature);
    macros.push(bundle);
    if (macros.length >= Math.max(opportunityCap, 1)) break;
  }
  macros.sort((left, right) => {
    const score = (value: MacroOpportunity): number =>
      value.priority +
      value.delta.sameTurnScoreWindowGain * 280 +
      value.delta.spiritGain * 220 +
      value.delta.opponentWindowDenyGain * 240 +
      value.delta.drainerSafetyDelta * 220 +
      value.delta.supermanaProgressGain * 120 +
      value.delta.opponentManaProgressGain * 112 +
      (value.delta.drainerAttack ? 820 : 0) +
      (bundleChunkCapForConfig(config) - value.compiledChunks.length) * 8;
    const order = compareNumber(score(right), score(left));
    if (order !== 0) return order;
    const familyOrder = compareNumber(
      familyRank(left.goalFamily),
      familyRank(right.goalFamily),
    );
    return familyOrder !== 0
      ? familyOrder
      : compareChunks(left.compiledChunks, right.compiledChunks);
  });
  return execution.session.checkpoint()
    ? []
    : macros.slice(0, Math.max(opportunityCap, 1));
}

export function macroNodeToPlan(
  utilityContext: TurnUtilityEvalContext,
  node: MacroPlanNode,
): TurnPlan {
  return {
    actions: node.actions,
    compiledChunks: node.compiledChunks,
    endGame: node.game.fork(),
    utility: evaluateStateUtilityWithSearchHash(
      utilityContext,
      node.game,
      node.stateHash,
    ),
    headUtility: node.headUtility,
    headFamily: node.headFamily,
    goalFamily: node.goalFamily,
    packageMeta: EMPTY_PACKAGE_META,
  };
}

export function generateMacroPlans(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: TurnEngineConfig,
  utilityContext: TurnUtilityEvalContext,
  opportunityCap: number,
  beamWidth: number,
  bundleCap: number,
  expansionCap: number,
): PlanGenerationResult {
  if (execution.session.checkpoint())
    return { status: PlanBuildStatus.BudgetExceeded };
  let expansions = 0;
  let budgetExhausted = false;
  const cap = Math.min(Math.max(bundleCap, 1), bundlePlanCapForConfig(config));
  const opportunities = discoverMacroOpportunities(
    execution,
    game,
    perspective,
    config,
    utilityContext,
    opportunityCap,
  );
  if (execution.session.checkpoint())
    return { status: PlanBuildStatus.BudgetExceeded };
  if (opportunities.length === 0) return { status: PlanBuildStatus.NoPlan };
  const seen = new Hash64Table<number>(LOCAL_HASH_COLLECTION_CAPACITY);
  let frontier: { readonly order: number; readonly node: MacroPlanNode }[] = [];
  for (const opportunity of opportunities) {
    if (execution.session.checkpoint())
      return { status: PlanBuildStatus.BudgetExceeded };
    expansions += 1;
    if (expansions > expansionCap) {
      budgetExhausted = true;
      break;
    }
    const order = quickOrderScoreWithSearchHash(
      utilityContext,
      opportunity.endGame,
      opportunity.endSnapshot.stateHash,
      opportunity.goalFamily,
      opportunity.compiledChunks.length,
    );
    if (execution.session.cancelled)
      return { status: PlanBuildStatus.BudgetExceeded };
    const existing = seen.get(
      opportunity.endSnapshot.stateHash,
      0,
      opportunity.signature,
    );
    if (existing !== undefined && order <= existing) continue;
    seen.set(
      opportunity.endSnapshot.stateHash,
      order,
      0,
      opportunity.signature,
    );
    frontier.push({
      order,
      node: {
        game: opportunity.endGame,
        stateHash: opportunity.endSnapshot.stateHash,
        actions: opportunity.actions,
        compiledChunks: opportunity.compiledChunks,
        headUtility: opportunity.headUtility,
        headFamily: opportunity.headFamily,
        goalFamily: opportunity.goalFamily,
        signature: opportunity.signature,
      },
    });
  }
  if (frontier.length === 0) {
    return {
      status: budgetExhausted
        ? PlanBuildStatus.BudgetExceeded
        : PlanBuildStatus.NoPlan,
    };
  }
  frontier.sort(compareOrderedNodes);
  frontier = frontier.slice(0, Math.max(beamWidth, 1));
  const terminal: MacroPlanNode[] = [];

  for (let round = 1; round < cap; round += 1) {
    if (execution.session.checkpoint())
      return { status: PlanBuildStatus.BudgetExceeded };
    const candidates: {
      readonly order: number;
      readonly node: MacroPlanNode;
    }[] = [];
    let expandedAny = false;
    let stopExpansion = false;
    for (const current of frontier) {
      const node = current.node;
      if (execution.session.checkpoint())
        return { status: PlanBuildStatus.BudgetExceeded };
      if (
        node.game.winnerColor() !== undefined ||
        node.game.activeColor !== perspective
      ) {
        terminal.push(node);
        continue;
      }
      const followupUtilityContext = createTurnUtilityEvalContextWithSearchHash(
        execution,
        node.game,
        node.stateHash,
        perspective,
        config,
      );
      const followups = discoverMacroOpportunities(
        execution,
        node.game,
        perspective,
        config,
        followupUtilityContext,
        opportunityCap,
        macroFollowupFamilies(node.headFamily, node.goalFamily),
      );
      if (execution.session.checkpoint())
        return { status: PlanBuildStatus.BudgetExceeded };
      if (followups.length === 0) {
        terminal.push(node);
        continue;
      }
      let nodeExpanded = false;
      for (const opportunity of followups) {
        if (execution.session.checkpoint())
          return { status: PlanBuildStatus.BudgetExceeded };
        expansions += 1;
        if (expansions > expansionCap) {
          terminal.push(node);
          budgetExhausted = true;
          stopExpansion = true;
          break;
        }
        const actions = [...node.actions, ...opportunity.actions];
        const chunks = [...node.compiledChunks, ...opportunity.compiledChunks];
        const goalFamily = mergePlanFamily(
          node.goalFamily,
          opportunity.goalFamily,
        );
        const signature = macroPlanSignature(node.signature, opportunity);
        const order = quickOrderScoreWithSearchHash(
          utilityContext,
          opportunity.endGame,
          opportunity.endSnapshot.stateHash,
          goalFamily,
          chunks.length,
        );
        if (execution.session.cancelled)
          return { status: PlanBuildStatus.BudgetExceeded };
        const existing = seen.get(
          opportunity.endSnapshot.stateHash,
          0,
          signature,
        );
        if (existing !== undefined && order <= existing) continue;
        seen.set(opportunity.endSnapshot.stateHash, order, 0, signature);
        candidates.push({
          order,
          node: {
            game: opportunity.endGame,
            stateHash: opportunity.endSnapshot.stateHash,
            actions,
            compiledChunks: chunks,
            headUtility: node.headUtility,
            headFamily: node.headFamily,
            goalFamily,
            signature,
          },
        });
        expandedAny = true;
        nodeExpanded = true;
      }
      if (stopExpansion) break;
      if (!nodeExpanded) terminal.push(node);
    }
    if (stopExpansion) {
      candidates.sort(compareOrderedNodes);
      frontier = candidates.slice(0, Math.max(beamWidth, 1));
      break;
    }
    if (!expandedAny || candidates.length === 0) {
      frontier = [];
      break;
    }
    candidates.sort(compareOrderedNodes);
    frontier = candidates.slice(0, Math.max(beamWidth, 1));
  }
  terminal.push(...frontier.map(({ node }) => node));
  if (execution.session.checkpoint())
    return { status: PlanBuildStatus.BudgetExceeded };
  if (terminal.length === 0) {
    return {
      status: budgetExhausted
        ? PlanBuildStatus.BudgetExceeded
        : PlanBuildStatus.NoPlan,
    };
  }
  const plans = terminal.map((node) => macroNodeToPlan(utilityContext, node));
  if (execution.session.checkpoint())
    return { status: PlanBuildStatus.BudgetExceeded };
  plans.sort((left, right) => -turnEngineComparePlans(left, right));
  return { status: "ok", plans };
}

export function generateTurnPlans(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: TurnEngineConfig,
  utilityContext: TurnUtilityEvalContext,
  seedCap: number,
  beamWidth: number,
  stepCap: number,
  expansionCap: number,
): PlanGenerationResult {
  if (execution.session.checkpoint())
    return { status: PlanBuildStatus.BudgetExceeded };
  let expansions = 0;
  const seeds = generateActionSeeds(
    execution,
    game,
    perspective,
    config,
    seedCap,
  );
  if (execution.session.checkpoint())
    return { status: PlanBuildStatus.BudgetExceeded };
  if (seeds.length === 0) return { status: PlanBuildStatus.NoPlan };
  const compilePool = new TransitionCompilePool(execution, game, seeds, config);
  if (execution.session.checkpoint())
    return { status: PlanBuildStatus.BudgetExceeded };
  const seen = new Hash64Table<number>(LOCAL_HASH_COLLECTION_CAPACITY);
  let frontier: { readonly order: number; readonly node: PlanNode }[] = [];

  for (const seed of seeds) {
    if (execution.session.checkpoint())
      return { status: PlanBuildStatus.BudgetExceeded };
    const compiled = compileActionFromPool(
      execution,
      game,
      perspective,
      seed.action,
      compilePool,
    );
    if (compiled === undefined) continue;
    if (execution.session.checkpoint())
      return { status: PlanBuildStatus.BudgetExceeded };
    expansions += 1;
    if (expansions > expansionCap) {
      return { status: PlanBuildStatus.BudgetExceeded };
    }
    const [after, chunk] = compiled;
    const hash = exactSearchStateHash(after);
    const order = quickOrderScoreWithSearchHash(
      utilityContext,
      after,
      hash,
      seed.family,
      1,
    );
    const existing = seen.get(hash);
    if (existing !== undefined && order <= existing) continue;
    seen.set(hash, order);
    const headUtility = evaluateStateUtilityWithSearchHash(
      utilityContext,
      after,
      hash,
    );
    if (execution.session.cancelled)
      return { status: PlanBuildStatus.BudgetExceeded };
    frontier.push({
      order,
      node: {
        game: after,
        stateHash: hash,
        actions: [seed.action],
        compiledChunks: [chunk],
        headUtility,
        headFamily: seed.family,
        goalFamily: seed.family,
      },
    });
  }
  if (frontier.length === 0) return { status: PlanBuildStatus.NoPlan };
  frontier.sort(compareOrderedNodes);
  frontier = frontier.slice(0, Math.max(beamWidth, 1));
  const terminal: PlanNode[] = [];

  for (let step = 1; step < Math.max(stepCap, 1); step += 1) {
    if (execution.session.checkpoint())
      return { status: PlanBuildStatus.BudgetExceeded };
    const candidates: { readonly order: number; readonly node: PlanNode }[] =
      [];
    let expandedAny = false;
    for (const current of frontier) {
      const node = current.node;
      if (execution.session.checkpoint())
        return { status: PlanBuildStatus.BudgetExceeded };
      if (
        node.game.winnerColor() !== undefined ||
        node.game.activeColor !== perspective
      ) {
        terminal.push(node);
        continue;
      }
      const nextSeeds = generateActionSeeds(
        execution,
        node.game,
        perspective,
        config,
        seedCap,
      );
      if (execution.session.checkpoint())
        return { status: PlanBuildStatus.BudgetExceeded };
      if (nextSeeds.length === 0) {
        terminal.push(node);
        continue;
      }
      const nextPool = new TransitionCompilePool(
        execution,
        node.game,
        nextSeeds,
        config,
      );
      if (execution.session.checkpoint())
        return { status: PlanBuildStatus.BudgetExceeded };
      let nodeExpanded = false;
      for (const seed of nextSeeds) {
        if (execution.session.checkpoint())
          return { status: PlanBuildStatus.BudgetExceeded };
        const compiled = compileActionFromPool(
          execution,
          node.game,
          perspective,
          seed.action,
          nextPool,
        );
        if (compiled === undefined) continue;
        if (execution.session.checkpoint())
          return { status: PlanBuildStatus.BudgetExceeded };
        expansions += 1;
        if (expansions > expansionCap) {
          return { status: PlanBuildStatus.BudgetExceeded };
        }
        const [after, chunk] = compiled;
        const actions = [...node.actions, seed.action];
        const chunks = [...node.compiledChunks, chunk];
        const hash = exactSearchStateHash(after);
        const order = quickOrderScoreWithSearchHash(
          utilityContext,
          after,
          hash,
          node.goalFamily,
          actions.length,
        );
        if (execution.session.cancelled)
          return { status: PlanBuildStatus.BudgetExceeded };
        const existing = seen.get(hash);
        if (existing !== undefined && order <= existing) continue;
        seen.set(hash, order);
        candidates.push({
          order,
          node: {
            game: after,
            stateHash: hash,
            actions,
            compiledChunks: chunks,
            headUtility: node.headUtility,
            headFamily: node.headFamily,
            goalFamily: node.goalFamily,
          },
        });
        expandedAny = true;
        nodeExpanded = true;
      }
      if (!nodeExpanded) terminal.push(node);
    }
    if (!expandedAny || candidates.length === 0) {
      frontier = [];
      break;
    }
    candidates.sort(compareOrderedNodes);
    frontier = candidates.slice(0, Math.max(beamWidth, 1));
  }

  terminal.push(...frontier.map(({ node }) => node));
  if (execution.session.checkpoint())
    return { status: PlanBuildStatus.BudgetExceeded };
  if (terminal.length === 0) return { status: PlanBuildStatus.NoPlan };
  const plans = terminal.map<TurnPlan>((node) => ({
    actions: [...node.actions],
    compiledChunks: node.compiledChunks.map((chunk) => chunk.slice()),
    endGame: node.game.fork(),
    utility: evaluateStateUtilityWithSearchHash(
      utilityContext,
      node.game,
      node.stateHash,
    ),
    headUtility: node.headUtility,
    headFamily: node.headFamily,
    goalFamily: node.goalFamily,
    packageMeta: EMPTY_PACKAGE_META,
  }));
  if (execution.session.checkpoint())
    return { status: PlanBuildStatus.BudgetExceeded };
  plans.sort((left, right) => -turnEngineComparePlans(left, right));
  return { status: "ok", plans };
}

function compareOrderedNodes(
  left: { readonly order: number; readonly node: PlanNode },
  right: { readonly order: number; readonly node: PlanNode },
): number {
  const order = compareNumber(right.order, left.order);
  return order !== 0
    ? order
    : compareChunks(left.node.compiledChunks, right.node.compiledChunks);
}

export function fallbackSingleActionPlan(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: TurnEngineConfig,
  utilityContext: TurnUtilityEvalContext,
): TurnPlan | undefined {
  if (execution.session.checkpoint()) return undefined;
  let seeds = generateActionSeeds(
    execution,
    game,
    perspective,
    config,
    Math.min(Math.max(config.ownSeedCap, 1) * 2, TURN_ENGINE_COMPILE_LIMIT_MAX),
  );
  if (execution.session.checkpoint()) return undefined;
  if (seeds.length === 0)
    seeds = fallbackWalkSeeds(execution, game, perspective);
  if (seeds.length === 0 || execution.session.checkpoint()) return undefined;
  const pool = new TransitionCompilePool(execution, game, seeds, config);
  let best: TurnPlan | undefined;
  for (const seed of seeds) {
    if (execution.session.checkpoint()) return undefined;
    const compiled = compileActionFromPool(
      execution,
      game,
      perspective,
      seed.action,
      pool,
    );
    if (compiled === undefined) continue;
    if (execution.session.checkpoint()) return undefined;
    const [after, chunk] = compiled;
    const afterHash = exactSearchStateHash(after);
    const stateUtility = evaluateStateUtilityWithSearchHash(
      utilityContext,
      after,
      afterHash,
    );
    const plan: TurnPlan = {
      actions: [seed.action],
      compiledChunks: [chunk.slice()],
      endGame: after.fork(),
      utility: stateUtility,
      headUtility: stateUtility,
      headFamily: seed.family,
      goalFamily: seed.family,
      packageMeta: EMPTY_PACKAGE_META,
    };
    if (execution.session.cancelled) return undefined;
    plan.utility = evaluatePlanWithReplies(
      execution,
      plan,
      perspective,
      config,
      utilityContext,
    );
    const sessionAfterReplyEvaluation = execution.session;
    if (sessionAfterReplyEvaluation.cancelled) return undefined;
    if (best === undefined || turnEngineComparePlans(plan, best) > 0) {
      best = plan;
    }
  }
  return execution.session.checkpoint() ? undefined : best;
}

export function fallbackSingleActionPlanFromAllowedHeads(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: TurnEngineConfig,
  utilityContext: TurnUtilityEvalContext,
  allowedFirstSteps: readonly (readonly Input[])[],
): TurnPlan | undefined {
  if (execution.session.checkpoint()) return undefined;
  let seeds = generateActionSeeds(
    execution,
    game,
    perspective,
    config,
    Math.min(Math.max(config.ownSeedCap, 1) * 2, TURN_ENGINE_COMPILE_LIMIT_MAX),
  );
  if (execution.session.checkpoint()) return undefined;
  if (seeds.length === 0)
    seeds = fallbackWalkSeeds(execution, game, perspective);
  if (execution.session.checkpoint() || seeds.length === 0) return undefined;

  const ranks = allowedRankMap(allowedFirstSteps);
  const pool = new TransitionCompilePool(execution, game, seeds, config);
  if (execution.session.checkpoint()) return undefined;
  let best: { plan: TurnPlan; meta: AllowedHeadSelectionMeta } | undefined;
  for (const seed of seeds) {
    if (execution.session.checkpoint()) return undefined;
    const compiled = compileActionFromPool(
      execution,
      game,
      perspective,
      seed.action,
      pool,
    );
    if (compiled === undefined) continue;
    if (execution.session.checkpoint()) return undefined;
    const [after, chunk] = compiled;
    const rank = ranks.get(inputChainKey(chunk));
    if (rank === undefined) continue;

    const afterHash = exactSearchStateHash(after);
    const stateUtility = evaluateStateUtilityWithSearchHash(
      utilityContext,
      after,
      afterHash,
    );
    const plan: TurnPlan = {
      actions: [seed.action],
      compiledChunks: [chunk.slice()],
      endGame: after.fork(),
      utility: stateUtility,
      headUtility: stateUtility,
      headFamily: seed.family,
      goalFamily: seed.family,
      packageMeta: EMPTY_PACKAGE_META,
    };
    if (execution.session.cancelled) return undefined;
    plan.utility = evaluatePlanWithReplies(
      execution,
      plan,
      perspective,
      config,
      utilityContext,
    );
    const sessionAfterAllowedReplyEvaluation = execution.session;
    if (sessionAfterAllowedReplyEvaluation.cancelled) return undefined;
    const meta: AllowedHeadSelectionMeta = {
      rank,
      allowedLength: allowedFirstSteps.length,
      firstStepOpponentImmediateLoss: opponentCanWinImmediatelyWithSearchHash(
        execution,
        after,
        perspective,
        afterHash,
      ),
      firstStepDrainerSafety: ownDrainerSafetyScore(
        execution,
        after.board,
        perspective,
      ),
    };
    if (
      best === undefined ||
      compareAllowedHeadPlans(plan, meta, best.plan, best.meta) > 0
    ) {
      best = { plan, meta };
    }
  }

  return execution.session.checkpoint() ? undefined : best?.plan;
}

export function replyShortlistLength(total: number, beam: number): number {
  return Math.min(total, Math.min(Math.max(Math.max(beam, 0) * 2, 4), 8));
}

export function evaluatePlanWithReplies(
  execution: AutomoveExecutionContext,
  plan: TurnPlan,
  perspective: Color,
  config: TurnEngineConfig,
  utilityContext: TurnUtilityEvalContext,
): TurnUtility {
  if (execution.session.checkpoint()) return EMPTY_TURN_UTILITY;
  const after = plan.endGame;
  const afterHash = exactSearchStateHash(after);
  const opponent = otherColor(perspective);
  if (after.winnerColor() !== undefined || after.activeColor !== opponent) {
    return evaluateStateUtilityWithSearchHash(utilityContext, after, afterHash);
  }

  const opponentConfig: TurnEngineConfig = {
    ...config,
    ownSeedCap: Math.max(config.opponentSeedCap, 1),
    ownBeam: Math.max(config.opponentBeam, 1),
    perNodeFamilyCap: Math.max(config.perNodeFamilyCap, 1),
    stepCap: Math.min(Math.max(config.stepCap, 1), 4),
    opponentSeedCap: Math.max(config.replySeedCap, 1),
    opponentBeam: Math.max(config.replyBeam, 1),
    replySeedCap: 0,
    replyBeam: 0,
    expansionCap: Math.max(Math.trunc(config.expansionCap / 2), 24),
  };
  const opponentUtilityContext = createTurnUtilityEvalContextWithSearchHash(
    execution,
    after,
    afterHash,
    opponent,
    opponentConfig,
  );
  const opponentResult = generatePlansForMode(
    execution,
    after,
    opponent,
    opponentConfig,
    opponentUtilityContext,
    opponentConfig.ownSeedCap,
    opponentConfig.ownBeam,
    opponentConfig.stepCap,
    opponentConfig.expansionCap,
  );
  if (execution.session.checkpoint()) return EMPTY_TURN_UTILITY;
  if (opponentResult.status !== "ok")
    return evaluateStateUtilityWithSearchHash(utilityContext, after, afterHash);
  const opponentPlans = opponentResult.plans;

  const shortlist = replyShortlistLength(
    opponentPlans.length,
    opponentConfig.ownBeam,
  );
  let bestOpponent = opponentPlans[0];
  if (bestOpponent === undefined)
    return evaluateStateUtilityWithSearchHash(utilityContext, after, afterHash);
  let bestOpponentHash = exactSearchStateHash(bestOpponent.endGame);
  let bestOpponentUtility = evaluateStateUtilityWithSearchHash(
    opponentUtilityContext,
    bestOpponent.endGame,
    bestOpponentHash,
  );
  for (const opponentPlan of opponentPlans.slice(1, shortlist)) {
    if (execution.session.checkpoint()) return EMPTY_TURN_UTILITY;
    const opponentPlanHash = exactSearchStateHash(opponentPlan.endGame);
    const utility = evaluateStateUtilityWithSearchHash(
      opponentUtilityContext,
      opponentPlan.endGame,
      opponentPlanHash,
    );
    const utilityOrder = compareTurnUtilities(utility, bestOpponentUtility);
    if (
      utilityOrder > 0 ||
      (utilityOrder === 0 &&
        compareChunks(
          opponentPlan.compiledChunks,
          bestOpponent.compiledChunks,
        ) < 0)
    ) {
      bestOpponent = opponentPlan;
      bestOpponentHash = opponentPlanHash;
      bestOpponentUtility = utility;
    }
  }

  const afterOpponent = bestOpponent.endGame;
  if (
    afterOpponent.winnerColor() !== undefined ||
    afterOpponent.activeColor !== perspective ||
    config.replySeedCap === 0
  ) {
    return evaluateStateUtilityWithSearchHash(
      utilityContext,
      afterOpponent,
      bestOpponentHash,
    );
  }

  const replyConfig: TurnEngineConfig = {
    ...config,
    ownSeedCap: Math.max(config.replySeedCap, 1),
    ownBeam: Math.max(config.replyBeam, 1),
    perNodeFamilyCap: Math.max(config.perNodeFamilyCap, 1),
    stepCap: Math.min(Math.max(config.stepCap, 1), 3),
    opponentSeedCap: 0,
    opponentBeam: 0,
    replySeedCap: 0,
    replyBeam: 0,
    expansionCap: Math.max(Math.trunc(config.expansionCap / 3), 16),
  };
  const replyUtilityContext = createTurnUtilityEvalContextWithSearchHash(
    execution,
    afterOpponent,
    bestOpponentHash,
    perspective,
    replyConfig,
  );
  const replyResult = generatePlansForMode(
    execution,
    afterOpponent,
    perspective,
    replyConfig,
    replyUtilityContext,
    replyConfig.ownSeedCap,
    replyConfig.ownBeam,
    replyConfig.stepCap,
    replyConfig.expansionCap,
  );
  if (execution.session.checkpoint()) return EMPTY_TURN_UTILITY;
  if (replyResult.status !== "ok") {
    return evaluateStateUtilityWithSearchHash(
      utilityContext,
      afterOpponent,
      bestOpponentHash,
    );
  }
  const replyPlans = replyResult.plans;
  let bestReply: TurnUtility | undefined;
  for (const reply of replyPlans.slice(
    0,
    replyShortlistLength(replyPlans.length, replyConfig.ownBeam),
  )) {
    const replyHash = exactSearchStateHash(reply.endGame);
    const utility = evaluateStateUtilityWithSearchHash(
      utilityContext,
      reply.endGame,
      replyHash,
    );
    if (
      bestReply === undefined ||
      compareTurnUtilities(utility, bestReply) > 0
    ) {
      bestReply = utility;
    }
  }
  return (
    bestReply ??
    evaluateStateUtilityWithSearchHash(
      utilityContext,
      afterOpponent,
      bestOpponentHash,
    )
  );
}
