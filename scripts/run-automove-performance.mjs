#!/usr/bin/env node

import path from "node:path";
import { pathToFileURL } from "node:url";

import {
  addInvalid,
  emptyInvalidCounts,
  loadPublicBundle,
  mergeInvalidCounts,
  parseCliOptions,
  positiveIntegerOption,
  preflightJsonReportDestination,
  readStateBank,
  requiredOption,
  runValidatedSuggestion,
  selectModes,
  timingRatios,
  timingSummary,
  validateStateBankVariants,
  writeJsonReport,
} from "./automove-evidence-support.mjs";

const measurementOrder = Object.freeze([
  "baseline",
  "candidate",
  "candidate",
  "baseline",
]);

const helpText = `Usage: node scripts/run-automove-performance.mjs \\
  --baseline <bundle.mjs> --candidate <bundle.mjs> \\
  --states <states.jsonl> --out <report.json> \\
  [--modes fast,normal,pro] [--repeat 5]

Each repeat is one ABBA block and produces two samples per bundle and state.`;

export async function runPerformanceEvidence(configuration) {
  const baseline = await loadPublicBundle(configuration.baseline, "baseline");
  const candidate = await loadPublicBundle(
    configuration.candidate,
    "candidate",
  );
  const modes = selectModes(configuration.modes);
  const stateBank = readStateBank(configuration.states);
  validateStateBankVariants(stateBank, baseline, candidate);
  const records = [];

  for (const mode of modes) {
    for (const state of stateBank.states) {
      records.push(
        measureState({
          baseline,
          candidate,
          mode,
          repeat: configuration.repeat,
          state,
        }),
      );
    }
  }

  const rows = records.map((record) => record.row);
  const modeSummaries = modes.map((mode) =>
    summarizeRecords(
      mode,
      records.filter((record) => record.row.mode === mode),
    ),
  );
  return {
    schemaVersion: 1,
    kind: "automove-performance",
    baseline: baseline.path,
    baselineSha256: baseline.sha256,
    candidate: candidate.path,
    candidateSha256: candidate.sha256,
    stateBank: stateBank.path,
    stateBankSha256: stateBank.sha256,
    config: {
      modes,
      repeat: configuration.repeat,
      order: "ABBA",
      samplesPerBundlePerState: configuration.repeat * 2,
    },
    states: stateBank.states.length,
    summary: summarizeRecords(null, records),
    modes: modeSummaries,
    rows,
  };
}

function measureState({ baseline, candidate, mode, repeat, state }) {
  const samples = { baseline: [], candidate: [] };
  const invalids = {
    baseline: emptyInvalidCounts(),
    candidate: emptyInvalidCounts(),
  };
  for (let block = 0; block < repeat; block += 1) {
    for (const role of measurementOrder) {
      const bundle = role === "baseline" ? baseline : candidate;
      const validationBundle = role === "baseline" ? candidate : baseline;
      const result = runValidatedSuggestion(
        bundle,
        validationBundle,
        state.fen,
        mode,
      );
      if (result.ok && typeof result.elapsedMs === "number") {
        samples[role].push(result.elapsedMs);
      }
      if (!result.ok) addInvalid(invalids[role], result.reason);
    }
  }
  const baselineTiming = timingSummary(samples.baseline);
  const candidateTiming = timingSummary(samples.candidate);
  return {
    row: {
      mode,
      id: state.id,
      fen: state.fen,
      callsPerBundle: repeat * 2,
      baseline: {
        ...baselineTiming,
        samplesMs: [...samples.baseline],
        invalids: invalids.baseline,
      },
      candidate: {
        ...candidateTiming,
        samplesMs: [...samples.candidate],
        invalids: invalids.candidate,
      },
      candidateBaselineRatio:
        invalids.baseline.total === 0 && invalids.candidate.total === 0
          ? timingRatios(candidateTiming, baselineTiming)
          : null,
    },
    samples,
    invalids,
  };
}

function summarizeRecords(mode, records) {
  const baselineSamples = records.flatMap((record) => record.samples.baseline);
  const candidateSamples = records.flatMap(
    (record) => record.samples.candidate,
  );
  const baselineTiming = timingSummary(baselineSamples);
  const candidateTiming = timingSummary(candidateSamples);
  const baselineInvalids = mergeInvalidCounts(
    records.map((record) => record.invalids.baseline),
  );
  const candidateInvalids = mergeInvalidCounts(
    records.map((record) => record.invalids.candidate),
  );
  return {
    ...(mode === null ? {} : { mode }),
    states: new Set(records.map((record) => record.row.id)).size,
    rows: records.length,
    callsPerBundle: records.reduce(
      (total, record) => total + record.row.callsPerBundle,
      0,
    ),
    baseline: { ...baselineTiming, invalids: baselineInvalids },
    candidate: { ...candidateTiming, invalids: candidateInvalids },
    candidateBaselineRatio:
      baselineInvalids.total === 0 && candidateInvalids.total === 0
        ? timingRatios(candidateTiming, baselineTiming)
        : null,
  };
}

export async function main(arguments_ = process.argv.slice(2)) {
  const options = parseCliOptions(arguments_, [
    "baseline",
    "candidate",
    "states",
    "repeat",
    "modes",
    "out",
  ]);
  if (options.has("help")) {
    console.log(helpText);
    return;
  }
  const configuration = {
    baseline: requiredOption(options, "baseline"),
    candidate: requiredOption(options, "candidate"),
    states: requiredOption(options, "states"),
    repeat: positiveIntegerOption(options, "repeat", 5, 100),
    modes: options.get("modes"),
  };
  const outputPath = preflightJsonReportDestination(
    requiredOption(options, "out"),
  );
  const report = await runPerformanceEvidence(configuration);
  writeJsonReport(outputPath, report);
  console.log(
    `automove performance: ${report.states} states, ` +
      `${report.config.modes.length} modes, ABBA x${report.config.repeat} -> ` +
      outputPath,
  );
  const invalidCalls =
    report.summary.baseline.invalids.total +
    report.summary.candidate.invalids.total;
  if (invalidCalls > 0) {
    throw new Error(
      `automove performance contained ${invalidCalls} invalid calls; report written to ${outputPath}`,
    );
  }
}

const invokedPath = process.argv[1]
  ? pathToFileURL(path.resolve(process.argv[1])).href
  : null;
if (invokedPath === import.meta.url) {
  main().catch((error) => {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  });
}
