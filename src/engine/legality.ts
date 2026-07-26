import type { Board } from "./board.js";
import {
  ACTIONS_PER_TURN,
  MANA_MOVES_PER_TURN,
  MONS_MOVES_PER_TURN,
  TARGET_SCORE,
} from "./config.js";
import { Color, type Color as ColorValue } from "./domain.js";

export type RulesStateView = {
  readonly board: Board;
  readonly whiteScore: number;
  readonly blackScore: number;
  readonly activeColor: ColorValue;
  readonly actionsUsedCount: number;
  readonly manaMovesCount: number;
  readonly monsMovesCount: number;
  readonly whitePotionsCount: number;
  readonly blackPotionsCount: number;
  readonly turnNumber: number;
};

export function winnerForState(state: RulesStateView): ColorValue | undefined {
  if (state.whiteScore >= TARGET_SCORE) return Color.White;
  return state.blackScore >= TARGET_SCORE ? Color.Black : undefined;
}

export function scoreForColor(
  state: Pick<RulesStateView, "whiteScore" | "blackScore">,
  color: ColorValue,
): number {
  return color === Color.White ? state.whiteScore : state.blackScore;
}

export function isFirstTurnState(state: RulesStateView): boolean {
  return state.turnNumber === 1;
}

export function currentPlayerPotions(state: RulesStateView): number {
  return state.activeColor === Color.White
    ? state.whitePotionsCount
    : state.blackPotionsCount;
}

export function canPlayerMoveMon(state: RulesStateView): boolean {
  return state.monsMovesCount < MONS_MOVES_PER_TURN;
}

export function canPlayerMoveMana(state: RulesStateView): boolean {
  return !isFirstTurnState(state) && state.manaMovesCount < MANA_MOVES_PER_TURN;
}

export function canPlayerUseAction(state: RulesStateView): boolean {
  return (
    !isFirstTurnState(state) &&
    (currentPlayerPotions(state) > 0 ||
      state.actionsUsedCount < ACTIONS_PER_TURN)
  );
}

export function shouldAdvanceTurn(state: RulesStateView): boolean {
  return (
    (isFirstTurnState(state) && !canPlayerMoveMon(state)) ||
    (!isFirstTurnState(state) && !canPlayerMoveMana(state)) ||
    (!isFirstTurnState(state) &&
      !canPlayerMoveMon(state) &&
      state.board.findMana(state.activeColor) === undefined)
  );
}
