import { BOARD_CELLS } from "../../engine/geometry.js";
import {
  ACTIONS_PER_TURN,
  MANA_MOVES_PER_TURN,
  MONS_MOVES_PER_TURN,
} from "../../engine/config.js";
import {
  COLOR_COUNT,
  KIND_DRAINER,
  OCC_CONS,
  OCC_MANA,
  POOL_DISTANCE,
  Z_SCALAR_HI,
  Z_SCALAR_LO,
  cellMana,
  cellMonKind,
  cellOccupancy,
  at,
  i32,
  manaScoreValue,
  squareIsPool,
  u16,
  u8,
} from "./board.js";
import {
  DEFAULT_WEIGHTS,
  WIN_VALUE,
  evaluatePosition,
  normalizeEvalWeights,
  type EvalWeights,
} from "./evaluate.js";
import { MAX_MOVES, generateMoves } from "./moves.js";
import {
  FastPosition,
  MOVE_BOMB,
  MOVE_DEMON,
  MOVE_MANA,
  MOVE_MON,
  MOVE_MYSTIC,
  MOVE_SPIRIT,
  FAST_MOVE_UNREPRESENTABLE,
  applyFastMove,
  moveAux,
  moveFrom,
  moveTo,
  moveType,
  positionWinner,
} from "./position.js";

const MAX_PLY = 48;
const DEFAULT_TRANSPOSITION_CAPACITY = 1 << 20;
const MAX_TRANSPOSITION_CAPACITY = DEFAULT_TRANSPOSITION_CAPACITY;
const FLAG_EXACT = 0;
const FLAG_LOWER = 1;
const FLAG_UPPER = 2;
const FLAG_MOVE_ONLY = 3;
const INFO_SELECTIVE = 1 << 30;
const TABLE_GENERATION_MASK = 0x3f_ffff;
const TIMEOUT_CHECK_MASK = 511;
const TACTICAL_THRESHOLD = 1 << 15;
const KEY_TT = 1 << 28;
const KEY_KILLER_A = 1 << 24;
const KEY_KILLER_B = 1 << 23;
const INFINITY_SCORE = WIN_VALUE * 2;
const MAX_NONTERMINAL_SCORE = WIN_VALUE - MAX_PLY - 1;
const NO_TIMEOUT = (): boolean => false;
export const MAX_SEARCH_DEPTH = MAX_PLY - 2;
const ACTIVE_STATES = COLOR_COUNT;
const MONS_MOVE_STATES = MONS_MOVES_PER_TURN + 1;
const MANA_MOVE_STATES = MANA_MOVES_PER_TURN + 1;
const ACTION_STATES = ACTIONS_PER_TURN + 1;
const FIRST_TURN_STATES = 2;
const POTION_BUCKET_STATES = 4;
const SCALAR_STATE_COUNT =
  ACTIVE_STATES *
  MONS_MOVE_STATES *
  MANA_MOVE_STATES *
  ACTION_STATES *
  FIRST_TURN_STATES *
  POTION_BUCKET_STATES *
  POTION_BUCKET_STATES;
const EMPTY_TT_ARRAY: Int32Array = new Int32Array(0);

if (
  Z_SCALAR_LO.length !== Z_SCALAR_HI.length ||
  Z_SCALAR_LO.length < SCALAR_STATE_COUNT
) {
  throw new RangeError(
    "fast scalar hash table is too small for the configured state space",
  );
}

export type SearchTuning = {
  readonly lateMoveReduction: boolean;
  readonly lateMoveIndex: number;
  readonly lateMoveDeepIndex: number;
  readonly moveCountPruning: boolean;
  readonly moveCountDepth: number;
  readonly moveCountBase: number;
  readonly moveCountFactor: number;
  readonly futilityMargin: number;
};

