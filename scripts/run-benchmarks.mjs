#!/usr/bin/env node

import { build } from "esbuild";
import { execFile } from "node:child_process";
import { createHash, randomUUID } from "node:crypto";
import { createServer } from "node:http";
import {
  access,
  link,
  lstat,
  mkdir,
  readFile,
  rename,
  unlink,
  writeFile,
} from "node:fs/promises";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { isDeepStrictEqual, parseArgs, promisify } from "node:util";

const execFileAsync = promisify(execFile);

const repositoryRoot = path.resolve(import.meta.dirname, "..");
const targetDirectory = path.join(repositoryRoot, "target", "benchmarks");
const suiteEntry = path.join(repositoryRoot, "benchmarks", "suite.ts");
const corpusFile = "test-data/automove-decisions/v4/decisions.jsonl";
const runnerFile = "scripts/run-benchmarks.mjs";
const browserReportMaxBytes = 1024 * 1024;
const benchmarkContractVersion = 2;
const corpusPath = path.join(repositoryRoot, ...corpusFile.split("/"));

const { values } = parseArgs({
  args: process.argv.slice(2),
  allowPositionals: false,
  options: {
    baseline: { type: "boolean" },
    "baseline-label": { type: "string", default: "baseline" },
    batches: { type: "string" },
    browser: { type: "boolean" },
    "build-only": { type: "boolean" },
    force: { type: "boolean" },
    label: { type: "string", default: "candidate" },
    node: { type: "boolean" },
    port: { type: "string", default: "4177" },
    "require-parity": { type: "boolean" },
    samples: { type: "string" },
    smoke: { type: "boolean" },
  },
  strict: true,
});

if ((values.browser ?? false) === (values.node ?? false)) {
  throw new Error("choose exactly one of --node or --browser");
}

const samples = Number(values.samples ?? (values.smoke ? 1 : 7));
if (!Number.isSafeInteger(samples) || samples < 1) {
  throw new RangeError("--samples must be a positive safe integer");
}
const batches = Number(values.batches ?? Math.min(3, samples));
if (!Number.isSafeInteger(batches) || batches < 1 || batches > samples) {
  throw new RangeError("--batches must be an integer from one through samples");
}
const label = values.label ?? "candidate";
const baselineLabel = values["baseline-label"] ?? "baseline";
if (
  !/^[a-z0-9][a-z0-9-]*$/u.test(label) ||
  !/^[a-z0-9][a-z0-9-]*$/u.test(baselineLabel)
) {
  throw new Error("labels must contain lowercase letters, digits, and hyphens");
}
const captureBaseline = values.baseline ?? label === "baseline";
const requireParity = values["require-parity"] ?? false;
const smoke = values.smoke ?? false;
const force = values.force ?? false;
const buildOnly = values["build-only"] ?? false;
if (values.node && buildOnly) {
  throw new Error("--build-only is only supported with --browser");
}
let browserPort;
if (values.browser && !buildOnly) {
  browserPort = Number(values.port ?? "4177");
  if (
    !Number.isSafeInteger(browserPort) ||
    browserPort < 1 ||
    browserPort > 65_535
  ) {
    throw new RangeError("--port must be an integer from 1 through 65535");
  }
}
const candidateStem = label === "candidate" ? label : `${label}-candidate`;
const platform = values.node ? "node" : "browser";
const browserSubmissionId = values.browser ? randomUUID() : undefined;
const suiteExtension = platform === "node" ? "mjs" : "js";
const baselineBundlePath = path.resolve(
  targetDirectory,
  `${baselineLabel}-${platform}-suite.${suiteExtension}`,
);
const candidateBundlePath = path.resolve(
  targetDirectory,
  `${candidateStem}-${platform}-suite.${suiteExtension}`,
);
const baselineReportPath = path.resolve(
  targetDirectory,
  `${baselineLabel}-${platform}.json`,
);
const candidateReportPath = path.resolve(
  targetDirectory,
  `${label}-${platform}.json`,
);

function rejectArtifactPathCollision(leftName, leftPath, rightName, rightPath) {
  if (path.resolve(leftPath) !== path.resolve(rightPath)) return;
  throw new Error(
    `benchmark artifact path collision: ${leftName} and ${rightName} both resolve to ${leftPath}`,
  );
}

if (!captureBaseline) {
  rejectArtifactPathCollision(
    "baseline bundle",
    baselineBundlePath,
    "candidate bundle",
    candidateBundlePath,
  );
  rejectArtifactPathCollision(
    "baseline report",
    baselineReportPath,
    "candidate report",
    candidateReportPath,
  );
  if (platform === "browser") {
    rejectArtifactPathCollision(
      "baseline browser controller",
      path.resolve(targetDirectory, `${baselineLabel}-browser-runner.js`),
      "candidate browser controller",
      path.resolve(targetDirectory, `${label}-browser-runner.js`),
    );
  }
}

await mkdir(targetDirectory, { recursive: true });
const corpusBytes = await readFile(corpusPath);
const runnerPath = path.join(repositoryRoot, "scripts", "run-benchmarks.mjs");
const runnerBytes = await readFile(runnerPath);

