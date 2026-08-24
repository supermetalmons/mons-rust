import { inputArrayFen } from "../engine/codec/input.js";
import { FOR_AUTOMOVE_START_INPUT_OPTIONS } from "../engine/game/input-support.js";
import type { MonsGame } from "../engine/game/mons-game.js";
import {
  MAX_INPUTS_PER_MOVE,
  MODIFIER_RANK,
  type Input,
  type Output,
} from "../engine/model/domain.js";
import {
  selectPackedInputs,
  type AutomoveClock,
  type AutomovePreference,
} from "./selector.js";

type AutomoveSuggestion = {
  readonly output: Output;
  readonly inputFen: string;
};

function inputTag(input: Input): number {
  switch (input.kind) {
    case "takeback":
      return 0;
    case "location":
      return 1;
    case "modifier":
      return 2;
  }
}

function compareInputs(left: Input, right: Input): number {
  const tagOrder = inputTag(left) - inputTag(right);
  if (tagOrder !== 0) return tagOrder;
  if (left.kind === "location" && right.kind === "location") {
    return left.location.i - right.location.i || left.location.j - right.location.j;
  }
  if (left.kind === "modifier" && right.kind === "modifier") {
    return MODIFIER_RANK[left.modifier] - MODIFIER_RANK[right.modifier];
  }
  return 0;
}

function nextInputs(output: Output): Input[] | undefined {
  switch (output.kind) {
    case "invalid-input":
    case "events":
      return undefined;
    case "locations-to-start-from":
      return output.locations.map((location) => ({ kind: "location", location }));
    case "next-input-options":
      return output.nextInputs.map(({ input }) => input);
  }
}

function applyInputs(
  game: MonsGame,
  inputs: readonly Input[],
): Extract<Output, { readonly kind: "events" }> | undefined {
  const output = game.fork().processInput(inputs, false, false);
  return output.kind === "events" ? output : undefined;
}

export function deterministicLegalFallbackInputs(game: MonsGame): Input[] {
  const simulated = game.fork();
  const inputs: Input[] = [];
  const findCompletion = (): boolean => {
    const output =
      inputs.length === 0
        ? simulated.processInputWithStartOptions(
            inputs,
            true,
            false,
            FOR_AUTOMOVE_START_INPUT_OPTIONS,
          )
        : simulated.inspectInputGrammar(inputs, FOR_AUTOMOVE_START_INPUT_OPTIONS);
    if (output.kind === "invalid-input") return false;
    if (output.kind === "events") {
      return inputs.length > 0 && applyInputs(simulated, inputs) !== undefined;
    }
    if (inputs.length === MAX_INPUTS_PER_MOVE) return false;

    const choices = nextInputs(output)?.sort(compareInputs);
    if (choices === undefined || choices.length === 0) return false;
    for (const choice of choices) {
      inputs.push(choice);
      if (findCompletion()) return true;
      inputs.pop();
    }
    return false;
  };

  return findCompletion() ? inputs : [];
}

export function suggestMove(
  game: MonsGame,
  preference: AutomovePreference,
  clock?: AutomoveClock,
): AutomoveSuggestion {
  const sourceFen = game.fen();
  const selected =
    clock === undefined
      ? selectPackedInputs(game, preference)
      : selectPackedInputs(game, preference, clock);
  let inputs =
    selected.kind === "selected"
      ? selected.inputs
      : deterministicLegalFallbackInputs(game);
  let output = applyInputs(game, inputs);
  if (output === undefined && selected.kind === "selected") {
    inputs = deterministicLegalFallbackInputs(game);
    output = applyInputs(game, inputs);
  }
  if (game.fen() !== sourceFen) {
    throw new Error("automove suggestion mutated its source game");
  }
  return output === undefined
    ? { output: { kind: "invalid-input" }, inputFen: "" }
    : { output, inputFen: inputArrayFen(inputs) };
}
