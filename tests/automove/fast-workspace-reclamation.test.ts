import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { runInNewContext } from "node:vm";

import { describe, expect, it } from "vitest";

import { loadPosition } from "../../src/automove/fast/bridge.js";
import { WIN_VALUE } from "../../src/automove/fast/evaluate.js";
import {
  FastSearcherPool,
  selectFastInputs,
  selectFastSelection,
  type FastProfile,
} from "../../src/automove/fast/index.js";
import {
  FastSearcher,
  MAX_SEARCH_DEPTH,
  scalarIndex,
  stateKeyHi,
  stateKeyLo,
  type FastSearcherOptions,
  type SearchLimits,
  type SearchOutcome,
  type SearchTuning,
  type TranspositionPart,
} from "../../src/automove/fast/search.js";
import {
  FastPosition,
  applyFastMove,
} from "../../src/automove/fast/position.js";
import { GameVariant } from "../../src/engine/config.js";
import { MonsGame } from "../../src/engine/game.js";
import { createTestAutomoveExecutionContext } from "./execution-context.test-helper.js";
import { TINY_PROFILE } from "./fast.test-helper.js";

const CHILD_SCENARIO_ENV = "MONS_FAST_WORKSPACE_CHILD_SCENARIO";
const TT_PARTS = [
  "keyLo",
  "keyHi",
  "score",
  "info",
  "move",
] as const satisfies readonly TranspositionPart[];
const TT_MOVE_ONLY_FLAG = 3;
const TT_SELECTIVE_FLAG = 1 << 30;
const TT_GENERATION_MASK = 0x3f_ffff;
const LOW_CAPACITY = 2_048;
const TABLE_CAPACITY = 1 << 17;
const PRO_CAPACITY = 1 << 20;
const TABLE_PROFILE: FastProfile = Object.freeze({
  ...TINY_PROFILE,
  maxDepth: 2,
  maxNodes: 100_000,
});
const LOW_PROFILE: FastProfile = Object.freeze({
  ...TABLE_PROFILE,
  maxNodes: LOW_CAPACITY,
});
const PRO_TABLE_PROFILE: FastProfile = Object.freeze({
  ...TABLE_PROFILE,
  maxNodes: 2_000_000,
});
const childScenario = process.env[CHILD_SCENARIO_ENV];
const testFile = fileURLToPath(import.meta.url);
const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const vitestCli = fileURLToPath(
  new URL("../../node_modules/vitest/vitest.mjs", import.meta.url),
);

type AllocateTranspositionPart = NonNullable<
  FastSearcherOptions["allocateTranspositionPart"]
>;

type AllocationRecord = {
  readonly part: TranspositionPart;
  readonly length: number;
  array: Int32Array | undefined;
};

type AllocationTracker = {
  readonly records: AllocationRecord[];
  readonly allocate: AllocateTranspositionPart;
};

function createAllocationTracker(
  onAttempt?: (record: AllocationRecord, ordinal: number) => void,
): AllocationTracker {
  const records: AllocationRecord[] = [];
  return {
    records,
    allocate: (part, length) => {
      const record: AllocationRecord = {
        part,
        length,
        array: undefined,
      };
      records.push(record);
      onAttempt?.(record, records.length);
      const array = new Int32Array(length);
      record.array = array;
      return array;
    },
  };
}

function expectAllocationBatch(
  records: readonly AllocationRecord[],
  start: number,
  capacity: number,
): void {
  const batch = records.slice(start, start + TT_PARTS.length);
  expect(batch.map(({ part }) => part)).toEqual(TT_PARTS);
  expect(batch.map(({ length }) => length)).toEqual(
    TT_PARTS.map(() => capacity),
  );
  expect(
    batch.reduce((bytes, record) => bytes + (record.array?.byteLength ?? 0), 0),
  ).toBe(TT_PARTS.length * capacity * Int32Array.BYTES_PER_ELEMENT);
}

function retainedWeakReference(searcher: FastSearcher): {
  deref(): FastSearcher;
} {
  return {
    deref: () => searcher,
  };
}

