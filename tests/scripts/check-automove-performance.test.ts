import { spawnSync, type SpawnSyncReturns } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import { afterEach, describe, expect, it } from "vitest";

import { writePerformanceStateBankFixture } from "./performance-state-bank.fixture.js";

const repositoryRoot = path.resolve(import.meta.dirname, "../..");
const script = path.join(repositoryRoot, "scripts/check-automove-performance.mjs");
const temporaryDirectories: string[] = [];

type InvalidCounts = {
  total: number;
  noSuggestion: number;
  sourceMutation: number;
  selectorError: number;
  illegalReplay: number;
};

type TimingSummary = {
  count: number;
  totalMs: number;
  meanMs: number | null;
  medianMs: number | null;
  p95Ms: number | null;
  maxMs: number | null;
};

type TimingRatios = {
  mean: number | null;
  median: number | null;
  p95: number | null;
  max: number | null;
};

type PerformanceSide = TimingSummary & {
  samplesMs: number[];
  invalids: InvalidCounts;
};

type PerformanceRow = {
  mode: string;
  id: string;
  fen: string;
  callsPerBundle: number;
  baseline: PerformanceSide;
  candidate: PerformanceSide;
  candidateBaselineRatio: TimingRatios | null;
};

type PerformanceSummary = {
  states: number;
  rows: number;
  callsPerBundle: number;
  baseline: TimingSummary & { invalids: InvalidCounts };
  candidate: TimingSummary & { invalids: InvalidCounts };
  candidateBaselineRatio: TimingRatios | null;
};

type PerformanceReport = {
  schemaVersion: number;
  kind: string;
  baseline: string;
  baselineSha256: string;
  candidate: string;
  candidateSha256: string;
  stateBank: string;
  stateBankSha256: string;
  stateBankManifest?: string;
  stateBankManifestSha256?: string;
  config: {
    modes: string[];
    repeat: number;
    order: string;
    samplesPerBundlePerState: number;
  };
  states: number;
  summary: PerformanceSummary;
  modes: (PerformanceSummary & { mode: string })[];
  rows: PerformanceRow[];
};

type ModeFixture = {
  mode: string;
  mean: number | null;
  p95: number | null;
  baselineInvalids?: number;
  candidateInvalids?: number;
};

type GeneratedManifestFixture = {
  algorithm: { sha256: string };
  source: {
    sha256: string;
    corpus: { sha256: string };
    bundle: { sha256: string };
  };
  selection: { colors: { white: number }; variants: string[] };
  output: { encoding: string };
};

afterEach(() => {
  for (const directory of temporaryDirectories.splice(0)) {
    fs.rmSync(directory, { force: true, recursive: true });
  }
});

