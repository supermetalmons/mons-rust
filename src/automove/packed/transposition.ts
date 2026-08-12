const DEFAULT_TRANSPOSITION_CAPACITY = 1 << 20;
const EMPTY_TT_ARRAY: Int32Array = new Int32Array(0);

function tryAllocateTranspositionPart(length: number): Int32Array | undefined {
  try {
    return new Int32Array(length);
  } catch (error) {
    if (error instanceof RangeError) return undefined;
    throw error;
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
    const keyLo = tryAllocateTranspositionPart(DEFAULT_TRANSPOSITION_CAPACITY);
    if (keyLo === undefined || checkTimeout()) return false;
    const keyHi = tryAllocateTranspositionPart(DEFAULT_TRANSPOSITION_CAPACITY);
    if (keyHi === undefined || checkTimeout()) return false;
    const score = tryAllocateTranspositionPart(DEFAULT_TRANSPOSITION_CAPACITY);
    if (score === undefined || checkTimeout()) return false;
    const info = tryAllocateTranspositionPart(DEFAULT_TRANSPOSITION_CAPACITY);
    if (info === undefined || checkTimeout()) return false;
    const move = tryAllocateTranspositionPart(DEFAULT_TRANSPOSITION_CAPACITY);
    if (move === undefined || checkTimeout()) return false;
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