function createPoolHarness(
  tracker: AllocationTracker,
  retain: boolean,
  transpositionCapacity = TABLE_CAPACITY,
): {
  readonly pool: FastSearcherPool;
  readonly creations: () => number;
} {
  let creations = 0;
  const pool = new FastSearcherPool({
    createSearcher: () => {
      creations += 1;
      return new FastSearcher({
        transpositionCapacity,
        allocateTranspositionPart: tracker.allocate,
      });
    },
    weakRefFactory: retain ? retainedWeakReference : false,
  });
  return {
    pool,
    creations: () => creations,
  };
}

function selectionContext(clock: () => number = () => 0) {
  const context = createTestAutomoveExecutionContext(clock);
  return {
    context,
    session: context.session,
  };
}

function expectApplicableSelection(
  selection: ReturnType<typeof selectFastSelection>,
  game: MonsGame,
): void {
  expect(selection.kind).toBe("supported");
  if (selection.kind !== "supported") return;
  expect(selection.inputs.length).toBeGreaterThan(0);
  expect(game.fork().processInput(selection.inputs, false, false).kind).toBe(
    "events",
  );
}

function selectOnce(
  pool: FastSearcherPool,
  profile: FastProfile = TABLE_PROFILE,
) {
  const { context } = selectionContext();
  const game = new MonsGame(true, GameVariant.Classic);
  const sourceFen = game.fen();
  const inputs = selectFastInputs(context, game, profile, pool);
  expect(inputs).toBeDefined();
  expect(inputs?.length).toBeGreaterThan(0);
  if (inputs !== undefined) {
    expect(game.fork().processInput(inputs, false, false).kind).toBe("events");
  }
  expect(game.fen()).toBe(sourceFen);
  return inputs ?? [];
}

function searchOpening(
  searcher: FastSearcher,
  limits: SearchLimits,
): SearchOutcome {
  loadPosition(searcher.root, new MonsGame(true, GameVariant.Classic));
  return searcher.search(limits, () => false, TABLE_PROFILE.weights);
}

function forceGc(): void {
  const gc = globalThis.gc;
  if (gc === undefined) throw new Error("forced GC is unavailable");
  gc();
}

async function collectAcrossJobs(): Promise<void> {
  for (let attempt = 0; attempt < 8; attempt += 1) {
    await new Promise<void>((resolve) => {
      setImmediate(resolve);
    });
    forceGc();
  }
}

function terminalGame(): MonsGame {
  const fields = new MonsGame(true, GameVariant.Classic).fen().split(" ");
  fields[0] = "5";
  const game = MonsGame.fromFen(fields.join(" "), true);
  if (game === undefined) throw new Error("terminal workspace FEN must parse");
  return game;
}

function runPreflightNoAllocation(): void {
  const tracker = createAllocationTracker();
  const harness = createPoolHarness(tracker, false);
  let invalidClockReads = 0;
  const invalid = selectionContext(() => {
    invalidClockReads += 1;
    return 0;
  });
  const invalidGame = new MonsGame(true, GameVariant.Classic);

  expect(() =>
    selectFastSelection(
      invalid.context,
      invalidGame,
      {
        ...TABLE_PROFILE,
        maxNodes: Number.NaN,
      },
      harness.pool,
    ),
  ).toThrow(new RangeError("maxNodes must be a nonnegative safe integer"));
  expect(invalidClockReads).toBe(0);

  const terminal = selectionContext();
  expect(
    selectFastInputs(
      terminal.context,
      terminalGame(),
      TABLE_PROFILE,
      harness.pool,
    ),
  ).toEqual([]);

  let expiredNow = 0;
  const expired = selectionContext(() => expiredNow);
  const timedOut = expired.session.withDeadlineIfAbsent(1, () => {
    expiredNow = 1;
    return selectFastInputs(
      expired.context,
      new MonsGame(true, GameVariant.Classic),
      TABLE_PROFILE,
      harness.pool,
    );
  });
  expect(timedOut).toEqual([]);
  expect(expired.session.takePreviousTimeout()).toBe(true);
  expect(harness.creations()).toBe(0);
  expect(tracker.records).toEqual([]);

  const directSearcher = new FastSearcher({
    allocateTranspositionPart: tracker.allocate,
  });
  loadPosition(directSearcher.root, invalidGame);
  let timeoutChecks = 0;
  const checkTimeout = (): boolean => {
    timeoutChecks += 1;
    return false;
  };
  expect(() =>
    directSearcher.search({ maxDepth: 1, maxNodes: Number.NaN }, checkTimeout),
  ).toThrow(new RangeError("maxNodes must be a nonnegative safe integer"));
  expect(() =>
    directSearcher.search(
      {
        maxDepth: 1,
        maxNodes: 1,
        tuning: {
          lateMoveReduction: true,
          lateMoveIndex: 3,
          lateMoveDeepIndex: 8,
          moveCountPruning: true,
          moveCountDepth: Number.NaN,
          moveCountBase: 4,
          moveCountFactor: 5,
          futilityMargin: 900,
        },
      },
      checkTimeout,
    ),
  ).toThrow(
    new RangeError(
      `tuning.moveCountDepth must be a safe integer from 0 through ${MAX_SEARCH_DEPTH}`,
    ),
  );
  expect(() =>
    directSearcher.search({ maxDepth: 1, maxNodes: 1 }, checkTimeout, {
      ...TABLE_PROFILE.weights,
      scoreUnit: Number.NaN,
    }),
  ).toThrow(
    new RangeError(
      "weights.scoreUnit must be a safe integer from -1000000 through 1000000",
    ),
  );
  expect(timeoutChecks).toBe(0);
  expect(tracker.records).toEqual([]);
}

