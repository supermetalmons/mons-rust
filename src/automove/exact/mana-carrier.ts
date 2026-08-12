import { Board } from "../../engine/board/storage.js";
import { Color, type Mana, MonKind } from "../../api/types.js";
import { isMonFainted, itemMon, manaScore } from "../../engine/model/domain.js";
import {
  BOARD_CELLS,
  type Location,
  locationIndex,
  nearbyLocations,
} from "../../engine/board/geometry.js";
import type { AutomoveExecutionContext } from "../core/execution-context.js";
import type { Hash64 } from "../core/hash64.js";
import { colorKey, exactCaches, exactCacheTag } from "./cache.js";
import { exactBoardHash, exactHashManaBits } from "./hash.js";
import {
  actorPayloadAfterMoveCompute,
  type AttackQueueEntry,
  NO_PAYLOAD,
  payloadSeenSlot,
  shortestPayloadState,
} from "./path.js";
import { type ExactDrainerPickupPath } from "./types.js";

export function exactCarrierStepsToAnyPoolWithHash(
  context: AutomoveExecutionContext,
  board: Board,
  start: Location,
  mana: Mana,
  boardHash: Hash64,
  maxSteps?: number,
): number | undefined {
  if (context.session.checkpoint() || (maxSteps !== undefined && maxSteps < 0)) {
    return undefined;
  }
  const tag = exactCacheTag(locationIndex(start), exactHashManaBits(mana));
  if (tag !== undefined && exactCaches(context).carrierSteps.has(boardHash, tag)) {
    const cached = exactCaches(context).carrierSteps.get(boardHash, tag);
    return maxSteps === undefined || cached === undefined || cached <= maxSteps
      ? cached
      : undefined;
  }
  const result = shortestPayloadState(
    context,
    board,
    start,
    MonKind.Drainer,
    Color.White,
    { kind: "mana", mana },
    false,
    maxSteps,
    (location, payload) =>
      payload.kind === "mana" && board.squareAt(location).kind === "mana-pool",
  );
  if (!context.session.cacheWriteAllowed) return undefined;
  if (tag !== undefined && (maxSteps === undefined || result !== undefined)) {
    exactCaches(context).carrierSteps.set(boardHash, result, tag);
  }
  return result;
}

export function exactDrainerToAnyManaSteps(
  context: AutomoveExecutionContext,
  board: Board,
  color: Color,
  start: Location,
): number | undefined {
  if (context.session.checkpoint()) return undefined;
  const boardHash = exactBoardHash(board);
  const tag = exactCacheTag(colorKey(color), locationIndex(start));
  if (tag !== undefined && exactCaches(context).drainerToMana.has(boardHash, tag)) {
    return exactCaches(context).drainerToMana.get(boardHash, tag);
  }
  const result = shortestPayloadState(
    context,
    board,
    start,
    MonKind.Drainer,
    color,
    NO_PAYLOAD,
    false,
    undefined,
    (_location, payload) => payload.kind === "mana",
  );
  if (!context.session.cacheWriteAllowed) return undefined;
  if (tag !== undefined) exactCaches(context).drainerToMana.set(boardHash, result, tag);
  return result;
}

function pickupPathBeats(
  candidate: ExactDrainerPickupPath,
  current: ExactDrainerPickupPath | undefined,
): boolean {
  if (current === undefined) return true;
  const candidateMetric = candidate.pathSteps * 3 - candidate.manaValue;
  const currentMetric = current.pathSteps * 3 - current.manaValue;
  return (
    candidateMetric < currentMetric ||
    (candidateMetric === currentMetric && candidate.manaValue > current.manaValue)
  );
}

