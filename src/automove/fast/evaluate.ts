import { BOARD_CELLS, BOARD_SIZE } from "../../engine/geometry.js";
import { MONS_MOVES_PER_TURN, TARGET_SCORE } from "../../engine/config.js";
import {
  CENTER_ROW_DISTANCE,
  CONS_BOMB,
  DEMON_TARGETS,
  KIND_ANGEL,
  KIND_DEMON,
  KIND_DRAINER,
  KIND_MYSTIC,
  KIND_SPIRIT,
  MANA_SUPER,
  COLOR_COUNT,
  MON_KIND_COUNT,
  MYSTIC_TARGETS,
  OCC_MANA,
  OCC_MON,
  OWN_POOL_DISTANCE,
  POOL_DISTANCE,
  SQ_MON_BASE,
  SQ_SUPERMANA_BASE,
  SUPERMANA_BASE_INDEX,
  cellConsumable,
  cellCooldown,
  cellMana,
  cellOccupancy,
  chebyshev,
  damagingAttackCanComplete,
  i32,
  manaScoreValue,
  midpoint,
  monId,
  u16,
  u8,
} from "./board.js";
import { awakeAngelGuards, type FastPosition } from "./position.js";

export const WIN_VALUE = 1_000_000;

const MAX_ABSOLUTE_EVAL_WEIGHT = 1_000_000;

const EVAL_WEIGHT_KEYS = [
  "scoreUnit",
  "potion",
  "bomb",
  "activeMon",
  "faintMon",
  "faintDrainer",
  "faintCooldownStep",
  "drainerCloseToMana",
  "drainerCloseToOwnPool",
  "drainerCloseToSupermana",
  "carrierCloseToPool",
  "carrierPointBonus",
  "carrierScoresThisTurn",
  "carrierScoresNextTurn",
  "winningCarrier",
  "drainerPickupScoresThisTurn",
  "manaToOwnerPool",
  "manaPointsAttraction",
  "manaDrainerControl",
  "supermanaDrainerControl",
  "monCloseToCenter",
  "spiritCloseToEnemy",
  "spiritOnOwnBase",
  "angelCloseToDrainer",
  "angelGuardingDrainer",
  "attackerCloseToEnemyDrainer",
  "drainerThreatImmediate",
  "drainerThreatWalk",
  "carrierThreatFactor",
  "manaToNearestPool",
] as const;

export type EvalWeights = Readonly<
  Record<(typeof EVAL_WEIGHT_KEYS)[number], number>
>;

export function normalizeEvalWeights(weights: unknown): EvalWeights {
  if (
    typeof weights !== "object" ||
    weights === null ||
    Array.isArray(weights)
  ) {
    throw new TypeError("fast evaluation weights must be an object");
  }
  const values = weights as Readonly<Record<string, unknown>>;
  const normalized = {} as {
    -readonly [Key in keyof EvalWeights]: EvalWeights[Key];
  };
  for (const key of EVAL_WEIGHT_KEYS) {
    const value = values[key];
    if (typeof value !== "number") {
      throw new TypeError(`weights.${key} must be a number`);
    }
    if (
      !Number.isSafeInteger(value) ||
      value < -MAX_ABSOLUTE_EVAL_WEIGHT ||
      value > MAX_ABSOLUTE_EVAL_WEIGHT
    ) {
      throw new RangeError(
        `weights.${key} must be a safe integer from -${MAX_ABSOLUTE_EVAL_WEIGHT} through ${MAX_ABSOLUTE_EVAL_WEIGHT}`,
      );
    }
    normalized[key] = value;
  }
  return Object.freeze(normalized);
}

