import { Board } from "../engine/board.js";
import { ACTIONS_PER_TURN, MONS_MOVES_PER_TURN } from "../engine/config.js";
import {
  Color,
  isMonFainted,
  itemMon,
  type Mana,
  manaEquals,
  manaScore,
  MonKind,
} from "../engine/domain.js";
import { MonsGame } from "../engine/game.js";
import {
  BOARD_CELLS,
  type Location,
  locationDistance,
  locationIndex,
  nearbyLocations,
} from "../engine/geometry.js";
import {
  findAwakeDrainer,
  isDrainerExactlySafeNextTurnOnBoard,
} from "./exact-attack.js";
import {
  colorKey,
  EXACT_SECURE_MANA_CACHE_MAX_ENTRIES,
  exactCaches,
  exactCacheTag,
} from "./exact-cache.js";
import {
  exactBoardHash,
  exactHashManaBits,
  exactSearchStateHash,
} from "./exact-hash.js";
import {
  actorPayloadAfterMoveCompute,
  type AttackQueueEntry,
  NO_PAYLOAD,
  payloadSeenSlot,
  shortestPayloadState,
} from "./exact-path.js";
import { type ExactDrainerPickupPath } from "./exact-types.js";
import { type AutomoveExecutionContext } from "./execution-context.js";
import { type Hash64, Hash64Set } from "./hash64.js";

