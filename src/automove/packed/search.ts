import { BOARD_CELLS } from "../../engine/board/geometry.js";
import {
  ACTIONS_PER_TURN,
  MANA_MOVES_PER_TURN,
  MONS_MOVES_PER_TURN,
} from "../../engine/board/config.js";
import { COLOR_COUNT, Z_SCALAR_HI, Z_SCALAR_LO, at, i32, u8 } from "./board.js";
import { WIN_VALUE, evaluateWithTables } from "./evaluation.js";
import {
  DEFAULT_EVAL_TABLES,
  DEFAULT_WEIGHTS,
  memoizedEvalTables,
  memoizedNormalizedEvalWeights,
  type EvalTables,
  type EvalWeights,
} from "./evaluation-weights.js";
import { MAX_MOVES, generateMoves } from "./moves.js";
import {
  FastPosition,
  FAST_MOVE_UNREPRESENTABLE,
  MOVE_MON,
  applyFastMove,
  moveFrom,
  moveTo,
  moveType,
  positionWinner,
} from "./state.js";
import {
  DEFAULT_TUNING,
  MAX_NONTERMINAL_SCORE,
  MAX_PLY,
  memoizedSearchLimits,
  type NormalizedSearchTuning,
  type SearchLimits,
} from "./search-tuning.js";
import { TranspositionTable } from "./transposition.js";

const EVAL_CACHE_BITS = 16;
const EVAL_CACHE_ENTRIES = 1 << EVAL_CACHE_BITS;
const EVAL_CACHE_MASK = EVAL_CACHE_ENTRIES - 1;
const EVAL_CACHE_EPOCH_LIMIT = 1 << 30;
const FLAG_EXACT = 0;
const FLAG_LOWER = 1;
const FLAG_UPPER = 2;
const FLAG_MOVE_ONLY = 3;
const TABLE_GENERATION_MASK = 0x3f_ffff;
const TIMEOUT_CHECK_MASK = 511;
const TACTICAL_THRESHOLD = 1 << 15;
const KEY_TT = 1 << 28;
const KEY_KILLER_A = 1 << 24;
const KEY_KILLER_B = 1 << 23;
const INFINITY_SCORE = WIN_VALUE * 2;
const NO_TIMEOUT = (): boolean => false;
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

if (
  Z_SCALAR_LO.length !== Z_SCALAR_HI.length ||
  Z_SCALAR_LO.length < SCALAR_STATE_COUNT
) {
  throw new RangeError(
    "fast scalar hash table is too small for the configured state space",
  );
}

function sameSquares(
  previous: ArrayLike<number> | undefined,
  current: ArrayLike<number>,
): boolean {
  if (previous === current) return true;
  if (previous?.length !== current.length) return false;
  for (let index = 0; index < current.length; index += 1) {
    if (u8(previous, index) !== u8(current, index)) return false;
  }
  return true;
}

