import type { Board } from "../../engine/board/storage.js";
import { Color, MonKind, type Mon } from "../../api/types.js";
import {
  isMonFainted,
  isSpiritTargetAllowed,
  manaScore,
  otherColor,
} from "../../engine/model/domain.js";
import {
  BOARD_CENTER_INDEX,
  BOARD_SIZE,
  MAX_LOCATION_INDEX,
  locationDistance,
  locationEquals,
  spiritReachableLocations,
  type Location,
} from "../../engine/board/geometry.js";
import { MONS_MOVES_PER_TURN } from "../../engine/board/config.js";
import {
  isDrainerUnderImmediateThreat,
  isDrainerUnderWalkThreatWithHash,
} from "../exact/drainer-safety.js";
import type { ExactColorSummary, ExactSpiritSummary } from "../exact/types.js";
import { saturatingScoreAdd, saturatingScoreMultiply } from "../core/score-math.js";
import {
  colorSlot,
  type ManaPathSnapshot,
  type ScoringEvalContext,
} from "./context.js";
import type { ScoringWeights } from "./profile-schema.js";

type DrainerSafetySnapshot = {
  readonly riskDanger: number;
  readonly minMana: number;
  readonly angelNearby: boolean;
  readonly exactDangerThreat: boolean;
  readonly walkThreat: boolean;
};

export function exactSafe(snapshot: DrainerSafetySnapshot): boolean {
  return !snapshot.exactDangerThreat && !snapshot.walkThreat;
}

export function guardedAgainstExactAttack(snapshot: DrainerSafetySnapshot): boolean {
  return snapshot.angelNearby && !snapshot.exactDangerThreat;
}

export function requireExactSummary(
  summary: ExactColorSummary | undefined,
): ExactColorSummary {
  if (summary === undefined) {
    throw new Error("exact strategic analysis should be available");
  }
  return summary;
}

export function exactSummaryForScoring(
  mySummary: ExactColorSummary,
  opponentSummary: ExactColorSummary,
  actorColor: Color,
  perspective: Color,
): ExactColorSummary {
  return actorColor === perspective ? mySummary : opponentSummary;
}

export function spiritOnOwnBasePenalty(
  board: Board,
  mon: Mon,
  location: Location,
  penalty: number,
): number {
  return mon.kind === MonKind.Spirit &&
    !isMonFainted(mon) &&
    locationEquals(location, board.base(mon))
    ? penalty
    : 0;
}

export function exactSpiritPressureBonus(
  spirit: ExactSpiritSummary,
  weights: ScoringWeights,
): number {
  const setupGain = Math.min(4, Math.max(0, spirit.nextTurnSetupGain));
  let bonus = 0;
  if (setupGain > 0) {
    bonus = saturatingScoreAdd(
      bonus,
      Math.trunc(
        saturatingScoreMultiply(
          Math.max(0, weights.race.scoreRacePathProgress),
          setupGain,
        ) / 4,
      ),
    );
    bonus = saturatingScoreAdd(
      bonus,
      Math.trunc(
        saturatingScoreMultiply(
          Math.max(0, weights.race.opponentScoreRacePathProgress),
          setupGain,
        ) / 6,
      ),
    );
    bonus = saturatingScoreAdd(
      bonus,
      Math.trunc(
        saturatingScoreMultiply(
          Math.max(0, weights.race.scoreRaceMultiPath),
          setupGain,
        ) / 8,
      ),
    );
    bonus = saturatingScoreAdd(
      bonus,
      Math.trunc(
        saturatingScoreMultiply(
          Math.max(0, weights.race.opponentScoreRaceMultiPath),
          setupGain,
        ) / 10,
      ),
    );
  }
  if (spirit.supermanaProgress && !spirit.sameTurnScore) {
    bonus = saturatingScoreAdd(
      saturatingScoreAdd(
        bonus,
        saturatingScoreMultiply(Math.max(0, weights.mana.supermanaRaceControl), 3),
      ),
      Math.trunc(Math.max(0, weights.threat.drainerBestManaPath) / 4),
    );
  }
  if (spirit.opponentManaProgress && !spirit.sameTurnOpponentManaScore) {
    bonus = saturatingScoreAdd(
      saturatingScoreAdd(
        saturatingScoreAdd(
          bonus,
          saturatingScoreMultiply(Math.max(0, weights.mana.opponentManaDenial), 3),
        ),
        Math.trunc(Math.max(0, weights.threat.drainerBestManaPath) / 4),
      ),
      Math.trunc(Math.max(0, weights.race.scoreRacePathProgress) / 5),
    );
  }
  return bonus;
}

export function heuristicSpiritActionUtility(board: Board, location: Location): number {
  let utility = 0;
  for (const target of spiritReachableLocations(location)) {
    const item = board.get(target);
    if (item === undefined) continue;
    if (isSpiritTargetAllowed(item)) utility += 1;
  }
  return utility;
}