function runMoveOnlyEntries(): void {
  const tuning: SearchTuning = {
    lateMoveReduction: false,
    lateMoveIndex: 0,
    lateMoveDeepIndex: 0,
    moveCountPruning: true,
    moveCountDepth: MAX_SEARCH_DEPTH,
    moveCountBase: 0,
    moveCountFactor: 0,
    futilityMargin: 0,
  };
  const tracker = createAllocationTracker();
  const searcher = new FastSearcher({
    transpositionCapacity: TABLE_CAPACITY,
    allocateTranspositionPart: tracker.allocate,
  });
  const outcome = searchOpening(searcher, {
    maxDepth: 2,
    maxNodes: TABLE_PROFILE.maxNodes,
    tuning,
  });
  expect(outcome.depth).toBe(2);
  expectAllocationBatch(tracker.records, 0, TABLE_CAPACITY);

  const info = tracker.records.find(({ part }) => part === "info")?.array;
  expect(info).toBeDefined();
  if (info === undefined) return;
  let moveOnlyEntries = 0;
  for (const value of info) {
    if ((value & 3) !== TT_MOVE_ONLY_FLAG) continue;
    moveOnlyEntries += 1;
    expect((value >> 2) & 63).toBe(0);
  }
  expect(moveOnlyEntries).toBeGreaterThan(0);
}

function runSelectiveEntries(): void {
  const tuning: SearchTuning = {
    lateMoveReduction: false,
    lateMoveIndex: 0,
    lateMoveDeepIndex: 0,
    moveCountPruning: true,
    moveCountDepth: 1,
    moveCountBase: 0,
    moveCountFactor: 0,
    futilityMargin: 999_951,
  };
  const tracker = createAllocationTracker();
  const searcher = new FastSearcher({
    transpositionCapacity: 4_096,
    allocateTranspositionPart: tracker.allocate,
  });
  const outcome = searchOpening(searcher, {
    maxDepth: 3,
    maxNodes: 100_000,
    tuning,
  });
  expect(outcome.depth).toBe(3);
  expect(outcome.nodes).toBe(1_414);

  const info = tracker.records.find(({ part }) => part === "info")?.array;
  expect(info).toBeDefined();
  if (info === undefined) return;
  let moveOnlyEntries = 0;
  let ancestorEntries = 0;
  for (const value of info) {
    if (value === 0) continue;
    expect((value >>> 8) & TT_GENERATION_MASK).toBe(2);
    const flag = value & 3;
    const depth = (value >> 2) & 63;
    if (flag === TT_MOVE_ONLY_FLAG) {
      moveOnlyEntries += 1;
      expect(value & TT_SELECTIVE_FLAG).toBe(TT_SELECTIVE_FLAG);
      expect(depth).toBe(0);
    } else if ((value & TT_SELECTIVE_FLAG) !== 0 && depth === 2) {
      ancestorEntries += 1;
    }
  }
  expect(moveOnlyEntries).toBeGreaterThan(0);
  expect(ancestorEntries).toBeGreaterThan(0);
}

