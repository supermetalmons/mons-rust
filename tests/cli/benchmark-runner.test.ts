import { spawn, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { once } from "node:events";
import {
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { connect, createServer } from "node:net";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import { afterEach, beforeAll, describe, expect, it } from "vitest";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));
const targetDirectory = path.join(repositoryRoot, "target", "benchmarks");
const runnerPath = path.join(repositoryRoot, "scripts", "run-benchmarks.mjs");
const corpusPath = path.join(
  repositoryRoot,
  "test-data",
  "automove-decisions",
  "v4",
  "decisions.jsonl",
);
const prefix = `benchmark-runner-test-${process.pid}`;
const bundleCollisionLabel = `${prefix}-bundle`;
const bundleCollisionBaselineLabel = `${bundleCollisionLabel}-candidate`;
const bundleCollisionPath = path.join(
  targetDirectory,
  `${bundleCollisionBaselineLabel}-node-suite.mjs`,
);
const reportCollisionLabel = `${prefix}-report`;
const reportCollisionPath = path.join(
  targetDirectory,
  `${reportCollisionLabel}-node.json`,
);
const digestBaselineLabel = `${prefix}-digest`;
const digestCandidateLabel = `${prefix}-digest-check`;
const digestBundlePath = path.join(
  targetDirectory,
  `${digestBaselineLabel}-node-suite.mjs`,
);
const digestManifestPath = path.join(
  targetDirectory,
  `${digestBaselineLabel}-node-manifest.json`,
);
const digestAdapterPath = path.join(
  targetDirectory,
  `${digestBaselineLabel}-node-implementation.mjs`,
);
const digestMarkerPath = path.join(
  targetDirectory,
  `${digestBaselineLabel}-executed.txt`,
);
const digestCandidateBundlePath = path.join(
  targetDirectory,
  `${digestCandidateLabel}-candidate-node-suite.mjs`,
);
const digestCandidateReportPath = path.join(
  targetDirectory,
  `${digestCandidateLabel}-node.json`,
);
const legacyBaselineLabel = `${prefix}-legacy`;
const legacyCandidateLabel = `${prefix}-legacy-check`;
const legacyBundlePath = path.join(
  targetDirectory,
  `${legacyBaselineLabel}-node-suite.mjs`,
);
const legacyManifestPath = path.join(
  targetDirectory,
  `${legacyBaselineLabel}-node-manifest.json`,
);
const legacyAdapterPath = path.join(
  targetDirectory,
  `${legacyBaselineLabel}-node-implementation.mjs`,
);
const legacyMarkerPath = path.join(
  targetDirectory,
  `${legacyBaselineLabel}-executed.txt`,
);
const legacyCandidateBundlePath = path.join(
  targetDirectory,
  `${legacyCandidateLabel}-candidate-node-suite.mjs`,
);
const legacyCandidateReportPath = path.join(
  targetDirectory,
  `${legacyCandidateLabel}-node.json`,
);
const legacyExecutionBaselineLabel = `${prefix}-legacy-execution`;
const legacyExecutionCandidateLabel = `${prefix}-legacy-execution-candidate`;
const legacyExecutionBundlePath = path.join(
  targetDirectory,
  `${legacyExecutionBaselineLabel}-node-suite.mjs`,
);
const legacyExecutionCandidateBundlePath = path.join(
  targetDirectory,
  `${legacyExecutionCandidateLabel}-candidate-node-suite.mjs`,
);
const legacyExecutionReportPath = path.join(
  targetDirectory,
  `${legacyExecutionCandidateLabel}-node.json`,
);
const injectedLegacyBaselineLabel = `${prefix}-injected-legacy`;
const injectedLegacyCandidateLabel = `${prefix}-injected-legacy-candidate`;
const injectedLegacyBundlePath = path.join(
  targetDirectory,
  `${injectedLegacyBaselineLabel}-node-suite.mjs`,
);
const injectedLegacyManifestPath = path.join(
  targetDirectory,
  `${injectedLegacyBaselineLabel}-node-manifest.json`,
);
const injectedLegacyCandidateBundlePath = path.join(
  targetDirectory,
  `${injectedLegacyCandidateLabel}-candidate-node-suite.mjs`,
);
const injectedLegacyReportPath = path.join(
  targetDirectory,
  `${injectedLegacyCandidateLabel}-node.json`,
);
const invalidPortLabel = `${prefix}-invalid-port`;
const invalidPortCandidatePath = path.join(
  targetDirectory,
  `${invalidPortLabel}-candidate-browser-suite.js`,
);
const occupiedPortLabel = `${prefix}-occupied-port`;
const occupiedPortBundlePath = path.join(
  targetDirectory,
  `${occupiedPortLabel}-browser-suite.js`,
);
const occupiedPortManifestPath = path.join(
  targetDirectory,
  `${occupiedPortLabel}-browser-manifest.json`,
);
const occupiedPortReportPath = path.join(
  targetDirectory,
  `${occupiedPortLabel}-browser.json`,
);
const occupiedPortControllerPath = path.join(
  targetDirectory,
  `${occupiedPortLabel}-browser-runner.js`,
);
const nodeBuildOnlyLabel = `${prefix}-node-build-only`;
const nodeCaptureLabel = `${prefix}-node-capture`;
const nodeCaptureCandidateLabel = `${prefix}-node-capture-check`;
const nodeCaptureManifestPath = path.join(
  targetDirectory,
  `${nodeCaptureLabel}-node-manifest.json`,
);
const nodeCaptureLegacyBundlePath = path.join(
  targetDirectory,
  `${nodeCaptureLabel}-node-suite.mjs`,
);
const nodeCaptureLegacyReportPath = path.join(
  targetDirectory,
  `${nodeCaptureLabel}-node.json`,
);
const nodeCaptureCandidateBundlePath = path.join(
  targetDirectory,
  `${nodeCaptureCandidateLabel}-candidate-node-suite.mjs`,
);
const nodeCaptureCandidateReportPath = path.join(
  targetDirectory,
  `${nodeCaptureCandidateLabel}-node.json`,
);
const invalidSchema3BaselineLabel = `${prefix}-invalid-schema3`;
const invalidSchema3CandidateLabel = `${prefix}-invalid-schema3-check`;
const invalidSchema3ManifestPath = path.join(
  targetDirectory,
  `${invalidSchema3BaselineLabel}-node-manifest.json`,
);
const invalidSchema3CandidateBundlePath = path.join(
  targetDirectory,
  `${invalidSchema3CandidateLabel}-candidate-node-suite.mjs`,
);
const concurrentNodeBaselineLabel = `${prefix}-concurrent-node`;
const concurrentNodeManifestPath = path.join(
  targetDirectory,
  `${concurrentNodeBaselineLabel}-node-manifest.json`,
);
const browserCaptureLabel = `${prefix}-browser-capture`;
const browserCaptureManifestPath = path.join(
  targetDirectory,
  `${browserCaptureLabel}-browser-manifest.json`,
);
const browserCaptureLegacyBundlePath = path.join(
  targetDirectory,
  `${browserCaptureLabel}-browser-suite.js`,
);
const browserCaptureLegacyReportPath = path.join(
  targetDirectory,
  `${browserCaptureLabel}-browser.json`,
);
const browserCaptureControllerPath = path.join(
  targetDirectory,
  `${browserCaptureLabel}-browser-runner.js`,
);
const browserCaptureHtmlPath = path.join(
  targetDirectory,
  `${browserCaptureLabel}-browser.html`,
);
const forcedBrowserCaptureLabel = `${prefix}-forced-browser-capture`;
const forcedBrowserBundlePath = path.join(
  targetDirectory,
  `${forcedBrowserCaptureLabel}-browser-suite.js`,
);
const forcedBrowserManifestPath = path.join(
  targetDirectory,
  `${forcedBrowserCaptureLabel}-browser-manifest.json`,
);
const forcedBrowserControllerPath = path.join(
  targetDirectory,
  `${forcedBrowserCaptureLabel}-browser-runner.js`,
);
const forcedBrowserHtmlPath = path.join(
  targetDirectory,
  `${forcedBrowserCaptureLabel}-browser.html`,
);
const splitBaselineLabel = `${prefix}-split-baseline`;
const splitCandidateLabel = `${prefix}-split-candidate`;
const splitBaselineBundlePath = path.join(
  targetDirectory,
  `${splitBaselineLabel}-browser-suite.js`,
);
const splitCandidateBundlePath = path.join(
  targetDirectory,
  `${splitCandidateLabel}-candidate-browser-suite.js`,
);
const splitControllerPath = path.join(
  targetDirectory,
  `${splitCandidateLabel}-browser-runner.js`,
);
const httpBaselineLabel = `${prefix}-http-baseline`;
const httpCandidateLabel = `${prefix}-http-candidate`;
const httpBaselineBundlePath = path.join(
  targetDirectory,
  `${httpBaselineLabel}-browser-suite.js`,
);
const httpBaselineManifestPath = path.join(
  targetDirectory,
  `${httpBaselineLabel}-browser-manifest.json`,
);
const httpCandidateBundlePath = path.join(
  targetDirectory,
  `${httpCandidateLabel}-candidate-browser-suite.js`,
);
const httpControllerPath = path.join(
  targetDirectory,
  `${httpCandidateLabel}-browser-runner.js`,
);
const httpReportPath = path.join(
  targetDirectory,
  `${httpCandidateLabel}-browser.json`,
);
const sameBytesBaselineLabel = `${prefix}-same-bytes-baseline`;
const sameBytesCandidateLabel = `${prefix}-same-bytes-candidate`;
const concurrentABaselineLabel = `${prefix}-concurrent-a-baseline`;
const concurrentACandidateLabel = `${prefix}-concurrent-a-candidate`;
const concurrentBBaselineLabel = `${prefix}-concurrent-b-baseline`;
const concurrentBCandidateLabel = `${prefix}-concurrent-b-candidate`;
const lifecycleBaselineLabel = `${prefix}-lifecycle-baseline`;
const strictSuccessCandidateLabel = `${prefix}-strict-success`;
const nonStrictCandidateLabel = `${prefix}-non-strict`;
const timedMetricNames = [
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

function browserComparisonPaths(baseline: string, candidate: string) {
  return {
    baselineBundle: path.join(targetDirectory, `${baseline}-browser-suite.js`),
    baselineManifest: path.join(
      targetDirectory,
      `${baseline}-browser-manifest.json`,
    ),
    baselineReport: path.join(targetDirectory, `${baseline}-browser.json`),
    candidateBundle: path.join(
      targetDirectory,
      `${candidate}-candidate-browser-suite.js`,
    ),
    candidateController: path.join(
      targetDirectory,
      `${candidate}-browser-runner.js`,
    ),
    candidateHtml: path.join(targetDirectory, `${candidate}-browser.html`),
    candidateReport: path.join(targetDirectory, `${candidate}-browser.json`),
  };
}

const sameBytesPaths = browserComparisonPaths(
  sameBytesBaselineLabel,
  sameBytesCandidateLabel,
);
const sameBytesBaselineControllerPath = path.join(
  targetDirectory,
  `${sameBytesBaselineLabel}-browser-runner.js`,
);
const sameBytesBaselineHtmlPath = path.join(
  targetDirectory,
  `${sameBytesBaselineLabel}-browser.html`,
);
const concurrentAPaths = browserComparisonPaths(
  concurrentABaselineLabel,
  concurrentACandidateLabel,
);
const concurrentBPaths = browserComparisonPaths(
  concurrentBBaselineLabel,
  concurrentBCandidateLabel,
);
const strictSuccessPaths = browserComparisonPaths(
  lifecycleBaselineLabel,
  strictSuccessCandidateLabel,
);
const nonStrictPaths = browserComparisonPaths(
  lifecycleBaselineLabel,
  nonStrictCandidateLabel,
);
const testFiles = [
  bundleCollisionPath,
  reportCollisionPath,
  digestBundlePath,
  digestManifestPath,
  digestAdapterPath,
  digestMarkerPath,
  digestCandidateBundlePath,
  digestCandidateReportPath,
  legacyBundlePath,
  legacyManifestPath,
  legacyAdapterPath,
  legacyMarkerPath,
  legacyCandidateBundlePath,
  legacyCandidateReportPath,
  legacyExecutionBundlePath,
  legacyExecutionCandidateBundlePath,
  legacyExecutionReportPath,
  injectedLegacyBundlePath,
  injectedLegacyManifestPath,
  injectedLegacyCandidateBundlePath,
  injectedLegacyReportPath,
  invalidPortCandidatePath,
  occupiedPortBundlePath,
  occupiedPortManifestPath,
  occupiedPortReportPath,
  occupiedPortControllerPath,
  nodeCaptureManifestPath,
  nodeCaptureLegacyBundlePath,
  nodeCaptureLegacyReportPath,
  nodeCaptureCandidateBundlePath,
  nodeCaptureCandidateReportPath,
  invalidSchema3ManifestPath,
  invalidSchema3CandidateBundlePath,
  concurrentNodeManifestPath,
  browserCaptureManifestPath,
  browserCaptureLegacyBundlePath,
  browserCaptureLegacyReportPath,
  browserCaptureControllerPath,
  browserCaptureHtmlPath,
  forcedBrowserBundlePath,
  forcedBrowserManifestPath,
  forcedBrowserControllerPath,
  forcedBrowserHtmlPath,
  splitBaselineBundlePath,
  splitCandidateBundlePath,
  splitControllerPath,
  path.join(targetDirectory, `${splitCandidateLabel}-browser.html`),
  httpBaselineBundlePath,
  httpBaselineManifestPath,
  httpCandidateBundlePath,
  httpControllerPath,
  path.join(targetDirectory, `${httpCandidateLabel}-browser.html`),
  httpReportPath,
  ...Object.values(sameBytesPaths),
  sameBytesBaselineControllerPath,
  sameBytesBaselineHtmlPath,
  ...Object.values(concurrentAPaths),
  ...Object.values(concurrentBPaths),
  ...Object.values(strictSuccessPaths),
  ...Object.values(nonStrictPaths),
];

function sha256(filePath: string): string {
  return createHash("sha256").update(readFileSync(filePath)).digest("hex");
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

function stateIdsChecksum(stateIds: readonly string[]): number {
  let checksum = 2_166_136_261;
  for (const stateId of stateIds) {
    checksum = mixChecksum(checksum, stringChecksum(stateId));
  }
  return checksum;
}

function syntheticBrowserImplementation(
  environment: { runtime: string; userAgent: string },
  stateCount: number,
  deadlineStateIds: readonly string[],
  fixedNodes = 50_000,
) {
  return {
    contractVersion: 2,
    generatedAt: new Date(0).toISOString(),
    environment,
    options: { samples: 1, smoke: true },
    stateCount,
    timed: timedMetricNames.map((name, index) => ({
      name,
      operations: 1,
      checksum: 100 + index,
      timedSinkChecksum: 200 + index,
      samples: [1],
      median: 1,
      p95: 1,
      microsecondsPerOperation: 1_000,
    })),
    packed: {
      name: "automove.pro.fixedWork",
      nodes: fixedNodes,
      nodesPerMillisecond: fixedNodes,
      depthChecksum: 300,
      moveChecksum: 301,
      configuration: {
        maxDepth: 40,
        maxNodes: 25_000,
        includesPositionLoad: false,
      },
      samples: [1],
      median: 1,
      p95: 1,
    },
    packedDeadline: {
      name: "automove.pro.deadline",
      budgetMs: 20,
      nodes: 40_000,
      nodesPerMillisecond: 40_000 / (deadlineStateIds.length * 20),
      depthChecksum: 400,
      moveChecksum: 401,
      configuration: {
        maxDepth: 40,
        maxNodes: Number.MAX_SAFE_INTEGER,
        includesPositionLoad: false,
      },
      stateIds: deadlineStateIds,
      stateChecksum: stateIdsChecksum(deadlineStateIds),
      samples: deadlineStateIds.map(() => 20),
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

function legacyNodeBundleSource(): string {
  return `
const timedMetricNames = ${JSON.stringify(timedMetricNames)};

function mixChecksum(checksum, value) {
  return Math.imul(checksum ^ value, 16777619) >>> 0;
}

function stringChecksum(value) {
  let checksum = 2166136261;
  for (let index = 0; index < value.length; index += 1) {
    checksum = mixChecksum(checksum, value.charCodeAt(index));
  }
  return checksum;
}

function stateIdsChecksum(stateIds) {
  let checksum = 2166136261;
  for (const stateId of stateIds) {
    checksum = mixChecksum(checksum, stringChecksum(stateId));
  }
  return checksum;
}

export function runBenchmarkSuite(states, options) {
  const samples = Array.from({ length: options.samples }, () => 1);
  const stateIndices = options.deadlineStateIndices ?? [0];
  const stateIds = stateIndices.flatMap((index) =>
    states[index] === undefined ? [] : [states[index].id],
  );
  const deadlineSamples = stateIds.map(() => 20);
  return {
    generatedAt: new Date(0).toISOString(),
    environment: { runtime: process.version, userAgent: "node" },
    options,
    stateCount: states.length,
    timed: timedMetricNames.map((name, index) => ({
      name,
      operations: 1,
      checksum: 10 + index,
      samples,
      median: 1,
      p95: 1,
      microsecondsPerOperation: 1000,
    })),
    packed: {
      name: "automove.pro.fixedWork",
      nodes: 1,
      nodesPerMillisecond: 1,
      depthChecksum: 30,
      moveChecksum: 31,
      samples,
      median: 1,
      p95: 1,
    },
    packedDeadline: {
      name: "automove.pro.deadline",
      budgetMs: options.smoke ? 20 : 460,
      nodes: stateIds.length,
      nodesPerMillisecond:
        stateIds.length /
        deadlineSamples.reduce((sum, value) => sum + value, 0),
      depthChecksum: 40,
      moveChecksum: 41,
      stateIds,
      stateChecksum: stateIdsChecksum(stateIds),
      samples: deadlineSamples,
      median: deadlineSamples[0] ?? 0,
      p95: deadlineSamples[0] ?? 0,
    },
    memory: { heapUsed: 1024, rss: 2048 },
  };
}
`;
}

function writeBrowserBaseline(label: string, source: string): void {
  const bundlePath = path.join(targetDirectory, `${label}-browser-suite.js`);
  const manifestPath = path.join(
    targetDirectory,
    `${label}-browser-manifest.json`,
  );
  writeFileSync(bundlePath, source);
  writeFileSync(
    manifestPath,
    `${JSON.stringify(
      {
        schemaVersion: 2,
        platform: "browser",
        capturedAt: new Date(0).toISOString(),
        artifact: {
          label,
          bundleFile: path.basename(bundlePath),
          bundleSha256: sha256(bundlePath),
          corpusFile: "test-data/automove-decisions/v4/decisions.jsonl",
          corpusSha256: sha256(corpusPath),
          runnerFile: "scripts/run-benchmarks.mjs",
          runnerSha256: sha256(runnerPath),
          sourceRevision: "test",
          sourceDirty: true,
        },
      },
      null,
      2,
    )}\n`,
  );
}

function runBenchmark(...arguments_: string[]) {
  return spawnSync(process.execPath, [runnerPath, ...arguments_], {
    cwd: repositoryRoot,
    encoding: "utf8",
    maxBuffer: 10 * 1024 * 1024,
  });
}

async function availablePort(): Promise<number> {
  const server = createServer();
  server.listen(0, "127.0.0.1");
  await once(server, "listening");
  const address = server.address();
  if (address === null || typeof address === "string") {
    throw new Error("test server did not receive a TCP port");
  }
  await new Promise<void>((resolve, reject) => {
    server.close((error) => {
      if (error === undefined) resolve();
      else reject(error);
    });
  });
  return address.port;
}

async function startBrowserBenchmark(
  port: number,
  baseline = httpBaselineLabel,
  candidate = httpCandidateLabel,
  requireParity = true,
) {
  const parityArguments = requireParity ? ["--require-parity"] : [];
  const child = spawn(
    process.execPath,
    [
      runnerPath,
      "--browser",
      `--baseline-label=${baseline}`,
      `--label=${candidate}`,
      `--port=${port}`,
      "--samples=1",
      "--batches=1",
      "--smoke",
      ...parityArguments,
    ],
    {
      cwd: repositoryRoot,
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
  child.stdout.setEncoding("utf8");
  child.stderr.setEncoding("utf8");
  let stderr = "";
  child.stderr.on("data", (chunk: string) => {
    stderr += chunk;
  });
  await new Promise<void>((resolve, reject) => {
    const timeout = setTimeout(() => {
      child.kill("SIGTERM");
      reject(new Error(`browser benchmark server timed out: ${stderr}`));
    }, 10_000);
    child.stdout.on("data", (chunk: string) => {
      if (!chunk.includes(`http://127.0.0.1:${port}/`)) return;
      clearTimeout(timeout);
      resolve();
    });
    child.once("exit", (code) => {
      clearTimeout(timeout);
      reject(
        new Error(
          `browser benchmark server exited before listening (${code}): ${stderr}`,
        ),
      );
    });
  });
  return { child, stderr: () => stderr };
}

async function startBrowserBaseline(
  port: number,
  baseline: string,
  force = false,
) {
  const forceArguments = force ? ["--force"] : [];
  const child = spawn(
    process.execPath,
    [
      runnerPath,
      "--browser",
      "--baseline",
      `--baseline-label=${baseline}`,
      `--label=${baseline}`,
      `--port=${port}`,
      "--samples=1",
      "--smoke",
      ...forceArguments,
    ],
    {
      cwd: repositoryRoot,
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
  child.stdout.setEncoding("utf8");
  child.stderr.setEncoding("utf8");
  let stderr = "";
  child.stderr.on("data", (chunk: string) => {
    stderr += chunk;
  });
  await new Promise<void>((resolve, reject) => {
    const timeout = setTimeout(() => {
      child.kill("SIGTERM");
      reject(new Error(`browser baseline server timed out: ${stderr}`));
    }, 10_000);
    child.stdout.on("data", (chunk: string) => {
      if (!chunk.includes(`http://127.0.0.1:${port}/`)) return;
      clearTimeout(timeout);
      resolve();
    });
    child.once("exit", (code) => {
      clearTimeout(timeout);
      reject(
        new Error(
          `browser baseline server exited before listening (${code}): ${stderr}`,
        ),
      );
    });
  });
  return { child, stderr: () => stderr };
}

async function stopChild(child: ReturnType<typeof spawn>): Promise<void> {
  if (child.exitCode !== null || child.signalCode !== null) return;
  child.kill("SIGTERM");
  await once(child, "exit");
}

async function abortHttpSubmission(
  port: number,
  report: unknown,
): Promise<void> {
  const body = Buffer.from(JSON.stringify(report));
  await new Promise<void>((resolve, reject) => {
    const socket = connect(port, "127.0.0.1");
    let connected = false;
    let settled = false;
    const settle = (error?: Error) => {
      if (settled) return;
      settled = true;
      socket.destroy();
      if (error === undefined) resolve();
      else reject(error);
    };
    socket.once("error", (error) => {
      if (connected) settle();
      else settle(error);
    });
    socket.once("connect", () => {
      connected = true;
      socket.write(
        [
          "POST /report HTTP/1.1",
          "Host: 127.0.0.1",
          "Content-Type: application/json",
          `Content-Length: ${body.length}`,
          "Connection: keep-alive",
          "",
          "",
        ].join("\r\n"),
      );
      socket.write(body, () => {
        setTimeout(() => settle(), 0);
      });
    });
  });
}

function removeNewBrowserSnapshots(previousFiles: ReadonlySet<string>): void {
  for (const fileName of readdirSync(targetDirectory)) {
    if (
      !previousFiles.has(fileName) &&
      (fileName.startsWith("module-browser-") ||
        fileName.startsWith("report-browser-baseline-"))
    ) {
      rmSync(path.join(targetDirectory, fileName), { force: true });
    }
  }
}

function removeNewNodeSnapshots(previousFiles: ReadonlySet<string>): void {
  for (const fileName of readdirSync(targetDirectory)) {
    if (
      !previousFiles.has(fileName) &&
      (fileName.startsWith("module-node-") ||
        fileName.startsWith("report-node-baseline-"))
    ) {
      rmSync(path.join(targetDirectory, fileName), { force: true });
    }
  }
}

function controllerArtifacts(source: string): {
  baseline: { label: string };
  candidate: { bundleSha256: string; label: string };
  run: { submissionId: string };
} {
  const artifactLine = source
    .split("\n")
    .find((line) => line.trimStart().startsWith("artifacts: "));
  if (artifactLine === undefined) {
    throw new Error("generated controller does not contain artifacts");
  }
  return JSON.parse(artifactLine.trim().slice("artifacts: ".length, -1));
}

async function comparisonSubmission(controllerPath: string) {
  const artifacts = controllerArtifacts(readFileSync(controllerPath, "utf8"));
  const candidateSnapshotPath = path.join(
    targetDirectory,
    `module-browser-candidate-${artifacts.candidate.bundleSha256}.js`,
  );
  const candidate = await import(
    `${pathToFileURL(candidateSnapshotPath).href}?test=${Date.now()}`
  );
  const corpusStates = readFileSync(corpusPath, "utf8")
    .trimEnd()
    .split("\n")
    .map((line) => JSON.parse(line));
  const environment = { runtime: "browser", userAgent: "runner-test" };
  const expectedContract = {
    stateCount: corpusStates.length,
    deadlineStateIds: [corpusStates[0].id],
  };
  const implementation = syntheticBrowserImplementation(
    environment,
    corpusStates.length,
    expectedContract.deadlineStateIds,
  );
  return {
    contractVersion: 2,
    generatedAt: new Date(0).toISOString(),
    environment,
    options: {
      samples: 1,
      batches: 1,
      batchSamples: [1],
      smoke: true,
      warmupRuns: 1,
      deadlineStateSchedule: [expectedContract.deadlineStateIds],
    },
    stateCount: corpusStates.length,
    executionOrder: ["baseline", "candidate"],
    metricParity: candidate.benchmarkMetricParity(
      implementation,
      implementation,
      expectedContract,
    ),
    implementations: {
      baseline: implementation,
      candidate: implementation,
    },
    artifacts,
  };
}

async function fetchUncachedText(url: string): Promise<string> {
  const response = await fetch(url);
  expect(response.status).toBe(200);
  expect(response.headers.get("cache-control")).toBe("no-store");
  return response.text();
}

beforeAll(() => {
  mkdirSync(targetDirectory, { recursive: true });
});

afterEach(() => {
  for (const filePath of testFiles) rmSync(filePath, { force: true });
});

describe("benchmark runner artifact safety", () => {
  it("rejects Node build-only mode before creating artifacts", () => {
    const previousFiles = new Set(readdirSync(targetDirectory));

    const result = runBenchmark(
      "--node",
      "--build-only",
      `--label=${nodeBuildOnlyLabel}`,
      "--smoke",
    );

    expect(result.status).not.toBe(0);
    expect(result.stderr).toContain(
      "--build-only is only supported with --browser",
    );
    expect(new Set(readdirSync(targetDirectory))).toEqual(previousFiles);
  });

  it("publishes and reloads complete Node baselines through schema 3", () => {
    const previousFiles = new Set(readdirSync(targetDirectory));
    try {
      const capture = runBenchmark(
        "--node",
        "--baseline",
        `--baseline-label=${nodeCaptureLabel}`,
        `--label=${nodeCaptureLabel}`,
        "--samples=1",
        "--smoke",
      );

      expect(capture.status).toBe(0);
      const output = JSON.parse(capture.stdout);
      const manifest = JSON.parse(
        readFileSync(nodeCaptureManifestPath, "utf8"),
      );
      expect(manifest.schemaVersion).toBe(3);
      expect(manifest.artifact.bundleFile).toBe(
        path.basename(output.baselineBundle),
      );
      expect(manifest.artifact.bundleSha256).toBe(
        sha256(output.baselineBundle),
      );
      expect(manifest.artifact.reportFile).toBe(
        path.basename(output.reportPath),
      );
      expect(manifest.artifact.reportSha256).toBe(sha256(output.reportPath));
      expect(existsSync(nodeCaptureLegacyBundlePath)).toBe(false);
      expect(existsSync(nodeCaptureLegacyReportPath)).toBe(false);

      const comparison = runBenchmark(
        "--node",
        `--baseline-label=${nodeCaptureLabel}`,
        `--label=${nodeCaptureCandidateLabel}`,
        "--samples=1",
        "--batches=1",
        "--smoke",
        "--require-parity",
      );

      expect(comparison.status).toBe(0);
      const report = JSON.parse(
        readFileSync(nodeCaptureCandidateReportPath, "utf8"),
      );
      expect(report.metricParity.complete).toBe(true);
      expect(report.artifacts.baseline.bundleFile).toBe(
        manifest.artifact.bundleFile,
      );
      expect(report.artifacts.baseline.reportFile).toBe(
        manifest.artifact.reportFile,
      );

      writeFileSync(output.reportPath, "tampered report\n");
      const rejected = runBenchmark(
        "--node",
        `--baseline-label=${nodeCaptureLabel}`,
        `--label=${nodeCaptureCandidateLabel}`,
        "--samples=1",
        "--batches=1",
        "--smoke",
        "--require-parity",
      );
      expect(rejected.status).not.toBe(0);
      expect(rejected.stderr).toContain(
        "baseline report digest does not match",
      );
    } finally {
      removeNewNodeSnapshots(previousFiles);
    }
  }, 30_000);

  it("rejects traversal in a schema-3 artifact path before building", () => {
    writeFileSync(
      invalidSchema3ManifestPath,
      `${JSON.stringify(
        {
          schemaVersion: 3,
          platform: "node",
          capturedAt: new Date(0).toISOString(),
          artifact: {
            label: invalidSchema3BaselineLabel,
            bundleFile: "../outside.mjs",
            bundleSha256: "0".repeat(64),
            corpusFile: "test-data/automove-decisions/v4/decisions.jsonl",
            corpusSha256: sha256(corpusPath),
            runnerFile: "scripts/run-benchmarks.mjs",
            runnerSha256: sha256(runnerPath),
            sourceRevision: "test",
            sourceDirty: true,
          },
        },
        null,
        2,
      )}\n`,
    );

    const result = runBenchmark(
      "--node",
      `--baseline-label=${invalidSchema3BaselineLabel}`,
      `--label=${invalidSchema3CandidateLabel}`,
      "--smoke",
      "--require-parity",
    );

    expect(result.status).not.toBe(0);
    expect(result.stderr).toContain("invalid baseline manifest");
    expect(existsSync(invalidSchema3CandidateBundlePath)).toBe(false);
  });

  it("allows exactly one concurrent no-force baseline publication", async () => {
    const previousFiles = new Set(readdirSync(targetDirectory));
    const capture = () => {
      const child = spawn(
        process.execPath,
        [
          runnerPath,
          "--node",
          "--baseline",
          `--baseline-label=${concurrentNodeBaselineLabel}`,
          `--label=${concurrentNodeBaselineLabel}`,
          "--samples=1",
          "--smoke",
        ],
        { cwd: repositoryRoot, stdio: ["ignore", "pipe", "pipe"] },
      );
      child.stdout.resume();
      child.stderr.setEncoding("utf8");
      let stderr = "";
      child.stderr.on("data", (chunk: string) => {
        stderr += chunk;
      });
      return new Promise<{ code: number | null; stderr: string }>((resolve) => {
        child.once("exit", (code) => resolve({ code, stderr }));
      });
    };

    try {
      const results = await Promise.all([capture(), capture()]);
      expect(results.map(({ code }) => code).sort()).toEqual([0, 1]);
      const failed = results.find(({ code }) => code !== 0);
      expect(failed?.stderr).toContain("baseline artifact");
      const manifest = JSON.parse(
        readFileSync(concurrentNodeManifestPath, "utf8"),
      );
      expect(manifest.schemaVersion).toBe(3);
      expect(manifest.artifact.reportFile).toMatch(
        /^report-node-baseline-[a-f0-9]{64}\.json$/u,
      );
    } finally {
      removeNewNodeSnapshots(previousFiles);
    }
  }, 30_000);

  it("publishes a browser baseline only after a valid report and exits", async () => {
    const previousFiles = new Set(readdirSync(targetDirectory));
    const port = await availablePort();
    let running: Awaited<ReturnType<typeof startBrowserBaseline>> | undefined;
    try {
      running = await startBrowserBaseline(port, browserCaptureLabel);
      const artifacts = controllerArtifacts(
        readFileSync(browserCaptureControllerPath, "utf8"),
      );
      const corpusStates = readFileSync(corpusPath, "utf8")
        .trimEnd()
        .split("\n")
        .map((line) => JSON.parse(line));
      const environment = { runtime: "browser", userAgent: "runner-test" };
      const report = {
        ...syntheticBrowserImplementation(environment, corpusStates.length, [
          corpusStates[0].id,
        ]),
        artifacts,
      };
      const endpoint = `http://127.0.0.1:${port}/report`;

      expect(existsSync(browserCaptureManifestPath)).toBe(false);
      const rejected = await fetch(endpoint, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          ...report,
          options: { samples: 2, smoke: true },
        }),
      });
      expect(rejected.status).toBe(400);
      expect(existsSync(browserCaptureManifestPath)).toBe(false);

      writeFileSync(browserCaptureManifestPath, "concurrent generation\n");
      const conflicted = await fetch(endpoint, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(report),
      });
      expect(conflicted.status).toBe(409);
      expect(running.child.exitCode).toBeNull();
      rmSync(browserCaptureManifestPath);

      const accepted = await fetch(endpoint, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(report),
      });
      expect(accepted.status).toBe(204);
      if (running.child.exitCode === null) await once(running.child, "exit");
      expect(running.child.exitCode).toBe(0);

      const manifest = JSON.parse(
        readFileSync(browserCaptureManifestPath, "utf8"),
      );
      const bundlePath = path.join(
        targetDirectory,
        manifest.artifact.bundleFile,
      );
      const reportPath = path.join(
        targetDirectory,
        manifest.artifact.reportFile,
      );
      expect(manifest.schemaVersion).toBe(3);
      expect(manifest.artifact.bundleSha256).toBe(sha256(bundlePath));
      expect(manifest.artifact.reportSha256).toBe(sha256(reportPath));
      expect(JSON.parse(readFileSync(reportPath, "utf8"))).toEqual(report);
      expect(existsSync(browserCaptureLegacyBundlePath)).toBe(false);
      expect(existsSync(browserCaptureLegacyReportPath)).toBe(false);
    } finally {
      if (running !== undefined) await stopChild(running.child);
      removeNewBrowserSnapshots(previousFiles);
    }
  }, 20_000);

  it("preserves the previous generation when a forced capture is aborted", async () => {
    const previousFiles = new Set(readdirSync(targetDirectory));
    const baselineSource =
      "export const packedDeadline = true;\nexport function runBenchmarkSuite() { return {}; }\n";
    writeBrowserBaseline(forcedBrowserCaptureLabel, baselineSource);
    const previousManifest = readFileSync(forcedBrowserManifestPath);
    const port = await availablePort();
    let running: Awaited<ReturnType<typeof startBrowserBaseline>> | undefined;
    try {
      running = await startBrowserBaseline(
        port,
        forcedBrowserCaptureLabel,
        true,
      );
      expect(readFileSync(forcedBrowserManifestPath)).toEqual(previousManifest);

      const rejected = await fetch(`http://127.0.0.1:${port}/report`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: "{}",
      });
      expect(rejected.status).toBe(400);
      await stopChild(running.child);
      expect(readFileSync(forcedBrowserManifestPath)).toEqual(previousManifest);
    } finally {
      if (running !== undefined) await stopChild(running.child);
      removeNewBrowserSnapshots(previousFiles);
    }
  }, 20_000);

  it("rejects a baseline and candidate bundle path collision even with force", () => {
    const sentinel = Buffer.from("existing baseline bundle\n");
    writeFileSync(bundleCollisionPath, sentinel);

    const result = runBenchmark(
      "--node",
      `--baseline-label=${bundleCollisionBaselineLabel}`,
      `--label=${bundleCollisionLabel}`,
      "--smoke",
      "--force",
    );

    expect(result.status).not.toBe(0);
    expect(result.stderr).toContain("benchmark artifact path collision");
    expect(readFileSync(bundleCollisionPath)).toEqual(sentinel);
  });

  it("rejects a baseline and candidate report path collision even with force", () => {
    const sentinel = Buffer.from("existing baseline report\n");
    writeFileSync(reportCollisionPath, sentinel);

    const result = runBenchmark(
      "--node",
      `--baseline-label=${reportCollisionLabel}`,
      `--label=${reportCollisionLabel}`,
      "--smoke",
      "--force",
    );

    expect(result.status).not.toBe(0);
    expect(result.stderr).toContain("benchmark artifact path collision");
    expect(readFileSync(reportCollisionPath)).toEqual(sentinel);
  });

  it("rejects a changed baseline before adaptation, execution, or candidate build", () => {
    writeFileSync(
      digestBundlePath,
      `import { writeFileSync } from "node:fs";\nwriteFileSync(${JSON.stringify(digestMarkerPath)}, "executed");\nexport function runBenchmarkSuite() { throw new Error("baseline executed"); }\n`,
    );
    writeFileSync(
      digestManifestPath,
      `${JSON.stringify(
        {
          schemaVersion: 2,
          platform: "node",
          capturedAt: new Date(0).toISOString(),
          artifact: {
            label: digestBaselineLabel,
            bundleFile: path.basename(digestBundlePath),
            bundleSha256: "0".repeat(64),
            corpusFile: "test-data/automove-decisions/v4/decisions.jsonl",
            corpusSha256: sha256(corpusPath),
            runnerFile: "scripts/run-benchmarks.mjs",
            runnerSha256: sha256(runnerPath),
            sourceRevision: "test",
            sourceDirty: true,
          },
        },
        null,
        2,
      )}\n`,
    );

    const result = runBenchmark(
      "--node",
      `--baseline-label=${digestBaselineLabel}`,
      `--label=${digestCandidateLabel}`,
      "--smoke",
    );

    expect(result.status).not.toBe(0);
    expect(result.stderr).toContain("baseline bundle digest does not match");
    expect(existsSync(digestMarkerPath)).toBe(false);
    expect(existsSync(digestAdapterPath)).toBe(false);
    expect(existsSync(digestCandidateBundlePath)).toBe(false);
    expect(existsSync(digestCandidateReportPath)).toBe(false);
  });

  it("rejects a legacy manifest in strict mode before executing either implementation", () => {
    writeFileSync(
      legacyBundlePath,
      `import { writeFileSync } from "node:fs";\nwriteFileSync(${JSON.stringify(legacyMarkerPath)}, "executed");\nexport function runBenchmarkSuite() { throw new Error("baseline executed"); }\n`,
    );
    writeFileSync(
      legacyManifestPath,
      `${JSON.stringify(
        {
          schemaVersion: 1,
          platform: "node",
          capturedAt: new Date(0).toISOString(),
          artifact: {
            label: legacyBaselineLabel,
            bundleFile: path.basename(legacyBundlePath),
            bundleSha256: sha256(legacyBundlePath),
            sourceRevision: "test",
            sourceDirty: true,
          },
        },
        null,
        2,
      )}\n`,
    );

    const result = runBenchmark(
      "--node",
      `--baseline-label=${legacyBaselineLabel}`,
      `--label=${legacyCandidateLabel}`,
      "--smoke",
      "--require-parity",
    );

    expect(result.status).not.toBe(0);
    expect(result.stderr).toContain(
      "strict parity requires a schema-2 or schema-3 baseline manifest",
    );
    expect(existsSync(legacyMarkerPath)).toBe(false);
    expect(existsSync(legacyAdapterPath)).toBe(false);
    expect(existsSync(legacyCandidateBundlePath)).toBe(false);
    expect(existsSync(legacyCandidateReportPath)).toBe(false);
  });

  it("runs an explicitly allowed legacy Node baseline in non-strict mode", () => {
    const previousFiles = new Set(readdirSync(targetDirectory));
    writeFileSync(legacyExecutionBundlePath, legacyNodeBundleSource());
    try {
      const result = runBenchmark(
        "--node",
        `--baseline-label=${legacyExecutionBaselineLabel}`,
        `--label=${legacyExecutionCandidateLabel}`,
        "--samples=1",
        "--batches=1",
        "--smoke",
      );

      expect(result.status).toBe(0);
      expect(result.stderr).toContain("baseline manifest is missing");
      expect(result.stderr).toContain("benchmark comparison is partial");
      const report = JSON.parse(
        readFileSync(legacyExecutionReportPath, "utf8"),
      );
      expect(report.metricParity.complete).toBe(false);
      expect(report.metricParity.contractVersionMatch).toBe(false);
      expect(report.options).not.toHaveProperty("allowLegacyBaseline");
      expect(report.implementations.baseline.contractVersion).toBeUndefined();
      expect(
        report.implementations.baseline.packed.configuration,
      ).toBeUndefined();
      expect(
        report.implementations.baseline.packedDeadline.configuration,
      ).toBeUndefined();
      expect(report.implementations.baseline.packedDeadline.stateIds).toEqual(
        report.options.deadlineStateSchedule.flat(),
      );
    } finally {
      removeNewNodeSnapshots(previousFiles);
    }
  }, 30_000);

  it("does not trust legacy permission injected into a schema-2 manifest", () => {
    const previousFiles = new Set(readdirSync(targetDirectory));
    const source = legacyNodeBundleSource();
    writeFileSync(injectedLegacyBundlePath, source);
    writeFileSync(
      injectedLegacyManifestPath,
      `${JSON.stringify(
        {
          schemaVersion: 2,
          platform: "node",
          capturedAt: new Date(0).toISOString(),
          artifact: {
            label: injectedLegacyBaselineLabel,
            bundleFile: path.basename(injectedLegacyBundlePath),
            bundleSha256: sha256(injectedLegacyBundlePath),
            corpusFile: "test-data/automove-decisions/v4/decisions.jsonl",
            corpusSha256: sha256(corpusPath),
            runnerFile: "scripts/run-benchmarks.mjs",
            runnerSha256: sha256(runnerPath),
            sourceRevision: "test",
            sourceDirty: true,
            legacy: true,
          },
        },
        null,
        2,
      )}\n`,
    );

    try {
      const result = runBenchmark(
        "--node",
        `--baseline-label=${injectedLegacyBaselineLabel}`,
        `--label=${injectedLegacyCandidateLabel}`,
        "--samples=1",
        "--batches=1",
        "--smoke",
      );

      expect(result.status).not.toBe(0);
      expect(result.stderr).toContain("unsupported benchmark contract version");
      expect(readFileSync(injectedLegacyBundlePath, "utf8")).toBe(source);
      expect(existsSync(injectedLegacyCandidateBundlePath)).toBe(true);
      expect(existsSync(injectedLegacyReportPath)).toBe(false);
    } finally {
      removeNewNodeSnapshots(previousFiles);
    }
  }, 30_000);

  it("rejects an invalid browser port before writing benchmark artifacts", () => {
    const result = runBenchmark(
      "--browser",
      `--baseline-label=${httpBaselineLabel}`,
      `--label=${invalidPortLabel}`,
      "--port=0",
      "--smoke",
    );

    expect(result.status).not.toBe(0);
    expect(result.stderr).toContain(
      "--port must be an integer from 1 through 65535",
    );
    expect(existsSync(invalidPortCandidatePath)).toBe(false);
  });

  it("reserves an available browser port before writing baseline artifacts", async () => {
    const occupied = createServer();
    occupied.listen(0, "127.0.0.1");
    await once(occupied, "listening");
    const address = occupied.address();
    if (address === null || typeof address === "string") {
      throw new Error("occupied test server did not receive a TCP port");
    }
    try {
      const result = runBenchmark(
        "--browser",
        "--baseline",
        `--baseline-label=${occupiedPortLabel}`,
        `--label=${occupiedPortLabel}`,
        `--port=${address.port}`,
        "--smoke",
      );

      expect(result.status).not.toBe(0);
      expect(result.stderr).toContain("EADDRINUSE");
      expect(existsSync(occupiedPortBundlePath)).toBe(false);
      expect(existsSync(occupiedPortManifestPath)).toBe(false);
      expect(existsSync(occupiedPortReportPath)).toBe(false);
      expect(existsSync(occupiedPortControllerPath)).toBe(false);
      expect(
        existsSync(
          path.join(targetDirectory, `${occupiedPortLabel}-browser.html`),
        ),
      ).toBe(false);
    } finally {
      await new Promise<void>((resolve, reject) => {
        occupied.close((error) => {
          if (error === undefined) resolve();
          else reject(error);
        });
      });
    }
  });

  it("replaces a split legacy bundle's trailing export block", async () => {
    const previousFiles = new Set(readdirSync(targetDirectory));
    const source =
      "class FastSearcher {}\nclass MonsGame {}\nfunction runBenchmarkSuite() {}\nfunction tryLoadPosition() {}\n\nexport { FastSearcher, MonsGame, runBenchmarkSuite, tryLoadPosition };\n";
    writeFileSync(splitBaselineBundlePath, source);

    try {
      const result = runBenchmark(
        "--browser",
        `--baseline-label=${splitBaselineLabel}`,
        `--label=${splitCandidateLabel}`,
        "--build-only",
        "--smoke",
      );

      expect(result.status).toBe(0);
      const output = JSON.parse(result.stdout);
      expect(output).not.toHaveProperty("servedFiles");
      const derivedSource = readFileSync(output.baselineBundle, "utf8");
      expect(readFileSync(output.controller, "utf8")).toContain(
        '"allowLegacyBaseline":true',
      );
      expect(readFileSync(splitBaselineBundlePath, "utf8")).toBe(source);
      expect(derivedSource.match(/export\s*\{/gu)).toHaveLength(1);
      expect(derivedSource.trimEnd()).toMatch(
        /export \{ FastSearcher, MonsGame, runBenchmarkSuite, tryLoadPosition \};$/u,
      );
      const derived = await import(
        `${pathToFileURL(output.baselineBundle).href}?test=${Date.now()}`
      );
      expect(Object.keys(derived).sort()).toEqual([
        "FastSearcher",
        "MonsGame",
        "runBenchmarkSuite",
        "tryLoadPosition",
      ]);
    } finally {
      removeNewBrowserSnapshots(previousFiles);
    }
  });

  it("uses distinct role-addressed snapshots for identical browser bundles", () => {
    const previousFiles = new Set(readdirSync(targetDirectory));
    try {
      const capture = runBenchmark(
        "--browser",
        "--baseline",
        `--baseline-label=${sameBytesBaselineLabel}`,
        `--label=${sameBytesBaselineLabel}`,
        "--build-only",
        "--smoke",
      );
      expect(capture.status).toBe(0);
      const captureOutput = JSON.parse(capture.stdout);
      const manifest = JSON.parse(
        readFileSync(sameBytesPaths.baselineManifest, "utf8"),
      );
      expect(manifest.schemaVersion).toBe(3);
      expect(manifest.artifact.bundleFile).toBe(
        path.basename(captureOutput.baselineBundle),
      );
      expect(manifest.artifact).not.toHaveProperty("reportFile");

      const comparison = runBenchmark(
        "--browser",
        `--baseline-label=${sameBytesBaselineLabel}`,
        `--label=${sameBytesCandidateLabel}`,
        "--build-only",
        "--smoke",
      );
      expect(comparison.status).toBe(0);
      const output = JSON.parse(comparison.stdout);

      expect(output).not.toHaveProperty("servedFiles");
      expect(output.baselineBundle).not.toBe(output.candidateBundle);
      expect(path.basename(output.baselineBundle)).toMatch(
        /^module-browser-baseline-[a-f0-9]{64}\.js$/u,
      );
      expect(path.basename(output.candidateBundle)).toMatch(
        /^module-browser-candidate-[a-f0-9]{64}\.js$/u,
      );
      const controller = readFileSync(output.controller, "utf8");
      expect(controller).toContain('"allowLegacyBaseline":false');
      for (const modulePath of [
        output.baselineBundle,
        output.candidateBundle,
      ]) {
        const moduleName = path.basename(modulePath);
        expect(controller).toContain(`from "./${moduleName}"`);
      }
      expect(existsSync(sameBytesPaths.baselineBundle)).toBe(false);
      expect(readFileSync(captureOutput.baselineBundle)).toEqual(
        readFileSync(sameBytesPaths.candidateBundle),
      );
      expect(readFileSync(output.baselineBundle)).toEqual(
        readFileSync(output.candidateBundle),
      );
    } finally {
      removeNewBrowserSnapshots(previousFiles);
    }
  }, 20_000);

  it("exits after strict success and keeps non-strict comparisons available", async () => {
    const previousFiles = new Set(readdirSync(targetDirectory));
    const baselineSource =
      "export const packedDeadline = true;\nexport function runBenchmarkSuite() { return {}; }\n";
    writeBrowserBaseline(lifecycleBaselineLabel, baselineSource);
    let strict: Awaited<ReturnType<typeof startBrowserBenchmark>> | undefined;
    let abortedStrict:
      Awaited<ReturnType<typeof startBrowserBenchmark>> | undefined;
    let nonStrict:
      Awaited<ReturnType<typeof startBrowserBenchmark>> | undefined;
    try {
      const strictPort = await availablePort();
      strict = await startBrowserBenchmark(
        strictPort,
        lifecycleBaselineLabel,
        strictSuccessCandidateLabel,
      );
      const strictReport = await comparisonSubmission(
        strictSuccessPaths.candidateController,
      );
      expect(strictReport.metricParity.complete).toBe(true);
      const strictResponse = await fetch(
        `http://127.0.0.1:${strictPort}/report`,
        {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify(strictReport),
        },
      );
      expect(strictResponse.status).toBe(204);
      if (strict.child.exitCode === null) await once(strict.child, "exit");
      expect(strict.child.exitCode).toBe(0);

      const abortedStrictPort = await availablePort();
      abortedStrict = await startBrowserBenchmark(
        abortedStrictPort,
        lifecycleBaselineLabel,
        strictSuccessCandidateLabel,
      );
      const abortedStrictReport = await comparisonSubmission(
        strictSuccessPaths.candidateController,
      );
      await abortHttpSubmission(abortedStrictPort, abortedStrictReport);
      if (abortedStrict.child.exitCode === null) {
        await once(abortedStrict.child, "exit");
      }
      expect(abortedStrict.child.exitCode).toBe(0);
      expect(
        JSON.parse(readFileSync(strictSuccessPaths.candidateReport, "utf8")),
      ).toEqual(abortedStrictReport);

      const nonStrictPort = await availablePort();
      nonStrict = await startBrowserBenchmark(
        nonStrictPort,
        lifecycleBaselineLabel,
        nonStrictCandidateLabel,
        false,
      );
      const nonStrictReport = await comparisonSubmission(
        nonStrictPaths.candidateController,
      );
      const nonStrictResponse = await fetch(
        `http://127.0.0.1:${nonStrictPort}/report`,
        {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify(nonStrictReport),
        },
      );
      expect(nonStrictResponse.status).toBe(204);
      expect(nonStrict.child.exitCode).toBeNull();
      expect(
        await fetchUncachedText(`http://127.0.0.1:${nonStrictPort}/`),
      ).toContain("Mons rules benchmark");
      expect(
        JSON.parse(readFileSync(nonStrictPaths.candidateReport, "utf8")),
      ).toEqual(nonStrictReport);
    } finally {
      if (strict !== undefined) await stopChild(strict.child);
      if (abortedStrict !== undefined) await stopChild(abortedStrict.child);
      if (nonStrict !== undefined) await stopChild(nonStrict.child);
      removeNewBrowserSnapshots(previousFiles);
    }
  }, 30_000);

  it("keeps concurrent browser servers bound to their captured assets", async () => {
    const previousFiles = new Set(readdirSync(targetDirectory));
    const baselineSource =
      "export const packedDeadline = true;\nexport function runBenchmarkSuite() { return {}; }\n";
    writeBrowserBaseline(concurrentABaselineLabel, baselineSource);
    writeBrowserBaseline(concurrentBBaselineLabel, baselineSource);

    let runningA: Awaited<ReturnType<typeof startBrowserBenchmark>> | undefined;
    let runningB: Awaited<ReturnType<typeof startBrowserBenchmark>> | undefined;
    try {
      const portA = await availablePort();
      runningA = await startBrowserBenchmark(
        portA,
        concurrentABaselineLabel,
        concurrentACandidateLabel,
      );
      const rootA = `http://127.0.0.1:${portA}`;
      const htmlA = await fetchUncachedText(`${rootA}/?run=a`);
      const controllerRouteA = `/${path.basename(
        concurrentAPaths.candidateController,
      )}`;
      const controllerA = await fetchUncachedText(
        `${rootA}${controllerRouteA}?run=a`,
      );

      const portB = await availablePort();
      runningB = await startBrowserBenchmark(
        portB,
        concurrentBBaselineLabel,
        concurrentBCandidateLabel,
      );
      const rootB = `http://127.0.0.1:${portB}`;
      const htmlB = await fetchUncachedText(`${rootB}/?run=b`);
      const controllerRouteB = `/${path.basename(
        concurrentBPaths.candidateController,
      )}`;
      const controllerB = await fetchUncachedText(
        `${rootB}${controllerRouteB}?run=b`,
      );

      expect(await fetchUncachedText(`${rootA}/?after=b`)).toBe(htmlA);
      expect(await fetchUncachedText(`${rootA}/index.html?after=b`)).toBe(
        htmlA,
      );
      expect(
        await fetchUncachedText(`${rootA}${controllerRouteA}?after=b`),
      ).toBe(controllerA);
      expect(htmlA).toContain(controllerRouteA);
      expect(htmlA).not.toContain(controllerRouteB);
      expect(htmlB).toContain(controllerRouteB);
      expect(htmlB).not.toContain(controllerRouteA);

      const artifactsA = controllerArtifacts(controllerA);
      const artifactsB = controllerArtifacts(controllerB);
      expect(artifactsA.baseline.label).toBe(concurrentABaselineLabel);
      expect(artifactsA.candidate.label).toBe(concurrentACandidateLabel);
      expect(artifactsB.baseline.label).toBe(concurrentBBaselineLabel);
      expect(artifactsB.candidate.label).toBe(concurrentBCandidateLabel);
      expect(artifactsA.run.submissionId).not.toBe(artifactsB.run.submissionId);

      writeFileSync(concurrentAPaths.candidateHtml, "changed on disk\n");
      writeFileSync(concurrentAPaths.candidateController, "changed on disk\n");
      expect(await fetchUncachedText(`${rootA}/?after=disk`)).toBe(htmlA);
      expect(await fetchUncachedText(`${rootA}/index.html?after=disk`)).toBe(
        htmlA,
      );
      expect(
        await fetchUncachedText(`${rootA}${controllerRouteA}?after=disk`),
      ).toBe(controllerA);
    } finally {
      if (runningA !== undefined) await stopChild(runningA.child);
      if (runningB !== undefined) await stopChild(runningB.child);
      removeNewBrowserSnapshots(previousFiles);
    }
  }, 30_000);

  it("binds strict browser submissions to the served run and trusted parity", async () => {
    const previousFiles = new Set(readdirSync(targetDirectory));
    const baselineSource =
      "export const packedDeadline = true;\nexport function runBenchmarkSuite() { return {}; }\n";
    writeBrowserBaseline(httpBaselineLabel, baselineSource);

    const port = await availablePort();
    let running: Awaited<ReturnType<typeof startBrowserBenchmark>> | undefined;
    try {
      running = await startBrowserBenchmark(port);
      const controller = readFileSync(httpControllerPath, "utf8");
      const artifacts = controllerArtifacts(controller);
      const candidateSnapshotPath = path.join(
        targetDirectory,
        `module-browser-candidate-${artifacts.candidate.bundleSha256}.js`,
      );
      const candidate = await import(
        `${pathToFileURL(candidateSnapshotPath).href}?test=${Date.now()}`
      );
      const corpusStates = readFileSync(corpusPath, "utf8")
        .trimEnd()
        .split("\n")
        .map((line) => JSON.parse(line));
      const environment = { runtime: "browser", userAgent: "runner-test" };
      const expectedContract = {
        stateCount: corpusStates.length,
        deadlineStateIds: [corpusStates[0].id],
      };
      const comparisonReport = (
        baseline: unknown,
        candidateReport: unknown,
      ) => ({
        contractVersion: 2,
        generatedAt: new Date(0).toISOString(),
        environment,
        options: {
          samples: 1,
          batches: 1,
          batchSamples: [1],
          smoke: true,
          warmupRuns: 1,
          deadlineStateSchedule: [expectedContract.deadlineStateIds],
        },
        stateCount: corpusStates.length,
        executionOrder: ["baseline", "candidate"],
        metricParity: candidate.benchmarkMetricParity(
          baseline,
          candidateReport,
          expectedContract,
        ),
        implementations: {
          baseline,
          candidate: candidateReport,
        },
        artifacts,
      });
      const equalImplementation = syntheticBrowserImplementation(
        environment,
        corpusStates.length,
        expectedContract.deadlineStateIds,
      );
      const firstTimedMetric = equalImplementation.timed[0];
      if (firstTimedMetric === undefined) {
        throw new Error("synthetic implementation has no timed metrics");
      }
      const malformedTimed = {
        ...equalImplementation,
        timed: [
          { ...firstTimedMetric, samples: [] },
          ...equalImplementation.timed.slice(1),
        ],
      };
      const malformedSummary = {
        ...equalImplementation,
        timed: [
          { ...firstTimedMetric, median: firstTimedMetric.median + 1 },
          ...equalImplementation.timed.slice(1),
        ],
      };
      const malformedFixed = {
        ...equalImplementation,
        packed: { ...equalImplementation.packed, samples: [] },
      };
      const malformedDeadline = {
        ...equalImplementation,
        packedDeadline: {
          ...equalImplementation.packedDeadline,
          samples: [],
        },
      };
      const report = comparisonReport(
        syntheticBrowserImplementation(
          environment,
          corpusStates.length,
          expectedContract.deadlineStateIds,
        ),
        syntheticBrowserImplementation(
          environment,
          corpusStates.length,
          expectedContract.deadlineStateIds,
          50_001,
        ),
      );
      const endpoint = `http://127.0.0.1:${port}/report`;

      const unsupported = await fetch(endpoint, {
        method: "POST",
        body: "{}",
      });
      expect(unsupported.status).toBe(415);

      const oversized = await fetch(endpoint, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: "x".repeat(1024 * 1024 + 1),
      });
      expect(oversized.status).toBe(413);

      const wrongOptions = await fetch(endpoint, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          ...report,
          options: { ...report.options, samples: 2 },
        }),
      });
      expect(wrongOptions.status).toBe(400);

      const forgedParity = await fetch(endpoint, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ ...report, metricParity: { complete: true } }),
      });
      expect(forgedParity.status).toBe(400);

      for (const malformedReport of [
        comparisonReport(malformedTimed, malformedTimed),
        comparisonReport(malformedSummary, malformedSummary),
        comparisonReport(malformedFixed, malformedFixed),
        comparisonReport(malformedDeadline, malformedDeadline),
      ]) {
        const malformed = await fetch(endpoint, {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify(malformedReport),
        });
        expect(malformed.status).toBe(400);
        expect(existsSync(httpReportPath)).toBe(false);
      }

      expect(report.metricParity.complete).toBe(false);
      expect(report.metricParity.fixedWorkNodeCountMatch).toBe(false);

      const accepted = await fetch(endpoint, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(report),
      });
      expect(accepted.status).toBe(422);
      if (running.child.exitCode === null) await once(running.child, "exit");
      expect(running.child.exitCode).toBe(1);
      expect(JSON.parse(readFileSync(httpReportPath, "utf8"))).toEqual(report);
    } finally {
      if (running !== undefined) await stopChild(running.child);
      removeNewBrowserSnapshots(previousFiles);
    }
  }, 20_000);
});