describe("check-automove-performance CLI", () => {
  it("accepts reports at the default and supplied ratio limits", () => {
    const root = temporaryDirectory();
    const defaultReport = writeReport(
      root,
      "default.json",
      performanceReport({ mean: 1.05, p95: 1.1 }),
    );
    const defaultResult = run(["--report", defaultReport]);
    expect(defaultResult.status, defaultResult.stderr).toBe(0);
    expect(defaultResult.stdout).toContain(
      "fast mean 1.05, p95 1.1; limits mean <= 1.05, p95 <= 1.1",
    );

    const customReport = writeReport(
      root,
      "custom.json",
      performanceReport({ mean: 1.06, p95: 1.11 }),
    );
    const customResult = run([
      "--report",
      customReport,
      "--max-mean-ratio",
      "1.06",
      "--max-p95-ratio=1.11",
    ]);
    expect(customResult.status, customResult.stderr).toBe(0);
  });

  it("rejects mean or p95 regressions above the configured limits", () => {
    const root = temporaryDirectory();
    const meanReport = writeReport(
      root,
      "mean.json",
      performanceReport({ mean: 1.050_001, p95: 1.1 }),
    );
    const meanResult = run(["--report", meanReport]);
    expect(meanResult.status).toBe(1);
    expect(meanResult.stderr).toContain("fast mean ratio 1.050001 exceeds 1.05");

    const p95Report = writeReport(
      root,
      "p95.json",
      performanceReport({ mean: 1.05, p95: 1.100_001 }),
    );
    const p95Result = run(["--report", p95Report]);
    expect(p95Result.status).toBe(1);
    expect(p95Result.stderr).toContain("fast p95 ratio 1.100001 exceeds 1.1");
  });

  it("rejects invalid calls and malformed call counts", () => {
    const root = temporaryDirectory();
    const invalidReport = writeReport(
      root,
      "invalid.json",
      performanceReport({
        mean: null,
        p95: null,
        baselineInvalids: 1,
        candidateInvalids: 2,
      }),
    );
    const invalidResult = run(["--report", invalidReport]);
    expect(invalidResult.status).toBe(1);
    expect(invalidResult.stderr).toContain("fast contained 192 invalid calls");

    const malformed = performanceReport({ mean: 1, p95: 1 });
    malformed.summary.candidate.invalids.total = 1;
    const malformedReport = writeReport(root, "malformed.json", malformed);
    const malformedResult = run(["--report", malformedReport]);
    expect(malformedResult.status).toBe(1);
    expect(malformedResult.stderr).toContain(
      "inconsistent summary candidate invalid call counts",
    );
  });

  it("checks every mode instead of allowing aggregate timing to hide a regression", () => {
    const root = temporaryDirectory();
    const report = performanceReportForModes([
      { mode: "fast", mean: 1, p95: 1 },
      { mode: "pro", mean: 1.06, p95: 1.11 },
    ]);
    const reportPath = writeReport(root, "per-mode.json", report);

    const result = run(["--report", reportPath]);

    expect(result.status).toBe(1);
    expect(result.stderr).toContain("pro mean ratio 1.06 exceeds 1.05");
    expect(result.stderr).toContain("pro p95 ratio 1.11 exceeds 1.1");
  });

  it("does not gate the non-compositional pooled p95", () => {
    const report = performanceReportForModes([
      { mode: "fast", mean: 1, p95: 1 },
      { mode: "normal", mean: 1, p95: 1 },
      { mode: "pro", mean: 1, p95: 1 },
    ]);
    const proRows = report.rows.filter((row) => row.mode === "pro");
    const baselineSamples = [
      ...Array.from({ length: 116 }, () => 10),
      ...Array.from({ length: 12 }, () => 100),
    ];
    const candidateSamples = [
      ...Array.from({ length: 108 }, () => 0),
      ...Array.from({ length: 20 }, () => 110),
    ];
    expect(proRows.length * 2).toBe(baselineSamples.length);
    for (const [index, row] of proRows.entries()) {
      row.baseline = performanceSide(
        baselineSamples.slice(index * 2, index * 2 + 2),
        0,
      );
      row.candidate = performanceSide(
        candidateSamples.slice(index * 2, index * 2 + 2),
        0,
      );
      row.candidateBaselineRatio = timingRatios(row.candidate, row.baseline);
    }
    rebuildReportSummaries(report);

    expect(
      report.modes.map(({ candidateBaselineRatio }) => candidateBaselineRatio?.p95),
    ).toEqual([1, 1, 1.1]);
    expect(report.summary.candidateBaselineRatio?.p95).toBe(11);
    const root = temporaryDirectory();
    const result = run(["--report", writeReport(root, "pooled-p95.json", report)]);
    expect(result.status, result.stderr).toBe(0);
  });

  it("rejects missing or duplicate configured mode coverage", () => {
    const root = temporaryDirectory();
    const missing = performanceReportForModes([
      { mode: "fast", mean: 1, p95: 1 },
      { mode: "pro", mean: 1, p95: 1 },
    ]);
    missing.modes.pop();
    const missingResult = run([
      "--report",
      writeReport(root, "missing-mode.json", missing),
    ]);
    expect(missingResult.status).toBe(1);
    expect(missingResult.stderr).toContain(
      "configured automove modes do not match mode summaries",
    );

    const duplicateConfiguration = performanceReport({ mean: 1, p95: 1 });
    duplicateConfiguration.config.modes.push("fast");
    const duplicateConfigurationResult = run([
      "--report",
      writeReport(root, "duplicate-config.json", duplicateConfiguration),
    ]);
    expect(duplicateConfigurationResult.status).toBe(1);
    expect(duplicateConfigurationResult.stderr).toContain(
      "duplicate automove configured modes",
    );

    const duplicateSummary = performanceReport({ mean: 1, p95: 1 });
    const [firstSummary] = duplicateSummary.modes;
    if (firstSummary === undefined) throw new Error("missing fixture mode summary");
    duplicateSummary.modes.push(structuredClone(firstSummary));
    const duplicateSummaryResult = run([
      "--report",
      writeReport(root, "duplicate-summary.json", duplicateSummary),
    ]);
    expect(duplicateSummaryResult.status).toBe(1);
    expect(duplicateSummaryResult.stderr).toContain(
      "duplicate automove mode summaries",
    );
  });

  it("requires every mode to cover the same state IDs and FENs", () => {
    const root = temporaryDirectory();
    const missingState = performanceReportForModes([
      { mode: "fast", mean: 1, p95: 1 },
      { mode: "pro", mean: 1, p95: 1 },
    ]);
    const missingStateId = missingState.rows.find((row) => row.mode === "pro")?.id;
    if (missingStateId === undefined) throw new Error("missing pro fixture row");
    missingState.rows = missingState.rows.filter(
      (row) => row.mode !== "pro" || row.id !== missingStateId,
    );
    rebuildReportSummaries(missingState);
    const missingStateResult = run([
      "--report",
      writeReport(root, "missing-state.json", missingState),
    ]);
    expect(missingStateResult.status).toBe(1);
    expect(missingStateResult.stderr).toContain(
      "mode pro has inconsistent state inventory",
    );

    const mismatchedFen = performanceReportForModes([
      { mode: "fast", mean: 1, p95: 1 },
      { mode: "pro", mean: 1, p95: 1 },
    ]);
    const proRow = mismatchedFen.rows.find((row) => row.mode === "pro");
    if (proRow === undefined) throw new Error("missing pro fixture row");
    proRow.fen = "different-fixture-fen";
    const mismatchedFenResult = run([
      "--report",
      writeReport(root, "mismatched-fen.json", mismatchedFen),
    ]);
    expect(mismatchedFenResult.status).toBe(1);
    expect(mismatchedFenResult.stderr).toContain(
      "mode pro has inconsistent state inventory",
    );
  });

  it("rejects noncanonical modes and truncated sampling protocols", () => {
    const root = temporaryDirectory();
    const fakeMode = performanceReport({ mean: 1, p95: 1 });
    const [fakeModeSummary] = fakeMode.modes;
    const [fakeModeRow] = fakeMode.rows;
    if (fakeModeSummary === undefined || fakeModeRow === undefined) {
      throw new Error("missing fake-mode fixture data");
    }
    fakeMode.config.modes[0] = "fake";
    fakeModeSummary.mode = "fake";
    fakeModeRow.mode = "fake";
    const fakeModeResult = run([
      "--report",
      writeReport(root, "fake-mode.json", fakeMode),
    ]);
    expect(fakeModeResult.status).toBe(1);
    expect(fakeModeResult.stderr).toContain("unsupported configured automove mode");

    const wrongOrder = performanceReport({ mean: 1, p95: 1 });
    wrongOrder.config.order = "BAAB";
    const wrongOrderResult = run([
      "--report",
      writeReport(root, "wrong-order.json", wrongOrder),
    ]);
    expect(wrongOrderResult.status).toBe(1);
    expect(wrongOrderResult.stderr).toContain(
      "invalid automove performance sampling configuration",
    );

    const truncatedRow = performanceReport({ mean: 1, p95: 1 });
    const [truncatedPerformanceRow] = truncatedRow.rows;
    if (truncatedPerformanceRow === undefined) {
      throw new Error("missing truncated-row fixture data");
    }
    truncatedPerformanceRow.callsPerBundle = 1;
    const truncatedRowResult = run([
      "--report",
      writeReport(root, "truncated-row.json", truncatedRow),
    ]);
    expect(truncatedRowResult.status).toBe(1);
    expect(truncatedRowResult.stderr).toContain(
      `inconsistent row fast/${truncatedPerformanceRow.id} sampling contract`,
    );
  });

  it("requires complete canonical provenance", () => {
    const root = temporaryDirectory();
    const missingPath = performanceReport({ mean: 1, p95: 1 });
    missingPath.baseline = "";
    const missingPathResult = run([
      "--report",
      writeReport(root, "missing-path.json", missingPath),
    ]);
    expect(missingPathResult.status).toBe(1);
    expect(missingPathResult.stderr).toContain(
      "invalid automove performance baseline path",
    );

    const missingHash = performanceReport({ mean: 1, p95: 1 });
    missingHash.stateBankSha256 = "missing";
    const missingHashResult = run([
      "--report",
      writeReport(root, "missing-hash.json", missingHash),
    ]);
    expect(missingHashResult.status).toBe(1);
    expect(missingHashResult.stderr).toContain(
      "invalid automove performance stateBank SHA-256",
    );
  });

  it("binds provenance hashes and rows to the referenced artifacts", () => {
    const root = temporaryDirectory();
    const missingArtifacts = performanceReport({ mean: 1, p95: 1 });
    missingArtifacts.baseline = path.join(root, "missing-baseline.mjs");
    missingArtifacts.baselineSha256 = "a".repeat(64);
    const missingResult = run([
      "--report",
      writeReport(root, "missing-artifacts.json", missingArtifacts),
    ]);
    expect(missingResult.status).toBe(1);
    expect(missingResult.stderr).toContain(
      "could not read automove performance baseline artifact",
    );

    const staleHash = performanceReport({ mean: 1, p95: 1 });
    staleHash.candidateSha256 = "0".repeat(64);
    const staleResult = run([
      "--report",
      writeReport(root, "stale-hash.json", staleHash),
    ]);
    expect(staleResult.status).toBe(1);
    expect(staleResult.stderr).toContain("candidate SHA-256 does not match artifact");

    const fabricatedRows = performanceReportForModes([
      { mode: "fast", mean: 1, p95: 1 },
      { mode: "normal", mean: 1, p95: 1 },
    ]);
    for (const row of fabricatedRows.rows) row.fen = `fabricated-${row.id}`;
    const fabricatedResult = run([
      "--report",
      writeReport(root, "fabricated-rows.json", fabricatedRows),
    ]);
    expect(fabricatedResult.status).toBe(1);
    expect(fabricatedResult.stderr).toContain("rows do not match state-bank inventory");

    const missingManifest = performanceReport({ mean: 1, p95: 1 });
    delete missingManifest.stateBankManifest;
    delete missingManifest.stateBankManifestSha256;
    const missingManifestResult = run([
      "--report",
      writeReport(root, "missing-manifest.json", missingManifest),
    ]);
    expect(missingManifestResult.status).toBe(1);
    expect(missingManifestResult.stderr).toContain("--state-manifest is required");

    const protectedBank = performanceReport({ mean: 1, p95: 1 });
    protectedBank.stateBank = path.join(
      repositoryRoot,
      "test-data/automove-decisions/v6/decisions.jsonl",
    );
    protectedBank.stateBankSha256 = fileSha256(protectedBank.stateBank);
    delete protectedBank.stateBankManifest;
    delete protectedBank.stateBankManifestSha256;
    const protectedBankResult = run([
      "--report",
      writeReport(root, "protected-bank.json", protectedBank),
    ]);
    expect(protectedBankResult.status).toBe(1);
    expect(protectedBankResult.stderr).toContain(
      "rows do not match state-bank inventory",
    );
    expect(protectedBankResult.stderr).not.toContain("state-manifest");
  });

  it("requires regular self-contained bundle artifacts", () => {
    const root = temporaryDirectory();
    const deviceReport = performanceReport({ mean: 1, p95: 1 });
    deviceReport.baseline = os.devNull;
    deviceReport.baselineSha256 = createHash("sha256").digest("hex");
    const deviceResult = run([
      "--report",
      writeReport(root, "device-artifact.json", deviceReport),
    ]);
    expect(deviceResult.status).toBe(1);
    expect(deviceResult.stderr).toContain("not a regular file");

    const importedReport = performanceReport({ mean: 1, p95: 1 });
    const implementation = path.join(root, "implementation.mjs");
    const wrapper = path.join(root, "wrapper.mjs");
    fs.writeFileSync(
      implementation,
      "export class Game {}\nexport const GameVariant = { Classic: 'Classic' };\n",
    );
    fs.writeFileSync(
      wrapper,
      'export { Game, GameVariant } from "./implementation.mjs";\n',
    );
    importedReport.baseline = wrapper;
    importedReport.baselineSha256 = fileSha256(wrapper);
    const importedResult = run([
      "--report",
      writeReport(root, "imported-bundle.json", importedReport),
    ]);
    expect(importedResult.status).toBe(1);
    expect(importedResult.stderr).toContain("not self-contained: ./implementation.mjs");

    const builtinReport = performanceReport({ mean: 1, p95: 1 });
    const builtin = path.join(root, "builtin-import.mjs");
    fs.writeFileSync(
      builtin,
      'import "node:fs";\nexport class Game {}\nexport const GameVariant = { Classic: "Classic" };\n',
    );
    builtinReport.baseline = builtin;
    builtinReport.baselineSha256 = fileSha256(builtin);
    const builtinResult = run([
      "--report",
      writeReport(root, "builtin-bundle.json", builtinReport),
    ]);
    expect(builtinResult.status).toBe(1);
    expect(builtinResult.stderr).toContain("not self-contained: node:fs");

    for (const [name, source, reason] of [
      [
        "indirect-eval",
        `const dependency = await (0, eval)("import('./implementation.mjs')");
export const Game = dependency.Game;
export const GameVariant = dependency.GameVariant;
`,
        "dynamic code eval",
      ],
      [
        "function-constructor",
        `const load = globalThis["Function"]("return import('./implementation.mjs')");
const dependency = await load();
export const Game = dependency.Game;
export const GameVariant = dependency.GameVariant;
`,
        "dynamic code Function",
      ],
      [
        "constructor-property",
        `const FunctionAlias = (() => {}).constructor;
const load = FunctionAlias("return import('./implementation.mjs')");
const dependency = await load();
export const Game = dependency.Game;
export const GameVariant = dependency.GameVariant;
`,
        "dynamic code constructor",
      ],
    ] as const) {
      const dynamicReport = performanceReport({ mean: 1, p95: 1 });
      const dynamicBundle = path.join(root, `${name}.mjs`);
      fs.writeFileSync(dynamicBundle, source);
      dynamicReport.baseline = dynamicBundle;
      dynamicReport.baselineSha256 = fileSha256(dynamicBundle);
      const dynamicResult = run([
        "--report",
        writeReport(root, `${name}.json`, dynamicReport),
      ]);
      expect(dynamicResult.status).toBe(1);
      expect(dynamicResult.stderr).toContain(`not self-contained: ${reason}`);
    }
  });

  it("rejects corrupted generated-manifest provenance", () => {
    const root = temporaryDirectory();
    const corruptions: [string, (manifest: GeneratedManifestFixture) => void][] = [
      ["algorithm", (manifest) => (manifest.algorithm.sha256 = "0".repeat(64))],
      ["source", (manifest) => (manifest.source.sha256 = "0".repeat(64))],
      ["colors", (manifest) => (manifest.selection.colors.white = 31)],
      ["variants", (manifest) => (manifest.selection.variants = ["Invented"])],
      ["encoding", (manifest) => (manifest.output.encoding = "UTF-16")],
    ];
    for (const [name, corrupt] of corruptions) {
      const report = performanceReport({ mean: 1, p95: 1 });
      const manifestPath = report.stateBankManifest;
      if (manifestPath === undefined) throw new Error("missing fixture manifest");
      const manifest = JSON.parse(
        fs.readFileSync(manifestPath, "utf8"),
      ) as GeneratedManifestFixture;
      corrupt(manifest);
      fs.writeFileSync(manifestPath, `${JSON.stringify(manifest)}\n`);
      report.stateBankManifestSha256 = fileSha256(manifestPath);
      const result = run(["--report", writeReport(root, `${name}.json`, report)]);
      expect(result.status).toBe(1);
      expect(result.stderr).toContain("manifest does not match state bank");
    }

    const report = performanceReport({ mean: 1, p95: 1 });
    const manifestPath = report.stateBankManifest;
    if (manifestPath === undefined) throw new Error("missing fixture manifest");
    const manifest = JSON.parse(
      fs.readFileSync(manifestPath, "utf8"),
    ) as GeneratedManifestFixture;
    manifest.source.bundle.sha256 = "b".repeat(64);
    manifest.source.sha256 = bytesSha256(
      JSON.stringify({
        bundleSha256: manifest.source.bundle.sha256,
        corpusSha256: manifest.source.corpus.sha256,
      }),
    );
    fs.writeFileSync(manifestPath, `${JSON.stringify(manifest)}\n`);
    report.stateBankManifestSha256 = fileSha256(manifestPath);
    const result = run([
      "--report",
      writeReport(root, "wrong-source-bundle.json", report),
    ]);
    expect(result.status).toBe(1);
    expect(result.stderr).toContain(
      "manifest source bundle does not match baseline artifact",
    );
  });

  it("enforces the producer repeat ceiling", () => {
    const report = performanceReport({ mean: 1, p95: 1 });
    report.config.repeat = 101;
    report.config.samplesPerBundlePerState = 202;
    for (const row of report.rows) {
      row.callsPerBundle = 202;
      row.baseline = performanceSide(
        Array.from({ length: 202 }, () => 1),
        0,
      );
      row.candidate = performanceSide(
        Array.from({ length: 202 }, () => 1),
        0,
      );
      row.candidateBaselineRatio = timingRatios(row.candidate, row.baseline);
    }
    rebuildReportSummaries(report);
    const root = temporaryDirectory();
    const result = run(["--report", writeReport(root, "repeat-101.json", report)]);
    expect(result.status).toBe(1);
    expect(result.stderr).toContain("performance repeat must be at most 100");
  });

  it("rejects summaries and timing samples that disagree with report rows", () => {
    const root = temporaryDirectory();
    const inconsistentSummary = performanceReport({ mean: 1, p95: 1 });
    inconsistentSummary.summary.rows += 1;
    const summaryResult = run([
      "--report",
      writeReport(root, "inconsistent-summary.json", inconsistentSummary),
    ]);
    expect(summaryResult.status).toBe(1);
    expect(summaryResult.stderr).toContain(
      "inconsistent automove performance summary row summary",
    );

    const inconsistentRow = performanceReport({ mean: 1, p95: 1 });
    const [row] = inconsistentRow.rows;
    if (row === undefined) throw new Error("missing fixture row");
    row.candidate.samplesMs[0] = 1.01;
    const rowResult = run([
      "--report",
      writeReport(root, "inconsistent-row.json", inconsistentRow),
    ]);
    expect(rowResult.status).toBe(1);
    expect(rowResult.stderr).toContain(
      `inconsistent row fast/${row.id} candidate timing summary`,
    );
  });

  it("fails closed for invalid reports and CLI limits", () => {
    const root = temporaryDirectory();
    const wrongKind = writeReport(root, "wrong-kind.json", {
      schemaVersion: 1,
      kind: "other-report",
    });
    const wrongKindResult = run(["--report", wrongKind]);
    expect(wrongKindResult.status).toBe(1);
    expect(wrongKindResult.stderr).toContain("invalid automove performance report");

    const valid = writeReport(
      root,
      "valid.json",
      performanceReport({ mean: 1, p95: 1 }),
    );
    const invalidLimitResult = run(["--report", valid, "--max-mean-ratio", "0"]);
    expect(invalidLimitResult.status).toBe(1);
    expect(invalidLimitResult.stderr).toContain(
      "--max-mean-ratio must be a positive finite number",
    );

    const missingReportResult = run([]);
    expect(missingReportResult.status).toBe(1);
    expect(missingReportResult.stderr).toContain("missing required option: --report");
  });
});