function sha256Bytes(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

const corpusSha256 = sha256Bytes(corpusBytes);
const runnerSha256 = sha256Bytes(runnerBytes);
const states = corpusBytes
  .toString("utf8")
  .trimEnd()
  .split("\n")
  .map((line) => JSON.parse(line));

async function fileExists(filePath) {
  try {
    await access(filePath);
    return true;
  } catch {
    return false;
  }
}

async function sourceState() {
  try {
    const [{ stdout: revision }, { stdout: status }] = await Promise.all([
      execFileAsync("git", ["rev-parse", "HEAD"], {
        cwd: repositoryRoot,
        encoding: "utf8",
      }),
      execFileAsync(
        "git",
        ["status", "--porcelain=v1", "--untracked-files=all"],
        { cwd: repositoryRoot, encoding: "utf8" },
      ),
    ]);
    return {
      sourceRevision: revision.trim(),
      sourceDirty: status.trim().length !== 0,
    };
  } catch {
    return { sourceRevision: "unknown", sourceDirty: true };
  }
}

function baselineManifestPath(platform) {
  return path.join(
    targetDirectory,
    `${baselineLabel}-${platform}-manifest.json`,
  );
}

async function describeBundle(bundlePath, artifactLabel, source, bundleBytes) {
  return {
    label: artifactLabel,
    bundleFile: path.basename(bundlePath),
    bundleSha256: sha256Bytes(bundleBytes ?? (await readFile(bundlePath))),
    corpusFile,
    corpusSha256,
    runnerFile,
    runnerSha256,
    ...source,
  };
}

async function refuseBaselineOverwrite(paths) {
  if (force) return;
  const existing = [];
  for (const filePath of paths) {
    if (await fileExists(filePath)) existing.push(path.basename(filePath));
  }
  if (existing.length !== 0) {
    throw new Error(
      `baseline artifacts already exist (${existing.join(", ")}); choose a new --baseline-label or pass --force to replace them`,
    );
  }
}

function isFileExistsError(error) {
  return (
    typeof error === "object" &&
    error !== null &&
    "code" in error &&
    error.code === "EEXIST"
  );
}

function baselineExistsError(filePath, cause) {
  const error = new Error(
    `baseline artifact already exists (${path.basename(filePath)}); choose a new --baseline-label or pass --force to replace it`,
    { cause },
  );
  error.code = "BASELINE_EXISTS";
  return error;
}

function isBaselineExistsError(error) {
  return (
    typeof error === "object" &&
    error !== null &&
    "code" in error &&
    error.code === "BASELINE_EXISTS"
  );
}

function isSha256(value) {
  return typeof value === "string" && /^[a-f0-9]{64}$/u.test(value);
}

function invalidManifest(manifestPath) {
  return new Error(`invalid baseline manifest: ${manifestPath}`);
}

function resolveManifestFile(fileName, manifestPath) {
  if (
    typeof fileName !== "string" ||
    fileName.length === 0 ||
    path.basename(fileName) !== fileName
  ) {
    throw invalidManifest(manifestPath);
  }
  const resolved = path.resolve(targetDirectory, fileName);
  if (path.dirname(resolved) !== path.resolve(targetDirectory)) {
    throw invalidManifest(manifestPath);
  }
  return resolved;
}

async function readRegularFile(filePath, manifestPath) {
  const stats = await lstat(filePath);
  if (!stats.isFile()) throw invalidManifest(manifestPath);
  return readFile(filePath);
}

async function verifyArtifactDigest(
  filePath,
  expectedDigest,
  manifestPath,
  description,
) {
  const bytes = await readRegularFile(filePath, manifestPath);
  if (sha256Bytes(bytes) !== expectedDigest) {
    throw new Error(
      `${description} digest does not match ${path.basename(manifestPath)}`,
    );
  }
  return bytes;
}

async function writeTemporarySibling(filePath, contents) {
  const temporaryPath = path.join(
    path.dirname(filePath),
    `.${path.basename(filePath)}.${randomUUID()}.tmp`,
  );
  try {
    await writeFile(temporaryPath, contents, { flag: "wx" });
    return temporaryPath;
  } catch (error) {
    await unlink(temporaryPath).catch(() => {});
    throw error;
  }
}

async function materializeImmutableFile(filePath, contents) {
  const bytes = Buffer.from(contents);
  const temporaryPath = await writeTemporarySibling(filePath, bytes);
  try {
    try {
      await link(temporaryPath, filePath);
    } catch (error) {
      if (!isFileExistsError(error)) throw error;
      const existing = await readRegularFile(filePath, filePath);
      if (!existing.equals(bytes)) {
        throw new Error(
          `immutable benchmark artifact has unexpected contents: ${path.basename(filePath)}`,
        );
      }
    }
  } finally {
    await unlink(temporaryPath).catch(() => {});
  }
  return filePath;
}

async function publishBaselineManifest(platform, artifact) {
  const manifest = {
    schemaVersion: 3,
    platform,
    capturedAt: new Date().toISOString(),
    artifact,
  };
  const manifestPath = baselineManifestPath(platform);
  const temporaryPath = await writeTemporarySibling(
    manifestPath,
    `${JSON.stringify(manifest, null, 2)}\n`,
  );
  try {
    if (force) {
      await rename(temporaryPath, manifestPath);
    } else {
      try {
        await link(temporaryPath, manifestPath);
      } catch (error) {
        if (!isFileExistsError(error)) throw error;
        throw baselineExistsError(manifestPath, error);
      }
    }
  } finally {
    await unlink(temporaryPath).catch(() => {});
  }
  return manifest;
}

async function materializeBaselineReport(platform, reportBytes) {
  const reportSha256 = sha256Bytes(reportBytes);
  const reportPath = path.join(
    targetDirectory,
    `report-${platform}-baseline-${reportSha256}.json`,
  );
  await materializeImmutableFile(reportPath, reportBytes);
  return { reportFile: path.basename(reportPath), reportPath, reportSha256 };
}

async function publishBaselineGeneration(platform, artifact, reportBytes) {
  let publishedArtifact = artifact;
  let reportPath;
  if (reportBytes !== undefined) {
    const report = await materializeBaselineReport(platform, reportBytes);
    reportPath = report.reportPath;
    publishedArtifact = {
      ...artifact,
      reportFile: report.reportFile,
      reportSha256: report.reportSha256,
    };
  }
  const manifest = await publishBaselineManifest(platform, publishedArtifact);
  return { manifest, reportPath };
}

function missingBaselineMessage(platform) {
  const displayPlatform = platform === "node" ? "Node" : "browser";
  return `baseline ${displayPlatform} bundle is missing; capture it first with --baseline --baseline-label=${baselineLabel}`;
}

async function firstExistingPath(paths) {
  for (const filePath of paths) {
    if (await fileExists(filePath)) return filePath;
  }
  return undefined;
}

async function loadBaseline(platform, legacyBundlePaths) {
  const manifestPath = baselineManifestPath(platform);
  if (!(await fileExists(manifestPath))) {
    const bundlePath = await firstExistingPath(legacyBundlePaths);
    if (bundlePath === undefined)
      throw new Error(missingBaselineMessage(platform));
    const bundleBytes = await readFile(bundlePath);
    const bundleSha256 = sha256Bytes(bundleBytes);
    if (requireParity) {
      throw new Error(
        `strict parity requires a schema-2 or schema-3 baseline manifest for ${path.basename(bundlePath)}`,
      );
    }
    console.warn(
      `baseline manifest is missing for ${path.basename(bundlePath)}; provenance is limited to the current bundle digest`,
    );
    return {
      artifact: {
        label: baselineLabel,
        bundleFile: path.basename(bundlePath),
        bundleSha256,
        sourceRevision: "unknown",
        sourceDirty: true,
        legacy: true,
      },
      bundlePath,
      bundleBytes,
    };
  }
  const manifest = JSON.parse(await readFile(manifestPath, "utf8"));
  if (
    typeof manifest !== "object" ||
    manifest === null ||
    manifest.platform !== platform ||
    typeof manifest.artifact !== "object" ||
    manifest.artifact === null
  ) {
    throw invalidManifest(manifestPath);
  }
  const artifact = manifest.artifact;
  if (manifest.schemaVersion === 3) {
    if (
      typeof manifest.capturedAt !== "string" ||
      artifact.label !== baselineLabel ||
      !isSha256(artifact.bundleSha256) ||
      artifact.bundleFile !==
        `module-${platform}-baseline-${artifact.bundleSha256}.${platform === "node" ? "mjs" : "js"}` ||
      artifact.corpusFile !== corpusFile ||
      !isSha256(artifact.corpusSha256) ||
      artifact.runnerFile !== runnerFile ||
      !isSha256(artifact.runnerSha256) ||
      typeof artifact.sourceRevision !== "string" ||
      typeof artifact.sourceDirty !== "boolean" ||
      (artifact.reportFile === undefined) !==
        (artifact.reportSha256 === undefined) ||
      (artifact.reportSha256 !== undefined &&
        (!isSha256(artifact.reportSha256) ||
          artifact.reportFile !==
            `report-${platform}-baseline-${artifact.reportSha256}.json`))
    ) {
      throw invalidManifest(manifestPath);
    }
    const bundlePath = resolveManifestFile(artifact.bundleFile, manifestPath);
    const bundleBytes = await verifyArtifactDigest(
      bundlePath,
      artifact.bundleSha256,
      manifestPath,
      "baseline bundle",
    );
    if (artifact.reportFile !== undefined) {
      const reportPath = resolveManifestFile(artifact.reportFile, manifestPath);
      await verifyArtifactDigest(
        reportPath,
        artifact.reportSha256,
        manifestPath,
        "baseline report",
      );
    }
    if (artifact.corpusSha256 !== corpusSha256) {
      throw new Error(
        `baseline corpus digest does not match ${path.basename(manifestPath)}`,
      );
    }
    if (artifact.runnerSha256 !== runnerSha256) {
      throw new Error(
        `baseline runner digest does not match ${path.basename(manifestPath)}`,
      );
    }
    const reportArtifact =
      artifact.reportFile === undefined
        ? {}
        : {
            reportFile: artifact.reportFile,
            reportSha256: artifact.reportSha256,
          };
    return {
      artifact: {
        label: artifact.label,
        bundleFile: artifact.bundleFile,
        bundleSha256: artifact.bundleSha256,
        corpusFile: artifact.corpusFile,
        corpusSha256: artifact.corpusSha256,
        runnerFile: artifact.runnerFile,
        runnerSha256: artifact.runnerSha256,
        sourceRevision: artifact.sourceRevision,
        sourceDirty: artifact.sourceDirty,
        capturedAt: manifest.capturedAt,
        ...reportArtifact,
      },
      bundlePath,
      bundleBytes,
    };
  }
  const bundlePath = legacyBundlePaths.find(
    (candidate) => path.basename(candidate) === artifact.bundleFile,
  );
  if (bundlePath === undefined) throw invalidManifest(manifestPath);
  const bundleBytes = await readRegularFile(bundlePath, manifestPath);
  const bundleSha256 = sha256Bytes(bundleBytes);
  if (artifact.bundleSha256 !== bundleSha256) {
    throw new Error(
      `baseline bundle digest does not match ${path.basename(manifestPath)}`,
    );
  }
  if (manifest.schemaVersion === 1) {
    if (
      typeof manifest.capturedAt !== "string" ||
      artifact.label !== baselineLabel ||
      artifact.bundleFile !== path.basename(bundlePath) ||
      !isSha256(artifact.bundleSha256) ||
      typeof artifact.sourceRevision !== "string" ||
      typeof artifact.sourceDirty !== "boolean"
    ) {
      throw invalidManifest(manifestPath);
    }
    if (requireParity) {
      throw new Error(
        `strict parity requires a schema-2 or schema-3 baseline manifest; ${path.basename(manifestPath)} is schema 1`,
      );
    }
    console.warn(
      `baseline manifest ${path.basename(manifestPath)} is schema 1; corpus and runner provenance cannot be verified`,
    );
    return {
      artifact: {
        label: artifact.label,
        bundleFile: path.basename(bundlePath),
        bundleSha256,
        sourceRevision: artifact.sourceRevision,
        sourceDirty: artifact.sourceDirty,
        capturedAt: manifest.capturedAt,
        legacy: true,
      },
      bundlePath,
      bundleBytes,
    };
  }
  if (
    manifest.schemaVersion !== 2 ||
    typeof manifest.capturedAt !== "string" ||
    artifact.label !== baselineLabel ||
    artifact.bundleFile !== path.basename(bundlePath) ||
    !isSha256(artifact.bundleSha256) ||
    artifact.corpusFile !== corpusFile ||
    !isSha256(artifact.corpusSha256) ||
    artifact.runnerFile !== runnerFile ||
    !isSha256(artifact.runnerSha256) ||
    typeof artifact.sourceRevision !== "string" ||
    typeof artifact.sourceDirty !== "boolean"
  ) {
    throw invalidManifest(manifestPath);
  }
  if (artifact.corpusSha256 !== corpusSha256) {
    throw new Error(
      `baseline corpus digest does not match ${path.basename(manifestPath)}`,
    );
  }
  if (artifact.runnerSha256 !== runnerSha256) {
    throw new Error(
      `baseline runner digest does not match ${path.basename(manifestPath)}`,
    );
  }
  return {
    artifact: {
      label: artifact.label,
      bundleFile: path.basename(bundlePath),
      bundleSha256,
      corpusFile: artifact.corpusFile,
      corpusSha256: artifact.corpusSha256,
      runnerFile: artifact.runnerFile,
      runnerSha256: artifact.runnerSha256,
      sourceRevision: artifact.sourceRevision,
      sourceDirty: artifact.sourceDirty,
      capturedAt: manifest.capturedAt,
    },
    bundlePath,
    bundleBytes,
  };
}

async function materializeModuleSnapshot(platform, role, bytes) {
  const digest = sha256Bytes(bytes);
  const extension = platform === "node" ? "mjs" : "js";
  const snapshotPath = path.join(
    targetDirectory,
    `module-${platform}-${role}-${digest}.${extension}`,
  );
  return materializeImmutableFile(snapshotPath, bytes);
}

async function buildSuite(outfile, platform) {
  const result = await build({
    bundle: true,
    entryPoints: [suiteEntry],
    format: "esm",
    keepNames: platform === "node",
    legalComments: "none",
    outfile,
    platform,
    target: platform === "node" ? "node22" : "es2020",
    write: false,
  });
  const output = result.outputFiles?.[0];
  if (output === undefined || result.outputFiles?.length !== 1) {
    throw new Error("benchmark suite build did not produce exactly one bundle");
  }
  return output.contents;
}

async function importFresh(modulePath) {
  return import(`${pathToFileURL(modulePath).href}?instance=${randomUUID()}`);
}

async function ensureNodeBaselineModule(bundleBytes) {
  const source = bundleBytes.toString("utf8");
  const moduleBytes = source.includes("packedDeadline")
    ? bundleBytes
    : Buffer.from(
        `${source}\nexport { FastSearcher, MonsGame, tryLoadPosition };\n`,
      );
  return materializeModuleSnapshot("node", "baseline", moduleBytes);
}

function baselineRunner(module, candidate) {
  return (benchmarkStates, options) => {
    const report = module.runBenchmarkSuite(benchmarkStates, options);
    if (report.packedDeadline !== undefined) return report;
    if (
      module.FastSearcher === undefined ||
      module.MonsGame === undefined ||
      module.tryLoadPosition === undefined
    ) {
      throw new Error("baseline bundle cannot provide packed deadline metrics");
    }
    const packedDeadline = candidate.runPackedDeadlineMetric(
      benchmarkStates,
      options,
      {
        createGame: (fen) => module.MonsGame.fromFen(fen, false),
        createSearcher: () => new module.FastSearcher(),
        tryLoadPosition: module.tryLoadPosition,
      },
    );
    return { ...report, packedDeadline };
  };
}

function checkMetricParity(report) {
  if (report.metricParity.complete) return;
  const message = JSON.stringify(report.metricParity, null, 2);
  if (requireParity) {
    throw new Error(`benchmark metric parity check failed:\n${message}`);
  }
  console.warn(`benchmark comparison is partial:\n${message}`);
}

async function runNodeBenchmark() {
  if (captureBaseline) {
    await refuseBaselineOverwrite([
      baselineBundlePath,
      baselineReportPath,
      baselineManifestPath("node"),
    ]);
    const baselineBytes = await buildSuite(baselineBundlePath, "node");
    const baselineModule = await materializeModuleSnapshot(
      "node",
      "baseline",
      baselineBytes,
    );
    const baseline = await importFresh(baselineModule);
    const report = baseline.runBenchmarkSuite(states, { samples, smoke });
    const deadlineStateIds = smoke
      ? Array.from(
          { length: samples },
          (_, sample) => states[sample % states.length]?.id ?? [],
        ).flat()
      : states.map((state) => state.id);
    baseline.validateBenchmarkReport(report, {
      options: { samples, smoke },
      stateCount: states.length,
      deadlineStateIds,
    });
    const trustedParity = baseline.benchmarkMetricParity(report, report, {
      stateCount: states.length,
      deadlineStateIds,
    });
    if (!trustedParity.complete) {
      throw new Error(
        "baseline report does not satisfy the benchmark contract",
      );
    }
    const artifact = await describeBundle(
      baselineModule,
      baselineLabel,
      await sourceState(),
      baselineBytes,
    );
    const reportWithArtifacts = {
      ...report,
      artifacts: { baseline: artifact },
    };
    const reportBytes = Buffer.from(
      `${JSON.stringify(reportWithArtifacts, null, 2)}\n`,
    );
    const generation = await publishBaselineGeneration(
      "node",
      artifact,
      reportBytes,
    );
    console.log(
      JSON.stringify(
        {
          baselineBundle: baselineModule,
          reportPath: generation.reportPath,
          report: reportWithArtifacts,
        },
        null,
        2,
      ),
    );
    return;
  }
  const loadedBaseline = await loadBaseline("node", [baselineBundlePath]);
  const baselineModule = await ensureNodeBaselineModule(
    loadedBaseline.bundleBytes,
  );
  const candidateBundle = candidateBundlePath;
  const candidateBytes = await buildSuite(candidateBundle, "node");
  await writeFile(candidateBundle, candidateBytes);
  const candidateModule = await materializeModuleSnapshot(
    "node",
    "candidate",
    candidateBytes,
  );
  const baseline = await importFresh(baselineModule);
  const candidate = await importFresh(candidateModule);
  const benchmarkReport = candidate.runInterleavedBenchmark(
    states,
    baselineRunner(baseline, candidate),
    candidate.runBenchmarkSuite,
    {
      allowLegacyBaseline:
        loadedBaseline.artifact.legacy === true && !requireParity,
      batches,
      samples,
      smoke,
    },
  );
  const report = {
    ...benchmarkReport,
    artifacts: {
      baseline: loadedBaseline.artifact,
      candidate: await describeBundle(
        candidateBundle,
        label,
        await sourceState(),
        candidateBytes,
      ),
    },
  };
  await writeFile(candidateReportPath, `${JSON.stringify(report, null, 2)}\n`);
  checkMetricParity(report);
  console.log(
    JSON.stringify(
      {
        baselineBundle: loadedBaseline.bundlePath,
        candidateBundle,
        reportPath: candidateReportPath,
        report,
      },
      null,
      2,
    ),
  );
}

function browserLegacyBaselineBundles() {
  const suitePath = path.join(
    targetDirectory,
    `${baselineLabel}-browser-suite.js`,
  );
  const legacyPath = path.join(targetDirectory, `${baselineLabel}-browser.js`);
  return [suitePath, legacyPath];
}

async function ensureBrowserBaselineModule(originalBundle, bundleBytes) {
  const source = bundleBytes.toString("utf8");
  const requiredExports =
    "export { FastSearcher, MonsGame, runBenchmarkSuite, tryLoadPosition };";
  if (source.includes("packedDeadline")) {
    return materializeModuleSnapshot("browser", "baseline", bundleBytes);
  }
  let importable;
  if (
    originalBundle.endsWith("-browser-suite.js") ||
    originalBundle.includes("module-browser-baseline-")
  ) {
    const trailingExport = source.match(/(?:^|\n)export\s*\{[^}]*\};\s*$/u);
    const implementation = (
      trailingExport === null
        ? source
        : source.slice(0, trailingExport.index ?? source.length)
    ).trimEnd();
    importable = `${implementation}\n\n${requiredExports}\n`;
  } else {
    const controllerMarker = source.lastIndexOf("\n// <stdin>\n");
    if (controllerMarker < 0) {
      throw new Error("legacy baseline browser bundle cannot be adapted");
    }
    importable = `${source.slice(0, controllerMarker)}\n${requiredExports}\n`;
  }
  return materializeModuleSnapshot(
    "browser",
    "baseline",
    Buffer.from(importable),
  );
}

