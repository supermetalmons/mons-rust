import {
  AvailableMoveKind,
  type AvailableMoveKind as EngineAvailableMoveKind,
  type Event as EngineEvent,
  type Input as EngineInput,
  type Item as EngineItem,
  type Mana as EngineMana,
  type Mon as EngineMon,
  type NextInput as EngineNextInput,
  type Output as EngineOutput,
  type Square as EngineSquare,
} from "../engine/domain.js";
import {
  BOARD_SIZE,
  isValidLocation,
  type Location as EnginePosition,
} from "../engine/geometry.js";
import type {
  AvailableMoveCounts,
  BoardItem,
  GameEvent,
  Input,
  InputOption,
  InputResolution,
  Mana,
  Mon,
  Position,
  Square,
} from "./types.js";

export function toEnginePosition(position: Position): EnginePosition {
  const enginePosition = { i: position.row, j: position.column };
  if (
    !Number.isInteger(position.row) ||
    !Number.isInteger(position.column) ||
    !isValidLocation(enginePosition)
  ) {
    throw new RangeError(
      `position must contain integer row and column values from 0 through ${BOARD_SIZE - 1}`,
    );
  }
  return enginePosition;
}

function fromEnginePosition(position: EnginePosition): Position {
  return { row: position.i, column: position.j };
}

function fromEngineMon(mon: EngineMon): Mon {
  return {
    kind: mon.kind,
    color: mon.color,
    cooldown: mon.cooldown,
  };
}

function fromEngineMana(mana: EngineMana): Mana {
  return mana.kind === "supermana"
    ? { kind: "supermana" }
    : { kind: "regular", color: mana.color };
}

export function fromEngineItem(item: EngineItem): BoardItem {
  switch (item.kind) {
    case "mon":
      return { kind: "mon", mon: fromEngineMon(item.mon) };
    case "mana":
      return { kind: "mana", mana: fromEngineMana(item.mana) };
    case "mon-with-mana":
      return {
        kind: "mon",
        mon: fromEngineMon(item.mon),
        carrying: { kind: "mana", mana: fromEngineMana(item.mana) },
      };
    case "mon-with-consumable":
      return {
        kind: "mon",
        mon: fromEngineMon(item.mon),
        carrying: {
          kind: "consumable",
          consumable: item.consumable,
        },
      };
    case "consumable":
      return {
        kind: "consumable",
        consumable: item.consumable,
      };
  }
}

export function fromEngineSquare(square: EngineSquare): Square {
  switch (square.kind) {
    case "regular":
    case "consumable-base":
    case "supermana-base":
      return { kind: square.kind };
    case "mana-base":
    case "mana-pool":
      return { kind: square.kind, color: square.color };
    case "mon-base":
      return {
        kind: square.kind,
        color: square.color,
        monKind: square.monKind,
      };
  }
}

export function toEngineInput(input: Input): EngineInput {
  switch (input.kind) {
    case "takeback":
      return { kind: "takeback" };
    case "position":
      return { kind: "location", location: toEnginePosition(input.position) };
    case "modifier":
      return { kind: "modifier", modifier: input.modifier };
  }
}

export function fromEngineInput(input: EngineInput): Input {
  switch (input.kind) {
    case "takeback":
      return { kind: "takeback" };
    case "location":
      return {
        kind: "position",
        position: fromEnginePosition(input.location),
      };
    case "modifier":
      return { kind: "modifier", modifier: input.modifier };
  }
}

function fromEngineInputOption(option: EngineNextInput): InputOption {
  const input = fromEngineInput(option.input);
  const actor =
    option.actorMonItem === undefined
      ? {}
      : { actor: fromEngineItem(option.actorMonItem) };

  if (option.kind === "select-consumable") {
    if (input.kind !== "modifier") {
      throw new Error("consumable selection requires a modifier input");
    }
    return { action: option.kind, input, ...actor };
  }
  if (input.kind !== "position") {
    throw new Error(`${option.kind} requires a position input`);
  }
  switch (option.kind) {
    case "mon-move":
    case "mana-move":
    case "mystic-action":
    case "demon-action":
    case "demon-additional-step":
    case "spirit-target-capture":
    case "spirit-target-move":
    case "bomb-attack":
      return { action: option.kind, input, ...actor };
  }
}

