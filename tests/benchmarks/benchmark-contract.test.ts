import { describe, expect, it } from "vitest";

import {
  aggregateBenchmarkReports,
  BENCHMARK_CONTRACT_VERSION,
  benchmarkMetricParity as compareBenchmarkMetricParity,
  runInterleavedBenchmark,
  validateBenchmarkReport,
  type BenchmarkExpectedContract,
  type BenchmarkReport,
  type BenchmarkRunner,
  type BenchmarkState,
} from "../../benchmarks/suite.js";

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

const EXPECTED_CONTRACT: BenchmarkExpectedContract = {
  stateCount: 2,
  deadlineStateIds: ["initial-Classic"],
};

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

function stateIdsChecksum(stateIds: readonly string[]): number {
  let checksum = 2_166_136_261;
  for (const stateId of stateIds) {
    checksum = mixChecksum(checksum, stringChecksum(stateId));
  }
  return checksum;
}

function benchmarkMetricParity(
  baseline: BenchmarkReport,
  candidate: BenchmarkReport,
  expected: BenchmarkExpectedContract = EXPECTED_CONTRACT,
) {
  return compareBenchmarkMetricParity(baseline, candidate, expected);
}

function benchmarkReport(): BenchmarkReport {
  return {
    contractVersion: BENCHMARK_CONTRACT_VERSION,
    generatedAt: "2026-08-02T00:00:00.000Z",
    environment: { runtime: "test", userAgent: "test" },
    options: { samples: 1, smoke: true },
    stateCount: 2,
    timed: REQUIRED_TIMED_METRIC_NAMES.map((name, index) => ({
      name,
      operations: 2_000,
      checksum: 11 + index,
      timedSinkChecksum: 12 + index,
      samples: [1],
      median: 1,
      p95: 1,
      microsecondsPerOperation: 0.5,
    })),
    packed: {
      name: "automove.pro.fixedWork",
      nodes: 50_000,
      nodesPerMillisecond: 2_000,
      depthChecksum: 21,
      moveChecksum: 22,
      configuration: {
        maxDepth: 40,
        maxNodes: 25_000,
        includesPositionLoad: false,
      },
      samples: [25],
      median: 25,
      p95: 25,
    },
    packedDeadline: {
      name: "automove.pro.deadline",
      budgetMs: 20,
      nodes: 40_000,
      nodesPerMillisecond: 2_000,
      depthChecksum: 31,
      moveChecksum: 32,
      configuration: {
        maxDepth: 40,
        maxNodes: Number.MAX_SAFE_INTEGER,
        includesPositionLoad: false,
      },
      stateIds: ["initial-Classic"],
      stateChecksum: stateIdsChecksum(["initial-Classic"]),
      samples: [20],
      median: 20,
      p95: 20,
    },
    memory: {
      scope: "shared-runtime",
      measurement: "end-snapshot",
      isPeak: false,
      attributableToImplementation: false,
      comparableAcrossImplementations: false,
    },
  };
}

function invocationReport(
  states: readonly BenchmarkState[],
  options: Parameters<BenchmarkRunner>[1],
  deadlineStateIds: readonly string[],
): BenchmarkReport {
  const report = benchmarkReport();
  const samples = Array.from({ length: options.samples }, () => 1);
  const deadlineSamples = deadlineStateIds.map(() => 20);
  const deadlineDuration = deadlineSamples.reduce(
    (sum, duration) => sum + duration,
    0,
  );
  return {
    ...report,
    options,
    stateCount: states.length,
    timed: report.timed.map((metric) => ({
      ...metric,
      samples,
      median: 1,
      p95: 1,
      microsecondsPerOperation: 1_000 / metric.operations,
    })),
    packed: {
      ...report.packed,
      samples,
      median: 1,
      p95: 1,
      nodesPerMillisecond: report.packed.nodes / samples.length,
    },
    packedDeadline: {
      ...deadlineMetric(report),
      stateIds: deadlineStateIds,
      stateChecksum: stateIdsChecksum(deadlineStateIds),
      samples: deadlineSamples,
      median: deadlineSamples[0] ?? 0,
      p95: deadlineSamples[0] ?? 0,
      nodesPerMillisecond:
        deadlineDuration === 0
          ? 0
          : deadlineMetric(report).nodes / deadlineDuration,
    },
  };
}

