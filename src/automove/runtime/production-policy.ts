import type { Input } from "../../engine/model/domain.js";
import type { MonsGame } from "../../engine/game/mons-game.js";
import type { AutomoveExecutionContext } from "../core/execution-context.js";
import type { AutomoveConfig } from "../config/types.js";
import { selectSearchInputsWithFreshPlanCache } from "./search-selection.js";
import {
  PRODUCTION_FALLBACK_GUARDS,
  PRODUCTION_PRESELECTION_GUARDS,
} from "./production-policy/guards.js";

export function selectProductionPolicyInputs(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  base: AutomoveConfig,
  production: AutomoveConfig,
): Input[] {
  for (const guard of PRODUCTION_PRESELECTION_GUARDS) {
    if (execution.session.checkpoint()) return [];
    const result = guard.evaluate(execution, game, base);
    if (result.kind === "select") return [...result.inputs];
  }

  const productionInputs = selectSearchInputsWithFreshPlanCache(
    execution,
    game,
    production,
  );
  if (execution.session.checkpoint()) return [];

  for (const guard of PRODUCTION_FALLBACK_GUARDS) {
    if (execution.session.checkpoint()) return [];
    const result = guard.evaluate(execution, game, base, productionInputs);
    if (result.kind === "select") return [...result.inputs];
  }
  return productionInputs;
}