function browserControllerSource(baselineBundle, candidateBundle, artifacts) {
  const baselineImport = `./${path.basename(baselineBundle)}`;
  const candidateImport = `./${path.basename(candidateBundle)}`;
  const allowLegacyBaseline =
    artifacts.baseline?.legacy === true && !requireParity;
  const invocation = captureBaseline
    ? `runCandidate(states, ${JSON.stringify({ samples, smoke })})`
    : `runInterleavedBenchmark(states, runBaseline, runCandidate, ${JSON.stringify({ allowLegacyBaseline, batches, samples, smoke })})`;
  return `
import * as baselineModule from ${JSON.stringify(baselineImport)};
import {
  runPackedDeadlineMetric,
  runBenchmarkSuite as runCandidate,
  runInterleavedBenchmark,
} from ${JSON.stringify(candidateImport)};

const states = ${JSON.stringify(states)};
const runBaseline = (benchmarkStates, options) => {
  const report = baselineModule.runBenchmarkSuite(benchmarkStates, options);
  if (report.packedDeadline !== undefined) return report;
  if (
    baselineModule.FastSearcher === undefined ||
    baselineModule.MonsGame === undefined ||
    baselineModule.tryLoadPosition === undefined
  ) {
    throw new Error("baseline bundle cannot provide packed deadline metrics");
  }
  return {
    ...report,
    packedDeadline: runPackedDeadlineMetric(benchmarkStates, options, {
      createGame: (fen) => baselineModule.MonsGame.fromFen(fen, false),
      createSearcher: () => new baselineModule.FastSearcher(),
      tryLoadPosition: baselineModule.tryLoadPosition,
    }),
  };
};
const output = document.querySelector("#output");
const run = document.querySelector("#run");
run.addEventListener("click", () => {
  run.disabled = true;
  setTimeout(async () => {
    let submitted = false;
    try {
      const benchmarkReport = ${invocation};
      const report = {
        ...benchmarkReport,
        artifacts: ${JSON.stringify(artifacts)},
      };
      if (report.metricParity !== undefined && !report.metricParity.complete) {
        const message = "benchmark metric parity check failed:\\n" +
          JSON.stringify(report.metricParity, null, 2);
        ${requireParity ? "" : "console.warn(message);"}
      }
      output.textContent = JSON.stringify(report, null, 2);
      const response = await fetch("/report", {
        body: JSON.stringify(report),
        headers: { "content-type": "application/json" },
        method: "POST",
      });
      submitted = response.ok ||
        (${JSON.stringify(requireParity)} && response.status === 422);
      if (!response.ok) {
        const detail = await response.text();
        throw new Error(
          "benchmark report was not accepted (" + response.status + ")" +
            (detail.length === 0 ? "" : ": " + detail),
        );
      }
    } catch (error) {
      output.textContent = String(error?.stack ?? error);
    } finally {
      if (!${JSON.stringify(captureBaseline || requireParity)} || !submitted) {
        run.disabled = false;
      }
    }
  }, 0);
});
`;
}