export function exactCarrierStepsToAnyPoolWithHash(
  context: AutomoveExecutionContext,
  board: Board,
  start: Location,
  mana: Mana,
  boardHash: Hash64,
  maxSteps?: number,
): number | undefined {
  if (
    context.session.checkpoint() ||
    (maxSteps !== undefined && maxSteps < 0)
  ) {
    return undefined;
  }
  const tag = exactCacheTag(locationIndex(start), exactHashManaBits(mana));
  if (
    tag !== undefined &&
    exactCaches(context).carrierSteps.has(boardHash, tag)
  ) {
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
  if (
    tag !== undefined &&
    exactCaches(context).drainerToMana.has(boardHash, tag)
  ) {
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
  if (tag !== undefined)
    exactCaches(context).drainerToMana.set(boardHash, result, tag);
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
    (candidateMetric === currentMetric &&
      candidate.manaValue > current.manaValue)
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
  const tag = exactCacheTag(
    colorKey(color),
    locationIndex(start),
    maxSteps ?? -1,
  );
  if (
    tag !== undefined &&
    exactCaches(context).pickupPath.has(boardHash, tag)
  ) {
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
  if (tag !== undefined)
    exactCaches(context).pickupPath.set(boardHash, best, tag);
  return best;
}

type SecureManaStateKey = {
  readonly hash: Hash64;
  readonly whiteRegularManaCount: number;
  readonly blackRegularManaCount: number;
};

function secureManaStateKey(game: MonsGame): SecureManaStateKey {
  let whiteRegularManaCount = 0;
  let blackRegularManaCount = 0;
  for (const [, item] of game.board.entries()) {
    if (item.kind !== "mana" || item.mana.kind !== "regular") continue;
    if (item.mana.color === Color.White) whiteRegularManaCount += 1;
    else blackRegularManaCount += 1;
  }
  return {
    hash: exactSearchStateHash(game),
    whiteRegularManaCount,
    blackRegularManaCount,
  };
}

function exactDistanceToWantedManaStepsLowerBound(
  board: Board,
  wanted: Mana,
  start: Location,
): number | undefined {
  let best: number | undefined;
  for (const [location, item] of board.entries()) {
    if (item.kind !== "mana" || !manaEquals(item.mana, wanted)) continue;
    const distance = locationDistance(start, location);
    best = best === undefined ? distance : Math.min(best, distance);
  }
  return best;
}

function applySecureDrainerWalk(
  context: AutomoveExecutionContext,
  game: MonsGame,
  from: Location,
  to: Location,
):
  | { readonly after: MonsGame; readonly scoredMana: Mana | undefined }
  | undefined {
  if (context.session.checkpoint()) return undefined;
  const after = game.fork();
  const output = after.processInput(
    [
      { kind: "location", location: from },
      { kind: "location", location: to },
    ],
    false,
    false,
  );
  if (output.kind !== "events") return undefined;
  const scored = output.events.find((event) => event.kind === "mana-scored");
  return {
    after,
    scoredMana: scored?.kind === "mana-scored" ? scored.mana : undefined,
  };
}

function secureSpecificManaQueryKey(
  game: MonsGame,
  color: Color,
  start: Location,
  wanted: Mana,
): { readonly hash: Hash64; readonly tag: number | undefined } {
  const state = secureManaStateKey(game);
  return {
    hash: state.hash,
    tag: exactCacheTag(
      state.whiteRegularManaCount,
      state.blackRegularManaCount,
      colorKey(color),
      exactHashManaBits(wanted),
      locationIndex(start),
    ),
  };
}

function secureSpecificManaStepsInGameUncached(
  context: AutomoveExecutionContext,
  game: MonsGame,
  color: Color,
  start: Location,
  wanted: Mana,
  visiting: Hash64Set,
): number | undefined {
  if (context.session.checkpoint()) return undefined;
  const startItem = game.board.get(start);
  const holdingWanted =
    startItem?.kind === "mon-with-mana" && manaEquals(startItem.mana, wanted);
  if (
    holdingWanted &&
    isDrainerExactlySafeNextTurnOnBoard(context, game.board, color, start)
  ) {
    return 0;
  }
  if (game.activeColor !== color || !game.playerCanMoveMon()) return undefined;
  const remainingMoves = Math.max(MONS_MOVES_PER_TURN - game.monsMovesCount, 0);
  if (!holdingWanted) {
    const lowerBound = exactDistanceToWantedManaStepsLowerBound(
      game.board,
      wanted,
      start,
    );
    if (lowerBound === undefined || lowerBound > remainingMoves) {
      return undefined;
    }
  }
  let best: number | undefined;
  for (const next of nearbyLocations(start)) {
    if (context.session.checkpoint()) return undefined;
    if (!holdingWanted) {
      const nextItem = game.board.get(next);
      const nextPicksWanted =
        nextItem?.kind === "mana" && manaEquals(nextItem.mana, wanted);
      if (!nextPicksWanted) {
        const lowerBound = exactDistanceToWantedManaStepsLowerBound(
          game.board,
          wanted,
          next,
        );
        if (lowerBound === undefined || lowerBound > remainingMoves - 1) {
          continue;
        }
      }
    }
    const transition = applySecureDrainerWalk(context, game, start, next);
    if (context.session.checkpoint()) return undefined;
    if (transition === undefined) continue;
    let candidate: number | undefined;
    if (
      transition.scoredMana !== undefined &&
      manaEquals(transition.scoredMana, wanted)
    ) {
      candidate = 1;
    } else {
      const nextStart = findAwakeDrainer(transition.after.board, color);
      if (nextStart === undefined) continue;
      const child = secureSpecificManaStepsInGame(
        context,
        transition.after,
        color,
        nextStart,
        wanted,
        visiting,
      );
      if (context.session.checkpoint()) return undefined;
      if (child !== undefined) {
        candidate = child + 1;
      }
    }
    if (candidate !== undefined && (best === undefined || candidate < best)) {
      best = candidate;
      if (candidate === 1) break;
    }
  }
  return best;
}

function secureSpecificManaStepsInGame(
  context: AutomoveExecutionContext,
  game: MonsGame,
  color: Color,
  start: Location,
  wanted: Mana,
  visiting: Hash64Set,
): number | undefined {
  if (context.session.checkpoint()) return undefined;
  const key = secureSpecificManaQueryKey(game, color, start, wanted);
  if (
    key.tag !== undefined &&
    exactCaches(context).secureMana.has(key.hash, key.tag)
  ) {
    return exactCaches(context).secureMana.get(key.hash, key.tag);
  }
  if (key.tag !== undefined && visiting.has(key.hash, key.tag)) {
    return undefined;
  }
  if (key.tag !== undefined) visiting.add(key.hash, key.tag);
  let result: number | undefined;
  try {
    result = secureSpecificManaStepsInGameUncached(
      context,
      game,
      color,
      start,
      wanted,
      visiting,
    );
  } finally {
    if (key.tag !== undefined) visiting.delete(key.hash, key.tag);
  }
  if (!context.session.cacheWriteAllowed) return undefined;
  if (key.tag !== undefined) {
    exactCaches(context).secureMana.set(key.hash, result, key.tag);
  }
  return result;
}

export function exactSecureSpecificManaStepsOnBoard(
  context: AutomoveExecutionContext,
  board: Board,
  color: Color,
  wanted: Mana,
  remainingMoves: number,
): number | undefined {
  if (remainingMoves < 0 || context.session.checkpoint()) return undefined;
  const drainerLocation = findAwakeDrainer(board, color);
  if (drainerLocation === undefined) return undefined;
  const startItem = board.get(drainerLocation);
  const holdingWanted =
    startItem?.kind === "mon-with-mana" && manaEquals(startItem.mana, wanted);
  if (!holdingWanted) {
    const lowerBound = exactDistanceToWantedManaStepsLowerBound(
      board,
      wanted,
      drainerLocation,
    );
    if (lowerBound === undefined || lowerBound > remainingMoves)
      return undefined;
  }
  const monsMovesCount = Math.max(
    0,
    Math.min(MONS_MOVES_PER_TURN, MONS_MOVES_PER_TURN - remainingMoves),
  );
  const simulation = MonsGame.newSimulationState({
    board: board.fork(),
    whiteScore: 0,
    blackScore: 0,
    activeColor: color,
    actionsUsedCount: ACTIONS_PER_TURN,
    manaMovesCount: 0,
    monsMovesCount,
    whitePotionsCount: 0,
    blackPotionsCount: 0,
    turnNumber: 2,
  });
  return secureSpecificManaStepsInGame(
    context,
    simulation,
    color,
    drainerLocation,
    wanted,
    new Hash64Set(EXACT_SECURE_MANA_CACHE_MAX_ENTRIES),
  );
}

export function exactSecureSpecificManaPathFrom(
  context: AutomoveExecutionContext,
  game: MonsGame,
  color: Color,
  start: Location,
  wanted: Mana,
): Location[] | undefined {
  if (context.session.checkpoint()) return undefined;
  let simulation = game.fork();
  let current = start;
  let remaining = secureSpecificManaStepsInGame(
    context,
    simulation,
    color,
    current,
    wanted,
    new Hash64Set(EXACT_SECURE_MANA_CACHE_MAX_ENTRIES),
  );
  if (remaining === undefined || context.session.checkpoint()) return undefined;
  const path: Location[] = [];
  while (remaining > 0) {
    let chosen:
      | {
          readonly next: Location;
          readonly after: MonsGame;
          readonly steps: number;
        }
      | undefined;
    for (const next of nearbyLocations(current)) {
      if (context.session.checkpoint()) return undefined;
      const transition = applySecureDrainerWalk(
        context,
        simulation,
        current,
        next,
      );
      if (context.session.checkpoint()) return undefined;
      if (transition === undefined) continue;
      if (
        remaining === 1 &&
        transition.scoredMana !== undefined &&
        manaEquals(transition.scoredMana, wanted)
      ) {
        chosen = { next, after: transition.after, steps: 0 };
        break;
      }
      const nextStart = findAwakeDrainer(transition.after.board, color);
      if (nextStart === undefined) continue;
      const child = secureSpecificManaStepsInGame(
        context,
        transition.after,
        color,
        nextStart,
        wanted,
        new Hash64Set(EXACT_SECURE_MANA_CACHE_MAX_ENTRIES),
      );
      if (context.session.checkpoint()) return undefined;
      if (child === remaining - 1) {
        chosen = { next, after: transition.after, steps: child };
        break;
      }
    }
    if (chosen === undefined) return undefined;
    path.push(chosen.next);
    simulation = chosen.after;
    current = chosen.next;
    remaining = chosen.steps;
  }
  return path;
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
    if (steps !== undefined)
      best = best === undefined ? steps : Math.min(best, steps);
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
