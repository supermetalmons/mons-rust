import { boardEquals } from "../engine/board.js";
import { BOARD_SIZE, MONS_MOVES_PER_TURN } from "../engine/config.js";
import {
  Color,
  MonKind,
  colorId,
  isMonFainted,
  itemMon,
  manaScore,
  otherColor,
  type Event,
} from "../engine/domain.js";
import { MonsGame } from "../engine/game.js";
import {
  locationDistance,
  locationEquals,
  type Location,
} from "../engine/geometry.js";
import {
  EXACT_TURN_TACTICAL_ALL_FLAGS,
  exactStrategicAnalysis,
  exactTurnTacticalProjectionWithSearchHash,
  type ExactTurnTacticalProjection,
} from "./exact.js";
import type { AutomoveExecutionContext } from "./execution-context.js";
import { Hash64Table, type Hash64 } from "./hash64.js";
import { hasMaterialEvent } from "./transitions.js";

const MOVE_EFFICIENCY_SNAPSHOT_CACHE_MAX_ENTRIES = 16_384;
const UNKNOWN_STEPS = BOARD_SIZE + 4;
const NO_EFFECT_ROOT_PENALTY = 120;
const LOW_IMPACT_ROOT_PENALTY = 40;
const SPIRIT_DEPLOY_EFFICIENCY_BONUS = 90;
const SPIRIT_ACTION_TARGET_DELTA_WEIGHT = 22;

/** Value snapshot used by both root and child move ordering. */
export type MoveEfficiencySnapshot = {
  readonly myBestCarrierSteps: number;
  readonly opponentBestCarrierSteps: number;
  readonly myBestDrainerToManaSteps: number;
  readonly opponentBestDrainerToManaSteps: number;
  readonly myCarrierCount: number;
  readonly opponentCarrierCount: number;
  readonly mySpiritOnBase: boolean;
  readonly opponentSpiritOnBase: boolean;
  readonly mySpiritActionTargets: number;
  readonly opponentSpiritActionTargets: number;
  readonly mySameTurnScoreValue: number;
  readonly opponentSameTurnScoreValue: number;
  readonly mySameTurnOpponentManaScoreValue: number;
  readonly opponentSameTurnOpponentManaScoreValue: number;
  readonly mySafeSupermanaProgress: boolean;
  readonly opponentSafeSupermanaProgress: boolean;
  readonly mySafeOpponentManaProgress: boolean;
  readonly opponentSafeOpponentManaProgress: boolean;
  readonly mySafeSupermanaProgressSteps: number;
  readonly opponentSafeSupermanaProgressSteps: number;
  readonly mySafeOpponentManaProgressSteps: number;
  readonly opponentSafeOpponentManaProgressSteps: number;
};

type MoveEfficiencyDeltaPolicy = {
  readonly isRoot: boolean;
  readonly applyBacktrackPenalty: boolean;
  readonly applyRootManaHandoffGuard: boolean;
  readonly rootBacktrackPenalty: number;
  readonly rootManaHandoffPenalty: number;
};

type MoveEfficiencyDeltaOptions = MoveEfficiencyDeltaPolicy & {
  readonly includeTacticalExact: boolean;
  readonly includeStrategicExact: boolean;
};

const MOVE_EFFICIENCY_CACHE = Symbol("move-efficiency-cache");

function moveEfficiencyCache(
  context: AutomoveExecutionContext,
): Hash64Table<MoveEfficiencySnapshot> {
  return context.caches.session.getOrCreate(
    MOVE_EFFICIENCY_CACHE,
    () =>
      new Hash64Table<MoveEfficiencySnapshot>(
        MOVE_EFFICIENCY_SNAPSHOT_CACHE_MAX_ENTRIES,
      ),
  );
}

function snapshotCacheTag(
  perspective: Color,
  includeTacticalExact: boolean,
  includeStrategicExact: boolean,
): number {
  return (
    colorId(perspective) |
    (Number(includeTacticalExact) << 1) |
    (Number(includeStrategicExact) << 2)
  );
}

function defaultTacticalProjection(
  sameTurnScoreWindowValue: number,
): ExactTurnTacticalProjection {
  return {
    safeSupermanaProgress: false,
    safeSupermanaProgressSteps: undefined,
    safeOpponentManaProgress: false,
    safeOpponentManaProgressSteps: undefined,
    spiritAssistedScore: false,
    spiritAssistedScoreValue: 0,
    spiritAssistedDenial: false,
    spiritAssistedDenialValue: 0,
    sameTurnScoreWindowValue,
  };
}

