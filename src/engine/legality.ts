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

export function winnerForScores(
  whiteScore: number,
  blackScore: number,
): ColorValue | undefined {
  if (whiteScore >= TARGET_SCORE) return Color.White;
  return blackScore >= TARGET_SCORE ? Color.Black : undefined;
}

export function winnerForState(state: RulesStateView): ColorValue | undefined {
  return winnerForScores(state.whiteScore, state.blackScore);
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

export function canMoveMonForCounts(monsMovesCount: number): boolean {
  return monsMovesCount < MONS_MOVES_PER_TURN;
}

export function canPlayerMoveMon(state: RulesStateView): boolean {
  return canMoveMonForCounts(state.monsMovesCount);
}

export function canMoveManaForCounts(
  firstTurn: boolean,
  manaMovesCount: number,
): boolean {
  return !firstTurn && manaMovesCount < MANA_MOVES_PER_TURN;
}

export function canPlayerMoveMana(state: RulesStateView): boolean {
  return canMoveManaForCounts(isFirstTurnState(state), state.manaMovesCount);
}

export function canUseActionForCounts(
  firstTurn: boolean,
  actionsUsedCount: number,
  currentPotionsCount: number,
): boolean {
  return (
    !firstTurn &&
    (currentPotionsCount > 0 || actionsUsedCount < ACTIONS_PER_TURN)
  );
}

export function canPlayerUseAction(state: RulesStateView): boolean {
  return canUseActionForCounts(
    isFirstTurnState(state),
    state.actionsUsedCount,
    currentPlayerPotions(state),
  );
}

export function shouldSuggestRegularManaStartsFromScalars(
  firstTurn: boolean,
  monsMovesCount: number,
  manaMovesCount: number,
  actionsUsedCount: number,
  currentPotionsCount: number,
  hasSuggestedMonStart: boolean,
  includeManaStartsWithPotionAction: boolean,
): boolean {
  if (!canMoveManaForCounts(firstTurn, manaMovesCount)) return false;
  const canMoveMon = canMoveMonForCounts(monsMovesCount);
  return (
    (!canMoveMon &&
      !canUseActionForCounts(
        firstTurn,
        actionsUsedCount,
        currentPotionsCount,
      )) ||
    !hasSuggestedMonStart ||
    (includeManaStartsWithPotionAction &&
      !canMoveMon &&
      actionsUsedCount >= ACTIONS_PER_TURN &&
      currentPotionsCount > 0)
  );
}

export function shouldSuggestRegularManaStarts(
  state: RulesStateView,
  hasSuggestedMonStart: boolean,
  includeManaStartsWithPotionAction: boolean,
): boolean {
  return shouldSuggestRegularManaStartsFromScalars(
    isFirstTurnState(state),
    state.monsMovesCount,
    state.manaMovesCount,
    state.actionsUsedCount,
    currentPlayerPotions(state),
    hasSuggestedMonStart,
    includeManaStartsWithPotionAction,
  );
}

export function shouldAdvanceTurnForCounts(
  firstTurn: boolean,
  monsMovesCount: number,
  manaMovesCount: number,
  hasFreeRegularMana: boolean,
): boolean {
  if (firstTurn) return !canMoveMonForCounts(monsMovesCount);
  if (!canMoveManaForCounts(firstTurn, manaMovesCount)) return true;
  return !canMoveMonForCounts(monsMovesCount) && !hasFreeRegularMana;
}

export function shouldAdvanceTurn(state: RulesStateView): boolean {
  const firstTurn = isFirstTurnState(state);
  const needsFreeManaCheck =
    !firstTurn &&
    canMoveManaForCounts(firstTurn, state.manaMovesCount) &&
    !canMoveMonForCounts(state.monsMovesCount);
  const hasFreeRegularMana =
    !needsFreeManaCheck ||
    state.board.findMana(state.activeColor) !== undefined;
  return shouldAdvanceTurnForCounts(
    firstTurn,
    state.monsMovesCount,
    state.manaMovesCount,
    hasFreeRegularMana,
  );
}
