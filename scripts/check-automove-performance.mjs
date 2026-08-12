#!/usr/bin/env node

import { readFileSync } from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";

import {
  AUTOMOVE_MODES,
  parseCliOptions,
  readPerformanceStateBank,
  requiredOption,
  timingRatios,
  timingSummary,
} from "./automove-evidence-support.mjs";
import {
  MAX_PERFORMANCE_REPEAT,
  readPublicBundleArtifact,
} from "./evidence/options.mjs";

const defaultMaxMeanRatio = 1.05;
const defaultMaxP95Ratio = 1.1;
const sha256Pattern = /^[0-9a-f]{64}$/u;
const invalidReasonFields = Object.freeze([
  "noSuggestion",
  "sourceMutation",
  "selectorError",
  "illegalReplay",
]);
const helpText = `Usage: node scripts/check-automove-performance.mjs \\
  --report <performance.json> \\
  [--max-mean-ratio 1.05] [--max-p95-ratio 1.10]`;

export function checkAutomovePerformance(
  report,
  { maxMeanRatio = defaultMaxMeanRatio, maxP95Ratio = defaultMaxP95Ratio } = {},
) {
  assertPositiveFinite(maxMeanRatio, "maximum mean ratio");
  assertPositiveFinite(maxP95Ratio, "maximum p95 ratio");
  if (
    report === null ||
    typeof report !== "object" ||
    Array.isArray(report) ||
    report.schemaVersion !== 1 ||
    report.kind !== "automove-performance"
  ) {
    throw new Error("invalid automove performance report");
  }
  const provenance = readProvenance(report);
  const configuration = readConfiguration(report.config);
  const configuredModes = configuration.modes;
  const modeSummaries = report.modes;
  if (!Array.isArray(modeSummaries) || modeSummaries.length === 0) {
    throw new Error("automove performance report has no mode summaries");
  }
  const modeNames = modeSummaries.map((modeSummary, index) => {
    if (
      modeSummary === null ||
      typeof modeSummary !== "object" ||
      Array.isArray(modeSummary) ||
      typeof modeSummary.mode !== "string" ||
      modeSummary.mode.length === 0
    ) {
      throw new Error(`invalid automove performance mode at index ${index}`);
    }
    return modeSummary.mode;
  });
  assertUniqueModes(modeNames, "mode summaries");
  if (
    configuredModes.length !== modeNames.length ||
    configuredModes.some((mode, index) => modeNames[index] !== mode)
  ) {
    throw new Error("configured automove modes do not match mode summaries");
  }

  const rows = readRows(
    report.rows,
    new Set(configuredModes),
    configuration.samplesPerBundlePerState,
  );
  assertCanonicalModeInventories(rows, configuredModes);
  assertStateBankInventory(rows, configuredModes, provenance.stateBank.states);
  const expectedSummary = summarizeRows(rows);
  const summary = readSummary(report.summary, "summary", expectedSummary);
  if (readNonnegativeInteger(report.states, "report state count") !== summary.states) {
    throw new Error("inconsistent automove performance report state count");
  }
  const checkedModes = modeSummaries.map((modeSummary, index) => {
    const mode = modeNames[index];
    const modeRows = rows.filter((row) => row.mode === mode);
    if (modeRows.length === 0) {
      throw new Error(`automove performance mode ${mode} has no rows`);
    }
    return {
      mode,
      ...readSummary(modeSummary, `mode ${mode}`, summarizeRows(modeRows)),
    };
  });
  const failures = [];
  for (const mode of checkedModes) {
    appendFailures(failures, mode.mode, mode, maxMeanRatio, maxP95Ratio);
  }
  if (failures.length > 0) {
    throw new Error(`automove performance regression: ${failures.join("; ")}`);
  }
  return {
    modes: checkedModes,
    maxMeanRatio,
    maxP95Ratio,
  };
}