function browserRunMetadata() {
  return {
    contractVersion: benchmarkContractVersion,
    submissionId: browserSubmissionId,
    samples,
    ...(captureBaseline ? {} : { batches }),
    smoke,
    stateCount: states.length,
  };
}

async function buildBrowserBenchmark() {
  let baselineBundle;
  let candidateBundle;
  let artifacts;
  if (captureBaseline) {
    await refuseBaselineOverwrite([
      baselineBundlePath,
      path.join(targetDirectory, `${baselineLabel}-browser.js`),
      baselineReportPath,
      baselineManifestPath("browser"),
    ]);
    const baselineBytes = await buildSuite(baselineBundlePath, "browser");
    baselineBundle = await materializeModuleSnapshot(
      "browser",
      "baseline",
      baselineBytes,
    );
    candidateBundle = await materializeModuleSnapshot(
      "browser",
      "candidate",
      baselineBytes,
    );
    const artifact = await describeBundle(
      baselineBundle,
      baselineLabel,
      await sourceState(),
      baselineBytes,
    );
    artifacts = { run: browserRunMetadata(), baseline: artifact };
  } else {
    const loadedBaseline = await loadBaseline(
      "browser",
      browserLegacyBaselineBundles(),
    );
    baselineBundle = await ensureBrowserBaselineModule(
      loadedBaseline.bundlePath,
      loadedBaseline.bundleBytes,
    );
    const persistedCandidateBundle = candidateBundlePath;
    const candidateBytes = await buildSuite(
      persistedCandidateBundle,
      "browser",
    );
    await writeFile(persistedCandidateBundle, candidateBytes);
    candidateBundle = await materializeModuleSnapshot(
      "browser",
      "candidate",
      candidateBytes,
    );
    artifacts = {
      run: browserRunMetadata(),
      baseline: loadedBaseline.artifact,
      candidate: await describeBundle(
        persistedCandidateBundle,
        label,
        await sourceState(),
        candidateBytes,
      ),
    };
  }
  const controller = path.join(
    targetDirectory,
    `${captureBaseline ? baselineLabel : label}-browser-runner.js`,
  );
  const controllerBytes = Buffer.from(
    browserControllerSource(baselineBundle, candidateBundle, artifacts),
  );
  await writeFile(controller, controllerBytes);
  const html = `<!doctype html>
<html lang="en"><head><meta charset="utf-8"><title>Mons rules benchmark</title></head>
<body><button id="run" type="button">Run benchmark</button><pre id="output"></pre>
<script type="module" src="/${path.basename(controller)}"></script></body></html>\n`;
  const htmlBytes = Buffer.from(html);
  const htmlPath = path.join(
    targetDirectory,
    `${captureBaseline ? baselineLabel : label}-browser.html`,
  );
  await writeFile(htmlPath, htmlBytes);
  const htmlResponse = {
    body: htmlBytes,
    contentType: "text/html; charset=utf-8",
  };
  const servedFiles = new Map([
    ["/", htmlResponse],
    ["/index.html", htmlResponse],
    [`/${path.basename(htmlPath)}`, htmlResponse],
    [
      `/${path.basename(controller)}`,
      {
        body: controllerBytes,
        contentType: "text/javascript; charset=utf-8",
      },
    ],
  ]);
  return {
    baselineBundle,
    candidateBundle,
    controller,
    htmlPath,
    artifacts,
    servedFiles,
  };
}