export function distanceToAnyPoolStepsForEfficiency(
  location: Location,
): number {
  const maxIndex = BOARD_SIZE - 1;
  return (
    Math.max(
      Math.min(location.i, maxIndex - location.i),
      Math.min(location.j, maxIndex - location.j),
    ) + 1
  );
}

export function distanceToColorPoolStepsForEfficiency(
  location: Location,
  color: Color,
): number {
  const maxIndex = BOARD_SIZE - 1;
  const poolRow = color === Color.White ? maxIndex : 0;
  return (
    Math.max(
      Math.abs(poolRow - location.i),
      Math.min(location.j, maxIndex - location.j),
    ) + 1
  );
}

function approximateBestDrainerToManaSteps(
  game: MonsGame,
  color: Color,
): number | undefined {
  let bestSteps: number | undefined;
  for (const [drainerLocation, item] of game.board.entries()) {
    const mon = itemMon(item);
    if (
      mon?.color !== color ||
      mon.kind !== MonKind.Drainer ||
      isMonFainted(mon)
    ) {
      continue;
    }

    for (const [manaLocation, manaItem] of game.board.entries()) {
      if (manaItem.kind !== "mana") continue;
      const candidateSteps =
        locationDistance(drainerLocation, manaLocation) - 1;
      bestSteps =
        bestSteps === undefined
          ? candidateSteps
          : Math.min(bestSteps, candidateSteps);
    }
  }
  return bestSteps;
}

export function approximateSameTurnScoreWindowValue(
  game: MonsGame,
  color: Color,
): number {
  if (game.activeColor !== color) return 0;
  const remainingMoves = Math.max(0, MONS_MOVES_PER_TURN - game.monsMovesCount);
  let best = 0;
  for (const [location, item] of game.board.entries()) {
    if (
      item.kind !== "mon-with-mana" ||
      item.mon.color !== color ||
      isMonFainted(item.mon)
    ) {
      continue;
    }
    const poolSteps = distanceToAnyPoolStepsForEfficiency(location) - 1;
    if (poolSteps <= remainingMoves) {
      best = Math.max(best, manaScore(item.mana, color));
    }
  }
  return best;
}

type MoveEfficiencySnapshotBuilder = {
  -readonly [Key in keyof MoveEfficiencySnapshot]: MoveEfficiencySnapshot[Key];
};

