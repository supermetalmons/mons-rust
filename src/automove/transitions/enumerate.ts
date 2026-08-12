import {
  MAX_INPUTS_PER_MOVE,
  type Event,
  type Input,
} from "../../engine/model/domain.js";
import {
  FOR_AUTOMOVE_START_INPUT_OPTIONS,
  type SuggestedStartInputOptions,
} from "../../engine/game/input-support.js";
import { MonsGame } from "../../engine/game/mons-game.js";
import {
  BOARD_CELLS,
  locationIndex,
  type Location,
} from "../../engine/board/geometry.js";
import type { SearchControl } from "../core/deadline.js";
import type { AutomoveExecutionContext } from "../core/execution-context.js";
import {
  compareInputChains,
  compareLocations,
  compareNextInputs,
  lexicographicValues,
} from "./order.js";
import type { LegalInputTransition } from "./types.js";

type TransitionTraversalPolicy = {
  readonly sortBranches: boolean;
  readonly allowedFirstLocations: Uint8Array | undefined;
};

const ENGINE_ORDER_POLICY: TransitionTraversalPolicy = Object.freeze({
  sortBranches: false,
  allowedFirstLocations: undefined,
});

function appendAppliedTransition(
  game: MonsGame,
  inputs: readonly Input[],
  events: readonly Event[],
  transitions: LegalInputTransition[],
): void {
  const applied = game.forkAndApplyEventsForSimulation(events);
  if (applied === undefined) return;
  transitions.push({
    inputs: inputs.slice(),
    game: applied.game,
    events: applied.events,
  });
}

function collectTransitions(
  control: SearchControl,
  game: MonsGame,
  partialInputs: Input[],
  transitions: LegalInputTransition[],
  maxMoves: number,
  startOptions: SuggestedStartInputOptions,
  policy: TransitionTraversalPolicy,
): void {
  if (control.checkpoint()) {
    transitions.length = 0;
    return;
  }
  if (transitions.length >= maxMoves || partialInputs.length > MAX_INPUTS_PER_MOVE) {
    return;
  }

  const output = game.processInputWithStartOptions(
    partialInputs,
    true,
    false,
    startOptions,
  );
  if (output.kind === "invalid-input") return;
  if (output.kind === "events") {
    if (policy.allowedFirstLocations !== undefined && partialInputs.length === 0) {
      return;
    }
    appendAppliedTransition(game, partialInputs, output.events, transitions);
    return;
  }

  if (output.kind === "locations-to-start-from") {
    const locations = policy.sortBranches
      ? lexicographicValues(output.locations, compareLocations)
      : output.locations;
    for (const at of locations) {
      if (transitions.length >= maxMoves) break;
      if (
        partialInputs.length === 0 &&
        policy.allowedFirstLocations !== undefined &&
        policy.allowedFirstLocations[locationIndex(at)] !== 1
      ) {
        continue;
      }
      partialInputs.push({ kind: "location", location: at });
      collectTransitions(
        control,
        game,
        partialInputs,
        transitions,
        maxMoves,
        startOptions,
        policy,
      );
      partialInputs.pop();
    }
    return;
  }

  const nextInputs = policy.sortBranches
    ? lexicographicValues(output.nextInputs, compareNextInputs)
    : output.nextInputs;
  for (const nextInput of nextInputs) {
    if (transitions.length >= maxMoves) break;
    const input = nextInput.input;
    if (
      partialInputs.length === 0 &&
      policy.allowedFirstLocations !== undefined &&
      (input.kind !== "location" ||
        policy.allowedFirstLocations[locationIndex(input.location)] !== 1)
    ) {
      continue;
    }
    partialInputs.push(input);
    collectTransitions(
      control,
      game,
      partialInputs,
      transitions,
      maxMoves,
      startOptions,
      policy,
    );
    partialInputs.pop();
  }
}

function locationMask(locations: readonly Location[]): Uint8Array {
  const mask = new Uint8Array(BOARD_CELLS);
  for (const at of locations) mask[locationIndex(at)] = 1;
  return mask;
}

function startsAtMaskedLocation(
  transition: LegalInputTransition,
  mask: Uint8Array,
): boolean {
  const first = transition.inputs[0];
  return first?.kind === "location" && mask[locationIndex(first.location)] === 1;
}

export function enumerateLegalTransitionsWithPriority(
  context: AutomoveExecutionContext,
  game: MonsGame,
  maxMoves: number,
  startOptions: SuggestedStartInputOptions,
  priorityLocations: readonly Location[],
): LegalInputTransition[] {
  if (priorityLocations.length === 0) {
    return enumerateLegalTransitions(context, game, maxMoves, startOptions);
  }

  const priorityMask = locationMask(priorityLocations);
  const priorityBudget = Math.max(Math.floor(maxMoves / 2), Math.max(0, maxMoves - 60));
  const remainingBudget = Math.max(0, maxMoves - priorityBudget);
  const priority: LegalInputTransition[] = [];
  const others: LegalInputTransition[] = [];
  for (const transition of enumerateLegalTransitions(
    context,
    game,
    maxMoves,
    startOptions,
  )) {
    if (startsAtMaskedLocation(transition, priorityMask)) {
      if (priority.length < priorityBudget) priority.push(transition);
    } else if (others.length < remainingBudget) {
      others.push(transition);
    }
    if (
      priority.length >= priorityBudget &&
      (remainingBudget === 0 || others.length >= remainingBudget)
    ) {
      break;
    }
  }
  priority.push(...others);
  return priority;
}

export function enumerateLegalTransitions(
  context: AutomoveExecutionContext,
  game: MonsGame,
  maxMoves: number,
  startOptions: SuggestedStartInputOptions = FOR_AUTOMOVE_START_INPUT_OPTIONS,
): LegalInputTransition[] {
  if (context.session.checkpoint() || maxMoves <= 0) return [];
  const transitions: LegalInputTransition[] = [];
  collectTransitions(
    context.session,
    game.fork(),
    [],
    transitions,
    maxMoves,
    startOptions,
    ENGINE_ORDER_POLICY,
  );
  if (context.session.cancelled) return [];
  transitions.sort((left, right) => compareInputChains(left.inputs, right.inputs));
  return transitions;
}

export function enumerateLegalTransitionsLexicographicBounded(
  context: AutomoveExecutionContext,
  game: MonsGame,
  maxMoves: number,
  startOptions: SuggestedStartInputOptions = FOR_AUTOMOVE_START_INPUT_OPTIONS,
  allowedFirstLocations?: readonly Location[],
): LegalInputTransition[] {
  if (maxMoves <= 0 || context.session.checkpoint()) return [];
  const policy: TransitionTraversalPolicy = {
    sortBranches: true,
    allowedFirstLocations:
      allowedFirstLocations === undefined
        ? undefined
        : locationMask(allowedFirstLocations),
  };
  const transitions: LegalInputTransition[] = [];
  collectTransitions(
    context.session,
    game.fork(),
    [],
    transitions,
    maxMoves,
    startOptions,
    policy,
  );
  return context.session.checkpoint() ? [] : transitions;
}