function readProvenance(report) {
  const artifacts = {};
  for (const field of ["baseline", "candidate", "stateBank"]) {
    const value = report[field];
    if (
      typeof value !== "string" ||
      value.length === 0 ||
      !path.isAbsolute(value) ||
      path.resolve(value) !== value
    ) {
      throw new Error(`invalid automove performance ${field} path`);
    }
    const sha256 = report[`${field}Sha256`];
    if (typeof sha256 !== "string" || !sha256Pattern.test(sha256)) {
      throw new Error(`invalid automove performance ${field} SHA-256`);
    }
    if (field === "stateBank") {
      let stateBank;
      try {
        stateBank = readPerformanceStateBank(value, report.stateBankManifest);
      } catch (error) {
        throw new Error(
          `could not read automove performance stateBank artifact: ${value}: ${errorMessage(error)}`,
        );
      }
      if (stateBank.sha256 !== sha256) {
        throw new Error(
          "automove performance stateBank SHA-256 does not match artifact",
        );
      }
      assertManifestProvenance(report, stateBank.manifest);
      artifacts.stateBank = stateBank;
      continue;
    }
    let artifact;
    try {
      artifact = readPublicBundleArtifact(value, field);
    } catch (error) {
      throw new Error(
        `could not read automove performance ${field} artifact: ${value}: ${errorMessage(error)}`,
      );
    }
    if (artifact.sha256 !== sha256) {
      throw new Error(`automove performance ${field} SHA-256 does not match artifact`);
    }
  }
  return artifacts;
}

function assertManifestProvenance(report, manifest) {
  const manifestPath = report.stateBankManifest;
  const manifestSha256 = report.stateBankManifestSha256;
  if (manifest === undefined) {
    if (manifestPath === undefined && manifestSha256 === undefined) return;
    throw new Error("invalid automove performance stateBank manifest provenance");
  }
  if (
    manifestPath !== manifest.path ||
    typeof manifestSha256 !== "string" ||
    !sha256Pattern.test(manifestSha256) ||
    manifestSha256 !== manifest.sha256
  ) {
    throw new Error(
      "automove performance stateBank manifest provenance does not match artifact",
    );
  }
  if (manifest.sourceBundleSha256 !== report.baselineSha256) {
    throw new Error(
      "automove performance stateBank manifest source bundle does not match baseline artifact",
    );
  }
}

function readConfiguration(config) {
  if (config === null || typeof config !== "object" || Array.isArray(config)) {
    throw new Error("invalid automove performance configuration");
  }
  const modes = config.modes;
  if (
    !Array.isArray(modes) ||
    modes.length === 0 ||
    modes.some((mode) => typeof mode !== "string" || mode.length === 0)
  ) {
    throw new Error("invalid configured automove modes");
  }
  if (modes.some((mode) => !AUTOMOVE_MODES.includes(mode))) {
    throw new Error("unsupported configured automove mode");
  }
  assertUniqueModes(modes, "configured modes");
  const canonicalModes = AUTOMOVE_MODES.filter((mode) => modes.includes(mode));
  if (modes.some((mode, index) => canonicalModes[index] !== mode)) {
    throw new Error("configured automove modes are not in canonical order");
  }
  const repeat = readPositiveInteger(config.repeat, "performance repeat");
  if (repeat > MAX_PERFORMANCE_REPEAT) {
    throw new Error(`performance repeat must be at most ${MAX_PERFORMANCE_REPEAT}`);
  }
  const samplesPerBundlePerState = readPositiveInteger(
    config.samplesPerBundlePerState,
    "samples per bundle per state",
  );
  if (
    config.order !== "ABBA" ||
    !Number.isSafeInteger(repeat * 2) ||
    samplesPerBundlePerState !== repeat * 2
  ) {
    throw new Error("invalid automove performance sampling configuration");
  }
  return { modes, repeat, samplesPerBundlePerState };
}

function assertUniqueModes(modes, label) {
  if (new Set(modes).size !== modes.length) {
    throw new Error(`duplicate automove ${label}`);
  }
}

