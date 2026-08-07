import type { Input } from "../engine/domain.js";
import { inputArrayFen } from "../engine/fen.js";
import type { MonsGame } from "../engine/game.js";
import type { AutomoveExecutionContext } from "./execution-context.js";
import { automoveConfigForGame } from "./selector-config.js";
import type { SmartAutomovePreference } from "./selector-types.js";
import { applyInputsForSearchWithEvents } from "./transitions.js";
import {
  selectProductionInputsWithDeadline,
  selectStrategicSearchInputsWithDeadline,
} from "./runtime/deadline-selection.js";
import { deterministicLegalFallbackInputs } from "./runtime/input-selection.js";
import type { AutomoveSuggestion } from "./runtime/types.js";

export type { AutomoveSuggestion } from "./runtime/types.js";

/** Choose and preview one legal move without mutating the source game. */
export function suggestMove(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  preference: SmartAutomovePreference,
): AutomoveSuggestion {
  const sourceFen = game.fen();
  const selected =
    preference === "pro"
      ? selectProductionInputsWithDeadline(execution, game)
      : selectStrategicSearchInputsWithDeadline(
          execution,
          game,
          automoveConfigForGame(game, preference),
        );
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
