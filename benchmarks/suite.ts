import { MonsGame } from "../src/engine/game.js";
import { AutomoveEngine } from "../src/automove/automove-engine.js";
import { tryLoadPosition } from "../src/automove/fast/bridge.js";
import { FastSearcher } from "../src/automove/fast/search.js";
import { enumerateLegalTransitions } from "../src/automove/transitions.js";
import { parseInputArrayFen } from "../src/engine/fen.js";
import { Game } from "../src/entrypoints/mons-rules.js";

type Preference = "fast" | "normal" | "pro";

export const BENCHMARK_CONTRACT_VERSION = 2;

const PACKED_MAX_DEPTH = 40;
const PACKED_FIXED_MAX_NODES = 250_000;
const PACKED_SMOKE_MAX_NODES = 25_000;
const PACKED_DEADLINE_BUDGET_MS = 460;
const PACKED_SMOKE_DEADLINE_BUDGET_MS = 20;
const REQUIRED_TIMED_METRIC_NAMES = [
  "game.fromFen",
  "game.toFen",
  "game.previewFen",
  "game.playFen",
  "game.startQuery.cold",
  "game.startQuery.warm",
  "game.forkApply",
  "engine.transitions",
  "engine.transitions.initial",
  "engine.transitions.midgame",
  "automove.fast",
  "automove.normal",
] as const;

export type BenchmarkState = {
  readonly id: string;
  readonly fen: string;
  readonly decisions: Readonly<
    Record<Preference, { readonly inputFen: string }>
  >;
};

export type BenchmarkOptions = {
  readonly samples: number;
  readonly smoke: boolean;
  readonly sampleOffset?: number;
  readonly deadlineStateIndices?: readonly number[];
};

export type InterleavedBenchmarkOptions = BenchmarkOptions & {
  readonly batches: number;
  readonly allowLegacyBaseline?: boolean;
};

type Distribution = {
  readonly samples: readonly number[];
  readonly median: number;
  readonly p95: number;
};

type TimedMetric = Distribution & {
  readonly name: string;
  readonly operations: number;
  readonly checksum?: number;
  readonly timedSinkChecksum?: number;
  readonly microsecondsPerOperation: number;
};

type PackedMetric = Distribution & {
  readonly name: string;
  readonly nodes: number;
  readonly nodesPerMillisecond: number;
  readonly depthChecksum?: number;
  readonly moveChecksum?: number;
  readonly configuration: PackedWorkConfiguration;
  readonly budgetMs?: number;
  readonly stateIds?: readonly string[];
  readonly stateChecksum?: number;
};

type PackedWorkConfiguration = {
  readonly maxDepth: number;
  readonly maxNodes: number;
  readonly includesPositionLoad: boolean;
};

type MemoryMetric = {
  readonly scope: "shared-runtime";
  readonly measurement: "end-snapshot" | "maximum-of-end-snapshots";
  readonly isPeak: false;
  readonly attributableToImplementation: false;
  readonly comparableAcrossImplementations: false;
  readonly heapUsed?: number;
  readonly rss?: number;
};

export type PackedDeadlineDependencies = {
  readonly createGame: (fen: string) => MonsGame | undefined;
  readonly createSearcher: () => FastSearcher;
  readonly tryLoadPosition: typeof tryLoadPosition;
};

export type BenchmarkReport = {
  readonly contractVersion: number;
  readonly generatedAt: string;
  readonly environment: {
    readonly runtime: string;
    readonly userAgent: string;
  };
  readonly options: BenchmarkOptions;
  readonly stateCount: number;
  readonly timed: readonly TimedMetric[];
  readonly packed: PackedMetric;
  readonly packedDeadline?: PackedMetric;
  readonly memory: MemoryMetric;
};

export type BenchmarkRunner = (
  states: readonly BenchmarkState[],
  options: BenchmarkOptions,
) => BenchmarkReport;

export type BenchmarkMetricParity = {
  readonly complete: boolean;
  readonly contractVersionMatch: boolean;
  readonly stateCountMatch: boolean;
  readonly requiredTimedMetricSetMatch: boolean;
  readonly missingRequiredBaselineTimedMetrics: readonly string[];
  readonly missingRequiredCandidateTimedMetrics: readonly string[];
  readonly unexpectedBaselineTimedMetrics: readonly string[];
  readonly unexpectedCandidateTimedMetrics: readonly string[];
  readonly comparableTimedMetrics: readonly string[];
  readonly missingFromBaseline: readonly string[];
  readonly missingFromCandidate: readonly string[];
  readonly duplicateBaselineTimedMetrics: readonly string[];
  readonly duplicateCandidateTimedMetrics: readonly string[];
  readonly checksumMismatches: readonly {
    readonly name: string;
    readonly baseline: number | null;
    readonly candidate: number | null;
  }[];
  readonly timedSinkChecksumMismatches: readonly {
    readonly name: string;
    readonly baseline: number | null;
    readonly candidate: number | null;
  }[];
  readonly operationMismatches: readonly {
    readonly name: string;
    readonly baseline: number | null;
    readonly candidate: number | null;
  }[];
  readonly fixedWorkConfigurationMatch: boolean;
  readonly fixedWorkDepthChecksumMatch: boolean;
  readonly fixedWorkMoveChecksumMatch: boolean;
  readonly fixedWorkNodeCountMatch: boolean;
  readonly deadlineConfigurationMatch: boolean;
  readonly deadlineBudgetMatch: boolean;
  readonly deadlineProgressFieldsPresent: boolean;
  readonly baselineDeadlineStateCoverageMatch: boolean;
  readonly candidateDeadlineStateCoverageMatch: boolean;
  readonly deadlineStateCoverageMatch: boolean;
  readonly deadlineStateChecksumMatch: boolean;
  readonly deadlineMoveChecksumMatch: boolean;
  readonly action?: string;
};

export type BenchmarkExpectedContract = {
  readonly stateCount: number;
  readonly deadlineStateIds: readonly string[];
};

export type BenchmarkReportValidation = {
  readonly options?: BenchmarkOptions;
  readonly stateCount?: number;
  readonly deadlineStateIds?: readonly string[];
  readonly allowLegacy?: boolean;
};

export type InterleavedBenchmarkReport = {
  readonly contractVersion: number;
  readonly generatedAt: string;
  readonly environment: BenchmarkReport["environment"];
  readonly options: {
    readonly samples: number;
    readonly batches: number;
    readonly batchSamples: readonly number[];
    readonly smoke: boolean;
    readonly warmupRuns: number;
    readonly deadlineStateSchedule: readonly (readonly string[])[];
  };
  readonly stateCount: number;
  readonly executionOrder: readonly string[];
  readonly metricParity: BenchmarkMetricParity;
  readonly implementations: {
    readonly baseline: BenchmarkReport;
    readonly candidate: BenchmarkReport;
  };
};

function percentile(values: readonly number[], ratio: number): number {
  const sorted = [...values].sort((left, right) => left - right);
  const index = Math.min(
    sorted.length - 1,
    Math.max(0, Math.ceil(sorted.length * ratio) - 1),
  );
  return sorted[index] ?? 0;
}

function distribution(samples: readonly number[]): Distribution {
  return {
    samples,
    median: percentile(samples, 0.5),
    p95: percentile(samples, 0.95),
  };
}

function mixChecksum(checksum: number, value: number): number {
  return Math.imul(checksum ^ value, 16_777_619) >>> 0;
}

function stringChecksum(value: string): number {
  let checksum = 2_166_136_261;
  for (let index = 0; index < value.length; index += 1) {
    checksum = mixChecksum(checksum, value.charCodeAt(index));
  }
  return checksum;
}