function temporaryDirectory(): string {
  const directory = fs.mkdtempSync(
    path.join(os.tmpdir(), "mons-automove-performance-check-"),
  );
  temporaryDirectories.push(directory);
  return directory;
}

function run(arguments_: string[]): SpawnSyncReturns<string> {
  return spawnSync(process.execPath, [script, ...arguments_], {
    cwd: repositoryRoot,
    encoding: "utf8",
  });
}

function writeReport(directory: string, name: string, report: unknown): string {
  const filePath = path.join(directory, name);
  fs.writeFileSync(filePath, `${JSON.stringify(report)}\n`);
  return filePath;
}

function performanceReport({
  mean,
  p95,
  baselineInvalids = 0,
  candidateInvalids = 0,
}: {
  mean: number | null;
  p95: number | null;
  baselineInvalids?: number;
  candidateInvalids?: number;
}): PerformanceReport {
  return performanceReportForModes([
    {
      mode: "fast",
      mean,
      p95,
      baselineInvalids,
      candidateInvalids,
    },
  ]);
}

function performanceReportForModes(
  fixtures: readonly ModeFixture[],
): PerformanceReport {
  const provenanceRoot = temporaryDirectory();
  const baseline = path.join(provenanceRoot, "baseline.mjs");
  const candidate = path.join(provenanceRoot, "candidate.mjs");
  fs.writeFileSync(
    baseline,
    "export class Game {}\nexport const GameVariant = { Classic: 'Classic' };\n",
  );
  fs.writeFileSync(
    candidate,
    "export class Game {}\nexport const GameVariant = { Classic: 'Classic' };\n",
  );
  const fixture = writePerformanceStateBankFixture(
    provenanceRoot,
    "states",
    fileSha256(baseline),
  );
  const rows = fixtures.flatMap((modeFixture) =>
    fixture.states.map(({ id, fen }) => performanceRow(modeFixture, id, fen)),
  );
  const summary = performanceSummary(rows);
  return {
    schemaVersion: 1,
    kind: "automove-performance",
    baseline,
    baselineSha256: fileSha256(baseline),
    candidate,
    candidateSha256: fileSha256(candidate),
    stateBank: fixture.stateBank,
    stateBankSha256: fileSha256(fixture.stateBank),
    stateBankManifest: fixture.manifest,
    stateBankManifestSha256: fileSha256(fixture.manifest),
    config: {
      modes: fixtures.map(({ mode }) => mode),
      repeat: 1,
      order: "ABBA",
      samplesPerBundlePerState: 2,
    },
    states: summary.states,
    summary,
    modes: fixtures.map(({ mode }) => ({
      mode,
      ...performanceSummary(rows.filter((row) => row.mode === mode)),
    })),
    rows,
  };
}

