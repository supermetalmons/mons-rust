import type { Input } from "../../engine/model/domain.js";
import { inputArrayFen } from "../../engine/codec/input.js";
import type { MonsGame } from "../../engine/game/mons-game.js";
import type { AutomoveExecutionContext } from "../core/execution-context.js";
import type { SmartAutomovePreference } from "../config/types.js";
import { applyInputsForSearchWithEvents } from "../transitions/simulation.js";
import {
  selectProductionInputsWithDeadline,
  selectStrategicSearchInputsWithDeadline,
} from "./deadline-selection.js";
import { deterministicLegalFallbackInputs } from "./input-selection.js";
import type { AutomoveSuggestion } from "./types.js";

export function suggestMove(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  preference: SmartAutomovePreference,
): AutomoveSuggestion {
  const sourceFen = game.fen();
  const selected =
    preference === "pro"
      ? selectProductionInputsWithDeadline(execution, game)
      : selectStrategicSearchInputsWithDeadline(execution, game, preference);
  const inputs: readonly Input[] =
    selected.length === 0 ? deterministicLegalFallbackInputs(game) : selected;
  let applied = applyInputsForSearchWithEvents(game, inputs);
  let appliedInputs = inputs;
  if (applied === undefined) {
    appliedInputs = deterministicLegalFallbackInputs(game);
    applied = applyInputsForSearchWithEvents(game, appliedInputs);
  }
  if (game.fen() !== sourceFen) {
    throw new Error("automove suggestion mutated its source game");
  }
  return applied === undefined
    ? { output: { kind: "invalid-input" }, inputFen: "" }
    : {
        output: { kind: "events", events: applied.events },
        inputFen: inputArrayFen(appliedInputs),
      };
}