function legacyInvocationReport(
  states: readonly BenchmarkState[],
  options: Parameters<BenchmarkRunner>[1],
  deadlineStateIds: readonly string[],
): BenchmarkReport {
  const report = invocationReport(states, options, deadlineStateIds);
  const packed = { ...report.packed } as Record<string, unknown>;
  Reflect.deleteProperty(packed, "configuration");
  const packedDeadline = { ...deadlineMetric(report) } as Record<
    string,
    unknown
  >;
  Reflect.deleteProperty(packedDeadline, "configuration");
  const legacy = {
    ...report,
    timed: report.timed.map((metric) => {
      const legacyMetric = { ...metric } as Record<string, unknown>;
      Reflect.deleteProperty(legacyMetric, "timedSinkChecksum");
      return legacyMetric;
    }),
    packed,
    packedDeadline,
    memory: { heapUsed: 1_024, rss: 2_048 },
  } as Record<string, unknown>;
  Reflect.deleteProperty(legacy, "contractVersion");
  return legacy as unknown as BenchmarkReport;
}

function firstTimedMetric(report: BenchmarkReport) {
  const metric = report.timed[0];
  if (metric === undefined) throw new Error("missing test timed metric");
  return metric;
}

function deadlineMetric(report: BenchmarkReport) {
  const metric = report.packedDeadline;
  if (metric === undefined) throw new Error("missing test deadline metric");
  return metric;
}

function requiredNumber(value: number | undefined): number {
  if (value === undefined) throw new Error("missing test number");
  return value;
}

function withoutTimedField(
  report: BenchmarkReport,
  field: "checksum" | "timedSinkChecksum",
): BenchmarkReport {
  const metric = { ...firstTimedMetric(report) } as Record<string, unknown>;
  Reflect.deleteProperty(metric, field);
  return {
    ...report,
    timed: [metric, ...report.timed.slice(1)],
  } as unknown as BenchmarkReport;
}

