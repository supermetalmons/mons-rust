import { Board } from "../../engine/board/storage.js";
import { Color, Consumable, type Mon, MonKind } from "../../api/types.js";
import { isMonFainted, type Item, itemMon } from "../../engine/model/domain.js";
import {
  BOARD_CELLS,
  demonReachableLocations,
  type Location,
  locationBetween,
  locationDistance,
  locationEquals,
  locationIndex,
  mysticReachableLocations,
  nearbyLocations,
} from "../../engine/board/geometry.js";
import type { AutomoveExecutionContext } from "../core/execution-context.js";
import type { Hash64 } from "../core/hash64.js";
import { demonHasLineAttack, exactIsLocationGuardedByAngel } from "./attack-shared.js";
import { colorKey, exactCaches, exactCacheTag } from "./cache.js";
import {
  actorPayloadAfterMoveCompute,
  type AttackQueueEntry,
  BOMB_PAYLOAD,
  type ExactActorPayload,
  NO_PAYLOAD,
  payloadSeenSlot,
} from "./path.js";

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
    return [this.#actionThreatCounts[slot] ?? 0, this.#bombThreatCounts[slot] ?? 0];
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
    if (item.kind === "consumable" && item.consumable === Consumable.BombOrPotion) {
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
  const targetGuarded = exactIsLocationGuardedByAngel(board, targetColor, target);
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
    const targetMon = targetItem === undefined ? undefined : itemMon(targetItem);
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
  const targetGuarded = exactIsLocationGuardedByAngel(board, targetColor, target);
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
      if (board.squareAt(current.location).kind !== "mon-base" && !targetGuarded) {
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
      if (lowerBound !== undefined && current.steps + lowerBound > remainingMoves) {
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
  if (tag !== undefined) exactCaches(context).attackReach.set(boardHash, result, tag);
  return result;
}