function runSeededSelectiveHit(maxDepth: number): SearchOutcome {
  const game = new MonsGame(true, GameVariant.Classic);
  const rootMove = 16_752_536;
  const child = new FastPosition();
  loadPosition(child, game);
  expect(applyFastMove(child, rootMove)).toBe(-1);
  const scalar = scalarIndex(child);
  const keyLo = stateKeyLo(child, scalar);
  const keyHi = stateKeyHi(child, scalar);
  const capacity = 4_096;
  const slot = (keyLo ^ (keyHi * 3)) & (capacity - 1);
  expect({ scalar, keyLo, keyHi, slot }).toEqual({
    scalar: 50,
    keyLo: -477_779_693,
    keyHi: 361_269_357,
    slot: 84,
  });

  const arrays: Partial<Record<TranspositionPart, Int32Array>> = {};
  const searcher = new FastSearcher({
    transpositionCapacity: capacity,
    allocateTranspositionPart: (part, length) => {
      const array = new Int32Array(length);
      arrays[part] = array;
      if (part === "move") {
        const keyLoArray = arrays.keyLo;
        const keyHiArray = arrays.keyHi;
        const scoreArray = arrays.score;
        const infoArray = arrays.info;
        if (
          keyLoArray === undefined ||
          keyHiArray === undefined ||
          scoreArray === undefined ||
          infoArray === undefined
        ) {
          throw new Error("TT seed arrays were not allocated in order");
        }
        keyLoArray[slot] = keyLo;
        keyHiArray[slot] = keyHi;
        scoreArray[slot] = WIN_VALUE - 1;
        infoArray[slot] = (2 << 8) | (1 << 2) | TT_SELECTIVE_FLAG;
        array[slot] = rootMove;
      }
      return array;
    },
  });
  loadPosition(searcher.root, game);
  return searcher.search({ maxDepth, maxNodes: 100_000 }, () => false);
}

function runRangeError(failurePart: TranspositionPart): void {
  let pendingFailure = true;
  const tracker = createAllocationTracker((record) => {
    if (!pendingFailure || record.part !== failurePart) return;
    pendingFailure = false;
    throw new RangeError("synthetic TT allocation failure");
  });
  const harness = createPoolHarness(tracker, true);
  const { context, session } = selectionContext();
  const game = new MonsGame(true, GameVariant.Classic);
  const sourceFen = game.fen();
  const depthOne = selectFastSelection(
    context,
    game,
    {
      ...TABLE_PROFILE,
      maxDepth: 1,
    },
    harness.pool,
  );
  expectApplicableSelection(depthOne, game);
  expect(tracker.records).toEqual([]);

  const first = selectFastSelection(context, game, TABLE_PROFILE, harness.pool);
  expectApplicableSelection(first, game);
  expect(first).toEqual(depthOne);
  expect(game.fen()).toBe(sourceFen);
  expect(context.caches.session.cacheCount).toBe(0);
  expect(context.caches.engine.cacheCount).toBe(0);
  expect(session.takePreviousTimeout()).toBe(false);

  const failureIndex = TT_PARTS.indexOf(failurePart);
  expect(tracker.records).toHaveLength(failureIndex + 1);
  expect(tracker.records.at(-1)).toMatchObject({
    part: failurePart,
    length: TABLE_CAPACITY,
    array: undefined,
  });
  expect(
    tracker.records.filter(({ array }) => array !== undefined),
  ).toHaveLength(failureIndex);

  const retryStart = tracker.records.length;
  const retry = selectFastSelection(context, game, TABLE_PROFILE, harness.pool);
  expectApplicableSelection(retry, game);
  expect(game.fen()).toBe(sourceFen);
  expectAllocationBatch(tracker.records, retryStart, TABLE_CAPACITY);
  expect(harness.creations()).toBe(1);
}

