import { Board } from "../../engine/board/storage.js";
import type { AutomoveExecutionContext } from "../core/execution-context.js";
import { MONS_MOVES_PER_TURN, TARGET_SCORE } from "../../engine/board/config.js";
import { Color, Consumable, MonKind, type Mana } from "../../api/types.js";
import {
  isMonFainted,
  itemConsumable,
  itemMana,
  itemMon,
  manaEquals,
  otherColor,
  type Item,
} from "../../engine/model/domain.js";
import type { MonsGame } from "../../engine/game/mons-game.js";
import {
  BOARD_SIZE,
  bombReachableLocations,
  demonReachableLocations,
  locationBetween,
  locationDistance,
  locationEquals,
  mysticReachableLocations,
  type Location,
} from "../../engine/board/geometry.js";
import {
  scoreForColor,
  spiritDestinationItemAllowed,
} from "../../engine/rules/legality.js";
import { activeTurnScoreWindow, ownDrainerSafetyScore } from "./evaluation.js";
import type { TurnAction } from "./model.js";
import { actionKey } from "./ordering.js";

export function actorCanAttackFromItem(item: Item): boolean {
  const mon = itemMon(item);
  return (
    mon !== undefined && (mon.kind === MonKind.Mystic || mon.kind === MonKind.Demon)
  );
}

export function actorCanBombFromItem(item: Item): boolean {
  const mon = itemMon(item);
  return (
    mon !== undefined &&
    !isMonFainted(mon) &&
    item.kind === "mon-with-consumable" &&
    item.consumable === Consumable.Bomb
  );
}

function locationGuardedByAngel(
  angelLocation: Location | undefined,
  at: Location,
): boolean {
  return angelLocation !== undefined && locationDistance(angelLocation, at) === 1;
}

function demonAttackPathClear(board: Board, from: Location, target: Location): boolean {
  const middle = locationBetween(from, target);
  const square = board.squareAt(middle);
  return (
    board.get(middle) === undefined &&
    square.kind !== "supermana-base" &&
    square.kind !== "mon-base"
  );
}

export function actorCanAttackTargetNow(
  board: Board,
  actor: Location,
  target: Location,
  item: Item,
  perspective: Color,
): boolean {
  if (board.squareAt(actor).kind === "mon-base") return false;
  const targetItem = board.get(target);
  const targetMon = targetItem === undefined ? undefined : itemMon(targetItem);
  if (
    targetMon?.color !== otherColor(perspective) ||
    isMonFainted(targetMon) ||
    locationGuardedByAngel(board.findAwakeAngel(otherColor(perspective)), target)
  ) {
    return false;
  }
  const mon = itemMon(item);
  if (mon?.kind === MonKind.Mystic) {
    return mysticReachableLocations(actor).some((at) => locationEquals(at, target));
  }
  return (
    mon?.kind === MonKind.Demon &&
    demonReachableLocations(actor).some((at) => locationEquals(at, target)) &&
    demonAttackPathClear(board, actor, target)
  );
}

export function actorCanBombTargetNow(
  board: Board,
  actor: Location,
  target: Location,
  item: Item,
  perspective: Color,
): boolean {
  if (!bombReachableLocations(actor).some((at) => locationEquals(at, target)))
    return false;
  if (!actorCanBombFromItem(item) || itemMon(item)?.color !== perspective) return false;
  const targetItem = board.get(target);
  const targetMon = targetItem === undefined ? undefined : itemMon(targetItem);
  return targetMon?.color === otherColor(perspective) && !isMonFainted(targetMon);
}

export function spiritDestinationAllowed(
  board: Board,
  targetItem: Item,
  destination: Location,
): boolean {
  const destinationItem = board.get(destination);
  const square = board.squareAt(destination);
  const targetMon = itemMon(targetItem);
  const targetMana = itemMana(targetItem);
  if (!spiritDestinationItemAllowed(targetItem, destinationItem)) return false;
  switch (square.kind) {
    case "regular":
    case "consumable-base":
    case "mana-base":
    case "mana-pool":
      return true;
    case "supermana-base":
      return (
        targetMana?.kind === "supermana" ||
        (targetMana === undefined && targetMon?.kind === MonKind.Drainer)
      );
    case "mon-base":
      return (
        targetMon?.kind === square.monKind &&
        targetMon.color === square.color &&
        targetMana === undefined &&
        itemConsumable(targetItem) === undefined
      );
  }
}

export function manaMoveDestinationAllowed(
  board: Board,
  destination: Location,
): boolean {
  const item = board.get(destination);
  const square = board.squareAt(destination);
  const ordinarySquare =
    square.kind === "regular" ||
    square.kind === "consumable-base" ||
    square.kind === "mana-base" ||
    square.kind === "mana-pool";
  if (item === undefined) return ordinarySquare;
  return (
    item.kind === "mon" &&
    ordinarySquare &&
    item.mon.kind === MonKind.Drainer &&
    !isMonFainted(item.mon)
  );
}

export function nearestWantedManaLocation(
  board: Board,
  wanted: Mana,
): Location | undefined {
  for (const [at, item] of board.entries()) {
    if (item.kind === "mana" && manaEquals(item.mana, wanted)) return at;
  }
  return undefined;
}

export function walkDestinationPlausible(
  board: Board,
  actor: Location,
  destination: Location,
): boolean {
  const actorItem = board.get(actor);
  const actorMon = actorItem === undefined ? undefined : itemMon(actorItem);
  if (actorMon === undefined) return false;
  const destinationItem = board.get(destination);
  if (destinationItem !== undefined && itemMon(destinationItem) !== undefined)
    return false;
  const square = board.squareAt(destination);
  switch (square.kind) {
    case "regular":
    case "consumable-base":
    case "mana-base":
    case "mana-pool":
      return true;
    case "supermana-base":
      return actorMon.kind === MonKind.Drainer;
    case "mon-base":
      return actorMon.kind === square.monKind && actorMon.color === square.color;
  }
}

export function remainingMovesForColor(game: MonsGame, color: Color): number {
  return game.activeColor === color
    ? Math.max(MONS_MOVES_PER_TURN - game.monsMovesCount, 0)
    : MONS_MOVES_PER_TURN;
}

export function distanceToNearestPool(at: Location, color: Color): number {
  const row = color === Color.Black ? 0 : BOARD_SIZE - 1;
  return Math.min(
    locationDistance(at, { i: row, j: 0 }),
    locationDistance(at, { i: row, j: BOARD_SIZE - 1 }),
  );
}

export function opponentDrainerKillIsHighValue(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  target: Location,
): boolean {
  const opponent = otherColor(perspective);
  return (
    ownDrainerSafetyScore(execution, game.board, perspective) < 0 ||
    activeTurnScoreWindow(execution, game, opponent) > 0 ||
    scoreForColor(game, opponent) >= TARGET_SCORE - 2 ||
    game.board.get(target)?.kind === "mon-with-mana" ||
    distanceToNearestPool(target, opponent) <= 3
  );
}

export function actionIdentity(action: TurnAction): string {
  if (action.kind !== "score-carry") return actionKey(action);
  return `${actionKey(action)}:${action.wanted.kind}:${
    action.wanted.kind === "regular" ? action.wanted.color : "s"
  }`;
}