function initializeMoveEfficiencySnapshot(
  context: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  opponent: Color,
  includeTacticalExact: boolean,
  includeStrategicExact: boolean,
  stateHash: Hash64,
): MoveEfficiencySnapshotBuilder {
  const strategic = includeStrategicExact
    ? exactStrategicAnalysis(context, game)
    : undefined;
  const mySummary = strategic?.colorSummary(perspective);
  const opponentSummary = strategic?.colorSummary(opponent);
  const tacticalFlags = EXACT_TURN_TACTICAL_ALL_FLAGS;
  const myTurnSummary =
    includeTacticalExact && game.activeColor === perspective
      ? exactTurnTacticalProjectionWithSearchHash(
          context,
          game,
          perspective,
          stateHash,
          tacticalFlags,
        )
      : defaultTacticalProjection(
          mySummary?.immediateWindow.bestScore ??
            approximateSameTurnScoreWindowValue(game, perspective),
        );
  const opponentTurnSummary =
    includeTacticalExact && game.activeColor === opponent
      ? exactTurnTacticalProjectionWithSearchHash(
          context,
          game,
          opponent,
          stateHash,
          tacticalFlags,
        )
      : defaultTacticalProjection(
          opponentSummary?.immediateWindow.bestScore ??
            approximateSameTurnScoreWindowValue(game, opponent),
        );

  const myBestDrainerToManaSteps =
    mySummary?.bestDrainerToManaSteps ??
    approximateBestDrainerToManaSteps(game, perspective) ??
    UNKNOWN_STEPS;
  const opponentBestDrainerToManaSteps =
    opponentSummary?.bestDrainerToManaSteps ??
    approximateBestDrainerToManaSteps(game, opponent) ??
    UNKNOWN_STEPS;

  return {
    myBestCarrierSteps: mySummary?.bestCarrierSteps ?? UNKNOWN_STEPS,
    opponentBestCarrierSteps:
      opponentSummary?.bestCarrierSteps ?? UNKNOWN_STEPS,
    myBestDrainerToManaSteps,
    opponentBestDrainerToManaSteps,
    myCarrierCount: 0,
    opponentCarrierCount: 0,
    mySpiritOnBase: false,
    opponentSpiritOnBase: false,
    mySpiritActionTargets: mySummary?.spirit.utility ?? 0,
    opponentSpiritActionTargets: opponentSummary?.spirit.utility ?? 0,
    mySameTurnScoreValue:
      game.activeColor === perspective
        ? includeTacticalExact
          ? myTurnSummary.spiritAssistedScoreValue
          : myTurnSummary.sameTurnScoreWindowValue
        : 0,
    opponentSameTurnScoreValue:
      game.activeColor === opponent
        ? includeTacticalExact
          ? opponentTurnSummary.spiritAssistedScoreValue
          : opponentTurnSummary.sameTurnScoreWindowValue
        : 0,
    mySameTurnOpponentManaScoreValue:
      game.activeColor === perspective && includeTacticalExact
        ? myTurnSummary.spiritAssistedDenialValue
        : 0,
    opponentSameTurnOpponentManaScoreValue:
      game.activeColor === opponent && includeTacticalExact
        ? opponentTurnSummary.spiritAssistedDenialValue
        : 0,
    mySafeSupermanaProgress:
      includeTacticalExact && myTurnSummary.safeSupermanaProgress,
    opponentSafeSupermanaProgress:
      includeTacticalExact && opponentTurnSummary.safeSupermanaProgress,
    mySafeOpponentManaProgress:
      includeTacticalExact && myTurnSummary.safeOpponentManaProgress,
    opponentSafeOpponentManaProgress:
      includeTacticalExact && opponentTurnSummary.safeOpponentManaProgress,
    mySafeSupermanaProgressSteps:
      myTurnSummary.safeSupermanaProgressSteps ?? UNKNOWN_STEPS,
    opponentSafeSupermanaProgressSteps:
      opponentTurnSummary.safeSupermanaProgressSteps ?? UNKNOWN_STEPS,
    mySafeOpponentManaProgressSteps:
      myTurnSummary.safeOpponentManaProgressSteps ?? UNKNOWN_STEPS,
    opponentSafeOpponentManaProgressSteps:
      opponentTurnSummary.safeOpponentManaProgressSteps ?? UNKNOWN_STEPS,
  };
}

function observeMoveEfficiencyBoard(
  game: MonsGame,
  perspective: Color,
  opponent: Color,
  snapshot: MoveEfficiencySnapshotBuilder,
): void {
  const mySpiritBase = game.board.base({
    kind: MonKind.Spirit,
    color: perspective,
    cooldown: 0,
  });
  const opponentSpiritBase = game.board.base({
    kind: MonKind.Spirit,
    color: opponent,
    cooldown: 0,
  });

  for (const [location, item] of game.board.entries()) {
    if (item.kind === "mon-with-mana") {
      if (isMonFainted(item.mon)) continue;
      const poolSteps = distanceToAnyPoolStepsForEfficiency(location) - 1;
      if (item.mon.color === perspective) {
        snapshot.myCarrierCount = snapshot.myCarrierCount + 1;
        snapshot.myBestCarrierSteps = Math.min(
          snapshot.myBestCarrierSteps,
          poolSteps,
        );
      } else {
        snapshot.opponentCarrierCount = snapshot.opponentCarrierCount + 1;
        snapshot.opponentBestCarrierSteps = Math.min(
          snapshot.opponentBestCarrierSteps,
          poolSteps,
        );
      }
      continue;
    }
    if (item.kind !== "mon" && item.kind !== "mon-with-consumable") {
      continue;
    }
    if (isMonFainted(item.mon) || item.mon.kind !== MonKind.Spirit) continue;
    if (item.mon.color === perspective) {
      snapshot.mySpiritOnBase = locationEquals(location, mySpiritBase);
    } else {
      snapshot.opponentSpiritOnBase = locationEquals(
        location,
        opponentSpiritBase,
      );
    }
  }
}