function browserBenchmarkOutput(benchmark) {
  return {
    baselineBundle: benchmark.baselineBundle,
    candidateBundle: benchmark.candidateBundle,
    controller: benchmark.controller,
    htmlPath: benchmark.htmlPath,
    artifacts: benchmark.artifacts,
  };
}

function isRecord(value) {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isBrowserEnvironment(environment) {
  return (
    isRecord(environment) &&
    Object.keys(environment).length === 2 &&
    environment.runtime === "browser" &&
    typeof environment.userAgent === "string"
  );
}

function isBenchmarkReport(
  report,
  run,
  environment,
  allowLegacyContract = false,
) {
  return (
    isRecord(report) &&
    (report.contractVersion === benchmarkContractVersion ||
      (allowLegacyContract && report.contractVersion === undefined)) &&
    typeof report.generatedAt === "string" &&
    isDeepStrictEqual(report.environment, environment) &&
    isDeepStrictEqual(report.options, {
      samples: run.samples,
      smoke: run.smoke,
    }) &&
    report.stateCount === run.stateCount &&
    Array.isArray(report.timed) &&
    isRecord(report.packed) &&
    isRecord(report.memory)
  );
}

function expectedBatchSamples() {
  const result = new Array(batches).fill(Math.floor(samples / batches));
  for (let index = 0; index < samples % batches; index += 1) {
    result[index] += 1;
  }
  return result;
}

function expectedDeadlineSchedule() {
  return Array.from({ length: samples }, (_, sample) => {
    let indices;
    if (smoke) {
      indices = [sample % states.length];
    } else {
      const start = Math.floor((sample * states.length) / samples);
      const end = Math.floor(((sample + 1) * states.length) / samples);
      indices =
        start === end
          ? [sample % states.length]
          : Array.from({ length: end - start }, (_, index) => start + index);
    }
    return indices.flatMap((index) => states[index]?.id ?? []);
  });
}

function expectedExecutionOrder() {
  return Array.from({ length: samples }, (_, sample) =>
    sample % 2 === 0 ? ["baseline", "candidate"] : ["candidate", "baseline"],
  ).flat();
}

function validateBrowserReport(
  report,
  benchmark,
  benchmarkMetricParity,
  validateBenchmarkReport,
) {
  const expectedArtifacts = benchmark.artifacts;
  const run = expectedArtifacts.run;
  if (!isRecord(report)) throw new Error("report must be a JSON object");
  if (JSON.stringify(report.artifacts) !== JSON.stringify(expectedArtifacts)) {
    throw new Error("report artifacts do not match the served benchmark");
  }
  if (captureBaseline) {
    if (
      !isBrowserEnvironment(report.environment) ||
      !isBenchmarkReport(report, run, report.environment)
    ) {
      throw new Error("baseline report has an invalid shape");
    }
    const deadlineStateIds = smoke
      ? Array.from(
          { length: samples },
          (_, sample) => states[sample % states.length]?.id ?? [],
        ).flat()
      : states.map((state) => state.id);
    validateBenchmarkReport(report, {
      options: { samples, smoke },
      stateCount: run.stateCount,
      deadlineStateIds,
    });
    const trustedParity = benchmarkMetricParity(report, report, {
      stateCount: run.stateCount,
      deadlineStateIds,
    });
    if (!trustedParity.complete) {
      throw new Error(
        "baseline report does not satisfy the benchmark contract",
      );
    }
    return trustedParity;
  }
  const deadlineStateSchedule = expectedDeadlineSchedule();
  const expectedOptions = {
    samples,
    batches,
    batchSamples: expectedBatchSamples(),
    smoke,
    warmupRuns: 1,
    deadlineStateSchedule,
  };
  const allowLegacyBaseline =
    !requireParity &&
    isRecord(expectedArtifacts.baseline) &&
    expectedArtifacts.baseline.legacy === true;
  if (
    report.contractVersion !== benchmarkContractVersion ||
    typeof report.generatedAt !== "string" ||
    !isBrowserEnvironment(report.environment) ||
    !isDeepStrictEqual(report.options, expectedOptions) ||
    report.stateCount !== run.stateCount ||
    !isDeepStrictEqual(report.executionOrder, expectedExecutionOrder()) ||
    !isRecord(report.metricParity) ||
    !isRecord(report.implementations) ||
    !isBenchmarkReport(
      report.implementations.baseline,
      run,
      report.environment,
      allowLegacyBaseline,
    ) ||
    !isBenchmarkReport(
      report.implementations.candidate,
      run,
      report.environment,
    )
  ) {
    throw new Error("comparison report has an invalid shape");
  }
  const implementationValidation = {
    options: { samples, smoke },
    stateCount: run.stateCount,
    deadlineStateIds: deadlineStateSchedule.flat(),
  };
  validateBenchmarkReport(report.implementations.candidate, {
    ...implementationValidation,
  });
  validateBenchmarkReport(report.implementations.baseline, {
    ...implementationValidation,
    allowLegacy: allowLegacyBaseline,
  });
  const trustedParity = benchmarkMetricParity(
    report.implementations.baseline,
    report.implementations.candidate,
    {
      stateCount: run.stateCount,
      deadlineStateIds: deadlineStateSchedule.flat(),
    },
  );
  if (!isDeepStrictEqual(report.metricParity, trustedParity)) {
    throw new Error("reported metric parity does not match recomputed parity");
  }
  return trustedParity;
}

async function serveBrowserBenchmark() {
  if (buildOnly) {
    const benchmark = await buildBrowserBenchmark();
    if (captureBaseline) {
      await publishBaselineGeneration("browser", benchmark.artifacts.baseline);
    }
    console.log(JSON.stringify(browserBenchmarkOutput(benchmark), null, 2));
    return;
  }
  const server = createServer();
  await new Promise((resolve, reject) => {
    const onError = (error) => {
      server.off("listening", onListening);
      reject(error);
    };
    const onListening = () => {
      server.off("error", onError);
      resolve();
    };
    server.once("error", onError);
    server.once("listening", onListening);
    server.listen(browserPort, "127.0.0.1");
  });
  try {
    const benchmark = await buildBrowserBenchmark();
    const candidate = await importFresh(benchmark.candidateBundle);
    if (
      typeof candidate.benchmarkMetricParity !== "function" ||
      typeof candidate.validateBenchmarkReport !== "function"
    ) {
      throw new Error(
        "candidate browser bundle does not export benchmark validation functions",
      );
    }
    const benchmarkMetricParity = candidate.benchmarkMetricParity;
    const validateBenchmarkReport = candidate.validateBenchmarkReport;
    const oneShotSubmission = captureBaseline || requireParity;
    let submissionClaimed = false;
    let shutdownRequested = false;
    const closeServerWhenResponseSettles = (response) => {
      const closeServer = () => {
        if (shutdownRequested) return;
        shutdownRequested = true;
        server.close();
      };
      if (response.destroyed) {
        closeServer();
        return;
      }
      response.once("finish", closeServer);
      response.once("close", closeServer);
    };
    const handleRequest = async (request, response) => {
      const pathname = new URL(request.url ?? "/", "http://127.0.0.1").pathname;
      if (request.method === "POST" && pathname === "/report") {
        if (oneShotSubmission) {
          if (submissionClaimed) {
            response.writeHead(409);
            response.end("benchmark report submission was already claimed");
            return;
          }
          submissionClaimed = true;
        }
        const releaseSubmission = () => {
          if (oneShotSubmission) submissionClaimed = false;
        };
        try {
          const contentType = request.headers["content-type"];
          if (
            typeof contentType !== "string" ||
            contentType.split(";", 1)[0]?.trim().toLowerCase() !==
              "application/json"
          ) {
            releaseSubmission();
            request.resume();
            response.writeHead(415);
            response.end("report content type must be application/json");
            return;
          }
          const contentLength = request.headers["content-length"];
          if (
            typeof contentLength === "string" &&
            Number(contentLength) > browserReportMaxBytes
          ) {
            releaseSubmission();
            request.resume();
            response.writeHead(413);
            response.end("benchmark report is too large");
            return;
          }
          const chunks = [];
          let bodyBytes = 0;
          for await (const chunk of request) {
            bodyBytes += chunk.length;
            if (bodyBytes > browserReportMaxBytes) {
              releaseSubmission();
              response.writeHead(413);
              response.end("benchmark report is too large");
              return;
            }
            chunks.push(chunk);
          }
          const body = Buffer.concat(chunks).toString("utf8");
          let report;
          let trustedParity;
          try {
            report = JSON.parse(body);
            trustedParity = validateBrowserReport(
              report,
              benchmark,
              benchmarkMetricParity,
              validateBenchmarkReport,
            );
          } catch (error) {
            releaseSubmission();
            response.writeHead(400);
            response.end(String(error?.message ?? error));
            return;
          }
          const reportBytes = Buffer.from(
            `${JSON.stringify(report, null, 2)}\n`,
          );
          if (captureBaseline) {
            try {
              await publishBaselineGeneration(
                "browser",
                benchmark.artifacts.baseline,
                reportBytes,
              );
            } catch (error) {
              if (!isBaselineExistsError(error)) throw error;
              releaseSubmission();
              response.writeHead(409);
              response.end("baseline artifacts already exist");
              return;
            }
          } else {
            await writeFile(candidateReportPath, reportBytes);
          }
          if (
            !captureBaseline &&
            requireParity &&
            trustedParity.complete === false
          ) {
            process.exitCode = 1;
            response.writeHead(422);
            closeServerWhenResponseSettles(response);
            response.end("benchmark metric parity check failed");
            return;
          }
          response.writeHead(204);
          if (oneShotSubmission) closeServerWhenResponseSettles(response);
          response.end();
          return;
        } catch (error) {
          releaseSubmission();
          throw error;
        }
      }
      const servedFile = benchmark.servedFiles.get(pathname);
      if (servedFile !== undefined) {
        response.writeHead(200, {
          "cache-control": "no-store",
          "content-type": servedFile.contentType,
        });
        response.end(servedFile.body);
        return;
      }
      const filePath = path.join(targetDirectory, path.basename(pathname));
      try {
        const body = await readFile(filePath);
        response.writeHead(200, {
          "cache-control": "no-store",
          "content-type": filePath.endsWith(".js")
            ? "text/javascript; charset=utf-8"
            : "text/html; charset=utf-8",
        });
        response.end(body);
      } catch {
        response.writeHead(404);
        response.end("not found");
      }
    };
    server.on("request", (request, response) => {
      void handleRequest(request, response).catch((error) => {
        console.error(error);
        if (response.headersSent) {
          response.destroy(error);
          return;
        }
        response.writeHead(500);
        response.end("benchmark server failed");
      });
    });
    console.log(`http://127.0.0.1:${browserPort}/`);
  } catch (error) {
    await new Promise((resolve) => {
      server.close(() => resolve());
    });
    throw error;
  }
}

if (values.node) await runNodeBenchmark();
else await serveBrowserBenchmark();
