import type { Event, Input } from "../../engine/model/domain.js";
import type { MonsGame } from "../../engine/game/mons-game.js";

export function applyInputsForSearch(
  game: MonsGame,
  inputs: readonly Input[],
): MonsGame | undefined {
  return applyInputsForSearchWithEvents(game, inputs)?.game;
}

export function applyInputsForSearchWithEvents(
  game: MonsGame,
  inputs: readonly Input[],
): { game: MonsGame; events: readonly Event[] } | undefined {
  const simulatedGame = game.fork();
  const output = simulatedGame.processInput(inputs, false, false);
  return output.kind === "events"
    ? { game: simulatedGame, events: output.events }
    : undefined;
}
