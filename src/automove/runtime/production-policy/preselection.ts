import { Color } from "../../../api/types.js";
import type { Input } from "../../../engine/model/domain.js";
import type { MonsGame } from "../../../engine/game/mons-game.js";
import { exactOpportunityContext } from "../../exact/turn-opportunity.js";
import type { AutomoveExecutionContext } from "../../core/execution-context.js";
import { automoveConfigForGame, withProductionPlanner } from "../../config/runtime.js";
import type { AutomoveConfig } from "../../config/types.js";
import {
  selectSearchInputs,
  selectSearchInputsWithFreshPlanCache,
} from "../search-selection.js";
import { ownDrainerUnsafe } from "./support.js";

export function selectEarlyWhiteFallbackInputs(
  execution: AutomoveExecutionContext,
  game: MonsGame,
): Input[] | undefined {
  const broadFallback =
    (game.activeColor === Color.White &&
      game.turnNumber <= 3 &&
      !game.playerCanUseAction() &&
      !game.playerCanMoveMana() &&
      (game.monsMovesCount === 0 || game.monsMovesCount === 3)) ||
    (game.activeColor === Color.White &&
      game.turnNumber === 1 &&
      game.monsMovesCount === 2 &&
      !game.playerCanUseAction() &&
      !game.playerCanMoveMana()) ||
    (game.activeColor === Color.White &&
      game.turnNumber === 3 &&
      game.monsMovesCount === 0 &&
      game.playerCanUseAction() &&
      game.playerCanMoveMana()) ||
    (game.activeColor === Color.White &&
      game.turnNumber === 3 &&
      game.monsMovesCount >= 3 &&
      game.playerCanUseAction() &&
      game.playerCanMoveMana());
  if (broadFallback) {
    return selectSearchInputs(execution, game, automoveConfigForGame(game, "pro"));
  }

  const manaOnly =
    game.activeColor === Color.White &&
    game.turnNumber === 3 &&
    game.monsMovesCount === 1 &&
    !game.playerCanUseAction() &&
    game.playerCanMoveMana();
  const midTurn =
    game.activeColor === Color.White &&
    game.turnNumber === 3 &&
    game.monsMovesCount > 0 &&
    !manaOnly &&
    (game.playerCanUseAction() || game.playerCanMoveMana());
  if (!midTurn || !ownDrainerUnsafe(execution, game)) return undefined;
  return selectSearchInputs(execution, game, automoveConfigForGame(game, "fast"));
}

export function selectScoreWindowTacticalFallbackInputs(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  base: AutomoveConfig,
): Input[] | undefined {
  const eligible =
    game.activeColor === Color.White &&
    game.turnNumber === 3 &&
    (game.monsMovesCount === 1 || game.monsMovesCount === 2) &&
    game.playerCanUseAction() &&
    game.playerCanMoveMana();
  if (!eligible) return undefined;
  const context = exactOpportunityContext(execution, game, game.activeColor);
  if (context.delta.sameTurnScoreWindowValue <= 0) return undefined;
  return selectSearchInputsWithFreshPlanCache(
    execution,
    game,
    withProductionPlanner(base),
  );
}

export function selectUnconditionalBlackFallbackInputs(
  execution: AutomoveExecutionContext,
  game: MonsGame,
): Input[] | undefined {
  const eligible =
    (game.activeColor === Color.Black &&
      game.turnNumber === 2 &&
      game.monsMovesCount === 0 &&
      game.playerCanUseAction() &&
      game.playerCanMoveMana()) ||
    (game.activeColor === Color.Black &&
      game.turnNumber === 2 &&
      game.monsMovesCount > 0 &&
      !game.playerCanUseAction() &&
      game.playerCanMoveMana()) ||
    (game.activeColor === Color.Black &&
      game.turnNumber === 4 &&
      game.monsMovesCount === 0 &&
      game.playerCanUseAction() &&
      game.playerCanMoveMana());
  return eligible
    ? selectSearchInputs(execution, game, automoveConfigForGame(game, "pro"))
    : undefined;
}
