import { ACTIONS_PER_TURN } from "../board/config.js";
import type { Location } from "../board/geometry.js";
import { Color, itemMon, manaScore, type Event } from "../model/domain.js";
import type { RulesState } from "./state.js";

type MutableRulesCounters = {
  whiteScore: number;
  blackScore: number;
  activeColor: Color;
  actionsUsedCount: number;
  manaMovesCount: number;
  monsMovesCount: number;
  whitePotionsCount: number;
  blackPotionsCount: number;
};

type NumericCounter = Exclude<keyof MutableRulesCounters, "activeColor">;

type CounterProjection =
  | { readonly valid: false }
  | {
      readonly valid: true;
      readonly potionUse?: {
        readonly from: Location;
        readonly to: Location;
      };
    };

export function copyRulesCounters(state: RulesState): MutableRulesCounters {
  return {
    whiteScore: state.whiteScore,
    blackScore: state.blackScore,
    activeColor: state.activeColor,
    actionsUsedCount: state.actionsUsedCount,
    manaMovesCount: state.manaMovesCount,
    monsMovesCount: state.monsMovesCount,
    whitePotionsCount: state.whitePotionsCount,
    blackPotionsCount: state.blackPotionsCount,
  };
}

function addCounter(
  state: MutableRulesCounters,
  counter: NumericCounter,
  amount: number,
  requireSafeInteger: boolean,
): boolean {
  const result = state[counter] + amount;
  if (requireSafeInteger && !Number.isSafeInteger(result)) return false;
  state[counter] = result;
  return true;
}

function consumeAction(
  state: MutableRulesCounters,
  from: Location,
  to: Location,
  requireSafeInteger: boolean,
): CounterProjection {
  if (state.actionsUsedCount < ACTIONS_PER_TURN) {
    state.actionsUsedCount += 1;
    return { valid: true };
  }

  const potionCounter =
    state.activeColor === Color.White ? "whitePotionsCount" : "blackPotionsCount";
  if (requireSafeInteger && state[potionCounter] <= 0) {
    return { valid: false };
  }
  state[potionCounter] -= 1;
  return { valid: true, potionUse: { from, to } };
}

export function projectEventCounters(
  state: MutableRulesCounters,
  event: Event,
  inputEvents: readonly Event[],
  requireSafeInteger: boolean,
): CounterProjection {
  switch (event.kind) {
    case "mon-move":
      return {
        valid: addCounter(state, "monsMovesCount", 1, requireSafeInteger),
      };
    case "mana-move":
      return {
        valid: addCounter(state, "manaMovesCount", 1, requireSafeInteger),
      };
    case "mana-scored": {
      const scoreCounter =
        state.activeColor === Color.White ? "whiteScore" : "blackScore";
      return {
        valid: addCounter(
          state,
          scoreCounter,
          manaScore(event.mana, state.activeColor),
          requireSafeInteger,
        ),
      };
    }
    case "mystic-action":
      return consumeAction(state, event.from, event.to, requireSafeInteger);
    case "demon-action": {
      const additionalDestination = inputEvents.find(
        (
          candidate,
        ): candidate is Extract<Event, { readonly kind: "demon-additional-step" }> =>
          candidate.kind === "demon-additional-step",
      )?.to;
      const potionLocation = additionalDestination ?? event.to;
      return consumeAction(state, potionLocation, potionLocation, requireSafeInteger);
    }
    case "spirit-target-move":
      return consumeAction(state, event.by, event.to, requireSafeInteger);
    case "pickup-potion": {
      const mon = itemMon(event.by);
      if (mon === undefined) return { valid: true };
      const potionCounter =
        mon.color === Color.White ? "whitePotionsCount" : "blackPotionsCount";
      return {
        valid: addCounter(state, potionCounter, 1, requireSafeInteger),
      };
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
      return { valid: true };
  }
}