function runInvalidAllocation(
  invalidPart: TranspositionPart,
  invalidArray: (length: number) => Int32Array,
): void {
  let rejectNext = true;
  const records: AllocationRecord[] = [];
  const searcher = new FastSearcher({
    transpositionCapacity: LOW_CAPACITY,
    allocateTranspositionPart: (part, length) => {
      const invalid = rejectNext && part === invalidPart;
      if (invalid) {
        rejectNext = false;
      }
      const array = invalid ? invalidArray(length) : new Int32Array(length);
      records.push({ part, length, array });
      return array;
    },
  });

  const first = searchOpening(searcher, TABLE_PROFILE);
  expect(first.depth).toBe(1);
  expect(searcher.size).toBe(0);
  expect(records).toHaveLength(TT_PARTS.indexOf(invalidPart) + 1);

  const retryStart = records.length;
  const retry = searchOpening(searcher, TABLE_PROFILE);
  expect(retry.depth).toBe(2);
  expect(searcher.size).toBeGreaterThan(0);
  expect(searcher.size).toBeLessThanOrEqual(LOW_CAPACITY);
  expectAllocationBatch(records, retryStart, LOW_CAPACITY);
}

function runOverlappingAllocation(): void {
  let overlapNext = true;
  let firstArray: Int32Array | undefined;
  const records: AllocationRecord[] = [];
  const searcher = new FastSearcher({
    transpositionCapacity: LOW_CAPACITY,
    allocateTranspositionPart: (part, length) => {
      let array: Int32Array;
      if (overlapNext && part === "keyLo") {
        firstArray = new Int32Array(length);
        array = firstArray;
      } else if (overlapNext && part === "keyHi") {
        if (firstArray === undefined) throw new Error("missing first TT part");
        array = firstArray;
        overlapNext = false;
      } else {
        array = new Int32Array(length);
      }
      records.push({ part, length, array });
      return array;
    },
  });

  expect(searchOpening(searcher, TABLE_PROFILE).depth).toBe(1);
  expect(searcher.size).toBe(0);
  expectAllocationBatch(records, 0, LOW_CAPACITY);

  const retryStart = records.length;
  expect(searchOpening(searcher, TABLE_PROFILE).depth).toBe(2);
  expect(searcher.size).toBeGreaterThan(0);
  expectAllocationBatch(records, retryStart, LOW_CAPACITY);
}

function runNonRangeError(): void {
  const sentinel = new TypeError("synthetic non-allocation failure");
  const tracker = createAllocationTracker(() => {
    throw sentinel;
  });
  const harness = createPoolHarness(tracker, true);
  const { context } = selectionContext();
  const game = new MonsGame(true, GameVariant.Classic);
  const sourceFen = game.fen();

  expect(() =>
    selectFastSelection(context, game, TABLE_PROFILE, harness.pool),
  ).toThrow(sentinel);
  expect(tracker.records).toEqual([
    {
      part: "keyLo",
      length: TABLE_CAPACITY,
      array: undefined,
    },
  ]);
  expect(harness.creations()).toBe(1);
  expect(game.fen()).toBe(sourceFen);
}

function runAllocationDeadline(recorded: boolean): void {
  let now = 0;
  const tracker = createAllocationTracker((_record, ordinal) => {
    if (ordinal === 1) now = TABLE_PROFILE.budgetMs;
  });
  const harness = createPoolHarness(tracker, true);
  const { context, session } = selectionContext(() => now);
  const game = new MonsGame(true, GameVariant.Classic);
  const sourceFen = game.fen();
  const depthOne = selectFastSelection(
    context,
    game,
    {
      ...TABLE_PROFILE,
      maxDepth: 1,
    },
    harness.pool,
  );
  expectApplicableSelection(depthOne, game);
  expect(tracker.records).toEqual([]);

  const select = () =>
    selectFastSelection(context, game, TABLE_PROFILE, harness.pool);
  const selection = recorded
    ? session.withDeadlineIfAbsent(TABLE_PROFILE.budgetMs, select)
    : select();
  expectApplicableSelection(selection, game);
  expect(selection).toEqual(depthOne);
  expect(game.fen()).toBe(sourceFen);
  expect(tracker.records).toHaveLength(1);
  expect(tracker.records[0]).toMatchObject({
    part: "keyLo",
    length: TABLE_CAPACITY,
  });
  expect(tracker.records[0]?.array).toBeInstanceOf(Int32Array);
  expect(session.takePreviousTimeout()).toBe(recorded);
}

async function runWeakReclamation(): Promise<void> {
  let observer: WeakRef<FastSearcher> | undefined;
  let creations = 0;
  const pool = new FastSearcherPool({
    createSearcher: () => {
      creations += 1;
      const searcher = new FastSearcher({
        transpositionCapacity: LOW_CAPACITY,
      });
      observer = new WeakRef(searcher);
      return searcher;
    },
  });

  selectOnce(pool, LOW_PROFILE);
  expect(creations).toBe(1);
  const reference = observer;
  expect(reference).toBeDefined();
  if (reference === undefined) return;
  expect(reference.deref()).toBeDefined();

  await collectAcrossJobs();

  expect(reference.deref()).toBeUndefined();
}