function fileSha256(filePath: string): string {
  return createHash("sha256").update(fs.readFileSync(filePath)).digest("hex");
}

function bytesSha256(value: NodeJS.ArrayBufferView | string): string {
  return createHash("sha256").update(value).digest("hex");
}

function performanceRow(
  { mode, mean, p95, baselineInvalids = 0, candidateInvalids = 0 }: ModeFixture,
  id = "state",
  fen = "fixture-fen",
): PerformanceRow {
  const callsPerBundle = Math.max(2, baselineInvalids, candidateInvalids);
  const baselineSamples = Array.from(
    { length: callsPerBundle - baselineInvalids },
    () => 1,
  );
  let candidateSamples: number[];
  if (candidateInvalids > 0 || baselineInvalids > 0) {
    candidateSamples = Array.from(
      { length: callsPerBundle - candidateInvalids },
      () => 1,
    );
  } else {
    if (mean === null || p95 === null) {
      throw new Error("valid fixture timings require mean and p95 ratios");
    }
    candidateSamples = [roundNumber(2 * mean - p95), p95];
  }
  const baseline = performanceSide(baselineSamples, baselineInvalids);
  const candidate = performanceSide(candidateSamples, candidateInvalids);
  return {
    mode,
    id,
    fen,
    callsPerBundle,
    baseline,
    candidate,
    candidateBaselineRatio:
      baselineInvalids + candidateInvalids === 0
        ? timingRatios(candidate, baseline)
        : null,
  };
}