describe("benchmark performance contract", () => {
  it("accepts equal reports", () => {
    expect(benchmarkMetricParity(benchmarkReport(), benchmarkReport())).toEqual(
      expect.objectContaining({
        complete: true,
        contractVersionMatch: true,
        stateCountMatch: true,
        requiredTimedMetricSetMatch: true,
        fixedWorkConfigurationMatch: true,
        deadlineConfigurationMatch: true,
        deadlineBudgetMatch: true,
      }),
    );
  });

  it("rejects equal reports with missing or unexpected timed workloads", () => {
    const missingBaseline = benchmarkReport();
    const missingCandidate = benchmarkReport();
    const missing = benchmarkMetricParity(
      { ...missingBaseline, timed: missingBaseline.timed.slice(1) },
      { ...missingCandidate, timed: missingCandidate.timed.slice(1) },
    );
    const extraMetric = {
      ...firstTimedMetric(benchmarkReport()),
      name: "game.unexpected",
    };
    const unexpectedBaseline = benchmarkReport();
    const unexpectedCandidate = benchmarkReport();
    const unexpected = benchmarkMetricParity(
      {
        ...unexpectedBaseline,
        timed: [...unexpectedBaseline.timed, extraMetric],
      },
      {
        ...unexpectedCandidate,
        timed: [...unexpectedCandidate.timed, { ...extraMetric }],
      },
    );

    expect(missing).toEqual(
      expect.objectContaining({
        complete: false,
        requiredTimedMetricSetMatch: false,
        missingRequiredBaselineTimedMetrics: ["game.fromFen"],
        missingRequiredCandidateTimedMetrics: ["game.fromFen"],
      }),
    );
    expect(unexpected).toEqual(
      expect.objectContaining({
        complete: false,
        requiredTimedMetricSetMatch: false,
        unexpectedBaselineTimedMetrics: ["game.unexpected"],
        unexpectedCandidateTimedMetrics: ["game.unexpected"],
      }),
    );
  });

  it("rejects missing full and timed-sink checksums", () => {
    const baseline = benchmarkReport();
    const missingFull = benchmarkMetricParity(
      withoutTimedField(baseline, "checksum"),
      benchmarkReport(),
    );
    const missingSink = benchmarkMetricParity(
      baseline,
      withoutTimedField(benchmarkReport(), "timedSinkChecksum"),
    );

    expect(missingFull.complete).toBe(false);
    expect(missingFull.checksumMismatches).toEqual([
      { name: "game.fromFen", baseline: null, candidate: 11 },
    ]);
    expect(missingSink.complete).toBe(false);
    expect(missingSink.timedSinkChecksumMismatches).toEqual([
      { name: "game.fromFen", baseline: 12, candidate: null },
    ]);
  });

  it("rejects timed workload and report contract mismatches", () => {
    const baseline = benchmarkReport();
    const timed = firstTimedMetric(baseline);
    const operationMismatch = benchmarkMetricParity(baseline, {
      ...benchmarkReport(),
      timed: [
        { ...timed, operations: timed.operations + 1 },
        ...baseline.timed.slice(1),
      ],
    });
    const sinkMismatch = benchmarkMetricParity(baseline, {
      ...benchmarkReport(),
      timed: [{ ...timed, timedSinkChecksum: 99 }, ...baseline.timed.slice(1)],
    });
    const stateCountMismatch = benchmarkMetricParity(baseline, {
      ...benchmarkReport(),
      stateCount: baseline.stateCount + 1,
    });
    const equalWrongStateCount = benchmarkMetricParity(
      { ...benchmarkReport(), stateCount: 1 },
      { ...benchmarkReport(), stateCount: 1 },
    );
    const versionMismatch = benchmarkMetricParity(baseline, {
      ...benchmarkReport(),
      contractVersion: BENCHMARK_CONTRACT_VERSION + 1,
    });

    expect(operationMismatch.operationMismatches).toHaveLength(1);
    expect(sinkMismatch.timedSinkChecksumMismatches).toHaveLength(1);
    expect(stateCountMismatch.stateCountMatch).toBe(false);
    expect(equalWrongStateCount.stateCountMatch).toBe(false);
    expect(versionMismatch.contractVersionMatch).toBe(false);
    expect(
      [
        operationMismatch,
        sinkMismatch,
        stateCountMismatch,
        equalWrongStateCount,
        versionMismatch,
      ].map((parity) => parity.complete),
    ).toEqual([false, false, false, false, false]);
  });

  it("rejects duplicate timed metric names", () => {
    const baseline = benchmarkReport();
    const metric = firstTimedMetric(baseline);
    const duplicate = {
      ...baseline,
      timed: [...baseline.timed, { ...metric }],
    };
    const parity = benchmarkMetricParity(duplicate, benchmarkReport());

    expect(parity.complete).toBe(false);
    expect(parity.duplicateBaselineTimedMetrics).toEqual(["game.fromFen"]);
  });

  it("requires fixed-work and deadline configurations", () => {
    const baseline = benchmarkReport();
    const fixedConfigurationMismatch = benchmarkMetricParity(baseline, {
      ...benchmarkReport(),
      packed: {
        ...baseline.packed,
        configuration: {
          ...baseline.packed.configuration,
          maxNodes: baseline.packed.configuration.maxNodes + 1,
        },
      },
    });
    const deadline = deadlineMetric(baseline);
    const deadlineConfigurationMismatch = benchmarkMetricParity(baseline, {
      ...benchmarkReport(),
      packedDeadline: {
        ...deadline,
        configuration: {
          ...deadline.configuration,
          includesPositionLoad: true,
        },
      },
    });
    const deadlineBudgetMismatch = benchmarkMetricParity(baseline, {
      ...benchmarkReport(),
      packedDeadline: { ...deadline, budgetMs: 21 },
    });

    expect(fixedConfigurationMismatch.fixedWorkConfigurationMatch).toBe(false);
    expect(deadlineConfigurationMismatch.deadlineConfigurationMatch).toBe(
      false,
    );
    expect(deadlineBudgetMismatch.deadlineBudgetMatch).toBe(false);
  });

  it("keeps fixed-work outputs exact while treating deadline outputs as diagnostics", () => {
    const baseline = benchmarkReport();
    const fixedNodes = benchmarkMetricParity(baseline, {
      ...benchmarkReport(),
      packed: { ...baseline.packed, nodes: baseline.packed.nodes + 1 },
    });
    const fixedDepth = benchmarkMetricParity(baseline, {
      ...benchmarkReport(),
      packed: { ...baseline.packed, depthChecksum: 99 },
    });
    const fixedMove = benchmarkMetricParity(baseline, {
      ...benchmarkReport(),
      packed: { ...baseline.packed, moveChecksum: 99 },
    });
    const deadline = deadlineMetric(baseline);
    const deeperDeadline = benchmarkMetricParity(baseline, {
      ...benchmarkReport(),
      packedDeadline: {
        ...deadline,
        nodes: deadline.nodes + 10_000,
        depthChecksum: requiredNumber(deadline.depthChecksum) + 1,
        moveChecksum: requiredNumber(deadline.moveChecksum) + 1,
      },
    });

    expect(fixedNodes.fixedWorkNodeCountMatch).toBe(false);
    expect(fixedDepth.fixedWorkDepthChecksumMatch).toBe(false);
    expect(fixedMove.fixedWorkMoveChecksumMatch).toBe(false);
    expect(deeperDeadline.complete).toBe(true);
    expect(deeperDeadline.deadlineMoveChecksumMatch).toBe(false);
  });

  it("keeps the existing baseline unchanged when output parity fails", () => {
    const baseline = benchmarkReport();
    const parity = benchmarkMetricParity(baseline, {
      ...benchmarkReport(),
      packed: { ...baseline.packed, moveChecksum: 99 },
    });

    expect(parity.complete).toBe(false);
    expect(parity.action).toMatch(/^Investigate/u);
    expect(parity.action).toContain("keep the existing baseline unchanged");
    expect(parity.action).not.toContain("--baseline");
  });

  it("requires deadline diagnostic fields without comparing their values", () => {
    const baseline = benchmarkReport();
    const deadline = { ...deadlineMetric(benchmarkReport()) } as Record<
      string,
      unknown
    >;
    Reflect.deleteProperty(deadline, "depthChecksum");
    const missingProgress = benchmarkMetricParity(baseline, {
      ...benchmarkReport(),
      packedDeadline: deadline,
    } as unknown as BenchmarkReport);
    const missingMoveReport = benchmarkReport();
    const missingMove = {
      ...deadlineMetric(missingMoveReport),
    } as Record<string, unknown>;
    Reflect.deleteProperty(missingMove, "moveChecksum");
    const missingMoveParity = benchmarkMetricParity(baseline, {
      ...missingMoveReport,
      packedDeadline: missingMove,
    } as unknown as BenchmarkReport);

    expect(missingProgress.complete).toBe(false);
    expect(missingProgress.deadlineProgressFieldsPresent).toBe(false);
    expect(missingMoveParity.complete).toBe(false);
    expect(missingMoveParity.deadlineProgressFieldsPresent).toBe(true);
    expect(missingMoveParity.deadlineMoveChecksumMatch).toBe(false);
  });

  it("rejects inconsistent aggregation contracts and duplicate names", () => {
    const baseline = benchmarkReport();
    const timed = firstTimedMetric(baseline);
    const inconsistentOperations = {
      ...benchmarkReport(),
      timed: [
        {
          ...timed,
          operations: timed.operations + 1,
          microsecondsPerOperation: 1_000 / (timed.operations + 1),
        },
        ...baseline.timed.slice(1),
      ],
    };
    const inconsistentFixedConfiguration = {
      ...benchmarkReport(),
      packed: {
        ...baseline.packed,
        configuration: {
          ...baseline.packed.configuration,
          maxNodes: baseline.packed.configuration.maxNodes + 1,
        },
      },
    };
    const duplicate = {
      ...benchmarkReport(),
      timed: [...baseline.timed, { ...timed }],
    };
    const missingPositionLoad = {
      ...baseline.packed.configuration,
    } as Record<string, unknown>;
    Reflect.deleteProperty(missingPositionLoad, "includesPositionLoad");

    expect(() =>
      aggregateBenchmarkReports([baseline, inconsistentOperations], 2, true),
    ).toThrow(/inconsistent operation count/u);
    expect(() =>
      aggregateBenchmarkReports(
        [baseline, inconsistentFixedConfiguration],
        2,
        true,
      ),
    ).toThrow(/fixed-work benchmark metric/u);
    expect(() =>
      aggregateBenchmarkReports([baseline, duplicate], 2, true),
    ).toThrow(/duplicate timed benchmark metrics/u);
    expect(() =>
      aggregateBenchmarkReports(
        [
          {
            ...baseline,
            packed: {
              ...baseline.packed,
              configuration: missingPositionLoad,
            },
          } as unknown as BenchmarkReport,
        ],
        1,
        true,
      ),
    ).toThrow(/fixed-work benchmark metric/u);
  });

  it("requires positive report samples and exact aggregate sample totals", () => {
    const report = benchmarkReport();
    for (const samples of [0, 1.5, Number.MAX_SAFE_INTEGER + 1]) {
      expect(() =>
        aggregateBenchmarkReports(
          [{ ...report, options: { ...report.options, samples } }],
          1,
          true,
        ),
      ).toThrow(/positive safe integer/u);
    }

    expect(() =>
      aggregateBenchmarkReports([benchmarkReport()], 2, true),
    ).toThrow(/expected 2 samples but reports contain 1/u);
    expect(() =>
      aggregateBenchmarkReports(
        [benchmarkReport(), benchmarkReport()],
        1,
        true,
      ),
    ).toThrow(/expected 1 samples but reports contain 2/u);
    expect(() =>
      aggregateBenchmarkReports([benchmarkReport()], 1, false),
    ).toThrow(/smoke mode does not match/u);
  });

  it("rejects missing, extra, and invalid duration samples", () => {
    const report = benchmarkReport();
    const timed = firstTimedMetric(report);
    const deadline = deadlineMetric(report);
    const malformedReports: readonly BenchmarkReport[] = [
      {
        ...report,
        timed: [{ ...timed, samples: [] }, ...report.timed.slice(1)],
      },
      {
        ...report,
        timed: [{ ...timed, samples: [1, 2] }, ...report.timed.slice(1)],
      },
      { ...report, packed: { ...report.packed, samples: [] } },
      { ...report, packed: { ...report.packed, samples: [Number.NaN] } },
      {
        ...report,
        packedDeadline: { ...deadline, samples: [] },
      },
      {
        ...report,
        packedDeadline: { ...deadline, samples: [20, 20] },
      },
      {
        ...report,
        packedDeadline: { ...deadline, samples: [-1] },
      },
    ];

    for (const malformedReport of malformedReports) {
      expect(() =>
        aggregateBenchmarkReports([malformedReport], 1, true),
      ).toThrow(/duration sample/u);
    }
  });

  it("accepts exact JSON summaries and rejects inconsistent derived values", () => {
    const report = benchmarkReport();
    const timed = firstTimedMetric(report);
    const deadline = deadlineMetric(report);
    const jsonReport = JSON.parse(JSON.stringify(report)) as BenchmarkReport;
    const malformedReports: readonly {
      report: BenchmarkReport;
      error: RegExp;
    }[] = [
      {
        report: {
          ...report,
          timed: [
            { ...timed, median: timed.median + Number.EPSILON },
            ...report.timed.slice(1),
          ],
        },
        error: /median does not match/u,
      },
      {
        report: {
          ...report,
          timed: [{ ...timed, p95: Number.NaN }, ...report.timed.slice(1)],
        },
        error: /p95 does not match/u,
      },
      {
        report: {
          ...report,
          timed: [
            {
              ...timed,
              microsecondsPerOperation:
                timed.microsecondsPerOperation + Number.EPSILON,
            },
            ...report.timed.slice(1),
          ],
        },
        error: /microseconds per operation does not match/u,
      },
      {
        report: {
          ...report,
          packed: { ...report.packed, p95: report.packed.p95 + 1 },
        },
        error: /fixed-work benchmark metric p95 does not match/u,
      },
      {
        report: {
          ...report,
          packed: {
            ...report.packed,
            nodesPerMillisecond: Number.POSITIVE_INFINITY,
          },
        },
        error: /fixed-work benchmark metric nodes per millisecond/u,
      },
      {
        report: {
          ...report,
          packedDeadline: { ...deadline, median: deadline.median + 1 },
        },
        error: /deadline benchmark metric median does not match/u,
      },
      {
        report: {
          ...report,
          packedDeadline: {
            ...deadline,
            nodesPerMillisecond: deadline.nodesPerMillisecond + 1,
          },
        },
        error: /deadline benchmark metric nodes per millisecond/u,
      },
    ];

    expect(() => validateBenchmarkReport(jsonReport)).not.toThrow();
    for (const malformed of malformedReports) {
      expect(() => validateBenchmarkReport(malformed.report)).toThrow(
        malformed.error,
      );
    }
  });

  it("rejects packed metrics with zero total duration", () => {
    const report = benchmarkReport();
    const deadline = deadlineMetric(report);
    const zeroFixedDuration: BenchmarkReport = {
      ...report,
      packed: {
        ...report.packed,
        samples: [0],
        median: 0,
        p95: 0,
        nodesPerMillisecond: 0,
      },
    };
    const zeroDeadlineDuration: BenchmarkReport = {
      ...report,
      packedDeadline: {
        ...deadline,
        samples: [0],
        median: 0,
        p95: 0,
        nodesPerMillisecond: 0,
      },
    };

    expect(() => validateBenchmarkReport(zeroFixedDuration)).toThrow(
      /fixed-work benchmark metric must have a positive total duration/u,
    );
    expect(() => validateBenchmarkReport(zeroDeadlineDuration)).toThrow(
      /deadline benchmark metric must have a positive total duration/u,
    );
  });

  it("rejects missing checksums during aggregation", () => {
    const missingFull = withoutTimedField(benchmarkReport(), "checksum");
    const missingSink = withoutTimedField(
      benchmarkReport(),
      "timedSinkChecksum",
    );

    const missingPackedDepth = benchmarkReport();
    const packed = { ...missingPackedDepth.packed } as Record<string, unknown>;
    Reflect.deleteProperty(packed, "depthChecksum");
    const missingDeadlineMove = benchmarkReport();
    const deadline = { ...deadlineMetric(missingDeadlineMove) } as Record<
      string,
      unknown
    >;
    Reflect.deleteProperty(deadline, "moveChecksum");

    expect(() => aggregateBenchmarkReports([missingFull], 1, true)).toThrow(
      /checksum/u,
    );
    expect(() => aggregateBenchmarkReports([missingSink], 1, true)).toThrow(
      /timed checksum/u,
    );
    expect(() =>
      aggregateBenchmarkReports(
        [{ ...missingPackedDepth, packed } as unknown as BenchmarkReport],
        1,
        true,
      ),
    ).toThrow(/fixed-work depth checksum/u);
    expect(() =>
      aggregateBenchmarkReports(
        [
          {
            ...missingDeadlineMove,
            packedDeadline: deadline,
          } as unknown as BenchmarkReport,
        ],
        1,
        true,
      ),
    ).toThrow(/deadline move checksum/u);
  });

  it("rejects missing or corrupt deadline state checksums", () => {
    const missingReport = benchmarkReport();
    const missingDeadline = { ...deadlineMetric(missingReport) } as Record<
      string,
      unknown
    >;
    Reflect.deleteProperty(missingDeadline, "stateChecksum");
    const corruptReport = benchmarkReport();
    const corruptDeadline = deadlineMetric(corruptReport);
    const malformedReports = [
      {
        ...missingReport,
        packedDeadline: missingDeadline,
      } as unknown as BenchmarkReport,
      {
        ...corruptReport,
        packedDeadline: {
          ...corruptDeadline,
          stateChecksum:
            (requiredNumber(corruptDeadline.stateChecksum) + 1) >>> 0,
        },
      },
    ];

    for (const malformedReport of malformedReports) {
      expect(() =>
        aggregateBenchmarkReports([malformedReport], 1, true),
      ).toThrow(/deadline benchmark state checksum/u);
    }
  });

  it("accepts exact deadline batches while allowing deadline progress to differ", () => {
    const decisions = {
      fast: { inputFen: "" },
      normal: { inputFen: "" },
      pro: { inputFen: "" },
    } as const;
    const states: readonly BenchmarkState[] = [
      { id: "state-a", fen: "unused", decisions },
      { id: "state-b", fen: "unused", decisions },
    ];
    const runner =
      (progress: number): BenchmarkRunner =>
      (benchmarkStates, options) => {
        const indices = options.deadlineStateIndices ?? [0];
        const stateIds = indices.flatMap((index) =>
          benchmarkStates[index] === undefined
            ? []
            : [benchmarkStates[index].id],
        );
        const report = invocationReport(benchmarkStates, options, stateIds);
        const deadline = deadlineMetric(report);
        return {
          ...report,
          packedDeadline: {
            ...deadline,
            nodes: deadline.nodes + progress,
            nodesPerMillisecond: (deadline.nodes + progress) / 20,
            depthChecksum: requiredNumber(deadline.depthChecksum) + progress,
            moveChecksum: requiredNumber(deadline.moveChecksum) + progress,
          },
        };
      };

    const report = runInterleavedBenchmark(states, runner(0), runner(1), {
      batches: 2,
      samples: 2,
      smoke: true,
    });

    expect(report.options.deadlineStateSchedule).toEqual([
      ["state-a"],
      ["state-b"],
    ]);
    expect(report.metricParity.complete).toBe(true);
    expect(report.implementations.baseline.packedDeadline?.nodes).not.toBe(
      report.implementations.candidate.packedDeadline?.nodes,
    );
    expect(
      report.implementations.baseline.packedDeadline?.depthChecksum,
    ).not.toBe(report.implementations.candidate.packedDeadline?.depthChecksum);
    expect(
      report.implementations.baseline.packedDeadline?.moveChecksum,
    ).not.toBe(report.implementations.candidate.packedDeadline?.moveChecksum);
    expect(report.metricParity.deadlineMoveChecksumMatch).toBe(false);
  });

  it("exports strict validation with an explicit legacy-only exception", () => {
    const decisions = {
      fast: { inputFen: "" },
      normal: { inputFen: "" },
      pro: { inputFen: "" },
    } as const;
    const states: readonly BenchmarkState[] = [
      { id: "state-a", fen: "unused", decisions },
    ];
    const options = {
      samples: 1,
      smoke: true,
      sampleOffset: 0,
      deadlineStateIndices: [0],
    } as const;
    const expected = {
      options,
      stateCount: 1,
      deadlineStateIds: ["state-a"],
    } as const;
    const strict = invocationReport(states, options, ["state-a"]);
    const legacy = legacyInvocationReport(states, options, ["state-a"]);
    const malformedV2 = withoutTimedField(strict, "timedSinkChecksum");

    expect(() => validateBenchmarkReport(strict, expected)).not.toThrow();
    expect(() => validateBenchmarkReport(legacy, expected)).toThrow(
      /unsupported benchmark contract version/u,
    );
    expect(() => aggregateBenchmarkReports([legacy], 1, true)).toThrow(
      /unsupported benchmark contract version/u,
    );
    expect(() =>
      validateBenchmarkReport(legacy, { ...expected, allowLegacy: true }),
    ).not.toThrow();
    expect(() =>
      validateBenchmarkReport(malformedV2, {
        ...expected,
        allowLegacy: true,
      }),
    ).toThrow(/timed checksum/u);
  });

  it("accepts an opted-in legacy baseline without weakening candidate validation", () => {
    const decisions = {
      fast: { inputFen: "" },
      normal: { inputFen: "" },
      pro: { inputFen: "" },
    } as const;
    const states: readonly BenchmarkState[] = [
      { id: "state-a", fen: "unused", decisions },
    ];
    const runner =
      (legacy: boolean): BenchmarkRunner =>
      (benchmarkStates, options) => {
        const indices = options.deadlineStateIndices ?? [0];
        const stateIds = indices.flatMap((index) => {
          const state = benchmarkStates[index];
          return state === undefined ? [] : [state.id];
        });
        return legacy
          ? legacyInvocationReport(benchmarkStates, options, stateIds)
          : invocationReport(benchmarkStates, options, stateIds);
      };
    const legacyRunner = runner(true);
    const strictRunner = runner(false);
    const options = { batches: 1, samples: 1, smoke: true } as const;

    expect(() =>
      runInterleavedBenchmark(states, legacyRunner, strictRunner, options),
    ).toThrow(/unsupported benchmark contract version/u);

    const report = runInterleavedBenchmark(states, legacyRunner, strictRunner, {
      ...options,
      allowLegacyBaseline: true,
    });

    expect(report.metricParity.complete).toBe(false);
    expect(report.metricParity.contractVersionMatch).toBe(false);
    expect(report.metricParity.fixedWorkConfigurationMatch).toBe(false);
    expect(report.metricParity.deadlineConfigurationMatch).toBe(false);
    expect(report.metricParity.timedSinkChecksumMismatches).not.toHaveLength(0);
    expect(report.options).not.toHaveProperty("allowLegacyBaseline");
    expect(report.implementations.baseline.contractVersion).toBeUndefined();
    expect(
      report.implementations.baseline.packed.configuration,
    ).toBeUndefined();
    expect(
      report.implementations.baseline.packedDeadline?.configuration,
    ).toBeUndefined();

    expect(() =>
      runInterleavedBenchmark(states, strictRunner, legacyRunner, {
        ...options,
        allowLegacyBaseline: true,
      }),
    ).toThrow(/unsupported benchmark contract version/u);
  });

  it("rejects malformed opted-in legacy baseline work", () => {
    const decisions = {
      fast: { inputFen: "" },
      normal: { inputFen: "" },
      pro: { inputFen: "" },
    } as const;
    const states: readonly BenchmarkState[] = [
      { id: "state-a", fen: "unused", decisions },
    ];
    const strictRunner: BenchmarkRunner = (benchmarkStates, options) =>
      invocationReport(benchmarkStates, options, ["state-a"]);
    const malformedCardinality: BenchmarkRunner = (
      benchmarkStates,
      options,
    ) => {
      const report = legacyInvocationReport(benchmarkStates, options, [
        "state-a",
      ]);
      const metric = firstTimedMetric(report);
      return {
        ...report,
        timed: [{ ...metric, samples: [] }, ...report.timed.slice(1)],
      };
    };
    const malformedSchedule: BenchmarkRunner = (benchmarkStates, options) =>
      legacyInvocationReport(benchmarkStates, options, ["state-b"]);
    const options = {
      allowLegacyBaseline: true,
      batches: 1,
      samples: 1,
      smoke: true,
    } as const;

    expect(() =>
      runInterleavedBenchmark(
        states,
        malformedCardinality,
        strictRunner,
        options,
      ),
    ).toThrow(/duration sample/u);
    expect(() =>
      runInterleavedBenchmark(states, malformedSchedule, strictRunner, options),
    ).toThrow(/inconsistent deadline state IDs/u);
  });

  it("rejects interleaved deadline regrouping before aggregation", () => {
    const decisions = {
      fast: { inputFen: "" },
      normal: { inputFen: "" },
      pro: { inputFen: "" },
    } as const;
    const states: readonly BenchmarkState[] = [
      { id: "state-a", fen: "unused", decisions },
      { id: "state-b", fen: "unused", decisions },
    ];
    const runner: BenchmarkRunner = (benchmarkStates, options) => {
      const deadlineStateIds =
        options.sampleOffset === undefined
          ? ["state-a"]
          : options.sampleOffset === 0
            ? ["state-a", "state-b"]
            : [];
      return invocationReport(benchmarkStates, options, deadlineStateIds);
    };

    expect(() =>
      runInterleavedBenchmark(states, runner, runner, {
        batches: 1,
        samples: 2,
        smoke: true,
      }),
    ).toThrow(/inconsistent deadline state IDs/u);
  });

  it("rejects options returned for a different invocation", () => {
    const decisions = {
      fast: { inputFen: "" },
      normal: { inputFen: "" },
      pro: { inputFen: "" },
    } as const;
    const states: readonly BenchmarkState[] = [
      { id: "state-a", fen: "unused", decisions },
    ];
    const runner: BenchmarkRunner = (benchmarkStates, options) => {
      const returnedOptions =
        options.sampleOffset === undefined
          ? options
          : { ...options, sampleOffset: options.sampleOffset + 1 };
      return invocationReport(benchmarkStates, returnedOptions, ["state-a"]);
    };

    expect(() =>
      runInterleavedBenchmark(states, runner, runner, {
        batches: 1,
        samples: 1,
        smoke: true,
      }),
    ).toThrow(/inconsistent options/u);
  });
});
