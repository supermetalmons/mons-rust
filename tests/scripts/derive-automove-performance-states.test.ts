import { spawnSync, type SpawnSyncReturns } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";

import { buildSync } from "esbuild";
import { afterAll, afterEach, beforeAll, describe, expect, it } from "vitest";

const repositoryRoot = path.resolve(import.meta.dirname, "../..");
const script = path.join(
  repositoryRoot,
  "scripts/derive-automove-performance-states.mjs",
);
const corpus = path.join(
  repositoryRoot,
  "test-data/complete-games/v1/complete-games.jsonl",
);
const expectedStateBankSha256 =
  "797f8634bdb3e9cc65c714620bbb1d78a5abfe01e2b29de8844937393d2da886";
const expectedVariants = [
  "AlternatingManaRows",
  "BentCenterManaRows",
  "CenterSpokeManaRows",
  "Classic",
  "CornerChainManaRows",
  "ForwardBridgeManaRows",
  "InnerWedgeManaRows",
  "OffsetArcManaRows",
  "OuterEdgeManaRows",
  "OuterWedgeManaRows",
  "SplitFlankManaRows",
  "SwappedManaRows",
];
const temporaryDirectories: string[] = [];
let bundleDirectory = "";
let bundle = "";

type DerivedState = {
  id: string;
  fen: string;
  variant: string;
  activeColor: "white" | "black";
  sourceLine: number;
  midpointTurnIndex: number;
  stateSha256: string;
};

beforeAll(() => {
  bundleDirectory = fs.mkdtempSync(
    path.join(os.tmpdir(), "mons-automove-derivation-bundle-"),
  );
  bundle = path.join(bundleDirectory, "mons-rules.mjs");
  buildSync({
    bundle: true,
    entryPoints: [path.join(repositoryRoot, "src/entrypoints/mons-rules.ts")],
    format: "esm",
    keepNames: true,
    legalComments: "none",
    minify: true,
    outfile: bundle,
    platform: "browser",
    target: "es2020",
  });
});

afterAll(() => {
  if (bundleDirectory !== "") {
    fs.rmSync(bundleDirectory, { force: true, recursive: true });
  }
});

afterEach(() => {
  for (const directory of temporaryDirectories.splice(0)) {
    fs.rmSync(directory, { force: true, recursive: true });
  }
});

