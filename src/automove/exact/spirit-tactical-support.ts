import { Board, type MutableBoard } from "../../engine/board/storage.js";
import { Color, Consumable, MonKind } from "../../api/types.js";
import {
  isMonFainted,
  type Item,
  itemMana,
  itemMon,
  manaEquals,
  manaScore,
  monItem,
  monWithManaItem,
  otherColor,
} from "../../engine/model/domain.js";
import {
  BOARD_CELLS,
  type Location,
  nearbyLocations,
} from "../../engine/board/geometry.js";
import type { AutomoveExecutionContext } from "../core/execution-context.js";
import { exactBoardHash } from "./hash.js";
import { exactCarrierStepsToAnyPoolWithHash } from "./mana-carrier.js";
import {
  actorPayloadAfterMoveCompute,
  type AttackQueueEntry,
  NO_PAYLOAD,
  payloadSeenSlot,
} from "./path.js";

export type ImmediateTacticalWindow = {
  readonly bestScore: number;
  readonly bestOpponentManaScore: number;
};

function exactDrainerImmediateTacticalWindow(
  context: AutomoveExecutionContext,
  board: Board,
  color: Color,
  start: Location,
  moveBudget: number,
  minScore: number,
  needScore: boolean,
  needDenial: boolean,
): ImmediateTacticalWindow {
  if ((!needScore && !needDenial) || context.session.checkpoint()) {
    return { bestScore: 0, bestOpponentManaScore: 0 };
  }
  const maxScore = needScore ? manaScore({ kind: "supermana" }, color) : 0;
  const maxOpponentManaScore = needDenial
    ? manaScore({ kind: "regular", color: otherColor(color) }, color)
    : 0;
  const queue: AttackQueueEntry[] = [
    { location: start, payload: NO_PAYLOAD, steps: 0 },
  ];
  const seen = new Uint8Array(BOARD_CELLS * 5);
  seen[payloadSeenSlot(start, NO_PAYLOAD)] = 1;
  let bestScore = 0;
  let bestOpponentManaScore = 0;
  let cursor = 0;
  while (cursor < queue.length) {
    const current = queue[cursor];
    cursor += 1;
    if (current === undefined || context.session.checkpoint()) {
      return { bestScore: 0, bestOpponentManaScore: 0 };
    }
    if (
      current.payload.kind === "mana" &&
      board.squareAt(current.location).kind === "mana-pool"
    ) {
      const score = manaScore(current.payload.mana, color);
      if (needScore && score >= minScore) {
        bestScore = Math.max(bestScore, score);
      }
      if (
        needDenial &&
        current.payload.mana.kind === "regular" &&
        current.payload.mana.color === otherColor(color)
      ) {
        bestOpponentManaScore = Math.max(bestOpponentManaScore, score);
      }
      if (
        (!needScore || bestScore >= maxScore) &&
        (!needDenial || bestOpponentManaScore >= maxOpponentManaScore)
      ) {
        return { bestScore, bestOpponentManaScore };
      }
    }
    if (current.steps >= moveBudget) continue;
    for (const next of nearbyLocations(current.location)) {
      if (context.session.checkpoint()) {
        return { bestScore: 0, bestOpponentManaScore: 0 };
      }
      const payload = actorPayloadAfterMoveCompute(
        board,
        MonKind.Drainer,
        color,
        current.payload,
        next,
        false,
      );
      if (payload === undefined || payload.kind === "bomb") continue;
      const slot = payloadSeenSlot(next, payload);
      if (seen[slot] !== 0) continue;
      seen[slot] = 1;
      queue.push({ location: next, payload, steps: current.steps + 1 });
    }
  }
  return { bestScore, bestOpponentManaScore };
}

