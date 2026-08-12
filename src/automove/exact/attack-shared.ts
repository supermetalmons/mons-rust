import { Board } from "../../engine/board/storage.js";
import type { Color } from "../../api/types.js";
import {
  type Location,
  locationBetween,
  locationDistance,
} from "../../engine/board/geometry.js";

export function exactIsLocationGuardedByAngel(
  board: Board,
  color: Color,
  location: Location,
): boolean {
  const angel = board.findAwakeAngel(color);
  return angel !== undefined && locationDistance(angel, location) === 1;
}

export function demonHasLineAttack(
  board: Board,
  source: Location,
  target: Location,
): boolean {
  const deltaI = Math.abs(source.i - target.i);
  const deltaJ = Math.abs(source.j - target.j);
  const middle = locationBetween(source, target);
  const middleSquare = board.squareAt(middle);
  return (
    ((deltaI === 2 && deltaJ === 0) || (deltaI === 0 && deltaJ === 2)) &&
    board.get(middle) === undefined &&
    middleSquare.kind !== "supermana-base" &&
    middleSquare.kind !== "mon-base"
  );
}