type SearchOutcome = {
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

export class FastSearcher {
  readonly #positions: FastPosition[] = [];
  readonly #moves: Int32Array[] = [];
  readonly #orderKeys: Int32Array[] = [];
  readonly #table: TranspositionTable;
  readonly #killers = new Int32Array(MAX_PLY * 2);
  readonly #history = new Int32Array(COLOR_COUNT * BOARD_CELLS * BOARD_CELLS);
  readonly #moveAtPly = new Int32Array(MAX_PLY + 1);
  readonly #evalKeyLo = new Int32Array(EVAL_CACHE_ENTRIES);
  readonly #evalKeyHi = new Int32Array(EVAL_CACHE_ENTRIES);
  readonly #evalEpochs = new Int32Array(EVAL_CACHE_ENTRIES);
  readonly #evalValues = new Float64Array(EVAL_CACHE_ENTRIES);
  #evalEpoch = 0;
  #evalCacheWeights: EvalWeights | undefined;
  #evalCacheThreat = 0;
  #evalCacheSquares: ArrayLike<number> | undefined;
  #tables: EvalTables = DEFAULT_EVAL_TABLES;
  #tuning: NormalizedSearchTuning = DEFAULT_TUNING;
  #windowThreat = 0;
  #nodes = 0;
  #nodeLimit = 0;
  #stopped = false;
  #unsupported = false;
  #selectiveEpoch = 0;
  #checkTimeout: () => boolean = NO_TIMEOUT;

  public constructor() {
    this.#table = new TranspositionTable();
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
    const normalizedLimits = memoizedSearchLimits(limits);
    const normalizedWeights = memoizedNormalizedEvalWeights(weights);
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

    this.#tables = memoizedEvalTables(normalizedWeights);
    const tuning = normalizedLimits.tuning;
    this.#tuning = tuning;
    this.#windowThreat = tuning.winsNextTurnThreat;
    // Cached evaluations stay valid across searches: the cache key covers the whole
    // position, so entries only go stale when the weights, the tuning's evaluation
    // term, or the variant's static square layout change.
    const squaresChanged = !sameSquares(this.#evalCacheSquares, this.root.squares);
    if (
      normalizedWeights !== this.#evalCacheWeights ||
      tuning.winsNextTurnThreat !== this.#evalCacheThreat ||
      squaresChanged
    ) {
      this.#evalCacheWeights = normalizedWeights;
      this.#evalCacheThreat = tuning.winsNextTurnThreat;
      this.#evalCacheSquares = this.root.squares;
      if (this.#evalEpoch >= EVAL_CACHE_EPOCH_LIMIT) {
        this.#evalEpoch = 0;
        this.#evalEpochs.fill(0);
      }
      this.#evalEpoch += 1;
    }
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
        let alphaStart = -INFINITY_SCORE;
        let betaStart = INFINITY_SCORE;
        if (
          tuning.aspirationDelta > 0 &&
          completedDepth > 0 &&
          depth >= tuning.aspirationMinDepth
        ) {
          alphaStart = bestScore - tuning.aspirationDelta;
          betaStart = bestScore + tuning.aspirationDelta;
        }
        let outcome = this.#searchRoot(depth, bestMove, alphaStart, betaStart);
        let widenings = 0;
        while (
          !this.#isStopped() &&
          outcome.move !== 0 &&
          (outcome.score <= alphaStart || outcome.score >= betaStart) &&
          (alphaStart > -INFINITY_SCORE || betaStart < INFINITY_SCORE)
        ) {
          widenings += 1;
          const widened = tuning.aspirationDelta * (1 << widenings);
          if (outcome.score <= alphaStart) {
            alphaStart = widenings >= 2 ? -INFINITY_SCORE : outcome.score - widened;
          }
          if (outcome.score >= betaStart) {
            betaStart = widenings >= 2 ? INFINITY_SCORE : outcome.score + widened;
          }
          outcome = this.#searchRoot(depth, outcome.move, alphaStart, betaStart);
        }
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
      this.#tables = DEFAULT_EVAL_TABLES;
      this.#tuning = DEFAULT_TUNING;
      this.#windowThreat = 0;
      this.#checkTimeout = NO_TIMEOUT;
    }
  }

  #advanceTableGeneration(): void {
    this.#table.generation += 1;
    if (this.#table.generation > TABLE_GENERATION_MASK) this.#table.clear();
  }

  #searchRoot(
    depth: number,
    previousBest: number,
    alphaStart: number,
    betaStart: number,
  ): RootSearchOutcome {
    const position = at(this.#positions, 0);
    const buffer = at(this.#moves, 0);
    const count = generateMoves(position, buffer, at(this.#orderKeys, 0));
    if (count === 0) {
      return {
        move: 0,
        score: 0,
        selective: false,
      };
    }
    this.#orderMoves(position, buffer, count, 0, previousBest);
    const rootKeys = at(this.#orderKeys, 0);

    let alpha = alphaStart;
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
        score = this.#child(position, move, 0, depth - 1, alpha, betaStart);
        selective = this.#selectiveEpoch !== selectiveEpoch;
      } else {
        score = this.#child(position, move, 0, depth - 1, alpha, alpha + 1);
        selective = this.#selectiveEpoch !== selectiveEpoch;
        if (score > alpha && !this.#isStopped()) {
          selectiveEpoch = this.#selectiveEpoch;
          score = this.#child(position, move, 0, depth - 1, alpha, betaStart);
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
      if (alpha >= betaStart) break;
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
    this.#moveAtPly[ply] = move;
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

  #negamax(depth: number, ply: number, alphaInput: number, beta: number): number {
    if (!this.#beginWorkUnit()) return 0;
    const position = at(this.#positions, ply);
    if (ply >= MAX_PLY - 2) return this.#staticScore(position);
    if (depth <= 0) return this.#staticScore(position);

    let alpha = alphaInput;
    const commutingMove = this.#commutingMonMoveContext(position, ply);
    const scalar = scalarIndex(position);
    const keyLo = stateKeyLo(position, scalar, commutingMove);
    const keyHi = stateKeyHi(position, scalar, commutingMove);
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
          return score;
        }
      }
    }

    const tuning = this.#tuning;
    const buffer = at(this.#moves, ply);
    let count = generateMoves(position, buffer, at(this.#orderKeys, ply));
    if (count === 0) return this.#staticScore(position, keyLo, keyHi);
    if (commutingMove !== 0) {
      count = this.#filterCommutingMonMoves(buffer, count, commutingMove, ply);
    }
    this.#orderMoves(position, buffer, count, ply, ttMove);
    const keys = at(this.#orderKeys, ply);

    let moveLimit = count;
    if (tuning.moveCountPruning && depth <= tuning.moveCountDepth) {
      const allowed = tuning.moveCountBase + tuning.moveCountFactor * depth;
      if (allowed < moveLimit) moveLimit = allowed;
    }
    const futilityEnabled = depth <= 2;
    if (futilityEnabled && !this.#beginWorkUnit()) return 0;
    if (this.#isStopped()) return 0;
    let futilityKnown = false;
    let futile = false;

    let bestScore = -INFINITY_SCORE;
    let bestMove = 0;
    const selectiveEpoch = this.#selectiveEpoch;
    for (let index = 0; index < count; index += 1) {
      this.#selectBest(buffer, keys, index, count);
      const move = i32(buffer, index);
      const tactical = i32(keys, index) >= TACTICAL_THRESHOLD;
      if (!tactical && bestMove !== 0) {
        let pruned = index >= moveLimit;
        if (!pruned && futilityEnabled) {
          if (!futilityKnown) {
            futile =
              this.#evaluateStatic(position, keyLo, keyHi) +
                tuning.futilityMargin * depth <=
              alphaInput;
            futilityKnown = true;
          }
          pruned = futile;
        }
        if (pruned) {
          this.#selectiveEpoch += 1;
          break;
        }
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
        if (!this.#isStopped() && score > alpha && (reduction > 0 || score < beta)) {
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
    const flag = subtreeSelective
      ? FLAG_MOVE_ONLY
      : bestScore >= beta
        ? FLAG_LOWER
        : bestScore <= alphaInput
          ? FLAG_UPPER
          : FLAG_EXACT;
    const storedScore = bestScore;
    const entryDepth = depth;
    if (storedScore < WIN_VALUE - MAX_PLY && storedScore > -WIN_VALUE + MAX_PLY) {
      const storedDepth = (slotInfo >> 2) & 63;
      if (!slotMatches || entryDepth >= storedDepth) {
        if (slotInfo === 0 && i32(table.info, slot) === 0) {
          table.entries += 1;
        }
        table.keyLo[slot] = keyLo;
        table.keyHi[slot] = keyHi;
        table.score[slot] = storedScore;
        table.info[slot] = (table.generation << 8) | (entryDepth << 2) | flag;
        table.move[slot] = bestMove;
      }
    }
    return bestScore;
  }

  // Mon sub-moves on four distinct squares commute exactly, so every permutation of a
  // commuting chain reaches the same position; keeping only the ascending order removes
  // the duplicates at the source while every set of moves stays reachable.
  #commutingMonMoveContext(position: FastPosition, ply: number): number {
    if (ply === 0) return 0;
    const previous = i32(this.#moveAtPly, ply - 1);
    return moveType(previous) === MOVE_MON &&
      at(this.#positions, ply - 1).active === position.active
      ? previous
      : 0;
  }

  #filterCommutingMonMoves(
    buffer: Int32Array,
    count: number,
    previous: number,
    ply: number,
  ): number {
    const previousFrom = moveFrom(previous);
    const previousTo = moveTo(previous);
    const keys = at(this.#orderKeys, ply);
    let write = 0;
    for (let read = 0; read < count; read += 1) {
      const move = i32(buffer, read);
      if (moveType(move) === MOVE_MON && move < previous) {
        const from = moveFrom(move);
        const to = moveTo(move);
        if (
          from !== previousFrom &&
          from !== previousTo &&
          to !== previousFrom &&
          to !== previousTo
        ) {
          continue;
        }
      }
      buffer[write] = move;
      keys[write] = i32(keys, read);
      write += 1;
    }
    return write === 0 ? count : write;
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

  #evaluateStatic(
    position: FastPosition,
    precomputedKeyLo?: number,
    precomputedKeyHi?: number,
  ): number {
    let keyLo = precomputedKeyLo;
    let keyHi = precomputedKeyHi;
    if (keyLo === undefined || keyHi === undefined) {
      const scalar = scalarIndex(position);
      keyLo = stateKeyLo(position, scalar);
      keyHi = stateKeyHi(position, scalar);
    }
    const slot = (keyLo ^ (keyHi * 3)) & EVAL_CACHE_MASK;
    let value: number;
    if (
      i32(this.#evalEpochs, slot) === this.#evalEpoch &&
      i32(this.#evalKeyLo, slot) === keyLo &&
      i32(this.#evalKeyHi, slot) === keyHi
    ) {
      value = this.#evalValues[slot] ?? 0;
    } else {
      value = evaluateWithTables(position, this.#tables, this.#windowThreat);
      this.#evalKeyLo[slot] = keyLo;
      this.#evalKeyHi[slot] = keyHi;
      this.#evalValues[slot] = value;
      this.#evalEpochs[slot] = this.#evalEpoch;
    }
    const score = position.active === 0 ? value : -value;
    if (score > MAX_NONTERMINAL_SCORE) return MAX_NONTERMINAL_SCORE;
    if (score < -MAX_NONTERMINAL_SCORE) return -MAX_NONTERMINAL_SCORE;
    return score;
  }

  #staticScore(
    position: FastPosition,
    precomputedKeyLo?: number,
    precomputedKeyHi?: number,
  ): number {
    if (!this.#beginWorkUnit()) return 0;
    return this.#evaluateStatic(position, precomputedKeyLo, precomputedKeyHi);
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
      let key = i32(keys, index);
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

function scalarIndex(position: FastPosition): number {
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

function stateKeyLo(position: FastPosition, scalar: number, commutingMove = 0): number {
  return (
    position.hashLo ^
    i32(Z_SCALAR_LO, scalar) ^
    (position.whiteScore * 0x9e3779b1) ^
    (position.blackScore * 0x7f4a7c15) ^
    i32(position.potions, 0) ^
    Math.imul(commutingMove, 0x27d4eb2d)
  );
}

function stateKeyHi(position: FastPosition, scalar: number, commutingMove = 0): number {
  return (
    position.hashHi ^
    i32(Z_SCALAR_HI, scalar) ^
    (position.whiteScore * 0x85ebca6b) ^
    (position.blackScore * 0xc2b2ae35) ^
    i32(position.potions, 1) ^
    Math.imul(commutingMove, 0x165667b1)
  );
}