function timedOutputSink(output: { readonly kind: string }): number {
  if (output.kind === "events") {
    const events = (
      output as unknown as { readonly events: readonly unknown[] }
    ).events;
    return (events.length << 3) | 1;
  }
  if (output.kind === "locations-to-start-from") {
    const locations = (
      output as unknown as { readonly locations: readonly unknown[] }
    ).locations;
    return (locations.length << 3) | 2;
  }
  if (output.kind === "next-input-options") {
    const inputs = (
      output as unknown as { readonly nextInputs: readonly unknown[] }
    ).nextInputs;
    return (inputs.length << 3) | 3;
  }
  return output.kind === "complete"
    ? 4
    : output.kind === "invalid-input" || output.kind === "invalid"
      ? 5
      : 6;
}

function verificationChecksum(value: unknown): number {
  let checksum = 2_166_136_261;
  const mixString = (text: string): void => {
    checksum = mixChecksum(checksum, text.length);
    for (let index = 0; index < text.length; index += 1) {
      checksum = mixChecksum(checksum, text.charCodeAt(index));
    }
  };
  const visit = (current: unknown): void => {
    if (current === null) {
      checksum = mixChecksum(checksum, 0);
      return;
    }
    switch (typeof current) {
      case "undefined":
        checksum = mixChecksum(checksum, 1);
        return;
      case "boolean":
        checksum = mixChecksum(checksum, current ? 3 : 2);
        return;
      case "number":
        checksum = mixChecksum(checksum, 4);
        mixString(Object.is(current, -0) ? "-0" : String(current));
        return;
      case "string":
        checksum = mixChecksum(checksum, 5);
        mixString(current);
        return;
      case "bigint":
        checksum = mixChecksum(checksum, 6);
        mixString(String(current));
        return;
      case "object": {
        if (Array.isArray(current)) {
          checksum = mixChecksum(checksum, 7);
          checksum = mixChecksum(checksum, current.length);
          for (const entry of current) visit(entry);
          return;
        }
        const keys = Object.keys(current).sort();
        checksum = mixChecksum(checksum, 8);
        checksum = mixChecksum(checksum, keys.length);
        for (const key of keys) {
          mixString(key);
          visit((current as Record<string, unknown>)[key]);
        }
        return;
      }
      default:
        throw new TypeError(`unsupported verification type: ${typeof current}`);
    }
  };
  visit(value);
  return checksum;
}

function measure(
  name: string,
  samples: number,
  operations: number,
  operation: (index: number, batchIndex: number) => number,
  verify: () => number,
  prepareBatch?: (startIndex: number, count: number) => void,
): TimedMetric {
  let timedSinkChecksum = 2_166_136_261;
  const batchSize = prepareBatch === undefined ? operations : 128;
  const runRound = (timed: boolean): number => {
    let elapsed = 0;
    for (let startIndex = 0; startIndex < operations; startIndex += batchSize) {
      const count = Math.min(batchSize, operations - startIndex);
      prepareBatch?.(startIndex, count);
      const start = timed ? performance.now() : 0;
      for (let batchIndex = 0; batchIndex < count; batchIndex += 1) {
        timedSinkChecksum = mixChecksum(
          timedSinkChecksum,
          operation(startIndex + batchIndex, batchIndex),
        );
      }
      if (timed) elapsed += performance.now() - start;
    }
    return elapsed;
  };
  runRound(false);
  const durations: number[] = [];
  for (let sample = 0; sample < samples; sample += 1) {
    durations.push(runRound(true));
  }
  const summary = distribution(durations);
  return {
    name,
    operations,
    checksum: verify(),
    timedSinkChecksum,
    ...summary,
    microsecondsPerOperation: (summary.median * 1_000) / operations,
  };
}

function runtimeEnvironment(): BenchmarkReport["environment"] {
  return {
    runtime: typeof window === "undefined" ? process.version : "browser",
    userAgent: typeof window === "undefined" ? "node" : navigator.userAgent,
  };
}

function memoryUsage(): BenchmarkReport["memory"] {
  const metadata = {
    scope: "shared-runtime",
    measurement: "end-snapshot",
    isPeak: false,
    attributableToImplementation: false,
    comparableAcrossImplementations: false,
  } as const;
  if (typeof window !== "undefined") {
    const browserMemory = (
      performance as Performance & {
        readonly memory?: { readonly usedJSHeapSize?: number };
      }
    ).memory;
    return browserMemory?.usedJSHeapSize === undefined
      ? metadata
      : { ...metadata, heapUsed: browserMemory.usedJSHeapSize };
  }
  const usage = process.memoryUsage();
  return { ...metadata, heapUsed: usage.heapUsed, rss: usage.rss };
}

function internalGame(state: BenchmarkState): MonsGame {
  const game = MonsGame.fromFen(state.fen, false);
  if (game === undefined) throw new Error(`invalid benchmark FEN: ${state.id}`);
  return game;
}

function publicGame(state: BenchmarkState): Game {
  const game = Game.fromFen(state.fen);
  if (game === undefined) throw new Error(`invalid benchmark FEN: ${state.id}`);
  return game;
}

function parsedFastInputs(state: BenchmarkState) {
  const inputs = parseInputArrayFen(state.decisions.fast.inputFen);
  if (inputs === undefined) {
    throw new Error(`invalid benchmark input FEN: ${state.id}`);
  }
  return inputs;
}

