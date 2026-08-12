import type { Color } from "../../../api/types.js";
import type { MonsGame } from "../../../engine/game/mons-game.js";
import type { AutomoveExecutionContext } from "../../core/execution-context.js";
import { rankRootCandidates } from "../../root/candidates.js";
import type { AutomoveConfig } from "../../config/types.js";
import {
  TurnEngineMode,
  type TurnEngineConfig,
  type TurnPlan,
} from "../../turn/model.js";
import {
  turnEngineCommitPlan,
  turnEngineStoreCachedStep,
} from "../../turn/planner-cache.js";
import { applyInputsForSearch } from "../../transitions/simulation.js";
import { turnEngineConfigForGame, turnEngineModeUsesMacroPlans } from "./config.js";
import { shouldResumeTurnEngineCachedStep } from "./tactical-prepass.js";

function seedTurnEngineFollowupCacheIfSafe(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: AutomoveConfig,
  mode: TurnEngineMode,
  plan: TurnPlan,
): void {
  if (!turnEngineModeUsesMacroPlans(mode) || plan.compiledChunks.length < 2) {
    return;
  }
  const first = plan.compiledChunks[0];
  const second = plan.compiledChunks[1];
  if (first === undefined || second === undefined) return;
  const afterFirst = applyInputsForSearch(game, first);
  if (afterFirst?.activeColor !== perspective) return;
  const afterConfig = turnEngineConfigForGame(afterFirst, config);
  const roots = rankRootCandidates(execution, afterFirst, perspective, config);
  if (!shouldResumeTurnEngineCachedStep(roots, second, mode)) return;
  turnEngineStoreCachedStep(execution, afterFirst, mode, afterConfig, second);
}

export function commitPlanAndSeedFollowup(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: AutomoveConfig,
  mode: TurnEngineMode,
  plan: TurnPlan,
  engineConfig: TurnEngineConfig,
): void {
  turnEngineCommitPlan(execution, game, perspective, mode, plan, engineConfig);
  seedTurnEngineFollowupCacheIfSafe(execution, game, perspective, config, mode, plan);
}
