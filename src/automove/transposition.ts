import { rethrowFastWorkspaceAllocation } from "./allocation.js";

const DEFAULT_TRANSPOSITION_CAPACITY = 1 << 20;
const EMPTY_TT_ARRAY: Int32Array = new Int32Array(0);

function allocateTranspositionPart(length: number): Int32Array {
  try {
    return new Int32Array(length);
  } catch (error) {
    rethrowFastWorkspaceAllocation(error);
  }
}

export class TranspositionTable {
  public keyLo: Int32Array = EMPTY_TT_ARRAY;
  public keyHi: Int32Array = EMPTY_TT_ARRAY;
  public score: Int32Array = EMPTY_TT_ARRAY;
  public info: Int32Array = EMPTY_TT_ARRAY;
  public move: Int32Array = EMPTY_TT_ARRAY;
  public entries = 0;
  public generation = 1;
  public mask = 0;

  public deactivate(): void {
    this.mask = 0;
  }

  public prepare(checkTimeout: () => boolean): boolean {
    if (this.keyLo.length !== 0) {
      this.mask = DEFAULT_TRANSPOSITION_CAPACITY - 1;
      return true;
    }
    if (checkTimeout()) return false;
    const keyLo = allocateTranspositionPart(DEFAULT_TRANSPOSITION_CAPACITY);
    if (checkTimeout()) return false;
    const keyHi = allocateTranspositionPart(DEFAULT_TRANSPOSITION_CAPACITY);
    if (checkTimeout()) return false;
    const score = allocateTranspositionPart(DEFAULT_TRANSPOSITION_CAPACITY);
    if (checkTimeout()) return false;
    const info = allocateTranspositionPart(DEFAULT_TRANSPOSITION_CAPACITY);
    if (checkTimeout()) return false;
    const move = allocateTranspositionPart(DEFAULT_TRANSPOSITION_CAPACITY);
    if (checkTimeout()) return false;
    this.keyLo = keyLo;
    this.keyHi = keyHi;
    this.score = score;
    this.info = info;
    this.move = move;
    this.entries = 0;
    this.generation = 1;
    this.mask = DEFAULT_TRANSPOSITION_CAPACITY - 1;
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