export function bestDrainerPickupPathWithSnapshot(
  snapshot: ManaPathSnapshot,
  color: Color,
  from: Location,
): readonly [number, number] | undefined {
  let best: readonly [number, number] | undefined;
  for (const candidate of snapshot.candidates) {
    const pickupSteps = locationDistance(from, candidate.location);
    const totalSteps = pickupSteps + candidate.scoreSteps;
    const manaValue = manaScore(candidate.mana, color);
    if (best === undefined) {
      best = [totalSteps, manaValue];
      continue;
    }
    const metric = totalSteps * 3 - manaValue;
    const bestMetric = best[0] * 3 - best[1];
    if (metric < bestMetric || (metric === bestMetric && manaValue > best[1])) {
      best = [totalSteps, manaValue];
    }
  }
  return best;
}

function drainerDistancesWithContext(
  color: Color,
  location: Location,
  useHeuristicFormula: boolean,
  context: ScoringEvalContext,
): readonly [number, number, boolean] {
  const summary = context.boardSummary();
  let minMana = BOARD_SIZE;
  let minDanger = BOARD_SIZE;
  for (const entry of summary.manaEntries) {
    minMana = Math.min(minMana, locationDistance(entry.location, location));
  }
  for (const danger of summary.dangerSources[colorSlot(otherColor(color))]) {
    if (useHeuristicFormula) {
      if (danger.heuristicThreat) {
        minDanger = Math.min(minDanger, locationDistance(danger.location, location));
      }
      continue;
    }
    let delta = 0x7fff_ffff;
    if (danger.exactActionThreat) {
      delta = locationDistance(danger.location, location);
    }
    if (danger.exactBombThreat) {
      const bombDelta = Math.max(1, locationDistance(danger.location, location) - 2);
      delta = Math.min(delta, bombDelta);
    }
    minDanger = Math.min(minDanger, delta);
  }
  if (useHeuristicFormula) {
    for (const consumable of summary.looseConsumableLocations) {
      minDanger = Math.min(minDanger, locationDistance(consumable, location));
    }
  }
  const angelNearby = summary.liveAngelLocations[colorSlot(color)].some(
    (angel) => locationDistance(angel, location) === 1,
  );
  return useHeuristicFormula
    ? [minDanger, minMana, angelNearby]
    : [Math.max(1, minDanger), Math.max(1, minMana), angelNearby];
}

export function drainerSafetySnapshotWithContext(
  color: Color,
  location: Location,
  useHeuristicFormula: boolean,
  includeWalkThreat: boolean,
  context: ScoringEvalContext,
): DrainerSafetySnapshot {
  const board = context.board;
  const [rawDanger, minMana, angelNearby] = drainerDistancesWithContext(
    color,
    location,
    useHeuristicFormula,
    context,
  );
  const exactDangerThreat = useHeuristicFormula
    ? isDrainerUnderImmediateThreat(
        context.execution,
        board,
        color,
        location,
        angelNearby,
      )
    : context.canAttackTargetOnBoard(
        otherColor(color),
        color,
        location,
        MONS_MOVES_PER_TURN,
        true,
      );
  const walkThreat =
    includeWalkThreat &&
    !exactDangerThreat &&
    isDrainerUnderWalkThreatWithHash(
      context.execution,
      board,
      context.boardHash,
      color,
      location,
      angelNearby,
    );
  return {
    riskDanger: useHeuristicFormula
      ? Math.max(1, rawDanger)
      : exactDangerThreat
        ? 1
        : Math.max(1, rawDanger),
    minMana,
    angelNearby,
    exactDangerThreat,
    walkThreat,
  };
}

export function drainerImmediateThreatsWithContext(
  color: Color,
  location: Location,
  context: ScoringEvalContext,
): readonly [number, number] {
  return context.drainerImmediateThreats(color, location);
}

export function nearestEnemyMonDistanceWithContext(
  color: Color,
  location: Location,
  context: ScoringEvalContext,
): number {
  let best = BOARD_SIZE;
  for (const occupied of context.boardSummary().liveMonLocations[
    colorSlot(otherColor(color))
  ]) {
    best = Math.min(best, locationDistance(occupied, location));
  }
  return Math.max(1, best);
}

export function nearestFriendlyDrainerDistanceWithContext(
  color: Color,
  location: Location,
  context: ScoringEvalContext,
): number {
  let best = BOARD_SIZE;
  for (const occupied of context.boardSummary().liveDrainerLocations[
    colorSlot(color)
  ]) {
    best = Math.min(best, locationDistance(occupied, location));
  }
  return Math.max(1, best);
}

export function distanceToLocation(location: Location, destination: Location): number {
  return locationDistance(location, destination) + 1;
}

export function distanceToCenter(location: Location): number {
  return Math.max(1, Math.abs(BOARD_CENTER_INDEX - location.i)) + 1;
}

export function distanceToAnyClosestPool(location: Location): number {
  return (
    Math.max(
      Math.min(location.i, Math.abs(MAX_LOCATION_INDEX - location.i)),
      Math.min(location.j, Math.abs(MAX_LOCATION_INDEX - location.j)),
    ) + 1
  );
}

export function distanceToClosestPool(location: Location, color: Color): number {
  const poolRow = color === Color.White ? MAX_LOCATION_INDEX : 0;
  return (
    Math.max(
      Math.abs(poolRow - location.i),
      Math.min(location.j, Math.abs(MAX_LOCATION_INDEX - location.j)),
    ) + 1
  );
}
