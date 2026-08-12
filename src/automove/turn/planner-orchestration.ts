import { inputChainKey } from "../../engine/model/domain.js";
import type { Color } from "../../api/types.js";
import { MonsGame } from "../../engine/game/mons-game.js";
import type { AutomoveExecutionContext } from "../core/execution-context.js";
import { createTurnUtilityEvalContext } from "./evaluation.js";
import {
  PlanBuildStatus,
  TurnEngineMode,
  type PlanBuildResult,
  type TurnEngineConfig,
  type TurnPlan,
} from "./model.js";
import { turnEngineComparePlans } from "./ordering.js";
import { fallbackSingleActionPlan } from "./planner-fallback.js";
import { generatePlansForMode } from "./planner-generation.js";
import { evaluatePlanWithReplies } from "./planner-replies.js";

export function buildBestPlan(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: TurnEngineConfig,
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
    if (execution.session.cancelled) return { status: PlanBuildStatus.BudgetExceeded };
    if (best === undefined || turnEngineComparePlans(plan, best) > 0) best = plan;
  }
  if (execution.session.checkpoint()) return { status: PlanBuildStatus.BudgetExceeded };
  return best === undefined
    ? { status: PlanBuildStatus.NoPlan }
    : { status: "ok", plan: best };
}
