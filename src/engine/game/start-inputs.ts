import { BOARD_CELLS, locationIndex, type Location } from "../board/geometry.js";
import type { Input, Output } from "../model/domain.js";
import { shouldSuggestRegularManaStarts } from "../rules/legality.js";
import type { RulesState } from "../rules/state.js";
import type { SuggestedStartInputOptions } from "./input-support.js";

type StartInputResolver = (inputs: readonly Input[]) => Output;

export function suggestedInputToStartWith(
  state: RulesState,
  options: SuggestedStartInputOptions,
  resolve: StartInputResolver,
): Output {
  const suggestedLocations: Location[] = [];
  const seenLocations = Array.from({ length: BOARD_CELLS }, () => false);

  const addWhenValid = (at: Location): void => {
    const output = resolve([{ kind: "location", location: at }]);
    if (output.kind !== "next-input-options" || output.nextInputs.length === 0) {
      return;
    }
    const index = locationIndex(at);
    if (seenLocations[index]) return;
    seenLocations[index] = true;
    suggestedLocations.push(at);
  };

  for (const at of state.board.allMonsLocations(state.activeColor)) {
    addWhenValid(at);
  }

  if (
    shouldSuggestRegularManaStarts(
      state,
      suggestedLocations.length !== 0,
      options.includeManaStartsWithPotionAction,
    )
  ) {
    for (const at of state.board.allFreeRegularManaLocations(state.activeColor)) {
      addWhenValid(at);
    }
  }

  return suggestedLocations.length === 0
    ? { kind: "invalid-input" }
    : { kind: "locations-to-start-from", locations: suggestedLocations };
}
