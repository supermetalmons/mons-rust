import {
  MAX_INPUTS_PER_MOVE,
  type Input,
  type Output,
} from "../../engine/domain.js";
import type { AutomoveExecutionContext } from "../execution-context.js";
import { inputArrayFen } from "../../engine/fen.js";
import {
  FOR_AUTOMOVE_START_INPUT_OPTIONS,
  type MonsGame,
} from "../../engine/game.js";
import {
  applyInputsForSearchWithEvents,
  compareInputs,
} from "../transitions.js";
import type { AutomoveSuggestion } from "./types.js";

/**
 * Select an index uniformly with uint32 rejection sampling.
 *
 * Exported from this internal module so boundary behavior can be tested without
 * widening the package API.
 */
export function randomIndex(
  execution: AutomoveExecutionContext,
  length: number,
): number {
  if (!Number.isSafeInteger(length) || length <= 0 || length > 0x1_0000_0000) {
    throw new RangeError(
      "random index requires a non-empty uint32-sized collection",
    );
  }

  const range = 0x1_0000_0000;
  const unbiasedUpperBound = range - (range % length);
  for (;;) {
    const value = execution.random.nextUint32();
    if (!Number.isInteger(value) || value < 0 || value >= range) {
      throw new RangeError("automove random source must return a uint32");
    }
    if (value < unbiasedUpperBound) return value % length;
  }
}

function nextInputsForPrompt(output: Output): Input[] | undefined {
  switch (output.kind) {
    case "invalid-input":
    case "events":
      return undefined;
    case "locations-to-start-from":
      return output.locations.map((at) => ({
        kind: "location",
        location: { i: at.i, j: at.j },
      }));
    case "next-input-options":
      return output.nextInputs.map((next) => next.input);
  }
}

export function randomAutomove(
  execution: AutomoveExecutionContext,
  game: MonsGame,
): AutomoveSuggestion {
  const inputs: Input[] = [];
  for (;;) {
    const output = game.processInputWithStartOptions(
      inputs,
      false,
      false,
      FOR_AUTOMOVE_START_INPUT_OPTIONS,
    );
    if (output.kind === "invalid-input") {
      return { output, inputFen: "" };
    }
    if (output.kind === "events") {
      return { output, inputFen: inputArrayFen(inputs) };
    }
    const choices = nextInputsForPrompt(output);
    if (choices === undefined || choices.length === 0) {
      return { output: { kind: "invalid-input" }, inputFen: "" };
    }
    if (inputs.length === MAX_INPUTS_PER_MOVE) {
      return { output: { kind: "invalid-input" }, inputFen: "" };
    }
    const choice = choices[randomIndex(execution, choices.length)];
    if (choice === undefined) {
      return { output: { kind: "invalid-input" }, inputFen: "" };
    }
    inputs.push(choice);
  }
}

export function deterministicLegalFallbackInputs(game: MonsGame): Input[] {
  const simulated = game.fork();
  const inputs: Input[] = [];
  const findFirstApplicableCompletion = (): boolean => {
    // Start categories depend on whether any mon/action completion is
    // applicable. Preserve that filtered decision, then use raw grammar
    // candidates below the root so the DFS can short-circuit sibling work.
    const output =
      inputs.length === 0
        ? simulated.processInputWithStartOptions(
            inputs,
            true,
            false,
            FOR_AUTOMOVE_START_INPUT_OPTIONS,
          )
        : simulated.inspectInputGrammar(
            inputs,
            FOR_AUTOMOVE_START_INPUT_OPTIONS,
          );
    if (output.kind === "invalid-input") return false;
    if (output.kind === "events") {
      return (
        inputs.length !== 0 &&
        applyInputsForSearchWithEvents(simulated, inputs) !== undefined
      );
    }
    if (inputs.length === MAX_INPUTS_PER_MOVE) return false;

    const choices = nextInputsForPrompt(output);
    if (choices === undefined || choices.length === 0) return false;
    choices.sort(compareInputs);
    for (const choice of choices) {
      inputs.push(choice);
      if (findFirstApplicableCompletion()) return true;
      inputs.pop();
    }
    return false;
  };

  return findFirstApplicableCompletion() ? inputs : [];
}
