import { inputChainKey, type Input } from "../../engine/model/domain.js";
import type { Color } from "../../api/types.js";
import { MonsGame } from "../../engine/game/mons-game.js";
import type { AutomoveExecutionContext } from "../core/execution-context.js";
import { exactSearchStateHash } from "../exact/hash.js";
import { TransitionCompilePool, compileActionFromPool } from "./compiler.js";
import {
  evaluateStateUtilityWithSearchHash,
  opponentCanWinImmediatelyWithSearchHash,
  ownDrainerSafetyScore,
  type TurnUtilityEvalContext,
} from "./evaluation.js";
import {
  EMPTY_PACKAGE_META,
  TURN_ENGINE_COMPILE_LIMIT_MAX,
  type TurnEngineConfig,
  type TurnPlan,
} from "./model.js";
import { fallbackWalkSeeds } from "./opportunity-seeds-progress.js";
import { generateActionSeeds } from "./opportunities.js";
import { turnEngineComparePlans } from "./ordering.js";
import {
  allowedRankMap,
  compareAllowedHeadPlans,
  type AllowedHeadSelectionMeta,
} from "./planner-allowed-heads.js";
import { evaluatePlanWithReplies } from "./planner-replies.js";

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
  if (seeds.length === 0) seeds = fallbackWalkSeeds(execution, game, perspective);
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
  if (seeds.length === 0) seeds = fallbackWalkSeeds(execution, game, perspective);
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
