import { Board } from "../../engine/board/storage.js";
import { Color, Consumable, MonKind } from "../../api/types.js";
import {
  isMonFainted,
  isSpiritTargetAllowed,
  type Item,
  itemConsumable,
  itemMana,
  itemMon,
  otherColor,
} from "../../engine/model/domain.js";
import {
  BOARD_CELLS,
  type Location,
  locationIndex,
  nearbyLocations,
  spiritReachableLocations,
} from "../../engine/board/geometry.js";
import { spiritDestinationItemAllowed } from "../../engine/rules/legality.js";
import { colorKey, exactCaches, exactCacheTag } from "./cache.js";
import { exactBoardHash } from "./hash.js";
import { defaultSpiritSummary, type ExactSpiritSummary } from "./types.js";
import type { AutomoveExecutionContext } from "../core/execution-context.js";

const EXACT_SPIRIT_UTILITY_CAP = 6;

export function reachableSpiritPositions(
  context: AutomoveExecutionContext,
  board: Board,
  start: Location,
  color: Color,
  remainingMonMoves: number,
): readonly (readonly [Location, number])[] {
  if (remainingMonMoves < 0 || context.session.checkpoint()) return [];
  const boardHash = exactBoardHash(board);
  const tag = exactCacheTag(locationIndex(start), colorKey(color), remainingMonMoves);
  const cached =
    tag === undefined
      ? undefined
      : exactCaches(context).spiritReach.get(boardHash, tag);
  if (cached !== undefined) return cached;
  const positions: (readonly [Location, number])[] = [];
  const queue: (readonly [Location, number])[] = [[start, 0]];
  const seen = new Uint8Array(BOARD_CELLS);
  seen[locationIndex(start)] = 1;
  let cursor = 0;
  while (cursor < queue.length) {
    const current = queue[cursor];
    cursor += 1;
    if (current === undefined || context.session.checkpoint()) return [];
    const [location, steps] = current;
    positions.push([location, steps]);
    if (steps >= remainingMonMoves) continue;
    for (const next of nearbyLocations(location)) {
      if (context.session.checkpoint()) return [];
      const index = locationIndex(next);
      if (seen[index] !== 0) continue;
      const item = board.get(next);
      const square = board.squareAt(next);
      let passable = false;
      if (item === undefined) {
        passable =
          square.kind === "regular" ||
          square.kind === "consumable-base" ||
          square.kind === "mana-base" ||
          square.kind === "mana-pool" ||
          (square.kind === "mon-base" &&
            square.monKind === MonKind.Spirit &&
            square.color === color);
      } else {
        passable =
          item.kind === "consumable" && item.consumable === Consumable.BombOrPotion;
      }
      if (!passable) continue;
      seen[index] = 1;
      queue.push([next, steps + 1]);
    }
  }
  if (!context.session.cacheWriteAllowed) return [];
  const result = Object.freeze(
    positions.map(([at, steps]) => Object.freeze([at, steps] as const)),
  );
  if (tag !== undefined) exactCaches(context).spiritReach.set(boardHash, result, tag);
  return result;
}

export function spiritDestinationAllowed(
  board: Board,
  targetItem: Item,
  destination: Location,
): boolean {
  const destinationItem = board.get(destination);
  const targetMon = itemMon(targetItem);
  const targetMana = itemMana(targetItem);
  if (!spiritDestinationItemAllowed(targetItem, destinationItem)) return false;
  const square = board.squareAt(destination);
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

export function exactPassiveSpiritSummary(
  context: AutomoveExecutionContext,
  board: Board,
  color: Color,
  remainingMonMoves: number,
  canUseAction: boolean,
): ExactSpiritSummary {
  if (remainingMonMoves < 0 || !canUseAction || context.session.checkpoint()) {
    return defaultSpiritSummary();
  }
  let best = defaultSpiritSummary();
  for (const [location, item] of board.entries()) {
    if (context.session.checkpoint()) return defaultSpiritSummary();
    const mon = itemMon(item);
    if (mon?.color !== color || mon.kind !== MonKind.Spirit || isMonFainted(mon)) {
      continue;
    }
    for (const [spiritPosition] of reachableSpiritPositions(
      context,
      board,
      location,
      color,
      remainingMonMoves,
    )) {
      if (context.session.checkpoint()) return defaultSpiritSummary();
      if (board.squareAt(spiritPosition).kind === "mon-base") continue;
      let reachableTargets = 0;
      let setupGain = 0;
      let supermanaProgress = false;
      let opponentManaProgress = false;
      for (const target of spiritReachableLocations(spiritPosition)) {
        const targetItem = board.get(target);
        if (
          targetItem === undefined ||
          !isSpiritTargetAllowed(targetItem) ||
          !nearbyLocations(target).some((destination) =>
            spiritDestinationAllowed(board, targetItem, destination),
          )
        ) {
          continue;
        }
        reachableTargets += 1;
        if (targetItem.kind === "mana") {
          if (targetItem.mana.kind === "supermana") {
            supermanaProgress = true;
            setupGain = Math.max(setupGain, 2);
          } else if (targetItem.mana.color === otherColor(color)) {
            opponentManaProgress = true;
            setupGain = Math.max(setupGain, 2);
          }
        } else {
          const targetMon = itemMon(targetItem);
          if (
            targetMon?.color === color &&
            targetMon.kind === MonKind.Drainer &&
            !isMonFainted(targetMon)
          ) {
            setupGain = Math.max(setupGain, 2);
          } else if (
            targetMon !== undefined &&
            targetMon.color !== color &&
            !isMonFainted(targetMon)
          ) {
            setupGain = Math.max(setupGain, 1);
          }
        }
      }
      const utility = Math.max(
        Math.min(reachableTargets, EXACT_SPIRIT_UTILITY_CAP),
        Math.min(1 + setupGain, EXACT_SPIRIT_UTILITY_CAP),
      );
      best = {
        ...best,
        utility: Math.max(best.utility, utility),
        nextTurnSetupGain:
          utility > best.utility
            ? setupGain
            : utility === best.utility
              ? Math.max(best.nextTurnSetupGain, setupGain)
              : best.nextTurnSetupGain,
        supermanaProgress: best.supermanaProgress || supermanaProgress,
        opponentManaProgress: best.opponentManaProgress || opponentManaProgress,
      };
    }
  }
  return context.session.checkpoint() ? defaultSpiritSummary() : best;
}
