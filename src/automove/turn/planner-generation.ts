import type { Color } from "../../api/types.js";
import { MonsGame } from "../../engine/game/mons-game.js";
import type { AutomoveExecutionContext } from "../core/execution-context.js";
import { type TurnUtilityEvalContext } from "./evaluation.js";
import {
  PlanBuildStatus,
  TurnEngineMode,
  type PlanGenerationResult,
  type TurnEngineConfig,
} from "./model.js";
import { bundlePlanCapForConfig } from "./planner-macro-policy.js";
import { generateMacroPlans } from "./planner-macro.js";
import { generateTurnPlans } from "./planner-normal.js";

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
  if (execution.session.checkpoint()) return { status: PlanBuildStatus.BudgetExceeded };
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