const DEFAULT_TUNING: SearchTuning = Object.freeze({
  lateMoveReduction: true,
  lateMoveIndex: 3,
  lateMoveDeepIndex: 8,
  moveCountPruning: true,
  moveCountDepth: 3,
  moveCountBase: 4,
  moveCountFactor: 5,
  futilityMargin: 900,
});

export type SearchLimits = {
  readonly maxDepth: number;
  readonly maxNodes: number;
  readonly tuning?: SearchTuning;
};

export type NormalizedSearchLimits = {
  readonly maxDepth: number;
  readonly maxNodes: number;
  readonly tuning: SearchTuning;
};

export type SearchOutcome = {
  readonly move: number;
  readonly score: number;
  readonly depth: number;
  readonly nodes: number;
  readonly supported: boolean;
};

type RootSearchOutcome = {
  readonly move: number;
  readonly score: number;
  readonly selective: boolean;
};

export type TranspositionPart = "keyLo" | "keyHi" | "score" | "info" | "move";

export type FastSearcherOptions = {
  readonly transpositionCapacity?: number;
  readonly allocateTranspositionPart?: (
    part: TranspositionPart,
    length: number,
  ) => Int32Array;
};

export function normalizeSearchLimits(limits: unknown): NormalizedSearchLimits {
  if (typeof limits !== "object" || limits === null || Array.isArray(limits)) {
    throw new TypeError("fast search limits must be an object");
  }
  const values = limits as Readonly<Record<string, unknown>>;
  const maxDepth = values["maxDepth"];
  if (
    !Number.isSafeInteger(maxDepth) ||
    (maxDepth as number) < 0 ||
    (maxDepth as number) > MAX_SEARCH_DEPTH
  ) {
    throw new RangeError(
      `maxDepth must be a safe integer from 0 through ${MAX_SEARCH_DEPTH}`,
    );
  }
  const maxNodes = values["maxNodes"];
  if (!Number.isSafeInteger(maxNodes) || (maxNodes as number) < 0) {
    throw new RangeError("maxNodes must be a nonnegative safe integer");
  }
  const tuning = values["tuning"];
  return Object.freeze({
    maxDepth: maxDepth as number,
    maxNodes: maxNodes as number,
    tuning:
      tuning === undefined ? DEFAULT_TUNING : normalizeSearchTuning(tuning),
  });
}

export function validateSearchLimits(limits: SearchLimits): void {
  normalizeSearchLimits(limits);
}

function normalizeSearchTuning(tuning: unknown): SearchTuning {
  if (typeof tuning !== "object" || tuning === null || Array.isArray(tuning)) {
    throw new TypeError("fast search tuning must be an object");
  }
  const values = tuning as Readonly<Record<string, unknown>>;
  return Object.freeze({
    lateMoveReduction: normalizedTuningBoolean(values, "lateMoveReduction"),
    lateMoveIndex: normalizedTuningInteger(values, "lateMoveIndex", MAX_MOVES),
    lateMoveDeepIndex: normalizedTuningInteger(
      values,
      "lateMoveDeepIndex",
      MAX_MOVES,
    ),
    moveCountPruning: normalizedTuningBoolean(values, "moveCountPruning"),
    moveCountDepth: normalizedTuningInteger(
      values,
      "moveCountDepth",
      MAX_SEARCH_DEPTH,
    ),
    moveCountBase: normalizedTuningInteger(values, "moveCountBase", MAX_MOVES),
    moveCountFactor: normalizedTuningInteger(
      values,
      "moveCountFactor",
      MAX_MOVES,
    ),
    futilityMargin: normalizedTuningInteger(
      values,
      "futilityMargin",
      MAX_NONTERMINAL_SCORE,
    ),
  });
}

function normalizedTuningBoolean(
  tuning: Readonly<Record<string, unknown>>,
  key: keyof SearchTuning,
): boolean {
  const value = tuning[key];
  if (typeof value !== "boolean") {
    throw new TypeError(`tuning.${key} must be a boolean`);
  }
  return value;
}

