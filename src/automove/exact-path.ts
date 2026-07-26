import { Board } from "../engine/board.js";
import {
  Color,
  Consumable,
  type Mana,
  MonKind,
  type Square,
} from "../engine/domain.js";
import {
  BOARD_CELLS,
  type Location,
  locationIndex,
  nearbyLocations,
} from "../engine/geometry.js";
import { type AutomoveExecutionContext } from "./execution-context.js";

export type ExactActorPayload =
  | { readonly kind: "none" }
  | { readonly kind: "mana"; readonly mana: Mana }
  | { readonly kind: "bomb" };

export type AttackQueueEntry = {
  readonly location: Location;
  readonly payload: ExactActorPayload;
  readonly steps: number;
};

export const NO_PAYLOAD: ExactActorPayload = Object.freeze({ kind: "none" });
export const BOMB_PAYLOAD: ExactActorPayload = Object.freeze({ kind: "bomb" });

function payloadSlot(payload: ExactActorPayload): number {
  switch (payload.kind) {
    case "none":
      return 0;
    case "bomb":
      return 1;
    case "mana":
      return payload.mana.kind === "supermana"
        ? 2
        : payload.mana.color === Color.White
          ? 3
          : 4;
  }
}

export function payloadSeenSlot(
  location: Location,
  payload: ExactActorPayload,
): number {
  return locationIndex(location) * 5 + payloadSlot(payload);
}

function squareAllowsEmptyMon(
  square: Square,
  monKind: MonKind,
  color: Color,
): boolean {
  switch (square.kind) {
    case "regular":
    case "consumable-base":
    case "mana-base":
    case "mana-pool":
      return true;
    case "supermana-base":
      return monKind === MonKind.Drainer;
    case "mon-base":
      return square.monKind === monKind && square.color === color;
  }
}

function squareAllowsManaCarrier(square: Square, mana: Mana): boolean {
  switch (square.kind) {
    case "regular":
    case "consumable-base":
    case "mana-base":
    case "mana-pool":
      return true;
    case "supermana-base":
      return mana.kind === "supermana";
    case "mon-base":
      return false;
  }
}

export function actorPayloadAfterMoveCompute(
  board: Board,
  monKind: MonKind,
  color: Color,
  payload: ExactActorPayload,
  destination: Location,
  allowPickBomb: boolean,
): ExactActorPayload | undefined {
  const item = board.get(destination);
  switch (payload.kind) {
    case "none": {
      if (item === undefined) {
        return squareAllowsEmptyMon(board.squareAt(destination), monKind, color)
          ? NO_PAYLOAD
          : undefined;
      }
      switch (item.kind) {
        case "mon":
        case "mon-with-mana":
        case "mon-with-consumable":
          return undefined;
        case "mana":
          return monKind === MonKind.Drainer
            ? { kind: "mana", mana: item.mana }
            : undefined;
        case "consumable":
          return item.consumable === Consumable.BombOrPotion
            ? allowPickBomb
              ? BOMB_PAYLOAD
              : NO_PAYLOAD
            : undefined;
      }
    }
    // eslint-disable-next-line no-fallthrough -- the nested exhaustive switch always returns.
    case "mana": {
      if (item === undefined) {
        return squareAllowsManaCarrier(
          board.squareAt(destination),
          payload.mana,
        )
          ? payload
          : undefined;
      }
      switch (item.kind) {
        case "mon":
        case "mon-with-mana":
        case "mon-with-consumable":
          return undefined;
        case "mana":
          return { kind: "mana", mana: item.mana };
        case "consumable":
          return item.consumable === Consumable.BombOrPotion
            ? payload
            : undefined;
      }
    }
    // eslint-disable-next-line no-fallthrough -- the nested exhaustive switch always returns.
    case "bomb": {
      if (item === undefined) {
        switch (board.squareAt(destination).kind) {
          case "regular":
          case "consumable-base":
          case "mana-base":
          case "mana-pool":
            return BOMB_PAYLOAD;
          case "supermana-base":
          case "mon-base":
            return undefined;
        }
      }
      if (
        item.kind === "consumable" &&
        item.consumable === Consumable.BombOrPotion
      ) {
        return BOMB_PAYLOAD;
      }
      return undefined;
    }
  }
}

export function shortestPayloadState(
  context: AutomoveExecutionContext,
  board: Board,
  start: Location,
  monKind: MonKind,
  color: Color,
  startPayload: ExactActorPayload,
  allowPickBomb: boolean,
  maxSteps: number | undefined,
  goal: (location: Location, payload: ExactActorPayload) => boolean,
): number | undefined {
  if (context.session.checkpoint()) return undefined;
  const queue: AttackQueueEntry[] = [
    { location: start, payload: startPayload, steps: 0 },
  ];
  const seen = new Uint8Array(BOARD_CELLS * 5);
  seen[payloadSeenSlot(start, startPayload)] = 1;
  let cursor = 0;
  while (cursor < queue.length) {
    const current = queue[cursor];
    cursor += 1;
    if (current === undefined || context.session.checkpoint()) return undefined;
    if (goal(current.location, current.payload)) return current.steps;
    if (maxSteps !== undefined && current.steps >= maxSteps) continue;
    for (const next of nearbyLocations(current.location)) {
      const nextPayload = actorPayloadAfterMoveCompute(
        board,
        monKind,
        color,
        current.payload,
        next,
        allowPickBomb,
      );
      if (nextPayload === undefined) continue;
      const slot = payloadSeenSlot(next, nextPayload);
      if (seen[slot] !== 0) continue;
      seen[slot] = 1;
      queue.push({
        location: next,
        payload: nextPayload,
        steps: current.steps + 1,
      });
    }
  }
  return undefined;
}
