import { inputChainKey, type Input } from "../../engine/model/domain.js";
import type { Color } from "../../api/types.js";
import { MonsGame } from "../../engine/game/mons-game.js";
import type { AutomoveExecutionContext } from "../core/execution-context.js";
import { applyInputsForSearch } from "../transitions/simulation.js";
import { compareInputChains } from "../transitions/order.js";
import { cacheKey, cacheKeyForMode, type TurnCacheKey } from "./fingerprint.js";
import {
  turnCacheAdd,
  turnCacheDelete,
  turnCacheGet,
  turnCacheHas,
  turnCacheSet,
  turnEngineCaches,
} from "./cache.js";
import {
  createTurnUtilityEvalContext,
  type TurnUtilityEvalContext,
} from "./evaluation.js";
import {
  EMPTY_TURN_UTILITY,
  PlanBuildStatus,
  TurnEngineMode,
  type PlanBuildResult,
  type TurnEngineConfig,
  type TurnPlan,
  type TurnUtility,
} from "./model.js";
import { copyPlan } from "./ordering.js";
import {
  allowedHeadSelectionMeta,
  allowedRankMap,
  compareAllowedHeadPlans,
  type AllowedHeadSelectionMeta,
} from "./planner-allowed-heads.js";
import { fallbackSingleActionPlanFromAllowedHeads } from "./planner-fallback.js";
import { generatePlansForMode } from "./planner-generation.js";
import { buildBestPlan } from "./planner-orchestration.js";
import { evaluatePlanWithReplies } from "./planner-replies.js";

function cachedBestPlanIfLegal(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  key: TurnCacheKey,
): TurnPlan | undefined {
  if (execution.session.checkpoint()) return undefined;
  const cached = turnCacheGet(turnEngineCaches(execution).bestPlan, key);
  if (cached === undefined) return undefined;
  const first = cached.compiledChunks[0];
  const legal = first !== undefined && applyInputsForSearch(game, first) !== undefined;
  if (execution.session.checkpoint()) return undefined;
  if (!legal) {
    if (execution.session.cacheWriteAllowed)
      turnCacheDelete(turnEngineCaches(execution).bestPlan, key);
    return undefined;
  }
  return copyPlan(cached);
}

function cachedStepIfLegal(
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
  if (execution.session.checkpoint() || !execution.session.cacheWriteAllowed) return;
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
  if (cached !== undefined) return execution.session.checkpoint() ? undefined : cached;

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
      turnCacheSet(turnEngineCaches(execution).bestPlan, key, copyPlan(result.plan));
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

function bestPlanFromAllowedHeads(
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

function buildBestPlanFromAllowedHeads(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: TurnEngineConfig,
  allowedFirstSteps: readonly (readonly Input[])[],
): PlanBuildResult {
  if (execution.session.checkpoint()) return { status: PlanBuildStatus.BudgetExceeded };
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
  if (execution.session.checkpoint()) return { status: PlanBuildStatus.BudgetExceeded };
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
  if (execution.session.checkpoint()) return { status: PlanBuildStatus.BudgetExceeded };
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

function registerPlanContinuations(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  mode: TurnEngineMode,
  plan: TurnPlan,
  config: TurnEngineConfig,
): void {
  if (execution.session.checkpoint() || plan.compiledChunks.length === 0) return;
  let state = game.fork();
  const startColor = game.activeColor;
  for (let index = 0; index < plan.compiledChunks.length; index += 1) {
    if (execution.session.checkpoint()) return;
    const chunk = plan.compiledChunks[index];
    if (chunk === undefined) continue;
    if (index > 0 && mode === TurnEngineMode.Production) {
      const fresh = turnEngineCandidatePlan(execution, state, perspective, config);
      if (fresh?.compiledChunks[0] === undefined || execution.session.checkpoint())
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
