import { Board } from "../engine/board.js";
import { MONS_MOVES_PER_TURN } from "../engine/config.js";
import {
  Color,
  Consumable,
  isMonFainted,
  type Item,
  itemMon,
  type Mon,
  MonKind,
  otherColor,
} from "../engine/domain.js";
import {
  BOARD_CELLS,
  bombReachableLocations,
  demonReachableLocations,
  type Location,
  locationBetween,
  locationDistance,
  locationEquals,
  locationIndex,
  mysticReachableLocations,
  nearbyLocations,
} from "../engine/geometry.js";
import { colorKey, exactCaches, exactCacheTag } from "./exact-cache.js";
import { exactBoardHash } from "./exact-hash.js";
import {
  actorPayloadAfterMoveCompute,
  type AttackQueueEntry,
  BOMB_PAYLOAD,
  type ExactActorPayload,
  NO_PAYLOAD,
  payloadSeenSlot,
} from "./exact-path.js";
import { type AutomoveExecutionContext } from "./execution-context.js";
import { type Hash64 } from "./hash64.js";

export class AttackReachSummary {
  readonly #actionThreatCounts: Uint8Array;
  readonly #bombThreatCounts: Uint8Array;
  readonly #guardedTargets: Uint8Array;

  public constructor() {
    this.#actionThreatCounts = new Uint8Array(BOARD_CELLS);
    this.#bombThreatCounts = new Uint8Array(BOARD_CELLS);
    this.#guardedTargets = new Uint8Array(BOARD_CELLS);
  }

