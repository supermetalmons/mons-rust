import type { Input } from "../../engine/model/domain.js";
import { parseInputArrayFen } from "../../engine/codec/input.js";
import type { MonsGame } from "../../engine/game/mons-game.js";
import type { AutomoveExecutionContext } from "../core/execution-context.js";
import {
  clearProductionSelectorCaches,
  smartSearchBestInputs,
} from "../policy/production/selector.js";
import type { AutomoveConfig } from "../config/types.js";
import { clearTurnEnginePlanCache } from "../turn/cache.js";
import { randomAutomove } from "./input-selection.js";

export function clearTimedOutSelectionCaches(
  execution: AutomoveExecutionContext,
): void {
  clearProductionSelectorCaches(execution);
}

export function selectSearchInputs(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  config: AutomoveConfig,
): Input[] {
  if (execution.session.checkpoint()) return [];
  const inputs = [...smartSearchBestInputs(execution, game, config, true)];
  if (inputs.length > 0) return inputs;
  if (execution.session.checkpoint()) return [];
  const random = randomAutomove(execution, game.fork());
  return random.output.kind === "events"
    ? (parseInputArrayFen(random.inputFen) ?? [])
    : [];
}

export function selectSearchInputsWithFreshPlanCache(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  config: AutomoveConfig,
): Input[] {
  if (execution.session.checkpoint()) return [];
  if (config.planner.enabled) clearTurnEnginePlanCache(execution);
  return selectSearchInputs(execution, game, config);
}