export function fromEngineEvent(event: EngineEvent): GameEvent {
  switch (event.kind) {
    case "mon-move":
      return {
        kind: event.kind,
        item: fromEngineItem(event.item),
        from: fromEnginePosition(event.from),
        to: fromEnginePosition(event.to),
      };
    case "mana-move":
      return {
        kind: event.kind,
        mana: fromEngineMana(event.mana),
        from: fromEnginePosition(event.from),
        to: fromEnginePosition(event.to),
      };
    case "mana-scored":
    case "mana-dropped":
      return {
        kind: event.kind,
        mana: fromEngineMana(event.mana),
        at: fromEnginePosition(event.at),
      };
    case "mystic-action":
      return {
        kind: event.kind,
        mystic: fromEngineMon(event.mystic),
        from: fromEnginePosition(event.from),
        to: fromEnginePosition(event.to),
      };
    case "demon-action":
    case "demon-additional-step":
      return {
        kind: event.kind,
        demon: fromEngineMon(event.demon),
        from: fromEnginePosition(event.from),
        to: fromEnginePosition(event.to),
      };
    case "spirit-target-move":
      return {
        kind: event.kind,
        item: fromEngineItem(event.item),
        from: fromEnginePosition(event.from),
        to: fromEnginePosition(event.to),
        by: fromEnginePosition(event.by),
      };
    case "pickup-bomb":
      return {
        kind: event.kind,
        by: fromEngineMon(event.by),
        at: fromEnginePosition(event.at),
      };
    case "pickup-potion":
      return {
        kind: event.kind,
        by: fromEngineItem(event.by),
        at: fromEnginePosition(event.at),
      };
    case "use-potion":
    case "supermana-back-to-base":
      return {
        kind: event.kind,
        from: fromEnginePosition(event.from),
        to: fromEnginePosition(event.to),
      };
    case "pickup-mana":
      return {
        kind: event.kind,
        mana: fromEngineMana(event.mana),
        by: fromEngineMon(event.by),
        at: fromEnginePosition(event.at),
      };
    case "mon-fainted":
      return {
        kind: event.kind,
        mon: fromEngineMon(event.mon),
        from: fromEnginePosition(event.from),
        to: fromEnginePosition(event.to),
      };
    case "bomb-attack":
      return {
        kind: event.kind,
        by: fromEngineMon(event.by),
        from: fromEnginePosition(event.from),
        to: fromEnginePosition(event.to),
      };
    case "mon-awake":
      return {
        kind: event.kind,
        mon: fromEngineMon(event.mon),
        at: fromEnginePosition(event.at),
      };
    case "bomb-explosion":
      return { kind: event.kind, at: fromEnginePosition(event.at) };
    case "next-turn":
      return { kind: event.kind, color: event.color };
    case "game-over":
      return { kind: event.kind, winner: event.winner };
    case "takeback":
      return { kind: event.kind };
  }
}

export function fromEngineOutput(
  output: EngineOutput,
  inputFen: string,
): InputResolution {
  switch (output.kind) {
    case "invalid-input":
      return { kind: "invalid", inputFen };
    case "locations-to-start-from":
      return {
        kind: "awaiting-start",
        inputFen,
        positions: output.locations.map(fromEnginePosition),
      };
    case "next-input-options":
      return {
        kind: "awaiting-input",
        inputFen,
        options: output.nextInputs.map(fromEngineInputOption),
      };
    case "events":
      return {
        kind: "complete",
        inputFen,
        events: output.events.map(fromEngineEvent),
      };
  }
}

export function availableMoveCountsFromEngine(
  counts: ReadonlyMap<EngineAvailableMoveKind, number>,
): AvailableMoveCounts {
  return {
    monMoves: counts.get(AvailableMoveKind.MonMove) ?? 0,
    manaMoves: counts.get(AvailableMoveKind.ManaMove) ?? 0,
    actions: counts.get(AvailableMoveKind.Action) ?? 0,
    potions: counts.get(AvailableMoveKind.Potion) ?? 0,
  };
}
