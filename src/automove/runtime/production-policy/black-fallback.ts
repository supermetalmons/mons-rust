import { Color, Modifier } from "../../../api/types.js";
import { inputChainsEqual, type Input } from "../../../engine/model/domain.js";
import type { MonsGame } from "../../../engine/game/mons-game.js";
import type { AutomoveExecutionContext } from "../../core/execution-context.js";
import { automoveConfigForGame } from "../../config/runtime.js";
import { selectSearchInputs } from "../search-selection.js";

export function selectLateBlackFallbackInputs(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  productionInputs: readonly Input[],
): Input[] | undefined {
  if (productionInputs.length === 0) return undefined;
  const transitionTurn =
    game.activeColor === Color.Black &&
    game.turnNumber === 4 &&
    game.monsMovesCount === 2 &&
    game.playerCanUseAction() &&
    game.playerCanMoveMana();
  const midTurn =
    game.activeColor === Color.Black &&
    game.turnNumber >= 4 &&
    game.monsMovesCount >= 3 &&
    game.playerCanUseAction() &&
    game.playerCanMoveMana();
  if (!transitionTurn && !midTurn) return undefined;
  const inputs = selectSearchInputs(
    execution,
    game,
    automoveConfigForGame(game, "pro"),
  );
  if (inputs.length === 0 || inputChainsEqual(inputs, productionInputs)) {
    return undefined;
  }
  if (
    transitionTurn &&
    inputs.length === 3 &&
    inputs[2]?.kind === "modifier" &&
    inputs[2].modifier === Modifier.SelectBomb
  ) {
    return inputs;
  }
  return midTurn ? inputs : undefined;
}