function buildMoveEfficiencySnapshot(
  context: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  includeTacticalExact: boolean,
  includeStrategicExact: boolean,
  stateHash: Hash64,
): MoveEfficiencySnapshot {
  const opponent = otherColor(perspective);
  const snapshot = initializeMoveEfficiencySnapshot(
    context,
    game,
    perspective,
    opponent,
    includeTacticalExact,
    includeStrategicExact,
    stateHash,
  );
  observeMoveEfficiencyBoard(game, perspective, opponent, snapshot);
  return snapshot;
}

export function moveEfficiencySnapshotWithHash(
  context: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  includeTacticalExact: boolean,
  includeStrategicExact: boolean,
  stateHash: Hash64,
): MoveEfficiencySnapshot {
  const cache = moveEfficiencyCache(context);
  const tag = snapshotCacheTag(
    perspective,
    includeTacticalExact,
    includeStrategicExact,
  );
  const cached = cache.get(stateHash, tag);
  if (cached !== undefined) return cached;

  const snapshot = buildMoveEfficiencySnapshot(
    context,
    game,
    perspective,
    includeTacticalExact,
    includeStrategicExact,
    stateHash,
  );
  if (context.session.cacheWriteAllowed) {
    cache.set(stateHash, snapshot, tag);
  }
  return snapshot;
}

export function moveEfficiencySnapshotUncachedWithHash(
  context: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  includeTacticalExact: boolean,
  includeStrategicExact: boolean,
  stateHash: Hash64,
): MoveEfficiencySnapshot {
  return buildMoveEfficiencySnapshot(
    context,
    game,
    perspective,
    includeTacticalExact,
    includeStrategicExact,
    stateHash,
  );
}

export function clearMoveEfficiencyCache(
  context: AutomoveExecutionContext,
): void {
  moveEfficiencyCache(context).clear();
}

function stepProgressDelta(
  beforeSteps: number,
  afterSteps: number,
  forwardWeight: number,
  backwardWeight: number,
  unknownSteps = UNKNOWN_STEPS,
): number {
  const beforeKnown = beforeSteps < unknownSteps;
  const afterKnown = afterSteps < unknownSteps;
  if (beforeKnown && afterKnown) {
    const deltaSteps = beforeSteps - afterSteps;
    if (deltaSteps > 0) return deltaSteps * forwardWeight;
    if (deltaSteps < 0) return deltaSteps * backwardWeight;
    return 0;
  }
  if (!beforeKnown && afterKnown) return forwardWeight;
  if (beforeKnown && !afterKnown) return 0 - backwardWeight;
  return 0;
}

export function hasRoundtripMonMove(events: readonly Event[]): boolean {
  const seenMoves: {
    readonly from: Location;
    readonly to: Location;
    readonly color: Color;
    readonly kind: MonKind;
  }[] = [];
  for (const event of events) {
    if (event.kind !== "mon-move") continue;
    const mon = itemMon(event.item);
    if (mon === undefined) continue;
    if (
      seenMoves.some(
        (move) =>
          locationEquals(move.from, event.to) &&
          locationEquals(move.to, event.from) &&
          move.color === mon.color &&
          move.kind === mon.kind,
      )
    ) {
      return true;
    }
    seenMoves.push({
      from: event.from,
      to: event.to,
      color: mon.color,
      kind: mon.kind,
    });
  }
  return false;
}

function isNoEffectTurnTransition(
  game: MonsGame,
  simulatedGame: MonsGame,
  events: readonly Event[],
): boolean {
  return (
    boardEquals(game.board, simulatedGame.board) &&
    game.whiteScore === simulatedGame.whiteScore &&
    game.blackScore === simulatedGame.blackScore &&
    game.whitePotionsCount === simulatedGame.whitePotionsCount &&
    game.blackPotionsCount === simulatedGame.blackPotionsCount &&
    !hasMaterialEvent(events)
  );
}

function eventsIncludeOpponentDrainerFainted(
  events: readonly Event[],
  perspective: Color,
): boolean {
  const opponent = otherColor(perspective);
  return events.some(
    (event) =>
      event.kind === "mon-fainted" &&
      event.mon.kind === MonKind.Drainer &&
      event.mon.color === opponent,
  );
}

