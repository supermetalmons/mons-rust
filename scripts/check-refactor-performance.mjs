#!/usr/bin/env node

import path from "node:path";
import { pathToFileURL } from "node:url";

import { parseCliOptions, requiredOption } from "./automove-evidence-support.mjs";
import { readRegularFileArtifact } from "./evidence/artifacts.mjs";
import { runBundleExecutingCli } from "./evidence/options.mjs";
import { checkRefactorPerformanceReports } from "./evidence/refactor-performance.mjs";

const helpText = `Usage: node scripts/check-refactor-performance.mjs \\
  --candidate <current-candidate-bundle.mjs> \\
  --v6-report <performance-v6.json> \\
  --midpoint-report <performance-midpoint-64.json>

Or: npm run check:refactor-performance -- \\
  --v6-report <performance-v6.json> \\
  --midpoint-report <performance-midpoint-64.json>`;

function readReport(filePath, label) {
  const absolutePath = path.resolve(filePath);
  try {
    return JSON.parse(
      readRegularFileArtifact(absolutePath, `${label} report`).bytes.toString("utf8"),
    );
  } catch {
    throw new Error(`could not read ${label} performance report: ${absolutePath}`);
  }
}

export async function main(arguments_ = process.argv.slice(2)) {
  const options = parseCliOptions(arguments_, [
    "candidate",
    "v6-report",
    "midpoint-report",
  ]);
  if (options.has("help")) {
    console.log(helpText);
    return;
  }
  const v6Report = readReport(requiredOption(options, "v6-report"), "v6");
  const midpointReport = readReport(
    requiredOption(options, "midpoint-report"),
    "midpoint",
  );
  const result = await checkRefactorPerformanceReports(
    v6Report,
    midpointReport,
    requiredOption(options, "candidate"),
  );
  console.log(
    `refactor performance check passed: baseline ${result.baselineSha256}; candidate ${result.candidateSha256}; v6 13 states; midpoint 64 states`,
  );
}

const invokedPath = process.argv[1]
  ? pathToFileURL(path.resolve(process.argv[1])).href
  : null;
if (invokedPath === import.meta.url) {
  runBundleExecutingCli(main).catch((error) => {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  });
}