function readRows(rows, configuredModes, expectedCallsPerBundle) {
  if (!Array.isArray(rows) || rows.length === 0) {
    throw new Error("automove performance report has no rows");
  }
  const identities = new Set();
  return rows.map((row, index) => {
    if (
      row === null ||
      typeof row !== "object" ||
      Array.isArray(row) ||
      typeof row.mode !== "string" ||
      !configuredModes.has(row.mode) ||
      typeof row.id !== "string" ||
      row.id.length === 0 ||
      typeof row.fen !== "string" ||
      row.fen.length === 0
    ) {
      throw new Error(`invalid automove performance row at index ${index}`);
    }
    const identity = `${row.mode}\0${row.id}`;
    if (identities.has(identity)) {
      throw new Error(`duplicate automove performance row: ${row.mode}/${row.id}`);
    }
    identities.add(identity);
    const label = `row ${row.mode}/${row.id}`;
    const callsPerBundle = readPositiveInteger(
      row.callsPerBundle,
      `${label} calls per bundle`,
    );
    if (callsPerBundle !== expectedCallsPerBundle) {
      throw new Error(`inconsistent ${label} sampling contract`);
    }
    const baseline = readRowSide(row.baseline, `${label} baseline`);
    const candidate = readRowSide(row.candidate, `${label} candidate`);
    if (
      baseline.samples.length + baseline.invalids.total !== callsPerBundle ||
      candidate.samples.length + candidate.invalids.total !== callsPerBundle
    ) {
      throw new Error(`inconsistent ${label} call counts`);
    }
    const ratios =
      baseline.invalids.total + candidate.invalids.total === 0
        ? timingRatios(candidate.timing, baseline.timing)
        : null;
    assertRatios(row.candidateBaselineRatio, ratios, label);
    return {
      mode: row.mode,
      id: row.id,
      fen: row.fen,
      callsPerBundle,
      baseline,
      candidate,
    };
  });
}

function assertCanonicalModeInventories(rows, configuredModes) {
  const [firstMode] = configuredModes;
  const canonicalRows = rows.filter((row) => row.mode === firstMode);
  if (canonicalRows.length === 0) {
    throw new Error(`automove performance mode ${firstMode} has no rows`);
  }
  const canonicalInventory = new Map(canonicalRows.map((row) => [row.id, row.fen]));
  for (const mode of configuredModes) {
    const modeRows = rows.filter((row) => row.mode === mode);
    const modeInventory = new Map(modeRows.map((row) => [row.id, row.fen]));
    if (
      modeInventory.size !== canonicalInventory.size ||
      [...canonicalInventory].some(([id, fen]) => modeInventory.get(id) !== fen)
    ) {
      throw new Error(
        `automove performance mode ${mode} has inconsistent state inventory`,
      );
    }
  }
}

function assertStateBankInventory(rows, configuredModes, states) {
  const reportInventory = new Map(
    rows
      .filter((row) => row.mode === configuredModes[0])
      .map((row) => [row.id, row.fen]),
  );
  const stateBankInventory = new Map(states.map((state) => [state.id, state.fen]));
  if (
    reportInventory.size !== stateBankInventory.size ||
    [...stateBankInventory].some(([id, fen]) => reportInventory.get(id) !== fen)
  ) {
    throw new Error("automove performance rows do not match state-bank inventory");
  }
}

function readRowSide(side, label) {
  if (side === null || typeof side !== "object" || Array.isArray(side)) {
    throw new Error(`invalid ${label} performance row`);
  }
  const samples = side.samplesMs;
  if (
    !Array.isArray(samples) ||
    samples.some(
      (sample) => typeof sample !== "number" || !Number.isFinite(sample) || sample < 0,
    )
  ) {
    throw new Error(`invalid ${label} timing samples`);
  }
  const timing = timingSummary(samples);
  assertTiming(side, timing, label);
  return {
    samples,
    timing,
    invalids: readInvalidCounts(side, label),
  };
}