export function manaHandoffPenalty(
  events: readonly Event[],
  perspective: Color,
  perStepPenalty: number,
): number {
  if (perStepPenalty <= 0) return 0;
  let penalty = 0;
  const opponent = otherColor(perspective);
  for (const event of events) {
    if (event.kind !== "mana-move") continue;
    const myBefore = distanceToColorPoolStepsForEfficiency(
      event.from,
      perspective,
    );
    const myAfter = distanceToColorPoolStepsForEfficiency(
      event.to,
      perspective,
    );
    const opponentBefore = distanceToColorPoolStepsForEfficiency(
      event.from,
      opponent,
    );
    const opponentAfter = distanceToColorPoolStepsForEfficiency(
      event.to,
      opponent,
    );
    const movedTowardOpponent = Math.max(opponentBefore - opponentAfter, 0);
    const movedTowardMe = Math.max(myBefore - myAfter, 0);
    if (movedTowardOpponent > movedTowardMe) {
      const excess = movedTowardOpponent - movedTowardMe;
      penalty =
        penalty + excess * manaScore(event.mana, opponent) * perStepPenalty;
    }
  }
  return penalty;
}

function applyCarrierAndDrainerEfficiencyDelta(
  before: MoveEfficiencySnapshot,
  after: MoveEfficiencySnapshot,
  initialDelta: number,
): number {
  let delta = initialDelta;
  delta =
    delta +
    stepProgressDelta(
      before.myBestCarrierSteps,
      after.myBestCarrierSteps,
      90,
      130,
    );
  delta =
    delta -
    stepProgressDelta(
      before.opponentBestCarrierSteps,
      after.opponentBestCarrierSteps,
      80,
      120,
    );
  delta =
    delta +
    stepProgressDelta(
      before.myBestDrainerToManaSteps,
      after.myBestDrainerToManaSteps,
      34,
      50,
    );
  delta =
    delta -
    stepProgressDelta(
      before.opponentBestDrainerToManaSteps,
      after.opponentBestDrainerToManaSteps,
      30,
      44,
    );
  delta = delta + (after.myCarrierCount - before.myCarrierCount) * 55;
  delta =
    delta - (after.opponentCarrierCount - before.opponentCarrierCount) * 48;
  return delta;
}

function applySpiritEfficiencyDelta(
  before: MoveEfficiencySnapshot,
  after: MoveEfficiencySnapshot,
  initialDelta: number,
): number {
  let delta = initialDelta;
  if (before.mySpiritOnBase && !after.mySpiritOnBase) {
    delta = delta + SPIRIT_DEPLOY_EFFICIENCY_BONUS;
  }
  if (!before.opponentSpiritOnBase && after.opponentSpiritOnBase) {
    delta = delta + Math.trunc(SPIRIT_DEPLOY_EFFICIENCY_BONUS / 3);
  }
  delta =
    delta +
    (after.mySpiritActionTargets - before.mySpiritActionTargets) *
      SPIRIT_ACTION_TARGET_DELTA_WEIGHT;
  delta =
    delta -
    (after.opponentSpiritActionTargets - before.opponentSpiritActionTargets) *
      Math.trunc(SPIRIT_ACTION_TARGET_DELTA_WEIGHT / 2);
  return delta;
}

function applyScoreWindowEfficiencyDelta(
  before: MoveEfficiencySnapshot,
  after: MoveEfficiencySnapshot,
  initialDelta: number,
): number {
  let delta = initialDelta;
  delta =
    delta + (after.mySameTurnScoreValue - before.mySameTurnScoreValue) * 55;
  delta =
    delta -
    (after.opponentSameTurnScoreValue - before.opponentSameTurnScoreValue) * 45;
  delta =
    delta +
    (after.mySameTurnOpponentManaScoreValue -
      before.mySameTurnOpponentManaScoreValue) *
      90;
  delta =
    delta -
    (after.opponentSameTurnOpponentManaScoreValue -
      before.opponentSameTurnOpponentManaScoreValue) *
      75;
  return delta;
}