function timedMetrics(
  states: readonly BenchmarkState[],
  options: BenchmarkOptions,
): TimedMetric[] {
  const codecOperations = options.smoke ? 2_000 : 50_000;
  const gameplayOperations = options.smoke ? 200 : 10_000;
  const warmQueryOperations = options.smoke ? 100_000 : 1_000_000;
  const transitionOperations = options.smoke ? 40 : 2_000;
  const games = states.map(publicGame);
  const internalGames = states.map(internalGame);
  const inputs = states.map(parsedFastInputs);
  const warmQueryGames = states.map(internalGame);
  for (const game of warmQueryGames) game.processInput([], true, false);
  const engine = new AutomoveEngine();
  const automoveStates = options.smoke ? states.slice(0, 2) : states;
  const initialStates = states.filter((state) =>
    state.id.startsWith("initial-"),
  );
  const midgameStates = states.filter(
    (state) => !state.id.startsWith("initial-"),
  );
  const retainedMidgameStates =
    midgameStates.length === 0 ? states : midgameStates;
  const initialGames = initialStates.map(internalGame);
  const midgameGames = retainedMidgameStates.map(internalGame);
  const repeatedPublicGames = (
    sourceStates: readonly BenchmarkState[],
    count: number,
    startIndex: number,
  ): Game[] => {
    if (sourceStates.length === 0) return [];
    return Array.from({ length: count }, (_, index) => {
      const state = sourceStates[(startIndex + index) % sourceStates.length];
      if (state === undefined) throw new RangeError("missing benchmark state");
      return publicGame(state);
    });
  };
  const repeatedInternalGames = (
    sourceStates: readonly BenchmarkState[],
    count: number,
    startIndex: number,
  ): MonsGame[] => {
    if (sourceStates.length === 0) return [];
    return Array.from({ length: count }, (_, index) => {
      const state = sourceStates[(startIndex + index) % sourceStates.length];
      if (state === undefined) throw new RangeError("missing benchmark state");
      return internalGame(state);
    });
  };
  const verifyTransitions = (sourceStates: readonly BenchmarkState[]): number =>
    verificationChecksum(
      sourceStates.map((state) => {
        const game = internalGame(state);
        const beforeFen = game.fen();
        const transitions = engine.run((execution) =>
          enumerateLegalTransitions(execution, game, 256),
        );
        return {
          id: state.id,
          beforeFen,
          afterFen: game.fen(),
          transitions: transitions.map((transition) => ({
            inputs: transition.inputs,
            events: transition.events,
            fen: transition.game.fen(),
          })),
        };
      }),
    );
  let playGames: Game[] = [];
  let coldQueryGames: MonsGame[] = [];
  let fastGames: Game[] = [];
  let normalGames: Game[] = [];
  const metricRunners: readonly (() => TimedMetric)[] = [
    () =>
      measure(
        "game.fromFen",
        options.samples,
        codecOperations,
        (index) => {
          const state = states[index % states.length];
          return state === undefined
            ? 0
            : (Game.fromFen(state.fen)?.turnNumber ?? 0);
        },
        () =>
          verificationChecksum(
            states.map((state) => ({
              id: state.id,
              fen: Game.fromFen(state.fen)?.toFen(),
            })),
          ),
      ),
    () =>
      measure(
        "game.toFen",
        options.samples,
        codecOperations,
        (index) => games[index % games.length]?.toFen().length ?? 0,
        () =>
          verificationChecksum(
            states.map((state, index) => ({
              id: state.id,
              fen: games[index]?.toFen(),
            })),
          ),
      ),
    () =>
      measure(
        "game.previewFen",
        options.samples,
        codecOperations,
        (index) => {
          const state = states[index % states.length];
          const game = games[index % games.length];
          if (state === undefined || game === undefined) return 0;
          return timedOutputSink(
            game.previewFen(state.decisions.fast.inputFen),
          );
        },
        () =>
          verificationChecksum(
            states.map((state, index) => {
              const game = games[index];
              if (game === undefined) return undefined;
              const beforeFen = game.toFen();
              const output = game.previewFen(state.decisions.fast.inputFen);
              return {
                id: state.id,
                inputFen: state.decisions.fast.inputFen,
                beforeFen,
                output,
                afterFen: game.toFen(),
              };
            }),
          ),
      ),
    () =>
      measure(
        "game.playFen",
        options.samples,
        gameplayOperations,
        (index, batchIndex) => {
          const state = states[index % states.length];
          const game = playGames[batchIndex];
          if (state === undefined || game === undefined) return 0;
          return timedOutputSink(game.playFen(state.decisions.fast.inputFen));
        },
        () =>
          verificationChecksum(
            states.map((state) => {
              const game = publicGame(state);
              const beforeFen = game.toFen();
              const output = game.playFen(state.decisions.fast.inputFen);
              return {
                id: state.id,
                inputFen: state.decisions.fast.inputFen,
                beforeFen,
                output,
                afterFen: game.toFen(),
              };
            }),
          ),
        (startIndex, count) => {
          playGames = repeatedPublicGames(states, count, startIndex);
        },
      ),
    () =>
      measure(
        "game.startQuery.cold",
        options.samples,
        gameplayOperations,
        (index, batchIndex) => {
          const state = states[index % states.length];
          const game = coldQueryGames[batchIndex];
          return state === undefined || game === undefined
            ? 0
            : timedOutputSink(game.processInput([], true, false));
        },
        () =>
          verificationChecksum(
            states.map((state) => {
              const game = internalGame(state);
              const beforeFen = game.fen();
              const output = game.processInput([], true, false);
              return {
                id: state.id,
                beforeFen,
                output,
                afterFen: game.fen(),
              };
            }),
          ),
        (startIndex, count) => {
          coldQueryGames = repeatedInternalGames(states, count, startIndex);
        },
      ),
    () =>
      measure(
        "game.startQuery.warm",
        options.samples,
        warmQueryOperations,
        (index) => {
          const game = warmQueryGames[index % warmQueryGames.length];
          return game === undefined
            ? 0
            : timedOutputSink(game.processInput([], true, false));
        },
        () =>
          verificationChecksum(
            states.map((state) => {
              const game = internalGame(state);
              const first = game.processInput([], true, false);
              const second = game.processInput([], true, false);
              return { id: state.id, first, second, fen: game.fen() };
            }),
          ),
      ),
    () =>
      measure(
        "game.forkApply",
        options.samples,
        gameplayOperations,
        (index) => {
          const slot = index % internalGames.length;
          const game = internalGames[slot];
          const move = inputs[slot];
          if (game === undefined || move === undefined) return 0;
          const fork = game.fork();
          const output = fork.processInput(move, false, false);
          return timedOutputSink(output);
        },
        () =>
          verificationChecksum(
            states.map((state, index) => {
              const game = internalGame(state);
              const move = inputs[index];
              if (move === undefined) return undefined;
              const beforeFen = game.fen();
              const fork = game.fork();
              const output = fork.processInput(move, false, false);
              return {
                id: state.id,
                move,
                beforeFen,
                sourceAfterFen: game.fen(),
                output,
                forkFen: fork.fen(),
              };
            }),
          ),
      ),
    () =>
      measure(
        "engine.transitions",
        options.samples,
        options.smoke ? 80 : 2_000,
        (index) => {
          const game = internalGames[index % internalGames.length];
          if (game === undefined) return 0;
          return engine.run(
            (execution) =>
              enumerateLegalTransitions(execution, game, 256).length,
          );
        },
        () => verifyTransitions(states),
      ),
    () =>
      measure(
        "engine.transitions.initial",
        options.samples,
        transitionOperations,
        (index) => {
          const game = initialGames[index % initialGames.length];
          if (game === undefined) return 0;
          return engine.run(
            (execution) =>
              enumerateLegalTransitions(execution, game, 256).length,
          );
        },
        () => verifyTransitions(initialStates),
      ),
    () =>
      measure(
        "engine.transitions.midgame",
        options.samples,
        transitionOperations,
        (index) => {
          const game = midgameGames[index % midgameGames.length];
          if (game === undefined) return 0;
          return engine.run(
            (execution) =>
              enumerateLegalTransitions(execution, game, 256).length,
          );
        },
        () => verifyTransitions(retainedMidgameStates),
      ),
    () =>
      measure(
        "automove.fast",
        options.samples,
        automoveStates.length * (options.smoke ? 1 : 2),
        (_index, batchIndex) => {
          return (
            fastGames[batchIndex]?.suggestMove("fast")?.inputFen.length ?? 0
          );
        },
        () =>
          verificationChecksum(
            automoveStates.map((state) => {
              const game = publicGame(state);
              const beforeFen = game.toFen();
              const suggestion = game.suggestMove("fast");
              return {
                id: state.id,
                beforeFen,
                suggestion,
                afterFen: game.toFen(),
              };
            }),
          ),
        (startIndex, count) => {
          fastGames = repeatedPublicGames(automoveStates, count, startIndex);
        },
      ),
    () =>
      measure(
        "automove.normal",
        options.samples,
        automoveStates.length * (options.smoke ? 1 : 2),
        (_index, batchIndex) => {
          return (
            normalGames[batchIndex]?.suggestMove("normal")?.inputFen.length ?? 0
          );
        },
        () =>
          verificationChecksum(
            automoveStates.map((state) => {
              const game = publicGame(state);
              const beforeFen = game.toFen();
              const suggestion = game.suggestMove("normal");
              return {
                id: state.id,
                beforeFen,
                suggestion,
                afterFen: game.toFen(),
              };
            }),
          ),
        (startIndex, count) => {
          normalGames = repeatedPublicGames(automoveStates, count, startIndex);
        },
      ),
  ];
  const offset = (options.sampleOffset ?? 0) % metricRunners.length;
  return metricRunners.map((_, index) => {
    const runMetric = metricRunners[(index + offset) % metricRunners.length];
    if (runMetric === undefined) throw new RangeError("missing timed metric");
    return runMetric();
  });
}

