import {
  FOR_AUTOMOVE_START_INPUT_OPTIONS,
  MonsGame,
  type SuggestedStartInputOptions,
} from "../engine/game.js";
import {
  MAX_INPUTS_PER_MOVE,
  MODIFIER_RANK,
  type Event,
  type Input,
  type NextInput,
} from "../engine/domain.js";
import {
  BOARD_CELLS,
  locationIndex,
  type Location,
} from "../engine/geometry.js";
import type { SearchControl } from "./deadline.js";
import type { AutomoveExecutionContext } from "./execution-context.js";

export type LegalInputTransition = {
  readonly inputs: readonly Input[];
  readonly game: MonsGame;
  readonly events: readonly Event[];
};

const MATERIAL_EVENT_KINDS: readonly Event["kind"][] = Object.freeze([
  "mana-scored",
  "pickup-mana",
  "mon-fainted",
  "use-potion",
  "pickup-bomb",
  "pickup-potion",
  "bomb-attack",
  "bomb-explosion",
]);

const QUIESCENCE_TACTICAL_EVENT_KINDS: readonly Event["kind"][] = Object.freeze(
  [
    "mana-scored",
    "pickup-mana",
    "mon-fainted",
    "use-potion",
    "bomb-attack",
    "bomb-explosion",
    "spirit-target-move",
    "supermana-back-to-base",
  ],
);

function compareNumber(left: number, right: number): number {
  return left < right ? -1 : left > right ? 1 : 0;
}

function inputTag(value: Input): number {
  switch (value.kind) {
    case "takeback":
      return 0;
    case "location":
      return 1;
    case "modifier":
      return 2;
  }
}

function compareLocations(left: Location, right: Location): number {
  return compareNumber(left.i, right.i) || compareNumber(left.j, right.j);
}

function compareNextInputs(left: NextInput, right: NextInput): number {
  return compareInputs(left.input, right.input);
}

/** Stable input order: Takeback, Location(i,j), then Modifier. */
export function compareInputs(left: Input, right: Input): number {
  const tagOrder = compareNumber(inputTag(left), inputTag(right));
  if (tagOrder !== 0) return tagOrder;
  if (left.kind === "location" && right.kind === "location") {
    return compareLocations(left.location, right.location);
  }
  if (left.kind === "modifier" && right.kind === "modifier") {
    return compareNumber(
      MODIFIER_RANK[left.modifier],
      MODIFIER_RANK[right.modifier],
    );
  }
  return 0;
}

export function compareInputChains(
  left: readonly Input[],
  right: readonly Input[],
): number {
  const sharedLength = Math.min(left.length, right.length);
  for (let index = 0; index < sharedLength; index += 1) {
    const leftInput = left[index];
    const rightInput = right[index];
    if (leftInput === undefined || rightInput === undefined) break;
    const order = compareInputs(leftInput, rightInput);
    if (order !== 0) return order;
  }
  return compareNumber(left.length, right.length);
}

function lexicographicValues<T>(
  values: readonly T[],
  compare: (left: T, right: T) => number,
): readonly T[] {
  for (let index = 1; index < values.length; index += 1) {
    const previous = values[index - 1];
    const current = values[index];
    if (
      previous !== undefined &&
      current !== undefined &&
      compare(previous, current) > 0
    ) {
      return [...values].sort(compare);
    }
  }
  return values;
}

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