export const DEFAULT_WEIGHTS: EvalWeights = Object.freeze({
  scoreUnit: 12_000,
  potion: 240,
  bomb: 220,
  activeMon: 45,
  faintMon: 520,
  faintDrainer: 900,
  faintCooldownStep: 80,
  drainerCloseToMana: 330,
  drainerCloseToOwnPool: 280,
  drainerCloseToSupermana: 180,
  carrierCloseToPool: 1600,
  carrierPointBonus: 420,
  carrierScoresThisTurn: 900,
  carrierScoresNextTurn: 700,
  winningCarrier: 8000,
  drainerPickupScoresThisTurn: 420,
  manaToOwnerPool: 170,
  manaPointsAttraction: 350,
  manaDrainerControl: 26,
  supermanaDrainerControl: 40,
  monCloseToCenter: 210,
  spiritCloseToEnemy: 200,
  spiritOnOwnBase: 180,
  angelCloseToDrainer: 150,
  angelGuardingDrainer: 260,
  attackerCloseToEnemyDrainer: 150,
  drainerThreatImmediate: 700,
  drainerThreatWalk: 240,
  carrierThreatFactor: 2,
  manaToNearestPool: 800,
});

// Marks an unreachable distance. Distance tables are sized to cover it and every real
// board distance, because table reads fall back to zero instead of throwing.
const UNREACHABLE_DISTANCE = 99;
const DISTANCE_TABLE_SIZE = Math.max(UNREACHABLE_DISTANCE + 1, BOARD_SIZE);
const MANA_POINT_SLOTS = 3;
const THREAT_WALK_TABLE_SIZE = MONS_MOVES_PER_TURN + 1;

export type EvalTables = {
  readonly weights: EvalWeights;
  readonly manaPointsAttraction: Int32Array;
  readonly manaToOwnerPool: Int32Array;
  readonly manaToNearestPool: Int32Array;
  readonly carrierCloseToPool: Int32Array;
  readonly drainerCloseToMana: Int32Array;
  readonly drainerCloseToOwnPool: Int32Array;
  readonly drainerCloseToSupermana: Int32Array;
  readonly angelCloseToDrainer: Int32Array;
  readonly spiritCloseToEnemy: Int32Array;
  readonly monCloseToCenter: Int32Array;
  readonly attackerCloseToEnemyDrainer: Int32Array;
  readonly threatWalkPlain: Float64Array;
  readonly threatWalkCarrier: Float64Array;
};

function distanceTable(numerator: number): Int32Array {
  const table = new Int32Array(DISTANCE_TABLE_SIZE);
  for (let distance = 0; distance < DISTANCE_TABLE_SIZE; distance += 1) {
    table[distance] = Math.trunc(numerator / (distance + 1));
  }
  return table;
}

function manaPointsAttractionTable(weight: number): Int32Array {
  const table = new Int32Array(MANA_POINT_SLOTS * DISTANCE_TABLE_SIZE);
  for (let points = 0; points < MANA_POINT_SLOTS; points += 1) {
    const base = points * DISTANCE_TABLE_SIZE;
    for (let distance = 0; distance < DISTANCE_TABLE_SIZE; distance += 1) {
      table[base + distance] = Math.trunc((points * weight) / (distance + 1));
    }
  }
  return table;
}

function threatWalkTable(weight: number, factor: number): Float64Array {
  const table = new Float64Array(THREAT_WALK_TABLE_SIZE);
  for (let steps = 0; steps < THREAT_WALK_TABLE_SIZE; steps += 1) {
    table[steps] = Math.trunc(
      (weight * factor * (MONS_MOVES_PER_TURN + 1 - steps)) /
        MONS_MOVES_PER_TURN,
    );
  }
  return table;
}