function packedFixedWorkMetric(
  states: readonly BenchmarkState[],
  options: BenchmarkOptions,
): PackedMetric {
  const searchStates = options.smoke ? states.slice(0, 2) : states;
  const searchGames = searchStates.map(internalGame);
  const maxDepth = PACKED_MAX_DEPTH;
  const maxNodes = options.smoke
    ? PACKED_SMOKE_MAX_NODES
    : PACKED_FIXED_MAX_NODES;
  const warmState = searchStates[0];
  const warmGame = searchGames[0];
  if (warmState !== undefined && warmGame !== undefined) {
    const warmSearcher = new FastSearcher();
    if (!tryLoadPosition(warmSearcher.root, warmGame, maxDepth)) {
      throw new Error(`unsupported benchmark state: ${warmState.id}`);
    }
    warmSearcher.search(
      { maxDepth, maxNodes: Math.min(PACKED_SMOKE_MAX_NODES, maxNodes) },
      () => false,
    );
  }
  const durations: number[] = [];
  let nodes = 0;
  let depthChecksum = 2_166_136_261;
  let moveChecksum = 2_166_136_261;
  for (let sample = 0; sample < options.samples; sample += 1) {
    const searcher = new FastSearcher();
    let elapsed = 0;
    for (let index = 0; index < searchStates.length; index += 1) {
      const state = searchStates[index];
      const game = searchGames[index];
      if (state === undefined || game === undefined) {
        throw new RangeError("missing packed benchmark state");
      }
      if (!tryLoadPosition(searcher.root, game, maxDepth)) {
        throw new Error(`unsupported benchmark state: ${state.id}`);
      }
      const start = performance.now();
      const outcome = searcher.search({ maxDepth, maxNodes }, () => false);
      elapsed += performance.now() - start;
      nodes += outcome.nodes;
      depthChecksum = mixChecksum(depthChecksum, outcome.depth);
      moveChecksum = mixChecksum(moveChecksum, outcome.move);
    }
    durations.push(elapsed);
  }
  const summary = distribution(durations);
  return {
    name: "automove.pro.fixedWork",
    nodes,
    nodesPerMillisecond:
      nodes / durations.reduce((sum, value) => sum + value, 0),
    depthChecksum,
    moveChecksum,
    configuration: { maxDepth, maxNodes, includesPositionLoad: false },
    ...summary,
  };
}

export function runPackedDeadlineMetric(
  states: readonly BenchmarkState[],
  options: BenchmarkOptions,
  dependencies: PackedDeadlineDependencies = {
    createGame: (fen) => MonsGame.fromFen(fen, false),
    createSearcher: () => new FastSearcher(),
    tryLoadPosition,
  },
): PackedMetric {
  const budgetMs = options.smoke
    ? PACKED_SMOKE_DEADLINE_BUDGET_MS
    : PACKED_DEADLINE_BUDGET_MS;
  const maxDepth = PACKED_MAX_DEPTH;
  const maxNodes = Number.MAX_SAFE_INTEGER;
  const durations: number[] = [];
  let nodes = 0;
  let depthChecksum = 2_166_136_261;
  let moveChecksum = 2_166_136_261;
  let stateChecksum = 2_166_136_261;
  const stateIds: string[] = [];
  const offset = options.sampleOffset ?? 0;
  const stateIndices =
    options.deadlineStateIndices ??
    (options.smoke
      ? Array.from(
          { length: options.samples },
          (_, sample) => (offset + sample) % states.length,
        )
      : states.map((_, index) => index));
  for (const stateIndex of stateIndices) {
    const state = states[stateIndex];
    if (state === undefined) continue;
    stateIds.push(state.id);
    stateChecksum = mixChecksum(stateChecksum, stringChecksum(state.id));
    const searcher = dependencies.createSearcher();
    const game = dependencies.createGame(state.fen);
    if (game === undefined) {
      throw new Error(`invalid benchmark FEN: ${state.id}`);
    }
    if (!dependencies.tryLoadPosition(searcher.root, game, maxDepth)) {
      throw new Error(`unsupported benchmark state: ${state.id}`);
    }
    const start = performance.now();
    const deadline = start + budgetMs;
    const outcome = searcher.search(
      { maxDepth, maxNodes },
      () => performance.now() >= deadline,
    );
    const elapsed = performance.now() - start;
    durations.push(elapsed);
    nodes += outcome.nodes;
    depthChecksum = mixChecksum(depthChecksum, outcome.depth);
    moveChecksum = mixChecksum(moveChecksum, outcome.move);
  }
  const summary = distribution(durations);
  return {
    name: "automove.pro.deadline",
    budgetMs,
    nodes,
    nodesPerMillisecond:
      nodes / durations.reduce((sum, value) => sum + value, 0),
    depthChecksum,
    moveChecksum,
    configuration: { maxDepth, maxNodes, includesPositionLoad: false },
    stateIds,
    stateChecksum,
    ...summary,
  };
}

export function runBenchmarkSuite(
  states: readonly BenchmarkState[],
  options: BenchmarkOptions,
): BenchmarkReport {
  if (states.length === 0) {
    throw new Error("benchmark states must not be empty");
  }
  if (!Number.isSafeInteger(options.samples) || options.samples < 1) {
    throw new RangeError("benchmark samples must be a positive safe integer");
  }
  const timed = timedMetrics(states, options);
  const packed = packedFixedWorkMetric(states, options);
  const packedDeadline = runPackedDeadlineMetric(states, options);
  return {
    contractVersion: BENCHMARK_CONTRACT_VERSION,
    generatedAt: new Date().toISOString(),
    environment: runtimeEnvironment(),
    options,
    stateCount: states.length,
    timed,
    packed,
    packedDeadline,
    memory: memoryUsage(),
  };
}

function combineChecksum(values: readonly number[]): number {
  let checksum = 2_166_136_261;
  for (const value of values) checksum = mixChecksum(checksum, value);
  return checksum;
}

function stateIdsChecksum(
  stateIds: readonly string[] | undefined,
): number | undefined {
  if (stateIds === undefined) return undefined;
  let checksum = 2_166_136_261;
  for (const stateId of stateIds) {
    if (typeof stateId !== "string") return undefined;
    checksum = mixChecksum(checksum, stringChecksum(stateId));
  }
  return checksum;
}

function aggregateChecksums(
  values: readonly (number | undefined)[],
): number | undefined {
  const checksums: number[] = [];
  for (const value of values) {
    if (value === undefined || !Number.isSafeInteger(value) || value < 0) {
      return undefined;
    }
    checksums.push(value);
  }
  return combineChecksum(checksums);
}

function duplicateMetricNames(metrics: readonly TimedMetric[]): string[] {
  const seen = new Set<string>();
  const duplicates = new Set<string>();
  for (const metric of metrics) {
    if (seen.has(metric.name)) duplicates.add(metric.name);
    else seen.add(metric.name);
  }
  return [...duplicates];
}

function optionalPackedConfiguration(metric: {
  readonly configuration?: PackedWorkConfiguration;
}): PackedWorkConfiguration | undefined {
  return metric.configuration;
}

function sameOptionalPackedConfiguration(
  left: PackedMetric,
  right: PackedMetric,
): boolean {
  const leftConfiguration = optionalPackedConfiguration(left);
  const rightConfiguration = optionalPackedConfiguration(right);
  if (leftConfiguration === undefined || rightConfiguration === undefined) {
    return leftConfiguration === rightConfiguration;
  }
  return (
    leftConfiguration.maxDepth === rightConfiguration.maxDepth &&
    leftConfiguration.maxNodes === rightConfiguration.maxNodes &&
    leftConfiguration.includesPositionLoad ===
      rightConfiguration.includesPositionLoad
  );
}

function sameNumberSequence(
  actual: readonly number[] | undefined,
  expected: readonly number[] | undefined,
): boolean {
  if (actual === undefined || expected === undefined) {
    return actual === expected;
  }
  return (
    actual.length === expected.length &&
    actual.every((value, index) => value === expected[index])
  );
}

function sameStringSequence(
  actual: readonly string[],
  expected: readonly string[],
): boolean {
  return (
    actual.length === expected.length &&
    actual.every((value, index) => value === expected[index])
  );
}

function assertDistribution(
  metric: Distribution,
  expectedCount: number,
  label: string,
): Distribution {
  const { samples } = metric;
  if (!Array.isArray(samples) || samples.length !== expectedCount) {
    throw new Error(
      `${label} must contain exactly ${expectedCount} duration samples`,
    );
  }
  for (const duration of samples) {
    if (
      typeof duration !== "number" ||
      !Number.isFinite(duration) ||
      duration < 0
    ) {
      throw new Error(`${label} contains an invalid duration sample`);
    }
  }
  const expected = distribution(samples);
  if (
    typeof metric.median !== "number" ||
    !Number.isFinite(metric.median) ||
    metric.median !== expected.median
  ) {
    throw new Error(`${label} median does not match its duration samples`);
  }
  if (
    typeof metric.p95 !== "number" ||
    !Number.isFinite(metric.p95) ||
    metric.p95 !== expected.p95
  ) {
    throw new Error(`${label} p95 does not match its duration samples`);
  }
  return expected;
}