function collectGeneratedTransitions(
  control: SearchControl,
  game: MonsGame,
  partialInputs: Input[],
  transitions: LegalInputTransition[],
  maxMoves: number,
  startOptions: SuggestedStartInputOptions,
): void {
  if (control.checkpoint()) {
    transitions.length = 0;
    return;
  }
  if (
    transitions.length >= maxMoves ||
    partialInputs.length > MAX_INPUTS_PER_MOVE
  )
    return;

  const output = game.processInputWithStartOptions(
    partialInputs,
    true,
    false,
    startOptions,
  );
  if (output.kind === "invalid-input") return;
  if (output.kind === "events") {
    appendAppliedTransition(game, partialInputs, output.events, transitions);
    return;
  }

  if (output.kind === "locations-to-start-from") {
    for (const at of output.locations) {
      if (transitions.length >= maxMoves) break;
      partialInputs.push({ kind: "location", location: at });
      collectGeneratedTransitions(
        control,
        game,
        partialInputs,
        transitions,
        maxMoves,
        startOptions,
      );
      partialInputs.pop();
    }
    return;
  }

  for (const nextInput of output.nextInputs) {
    if (transitions.length >= maxMoves) break;
    partialInputs.push(nextInput.input);
    collectGeneratedTransitions(
      control,
      game,
      partialInputs,
      transitions,
      maxMoves,
      startOptions,
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
  return (
    first?.kind === "location" && mask[locationIndex(first.location)] === 1
  );
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
  const priorityBudget = Math.max(
    Math.floor(maxMoves / 2),
    Math.max(0, maxMoves - 60),
  );
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

function collectLexicographicTransitions(
  control: SearchControl,
  game: MonsGame,
  partialInputs: Input[],
  transitions: LegalInputTransition[],
  maxMoves: number,
  startOptions: SuggestedStartInputOptions,
  allowedFirstLocations: Uint8Array | undefined,
): void {
  if (control.checkpoint()) {
    transitions.length = 0;
    return;
  }
  if (
    transitions.length >= maxMoves ||
    partialInputs.length > MAX_INPUTS_PER_MOVE
  )
    return;

  const output = game.processInputWithStartOptions(
    partialInputs,
    true,
    false,
    startOptions,
  );
  if (output.kind === "invalid-input") return;
  if (output.kind === "events") {
    if (allowedFirstLocations !== undefined && partialInputs.length === 0)
      return;
    appendAppliedTransition(game, partialInputs, output.events, transitions);
    return;
  }

  if (output.kind === "locations-to-start-from") {
    const locations = lexicographicValues(output.locations, compareLocations);
    for (const at of locations) {
      if (transitions.length >= maxMoves) break;
      if (
        partialInputs.length === 0 &&
        allowedFirstLocations !== undefined &&
        allowedFirstLocations[locationIndex(at)] !== 1
      ) {
        continue;
      }
      partialInputs.push({ kind: "location", location: at });
      collectLexicographicTransitions(
        control,
        game,
        partialInputs,
        transitions,
        maxMoves,
        startOptions,
        allowedFirstLocations,
      );
      partialInputs.pop();
    }
    return;
  }

  const nextInputs = lexicographicValues(output.nextInputs, compareNextInputs);
  for (const nextInput of nextInputs) {
    if (transitions.length >= maxMoves) break;
    const input = nextInput.input;
    if (
      partialInputs.length === 0 &&
      allowedFirstLocations !== undefined &&
      (input.kind !== "location" ||
        allowedFirstLocations[locationIndex(input.location)] !== 1)
    ) {
      continue;
    }
    partialInputs.push(input);
    collectLexicographicTransitions(
      control,
      game,
      partialInputs,
      transitions,
      maxMoves,
      startOptions,
      allowedFirstLocations,
    );
    partialInputs.pop();
  }
}

export function enumerateLegalTransitions(
  context: AutomoveExecutionContext,
  game: MonsGame,
  maxMoves: number,
  startOptions: SuggestedStartInputOptions = FOR_AUTOMOVE_START_INPUT_OPTIONS,
): LegalInputTransition[] {
  if (context.session.checkpoint() || maxMoves <= 0) return [];
  const transitions: LegalInputTransition[] = [];
  collectGeneratedTransitions(
    context.session,
    game.fork(),
    [],
    transitions,
    maxMoves,
    startOptions,
  );
  if (context.session.cancelled) return [];
  transitions.sort((left, right) =>
    compareInputChains(left.inputs, right.inputs),
  );
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
  const mask =
    allowedFirstLocations === undefined
      ? undefined
      : locationMask(allowedFirstLocations);
  const transitions: LegalInputTransition[] = [];
  collectLexicographicTransitions(
    context.session,
    game.fork(),
    [],
    transitions,
    maxMoves,
    startOptions,
    mask,
  );
  return context.session.checkpoint() ? [] : transitions;
}

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

function hasEventKind(
  events: readonly Event[],
  kinds: readonly Event["kind"][],
): boolean {
  return events.some((event) => kinds.includes(event.kind));
}

export function hasMaterialEvent(events: readonly Event[]): boolean {
  return hasEventKind(events, MATERIAL_EVENT_KINDS);
}

export function isQuiescenceTacticalTransition(
  events: readonly Event[],
): boolean {
  return hasEventKind(events, QUIESCENCE_TACTICAL_EVENT_KINDS);
}