export function exactBestImmediateTacticalWindow(
  context: AutomoveExecutionContext,
  board: Board,
  color: Color,
  moveBudget: number,
  needScore: boolean,
  needDenial: boolean,
  minScore = 1,
): ImmediateTacticalWindow {
  if (moveBudget < 0 || (!needScore && !needDenial) || context.session.checkpoint()) {
    return { bestScore: 0, bestOpponentManaScore: 0 };
  }
  const scoreFloor = needScore ? Math.max(minScore, 1) : 0;
  const opponentMana = { kind: "regular", color: otherColor(color) } as const;
  const maxScore = needScore ? manaScore({ kind: "supermana" }, color) : 0;
  const maxOpponentManaScore = needDenial ? manaScore(opponentMana, color) : 0;
  const boardHash = exactBoardHash(board);
  let bestScore = 0;
  let bestOpponentManaScore = 0;
  for (const [location, item] of board.entries()) {
    if (context.session.checkpoint()) return { bestScore: 0, bestOpponentManaScore: 0 };
    const mon = itemMon(item);
    if (mon?.color !== color || isMonFainted(mon)) continue;
    if (item.kind === "mon-with-mana") {
      const score = manaScore(item.mana, color);
      const relevantForScore = needScore && score >= scoreFloor;
      const relevantForDenial = needDenial && manaEquals(item.mana, opponentMana);
      if (!relevantForScore && !relevantForDenial) continue;
      const steps = exactCarrierStepsToAnyPoolWithHash(
        context,
        board,
        location,
        item.mana,
        boardHash,
        moveBudget,
      );
      if (context.session.checkpoint())
        return { bestScore: 0, bestOpponentManaScore: 0 };
      if (steps === undefined) continue;
      if (relevantForScore) bestScore = Math.max(bestScore, score);
      if (relevantForDenial) {
        bestOpponentManaScore = Math.max(bestOpponentManaScore, score);
      }
    } else if (
      mon.kind === MonKind.Drainer &&
      (item.kind === "mon" || item.kind === "mon-with-consumable")
    ) {
      const window = exactDrainerImmediateTacticalWindow(
        context,
        board,
        color,
        location,
        moveBudget,
        scoreFloor,
        needScore,
        needDenial,
      );
      if (context.session.checkpoint())
        return { bestScore: 0, bestOpponentManaScore: 0 };
      if (needScore) bestScore = Math.max(bestScore, window.bestScore);
      if (needDenial) {
        bestOpponentManaScore = Math.max(
          bestOpponentManaScore,
          window.bestOpponentManaScore,
        );
      }
    }
    if (
      (!needScore || bestScore >= maxScore) &&
      (!needDenial || bestOpponentManaScore >= maxOpponentManaScore)
    ) {
      return { bestScore, bestOpponentManaScore };
    }
  }
  return context.session.checkpoint()
    ? { bestScore: 0, bestOpponentManaScore: 0 }
    : { bestScore, bestOpponentManaScore };
}

type SpiritPreviewUndo = {
  readonly from: Location;
  readonly fromItem: Item | undefined;
  readonly to: Location;
  readonly toItem: Item | undefined;
};

export function applySpiritMovePreviewInPlace(
  board: MutableBoard,
  from: Location,
  targetItem: Item,
  to: Location,
  perspective: Color,
): {
  readonly undo: SpiritPreviewUndo;
  readonly scoreDelta: number;
  readonly opponentManaScoreDelta: number;
} {
  const fromItem = board.get(from);
  const destinationItem = board.get(to);
  const undo: SpiritPreviewUndo = {
    from,
    fromItem,
    to,
    toItem: destinationItem,
  };
  const destinationSquare = board.squareAt(to);
  board.delete(from);
  let placedItem = targetItem;
  if (targetItem.kind === "mon" && destinationItem?.kind === "mana") {
    placedItem = monWithManaItem(targetItem.mon, destinationItem.mana);
  } else if (targetItem.kind === "mana" && destinationItem?.kind === "mon") {
    placedItem = monWithManaItem(destinationItem.mon, targetItem.mana);
  } else if (targetItem.kind === "mon-with-mana" && destinationItem?.kind === "mana") {
    board.set(from, { kind: "mana", mana: targetItem.mana });
    placedItem = monWithManaItem(targetItem.mon, destinationItem.mana);
  } else if (targetItem.kind === "consumable" && destinationItem?.kind === "mon") {
    placedItem = monItem(destinationItem.mon);
  } else if (targetItem.kind === "mon" && destinationItem?.kind === "consumable") {
    placedItem = monItem(targetItem.mon);
  } else if (
    targetItem.kind === "mon-with-mana" &&
    destinationItem?.kind === "consumable"
  ) {
    placedItem = monWithManaItem(targetItem.mon, targetItem.mana);
  } else if (
    targetItem.kind === "mon-with-consumable" &&
    destinationItem?.kind === "consumable"
  ) {
    placedItem = {
      kind: "mon-with-consumable",
      mon: targetItem.mon,
      consumable: Consumable.Bomb,
    };
  }
  let scoreDelta = 0;
  let opponentManaScoreDelta = 0;
  const placedMana = itemMana(placedItem);
  if (destinationSquare.kind === "mana-pool" && placedMana !== undefined) {
    scoreDelta = manaScore(placedMana, perspective);
    if (placedMana.kind === "regular" && placedMana.color === otherColor(perspective)) {
      opponentManaScoreDelta = scoreDelta;
    }
    const placedMon = itemMon(placedItem);
    if (placedMon === undefined) {
      board.delete(to);
      return { undo, scoreDelta, opponentManaScoreDelta };
    }
    placedItem = monItem(placedMon);
  }
  board.set(to, placedItem);
  return { undo, scoreDelta, opponentManaScoreDelta };
}

export function undoSpiritMovePreview(
  board: MutableBoard,
  undo: SpiritPreviewUndo,
): void {
  if (undo.fromItem === undefined) board.delete(undo.from);
  else board.set(undo.from, undo.fromItem);
  if (undo.toItem === undefined) board.delete(undo.to);
  else board.set(undo.to, undo.toItem);
}