function summarizeRows(rows) {
  const baselineSamples = rows.flatMap((row) => row.baseline.samples);
  const candidateSamples = rows.flatMap((row) => row.candidate.samples);
  const baseline = {
    timing: timingSummary(baselineSamples),
    invalids: sumInvalidCounts(rows.map((row) => row.baseline.invalids)),
  };
  const candidate = {
    timing: timingSummary(candidateSamples),
    invalids: sumInvalidCounts(rows.map((row) => row.candidate.invalids)),
  };
  return {
    rows: rows.length,
    states: new Set(rows.map((row) => row.id)).size,
    callsPerBundle: sumSafeIntegers(
      rows.map((row) => row.callsPerBundle),
      "summary calls per bundle",
    ),
    baseline,
    candidate,
    ratios:
      baseline.invalids.total + candidate.invalids.total === 0
        ? timingRatios(candidate.timing, baseline.timing)
        : null,
  };
}

function readSummary(summary, label, expected) {
  if (summary === null || typeof summary !== "object" || Array.isArray(summary)) {
    throw new Error(`invalid automove performance ${label}`);
  }
  const rows = readNonnegativeInteger(summary.rows, `${label} row count`);
  const states = readNonnegativeInteger(summary.states, `${label} state count`);
  const callsPerBundle = readNonnegativeInteger(
    summary.callsPerBundle,
    `${label} calls per bundle`,
  );
  if (
    rows !== expected.rows ||
    states !== expected.states ||
    callsPerBundle !== expected.callsPerBundle
  ) {
    throw new Error(`inconsistent automove performance ${label} row summary`);
  }
  const baselineInvalids = readInvalidCounts(summary.baseline, `${label} baseline`);
  const candidateInvalids = readInvalidCounts(summary.candidate, `${label} candidate`);
  assertInvalidCounts(
    baselineInvalids,
    expected.baseline.invalids,
    `${label} baseline`,
  );
  assertInvalidCounts(
    candidateInvalids,
    expected.candidate.invalids,
    `${label} candidate`,
  );
  assertTiming(summary.baseline, expected.baseline.timing, `${label} baseline`);
  assertTiming(summary.candidate, expected.candidate.timing, `${label} candidate`);
  assertRatios(summary.candidateBaselineRatio, expected.ratios, label);
  const invalidCalls = baselineInvalids.total + candidateInvalids.total;
  if (invalidCalls > 0) {
    return { rows, states, callsPerBundle, invalidCalls, meanRatio: 0, p95Ratio: 0 };
  }
  if (
    expected.ratios === null ||
    typeof expected.ratios.mean !== "number" ||
    typeof expected.ratios.p95 !== "number"
  ) {
    throw new Error(`${label} has no valid timing ratios`);
  }
  return {
    rows,
    states,
    callsPerBundle,
    invalidCalls,
    meanRatio: expected.ratios.mean,
    p95Ratio: expected.ratios.p95,
  };
}

function readInvalidCounts(side, role) {
  if (side === null || typeof side !== "object" || Array.isArray(side)) {
    throw new Error(`invalid ${role} performance summary`);
  }
  const invalids = side.invalids;
  if (invalids === null || typeof invalids !== "object" || Array.isArray(invalids)) {
    throw new Error(`invalid ${role} call counts`);
  }
  const total = readNonnegativeInteger(invalids.total, `${role} invalid total`);
  const reasonTotal = invalidReasonFields.reduce(
    (sum, field) =>
      sum + readNonnegativeInteger(invalids[field], `${role} invalid ${field} count`),
    0,
  );
  if (total !== reasonTotal) {
    throw new Error(`inconsistent ${role} invalid call counts`);
  }
  return Object.fromEntries([
    ["total", total],
    ...invalidReasonFields.map((field) => [field, invalids[field]]),
  ]);
}

function sumInvalidCounts(items) {
  return Object.fromEntries(
    ["total", ...invalidReasonFields].map((field) => [
      field,
      sumSafeIntegers(
        items.map((item) => item[field]),
        `aggregate invalid ${field} count`,
      ),
    ]),
  );
}

function assertInvalidCounts(actual, expected, label) {
  if (
    ["total", ...invalidReasonFields].some((field) => actual[field] !== expected[field])
  ) {
    throw new Error(`inconsistent ${label} invalid call counts`);
  }
}

