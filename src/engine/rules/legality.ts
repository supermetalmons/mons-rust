import {
  ACTIONS_PER_TURN,
  MANA_MOVES_PER_TURN,
  MONS_MOVES_PER_TURN,
  TARGET_SCORE,
} from "../board/config.js";
import {
  Color,
  Consumable,
  MonKind,
  isMonFainted,
  itemMon,
  type Color as ColorValue,
  type Item,
  type Square,
} from "../model/domain.js";
import { nearbyLocations, type Location } from "../board/geometry.js";
import type { Board } from "../board/storage.js";
import type { RulesState } from "./state.js";

export function winnerForScores(
  whiteScore: number,
  blackScore: number,
): ColorValue | undefined {
  if (whiteScore >= TARGET_SCORE) return Color.White;
  return blackScore >= TARGET_SCORE ? Color.Black : undefined;
}

export function winnerForState(state: RulesState): ColorValue | undefined {
  return winnerForScores(state.whiteScore, state.blackScore);
}

export function scoreForColor(
  state: Pick<RulesState, "whiteScore" | "blackScore">,
  color: ColorValue,
): number {
  return color === Color.White ? state.whiteScore : state.blackScore;
}

export function isFirstTurnState(state: RulesState): boolean {
  return state.turnNumber === 1;
}

export function currentPlayerPotions(state: RulesState): number {
  return state.activeColor === Color.White
    ? state.whitePotionsCount
    : state.blackPotionsCount;
}

export function canMoveMonForCounts(monsMovesCount: number): boolean {
  return monsMovesCount < MONS_MOVES_PER_TURN;
}

export function canPlayerMoveMon(state: RulesState): boolean {
  return canMoveMonForCounts(state.monsMovesCount);
}

export function canMoveManaForCounts(
  firstTurn: boolean,
  manaMovesCount: number,
): boolean {
  return !firstTurn && manaMovesCount < MANA_MOVES_PER_TURN;
}

export function canPlayerMoveMana(state: RulesState): boolean {
  return canMoveManaForCounts(isFirstTurnState(state), state.manaMovesCount);
}

export function canUseActionForCounts(
  firstTurn: boolean,
  actionsUsedCount: number,
  currentPotionsCount: number,
): boolean {
  return !firstTurn && (currentPotionsCount > 0 || actionsUsedCount < ACTIONS_PER_TURN);
}

export function canPlayerUseAction(state: RulesState): boolean {
  return canUseActionForCounts(
    isFirstTurnState(state),
    state.actionsUsedCount,
    currentPlayerPotions(state),
  );
}

export function spiritDestinationItemAllowed(
  targetItem: Item,
  destinationItem: Item | undefined,
): boolean {
  if (destinationItem === undefined) return true;
  const targetMon = itemMon(targetItem);
  switch (destinationItem.kind) {
    case "mon":
      if (targetMon !== undefined) return false;
      if (targetItem.kind === "mana") {
        return (
          destinationItem.mon.kind === MonKind.Drainer &&
          !isMonFainted(destinationItem.mon)
        );
      }
      return (
        targetItem.kind === "consumable" &&
        targetItem.consumable === Consumable.BombOrPotion
      );
    case "mana":
      return targetMon?.kind === MonKind.Drainer && !isMonFainted(targetMon);
    case "mon-with-mana":
    case "mon-with-consumable":
      return (
        targetItem.kind === "consumable" &&
        targetItem.consumable === Consumable.BombOrPotion
      );
    case "consumable":
      return (
        destinationItem.consumable === Consumable.BombOrPotion &&
        targetMon !== undefined
      );
  }
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
      !canUseActionForCounts(firstTurn, actionsUsedCount, currentPotionsCount)) ||
    !hasSuggestedMonStart ||
    (includeManaStartsWithPotionAction &&
      !canMoveMon &&
      actionsUsedCount >= ACTIONS_PER_TURN &&
      currentPotionsCount > 0)
  );
}

export function shouldSuggestRegularManaStarts(
  state: RulesState,
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
  hasMovableFreeRegularMana: boolean,
): boolean {
  if (firstTurn) return !canMoveMonForCounts(monsMovesCount);
  if (!canMoveManaForCounts(firstTurn, manaMovesCount)) return true;
  return !canMoveMonForCounts(monsMovesCount) && !hasMovableFreeRegularMana;
}

export function regularSquareForMovement(square: Square): boolean {
  switch (square.kind) {
    case "regular":
    case "consumable-base":
    case "mana-base":
    case "mana-pool":
      return true;
    case "supermana-base":
    case "mon-base":
      return false;
  }
}

export function regularManaMoveDestinationAllowed(board: Board, at: Location): boolean {
  const item = board.get(at);
  const square = board.squareAt(at);
  if (item?.kind === "mon") {
    return regularSquareForMovement(square) && item.mon.kind === MonKind.Drainer;
  }
  if (item !== undefined) return false;
  return regularSquareForMovement(square);
}

// A free mana with no legal destination cannot satisfy the mandatory mana move, so
// it must not hold the turn open; otherwise a player whose mon moves and action are
// spent would be left without any legal input while the game is not over.
function hasMovableFreeRegularMana(state: RulesState): boolean {
  for (const start of state.board.allFreeRegularManaLocations(state.activeColor)) {
    for (const destination of nearbyLocations(start)) {
      if (regularManaMoveDestinationAllowed(state.board, destination)) return true;
    }
  }
  return false;
}

export function shouldAdvanceTurn(state: RulesState): boolean {
  const firstTurn = isFirstTurnState(state);
  const needsFreeManaCheck =
    !firstTurn &&
    canMoveManaForCounts(firstTurn, state.manaMovesCount) &&
    !canMoveMonForCounts(state.monsMovesCount);
  const movableFreeRegularMana =
    !needsFreeManaCheck || hasMovableFreeRegularMana(state);
  return shouldAdvanceTurnForCounts(
    firstTurn,
    state.monsMovesCount,
    state.manaMovesCount,
    movableFreeRegularMana,
  );
}