function rebuildReportSummaries(report: PerformanceReport): void {
  report.summary = performanceSummary(report.rows);
  report.states = report.summary.states;
  report.modes = report.config.modes.map((mode) => ({
    mode,
    ...performanceSummary(report.rows.filter((row) => row.mode === mode)),
  }));
}

function performanceSide(samplesMs: number[], invalidTotal: number): PerformanceSide {
  return {
    ...timingSummary(samplesMs),
    samplesMs,
    invalids: invalidCounts(invalidTotal),
  };
}

function performanceSummary(rows: readonly PerformanceRow[]): PerformanceSummary {
  const baselineSamples = rows.flatMap(({ baseline }) => baseline.samplesMs);
  const candidateSamples = rows.flatMap(({ candidate }) => candidate.samplesMs);
  const baselineInvalids = sumInvalidCounts(
    rows.map(({ baseline }) => baseline.invalids),
  );
  const candidateInvalids = sumInvalidCounts(
    rows.map(({ candidate }) => candidate.invalids),
  );
  const baselineTiming = timingSummary(baselineSamples);
  const candidateTiming = timingSummary(candidateSamples);
  return {
    states: new Set(rows.map(({ id }) => id)).size,
    rows: rows.length,
    callsPerBundle: rows.reduce((sum, row) => sum + row.callsPerBundle, 0),
    baseline: { ...baselineTiming, invalids: baselineInvalids },
    candidate: { ...candidateTiming, invalids: candidateInvalids },
    candidateBaselineRatio:
      baselineInvalids.total + candidateInvalids.total === 0
        ? timingRatios(candidateTiming, baselineTiming)
        : null,
  };
}

