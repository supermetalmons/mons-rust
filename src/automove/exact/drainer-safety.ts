import { Board } from "../../engine/board/storage.js";
import { MONS_MOVES_PER_TURN } from "../../engine/board/config.js";
import { Color, Consumable, MonKind } from "../../api/types.js";
import { isMonFainted, itemMon, otherColor } from "../../engine/model/domain.js";
import {
  bombReachableLocations,
  demonReachableLocations,
  type Location,
  locationDistance,
  locationIndex,
  mysticReachableLocations,
} from "../../engine/board/geometry.js";
import type { AutomoveExecutionContext } from "../core/execution-context.js";
import type { Hash64 } from "../core/hash64.js";
import { canAttackTargetOnBoardWithHash } from "./attack-reach.js";
import { demonHasLineAttack, exactIsLocationGuardedByAngel } from "./attack-shared.js";
import { colorKey, exactCaches, exactCacheTag } from "./cache.js";
import { exactBoardHash } from "./hash.js";

export function drainerImmediateThreats(
  context: AutomoveExecutionContext,
  board: Board,
  color: Color,
  location: Location,
): readonly [number, number] {
  if (context.session.checkpoint()) return [1, 1];
  let actionThreats = 0;
  let bombThreats = 0;
  for (const threatLocation of mysticReachableLocations(location)) {
    if (context.session.checkpoint()) return [1, 1];
    const item = board.get(threatLocation);
    const mon = item === undefined ? undefined : itemMon(item);
    if (
      mon?.kind === MonKind.Mystic &&
      mon.color !== color &&
      !isMonFainted(mon) &&
      board.squareAt(threatLocation).kind !== "mon-base"
    ) {
      actionThreats += 1;
    }
  }
  for (const threatLocation of demonReachableLocations(location)) {
    if (context.session.checkpoint()) return [1, 1];
    const item = board.get(threatLocation);
    const mon = item === undefined ? undefined : itemMon(item);
    if (
      mon?.kind === MonKind.Demon &&
      mon.color !== color &&
      !isMonFainted(mon) &&
      board.squareAt(threatLocation).kind !== "mon-base" &&
      demonHasLineAttack(board, threatLocation, location)
    ) {
      actionThreats += 1;
    }
  }
  for (const threatLocation of bombReachableLocations(location)) {
    if (context.session.checkpoint()) return [1, 1];
    const item = board.get(threatLocation);
    if (
      item?.kind === "mon-with-consumable" &&
      item.consumable === Consumable.Bomb &&
      item.mon.color !== color &&
      !isMonFainted(item.mon) &&
      board.squareAt(threatLocation).kind !== "mon-base"
    ) {
      bombThreats += 1;
    }
  }
  return context.session.checkpoint() ? [1, 1] : [actionThreats, bombThreats];
}

export function isDrainerUnderImmediateThreat(
  context: AutomoveExecutionContext,
  board: Board,
  color: Color,
  location: Location,
  angelNearby: boolean,
): boolean {
  if (context.session.checkpoint()) return true;
  const [actionThreats, bombThreats] = drainerImmediateThreats(
    context,
    board,
    color,
    location,
  );
  if (context.session.checkpoint()) return true;
  return angelNearby ? bombThreats > 0 : actionThreats + bombThreats > 0;
}

function isDrainerUnderWalkThreatUncached(
  context: AutomoveExecutionContext,
  board: Board,
  color: Color,
  location: Location,
  angelNearby: boolean,
): boolean {
  if (context.session.checkpoint()) return false;
  if (angelNearby) {
    for (const [threatLocation, item] of board.entries()) {
      if (
        item.kind === "mon-with-consumable" &&
        item.consumable === Consumable.Bomb &&
        item.mon.color !== color &&
        !isMonFainted(item.mon) &&
        board.squareAt(threatLocation).kind !== "mon-base" &&
        locationDistance(threatLocation, location) <= 4
      ) {
        return true;
      }
    }
    return false;
  }
  for (const [threatLocation, item] of board.entries()) {
    if (context.session.checkpoint()) return false;
    const mon = itemMon(item);
    if (
      mon === undefined ||
      mon.color === color ||
      isMonFainted(mon) ||
      board.squareAt(threatLocation).kind === "mon-base"
    ) {
      continue;
    }
    if (mon.kind === MonKind.Mystic || mon.kind === MonKind.Demon) {
      for (let deltaI = -1; deltaI <= 1; deltaI += 1) {
        for (let deltaJ = -1; deltaJ <= 1; deltaJ += 1) {
          if (deltaI === 0 && deltaJ === 0) continue;
          const neighbor = {
            i: threatLocation.i + deltaI,
            j: threatLocation.j + deltaJ,
          };
          if (
            neighbor.i < 0 ||
            neighbor.i > 10 ||
            neighbor.j < 0 ||
            neighbor.j > 10 ||
            board.get(neighbor) !== undefined
          ) {
            continue;
          }
          const square = board.squareAt(neighbor);
          if (square.kind === "mon-base" || square.kind === "supermana-base") {
            continue;
          }
          if (
            mon.kind === MonKind.Mystic &&
            Math.abs(neighbor.i - location.i) === 2 &&
            Math.abs(neighbor.j - location.j) === 2
          ) {
            return true;
          }
          if (
            mon.kind === MonKind.Demon &&
            demonHasLineAttack(board, neighbor, location)
          ) {
            return true;
          }
        }
      }
    }
    if (
      item.kind === "mon-with-consumable" &&
      item.consumable === Consumable.Bomb &&
      locationDistance(threatLocation, location) <= 4
    ) {
      return true;
    }
  }
  return false;
}

