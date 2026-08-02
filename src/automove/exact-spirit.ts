import { Board, type MutableBoard } from "../engine/board.js";
import {
  Color,
  Consumable,
  isMonFainted,
  isSpiritTargetAllowed,
  type Item,
  itemConsumable,
  itemMana,
  itemMon,
  manaEquals,
  manaScore,
  monItem,
  MonKind,
  monWithManaItem,
  otherColor,
} from "../engine/domain.js";
import {
  BOARD_CELLS,
  type Location,
  locationEquals,
  locationIndex,
  nearbyLocations,
  spiritReachableLocations,
} from "../engine/geometry.js";
import { spiritDestinationItemAllowed } from "../engine/legality.js";
import {
  colorKey,
  EXACT_SPIRIT_SUMMARY_CACHE_MAX_ENTRIES,
  exactCaches,
  exactCacheTag,
} from "./exact-cache.js";
import { exactBoardHash } from "./exact-hash.js";
import {
  exactCarrierStepsToAnyPoolWithHash,
  exactSecureSpecificManaStepsOnBoard,
} from "./exact-mana.js";
import {
  actorPayloadAfterMoveCompute,
  type AttackQueueEntry,
  NO_PAYLOAD,
  payloadSeenSlot,
} from "./exact-path.js";
import {
  defaultSpiritSummary,
  type ExactSpiritSummary,
} from "./exact-types.js";
import { type AutomoveExecutionContext } from "./execution-context.js";
import { Hash64Table } from "./hash64.js";

const EXACT_SPIRIT_UTILITY_CAP = 6;

function reachableSpiritPositions(
  context: AutomoveExecutionContext,
  board: Board,
  start: Location,
  color: Color,
  remainingMonMoves: number,
): readonly (readonly [Location, number])[] {
  if (remainingMonMoves < 0 || context.session.checkpoint()) return [];
  const boardHash = exactBoardHash(board);
  const tag = exactCacheTag(
    locationIndex(start),
    colorKey(color),
    remainingMonMoves,
  );
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
          item.kind === "consumable" &&
          item.consumable === Consumable.BombOrPotion;
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
  if (tag !== undefined)
    exactCaches(context).spiritReach.set(boardHash, result, tag);
  return result;
}