export function exactBestDrainerPickupPathWithHash(
  context: AutomoveExecutionContext,
  board: Board,
  color: Color,
  start: Location,
  maxSteps: number | undefined,
  boardHash: Hash64,
): ExactDrainerPickupPath | undefined {
  if (context.session.checkpoint()) return undefined;
  const tag = exactCacheTag(colorKey(color), locationIndex(start), maxSteps ?? -1);
  if (tag !== undefined && exactCaches(context).pickupPath.has(boardHash, tag)) {
    return exactCaches(context).pickupPath.get(boardHash, tag);
  }
  const queue: AttackQueueEntry[] = [
    { location: start, payload: NO_PAYLOAD, steps: 0 },
  ];
  const seen = new Uint8Array(BOARD_CELLS * 5);
  seen[payloadSeenSlot(start, NO_PAYLOAD)] = 1;
  let best: ExactDrainerPickupPath | undefined;
  let cursor = 0;
  while (cursor < queue.length) {
    const current = queue[cursor];
    cursor += 1;
    if (current === undefined || context.session.checkpoint()) return undefined;
    if (maxSteps !== undefined && current.steps > maxSteps) continue;
    if (
      current.payload.kind === "mana" &&
      board.squareAt(current.location).kind === "mana-pool"
    ) {
      const candidate: ExactDrainerPickupPath = {
        pathSteps: Math.max(current.steps - 1, 0),
        totalMoves: current.steps,
        manaValue: manaScore(current.payload.mana, color),
        mana: current.payload.mana,
      };
      if (pickupPathBeats(candidate, best)) best = candidate;
    }
    if (maxSteps !== undefined && current.steps >= maxSteps) continue;
    for (const next of nearbyLocations(current.location)) {
      const nextPayload = actorPayloadAfterMoveCompute(
        board,
        MonKind.Drainer,
        color,
        current.payload,
        next,
        false,
      );
      if (nextPayload === undefined || nextPayload.kind === "bomb") continue;
      const slot = payloadSeenSlot(next, nextPayload);
      if (seen[slot] !== 0) continue;
      seen[slot] = 1;
      queue.push({
        location: next,
        payload: nextPayload,
        steps: current.steps + 1,
      });
    }
  }
  if (!context.session.cacheWriteAllowed) return undefined;
  if (tag !== undefined) exactCaches(context).pickupPath.set(boardHash, best, tag);
  return best;
}

export function exactBestScoreStepsOnBoard(
  context: AutomoveExecutionContext,
  board: Board,
  color: Color,
): number | undefined {
  if (context.session.checkpoint()) return undefined;
  const boardHash = exactBoardHash(board);
  let best: number | undefined;
  for (const [location, item] of board.entries()) {
    if (context.session.checkpoint()) return undefined;
    const mon = itemMon(item);
    if (mon?.color !== color || isMonFainted(mon)) continue;
    let steps: number | undefined;
    if (item.kind === "mon-with-mana") {
      steps = exactCarrierStepsToAnyPoolWithHash(
        context,
        board,
        location,
        item.mana,
        boardHash,
      );
    } else if (
      (item.kind === "mon" || item.kind === "mon-with-consumable") &&
      mon.kind === MonKind.Drainer
    ) {
      steps = exactBestDrainerPickupPathWithHash(
        context,
        board,
        color,
        location,
        undefined,
        boardHash,
      )?.totalMoves;
    }
    if (steps !== undefined) best = best === undefined ? steps : Math.min(best, steps);
  }
  return context.session.checkpoint() ? undefined : best;
}

export function exactBestImmediateScoreOnBoard(
  context: AutomoveExecutionContext,
  board: Board,
  color: Color,
  moveBudget: number,
): number {
  if (moveBudget < 0 || context.session.checkpoint()) return 0;
  const boardHash = exactBoardHash(board);
  let best = 0;
  for (const [location, item] of board.entries()) {
    if (context.session.checkpoint()) return 0;
    const mon = itemMon(item);
    if (mon?.color !== color || isMonFainted(mon)) continue;
    if (item.kind === "mon-with-mana") {
      if (
        exactCarrierStepsToAnyPoolWithHash(
          context,
          board,
          location,
          item.mana,
          boardHash,
          moveBudget,
        ) !== undefined
      ) {
        best = Math.max(best, manaScore(item.mana, color));
      }
    } else if (
      (item.kind === "mon" || item.kind === "mon-with-consumable") &&
      mon.kind === MonKind.Drainer
    ) {
      const pickup = exactBestDrainerPickupPathWithHash(
        context,
        board,
        color,
        location,
        moveBudget,
        boardHash,
      );
      if (pickup !== undefined) best = Math.max(best, pickup.manaValue);
    }
    if (best >= 2) return best;
  }
  return best;
}