function normalizedTuningInteger(
  tuning: Readonly<Record<string, unknown>>,
  key: keyof SearchTuning,
  maximum: number,
): number {
  const value = tuning[key];
  if (typeof value !== "number") {
    throw new TypeError(`tuning.${key} must be a number`);
  }
  if (!Number.isSafeInteger(value) || value < 0 || value > maximum) {
    throw new RangeError(
      `tuning.${key} must be a safe integer from 0 through ${maximum}`,
    );
  }
  return value;
}

function normalizeTranspositionCapacity(capacity: unknown): number {
  if (capacity === undefined) return DEFAULT_TRANSPOSITION_CAPACITY;
  if (
    !Number.isSafeInteger(capacity) ||
    (capacity as number) < 1 ||
    (capacity as number) > MAX_TRANSPOSITION_CAPACITY ||
    ((capacity as number) & ((capacity as number) - 1)) !== 0
  ) {
    throw new RangeError(
      `transpositionCapacity must be a power of two from 1 through ${MAX_TRANSPOSITION_CAPACITY}`,
    );
  }
  return capacity as number;
}

function defaultTranspositionAllocator(
  _part: TranspositionPart,
  length: number,
): Int32Array {
  return new Int32Array(length);
}

function tryAllocateTranspositionPart(
  allocate: NonNullable<FastSearcherOptions["allocateTranspositionPart"]>,
  part: TranspositionPart,
  length: number,
): Int32Array | undefined {
  try {
    const array = allocate(part, length);
    return ArrayBuffer.isView(array) &&
      Object.prototype.toString.call(array) === "[object Int32Array]" &&
      array.length === length
      ? array
      : undefined;
  } catch (error) {
    if (error instanceof RangeError) return undefined;
    throw error;
  }
}

function transpositionPartsOverlap(parts: readonly Int32Array[]): boolean {
  for (let leftIndex = 0; leftIndex < parts.length; leftIndex += 1) {
    const left = at(parts, leftIndex);
    const leftEnd = left.byteOffset + left.byteLength;
    for (
      let rightIndex = leftIndex + 1;
      rightIndex < parts.length;
      rightIndex += 1
    ) {
      const right = at(parts, rightIndex);
      if (left.buffer !== right.buffer) continue;
      const rightEnd = right.byteOffset + right.byteLength;
      if (left.byteOffset < rightEnd && right.byteOffset < leftEnd) return true;
    }
  }
  return false;
}

class TranspositionTable {
  public keyLo: Int32Array = EMPTY_TT_ARRAY;
  public keyHi: Int32Array = EMPTY_TT_ARRAY;
  public score: Int32Array = EMPTY_TT_ARRAY;
  public info: Int32Array = EMPTY_TT_ARRAY;
  public move: Int32Array = EMPTY_TT_ARRAY;
  public entries = 0;
  public generation = 1;
  public mask = 0;
  readonly #allocate: NonNullable<
    FastSearcherOptions["allocateTranspositionPart"]
  >;
  readonly #capacity: number;

  public constructor(
    allocate: NonNullable<FastSearcherOptions["allocateTranspositionPart"]>,
    capacity: number,
  ) {
    this.#allocate = allocate;
    this.#capacity = capacity;
  }

  public deactivate(): void {
    this.mask = 0;
  }