export function createEvalTables(weights: EvalWeights): EvalTables {
  return {
    weights,
    manaPointsAttraction: manaPointsAttractionTable(
      weights.manaPointsAttraction,
    ),
    manaToOwnerPool: distanceTable(weights.manaToOwnerPool),
    manaToNearestPool: distanceTable(weights.manaToNearestPool),
    carrierCloseToPool: distanceTable(weights.carrierCloseToPool),
    drainerCloseToMana: distanceTable(weights.drainerCloseToMana),
    drainerCloseToOwnPool: distanceTable(weights.drainerCloseToOwnPool),
    drainerCloseToSupermana: distanceTable(weights.drainerCloseToSupermana),
    angelCloseToDrainer: distanceTable(weights.angelCloseToDrainer),
    spiritCloseToEnemy: distanceTable(weights.spiritCloseToEnemy),
    monCloseToCenter: distanceTable(weights.monCloseToCenter),
    attackerCloseToEnemyDrainer: distanceTable(
      weights.attackerCloseToEnemyDrainer,
    ),
    threatWalkPlain: threatWalkTable(weights.drainerThreatWalk, 1),
    threatWalkCarrier: threatWalkTable(
      weights.drainerThreatWalk,
      weights.carrierThreatFactor,
    ),
  };
}

export const DEFAULT_EVAL_TABLES = createEvalTables(DEFAULT_WEIGHTS);

function ownPoolDistance(index: number, color: number): number {
  return u8(OWN_POOL_DISTANCE, color * BOARD_CELLS + index);
}

function centerDistance(index: number): number {
  return u8(CENTER_ROW_DISTANCE, index);
}

function estimatedAttackSteps(
  position: FastPosition,
  at: number,
  ownerColor: number,
  guarded: boolean,
): number {
  if (!damagingAttackCanComplete(u16(position.cells, at))) {
    return UNREACHABLE_DISTANCE;
  }
  const enemy = ownerColor ^ 1;
  let best = UNREACHABLE_DISTANCE;

  for (let kind = 0; kind < MON_KIND_COUNT; kind += 1) {
    const index = i32(position.monLocations, monId(kind, enemy));
    if (index < 0) continue;
    const cell = u16(position.cells, index);
    if (cellOccupancy(cell) !== OCC_MON) continue;
    if (cellConsumable(cell) === CONS_BOMB) {
      const reach = chebyshev(index, at) - 3;
      const steps = reach < 0 ? 0 : reach;
      if (steps === 0) return 0;
      if (steps < best) best = steps;
      continue;
    }
    if (cellCooldown(cell) !== 0) continue;
    if (cellConsumable(cell) !== 0) continue;
    if (guarded) continue;
    if (cellMana(cell) !== 0) continue;
    const targets =
      kind === KIND_MYSTIC
        ? MYSTIC_TARGETS
        : kind === KIND_DEMON
          ? DEMON_TARGETS
          : undefined;
    if (targets === undefined) continue;
    const targetStart = i32(targets.starts, at);
    const targetCount = i32(targets.counts, at);
    for (let offset = 0; offset < targetCount; offset += 1) {
      const origin = i32(targets.list, targetStart + offset);
      if (u8(position.squares, origin) >= SQ_MON_BASE) continue;
      if (kind === KIND_DEMON) {
        const between = midpoint(at, origin);
        if (between !== index && u16(position.cells, between) !== 0) continue;
        const betweenSquare = u8(position.squares, between);
        if (
          betweenSquare === SQ_SUPERMANA_BASE ||
          betweenSquare >= SQ_MON_BASE
        ) {
          continue;
        }
      }
      const steps = chebyshev(index, origin);
      if (steps === 0) return 0;
      if (steps < best) best = steps;
    }
  }
  return best;
}

export function evaluatePosition(
  position: FastPosition,
  weights: EvalWeights,
): number {
  return evaluateWithTables(
    position,
    createEvalTables(normalizeEvalWeights(weights)),
  );
}