function applySafeProgressEfficiencyDelta(
  before: MoveEfficiencySnapshot,
  after: MoveEfficiencySnapshot,
  initialDelta: number,
): number {
  let delta = initialDelta;
  if (!before.mySafeSupermanaProgress && after.mySafeSupermanaProgress) {
    delta = delta + 140;
  }
  if (
    !before.opponentSafeSupermanaProgress &&
    after.opponentSafeSupermanaProgress
  ) {
    delta = delta - 120;
  }
  if (!before.mySafeOpponentManaProgress && after.mySafeOpponentManaProgress) {
    delta = delta + 120;
  }
  if (
    !before.opponentSafeOpponentManaProgress &&
    after.opponentSafeOpponentManaProgress
  ) {
    delta = delta - 110;
  }
  delta =
    delta +
    stepProgressDelta(
      before.mySafeSupermanaProgressSteps,
      after.mySafeSupermanaProgressSteps,
      26,
      40,
    );
  delta =
    delta -
    stepProgressDelta(
      before.opponentSafeSupermanaProgressSteps,
      after.opponentSafeSupermanaProgressSteps,
      22,
      36,
    );
  delta =
    delta +
    stepProgressDelta(
      before.mySafeOpponentManaProgressSteps,
      after.mySafeOpponentManaProgressSteps,
      22,
      34,
    );
  delta =
    delta -
    stepProgressDelta(
      before.opponentSafeOpponentManaProgressSteps,
      after.opponentSafeOpponentManaProgressSteps,
      18,
      30,
    );
  return delta;
}

function applyRootEfficiencyPenalties(
  game: MonsGame,
  simulatedGame: MonsGame,
  perspective: Color,
  events: readonly Event[],
  policy: MoveEfficiencyDeltaPolicy,
  initialDelta: number,
): number {
  let delta = initialDelta;
  const rootCompensatesHandoff =
    events.some((event) => event.kind === "mana-scored") ||
    eventsIncludeOpponentDrainerFainted(events, perspective);
  if (policy.applyRootManaHandoffGuard && !rootCompensatesHandoff) {
    delta =
      delta -
      manaHandoffPenalty(events, perspective, policy.rootManaHandoffPenalty);
  }
  if (isNoEffectTurnTransition(game, simulatedGame, events)) {
    delta = delta - NO_EFFECT_ROOT_PENALTY;
  } else if (!hasMaterialEvent(events) && delta <= 0) {
    delta = delta - LOW_IMPACT_ROOT_PENALTY;
  }
  if (
    policy.applyBacktrackPenalty &&
    policy.rootBacktrackPenalty > 0 &&
    hasRoundtripMonMove(events)
  ) {
    delta = delta - policy.rootBacktrackPenalty;
  }
  return delta;
}

/**
 * Complete weighted snapshot delta. Passing a before-snapshot captured for a
 * different perspective intentionally preserves the established child order.
 */
function moveEfficiencyDeltaFromBeforeSnapshotWithAfterSnapshot(
  game: MonsGame,
  simulatedGame: MonsGame,
  perspective: Color,
  events: readonly Event[],
  before: MoveEfficiencySnapshot,
  after: MoveEfficiencySnapshot,
  policy: MoveEfficiencyDeltaPolicy,
): number {
  let delta = 0;
  delta = applyCarrierAndDrainerEfficiencyDelta(before, after, delta);
  delta = applySpiritEfficiencyDelta(before, after, delta);
  delta = applyScoreWindowEfficiencyDelta(before, after, delta);
  delta = applySafeProgressEfficiencyDelta(before, after, delta);

  if (policy.isRoot) {
    delta = applyRootEfficiencyPenalties(
      game,
      simulatedGame,
      perspective,
      events,
      policy,
      delta,
    );
  }

  return delta;
}

export function moveEfficiencyDeltaFromBeforeSnapshot(
  context: AutomoveExecutionContext,
  game: MonsGame,
  simulatedGame: MonsGame,
  perspective: Color,
  events: readonly Event[],
  before: MoveEfficiencySnapshot,
  simulatedStateHash: Hash64,
  options: MoveEfficiencyDeltaOptions,
): number {
  const after = moveEfficiencySnapshotUncachedWithHash(
    context,
    simulatedGame,
    perspective,
    options.includeTacticalExact && simulatedGame.activeColor === perspective,
    options.includeStrategicExact,
    simulatedStateHash,
  );
  return moveEfficiencyDeltaFromBeforeSnapshotWithAfterSnapshot(
    game,
    simulatedGame,
    perspective,
    events,
    before,
    after,
    options,
  );
}
