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

export const NORMAL_WEIGHTS: EvalWeights = normalizeEvalWeights({
  ...DEFAULT_WEIGHTS,
  threatMoverScaleSpare: 35,
});

export const LEARNED_PRO_WEIGHTS: EvalWeights = normalizeEvalWeights(DEFAULT_WEIGHTS);

export const LEARNED_PRO_MODEL_FILE_SHA256 =
  "64e7236d7d22c93dc5d7cbbc79874875b10e0109793d45d71a924dff49990f0f";
export const LEARNED_PRO_MODEL_SHA256 =
  "95ac611d89e4c668a1f69043e18d4b30c1da403a639ba96c0921c104638ac564";
export const LEARNED_PRO_TABLE_SHA256 =
  "bdeb2ed8e5541690322e3dd5ee31b48e9eea4e426a1041069b637d09fa93c88d";
export const LEARNED_PRO_RESIDUAL_SCALE = 0.5;
const LEARNED_PRO_PHASE_NUMERATORS_BASE64 =
  "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";
let learnedProPhaseNumerators: Int16Array | undefined;

function memoizedLearnedProPhaseNumerators(): Int16Array {
  if (learnedProPhaseNumerators !== undefined) return learnedProPhaseNumerators;
  try {
    const table = new Int16Array(3_025);
    const alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bits = 0,
      buffer = 0,
      byteIndex = 0;
    let lowByte = 0;
    for (const character of LEARNED_PRO_PHASE_NUMERATORS_BASE64) {
      if (character === "=") break;
      const value = alphabet.indexOf(character);
      buffer = (buffer << 6) | value;
      bits += 6;
      if (bits < 8) continue;
      bits -= 8;
      const byte = (buffer >> bits) & 0xff;
      buffer &= (1 << bits) - 1;
      if ((byteIndex & 1) === 0) {
        lowByte = byte;
      } else {
        const word = lowByte | (byte << 8);
        table[byteIndex >> 1] = word >= 0x8000 ? word - 0x1_0000 : word;
      }
      byteIndex += 1;
    }
    learnedProPhaseNumerators = table;
    return table;
  } catch (error) {
    rethrowFastWorkspaceAllocation(error);
  }
}

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
  readonly learnedPro: Int16Array | undefined;
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
    learnedPro:
      weights === LEARNED_PRO_WEIGHTS ? memoizedLearnedProPhaseNumerators() : undefined,
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
  if (
    weights === DEFAULT_WEIGHTS ||
    weights === NORMAL_WEIGHTS ||
    weights === LEARNED_PRO_WEIGHTS
  ) {
    return weights;
  }
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
