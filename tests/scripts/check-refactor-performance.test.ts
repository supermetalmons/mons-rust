import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";

import { afterEach, describe, expect, it } from "vitest";

const repositoryRoot = path.resolve(import.meta.dirname, "../..");
const refactorPerformanceModule = pathToFileURL(
  path.join(repositoryRoot, "scripts/evidence/refactor-performance.mjs"),
).href;
const {
  PRISTINE_BASELINE_SHA256,
  assertCurrentCandidateUnchanged,
  assertDerivedMidpointStateBank,
  assertRefactorPerformanceContract,
} = (await import(refactorPerformanceModule)) as RefactorPerformanceModule;
const optionsModule = pathToFileURL(
  path.join(repositoryRoot, "scripts/evidence/options.mjs"),
).href;
const { readPublicBundleArtifact } = (await import(optionsModule)) as OptionsModule;
const v6StateBank = path.join(
  repositoryRoot,
  "test-data/automove-decisions/v6/decisions.jsonl",
);
const packageManifest = JSON.parse(
  fs.readFileSync(path.join(repositoryRoot, "package.json"), "utf8"),
) as { readonly scripts?: Record<string, string> };
const temporaryDirectories: string[] = [];

afterEach(() => {
  for (const directory of temporaryDirectories.splice(0)) {
    fs.rmSync(directory, { force: true, recursive: true });
  }
});

describe("refactor performance acceptance contract", () => {
  it("rebuilds and binds the npm acceptance command to the current bundle", () => {
    expect(packageManifest.scripts?.["check:refactor-performance"]).toBe(
      "npm run build && node ./scripts/check-refactor-performance.mjs --candidate ./dist/mons-rules.js",
    );
  });

  it("requires the pristine baseline, distinct shared candidate, exact modes and banks", () => {
    const { midpoint, v6 } = finalReports();
    expect(() =>
      assertRefactorPerformanceContract(v6, midpoint, "b".repeat(64)),
    ).not.toThrow();

    const mutations: readonly [
      string,
      (reports: ReturnType<typeof finalReports>) => void,
    ][] = [
      [
        "pristine baseline",
        ({ v6: report }) => (report.baselineSha256 = "0".repeat(64)),
      ],
      [
        "baseline with itself",
        ({ midpoint: report }) => (report.candidateSha256 = PRISTINE_BASELINE_SHA256),
      ],
      ["all modes", ({ v6: report }) => (report.config.modes = ["fast"])],
      ["repeat 5", ({ midpoint: report }) => (report.config.repeat = 1)],
      ["exactly 13 states", ({ v6: report }) => (report.states = 12)],
      ["exact protected bank", ({ v6: report }) => (report.stateBank += ".copy")],
      ["exactly 64 states", ({ midpoint: report }) => (report.states = 63)],
      [
        "same candidate artifact",
        ({ midpoint: report }) => (report.candidateSha256 = "c".repeat(64)),
      ],
      [
        "state-bank manifest",
        ({ midpoint: report }) => delete report.stateBankManifest,
      ],
    ];

    for (const [message, mutate] of mutations) {
      const reports = finalReports();
      mutate(reports);
      expect(
        () =>
          assertRefactorPerformanceContract(
            reports.v6,
            reports.midpoint,
            "b".repeat(64),
          ),
        message,
      ).toThrow();
    }

    expect(() =>
      assertRefactorPerformanceContract(v6, midpoint, "c".repeat(64)),
    ).toThrow("do not use the supplied current candidate artifact");
  });

  it("accepts a same-byte candidate copy and rejects path replacement", () => {
    const root = temporaryDirectory();
    const source = path.join(root, "candidate.mjs");
    const copy = path.join(root, "candidate-copy.mjs");
    const sourceText =
      'export class Game {}\nexport const GameVariant = { Classic: "Classic" };\n';
    fs.writeFileSync(source, sourceText);
    fs.copyFileSync(source, copy);
    const sourceArtifact = readPublicBundleArtifact(source, "current candidate");
    const copyArtifact = readPublicBundleArtifact(copy, "current candidate");
    const reports = finalReports();
    reports.v6.candidateSha256 = sourceArtifact.sha256;
    reports.midpoint.candidateSha256 = sourceArtifact.sha256;

    expect(copyArtifact.sha256).toBe(sourceArtifact.sha256);
    expect(() =>
      assertRefactorPerformanceContract(
        reports.v6,
        reports.midpoint,
        copyArtifact.sha256,
      ),
    ).not.toThrow();
    expect(() => assertCurrentCandidateUnchanged(sourceArtifact)).not.toThrow();

    fs.renameSync(source, `${source}.replaced`);
    fs.writeFileSync(source, sourceText);
    expect(() => assertCurrentCandidateUnchanged(sourceArtifact)).toThrow(
      "current candidate bundle changed while checking reports",
    );
  });

  it("byte-compares the midpoint bank with a fresh derivation", () => {
    const root = temporaryDirectory();
    const stateBank = path.join(root, "midpoint.jsonl");
    const derived = '{"id":"derived","fen":"state"}\n';
    fs.writeFileSync(stateBank, derived);
    const report = {
      stateBank,
      stateBankSha256: sha256(derived),
    };

    expect(() => assertDerivedMidpointStateBank(report, derived)).not.toThrow();
    report.stateBankSha256 = "0".repeat(64);
    expect(() => assertDerivedMidpointStateBank(report, derived)).toThrow(
      "differs from fresh pristine-baseline derivation",
    );
    report.stateBankSha256 = sha256(derived);
    expect(() =>
      assertDerivedMidpointStateBank(report, '{"id":"different","fen":"state"}\n'),
    ).toThrow("differs from fresh pristine-baseline derivation");
  });
});