  public canAttackTarget(target: Location): boolean {
    const slot = locationIndex(target);
    return (
      (this.#bombThreatCounts[slot] ?? 0) > 0 ||
      ((this.#guardedTargets[slot] ?? 0) === 0 &&
        (this.#actionThreatCounts[slot] ?? 0) > 0)
    );
  }

  public immediateThreats(target: Location): readonly [number, number] {
    const slot = locationIndex(target);
    return [
      this.#actionThreatCounts[slot] ?? 0,
      this.#bombThreatCounts[slot] ?? 0,
    ];
  }

  public markGuarded(target: Location, guarded: boolean): void {
    this.#guardedTargets[locationIndex(target)] = guarded ? 1 : 0;
  }

  public addActionThreat(target: Location): void {
    const slot = locationIndex(target);
    this.#actionThreatCounts[slot] = Math.min(
      0xff,
      (this.#actionThreatCounts[slot] ?? 0) + 1,
    );
  }

  public addBombThreat(target: Location): void {
    const slot = locationIndex(target);
    this.#bombThreatCounts[slot] = Math.min(
      0xff,
      (this.#bombThreatCounts[slot] ?? 0) + 1,
    );
  }
}

function exactIsLocationGuardedByAngel(
  board: Board,
  color: Color,
  location: Location,
): boolean {
  const angel = board.findAwakeAngel(color);
  return angel !== undefined && locationDistance(angel, location) === 1;
}

function demonHasLineAttack(
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

function exactAttackPayloadAfterMove(
  board: Board,
  monKind: MonKind,
  color: Color,
  payload: ExactActorPayload,
  destination: Location,
  allowPickBomb: boolean,
): ExactActorPayload | undefined {
  if (payload.kind === "mana" || board.get(destination)?.kind === "mana") {
    return undefined;
  }
  return actorPayloadAfterMoveCompute(
    board,
    monKind,
    color,
    payload,
    destination,
    allowPickBomb,
  );
}

function exactAttackActionSourceAvailable(
  board: Board,
  currentLocation: Location,
  source: Location,
): boolean {
  return (
    board.squareAt(source).kind !== "mon-base" &&
    (locationEquals(source, currentLocation) || board.get(source) === undefined)
  );
}

function exactAttackActionStepsLowerBound(
  board: Board,
  monKind: MonKind,
  location: Location,
  target: Location,
): number | undefined {
  let sources: readonly Location[];
  switch (monKind) {
    case MonKind.Mystic:
      sources = mysticReachableLocations(target);
      break;
    case MonKind.Demon:
      sources = demonReachableLocations(target);
      break;
    case MonKind.Drainer:
    case MonKind.Angel:
    case MonKind.Spirit:
      return undefined;
  }
  let best: number | undefined;
  for (const source of sources) {
    if (!exactAttackActionSourceAvailable(board, location, source)) continue;
    if (
      monKind === MonKind.Demon &&
      board.get(locationBetween(source, target)) !== undefined
    ) {
      continue;
    }
    const distance = locationDistance(location, source);
    best = best === undefined ? distance : Math.min(best, distance);
  }
  return best;
}

function exactAttackRemainingStepsLowerBound(
  board: Board,
  target: Location,
  targetGuarded: boolean,
  bombPickupLocations: readonly Location[],
  location: Location,
  payload: ExactActorPayload,
  monKind: MonKind,
  allowPickBomb: boolean,
): number | undefined {
  if (payload.kind === "bomb") {
    return Math.max(locationDistance(location, target) - 3, 0);
  }
  if (payload.kind === "mana") return undefined;
  let best = targetGuarded
    ? undefined
    : exactAttackActionStepsLowerBound(board, monKind, location, target);
  if (allowPickBomb) {
    for (const bombLocation of bombPickupLocations) {
      const candidate =
        locationDistance(location, bombLocation) +
        Math.max(locationDistance(bombLocation, target) - 3, 0);
      best = best === undefined ? candidate : Math.min(best, candidate);
    }
  }
  return best;
}

function bombPickupLocations(board: Board): Location[] {
  const result: Location[] = [];
  for (const [location, item] of board.entries()) {
    if (
      item.kind === "consumable" &&
      item.consumable === Consumable.BombOrPotion
    ) {
      result.push(location);
    }
  }
  return result;
}

function exactAttackTargetPlausibleForAttacker(
  context: AutomoveExecutionContext,
  board: Board,
  target: Location,
  remainingMoves: number,
  targetGuarded: boolean,
  location: Location,
  item: Item,
  mon: Mon,
  bombs: readonly Location[],
): boolean {
  if (context.session.checkpoint()) return false;
  if (
    item.kind === "mon-with-consumable" &&
    item.consumable === Consumable.Bomb &&
    locationDistance(location, target) <= remainingMoves + 3
  ) {
    return true;
  }
  if (!targetGuarded) {
    const distance = exactAttackActionStepsLowerBound(
      board,
      mon.kind,
      location,
      target,
    );
    if (distance !== undefined && distance <= remainingMoves) return true;
  }
  if (item.kind === "mon-with-mana") return false;
  for (const bombLocation of bombs) {
    if (context.session.checkpoint()) return false;
    const toBomb = locationDistance(location, bombLocation);
    if (toBomb > remainingMoves) continue;
    if (locationDistance(bombLocation, target) <= remainingMoves - toBomb + 3) {
      return true;
    }
  }
  return false;
}

function exactAttackTargetPlausibleOnBoard(
  context: AutomoveExecutionContext,
  board: Board,
  attackerColor: Color,
  targetColor: Color,
  target: Location,
  remainingMoves: number,
  canUseAction: boolean,
): boolean {
  if (
    remainingMoves < 0 ||
    !canUseAction ||
    board.get(target) === undefined ||
    context.session.checkpoint()
  ) {
    return false;
  }
  const targetGuarded = exactIsLocationGuardedByAngel(
    board,
    targetColor,
    target,
  );
  const bombs = bombPickupLocations(board);
  for (const [location, item] of board.entries()) {
    if (context.session.checkpoint()) return false;
    const mon = itemMon(item);
    if (mon?.color !== attackerColor || isMonFainted(mon)) {
      continue;
    }
    if (
      exactAttackTargetPlausibleForAttacker(
        context,
        board,
        target,
        remainingMoves,
        targetGuarded,
        location,
        item,
        mon,
        bombs,
      )
    ) {
      return true;
    }
  }
  return false;
}

export function attackReachSummaryTargetLocations(
  board: Board,
  targetColor: Color,
): Location[] {
  const targets: Location[] = [];
  for (const [location, item] of board.entries()) {
    if (itemMon(item)?.color === targetColor) targets.push(location);
  }
  return targets;
}

export function attackReachSummaryForTargets(
  context: AutomoveExecutionContext,
  board: Board,
  attackerColor: Color,
  remainingMoves: number,
  canUseAction: boolean,
  targets: readonly Location[],
): AttackReachSummary {
  const summary = new AttackReachSummary();
  if (
    remainingMoves < 0 ||
    !canUseAction ||
    targets.length === 0 ||
    context.session.checkpoint()
  ) {
    return summary;
  }
  for (const target of targets) {
    if (context.session.checkpoint()) return new AttackReachSummary();
    const targetItem = board.get(target);
    const targetMon =
      targetItem === undefined ? undefined : itemMon(targetItem);
    if (targetMon !== undefined) {
      summary.markGuarded(
        target,
        exactIsLocationGuardedByAngel(board, targetMon.color, target),
      );
    }
  }
  for (const [start, item] of board.entries()) {
    if (context.session.checkpoint()) return new AttackReachSummary();
    const mon = itemMon(item);
    if (mon?.color !== attackerColor || isMonFainted(mon)) {
      continue;
    }
    const allowPickBomb = item.kind !== "mon-with-mana";
    const startPayload =
      item.kind === "mon-with-consumable" && item.consumable === Consumable.Bomb
        ? BOMB_PAYLOAD
        : NO_PAYLOAD;
    const queue: AttackQueueEntry[] = [
      { location: start, payload: startPayload, steps: 0 },
    ];
    const seen = new Uint8Array(BOARD_CELLS * 5);
    seen[payloadSeenSlot(start, startPayload)] = 1;
    let cursor = 0;
    while (cursor < queue.length) {
      const current = queue[cursor];
      cursor += 1;
      if (current === undefined) break;
      if (context.session.checkpoint()) return new AttackReachSummary();
      if (current.steps > remainingMoves) continue;
      if (current.payload.kind === "bomb") {
        for (const target of targets) {
          if (locationDistance(current.location, target) <= 3) {
            summary.addBombThreat(target);
          }
        }
      }
      if (board.squareAt(current.location).kind !== "mon-base") {
        for (const target of targets) {
          if (
            mon.kind === MonKind.Mystic &&
            Math.abs(current.location.i - target.i) === 2 &&
            Math.abs(current.location.j - target.j) === 2
          ) {
            summary.addActionThreat(target);
          } else if (
            mon.kind === MonKind.Demon &&
            demonHasLineAttack(board, current.location, target)
          ) {
            summary.addActionThreat(target);
          }
        }
      }
      if (current.steps === remainingMoves) continue;
      for (const next of nearbyLocations(current.location)) {
        const nextPayload = exactAttackPayloadAfterMove(
          board,
          mon.kind,
          mon.color,
          current.payload,
          next,
          allowPickBomb,
        );
        if (nextPayload === undefined) continue;
        const seenSlot = payloadSeenSlot(next, nextPayload);
        if (seen[seenSlot] !== 0) continue;
        seen[seenSlot] = 1;
        queue.push({
          location: next,
          payload: nextPayload,
          steps: current.steps + 1,
        });
      }
    }
  }
  return context.session.checkpoint() ? new AttackReachSummary() : summary;
}

export function attackReachSummary(
  context: AutomoveExecutionContext,
  board: Board,
  attackerColor: Color,
  targetColor: Color,
  remainingMoves: number,
  canUseAction: boolean,
): AttackReachSummary {
  return attackReachSummaryForTargets(
    context,
    board,
    attackerColor,
    remainingMoves,
    canUseAction,
    attackReachSummaryTargetLocations(board, targetColor),
  );
}

function canAttackTargetOnBoardUncached(
  context: AutomoveExecutionContext,
  board: Board,
  attackerColor: Color,
  targetColor: Color,
  target: Location,
  remainingMoves: number,
): boolean {
  if (context.session.checkpoint()) return false;
  const targetGuarded = exactIsLocationGuardedByAngel(
    board,
    targetColor,
    target,
  );
  const bombs = bombPickupLocations(board);
  for (const [start, item] of board.entries()) {
    if (context.session.checkpoint()) return false;
    const mon = itemMon(item);
    if (
      mon?.color !== attackerColor ||
      isMonFainted(mon) ||
      !exactAttackTargetPlausibleForAttacker(
        context,
        board,
        target,
        remainingMoves,
        targetGuarded,
        start,
        item,
        mon,
        bombs,
      )
    ) {
      continue;
    }
    const allowPickBomb = item.kind !== "mon-with-mana";
    const startPayload =
      item.kind === "mon-with-consumable" && item.consumable === Consumable.Bomb
        ? BOMB_PAYLOAD
        : NO_PAYLOAD;
    const queue: AttackQueueEntry[] = [
      { location: start, payload: startPayload, steps: 0 },
    ];
    const seen = new Uint8Array(BOARD_CELLS * 5);
    seen[payloadSeenSlot(start, startPayload)] = 1;
    let cursor = 0;
    while (cursor < queue.length) {
      const current = queue[cursor];
      cursor += 1;
      if (current === undefined || context.session.checkpoint()) return false;
      if (current.steps > remainingMoves) continue;
      if (
        current.payload.kind === "bomb" &&
        board.get(target) !== undefined &&
        locationDistance(current.location, target) <= 3
      ) {
        return true;
      }
      if (
        board.squareAt(current.location).kind !== "mon-base" &&
        !targetGuarded
      ) {
        if (
          mon.kind === MonKind.Mystic &&
          Math.abs(current.location.i - target.i) === 2 &&
          Math.abs(current.location.j - target.j) === 2
        ) {
          return true;
        }
        if (
          mon.kind === MonKind.Demon &&
          demonHasLineAttack(board, current.location, target)
        ) {
          return true;
        }
      }
      if (current.steps === remainingMoves) continue;
      const lowerBound = exactAttackRemainingStepsLowerBound(
        board,
        target,
        targetGuarded,
        bombs,
        current.location,
        current.payload,
        mon.kind,
        allowPickBomb,
      );
      if (
        lowerBound !== undefined &&
        current.steps + lowerBound > remainingMoves
      ) {
        continue;
      }
      for (const next of nearbyLocations(current.location)) {
        const nextPayload = exactAttackPayloadAfterMove(
          board,
          mon.kind,
          mon.color,
          current.payload,
          next,
          allowPickBomb,
        );
        if (nextPayload === undefined) continue;
        const seenSlot = payloadSeenSlot(next, nextPayload);
        if (seen[seenSlot] !== 0) continue;
        seen[seenSlot] = 1;
        queue.push({
          location: next,
          payload: nextPayload,
          steps: current.steps + 1,
        });
      }
    }
  }
  return false;
}

export function canAttackTargetOnBoardWithHash(
  context: AutomoveExecutionContext,
  board: Board,
  boardHash: Hash64,
  attackerColor: Color,
  targetColor: Color,
  target: Location,
  remainingMoves: number,
  canUseAction: boolean,
): boolean {
  if (
    remainingMoves < 0 ||
    !canUseAction ||
    board.get(target) === undefined ||
    context.session.checkpoint()
  ) {
    return false;
  }
  const tag = exactCacheTag(
    colorKey(attackerColor),
    colorKey(targetColor),
    locationIndex(target),
    remainingMoves,
    1,
  );
  const cached =
    tag === undefined
      ? undefined
      : exactCaches(context).attackReach.get(boardHash, tag);
  if (cached !== undefined) return cached;
  if (
    !exactAttackTargetPlausibleOnBoard(
      context,
      board,
      attackerColor,
      targetColor,
      target,
      remainingMoves,
      canUseAction,
    )
  ) {
    if (context.session.cacheWriteAllowed && tag !== undefined) {
      exactCaches(context).attackReach.set(boardHash, false, tag);
    }
    return false;
  }
  const result = canAttackTargetOnBoardUncached(
    context,
    board,
    attackerColor,
    targetColor,
    target,
    remainingMoves,
  );
  if (!context.session.cacheWriteAllowed) return false;
  if (tag !== undefined)
    exactCaches(context).attackReach.set(boardHash, result, tag);
  return result;
}

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
    tag === undefined
      ? undefined
      : exactCaches(context).walkThreat.get(boardHash, tag);
  if (cached !== undefined) return cached || context.session.checkpoint();
  const result = isDrainerUnderWalkThreatUncached(
    context,
    board,
    color,
    location,
    angelNearby,
  );
  if (!context.session.cacheWriteAllowed) return true;
  if (tag !== undefined)
    exactCaches(context).walkThreat.set(boardHash, result, tag);
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

export function findAwakeDrainer(
  board: Board,
  color: Color,
): Location | undefined {
  for (const [location, item] of board.entries()) {
    const mon = itemMon(item);
    if (
      mon?.color === color &&
      mon.kind === MonKind.Drainer &&
      !isMonFainted(mon)
    ) {
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
    const angelNearby = exactIsLocationGuardedByAngel(
      board,
      color,
      drainerLocation,
    );
    const [actionThreats, bombThreats] = drainerImmediateThreats(
      context,
      board,
      color,
      drainerLocation,
    );
    const immediate = angelNearby
      ? bombThreats > 0
      : actionThreats + bombThreats > 0;
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
    result = exactSafe
      ? immediate || walk
        ? 1
        : 2
      : immediate || walk
        ? -2
        : -1;
  }
  if (!context.session.cacheWriteAllowed) return 0;
  if (tag !== undefined)
    exactCaches(context).drainerSafety.set(boardHash, result, tag);
  return result;
}
