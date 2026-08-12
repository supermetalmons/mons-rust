import {
  Color,
  Consumable,
  SUPERMANA,
  decreaseMonCooldown,
  faintMon,
  isMonFainted,
  itemMon,
  manaItem,
  monItem,
  monWithConsumableItem,
  monWithManaItem,
  otherColor,
  type Event,
} from "../model/domain.js";
import { copyRulesCounters, projectEventCounters } from "./event-counters.js";
import { shouldAdvanceTurn, winnerForState } from "./legality.js";
import type { MutableRulesState } from "./state.js";

type EventReduction = {
  readonly events: Event[];
  readonly turnAdvanced: boolean;
  readonly winner: Color | undefined;
};

/**
 * Check every unbounded numeric mutation before touching board or history
 * state. A game at the largest safe turn may still finish by scoring, but it
 * cannot apply a move that would need another turn number.
 */
export function canApplyRulesEvents(
  state: MutableRulesState,
  events: readonly Event[],
): boolean {
  const counters = copyRulesCounters(state);
  for (const event of events) {
    if (!projectEventCounters(counters, event, events, true).valid) return false;
  }

  if (state.turnNumber < Number.MAX_SAFE_INTEGER) return true;

  // Only this numeric boundary needs a dry run. Use a safe probe turn with an
  // independent board; turn-advance rules are otherwise identical for every
  // non-first turn.
  const probe: MutableRulesState = {
    ...state,
    board: state.board.fork(),
    turnNumber: Number.MAX_SAFE_INTEGER - 1,
  };
  return !applyRulesEvents(probe, events).turnAdvanced;
}

function applyEvent(
  state: MutableRulesState,
  event: Event,
  inputEvents: readonly Event[],
  resultingEvents: Event[],
): void {
  const projection = projectEventCounters(state, event, inputEvents, false);
  if (projection.valid && projection.potionUse !== undefined) {
    resultingEvents.push({ kind: "use-potion", ...projection.potionUse });
  }
  switch (event.kind) {
    case "mon-move":
      state.board.delete(event.from);
      state.board.set(event.to, event.item);
      break;
    case "mana-move":
      state.board.delete(event.from);
      state.board.set(event.to, manaItem(event.mana));
      break;
    case "mana-scored": {
      const item = state.board.get(event.at);
      if (item !== undefined) {
        const mon = itemMon(item);
        if (mon === undefined) {
          state.board.delete(event.at);
        } else {
          state.board.set(event.at, monItem(mon));
        }
      }
      break;
    }
    case "mystic-action":
      state.board.delete(event.to);
      break;
    case "demon-action": {
      state.board.delete(event.from);
      const additionalDestination = inputEvents.find(
        (
          candidate,
        ): candidate is Extract<Event, { readonly kind: "demon-additional-step" }> =>
          candidate.kind === "demon-additional-step",
      )?.to;
      if (additionalDestination === undefined) {
        state.board.set(event.to, monItem(event.demon));
      } else {
        state.board.delete(event.to);
      }
      break;
    }
    case "demon-additional-step":
      state.board.set(event.to, monItem(event.demon));
      break;
    case "spirit-target-move":
      state.board.delete(event.from);
      state.board.set(event.to, event.item);
      break;
    case "pickup-bomb":
      state.board.set(event.at, monWithConsumableItem(event.by, Consumable.Bomb));
      break;
    case "pickup-potion": {
      const mon = itemMon(event.by);
      if (mon === undefined) break;
      state.board.set(event.at, event.by);
      break;
    }
    case "pickup-mana":
      state.board.set(event.at, monWithManaItem(event.by, event.mana));
      break;
    case "mon-fainted":
      state.board.set(event.to, monItem(faintMon(event.mon)));
      break;
    case "mana-dropped":
      state.board.set(event.at, manaItem(event.mana));
      break;
    case "supermana-back-to-base": {
      const item = state.board.get(event.to);
      state.board.set(
        event.to,
        item?.kind === "mon"
          ? monWithManaItem(item.mon, SUPERMANA)
          : manaItem(SUPERMANA),
      );
      break;
    }
    case "bomb-attack":
      state.board.delete(event.to);
      state.board.set(event.from, monItem(event.by));
      break;
    case "bomb-explosion":
      state.board.delete(event.at);
      break;
    case "mon-awake":
    case "game-over":
    case "next-turn":
    case "takeback":
    case "use-potion":
      break;
  }
}

function advanceTurn(state: MutableRulesState, resultingEvents: Event[]): void {
  state.activeColor = otherColor(state.activeColor);
  state.turnNumber += 1;
  state.actionsUsedCount = 0;
  state.manaMovesCount = 0;
  state.monsMovesCount = 0;
  resultingEvents.push({ kind: "next-turn", color: state.activeColor });

  for (const monLocation of state.board.faintedMonsLocations(state.activeColor)) {
    const item = state.board.get(monLocation);
    const mon = item === undefined ? undefined : itemMon(item);
    if (mon === undefined) continue;
    const updatedMon = decreaseMonCooldown(mon);
    if (!isMonFainted(updatedMon)) {
      resultingEvents.push({
        kind: "mon-awake",
        mon: updatedMon,
        at: monLocation,
      });
    }
    state.board.set(monLocation, monItem(updatedMon));
  }
}

/**
 * Apply a complete, already-validated event sequence and perform any resulting
 * turn transition. Input validation remains outside this reducer.
 */
export function applyRulesEvents(
  state: MutableRulesState,
  events: readonly Event[],
): EventReduction {
  const resultingEvents: Event[] = [];
  for (const event of events) {
    applyEvent(state, event, events, resultingEvents);
  }

  const winner = winnerForState(state);
  let turnAdvanced = false;
  if (winner !== undefined) {
    resultingEvents.push({ kind: "game-over", winner });
  } else if (shouldAdvanceTurn(state)) {
    advanceTurn(state, resultingEvents);
    turnAdvanced = true;
  }

  return {
    events: [...events, ...resultingEvents],
    turnAdvanced,
    winner,
  };
}