type FinalReport = {
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
};

type RefactorPerformanceModule = {
  PRISTINE_BASELINE_SHA256: string;
  assertCurrentCandidateUnchanged: (artifact: PublicBundleArtifact) => void;
  assertDerivedMidpointStateBank: (
    report: Pick<FinalReport, "stateBank" | "stateBankSha256">,
    derivedStateBank: string,
  ) => void;
  assertRefactorPerformanceContract: (
    v6Report: FinalReport,
    midpointReport: FinalReport,
    currentCandidateSha256: string,
  ) => void;
};

type PublicBundleArtifact = {
  readonly path: string;
  readonly bytes: Buffer;
  readonly identity: string;
  readonly sha256: string;
};

type OptionsModule = {
  readPublicBundleArtifact: (filePath: string, role: string) => PublicBundleArtifact;
};

function finalReports(): { midpoint: FinalReport; v6: FinalReport } {
  const common = {
    baseline: "/tmp/pristine-baseline.mjs",
    baselineSha256: PRISTINE_BASELINE_SHA256,
    candidate: "/tmp/candidate.mjs",
    candidateSha256: "b".repeat(64),
    config: {
      modes: ["fast", "normal", "pro"],
      repeat: 5,
      order: "ABBA",
      samplesPerBundlePerState: 10,
    },
  };
  return {
    v6: {
      ...structuredClone(common),
      stateBank: v6StateBank,
      stateBankSha256:
        "32799d75bebfe49494770af1657ff232208244081f7eafe9adaa606b1a251ee1",
      states: 13,
    },
    midpoint: {
      ...structuredClone(common),
      stateBank: "/tmp/midpoint-64.jsonl",
      stateBankSha256: "d".repeat(64),
      stateBankManifest: "/tmp/midpoint-64.manifest.json",
      stateBankManifestSha256: "e".repeat(64),
      states: 64,
    },
  };
}

function temporaryDirectory(): string {
  const directory = fs.mkdtempSync(
    path.join(os.tmpdir(), "mons-refactor-performance-check-"),
  );
  temporaryDirectories.push(directory);
  return directory;
}

function sha256(value: string): string {
  return createHash("sha256").update(value).digest("hex");
}