export function isDrainerUnderWalkThreatWithHash(
  context: AutomoveExecutionContext,
  board: Board,
  boardHash: Hash64,
  color: Color,
  location: Location,
  angelNearby: boolean,
): boolean {
  if (context.session.checkpoint()) return true;
  const tag = exactCacheTag(
    colorKey(color),
    locationIndex(location),
    angelNearby ? 1 : 0,
  );
  const cached =
    tag === undefined ? undefined : exactCaches(context).walkThreat.get(boardHash, tag);
  if (cached !== undefined) return cached || context.session.checkpoint();
  const result = isDrainerUnderWalkThreatUncached(
    context,
    board,
    color,
    location,
    angelNearby,
  );
  if (!context.session.cacheWriteAllowed) return true;
  if (tag !== undefined) exactCaches(context).walkThreat.set(boardHash, result, tag);
  return result;
}

export function isDrainerExactlySafeNextTurnOnBoardWithHash(
  context: AutomoveExecutionContext,
  board: Board,
  boardHash: Hash64,
  color: Color,
  location: Location,
): boolean {
  if (context.session.checkpoint()) return false;
  const angelNearby = exactIsLocationGuardedByAngel(board, color, location);
  const canAttack = canAttackTargetOnBoardWithHash(
    context,
    board,
    boardHash,
    otherColor(color),
    color,
    location,
    MONS_MOVES_PER_TURN,
    true,
  );
  if (context.session.checkpoint()) return false;
  return (
    !canAttack &&
    !isDrainerUnderWalkThreatWithHash(
      context,
      board,
      boardHash,
      color,
      location,
      angelNearby,
    ) &&
    !context.session.checkpoint()
  );
}

export function isDrainerExactlySafeNextTurnOnBoard(
  context: AutomoveExecutionContext,
  board: Board,
  color: Color,
  location: Location,
): boolean {
  return isDrainerExactlySafeNextTurnOnBoardWithHash(
    context,
    board,
    exactBoardHash(board),
    color,
    location,
  );
}

export function findAwakeDrainer(board: Board, color: Color): Location | undefined {
  for (const [location, item] of board.entries()) {
    const mon = itemMon(item);
    if (mon?.color === color && mon.kind === MonKind.Drainer && !isMonFainted(mon)) {
      return location;
    }
  }
  return undefined;
}

export function exactOwnDrainerSafetyScoreWithHash(
  context: AutomoveExecutionContext,
  board: Board,
  boardHash: Hash64,
  color: Color,
): number {
  if (context.session.checkpoint()) return 0;
  const tag = exactCacheTag(colorKey(color));
  const cached =
    tag === undefined
      ? undefined
      : exactCaches(context).drainerSafety.get(boardHash, tag);
  if (cached !== undefined) return cached;
  const drainerLocation = findAwakeDrainer(board, color);
  let result = 0;
  if (drainerLocation !== undefined) {
    const angelNearby = exactIsLocationGuardedByAngel(board, color, drainerLocation);
    const [actionThreats, bombThreats] = drainerImmediateThreats(
      context,
      board,
      color,
      drainerLocation,
    );
    const immediate = angelNearby ? bombThreats > 0 : actionThreats + bombThreats > 0;
    const walk = isDrainerUnderWalkThreatWithHash(
      context,
      board,
      boardHash,
      color,
      drainerLocation,
      angelNearby,
    );
    const exactSafe = isDrainerExactlySafeNextTurnOnBoardWithHash(
      context,
      board,
      boardHash,
      color,
      drainerLocation,
    );
    result = exactSafe ? (immediate || walk ? 1 : 2) : immediate || walk ? -2 : -1;
  }
  if (!context.session.cacheWriteAllowed) return 0;
  if (tag !== undefined) exactCaches(context).drainerSafety.set(boardHash, result, tag);
  return result;
}