export function evaluateWithTables(
  position: FastPosition,
  tables: EvalTables,
): number {
  const weights = tables.weights;
  let value =
    (position.whiteScore - position.blackScore) * weights.scoreUnit +
    (i32(position.potions, 0) - i32(position.potions, 1)) * weights.potion;

  const drainerWhite = i32(position.monLocations, monId(KIND_DRAINER, 0));
  const drainerBlack = i32(position.monLocations, monId(KIND_DRAINER, 1));
  const drainerReadyWhite =
    drainerWhite >= 0 && cellCooldown(u16(position.cells, drainerWhite)) === 0;
  const drainerReadyBlack =
    drainerBlack >= 0 && cellCooldown(u16(position.cells, drainerBlack)) === 0;
  let nearestManaWhite = UNREACHABLE_DISTANCE;
  let nearestManaBlack = UNREACHABLE_DISTANCE;
  let shortestPickupScoreWhite = UNREACHABLE_DISTANCE;
  let shortestPickupScoreBlack = UNREACHABLE_DISTANCE;
  const remaining = MONS_MOVES_PER_TURN - position.monsMoves;

  for (let slot = 0; slot < position.manaCount; slot += 1) {
    const index = i32(position.manaIndices, slot);
    const cell = u16(position.cells, index);
    if (cellOccupancy(cell) !== OCC_MANA) continue;
    const mana = cellMana(cell);
    const distanceWhite = drainerReadyWhite
      ? chebyshev(drainerWhite, index)
      : UNREACHABLE_DISTANCE;
    const distanceBlack = drainerReadyBlack
      ? chebyshev(drainerBlack, index)
      : UNREACHABLE_DISTANCE;
    if (distanceWhite < nearestManaWhite) nearestManaWhite = distanceWhite;
    if (distanceBlack < nearestManaBlack) nearestManaBlack = distanceBlack;
    const scoreDistance = u8(POOL_DISTANCE, index);
    if (drainerReadyWhite) {
      const pickupScoreDistance = distanceWhite + scoreDistance;
      if (pickupScoreDistance < shortestPickupScoreWhite) {
        shortestPickupScoreWhite = pickupScoreDistance;
      }
    }
    if (drainerReadyBlack) {
      const pickupScoreDistance = distanceBlack + scoreDistance;
      if (pickupScoreDistance < shortestPickupScoreBlack) {
        shortestPickupScoreBlack = pickupScoreDistance;
      }
    }
    let control = distanceBlack - distanceWhite;
    if (control > 4) control = 4;
    if (control < -4) control = -4;
    if (weights.manaPointsAttraction !== 0) {
      const attraction = tables.manaPointsAttraction;
      if (drainerReadyWhite) {
        value += i32(
          attraction,
          manaScoreValue(mana, 0) * DISTANCE_TABLE_SIZE + distanceWhite,
        );
      }
      if (drainerReadyBlack) {
        value -= i32(
          attraction,
          manaScoreValue(mana, 1) * DISTANCE_TABLE_SIZE + distanceBlack,
        );
      }
    }
    if (mana === MANA_SUPER) {
      value += weights.supermanaDrainerControl * control;
    } else {
      const ownerColor = mana - 1;
      const ownerSign = ownerColor === 0 ? 1 : -1;
      value +=
        ownerSign *
        i32(tables.manaToOwnerPool, ownPoolDistance(index, ownerColor));
      value += ownerSign * i32(tables.manaToNearestPool, scoreDistance);
      value += weights.manaDrainerControl * control;
    }
  }

  for (let color = 0; color < COLOR_COUNT; color += 1) {
    const sign = color === 0 ? 1 : -1;
    const enemy = color ^ 1;
    for (let kind = 0; kind < MON_KIND_COUNT; kind += 1) {
      const index = i32(position.monLocations, monId(kind, color));
      if (index < 0) continue;
      const cell = u16(position.cells, index);
      if (cellOccupancy(cell) !== OCC_MON) continue;
      const cooldown = cellCooldown(cell);
      if (cooldown !== 0) {
        value -=
          sign *
          ((kind === KIND_DRAINER ? weights.faintDrainer : weights.faintMon) +
            weights.faintCooldownStep * cooldown);
      }
      const square = u8(position.squares, index);
      if (cooldown === 0 && square < SQ_MON_BASE) {
        value += sign * weights.activeMon;
      }

      const carriedMana = cellMana(cell);
      if (carriedMana !== 0) {
        const points = manaScoreValue(carriedMana, color);
        const poolDistance = u8(POOL_DISTANCE, index);
        value +=
          sign *
          (i32(tables.carrierCloseToPool, poolDistance) +
            points * weights.carrierPointBonus);
        if (color === position.active && poolDistance <= remaining) {
          value += sign * weights.carrierScoresThisTurn * points;
        } else if (
          color !== position.active &&
          poolDistance <= MONS_MOVES_PER_TURN
        ) {
          value += sign * weights.carrierScoresNextTurn * points;
        }
        const ownScore =
          color === 0 ? position.whiteScore : position.blackScore;
        if (points >= TARGET_SCORE - ownScore) {
          value += sign * weights.winningCarrier;
        }
      } else if (cellConsumable(cell) === CONS_BOMB) {
        value += sign * weights.bomb;
      }

      if (cooldown !== 0) continue;

      if (kind === KIND_DRAINER) {
        const minMana = color === 0 ? nearestManaWhite : nearestManaBlack;
        const pickupScoreDistance =
          color === 0 ? shortestPickupScoreWhite : shortestPickupScoreBlack;
        value += sign * i32(tables.drainerCloseToMana, minMana);
        value +=
          sign *
          i32(tables.drainerCloseToOwnPool, ownPoolDistance(index, color));
        value +=
          sign *
          i32(
            tables.drainerCloseToSupermana,
            chebyshev(index, SUPERMANA_BASE_INDEX),
          );
        if (
          carriedMana === 0 &&
          cellConsumable(cell) === 0 &&
          color === position.active &&
          pickupScoreDistance <= remaining
        ) {
          value += sign * weights.drainerPickupScoresThisTurn;
        }
        const guarded = awakeAngelGuards(position, color, index);
        const threatSteps = estimatedAttackSteps(
          position,
          index,
          color,
          guarded,
        );
        const carrying = carriedMana !== 0;
        if (threatSteps === 0) {
          const factor = carrying ? weights.carrierThreatFactor : 1;
          value -= sign * weights.drainerThreatImmediate * factor;
        } else if (threatSteps <= MONS_MOVES_PER_TURN) {
          const walk = carrying
            ? tables.threatWalkCarrier
            : tables.threatWalkPlain;
          value -= sign * (walk[threatSteps] ?? 0);
        }
        if (guarded) {
          value += sign * weights.angelGuardingDrainer;
        }
      } else if (kind === KIND_ANGEL) {
        const own = color === 0 ? drainerWhite : drainerBlack;
        if (own >= 0) {
          value +=
            sign * i32(tables.angelCloseToDrainer, chebyshev(index, own));
        }
      } else if (kind === KIND_SPIRIT) {
        let nearestEnemy = UNREACHABLE_DISTANCE;
        for (let other = 0; other < MON_KIND_COUNT; other += 1) {
          const enemyIndex = i32(position.monLocations, monId(other, enemy));
          if (enemyIndex < 0) continue;
          const enemyCell = u16(position.cells, enemyIndex);
          if (cellCooldown(enemyCell) !== 0) continue;
          const distance = chebyshev(index, enemyIndex);
          if (distance < nearestEnemy) nearestEnemy = distance;
        }
        value += sign * i32(tables.spiritCloseToEnemy, nearestEnemy);
        if (square >= SQ_MON_BASE) value -= sign * weights.spiritOnOwnBase;
      } else {
        value += sign * i32(tables.monCloseToCenter, centerDistance(index));
        const enemyDrainer = enemy === 0 ? drainerWhite : drainerBlack;
        const enemyDrainerReady =
          enemy === 0 ? drainerReadyWhite : drainerReadyBlack;
        if (enemyDrainer >= 0 && enemyDrainerReady) {
          value +=
            sign *
            i32(
              tables.attackerCloseToEnemyDrainer,
              chebyshev(index, enemyDrainer),
            );
        }
      }
    }
  }

  return value;
}