function runIsolatedScenario(scenario: string): void {
  const child = spawnSync(
    process.execPath,
    [
      vitestCli,
      "run",
      testFile,
      "--pool=forks",
      "--maxWorkers=1",
      "--no-file-parallelism",
      "--reporter=dot",
      "--execArgv=--expose-gc",
    ],
    {
      cwd: repositoryRoot,
      encoding: "utf8",
      env: {
        ...process.env,
        [CHILD_SCENARIO_ENV]: scenario,
      },
      timeout: 60_000,
    },
  );

  expect(
    {
      error: child.error?.message,
      signal: child.signal,
      status: child.status,
      stderr: child.stderr,
      stdout: child.stdout,
    },
    `${scenario} subprocess failed`,
  ).toMatchObject({
    error: undefined,
    signal: null,
    status: 0,
  });
}

if (childScenario === "weak-reclamation") {
  describe("fast workspace weak-reclamation child", () => {
    it("releases the weakly pooled searcher", async () => {
      await runWeakReclamation();
    });
  });
} else if (childScenario === undefined) {
  describe("fast workspace lifecycle", () => {
    it("does not create a searcher or table for preflight exits", () => {
      runPreflightNoAllocation();
    });

    it.each([undefined, 1, LOW_CAPACITY, TABLE_CAPACITY, PRO_CAPACITY])(
      "accepts transposition capacity %s without allocating",
      (transpositionCapacity) => {
        const tracker = createAllocationTracker();
        const options =
          transpositionCapacity === undefined
            ? { allocateTranspositionPart: tracker.allocate }
            : {
                transpositionCapacity,
                allocateTranspositionPart: tracker.allocate,
              };
        expect(() => new FastSearcher(options)).not.toThrow();
        expect(tracker.records).toEqual([]);
      },
    );

    it.each([
      0,
      -1,
      1.5,
      3,
      Number.NaN,
      Number.POSITIVE_INFINITY,
      PRO_CAPACITY + 1,
      1 << 21,
    ])("rejects invalid transposition capacity %s", (transpositionCapacity) => {
      expect(() => new FastSearcher({ transpositionCapacity })).toThrow(
        new RangeError(
          "transpositionCapacity must be a power of two from 1 through 1048576",
        ),
      );
    });

    it.each([
      {
        name: "low",
        profile: PRO_TABLE_PROFILE,
        capacity: LOW_CAPACITY,
        bytes: 40_960,
      },
      {
        name: "table",
        profile: LOW_PROFILE,
        capacity: TABLE_CAPACITY,
        bytes: 2_621_440,
      },
      {
        name: "pro",
        profile: TABLE_PROFILE,
        capacity: undefined,
        bytes: 20_971_520,
      },
    ])(
      "sizes a cold $name table from searcher configuration",
      ({ profile, capacity, bytes }) => {
        const tracker = createAllocationTracker();
        const options =
          capacity === undefined
            ? { allocateTranspositionPart: tracker.allocate }
            : {
                transpositionCapacity: capacity,
                allocateTranspositionPart: tracker.allocate,
              };
        const searcher = new FastSearcher(options);
        const outcome = searchOpening(searcher, profile);

        expect(outcome.depth).toBe(2);
        expectAllocationBatch(tracker.records, 0, capacity ?? PRO_CAPACITY);
        expect(
          tracker.records.reduce(
            (total, record) => total + (record.array?.byteLength ?? 0),
            0,
          ),
        ).toBe(bytes);
      },
    );

    it("reuses one searcher and same-capacity table", () => {
      const tracker = createAllocationTracker();
      const harness = createPoolHarness(tracker, true);
      const first = selectOnce(harness.pool);
      const second = selectOnce(harness.pool);

      expect(second).toEqual(first);
      expect(harness.creations()).toBe(1);
      expect(tracker.records).toHaveLength(TT_PARTS.length);
      expectAllocationBatch(tracker.records, 0, TABLE_CAPACITY);
    });

    it("creates independent searchers when pooling is disabled", () => {
      const tracker = createAllocationTracker();
      const harness = createPoolHarness(tracker, false);
      const first = selectOnce(harness.pool);
      const second = selectOnce(harness.pool);

      expect(second).toEqual(first);
      expect(harness.creations()).toBe(2);
      expect(tracker.records).toHaveLength(TT_PARTS.length * 2);
      expectAllocationBatch(tracker.records, 0, TABLE_CAPACITY);
      expectAllocationBatch(tracker.records, TT_PARTS.length, TABLE_CAPACITY);
    });

    it("reuses configured backing across different node limits", () => {
      const tracker = createAllocationTracker();
      const searcher = new FastSearcher({
        transpositionCapacity: TABLE_CAPACITY,
        allocateTranspositionPart: tracker.allocate,
      });

      expect(searchOpening(searcher, PRO_TABLE_PROFILE).depth).toBe(2);
      expect(searchOpening(searcher, LOW_PROFILE).depth).toBe(2);
      expect(tracker.records).toHaveLength(TT_PARTS.length);
      expectAllocationBatch(tracker.records, 0, TABLE_CAPACITY);
    });

    it("keeps live slot occupancy bounded under collisions", () => {
      let infoBacking: Int32Array | undefined;
      const searcher = new FastSearcher({
        transpositionCapacity: 1,
        allocateTranspositionPart: (part, length) => {
          const array = new Int32Array(length);
          if (part !== "info") return array;
          infoBacking = array;
          return array;
        },
      });

      expect(searchOpening(searcher, TABLE_PROFILE).depth).toBe(2);
      expect(
        infoBacking?.reduce((count, value) => count + Number(value !== 0), 0),
      ).toBe(1);
      expect(searcher.size).toBe(1);
    });

    it("records move-only entries in the named info part", () => {
      runMoveOnlyEntries();
    });

    it("propagates descendant selectivity through score-bearing ancestors", () => {
      runSelectiveEntries();
    });

    it("reuses tainted TT scores without treating them as proven terminals", () => {
      expect(runSeededSelectiveHit(2)).toEqual({
        move: 16_752_536,
        score: WIN_VALUE - 1,
        depth: 2,
        nodes: 99,
        supported: true,
      });
      expect(runSeededSelectiveHit(3)).toEqual({
        move: 16_752_536,
        score: 1_547,
        depth: 3,
        nodes: 1_389,
        supported: true,
      });
    });

    it.each([
      { name: "first", part: "keyLo" },
      { name: "last", part: "move" },
    ] as const)(
      "retains a legal depth-one result after a RangeError in the $name part",
      ({ part }) => {
        runRangeError(part);
      },
    );

    it("fails soft when an allocator returns a short Int32Array", () => {
      runInvalidAllocation("score", (length) => new Int32Array(length - 1));
    });

    it("fails soft when an allocator returns another typed-array kind", () => {
      runInvalidAllocation(
        "keyLo",
        (length) => new Uint32Array(length) as unknown as Int32Array,
      );
    });

    it("fails soft when an allocator returns a typed-array proxy", () => {
      runInvalidAllocation(
        "info",
        (length) => new Proxy(new Int32Array(length), {}),
      );
    });

    it("accepts genuine cross-realm Int32Array parts", () => {
      const searcher = new FastSearcher({
        transpositionCapacity: LOW_CAPACITY,
        allocateTranspositionPart: (_part, length) =>
          runInNewContext("new Int32Array(length)", {
            length,
          }) as Int32Array,
      });

      expect(searchOpening(searcher, TABLE_PROFILE).depth).toBe(2);
      expect(searcher.size).toBeGreaterThan(0);
    });

    it("fails soft when transposition parts overlap", () => {
      runOverlappingAllocation();
    });

    it("propagates non-Range allocation errors", () => {
      runNonRangeError();
    });

    it.each([
      { name: "unrecorded", recorded: false },
      { name: "recorded", recorded: true },
    ])("keeps an allocation deadline $name", ({ recorded }) => {
      runAllocationDeadline(recorded);
    });

    it("reclaims the weakly pooled searcher", () => {
      runIsolatedScenario("weak-reclamation");
    }, 60_000);
  });
} else {
  throw new Error(`unknown fast workspace child scenario: ${childScenario}`);
}