function assertDerivedRate(
  value: unknown,
  expected: number,
  label: string,
): void {
  if (
    !Number.isFinite(expected) ||
    typeof value !== "number" ||
    !Number.isFinite(value) ||
    value !== expected
  ) {
    throw new Error(`${label} does not match its benchmark work`);
  }
}

function assertPackedRate(metric: PackedMetric, label: string): void {
  const totalDuration = metric.samples.reduce(
    (sum, duration) => sum + duration,
    0,
  );
  if (!(totalDuration > 0) || !Number.isFinite(totalDuration)) {
    throw new Error(`${label} must have a positive total duration`);
  }
  assertDerivedRate(
    metric.nodesPerMillisecond,
    metric.nodes / totalDuration,
    `${label} nodes per millisecond`,
  );
}

function assertChecksum(value: unknown, label: string): void {
  if (checksumValue(value) === null) {
    throw new Error(`${label} must be a nonnegative safe integer`);
  }
}

function assertPackedConfiguration(
  metric: PackedMetric,
  maxNodes: number,
  label: string,
): void {
  if (!packedConfigurationMatches(metric, maxNodes)) {
    throw new Error(`${label} has an invalid work configuration`);
  }
}

function benchmarkContractVersion(report: BenchmarkReport): unknown {
  return (report as { readonly contractVersion?: unknown }).contractVersion;
}

function assertRawBenchmarkReport(
  report: BenchmarkReport,
  expectation: BenchmarkReportValidation = {},
  legacy = false,
): void {
  const contractVersion = benchmarkContractVersion(report);
  if (
    legacy
      ? contractVersion !== undefined
      : contractVersion !== BENCHMARK_CONTRACT_VERSION
  ) {
    throw new Error(`unsupported benchmark contract version`);
  }
  if (
    !Number.isSafeInteger(report.options.samples) ||
    report.options.samples < 1
  ) {
    throw new Error("benchmark report samples must be a positive safe integer");
  }
  if (typeof report.options.smoke !== "boolean") {
    throw new Error("benchmark report smoke mode must be boolean");
  }
  if (
    report.options.sampleOffset !== undefined &&
    (!Number.isSafeInteger(report.options.sampleOffset) ||
      report.options.sampleOffset < 0)
  ) {
    throw new Error("benchmark report sample offset must be nonnegative");
  }
  if (
    report.options.deadlineStateIndices !== undefined &&
    (!Array.isArray(report.options.deadlineStateIndices) ||
      report.options.deadlineStateIndices.some(
        (index) => !Number.isSafeInteger(index) || index < 0,
      ))
  ) {
    throw new Error("benchmark report deadline state indices are invalid");
  }
  if (!Number.isSafeInteger(report.stateCount) || report.stateCount < 1) {
    throw new Error("benchmark report state count must be positive");
  }

  const expectedOptions = expectation.options;
  if (
    expectedOptions !== undefined &&
    (report.options.samples !== expectedOptions.samples ||
      report.options.smoke !== expectedOptions.smoke ||
      report.options.sampleOffset !== expectedOptions.sampleOffset ||
      !sameNumberSequence(
        report.options.deadlineStateIndices,
        expectedOptions.deadlineStateIndices,
      ))
  ) {
    throw new Error("benchmark runner returned inconsistent options");
  }
  if (
    expectation.stateCount !== undefined &&
    report.stateCount !== expectation.stateCount
  ) {
    throw new Error("benchmark runner returned an inconsistent state count");
  }

  if (!Array.isArray(report.timed)) {
    throw new Error("timed benchmark metrics must be an array");
  }
  const timedMetrics = report.timed as readonly TimedMetric[];
  const duplicates = duplicateMetricNames(timedMetrics);
  if (duplicates.length !== 0) {
    throw new Error(
      `duplicate timed benchmark metrics: ${duplicates.join(", ")}`,
    );
  }
  for (const metric of timedMetrics) {
    if (!Number.isSafeInteger(metric.operations) || metric.operations < 1) {
      throw new Error(
        `timed benchmark metric ${metric.name} has an invalid operation count`,
      );
    }
    assertChecksum(metric.checksum, `${metric.name} checksum`);
    if (!legacy || metric.timedSinkChecksum !== undefined) {
      assertChecksum(metric.timedSinkChecksum, `${metric.name} timed checksum`);
    }
    const summary = assertDistribution(
      metric,
      report.options.samples,
      `timed benchmark metric ${metric.name}`,
    );
    assertDerivedRate(
      metric.microsecondsPerOperation,
      (summary.median * 1_000) / metric.operations,
      `timed benchmark metric ${metric.name} microseconds per operation`,
    );
  }

  if (report.packed.name !== "automove.pro.fixedWork") {
    throw new Error("fixed-work benchmark metric has an invalid name");
  }
  assertDistribution(
    report.packed,
    report.options.samples,
    "fixed-work benchmark metric",
  );
  if (nodeCountValue(report.packed.nodes) === null) {
    throw new Error("fixed-work benchmark nodes must be nonnegative");
  }
  assertPackedRate(report.packed, "fixed-work benchmark metric");
  assertChecksum(report.packed.depthChecksum, "fixed-work depth checksum");
  assertChecksum(report.packed.moveChecksum, "fixed-work move checksum");
  if (!legacy || optionalPackedConfiguration(report.packed) !== undefined) {
    assertPackedConfiguration(
      report.packed,
      report.options.smoke ? PACKED_SMOKE_MAX_NODES : PACKED_FIXED_MAX_NODES,
      "fixed-work benchmark metric",
    );
  }

  const deadline = report.packedDeadline;
  if (deadline === undefined) {
    throw new Error("deadline benchmark metric is missing");
  }
  if (deadline.name !== "automove.pro.deadline") {
    throw new Error("deadline benchmark metric has an invalid name");
  }
  const expectedBudget = report.options.smoke
    ? PACKED_SMOKE_DEADLINE_BUDGET_MS
    : PACKED_DEADLINE_BUDGET_MS;
  if (deadline.budgetMs !== expectedBudget) {
    throw new Error("deadline benchmark metric has an invalid budget");
  }
  if (!legacy || optionalPackedConfiguration(deadline) !== undefined) {
    assertPackedConfiguration(
      deadline,
      Number.MAX_SAFE_INTEGER,
      "deadline benchmark metric",
    );
  }
  if (!Array.isArray(deadline.stateIds)) {
    throw new Error("deadline benchmark state IDs are missing");
  }
  for (const stateId of deadline.stateIds) {
    if (typeof stateId !== "string") {
      throw new Error("deadline benchmark state IDs are invalid");
    }
  }
  if (
    expectation.deadlineStateIds !== undefined &&
    !sameStringSequence(deadline.stateIds, expectation.deadlineStateIds)
  ) {
    throw new Error(
      "benchmark runner returned inconsistent deadline state IDs",
    );
  }
  assertDistribution(
    deadline,
    deadline.stateIds.length,
    "deadline benchmark metric",
  );
  if (nodeCountValue(deadline.nodes) === null) {
    throw new Error("deadline benchmark nodes must be nonnegative");
  }
  assertPackedRate(deadline, "deadline benchmark metric");
  assertChecksum(deadline.depthChecksum, "deadline depth checksum");
  assertChecksum(deadline.moveChecksum, "deadline move checksum");
  const expectedStateChecksum = stateIdsChecksum(deadline.stateIds);
  if (
    expectedStateChecksum === undefined ||
    checksumValue(deadline.stateChecksum) !== expectedStateChecksum
  ) {
    throw new Error("deadline benchmark state checksum is invalid");
  }
}

export function validateBenchmarkReport(
  report: BenchmarkReport,
  validation: BenchmarkReportValidation = {},
): void {
  const legacy =
    validation.allowLegacy === true &&
    benchmarkContractVersion(report) === undefined;
  assertRawBenchmarkReport(report, validation, legacy);
}