  public prepare(checkTimeout: () => boolean): boolean {
    if (this.keyLo.length !== 0) {
      this.mask = this.#capacity - 1;
      return true;
    }
    if (checkTimeout()) return false;
    const keyLo = tryAllocateTranspositionPart(
      this.#allocate,
      "keyLo",
      this.#capacity,
    );
    if (keyLo === undefined || checkTimeout()) return false;
    const keyHi = tryAllocateTranspositionPart(
      this.#allocate,
      "keyHi",
      this.#capacity,
    );
    if (keyHi === undefined || checkTimeout()) return false;
    const score = tryAllocateTranspositionPart(
      this.#allocate,
      "score",
      this.#capacity,
    );
    if (score === undefined || checkTimeout()) return false;
    const info = tryAllocateTranspositionPart(
      this.#allocate,
      "info",
      this.#capacity,
    );
    if (info === undefined || checkTimeout()) return false;
    const move = tryAllocateTranspositionPart(
      this.#allocate,
      "move",
      this.#capacity,
    );
    if (move === undefined || checkTimeout()) return false;
    if (transpositionPartsOverlap([keyLo, keyHi, score, info, move])) {
      return false;
    }
    this.keyLo = keyLo;
    this.keyHi = keyHi;
    this.score = score;
    this.info = info;
    this.move = move;
    this.entries = 0;
    this.generation = 1;
    this.mask = this.#capacity - 1;
    return true;
  }

  public clear(): void {
    this.keyLo.fill(0);
    this.keyHi.fill(0);
    this.score.fill(0);
    this.info.fill(0);
    this.move.fill(0);
    this.entries = 0;
    this.generation = 1;
  }
}

export class FastSearcher {
  readonly #positions: FastPosition[] = [];
  readonly #moves: Int32Array[] = [];
  readonly #orderKeys: Int32Array[] = [];
  readonly #table: TranspositionTable;
  readonly #killers = new Int32Array(MAX_PLY * 2);
  readonly #history = new Int32Array(COLOR_COUNT * BOARD_CELLS * BOARD_CELLS);
  #weights: EvalWeights = DEFAULT_WEIGHTS;
  #tuning: SearchTuning = DEFAULT_TUNING;
  #nodes = 0;
  #nodeLimit = 0;
  #stopped = false;
  #unsupported = false;
  #selectiveEpoch = 0;
  #checkTimeout: () => boolean = NO_TIMEOUT;