describe("derive-automove-performance-states CLI", () => {
  it("derives the approved balanced midpoint bank deterministically", () => {
    const root = temporaryDirectory();
    const corpusBefore = fileSha256(corpus);
    const firstStates = path.join(root, "first", "states.jsonl");
    const firstManifest = path.join(root, "first", "manifest.json");
    const secondStates = path.join(root, "second", "states.jsonl");
    const secondManifest = path.join(root, "second", "manifest.json");

    const destinations: [string, string][] = [
      [firstStates, firstManifest],
      [secondStates, secondManifest],
    ];
    for (const [states, manifest] of destinations) {
      const result = run(["--bundle", bundle, "--out", states, "--manifest", manifest]);
      expect(result.status, result.stderr).toBe(0);
      expect(result.stdout).toContain("automove performance states: 64 states");
    }

    expect(fs.readFileSync(secondStates)).toEqual(fs.readFileSync(firstStates));
    expect(fs.readFileSync(secondManifest)).toEqual(fs.readFileSync(firstManifest));
    expect(fs.readdirSync(path.dirname(firstStates)).sort()).toEqual([
      "manifest.json",
      "states.jsonl",
    ]);
    expect(fileSha256(firstStates)).toBe(expectedStateBankSha256);
    expect(fileSha256(corpus)).toBe(corpusBefore);

    const states = readStateBank(firstStates);
    expect(states).toHaveLength(64);
    expect(new Set(states.map((state) => state.id)).size).toBe(64);
    expect(countColors(states)).toEqual({ white: 32, black: 32 });
    expect([...new Set(states.map((state) => state.variant))].sort()).toEqual(
      expectedVariants,
    );
    for (const [index, state] of states.entries()) {
      expect(state.stateSha256).toBe(sha256(state.fen));
      expect(state.id).toBe(
        `midpoint-${state.stateSha256}-${String(state.sourceLine).padStart(10, "0")}`,
      );
      expect(state.midpointTurnIndex).toBeGreaterThan(0);
      if (index > 0) {
        const previous = states[index - 1];
        if (previous === undefined) throw new Error("missing previous state");
        expect(compareSelectionKeys(previous, state)).toBeLessThanOrEqual(0);
      }
    }

    const manifest = readJson(firstManifest);
    expect(manifest).toMatchObject({
      schemaVersion: 1,
      kind: "automove-performance-state-bank",
      algorithm: {
        id: "complete-games-v1-midpoint-balanced-v1",
        midpointTurnIndex: "floor(turns.length / 2)",
        statesPerColor: 32,
      },
      source: {
        corpus: {
          path: "test-data/complete-games/v1/complete-games.jsonl",
          bytes: 2_273_026,
          sha256: "5bc194f15516a9c275807415910c95b2e62ce63df9e575ac93e1dd93013197eb",
          records: 1527,
        },
        bundle: { sha256: fileSha256(bundle) },
      },
      candidates: {
        eligible: 1527,
        excludedTerminal: 0,
        excludedTooShort: 0,
      },
      selection: {
        states: 64,
        colors: { white: 32, black: 32 },
        variants: expectedVariants,
      },
      output: {
        format: "jsonl",
        encoding: "UTF-8",
        trailingNewline: true,
        bytes: fs.statSync(firstStates).size,
        sha256: expectedStateBankSha256,
      },
    });
    const { sha256: algorithmSha256, ...algorithm } = manifest.algorithm;
    expect(algorithmSha256).toBe(sha256(JSON.stringify(algorithm)));
    expect(manifest.source.sha256).toBe(
      sha256(
        JSON.stringify({
          bundleSha256: manifest.source.bundle.sha256,
          corpusSha256: manifest.source.corpus.sha256,
        }),
      ),
    );
    const artifacts = pathToFileURL(
      path.join(repositoryRoot, "scripts/evidence/artifacts.mjs"),
    ).href;
    const validation = spawnSync(
      process.execPath,
      [
        "--input-type=module",
        "--eval",
        `import { readPerformanceStateBank } from ${JSON.stringify(artifacts)}; readPerformanceStateBank(${JSON.stringify(firstStates)}, ${JSON.stringify(firstManifest)});`,
      ],
      { cwd: repositoryRoot, encoding: "utf8" },
    );
    expect(validation.status, validation.stderr).toBe(0);
  }, 30_000);

  it("rejects protected and colliding destinations before derivation", () => {
    const protectedOutput = path.join(
      repositoryRoot,
      "test-data",
      `derived-performance-states-${process.pid}-${Date.now()}.jsonl`,
    );
    const protectedResult = run(["--bundle", bundle, "--out", protectedOutput]);
    expect(protectedResult.status).toBe(1);
    expect(protectedResult.stderr).toContain(
      "refusing to write an evidence report under test-data/",
    );
    expect(fs.existsSync(protectedOutput)).toBe(false);

    const root = temporaryDirectory();
    const destination = path.join(root, "same.json");
    const collidingResult = run([
      "--bundle",
      bundle,
      "--out",
      destination,
      "--manifest",
      destination,
    ]);
    expect(collidingResult.status).toBe(1);
    expect(collidingResult.stderr).toContain(
      "state-bank and manifest destinations must be different",
    );
    expect(fs.existsSync(destination)).toBe(false);
  });

  it("removes a staged state bank when manifest creation fails", () => {
    const root = temporaryDirectory();
    const states = path.join(root, "states.jsonl");
    const manifest = path.join(root, "manifest.json");
    const artifacts = pathToFileURL(
      path.join(repositoryRoot, "scripts/evidence/artifacts.mjs"),
    ).href;
    fs.writeFileSync(manifest, "occupied");
    const probe = [
      `import { createExclusiveEvidencePair } from ${JSON.stringify(artifacts)};`,
      `createExclusiveEvidencePair(${JSON.stringify(states)}, "state\\n", ${JSON.stringify(manifest)}, "manifest\\n");`,
    ].join("\n");
    const result = spawnSync(
      process.execPath,
      ["--input-type=module", "--eval", probe],
      { cwd: repositoryRoot, encoding: "utf8" },
    );

    expect(result.status).toBe(1);
    expect(result.stderr).toContain("refusing to overwrite existing evidence report");
    expect(fs.existsSync(states)).toBe(false);
    expect(fs.readFileSync(manifest, "utf8")).toBe("occupied");
  });

  it("does not expose a final state bank when interrupted during staging", () => {
    const root = temporaryDirectory();
    const states = path.join(root, "states.jsonl");
    const artifacts = pathToFileURL(
      path.join(repositoryRoot, "scripts/evidence/artifacts.mjs"),
    ).href;
    const probe = [
      `import { stageExclusiveEvidenceFile } from ${JSON.stringify(artifacts)};`,
      `stageExclusiveEvidenceFile(${JSON.stringify(states)}, "staged-state\\n");`,
      'process.kill(process.pid, "SIGKILL");',
    ].join("\n");

    const result = spawnSync(
      process.execPath,
      ["--input-type=module", "--eval", probe],
      { cwd: repositoryRoot, encoding: "utf8" },
    );

    expect(result.signal).toBe("SIGKILL");
    expect(fs.existsSync(states)).toBe(false);
    expect(fs.readdirSync(root).some((name) => name.startsWith(".states.jsonl."))).toBe(
      true,
    );
  });

  it("revalidates the state-bank destination after bundle execution", () => {
    const root = temporaryDirectory();
    const outputDirectory = path.join(root, "output");
    fs.mkdirSync(outputDirectory);
    const protectedDestination = path.join(
      repositoryRoot,
      "test-data/rules-regressions.jsonl.gz",
    );
    const protectedSha256 = fileSha256(protectedDestination);
    const fileName = path.basename(protectedDestination);
    const states = path.join(outputDirectory, fileName);
    const artifacts = pathToFileURL(
      path.join(repositoryRoot, "scripts/evidence/artifacts.mjs"),
    ).href;
    const probe = [
      'import { rmdirSync, symlinkSync } from "node:fs";',
      `import { createExclusiveEvidenceFile, preflightJsonReportDestination } from ${JSON.stringify(artifacts)};`,
      `const outputDirectory = ${JSON.stringify(outputDirectory)};`,
      `const destination = preflightJsonReportDestination(${JSON.stringify(states)});`,
      "rmdirSync(outputDirectory);",
      `symlinkSync(${JSON.stringify(path.join(repositoryRoot, "test-data"))}, outputDirectory, "dir");`,
      'createExclusiveEvidenceFile(destination, "must-not-be-written");',
    ].join("\n");

    const result = spawnSync(
      process.execPath,
      ["--input-type=module", "--eval", probe],
      { cwd: repositoryRoot, encoding: "utf8" },
    );

    expect(result.status).toBe(1);
    expect(result.stderr).toContain(
      "refusing to write an evidence report under test-data/",
    );
    expect(fileSha256(protectedDestination)).toBe(protectedSha256);
  });

  it("never removes a replacement while rolling back publication", () => {
    const root = temporaryDirectory();
    const states = path.join(root, "states.jsonl");
    const manifest = path.join(root, "manifest.json");
    const artifacts = pathToFileURL(
      path.join(repositoryRoot, "scripts/evidence/artifacts.mjs"),
    ).href;
    const probe = [
      'import { unlinkSync, writeFileSync } from "node:fs";',
      `import { createExclusiveEvidenceFile, removeEvidenceFileIfCreated } from ${JSON.stringify(artifacts)};`,
      `const states = ${JSON.stringify(states)};`,
      `const manifest = ${JSON.stringify(manifest)};`,
      'const created = createExclusiveEvidenceFile(states, "original");',
      "unlinkSync(states);",
      'writeFileSync(states, "replacement", { flag: "wx" });',
      'writeFileSync(manifest, "occupied", { flag: "wx" });',
      "try {",
      '  createExclusiveEvidenceFile(manifest, "manifest");',
      "} catch (error) {",
      "  try {",
      "    removeEvidenceFileIfCreated(created);",
      "  } catch (cleanupError) {",
      "    console.error(cleanupError.message);",
      "    process.exitCode = 1;",
      "  }",
      "}",
    ].join("\n");

    const result = spawnSync(
      process.execPath,
      ["--input-type=module", "--eval", probe],
      { cwd: repositoryRoot, encoding: "utf8" },
    );

    expect(result.status).toBe(1);
    expect(result.stderr).toContain("refusing to remove replaced evidence file");
    expect(fs.readFileSync(states, "utf8")).toBe("replacement");
    expect(fs.readFileSync(manifest, "utf8")).toBe("occupied");
  });
});

function temporaryDirectory(): string {
  const directory = fs.mkdtempSync(
    path.join(os.tmpdir(), "mons-automove-state-derivation-"),
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

function readStateBank(filePath: string): DerivedState[] {
  return fs
    .readFileSync(filePath, "utf8")
    .trimEnd()
    .split("\n")
    .map((line) => JSON.parse(line) as DerivedState);
}

function readJson(filePath: string) {
  return JSON.parse(fs.readFileSync(filePath, "utf8"));
}

function countColors(states: DerivedState[]): Record<string, number> {
  return Object.fromEntries(
    ["white", "black"].map((color) => [
      color,
      states.filter((state) => state.activeColor === color).length,
    ]),
  );
}

function compareSelectionKeys(left: DerivedState, right: DerivedState): number {
  if (left.stateSha256 !== right.stateSha256) {
    return left.stateSha256 < right.stateSha256 ? -1 : 1;
  }
  return left.sourceLine - right.sourceLine;
}

function fileSha256(filePath: string): string {
  return sha256(fs.readFileSync(filePath));
}

function sha256(value: NodeJS.ArrayBufferView | string): string {
  return createHash("sha256").update(value).digest("hex");
}