function assertTiming(actual, expected, label) {
  if (
    actual === null ||
    typeof actual !== "object" ||
    Array.isArray(actual) ||
    ["count", "totalMs", "meanMs", "medianMs", "p95Ms", "maxMs"].some(
      (field) => actual[field] !== expected[field],
    )
  ) {
    throw new Error(`inconsistent ${label} timing summary`);
  }
}

function assertRatios(actual, expected, label) {
  if (expected === null) {
    if (actual !== null) {
      throw new Error(`inconsistent ${label} timing ratios`);
    }
    return;
  }
  if (
    actual === null ||
    typeof actual !== "object" ||
    Array.isArray(actual) ||
    ["mean", "median", "p95", "max"].some((field) => actual[field] !== expected[field])
  ) {
    throw new Error(`inconsistent ${label} timing ratios`);
  }
}

function sumSafeIntegers(values, label) {
  const total = values.reduce((sum, value) => sum + value, 0);
  if (!Number.isSafeInteger(total) || total < 0) {
    throw new Error(`${label} must be a nonnegative safe integer`);
  }
  return total;
}

function readNonnegativeInteger(value, label) {
  if (!Number.isSafeInteger(value) || value < 0) {
    throw new Error(`${label} must be a nonnegative safe integer`);
  }
  return value;
}

function readPositiveInteger(value, label) {
  const parsed = readNonnegativeInteger(value, label);
  if (parsed === 0) {
    throw new Error(`${label} must be positive`);
  }
  return parsed;
}

function appendFailures(failures, label, summary, maxMeanRatio, maxP95Ratio) {
  if (summary.invalidCalls > 0) {
    failures.push(`${label} contained ${summary.invalidCalls} invalid calls`);
  }
  if (summary.meanRatio > maxMeanRatio) {
    failures.push(`${label} mean ratio ${summary.meanRatio} exceeds ${maxMeanRatio}`);
  }
  if (summary.p95Ratio > maxP95Ratio) {
    failures.push(`${label} p95 ratio ${summary.p95Ratio} exceeds ${maxP95Ratio}`);
  }
}

function assertPositiveFinite(value, label) {
  if (typeof value !== "number" || !Number.isFinite(value) || value <= 0) {
    throw new Error(`${label} must be a positive finite number`);
  }
}

function errorMessage(error) {
  return error instanceof Error ? error.message : String(error);
}

function ratioOption(options, name, defaultValue) {
  const value = options.get(name);
  if (value === undefined) return defaultValue;
  if (typeof value !== "string" || value.trim().length === 0) {
    throw new Error(`--${name} must be a positive finite number`);
  }
  const parsed = Number(value);
  if (!Number.isFinite(parsed) || parsed <= 0) {
    throw new Error(`--${name} must be a positive finite number`);
  }
  return parsed;
}

function readReport(filePath) {
  const absolutePath = path.resolve(filePath);
  let report;
  try {
    report = JSON.parse(readFileSync(absolutePath, "utf8"));
  } catch {
    throw new Error(`could not read automove performance report: ${absolutePath}`);
  }
  return report;
}

export function main(arguments_ = process.argv.slice(2)) {
  const options = parseCliOptions(arguments_, [
    "report",
    "max-mean-ratio",
    "max-p95-ratio",
  ]);
  if (options.has("help")) {
    console.log(helpText);
    return;
  }
  const report = readReport(requiredOption(options, "report"));
  const result = checkAutomovePerformance(report, {
    maxMeanRatio: ratioOption(options, "max-mean-ratio", defaultMaxMeanRatio),
    maxP95Ratio: ratioOption(options, "max-p95-ratio", defaultMaxP95Ratio),
  });
  const checked = result.modes
    .map(
      ({ mode, meanRatio, p95Ratio }) => `${mode} mean ${meanRatio}, p95 ${p95Ratio}`,
    )
    .join("; ");
  console.log(
    `automove performance check passed: ${checked}; limits mean <= ${result.maxMeanRatio}, p95 <= ${result.maxP95Ratio}`,
  );
}

const invokedPath = process.argv[1]
  ? pathToFileURL(path.resolve(process.argv[1])).href
  : null;
if (invokedPath === import.meta.url) {
  try {
    main();
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  }
}