  public constructor(options: FastSearcherOptions = {}) {
    this.#table = new TranspositionTable(
      options.allocateTranspositionPart ?? defaultTranspositionAllocator,
      normalizeTranspositionCapacity(options.transpositionCapacity),
    );
    for (let ply = 0; ply <= MAX_PLY; ply += 1) {
      this.#positions.push(new FastPosition());
      this.#moves.push(new Int32Array(MAX_MOVES));
      this.#orderKeys.push(new Int32Array(MAX_MOVES));
    }
  }

  public get root(): FastPosition {
    return at(this.#positions, 0);
  }

  public get size(): number {
    return this.#table.entries;
  }

  public search(
    limits: SearchLimits,
    checkTimeout: () => boolean,
    weights: EvalWeights = DEFAULT_WEIGHTS,
  ): SearchOutcome {
    const normalizedLimits = normalizeSearchLimits(limits);
    const normalizedWeights = normalizeEvalWeights(weights);
    this.#unsupported = false;
    this.#selectiveEpoch = 0;
    if (positionWinner(this.root) !== -1) {
      return {
        move: 0,
        score: 0,
        depth: 0,
        nodes: 0,
        supported: true,
      };
    }

    this.#weights = normalizedWeights;
    this.#tuning = normalizedLimits.tuning;
    this.#nodes = 0;
    this.#nodeLimit = normalizedLimits.maxNodes;
    this.#stopped = false;
    this.#checkTimeout = checkTimeout;
    this.#table.deactivate();
    try {
      this.#killers.fill(0);
      this.#history.fill(0);

      let bestMove = 0;
      let bestScore = 0;
      let completedDepth = 0;
      for (let depth = 1; depth <= normalizedLimits.maxDepth; depth += 1) {
        if (depth === 2) {
          if (
            this.#nodes >= this.#nodeLimit ||
            !this.#table.prepare(this.#checkTimeout)
          ) {
            this.#stopped = true;
            break;
          }
          this.#advanceTableGeneration();
        }
        const outcome = this.#searchRoot(depth, bestMove);
        if (outcome.move === 0) break;
        if (this.#isStopped()) {
          if (completedDepth === 0) {
            bestMove = outcome.move;
            bestScore = outcome.score;
          }
          break;
        }
        bestMove = outcome.move;
        bestScore = outcome.score;
        completedDepth = depth;
        if (!outcome.selective && bestScore >= WIN_VALUE - MAX_PLY) break;
      }
      return {
        move: bestMove,
        score: bestScore,
        depth: completedDepth,
        nodes: this.#nodes,
        supported: !this.#unsupported,
      };
    } finally {
      this.#table.deactivate();
      this.#weights = DEFAULT_WEIGHTS;
      this.#tuning = DEFAULT_TUNING;
      this.#checkTimeout = NO_TIMEOUT;
    }
  }

  #advanceTableGeneration(): void {
    this.#table.generation += 1;
    if (this.#table.generation > TABLE_GENERATION_MASK) this.#table.clear();
  }

  #searchRoot(depth: number, previousBest: number): RootSearchOutcome {
    const position = at(this.#positions, 0);
    const buffer = at(this.#moves, 0);
    const count = generateMoves(position, buffer);
    if (count === 0) {
      return {
        move: 0,
        score: 0,
        selective: false,
      };
    }
    this.#orderMoves(position, buffer, count, 0, previousBest);
    const rootKeys = at(this.#orderKeys, 0);

    let alpha = -INFINITY_SCORE;
    let bestMove = 0;
    let bestScore = -INFINITY_SCORE;
    let bestSelective = false;
    for (let index = 0; index < count; index += 1) {
      this.#selectBest(buffer, rootKeys, index, count);
      const move = i32(buffer, index);
      let score: number;
      let selectiveEpoch = this.#selectiveEpoch;
      let selective: boolean;
      if (index === 0) {
        score = this.#child(
          position,
          move,
          0,
          depth - 1,
          alpha,
          INFINITY_SCORE,
        );
        selective = this.#selectiveEpoch !== selectiveEpoch;
      } else {
        score = this.#child(position, move, 0, depth - 1, alpha, alpha + 1);
        selective = this.#selectiveEpoch !== selectiveEpoch;
        if (score > alpha && !this.#isStopped()) {
          selectiveEpoch = this.#selectiveEpoch;
          score = this.#child(
            position,
            move,
            0,
            depth - 1,
            alpha,
            INFINITY_SCORE,
          );
          selective = this.#selectiveEpoch !== selectiveEpoch;
        }
      }
      if (this.#isStopped()) break;
      if (score > bestScore) {
        bestScore = score;
        bestMove = move;
        bestSelective = selective;
        if (score > alpha) alpha = score;
      }
    }
    if (bestMove === 0) {
      bestMove = i32(buffer, 0);
      bestScore = 0;
      bestSelective = true;
    }
    return {
      move: bestMove,
      score: bestScore,
      selective: bestSelective,
    };
  }

  #child(
    parent: FastPosition,
    move: number,
    ply: number,
    depth: number,
    alpha: number,
    beta: number,
  ): number {
    const child = at(this.#positions, ply + 1);
    child.copyFrom(parent);
    const winner = applyFastMove(child, move);
    if (winner === FAST_MOVE_UNREPRESENTABLE) {
      this.#unsupported = true;
      this.#stopped = true;
      return 0;
    }
    if (winner !== -1) {
      return winner === parent.active ? WIN_VALUE - ply : -(WIN_VALUE - ply);
    }
    if (child.active === parent.active) {
      return this.#negamax(depth, ply + 1, alpha, beta);
    }
    return -this.#negamax(depth, ply + 1, -beta, -alpha);
  }

  #negamax(
    depth: number,
    ply: number,
    alphaInput: number,
    beta: number,
  ): number {
    if (!this.#beginWorkUnit()) return 0;
    const position = at(this.#positions, ply);
    if (ply >= MAX_PLY - 2) return this.#staticScore(position);
    if (depth <= 0) return this.#staticScore(position);

    let alpha = alphaInput;
    const scalar = scalarIndex(position);
    const keyLo = stateKeyLo(position, scalar);
    const keyHi = stateKeyHi(position, scalar);
    const table = this.#table;
    const slot = (keyLo ^ (keyHi * 3)) & table.mask;
    let ttMove = 0;
    const slotInfo = i32(table.info, slot);
    const slotMatches =
      i32(table.keyLo, slot) === keyLo &&
      i32(table.keyHi, slot) === keyHi &&
      ((slotInfo >>> 8) & TABLE_GENERATION_MASK) === table.generation;
    if (slotMatches) {
      ttMove = i32(table.move, slot);
      const storedDepth = (slotInfo >> 2) & 63;
      if (storedDepth >= depth) {
        const score = i32(table.score, slot);
        const flag = slotInfo & 3;
        if (
          flag === FLAG_EXACT ||
          (flag === FLAG_LOWER && score >= beta) ||
          (flag === FLAG_UPPER && score <= alpha)
        ) {
          if ((slotInfo & INFO_SELECTIVE) !== 0) this.#selectiveEpoch += 1;
          return score;
        }
      }
    }

    const buffer = at(this.#moves, ply);
    const count = generateMoves(position, buffer);
    if (count === 0) return this.#staticScore(position);
    this.#orderMoves(position, buffer, count, ply, ttMove);
    const keys = at(this.#orderKeys, ply);

    const tuning = this.#tuning;
    let moveLimit = count;
    if (tuning.moveCountPruning && depth <= tuning.moveCountDepth) {
      const allowed = tuning.moveCountBase + tuning.moveCountFactor * depth;
      if (allowed < moveLimit) moveLimit = allowed;
    }
    const futile =
      depth <= 2 &&
      this.#staticScore(position) + tuning.futilityMargin * depth <= alpha;
    if (this.#isStopped()) return 0;

    let bestScore = -INFINITY_SCORE;
    let bestMove = 0;
    let selectivelyPruned = false;
    const selectiveEpoch = this.#selectiveEpoch;
    for (let index = 0; index < count; index += 1) {
      this.#selectBest(buffer, keys, index, count);
      const move = i32(buffer, index);
      const tactical = i32(keys, index) >= TACTICAL_THRESHOLD;
      if (!tactical && bestMove !== 0 && (index >= moveLimit || futile)) {
        selectivelyPruned = true;
        this.#selectiveEpoch += 1;
        break;
      }

      let score: number;
      if (index === 0) {
        score = this.#child(position, move, ply, depth - 1, alpha, beta);
      } else {
        let reduction = 0;
        if (
          tuning.lateMoveReduction &&
          !tactical &&
          depth >= 3 &&
          index >= tuning.lateMoveIndex
        ) {
          reduction = index >= tuning.lateMoveDeepIndex ? 2 : 1;
          if (reduction >= depth) reduction = depth - 1;
        }
        score = this.#child(
          position,
          move,
          ply,
          depth - 1 - reduction,
          alpha,
          alpha + 1,
        );
        if (
          !this.#isStopped() &&
          score > alpha &&
          (reduction > 0 || score < beta)
        ) {
          score = this.#child(position, move, ply, depth - 1, alpha, beta);
        }
      }
      if (this.#isStopped()) return 0;
      if (score > bestScore) {
        bestScore = score;
        bestMove = move;
      }
      if (score > alpha) alpha = score;
      if (alpha >= beta) {
        this.#recordCutoff(position, move, ply, depth, tactical);
        break;
      }
    }

    if (bestMove === 0) return this.#staticScore(position);

    const subtreeSelective = this.#selectiveEpoch !== selectiveEpoch;
    const flag = selectivelyPruned
      ? FLAG_MOVE_ONLY
      : bestScore >= beta
        ? FLAG_LOWER
        : bestScore <= alphaInput
          ? FLAG_UPPER
          : FLAG_EXACT;
    const entryDepth = selectivelyPruned ? 0 : depth;
    if (bestScore < WIN_VALUE - MAX_PLY && bestScore > -WIN_VALUE + MAX_PLY) {
      const storedDepth = (slotInfo >> 2) & 63;
      if (!slotMatches || entryDepth >= storedDepth) {
        if (slotInfo === 0 && i32(table.info, slot) === 0) {
          table.entries += 1;
        }
        table.keyLo[slot] = keyLo;
        table.keyHi[slot] = keyHi;
        table.score[slot] = bestScore;
        table.info[slot] =
          (table.generation << 8) |
          (entryDepth << 2) |
          flag |
          (subtreeSelective ? INFO_SELECTIVE : 0);
        table.move[slot] = bestMove;
      }
    }
    return bestScore;
  }

  #isStopped(): boolean {
    return this.#stopped;
  }

  #beginWorkUnit(): boolean {
    if (this.#stopped) return false;
    if (this.#nodes >= this.#nodeLimit) {
      this.#stopped = true;
      return false;
    }
    this.#nodes += 1;
    if ((this.#nodes & TIMEOUT_CHECK_MASK) === 0 && this.#checkTimeout()) {
      this.#stopped = true;
      return false;
    }
    return true;
  }

  #staticScore(position: FastPosition): number {
    if (!this.#beginWorkUnit()) return 0;
    const value = evaluatePosition(position, this.#weights);
    const score = position.active === 0 ? value : -value;
    if (score > MAX_NONTERMINAL_SCORE) return MAX_NONTERMINAL_SCORE;
    if (score < -MAX_NONTERMINAL_SCORE) return -MAX_NONTERMINAL_SCORE;
    return score;
  }

  #recordCutoff(
    position: FastPosition,
    move: number,
    ply: number,
    depth: number,
    tactical: boolean,
  ): void {
    if (tactical) return;
    const killerIndex = ply * 2;
    if (i32(this.#killers, killerIndex) !== move) {
      this.#killers[killerIndex + 1] = i32(this.#killers, killerIndex);
      this.#killers[killerIndex] = move;
    }
    const historyIndex =
      position.active * BOARD_CELLS * BOARD_CELLS +
      moveFrom(move) * BOARD_CELLS +
      moveTo(move);
    const updated = i32(this.#history, historyIndex) + depth * depth;
    this.#history[historyIndex] = updated > 1 << 20 ? 1 << 20 : updated;
  }

  #orderMoves(
    position: FastPosition,
    buffer: Int32Array,
    count: number,
    ply: number,
    ttMove: number,
  ): void {
    const keys = at(this.#orderKeys, ply);
    const killerA = i32(this.#killers, ply * 2);
    const killerB = i32(this.#killers, ply * 2 + 1);
    const historyBase = position.active * BOARD_CELLS * BOARD_CELLS;
    for (let index = 0; index < count; index += 1) {
      const move = i32(buffer, index);
      let key = moveHeuristic(position, move);
      if (move === ttMove) key += KEY_TT;
      else if (move === killerA) key += KEY_KILLER_A;
      else if (move === killerB) key += KEY_KILLER_B;
      else {
        const history = i32(
          this.#history,
          historyBase + moveFrom(move) * BOARD_CELLS + moveTo(move),
        );
        key += history > 8191 ? 8191 : history;
      }
      keys[index] = key;
    }
  }

  #selectBest(
    buffer: Int32Array,
    keys: Int32Array,
    index: number,
    count: number,
  ): void {
    let bestSlot = index;
    let bestKey = i32(keys, index);
    for (let slot = index + 1; slot < count; slot += 1) {
      const key = i32(keys, slot);
      if (key > bestKey) {
        bestKey = key;
        bestSlot = slot;
      }
    }
    if (bestSlot === index) return;
    const move = i32(buffer, index);
    buffer[index] = i32(buffer, bestSlot);
    buffer[bestSlot] = move;
    keys[bestSlot] = i32(keys, index);
    keys[index] = bestKey;
  }
}

function moveHeuristic(position: FastPosition, move: number): number {
  const type = moveType(move);
  const from = moveFrom(move);
  const to = moveTo(move);
  switch (type) {
    case MOVE_MON: {
      const start = u16(position.cells, from);
      const carried = cellMana(start);
      let key = 0;
      if (carried !== 0 && squareIsPool(u8(position.squares, to))) {
        key += (1 << 21) + manaScoreValue(carried, position.active) * (1 << 18);
      }
      const target = u16(position.cells, to);
      if (target !== 0) {
        const occupancy = cellOccupancy(target);
        if (occupancy === OCC_MANA) {
          key +=
            (1 << 16) + manaScoreValue(cellMana(target), position.active) * 512;
        } else if (occupancy === OCC_CONS) {
          key += 1 << 14;
        }
      }
      if (carried !== 0) {
        const before = u8(POOL_DISTANCE, from);
        const after = u8(POOL_DISTANCE, to);
        key += (before - after) * 2048 + 4096;
      } else if (cellMonKind(start) === KIND_DRAINER) {
        key += 1024;
      }
      return key;
    }
    case MOVE_MANA:
      return squareIsPool(u8(position.squares, to))
        ? (1 << 21) + (1 << 18)
        : 16;
    case MOVE_MYSTIC:
    case MOVE_DEMON:
    case MOVE_BOMB: {
      const target = u16(position.cells, to);
      let key = 1 << 17;
      if (cellMana(target) !== 0) key += 1 << 20;
      if (cellMonKind(target) === KIND_DRAINER) key += 1 << 19;
      return key;
    }
    case MOVE_SPIRIT: {
      const target = u16(position.cells, to);
      const aux = moveAux(move);
      let key = 1 << 13;
      const mana = cellMana(target);
      if (mana !== 0 && squareIsPool(u8(position.squares, aux))) {
        key += (1 << 21) + manaScoreValue(mana, position.active) * (1 << 18);
      } else if (cellOccupancy(target) === OCC_MANA) {
        key += 1 << 15;
      }
      return key;
    }
    default:
      throw new RangeError(`unsupported fast move type: ${type}`);
  }
}

export function scalarIndex(position: FastPosition): number {
  const whitePotions = i32(position.potions, 0);
  const blackPotions = i32(position.potions, 1);
  const whitePotionBucket = whitePotions > 2 ? 3 : whitePotions;
  const blackPotionBucket = blackPotions > 2 ? 3 : blackPotions;
  let scalar = blackPotionBucket;
  scalar = whitePotionBucket + POTION_BUCKET_STATES * scalar;
  scalar = (position.firstTurn ? 1 : 0) + FIRST_TURN_STATES * scalar;
  scalar = position.actionsUsed + ACTION_STATES * scalar;
  scalar = position.manaMoves + MANA_MOVE_STATES * scalar;
  scalar = position.monsMoves + MONS_MOVE_STATES * scalar;
  return position.active + ACTIVE_STATES * scalar;
}

export function stateKeyLo(position: FastPosition, scalar: number): number {
  return (
    position.hashLo ^
    i32(Z_SCALAR_LO, scalar) ^
    (position.whiteScore * 0x9e3779b1) ^
    (position.blackScore * 0x7f4a7c15) ^
    i32(position.potions, 0)
  );
}

export function stateKeyHi(position: FastPosition, scalar: number): number {
  return (
    position.hashHi ^
    i32(Z_SCALAR_HI, scalar) ^
    (position.whiteScore * 0x85ebca6b) ^
    (position.blackScore * 0xc2b2ae35) ^
    i32(position.potions, 1)
  );
}
