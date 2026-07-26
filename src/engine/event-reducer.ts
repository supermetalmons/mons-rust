import type { MutableBoard } from "./board.js";
import {
  Color,
  Consumable,
  SUPERMANA,
  decreaseMonCooldown,
  faintMon,
  isMonFainted,
  itemMon,
  manaItem,
  manaScore,
  monItem,
  monWithConsumableItem,
  monWithManaItem,
  otherColor,
  type Event,
} from "./domain.js";
import { ACTIONS_PER_TURN } from "./config.js";
import type { Location } from "./geometry.js";
import { shouldAdvanceTurn, winnerForState } from "./legality.js";

export type MutableRulesState = {
  board: MutableBoard;
  whiteScore: number;
  blackScore: number;
  activeColor: Color;
  actionsUsedCount: number;
  manaMovesCount: number;
  monsMovesCount: number;
  whitePotionsCount: number;
  blackPotionsCount: number;
  turnNumber: number;
};

export type EventReduction = {
  readonly events: Event[];
  readonly turnAdvanced: boolean;
  readonly winner: Color | undefined;
};

type CounterPreflight = {
  actionsUsedCount: number;
  manaMovesCount: number;
  monsMovesCount: number;
  whitePotionsCount: number;
  blackPotionsCount: number;
  whiteScore: number;
  blackScore: number;
};

function incrementSafe(value: number, amount = 1): number | undefined {
  const result = value + amount;
  return Number.isSafeInteger(result) ? result : undefined;
}

function consumeActionForPreflight(
  counters: CounterPreflight,
  activeColor: Color,
): boolean {
  if (counters.actionsUsedCount < ACTIONS_PER_TURN) {
    counters.actionsUsedCount += 1;
    return true;
  }
  if (activeColor === Color.White) {
    if (counters.whitePotionsCount <= 0) return false;
    counters.whitePotionsCount -= 1;
  } else {
    if (counters.blackPotionsCount <= 0) return false;
    counters.blackPotionsCount -= 1;
  }
  return true;
}

/**
 * Check every unbounded numeric mutation before touching board or history
 * state. A game at the largest safe turn may still finish by scoring, but it
 * cannot apply a move that would need another turn number.
 */
export function canApplyRulesEvents(
  state: MutableRulesState,
  events: readonly Event[],
): boolean {
  const counters: CounterPreflight = {
    actionsUsedCount: state.actionsUsedCount,
    manaMovesCount: state.manaMovesCount,
    monsMovesCount: state.monsMovesCount,
    whitePotionsCount: state.whitePotionsCount,
    blackPotionsCount: state.blackPotionsCount,
    whiteScore: state.whiteScore,
    blackScore: state.blackScore,
  };

  for (const event of events) {
    switch (event.kind) {
      case "mon-move": {
        const next = incrementSafe(counters.monsMovesCount);
        if (next === undefined) return false;
        counters.monsMovesCount = next;
        break;
      }
      case "mana-move": {
        const next = incrementSafe(counters.manaMovesCount);
        if (next === undefined) return false;
        counters.manaMovesCount = next;
        break;
      }
      case "mana-scored": {
        const score = manaScore(event.mana, state.activeColor);
        if (state.activeColor === Color.White) {
          const next = incrementSafe(counters.whiteScore, score);
          if (next === undefined) return false;
          counters.whiteScore = next;
        } else {
          const next = incrementSafe(counters.blackScore, score);
          if (next === undefined) return false;
          counters.blackScore = next;
        }
        break;
      }
      case "mystic-action":
      case "demon-action":
      case "spirit-target-move":
        if (!consumeActionForPreflight(counters, state.activeColor)) {
          return false;
        }
        break;
      case "pickup-potion": {
        const mon = itemMon(event.by);
        if (mon === undefined) break;
        if (mon.color === Color.White) {
          const next = incrementSafe(counters.whitePotionsCount);
          if (next === undefined) return false;
          counters.whitePotionsCount = next;
        } else {
          const next = incrementSafe(counters.blackPotionsCount);
          if (next === undefined) return false;
          counters.blackPotionsCount = next;
        }
        break;
      }
      case "demon-additional-step":
      case "pickup-bomb":
      case "use-potion":
      case "pickup-mana":
      case "mon-fainted":
      case "mana-dropped":
      case "supermana-back-to-base":
      case "bomb-attack":
      case "mon-awake":
      case "bomb-explosion":
      case "next-turn":
      case "game-over":
      case "takeback":
        break;
    }
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

function consumeAction(
  state: MutableRulesState,
  from: Location,
  to: Location,
  resultingEvents: Event[],
): void {
  if (state.actionsUsedCount < ACTIONS_PER_TURN) {
    state.actionsUsedCount += 1;
    return;
  }

  if (state.activeColor === Color.White) {
    state.whitePotionsCount -= 1;
  } else {
    state.blackPotionsCount -= 1;
  }
  resultingEvents.push({ kind: "use-potion", from, to });
}

function applyEvent(
  state: MutableRulesState,
  event: Event,
  inputEvents: readonly Event[],
  resultingEvents: Event[],
): void {
  switch (event.kind) {
    case "mon-move":
      state.monsMovesCount += 1;
      state.board.delete(event.from);
      state.board.set(event.to, event.item);
      break;
    case "mana-move":
      state.manaMovesCount += 1;
      state.board.delete(event.from);
      state.board.set(event.to, manaItem(event.mana));
      break;
    case "mana-scored": {
      const score = manaScore(event.mana, state.activeColor);
      if (state.activeColor === Color.White) {
        state.whiteScore += score;
      } else {
        state.blackScore += score;
      }
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
      consumeAction(state, event.from, event.to, resultingEvents);
      state.board.delete(event.to);
      break;
    case "demon-action": {
      state.board.delete(event.from);
      const additionalDestination = inputEvents.find(
        (
          candidate,
        ): candidate is Extract<
          Event,
          { readonly kind: "demon-additional-step" }
        > => candidate.kind === "demon-additional-step",
      )?.to;
      if (additionalDestination === undefined) {
        state.board.set(event.to, monItem(event.demon));
      } else {
        state.board.delete(event.to);
      }
      const potionLocation = additionalDestination ?? event.to;
      consumeAction(state, potionLocation, potionLocation, resultingEvents);
      break;
    }
    case "demon-additional-step":
      state.board.set(event.to, monItem(event.demon));
      break;
    case "spirit-target-move":
      consumeAction(state, event.by, event.to, resultingEvents);
      state.board.delete(event.from);
      state.board.set(event.to, event.item);
      break;
    case "pickup-bomb":
      state.board.set(
        event.at,
        monWithConsumableItem(event.by, Consumable.Bomb),
      );
      break;
    case "pickup-potion": {
      const mon = itemMon(event.by);
      if (mon === undefined) break;
      if (mon.color === Color.White) {
        state.whitePotionsCount += 1;
      } else {
        state.blackPotionsCount += 1;
      }
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

  for (const monLocation of state.board.faintedMonsLocations(
    state.activeColor,
  )) {
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