function assertAggregationContracts(reports: readonly BenchmarkReport[]): void {
  const first = reports[0];
  if (first === undefined) throw new Error("benchmark report set is empty");
  const firstDuplicates = duplicateMetricNames(first.timed);
  if (firstDuplicates.length !== 0) {
    throw new Error(
      `duplicate timed benchmark metrics: ${firstDuplicates.join(", ")}`,
    );
  }
  const firstMetrics = new Map(
    first.timed.map((metric) => [metric.name, metric]),
  );
  for (const report of reports) {
    if (report.contractVersion !== first.contractVersion) {
      throw new Error("inconsistent benchmark contract versions");
    }
    if (report.stateCount !== first.stateCount) {
      throw new Error("inconsistent benchmark state counts");
    }
    if (report.options.smoke !== first.options.smoke) {
      throw new Error("inconsistent benchmark smoke modes");
    }
    const duplicates = duplicateMetricNames(report.timed);
    if (duplicates.length !== 0) {
      throw new Error(
        `duplicate timed benchmark metrics: ${duplicates.join(", ")}`,
      );
    }
    if (report.timed.length !== firstMetrics.size) {
      throw new Error("inconsistent timed benchmark metric sets");
    }
    for (const metric of report.timed) {
      const expected = firstMetrics.get(metric.name);
      if (expected === undefined) {
        throw new Error("inconsistent timed benchmark metric sets");
      }
      if (metric.operations !== expected.operations) {
        throw new Error(
          `inconsistent operation count for timed metric ${metric.name}`,
        );
      }
    }
    if (
      report.packed.name !== first.packed.name ||
      report.packed.budgetMs !== first.packed.budgetMs ||
      !sameOptionalPackedConfiguration(report.packed, first.packed)
    ) {
      throw new Error("inconsistent fixed-work benchmark configuration");
    }
    const deadline = report.packedDeadline;
    const firstDeadline = first.packedDeadline;
    if (deadline === undefined || firstDeadline === undefined) {
      if (deadline !== firstDeadline) {
        throw new Error("inconsistent deadline benchmark presence");
      }
    } else if (
      deadline.name !== firstDeadline.name ||
      deadline.budgetMs !== firstDeadline.budgetMs ||
      !sameOptionalPackedConfiguration(deadline, firstDeadline)
    ) {
      throw new Error("inconsistent deadline benchmark configuration");
    }
  }
}

function aggregateTimedMetrics(
  reports: readonly BenchmarkReport[],
): TimedMetric[] {
  const names = new Set<string>();
  for (const report of reports) {
    for (const metric of report.timed) names.add(metric.name);
  }
  const metrics: TimedMetric[] = [];
  for (const name of names) {
    const matching = reports.flatMap((report) =>
      report.timed.filter((metric) => metric.name === name),
    );
    const first = matching[0];
    if (first === undefined) continue;
    const samples = matching.flatMap((metric) => metric.samples);
    const summary = distribution(samples);
    const checksum = aggregateChecksums(
      matching.map((metric) => metric.checksum),
    );
    const timedSinkChecksum = aggregateChecksums(
      matching.map((metric) => metric.timedSinkChecksum),
    );
    metrics.push({
      name,
      operations: first.operations,
      ...(checksum === undefined ? {} : { checksum }),
      ...(timedSinkChecksum === undefined ? {} : { timedSinkChecksum }),
      ...summary,
      microsecondsPerOperation: (summary.median * 1_000) / first.operations,
    });
  }
  return metrics;
}

function aggregatePackedMetrics(
  reports: readonly BenchmarkReport[],
  select: (report: BenchmarkReport) => PackedMetric | undefined,
  validateStateCoverage: boolean,
): PackedMetric | undefined {
  const matching = reports.map(select).filter((metric) => metric !== undefined);
  const first = matching[0];
  if (first === undefined) return undefined;
  const samples = matching.flatMap((metric) => metric.samples);
  const nodes = matching.reduce((sum, metric) => sum + metric.nodes, 0);
  const stateIds = matching.flatMap((metric) => metric.stateIds ?? []);
  const summary = distribution(samples);
  const totalDuration = samples.reduce((sum, value) => sum + value, 0);
  const depthChecksum = aggregateChecksums(
    matching.map((metric) => metric.depthChecksum),
  );
  const moveChecksum = aggregateChecksums(
    matching.map((metric) => metric.moveChecksum),
  );
  let aggregateStateChecksum: number | undefined;
  if (
    validateStateCoverage &&
    matching.every((metric) => {
      const expected = stateIdsChecksum(metric.stateIds);
      return (
        expected !== undefined &&
        checksumValue(metric.stateChecksum) === expected
      );
    })
  ) {
    aggregateStateChecksum = stateIdsChecksum(stateIds);
  }
  return {
    name: first.name,
    ...(first.budgetMs === undefined ? {} : { budgetMs: first.budgetMs }),
    nodes,
    nodesPerMillisecond: nodes / totalDuration,
    ...(depthChecksum === undefined ? {} : { depthChecksum }),
    ...(moveChecksum === undefined ? {} : { moveChecksum }),
    configuration: first.configuration,
    ...(stateIds.length === 0
      ? {}
      : {
          stateIds,
          ...(aggregateStateChecksum === undefined
            ? {}
            : { stateChecksum: aggregateStateChecksum }),
        }),
    ...summary,
  };
}

function aggregateMemory(reports: readonly BenchmarkReport[]): MemoryMetric {
  let heapUsed: number | undefined;
  let rss: number | undefined;
  for (const report of reports) {
    if (report.memory.heapUsed !== undefined) {
      heapUsed = Math.max(heapUsed ?? 0, report.memory.heapUsed);
    }
    if (report.memory.rss !== undefined) {
      rss = Math.max(rss ?? 0, report.memory.rss);
    }
  }
  return {
    scope: "shared-runtime",
    measurement: "maximum-of-end-snapshots",
    isPeak: false,
    attributableToImplementation: false,
    comparableAcrossImplementations: false,
    ...(heapUsed === undefined ? {} : { heapUsed }),
    ...(rss === undefined ? {} : { rss }),
  };
}

function aggregateBenchmarkReportsInternal(
  reports: readonly BenchmarkReport[],
  samples: number,
  smoke: boolean,
  allowLegacy: boolean,
): BenchmarkReport {
  if (!Number.isSafeInteger(samples) || samples < 1) {
    throw new Error("aggregate samples must be a positive safe integer");
  }
  if (typeof smoke !== "boolean") {
    throw new Error("aggregate smoke mode must be boolean");
  }
  let reportSamples = 0;
  for (const report of reports) {
    validateBenchmarkReport(report, { allowLegacy });
    if (report.options.smoke !== smoke) {
      throw new Error("benchmark report smoke mode does not match aggregate");
    }
    reportSamples += report.options.samples;
    if (!Number.isSafeInteger(reportSamples)) {
      throw new Error("aggregate benchmark sample count is too large");
    }
  }
  if (reportSamples !== samples) {
    throw new Error(
      `aggregate expected ${samples} samples but reports contain ${reportSamples}`,
    );
  }
  assertAggregationContracts(reports);
  const first = reports[0];
  if (first === undefined) throw new Error("benchmark report set is empty");
  const packed = aggregatePackedMetrics(
    reports,
    (report) => report.packed,
    false,
  );
  if (packed === undefined) throw new Error("fixed-work metric is missing");
  const packedDeadline = aggregatePackedMetrics(
    reports,
    (report) => report.packedDeadline,
    true,
  );
  return {
    contractVersion: first.contractVersion,
    generatedAt: new Date().toISOString(),
    environment: first.environment,
    options: { samples, smoke },
    stateCount: first.stateCount,
    timed: aggregateTimedMetrics(reports),
    packed,
    ...(packedDeadline === undefined ? {} : { packedDeadline }),
    memory: aggregateMemory(reports),
  };
}