function invalidCounts(total: number): InvalidCounts {
  return {
    total,
    noSuggestion: total,
    sourceMutation: 0,
    selectorError: 0,
    illegalReplay: 0,
  };
}

function sumInvalidCounts(items: readonly InvalidCounts[]): InvalidCounts {
  return items.reduce(
    (sum, item) => ({
      total: sum.total + item.total,
      noSuggestion: sum.noSuggestion + item.noSuggestion,
      sourceMutation: sum.sourceMutation + item.sourceMutation,
      selectorError: sum.selectorError + item.selectorError,
      illegalReplay: sum.illegalReplay + item.illegalReplay,
    }),
    invalidCounts(0),
  );
}

function timingSummary(values: readonly number[]): TimingSummary {
  const sorted = values.map(roundNumber).sort((left, right) => left - right);
  if (sorted.length === 0) {
    return {
      count: 0,
      totalMs: 0,
      meanMs: null,
      medianMs: null,
      p95Ms: null,
      maxMs: null,
    };
  }
  const total = sorted.reduce((sum, value) => sum + value, 0);
  const middle = Math.floor(sorted.length / 2);
  const lowerMiddle = sorted[middle - 1];
  const upperMiddle = sorted[middle];
  if (upperMiddle === undefined) throw new Error("missing fixture timing");
  const median =
    sorted.length % 2 === 0
      ? ((lowerMiddle ?? upperMiddle) + upperMiddle) / 2
      : upperMiddle;
  const p95 = sorted[Math.max(0, Math.ceil(sorted.length * 0.95) - 1)];
  const maximum = sorted[sorted.length - 1];
  if (p95 === undefined || maximum === undefined) {
    throw new Error("missing fixture timing percentile");
  }
  return {
    count: sorted.length,
    totalMs: roundNumber(total),
    meanMs: roundNumber(total / sorted.length),
    medianMs: roundNumber(median),
    p95Ms: p95,
    maxMs: maximum,
  };
}

function timingRatios(candidate: TimingSummary, baseline: TimingSummary): TimingRatios {
  return {
    mean: ratio(candidate.meanMs, baseline.meanMs),
    median: ratio(candidate.medianMs, baseline.medianMs),
    p95: ratio(candidate.p95Ms, baseline.p95Ms),
    max: ratio(candidate.maxMs, baseline.maxMs),
  };
}

function ratio(numerator: number | null, denominator: number | null): number | null {
  return numerator === null || denominator === null || denominator === 0
    ? null
    : roundNumber(numerator / denominator);
}

function roundNumber(value: number): number {
  return Math.round(value * 1_000_000) / 1_000_000;
}