function spiritDestinationAllowed(
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
    if (
      mon?.color !== color ||
      mon.kind !== MonKind.Spirit ||
      isMonFainted(mon)
    ) {
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

type ImmediateTacticalWindow = {
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

function exactBestImmediateTacticalWindow(
  context: AutomoveExecutionContext,
  board: Board,
  color: Color,
  moveBudget: number,
  needScore: boolean,
  needDenial: boolean,
  minScore = 1,
): ImmediateTacticalWindow {
  if (
    moveBudget < 0 ||
    (!needScore && !needDenial) ||
    context.session.checkpoint()
  ) {
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
    if (context.session.checkpoint())
      return { bestScore: 0, bestOpponentManaScore: 0 };
    const mon = itemMon(item);
    if (mon?.color !== color || isMonFainted(mon)) continue;
    if (item.kind === "mon-with-mana") {
      const score = manaScore(item.mana, color);
      const relevantForScore = needScore && score >= scoreFloor;
      const relevantForDenial =
        needDenial && manaEquals(item.mana, opponentMana);
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

function applySpiritMovePreviewInPlace(
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
  } else if (
    targetItem.kind === "mon-with-mana" &&
    destinationItem?.kind === "mana"
  ) {
    board.set(from, { kind: "mana", mana: targetItem.mana });
    placedItem = monWithManaItem(targetItem.mon, destinationItem.mana);
  } else if (
    targetItem.kind === "consumable" &&
    destinationItem?.kind === "mon"
  ) {
    placedItem = monItem(destinationItem.mon);
  } else if (
    targetItem.kind === "mon" &&
    destinationItem?.kind === "consumable"
  ) {
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
    if (
      placedMana.kind === "regular" &&
      placedMana.color === otherColor(perspective)
    ) {
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

function undoSpiritMovePreview(
  board: MutableBoard,
  undo: SpiritPreviewUndo,
): void {
  if (undo.fromItem === undefined) board.delete(undo.from);
  else board.set(undo.from, undo.fromItem);
  if (undo.toItem === undefined) board.delete(undo.to);
  else board.set(undo.to, undo.toItem);
}

export const EXACT_TACTICAL_SPIRIT_NEED_SCORE = 1 << 0;
export const EXACT_TACTICAL_SPIRIT_NEED_DENIAL = 1 << 1;
export const EXACT_TACTICAL_SPIRIT_NEED_PROGRESS = 1 << 2;
const EXACT_TACTICAL_SPIRIT_ALL_FIELDS =
  EXACT_TACTICAL_SPIRIT_NEED_SCORE |
  EXACT_TACTICAL_SPIRIT_NEED_DENIAL |
  EXACT_TACTICAL_SPIRIT_NEED_PROGRESS;

function tacticalSpiritSummaryForFields(
  summary: ExactSpiritSummary,
  fields: number,
): ExactSpiritSummary {
  const needScore = (fields & EXACT_TACTICAL_SPIRIT_NEED_SCORE) !== 0;
  const needDenial = (fields & EXACT_TACTICAL_SPIRIT_NEED_DENIAL) !== 0;
  const needProgress = (fields & EXACT_TACTICAL_SPIRIT_NEED_PROGRESS) !== 0;
  return {
    ...defaultSpiritSummary(),
    sameTurnScore: needScore && summary.sameTurnScore,
    sameTurnScoreValue: needScore ? summary.sameTurnScoreValue : 0,
    sameTurnOpponentManaScore: needDenial && summary.sameTurnOpponentManaScore,
    sameTurnOpponentManaScoreValue: needDenial
      ? summary.sameTurnOpponentManaScoreValue
      : 0,
    supermanaProgress: needProgress && summary.supermanaProgress,
    opponentManaProgress: needProgress && summary.opponentManaProgress,
  };
}

export function exactTacticalSpiritSummary(
  context: AutomoveExecutionContext,
  board: Board,
  color: Color,
  remainingMonMoves: number,
  canUseAction: boolean,
  fields: number,
): ExactSpiritSummary {
  fields &= EXACT_TACTICAL_SPIRIT_ALL_FIELDS;
  if (
    remainingMonMoves < 0 ||
    fields === 0 ||
    !canUseAction ||
    context.session.checkpoint()
  ) {
    return defaultSpiritSummary();
  }
  const boardHash = exactBoardHash(board);
  const cacheTag = exactCacheTag(
    colorKey(color),
    remainingMonMoves,
    Number(canUseAction),
    fields,
  );
  const cached =
    cacheTag === undefined
      ? undefined
      : exactCaches(context).spiritTacticalSummary.get(boardHash, cacheTag);
  if (cached !== undefined) return cached;
  for (
    let supersetFields = 1;
    supersetFields <= EXACT_TACTICAL_SPIRIT_ALL_FIELDS;
    supersetFields += 1
  ) {
    if (supersetFields === fields || (supersetFields & fields) !== fields) {
      continue;
    }
    const supersetTag = exactCacheTag(
      colorKey(color),
      remainingMonMoves,
      Number(canUseAction),
      supersetFields,
    );
    const superset =
      supersetTag === undefined
        ? undefined
        : exactCaches(context).spiritTacticalSummary.get(
            boardHash,
            supersetTag,
          );
    if (superset === undefined) continue;
    const derived = tacticalSpiritSummaryForFields(superset, fields);
    if (!context.session.cacheWriteAllowed) return defaultSpiritSummary();
    if (cacheTag !== undefined) {
      exactCaches(context).spiritTacticalSummary.set(
        boardHash,
        derived,
        cacheTag,
      );
    }
    return derived;
  }
  const result = exactTacticalSpiritSummaryUncached(
    context,
    board,
    color,
    remainingMonMoves,
    canUseAction,
    fields,
  );
  if (!context.session.cacheWriteAllowed) return defaultSpiritSummary();
  if (cacheTag !== undefined) {
    exactCaches(context).spiritTacticalSummary.set(boardHash, result, cacheTag);
  }
  return result;
}

function exactTacticalSpiritSummaryUncached(
  context: AutomoveExecutionContext,
  board: Board,
  color: Color,
  remainingMonMoves: number,
  canUseAction: boolean,
  fields: number,
): ExactSpiritSummary {
  if (!canUseAction || context.session.checkpoint())
    return defaultSpiritSummary();
  const needScore = (fields & EXACT_TACTICAL_SPIRIT_NEED_SCORE) !== 0;
  const needDenial = (fields & EXACT_TACTICAL_SPIRIT_NEED_DENIAL) !== 0;
  const needProgress = (fields & EXACT_TACTICAL_SPIRIT_NEED_PROGRESS) !== 0;
  const before = exactBestImmediateTacticalWindow(
    context,
    board,
    color,
    remainingMonMoves,
    needScore,
    needDenial,
  );
  if (context.session.checkpoint()) return defaultSpiritSummary();
  const maxSameTurnScore = needScore
    ? manaScore({ kind: "supermana" }, color)
    : 0;
  const maxSameTurnOpponentScore = needDenial
    ? manaScore({ kind: "regular", color: otherColor(color) }, color)
    : 0;
  let best = defaultSpiritSummary();
  const afterWindowCache = new Hash64Table<ImmediateTacticalWindow>(
    EXACT_SPIRIT_SUMMARY_CACHE_MAX_ENTRIES,
  );
  for (const [location, spiritItem] of board.entries()) {
    if (context.session.checkpoint()) return defaultSpiritSummary();
    const spirit = itemMon(spiritItem);
    if (
      spirit?.color !== color ||
      spirit.kind !== MonKind.Spirit ||
      isMonFainted(spirit)
    ) {
      continue;
    }
    for (const [spiritPosition, spiritSteps] of reachableSpiritPositions(
      context,
      board,
      location,
      color,
      remainingMonMoves,
    )) {
      if (context.session.checkpoint()) return defaultSpiritSummary();
      if (board.squareAt(spiritPosition).kind === "mon-base") continue;
      const actionBoard = board.fork();
      if (!locationEquals(spiritPosition, location)) {
        actionBoard.delete(location);
        actionBoard.set(spiritPosition, spiritItem);
      }
      const remainingAfterAction = remainingMonMoves - spiritSteps;
      for (const target of spiritReachableLocations(spiritPosition)) {
        if (context.session.checkpoint()) return defaultSpiritSummary();
        const targetItem = actionBoard.get(target);
        if (targetItem === undefined || !isSpiritTargetAllowed(targetItem))
          continue;
        for (const destination of nearbyLocations(target)) {
          if (context.session.checkpoint()) return defaultSpiritSummary();
          if (!spiritDestinationAllowed(actionBoard, targetItem, destination)) {
            continue;
          }
          const preview = applySpiritMovePreviewInPlace(
            actionBoard,
            target,
            targetItem,
            destination,
            color,
          );
          try {
            if (context.session.checkpoint()) return defaultSpiritSummary();
            const scoreFloor = Math.max(
              best.sameTurnScoreValue,
              before.bestScore,
              preview.scoreDelta,
            );
            const denialFloor = Math.max(
              best.sameTurnOpponentManaScoreValue,
              before.bestOpponentManaScore,
              preview.opponentManaScoreDelta,
            );
            const needAfterScore = needScore && scoreFloor < maxSameTurnScore;
            const needAfterDenial =
              needDenial && denialFloor < maxSameTurnOpponentScore;
            let after: ImmediateTacticalWindow = {
              bestScore: 0,
              bestOpponentManaScore: 0,
            };
            if (needAfterScore || needAfterDenial) {
              const minScore = needAfterScore ? scoreFloor + 1 : 1;
              const afterHash = exactBoardHash(actionBoard);
              const afterTag = exactCacheTag(
                colorKey(color),
                remainingAfterAction,
                minScore,
                Number(needAfterScore),
                Number(needAfterDenial),
              );
              const cachedAfter =
                afterTag === undefined
                  ? undefined
                  : afterWindowCache.get(afterHash, afterTag);
              if (cachedAfter !== undefined) {
                after = cachedAfter;
              } else {
                after = exactBestImmediateTacticalWindow(
                  context,
                  actionBoard,
                  color,
                  remainingAfterAction,
                  needAfterScore,
                  needAfterDenial,
                  minScore,
                );
                if (context.session.checkpoint()) return defaultSpiritSummary();
                if (afterTag !== undefined) {
                  afterWindowCache.set(afterHash, after, afterTag);
                }
              }
            }
            if (context.session.checkpoint()) return defaultSpiritSummary();
            const afterScore = Math.max(preview.scoreDelta, after.bestScore);
            const afterDenial = Math.max(
              preview.opponentManaScoreDelta,
              after.bestOpponentManaScore,
            );
            if (
              needScore &&
              (preview.scoreDelta > 0 || afterScore > before.bestScore)
            ) {
              best = {
                ...best,
                sameTurnScore: true,
                sameTurnScoreValue: Math.max(
                  best.sameTurnScoreValue,
                  afterScore,
                ),
              };
            }
            if (
              needDenial &&
              (preview.opponentManaScoreDelta > 0 ||
                afterDenial > before.bestOpponentManaScore)
            ) {
              best = {
                ...best,
                sameTurnOpponentManaScore: true,
                sameTurnOpponentManaScoreValue: Math.max(
                  best.sameTurnOpponentManaScoreValue,
                  afterDenial,
                ),
              };
            }
            if (needProgress && !best.supermanaProgress) {
              const movedSupermana =
                targetItem.kind === "mana" &&
                targetItem.mana.kind === "supermana" &&
                preview.scoreDelta > 0;
              let hasSupermanaProgress = movedSupermana;
              if (!hasSupermanaProgress) {
                hasSupermanaProgress =
                  exactSecureSpecificManaStepsOnBoard(
                    context,
                    actionBoard,
                    color,
                    { kind: "supermana" },
                    remainingAfterAction,
                  ) !== undefined;
                if (context.session.checkpoint()) return defaultSpiritSummary();
              }
              if (hasSupermanaProgress) {
                best = { ...best, supermanaProgress: true };
              }
            }
            if (needProgress && !best.opponentManaProgress) {
              let hasOpponentManaProgress = preview.opponentManaScoreDelta > 0;
              if (!hasOpponentManaProgress) {
                hasOpponentManaProgress =
                  exactSecureSpecificManaStepsOnBoard(
                    context,
                    actionBoard,
                    color,
                    { kind: "regular", color: otherColor(color) },
                    remainingAfterAction,
                  ) !== undefined;
                if (context.session.checkpoint()) return defaultSpiritSummary();
              }
              if (hasOpponentManaProgress) {
                best = { ...best, opponentManaProgress: true };
              }
            }
            if (context.session.checkpoint()) return defaultSpiritSummary();
            if (
              (!needScore || best.sameTurnScoreValue >= maxSameTurnScore) &&
              (!needDenial ||
                best.sameTurnOpponentManaScoreValue >=
                  maxSameTurnOpponentScore) &&
              (!needProgress ||
                (best.supermanaProgress && best.opponentManaProgress))
            ) {
              return best;
            }
          } finally {
            undoSpiritMovePreview(actionBoard, preview.undo);
          }
        }
      }
    }
  }
  return context.session.checkpoint() ? defaultSpiritSummary() : best;
}