export function aggregateBenchmarkReports(
  reports: readonly BenchmarkReport[],
  samples: number,
  smoke: boolean,
): BenchmarkReport {
  return aggregateBenchmarkReportsInternal(reports, samples, smoke, false);
}

function samplesByBatch(samples: number, batches: number): number[] {
  const result = new Array<number>(batches).fill(Math.floor(samples / batches));
  for (let index = 0; index < samples % batches; index += 1) {
    result[index] = (result[index] ?? 0) + 1;
  }
  return result;
}

function deadlineStateIndicesForSample(
  stateCount: number,
  sample: number,
  samples: number,
  smoke: boolean,
): number[] {
  if (smoke) return [sample % stateCount];
  const start = Math.floor((sample * stateCount) / samples);
  const end = Math.floor(((sample + 1) * stateCount) / samples);
  if (start !== end) {
    return Array.from({ length: end - start }, (_, index) => start + index);
  }
  return [sample % stateCount];
}

function checksumValue(value: unknown): number | null {
  return Number.isSafeInteger(value) && (value as number) >= 0
    ? (value as number)
    : null;
}

function operationCountValue(value: unknown): number | null {
  return Number.isSafeInteger(value) && (value as number) > 0
    ? (value as number)
    : null;
}

function nodeCountValue(value: unknown): number | null {
  return Number.isSafeInteger(value) && (value as number) >= 0
    ? (value as number)
    : null;
}

function packedConfigurationMatches(
  metric: PackedMetric | undefined,
  maxNodes: number,
): boolean {
  if (metric === undefined) return false;
  const configuration = optionalPackedConfiguration(metric);
  return (
    configuration?.maxDepth === PACKED_MAX_DEPTH &&
    configuration.maxNodes === maxNodes &&
    (configuration as Partial<PackedWorkConfiguration>).includesPositionLoad ===
      false
  );
}

function stateIdsMatch(
  actual: readonly string[] | undefined,
  expected: readonly string[],
): boolean {
  return (
    actual !== undefined &&
    expected.length > 0 &&
    actual.length === expected.length &&
    actual.every((stateId, index) => stateId === expected[index])
  );
}

export function benchmarkMetricParity(
  baseline: BenchmarkReport,
  candidate: BenchmarkReport,
  expected: BenchmarkExpectedContract,
): BenchmarkMetricParity {
  const duplicateBaselineTimedMetrics = duplicateMetricNames(baseline.timed);
  const duplicateCandidateTimedMetrics = duplicateMetricNames(candidate.timed);
  const baselineMetrics = new Map(
    baseline.timed.map((metric) => [metric.name, metric]),
  );
  const candidateMetrics = new Map(
    candidate.timed.map((metric) => [metric.name, metric]),
  );
  const baselineNames = [...baselineMetrics.keys()];
  const candidateNames = [...candidateMetrics.keys()];
  const requiredTimedMetricNames = new Set<string>(REQUIRED_TIMED_METRIC_NAMES);
  const missingRequiredBaselineTimedMetrics =
    REQUIRED_TIMED_METRIC_NAMES.filter((name) => !baselineMetrics.has(name));
  const missingRequiredCandidateTimedMetrics =
    REQUIRED_TIMED_METRIC_NAMES.filter((name) => !candidateMetrics.has(name));
  const unexpectedBaselineTimedMetrics = baselineNames.filter(
    (name) => !requiredTimedMetricNames.has(name),
  );
  const unexpectedCandidateTimedMetrics = candidateNames.filter(
    (name) => !requiredTimedMetricNames.has(name),
  );
  const requiredTimedMetricSetMatch =
    missingRequiredBaselineTimedMetrics.length === 0 &&
    missingRequiredCandidateTimedMetrics.length === 0 &&
    unexpectedBaselineTimedMetrics.length === 0 &&
    unexpectedCandidateTimedMetrics.length === 0 &&
    duplicateBaselineTimedMetrics.length === 0 &&
    duplicateCandidateTimedMetrics.length === 0;
  const comparableTimedMetrics = baselineNames.filter((name) =>
    candidateMetrics.has(name),
  );
  const missingFromBaseline = candidateNames.filter(
    (name) => !baselineMetrics.has(name),
  );
  const missingFromCandidate = baselineNames.filter(
    (name) => !candidateMetrics.has(name),
  );
  const checksumMismatches = comparableTimedMetrics.flatMap((name) => {
    const baselineChecksum = checksumValue(baselineMetrics.get(name)?.checksum);
    const candidateChecksum = checksumValue(
      candidateMetrics.get(name)?.checksum,
    );
    return baselineChecksum !== null &&
      candidateChecksum !== null &&
      baselineChecksum === candidateChecksum
      ? []
      : [
          {
            name,
            baseline: baselineChecksum,
            candidate: candidateChecksum,
          },
        ];
  });
  const timedSinkChecksumMismatches = comparableTimedMetrics.flatMap((name) => {
    const baselineChecksum = checksumValue(
      baselineMetrics.get(name)?.timedSinkChecksum,
    );
    const candidateChecksum = checksumValue(
      candidateMetrics.get(name)?.timedSinkChecksum,
    );
    return baselineChecksum !== null &&
      candidateChecksum !== null &&
      baselineChecksum === candidateChecksum
      ? []
      : [
          {
            name,
            baseline: baselineChecksum,
            candidate: candidateChecksum,
          },
        ];
  });
  const operationMismatches = comparableTimedMetrics.flatMap((name) => {
    const baselineOperations = operationCountValue(
      baselineMetrics.get(name)?.operations,
    );
    const candidateOperations = operationCountValue(
      candidateMetrics.get(name)?.operations,
    );
    return baselineOperations !== null &&
      candidateOperations !== null &&
      baselineOperations === candidateOperations
      ? []
      : [
          {
            name,
            baseline: baselineOperations,
            candidate: candidateOperations,
          },
        ];
  });
  const contractVersionMatch =
    baseline.contractVersion === BENCHMARK_CONTRACT_VERSION &&
    candidate.contractVersion === BENCHMARK_CONTRACT_VERSION;
  const stateCountMatch =
    Number.isSafeInteger(expected.stateCount) &&
    expected.stateCount > 0 &&
    baseline.stateCount === expected.stateCount &&
    candidate.stateCount === expected.stateCount;
  const fixedMaxNodes = baseline.options.smoke
    ? PACKED_SMOKE_MAX_NODES
    : PACKED_FIXED_MAX_NODES;
  const fixedWorkConfigurationMatch =
    baseline.options.smoke === candidate.options.smoke &&
    baseline.packed.name === "automove.pro.fixedWork" &&
    candidate.packed.name === "automove.pro.fixedWork" &&
    packedConfigurationMatches(baseline.packed, fixedMaxNodes) &&
    packedConfigurationMatches(candidate.packed, fixedMaxNodes);
  const fixedWorkDepthChecksumMatch =
    checksumValue(baseline.packed.depthChecksum) !== null &&
    baseline.packed.depthChecksum === candidate.packed.depthChecksum;
  const fixedWorkMoveChecksumMatch =
    checksumValue(baseline.packed.moveChecksum) !== null &&
    baseline.packed.moveChecksum === candidate.packed.moveChecksum;
  const fixedWorkNodeCountMatch =
    nodeCountValue(baseline.packed.nodes) !== null &&
    baseline.packed.nodes === candidate.packed.nodes;
  const baselineDeadline = baseline.packedDeadline;
  const candidateDeadline = candidate.packedDeadline;
  const deadlineConfigurationMatch =
    baselineDeadline?.name === "automove.pro.deadline" &&
    candidateDeadline?.name === "automove.pro.deadline" &&
    packedConfigurationMatches(baselineDeadline, Number.MAX_SAFE_INTEGER) &&
    packedConfigurationMatches(candidateDeadline, Number.MAX_SAFE_INTEGER);
  const expectedDeadlineBudget = baseline.options.smoke
    ? PACKED_SMOKE_DEADLINE_BUDGET_MS
    : PACKED_DEADLINE_BUDGET_MS;
  const deadlineBudgetMatch =
    baseline.options.smoke === candidate.options.smoke &&
    baselineDeadline?.budgetMs === expectedDeadlineBudget &&
    candidateDeadline?.budgetMs === expectedDeadlineBudget;
  const deadlineProgressFieldsPresent =
    nodeCountValue(baselineDeadline?.nodes) !== null &&
    nodeCountValue(candidateDeadline?.nodes) !== null &&
    checksumValue(baselineDeadline?.depthChecksum) !== null &&
    checksumValue(candidateDeadline?.depthChecksum) !== null;
  const baselineDeadlineStateCoverageMatch = stateIdsMatch(
    baselineDeadline?.stateIds,
    expected.deadlineStateIds,
  );
  const candidateDeadlineStateCoverageMatch = stateIdsMatch(
    candidateDeadline?.stateIds,
    expected.deadlineStateIds,
  );
  const deadlineStateCoverageMatch =
    baselineDeadlineStateCoverageMatch && candidateDeadlineStateCoverageMatch;
  const expectedDeadlineStateChecksum = stateIdsChecksum(
    expected.deadlineStateIds,
  );
  const deadlineStateChecksumMatch =
    expectedDeadlineStateChecksum !== undefined &&
    checksumValue(baselineDeadline?.stateChecksum) ===
      expectedDeadlineStateChecksum &&
    checksumValue(candidateDeadline?.stateChecksum) ===
      expectedDeadlineStateChecksum;
  const baselineDeadlineMoveChecksum = checksumValue(
    baselineDeadline?.moveChecksum,
  );
  const candidateDeadlineMoveChecksum = checksumValue(
    candidateDeadline?.moveChecksum,
  );
  const deadlineMoveChecksumsPresent =
    baselineDeadlineMoveChecksum !== null &&
    candidateDeadlineMoveChecksum !== null;
  const deadlineMoveChecksumMatch =
    deadlineMoveChecksumsPresent &&
    baselineDeadlineMoveChecksum === candidateDeadlineMoveChecksum;
  const complete =
    contractVersionMatch &&
    stateCountMatch &&
    requiredTimedMetricSetMatch &&
    missingFromBaseline.length === 0 &&
    missingFromCandidate.length === 0 &&
    duplicateBaselineTimedMetrics.length === 0 &&
    duplicateCandidateTimedMetrics.length === 0 &&
    checksumMismatches.length === 0 &&
    timedSinkChecksumMismatches.length === 0 &&
    operationMismatches.length === 0 &&
    fixedWorkConfigurationMatch &&
    fixedWorkDepthChecksumMatch &&
    fixedWorkMoveChecksumMatch &&
    fixedWorkNodeCountMatch &&
    deadlineConfigurationMatch &&
    deadlineBudgetMatch &&
    deadlineProgressFieldsPresent &&
    deadlineStateCoverageMatch &&
    deadlineStateChecksumMatch &&
    deadlineMoveChecksumsPresent;
  return {
    complete,
    contractVersionMatch,
    stateCountMatch,
    requiredTimedMetricSetMatch,
    missingRequiredBaselineTimedMetrics,
    missingRequiredCandidateTimedMetrics,
    unexpectedBaselineTimedMetrics,
    unexpectedCandidateTimedMetrics,
    comparableTimedMetrics,
    missingFromBaseline,
    missingFromCandidate,
    duplicateBaselineTimedMetrics,
    duplicateCandidateTimedMetrics,
    checksumMismatches,
    timedSinkChecksumMismatches,
    operationMismatches,
    fixedWorkConfigurationMatch,
    fixedWorkDepthChecksumMatch,
    fixedWorkMoveChecksumMatch,
    fixedWorkNodeCountMatch,
    deadlineConfigurationMatch,
    deadlineBudgetMatch,
    deadlineProgressFieldsPresent,
    baselineDeadlineStateCoverageMatch,
    candidateDeadlineStateCoverageMatch,
    deadlineStateCoverageMatch,
    deadlineStateChecksumMatch,
    deadlineMoveChecksumMatch,
    ...(complete
      ? {}
      : {
          action:
            "Investigate the reported contract and output mismatches; keep the existing baseline unchanged until candidate behavior differences are understood.",
        }),
  };
}

