import { BOARD_SIZE } from "../engine/board/geometry.js";
import { MONS_MOVES_PER_TURN, TARGET_SCORE } from "../engine/board/config.js";
import { i32 } from "./board.js";
import { rethrowFastWorkspaceAllocation } from "./allocation.js";

const MAX_ABSOLUTE_EVAL_WEIGHT = 1_000_000;
const INT32_MIN = -0x8000_0000;
const INT32_MAX = 0x7fff_ffff;

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
  "manaStepQueue1",
  "manaStepQueue2",
  "manaStepQueue3",
  "manaStepQueue4",
  "manaStepQueue5",
  "manaStepWinThreat",
  "drainerTripTurn1",
  "drainerTripTurn2",
  "drainerTripTurn3",
  "drainerTripTurn4",
  "supermanaCarrier",
  "scoreShape10",
  "scoreShape20",
  "scoreShape21",
  "scoreShape30",
  "scoreShape31",
  "scoreShape32",
  "tripGradient",
  "raceHalfTurn",
  "threatMoverScaleSpare",
  "threatMoverScaleFew",
  "tripTwoPointScale",
] as const;

export type EvalWeights = Readonly<Record<(typeof EVAL_WEIGHT_KEYS)[number], number>>;

export function normalizeEvalWeights(weights: unknown): EvalWeights {
  if (typeof weights !== "object" || weights === null || Array.isArray(weights)) {
    throw new TypeError("fast evaluation weights must be an object");
  }
  const values = weights as Readonly<Record<string, unknown>>;
  // The all-keys literal gives every normalized object one shared in-object-property
  // map; incremental construction from {} would store properties out-of-object and
  // make every weights load in the hot evaluator measurably slower.
  const normalized: {
    -readonly [Key in keyof EvalWeights]: EvalWeights[Key];
  } = {
    scoreUnit: 0,
    potion: 0,
    bomb: 0,
    activeMon: 0,
    faintMon: 0,
    faintDrainer: 0,
    faintCooldownStep: 0,
    drainerCloseToMana: 0,
    drainerCloseToOwnPool: 0,
    drainerCloseToSupermana: 0,
    carrierCloseToPool: 0,
    carrierPointBonus: 0,
    carrierScoresThisTurn: 0,
    carrierScoresNextTurn: 0,
    winningCarrier: 0,
    drainerPickupScoresThisTurn: 0,
    manaToOwnerPool: 0,
    manaPointsAttraction: 0,
    manaDrainerControl: 0,
    supermanaDrainerControl: 0,
    monCloseToCenter: 0,
    spiritCloseToEnemy: 0,
    spiritOnOwnBase: 0,
    angelCloseToDrainer: 0,
    angelGuardingDrainer: 0,
    attackerCloseToEnemyDrainer: 0,
    drainerThreatImmediate: 0,
    drainerThreatWalk: 0,
    carrierThreatFactor: 0,
    manaToNearestPool: 0,
    manaStepQueue1: 0,
    manaStepQueue2: 0,
    manaStepQueue3: 0,
    manaStepQueue4: 0,
    manaStepQueue5: 0,
    manaStepWinThreat: 0,
    drainerTripTurn1: 0,
    drainerTripTurn2: 0,
    drainerTripTurn3: 0,
    drainerTripTurn4: 0,
    supermanaCarrier: 0,
    scoreShape10: 0,
    scoreShape20: 0,
    scoreShape21: 0,
    scoreShape30: 0,
    scoreShape31: 0,
    scoreShape32: 0,
    tripGradient: 0,
    raceHalfTurn: 0,
    threatMoverScaleSpare: 0,
    threatMoverScaleFew: 0,
    tripTwoPointScale: 0,
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

export const DEFAULT_WEIGHTS: EvalWeights = normalizeEvalWeights({
  scoreUnit: 12_000,
  potion: 240,
  bomb: 1_200,
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
  drainerThreatImmediate: 2_100,
  drainerThreatWalk: 720,
  carrierThreatFactor: 2,
  manaToNearestPool: 0,
  manaStepQueue1: 3_600,
  manaStepQueue2: 2_500,
  manaStepQueue3: 1_050,
  manaStepQueue4: 800,
  manaStepQueue5: 800,
  manaStepWinThreat: 8_000,
  drainerTripTurn1: 4_200,
  drainerTripTurn2: 2_200,
  drainerTripTurn3: 1_000,
  drainerTripTurn4: 400,
  supermanaCarrier: 4_000,
  scoreShape10: -6_824,
  scoreShape20: -6_978,
  scoreShape21: -3_085,
  scoreShape30: -8_259,
  scoreShape31: -4_275,
  scoreShape32: -1_052,
  tripGradient: 400,
  raceHalfTurn: 900,
  threatMoverScaleSpare: 25,
  threatMoverScaleFew: 55,
  tripTwoPointScale: 290,
});

// Marks an unreachable distance. Distance tables are sized to cover it and every real
// board distance, because table reads fall back to zero instead of throwing.
export const UNREACHABLE_DISTANCE = 99;
export const DISTANCE_TABLE_SIZE = Math.max(UNREACHABLE_DISTANCE + 1, BOARD_SIZE);
const MANA_POINT_SLOTS = 3;
export const SCORE_SHAPE_STRIDE = TARGET_SCORE + 2;
const THREAT_WALK_TABLE_SIZE = MONS_MOVES_PER_TURN + 1;
// How much of its own turn the threatened side still holds: none of it, one or two sub-moves,
// or enough of it to walk away from anything.
export const THREAT_BUCKET_EXPOSED = 0;
export const THREAT_BUCKET_FEW = 1;
export const THREAT_BUCKET_SPARE = 2;
export const THREAT_BUCKETS = 3;
export const THREAT_SPARE_MOVES = 3;
export const THREAT_WALK_STRIDE = THREAT_WALK_TABLE_SIZE;
export const RACE_SPAN = 6;
const RACE_TABLE_SIZE = RACE_SPAN * 2 + 1;
export const RACE_MAX_TURNS = 6;
export const RACE_LATE_NEED = 2;
const TRIP_STEP_MAX = 12;

export type EvalTables = {
  readonly weights: EvalWeights;
  readonly scoreShape: Int32Array;
  readonly drainerTrip: Int32Array;
  readonly drainerTripTwoPoint: Int32Array;
  readonly race: Int32Array;
  readonly tripStep: Int32Array;
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
  readonly threatImmediate: Int32Array;
  readonly threatWalk: Float64Array;
};

function distanceTable(numerator: number): Int32Array {
  const table = createInt32Table(DISTANCE_TABLE_SIZE);
  for (let distance = 0; distance < DISTANCE_TABLE_SIZE; distance += 1) {
    table[distance] = Math.trunc(numerator / (distance + 1));
  }
  return table;
}

// The free mana step delivers one own loose mana one square per turn, so pool distance is a
// queue position rather than a proximity gradient: the fitted shape is a cliff at the
// distances that bank the point inside the opponent's reply horizon, not a reciprocal.
function manaStepQueueTable(weights: EvalWeights): Int32Array {
  const table = createInt32Table(DISTANCE_TABLE_SIZE);
  const base = weights.manaToNearestPool;
  for (let distance = 0; distance < DISTANCE_TABLE_SIZE; distance += 1) {
    const queue =
      distance <= 1
        ? weights.manaStepQueue1
        : distance === 2
          ? weights.manaStepQueue2
          : distance === 3
            ? weights.manaStepQueue3
            : distance === 4
              ? weights.manaStepQueue4
              : weights.manaStepQueue5;
    table[distance] = queue + Math.trunc(base / (distance + 1));
  }
  return table;
}

// A race to a fixed target is not a function of the score difference: the marginal value of a
// point rises as the need falls. The correction is antisymmetric by construction, so the
// rotation axiom holds for either side to move without a mover-relative branch.
function scoreShapeTable(weights: EvalWeights): Int32Array {
  const table = createInt32Table(SCORE_SHAPE_STRIDE * SCORE_SHAPE_STRIDE);
  const written = new Set<number>();
  const set = (own: number, other: number, value: number): void => {
    if (own === other) {
      throw new RangeError("score shape cells must be off the diagonal");
    }
    const index = own * SCORE_SHAPE_STRIDE + other;
    const mirror = other * SCORE_SHAPE_STRIDE + own;
    if (written.has(index) || written.has(mirror)) {
      throw new RangeError("score shape cells must be written once");
    }
    written.add(index);
    written.add(mirror);
    table[index] = value;
    table[mirror] = -value;
  };
  set(1, 0, weights.scoreShape10);
  set(2, 0, weights.scoreShape20);
  set(2, 1, weights.scoreShape21);
  set(3, 0, weights.scoreShape30);
  set(3, 1, weights.scoreShape31);
  set(3, 2, weights.scoreShape32);
  // The correction must never outrun the linear term, or a scored point could lower the
  // evaluation and break the monotonicity axiom the theorem suite asserts.
  for (let own = 0; own + 1 < SCORE_SHAPE_STRIDE; own += 1) {
    for (let other = 0; other < SCORE_SHAPE_STRIDE; other += 1) {
      const gain =
        weights.scoreUnit +
        i32(table, (own + 1) * SCORE_SHAPE_STRIDE + other) -
        i32(table, own * SCORE_SHAPE_STRIDE + other);
      if (gain < 0) {
        throw new RangeError("score shape corrections must keep the score monotone");
      }
    }
  }
  return table;
}

// Progress toward a point is counted in steps but paid in turns, so the fused pick-up and
// delivery distance is bucketed by the turns it still needs beyond the current budget.
function drainerTripTable(weights: EvalWeights): Int32Array {
  const table = createInt32Table(DISTANCE_TABLE_SIZE);
  for (let excess = 0; excess < DISTANCE_TABLE_SIZE; excess += 1) {
    const turns = 1 + Math.ceil(excess / MONS_MOVES_PER_TURN);
    table[excess] =
      turns <= 1
        ? weights.drainerTripTurn1
        : turns === 2
          ? weights.drainerTripTurn2
          : turns === 3
            ? weights.drainerTripTurn3
            : weights.drainerTripTurn4;
  }
  return table;
}

// A trip that ends in two points is a different plan from one that ends in one, so the drainer
// weighs them against each other instead of walking to whichever item is nearest. At the neutral
// scale the two prices coincide and the choice reduces to the shorter trip.
function twoPointTripTable(weights: EvalWeights): Int32Array {
  const table = drainerTripTable(weights);
  for (let excess = 0; excess < DISTANCE_TABLE_SIZE; excess += 1) {
    table[excess] = scaledInt32(
      i32(table, excess),
      weights.tripTwoPointScale,
      "two-point trip",
    );
  }
  return table;
}

// The race the objective describes is between the two tempos, not between two independent
// distances: an additive form prices each side's remaining turns but never the lead itself.
// Half-turn units carry the side to move, and the table is antisymmetric by construction.
function raceTable(weight: number): Int32Array {
  const table = createInt32Table(RACE_TABLE_SIZE);
  for (let slot = 0; slot < RACE_TABLE_SIZE; slot += 1) {
    table[slot] = (slot - RACE_SPAN) * weight;
  }
  return table;
}

// Turn buckets are flat inside a turn, so on their own they give a plan no reason to walk the
// steps it has already paid for. The gradient is a within-bucket tie-break on the same fused
// distance the buckets are cut from, not a second proximity opinion.
function tripStepTable(weight: number): Int32Array {
  const table = createInt32Table(DISTANCE_TABLE_SIZE);
  for (let distance = 0; distance < DISTANCE_TABLE_SIZE; distance += 1) {
    const steps = distance > TRIP_STEP_MAX ? TRIP_STEP_MAX : distance;
    table[distance] = steps * weight;
  }
  return table;
}

function manaPointsAttractionTable(weight: number): Int32Array {
  const table = createInt32Table(MANA_POINT_SLOTS * DISTANCE_TABLE_SIZE);
  for (let points = 0; points < MANA_POINT_SLOTS; points += 1) {
    const base = points * DISTANCE_TABLE_SIZE;
    for (let distance = 0; distance < DISTANCE_TABLE_SIZE; distance += 1) {
      table[base + distance] = Math.trunc((points * weight) / (distance + 1));
    }
  }
  return table;
}

// A threat is a threat only if the threatened side does not move first: with sub-moves still
// in hand the owner can deliver, step away, or block before the attack can be played, and the
// more of them it holds the likelier that is. Both threat tables are read through the same
// bucket, so the discount cannot drift between the immediate and the walking case.
function moverScale(weights: EvalWeights, bucket: number): number {
  if (bucket === THREAT_BUCKET_EXPOSED) return 100;
  return bucket === THREAT_BUCKET_SPARE
    ? weights.threatMoverScaleSpare
    : weights.threatMoverScaleFew;
}

function scaledSafeInteger(value: number, scale: number, name: string): number {
  const product = value * scale;
  if (!Number.isSafeInteger(product)) {
    throw new RangeError(`${name} derived value requires a safe integer product`);
  }
  return Math.trunc(product / 100);
}

function scaledInt32(value: number, scale: number, name: string): number {
  const result = scaledSafeInteger(value, scale, name);
  if (result < INT32_MIN || result > INT32_MAX) {
    throw new RangeError(`${name} derived value must fit a signed 32-bit integer`);
  }
  return result;
}

function threatImmediateTable(weights: EvalWeights): Int32Array {
  const table = createInt32Table(2 * THREAT_BUCKETS);
  for (let carrying = 0; carrying < 2; carrying += 1) {
    const factor = carrying === 1 ? weights.carrierThreatFactor : 1;
    for (let bucket = 0; bucket < THREAT_BUCKETS; bucket += 1) {
      table[carrying * THREAT_BUCKETS + bucket] = scaledInt32(
        weights.drainerThreatImmediate * factor,
        moverScale(weights, bucket),
        "immediate threat",
      );
    }
  }
  return table;
}

function threatWalkTable(weights: EvalWeights): Float64Array {
  const table = createFloat64Table(2 * THREAT_BUCKETS * THREAT_WALK_TABLE_SIZE);
  for (let carrying = 0; carrying < 2; carrying += 1) {
    const factor = carrying === 1 ? weights.carrierThreatFactor : 1;
    for (let bucket = 0; bucket < THREAT_BUCKETS; bucket += 1) {
      const scale = moverScale(weights, bucket);
      const base = (carrying * THREAT_BUCKETS + bucket) * THREAT_WALK_TABLE_SIZE;
      for (let steps = 0; steps < THREAT_WALK_TABLE_SIZE; steps += 1) {
        table[base + steps] = scaledSafeInteger(
          Math.trunc(
            (weights.drainerThreatWalk * factor * (MONS_MOVES_PER_TURN + 1 - steps)) /
              MONS_MOVES_PER_TURN,
          ),
          scale,
          "walking threat",
        );
      }
    }
  }
  return table;
}

function createInt32Table(length: number): Int32Array {
  try {
    return new Int32Array(length);
  } catch (error) {
    rethrowFastWorkspaceAllocation(error);
  }
}

function createFloat64Table(length: number): Float64Array {
  try {
    return new Float64Array(length);
  } catch (error) {
    rethrowFastWorkspaceAllocation(error);
  }
}

// Crossing into the endgame band switches the race correction on, so a point that crosses it
// must still be worth more than the correction can take away, or the monotonicity axiom would
// depend on which side happens to lead the race.
function assertRaceStaysMonotone(weights: EvalWeights, scoreShape: Int32Array): void {
  const swing = RACE_SPAN * Math.abs(weights.raceHalfTurn);
  if (swing === 0) return;
  const gateRow = TARGET_SCORE - RACE_LATE_NEED;
  for (let other = 0; other < gateRow; other += 1) {
    const gain =
      weights.scoreUnit +
      i32(scoreShape, gateRow * SCORE_SHAPE_STRIDE + other) -
      i32(scoreShape, (gateRow - 1) * SCORE_SHAPE_STRIDE + other);
    if (swing > gain) {
      throw new RangeError(
        "race correction must not outweigh the point that opens the endgame band",
      );
    }
  }
}

export function createEvalTables(weights: EvalWeights): EvalTables {
  const scoreShape = scoreShapeTable(weights);
  assertRaceStaysMonotone(weights, scoreShape);
  return {
    weights,
    manaPointsAttraction: manaPointsAttractionTable(weights.manaPointsAttraction),
    manaToOwnerPool: distanceTable(weights.manaToOwnerPool),
    manaToNearestPool: manaStepQueueTable(weights),
    scoreShape,
    drainerTrip: drainerTripTable(weights),
    drainerTripTwoPoint: twoPointTripTable(weights),
    race: raceTable(weights.raceHalfTurn),
    tripStep: tripStepTable(weights.tripGradient),
    carrierCloseToPool: distanceTable(weights.carrierCloseToPool),
    drainerCloseToMana: distanceTable(weights.drainerCloseToMana),
    drainerCloseToOwnPool: distanceTable(weights.drainerCloseToOwnPool),
    drainerCloseToSupermana: distanceTable(weights.drainerCloseToSupermana),
    angelCloseToDrainer: distanceTable(weights.angelCloseToDrainer),
    spiritCloseToEnemy: distanceTable(weights.spiritCloseToEnemy),
    monCloseToCenter: distanceTable(weights.monCloseToCenter),
    attackerCloseToEnemyDrainer: distanceTable(weights.attackerCloseToEnemyDrainer),
    threatImmediate: threatImmediateTable(weights),
    threatWalk: threatWalkTable(weights),
  };
}

export const DEFAULT_EVAL_TABLES = createEvalTables(DEFAULT_WEIGHTS);

const NORMALIZED_WEIGHTS_MEMO = new WeakMap<object, EvalWeights>();

export function memoizedNormalizedEvalWeights(weights: EvalWeights): EvalWeights {
  if (weights === DEFAULT_WEIGHTS) return DEFAULT_WEIGHTS;
  if (!Object.isFrozen(weights)) return normalizeEvalWeights(weights);
  let normalized = NORMALIZED_WEIGHTS_MEMO.get(weights);
  if (normalized === undefined) {
    normalized = normalizeEvalWeights(weights);
    NORMALIZED_WEIGHTS_MEMO.set(weights, normalized);
  }
  return normalized;
}

const EVAL_TABLES_MEMO = new WeakMap<EvalWeights, EvalTables>();

export function memoizedEvalTables(normalizedWeights: EvalWeights): EvalTables {
  if (normalizedWeights === DEFAULT_WEIGHTS) return DEFAULT_EVAL_TABLES;
  let tables = EVAL_TABLES_MEMO.get(normalizedWeights);
  if (tables === undefined) {
    tables = createEvalTables(normalizedWeights);
    EVAL_TABLES_MEMO.set(normalizedWeights, tables);
  }
  return tables;
}