export function runInterleavedBenchmark(
  states: readonly BenchmarkState[],
  baselineRunner: BenchmarkRunner,
  candidateRunner: BenchmarkRunner,
  options: InterleavedBenchmarkOptions,
): InterleavedBenchmarkReport {
  if (states.length === 0) {
    throw new Error("benchmark states must not be empty");
  }
  if (!Number.isSafeInteger(options.samples) || options.samples < 1) {
    throw new RangeError("benchmark samples must be a positive safe integer");
  }
  if (
    !Number.isSafeInteger(options.batches) ||
    options.batches < 1 ||
    options.batches > options.samples
  ) {
    throw new RangeError("benchmark batches must be from one through samples");
  }
  const warmupOptions = { samples: 1, smoke: true } as const;
  baselineRunner(states, warmupOptions);
  candidateRunner(states, warmupOptions);
  const baselineReports: BenchmarkReport[] = [];
  const candidateReports: BenchmarkReport[] = [];
  const executionOrder: string[] = [];
  const batchSamples = samplesByBatch(options.samples, options.batches);
  const deadlineStateSchedule = Array.from(
    { length: options.samples },
    (_, sample) =>
      deadlineStateIndicesForSample(
        states.length,
        sample,
        options.samples,
        options.smoke,
      ),
  );
  const deadlineStateScheduleIds = deadlineStateSchedule.map((indices) =>
    indices.flatMap((index) => states[index]?.id ?? []),
  );
  let sampleOffset = 0;
  for (const batchSize of batchSamples) {
    for (let sample = 0; sample < batchSize; sample += 1) {
      const runOptions = {
        samples: 1,
        smoke: options.smoke,
        sampleOffset,
        deadlineStateIndices: deadlineStateSchedule[sampleOffset] ?? [],
      };
      const baselineFirst = sampleOffset % 2 === 0;
      const expectedDeadlineStateIds = deadlineStateScheduleIds[sampleOffset];
      if (expectedDeadlineStateIds === undefined) {
        throw new RangeError("missing deadline benchmark schedule");
      }
      const run = (
        runner: BenchmarkRunner,
        allowLegacy: boolean,
      ): BenchmarkReport => {
        const report = runner(states, runOptions);
        validateBenchmarkReport(report, {
          options: runOptions,
          stateCount: states.length,
          deadlineStateIds: expectedDeadlineStateIds,
          allowLegacy,
        });
        return report;
      };
      const allowLegacyBaseline = options.allowLegacyBaseline === true;
      if (baselineFirst) {
        executionOrder.push("baseline", "candidate");
        baselineReports.push(run(baselineRunner, allowLegacyBaseline));
        candidateReports.push(run(candidateRunner, false));
      } else {
        executionOrder.push("candidate", "baseline");
        candidateReports.push(run(candidateRunner, false));
        baselineReports.push(run(baselineRunner, allowLegacyBaseline));
      }
      sampleOffset += 1;
    }
  }
  const baseline = aggregateBenchmarkReportsInternal(
    baselineReports,
    options.samples,
    options.smoke,
    options.allowLegacyBaseline === true,
  );
  const candidate = aggregateBenchmarkReports(
    candidateReports,
    options.samples,
    options.smoke,
  );
  const parity = benchmarkMetricParity(baseline, candidate, {
    stateCount: states.length,
    deadlineStateIds: deadlineStateScheduleIds.flatMap((stateIds) => stateIds),
  });
  return {
    contractVersion: BENCHMARK_CONTRACT_VERSION,
    generatedAt: new Date().toISOString(),
    environment: candidate.environment,
    options: {
      samples: options.samples,
      batches: options.batches,
      batchSamples,
      smoke: options.smoke,
      warmupRuns: 1,
      deadlineStateSchedule: deadlineStateScheduleIds,
    },
    stateCount: states.length,
    executionOrder,
    metricParity: parity,
    implementations: { baseline, candidate },
  };
}
