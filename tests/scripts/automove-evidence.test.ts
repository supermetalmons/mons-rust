import { spawn, spawnSync, type SpawnSyncReturns } from "node:child_process";
import { createHash } from "node:crypto";
import { once } from "node:events";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";

import { afterEach, describe, expect, it, vi } from "vitest";

import { writePerformanceStateBankFixture } from "./performance-state-bank.fixture.js";

const repositoryRoot = path.resolve(import.meta.dirname, "../..");
const strengthScript = path.join(repositoryRoot, "scripts/run-automove-strength.mjs");
const performanceScript = path.join(
  repositoryRoot,
  "scripts/run-automove-performance.mjs",
);
const temporaryDirectories: string[] = [];

afterEach(() => {
  for (const directory of temporaryDirectories.splice(0)) {
    fs.rmSync(directory, { force: true, recursive: true });
  }
});

describe("automove evidence runners", () => {
  it("runs mirrored strength pairs with stable rows and explicit invalids", () => {
    const root = temporaryDirectory();
    const baseline = writeBundle(root, "baseline.mjs", "B");
    const candidate = writeBundle(root, "candidate.mjs", "C");
    const baselineBefore = fs.readFileSync(baseline);
    const candidateBefore = fs.readFileSync(candidate);
    const output = path.join(root, "reports", "strength.json");
    const result = run(strengthScript, [
      "--baseline",
      baseline,
      "--candidate",
      candidate,
      "--modes",
      "normal,fast",
      "--variants",
      "Classic",
      "--max-plies",
      "2",
      "--out",
      output,
    ]);

    expect(result.status, result.stderr).toBe(0);
    const report = readJson(output);
    expect(report.baselineSha256).toBe(bytesSha256(baselineBefore));
    expect(report.candidateSha256).toBe(bytesSha256(candidateBefore));
    expect(report.config).toMatchObject({
      modes: ["fast", "normal"],
      variants: ["Classic"],
      maxPlies: 2,
      gamesPerPairedUnit: 2,
    });
    expect(report.modes).toHaveLength(2);
    expect(report.modes[0]).toMatchObject({
      mode: "fast",
      games: 2,
      validGames: 2,
      wins: 2,
      draws: 0,
      losses: 0,
      cutoffs: 0,
      score: 1,
      invalids: { total: 0 },
      pairedUnits: { total: 1, wins: 1, inconclusive: 0 },
    });
    expect(
      report.rows.map((row: { mode: string; variant: string }) => [
        row.mode,
        row.variant,
      ]),
    ).toEqual([
      ["fast", "Classic"],
      ["normal", "Classic"],
    ]);
    expect(
      report.games.map(
        (game: { mode: string; candidateColor: string; result: string }) => [
          game.mode,
          game.candidateColor,
          game.result,
        ],
      ),
    ).toEqual([
      ["fast", "white", "win"],
      ["fast", "black", "win"],
      ["normal", "white", "win"],
      ["normal", "black", "win"],
    ]);
    expect(report.modes[0].timing).toMatchObject({
      white: { count: 2 },
      black: { count: 2 },
      baseline: { count: 2 },
      candidate: { count: 2 },
    });
    expect(report.pairedUnits[0]).toMatchObject({
      id: "fast/Classic",
      outcome: "win",
      candidatePoints: 2,
    });
    expect(report.config.gameDriving).toBe("fresh");
    expect(fs.readFileSync(baseline)).toEqual(baselineBefore);
    expect(fs.readFileSync(candidate)).toEqual(candidateBefore);

    for (const invalidCase of [
      {
        name: "mutation",
        strategy: "C",
        mutates: true,
        mutatesMetadata: false,
        invalidMetadata: false,
        key: "sourceMutation",
      },
      {
        name: "history-mutation",
        strategy: "C",
        mutates: false,
        mutatesMetadata: true,
        invalidMetadata: false,
        key: "sourceMutation",
      },
      {
        name: "payload",
        strategy: "payload",
        mutates: false,
        mutatesMetadata: false,
        invalidMetadata: false,
        key: "illegalReplay",
      },
      {
        name: "replay",
        strategy: "Z",
        mutates: false,
        mutatesMetadata: false,
        invalidMetadata: false,
        key: "illegalReplay",
      },
      {
        name: "metadata-type",
        strategy: "C",
        mutates: false,
        mutatesMetadata: false,
        invalidMetadata: true,
        key: "illegalReplay",
      },
      {
        name: "missing",
        strategy: "none",
        mutates: false,
        mutatesMetadata: false,
        invalidMetadata: false,
        key: "noSuggestion",
      },
    ] as const) {
      const invalidCandidate = writeBundle(
        root,
        `${invalidCase.name}.mjs`,
        invalidCase.strategy,
        invalidCase.mutates,
        invalidCase.mutatesMetadata,
        invalidCase.invalidMetadata,
      );
      const invalidOutput = path.join(root, `${invalidCase.name}.json`);
      const invalidResult = run(strengthScript, [
        "--baseline",
        baseline,
        "--candidate",
        invalidCandidate,
        "--modes",
        "fast",
        "--variants",
        "all",
        "--max-plies",
        "2",
        "--out",
        invalidOutput,
      ]);
      expect(invalidResult.status, invalidResult.stderr).toBe(1);
      expect(invalidResult.stderr).toContain("report written");
      const invalidReport = readJson(invalidOutput);
      expect(invalidReport.summary.score).toBeNull();
      expect(invalidReport.summary.invalids).toMatchObject({
        total: 2,
        [invalidCase.key]: 2,
      });
      expect(invalidReport.summary.pairedUnits).toMatchObject({
        total: 1,
        inconclusive: 1,
        invalids: 1,
      });
      expect(invalidReport.pairedUnits[0]).toMatchObject({
        outcome: "inconclusive",
        candidatePoints: null,
        inconclusiveReason: "invalid",
      });
    }
  });

  it("drives held games across plies when requested", () => {
    const root = temporaryDirectory();
    const baseline = writeBundle(root, "baseline.mjs", "B");
    const candidate = writeBundle(root, "candidate.mjs", "C");
    const output = path.join(root, "reports", "strength-held.json");
    const result = run(strengthScript, [
      "--baseline",
      baseline,
      "--candidate",
      candidate,
      "--modes",
      "fast",
      "--variants",
      "Classic",
      "--max-plies",
      "2",
      "--game-driving",
      "held",
      "--out",
      output,
    ]);

    expect(result.status, result.stderr).toBe(0);
    const report = readJson(output);
    expect(report.config.gameDriving).toBe("held");
    expect(report.modes[0]).toMatchObject({
      mode: "fast",
      games: 2,
      validGames: 2,
      wins: 2,
      losses: 0,
      cutoffs: 0,
      invalids: { total: 0 },
    });
  });

  it("classifies held-game drift as illegal replay", () => {
    const root = temporaryDirectory();
    const baseline = writeBundle(root, "baseline.mjs", "B");
    const candidate = writeBundle(root, "candidate.mjs", "stateful-playfen");
    const freshOutput = path.join(root, "reports", "strength-fresh.json");
    const freshResult = run(strengthScript, [
      "--baseline",
      baseline,
      "--candidate",
      candidate,
      "--modes",
      "fast",
      "--variants",
      "Classic",
      "--max-plies",
      "2",
      "--out",
      freshOutput,
    ]);
    expect(freshResult.status, freshResult.stderr).toBe(0);

    const heldOutput = path.join(root, "reports", "strength-drift.json");
    const heldResult = run(strengthScript, [
      "--baseline",
      baseline,
      "--candidate",
      candidate,
      "--modes",
      "fast",
      "--variants",
      "Classic",
      "--max-plies",
      "2",
      "--game-driving",
      "held",
      "--out",
      heldOutput,
    ]);
    expect(heldResult.status).toBe(1);
    expect(heldResult.stderr).toContain("invalid games");
    const report = readJson(heldOutput);
    expect(report.summary.invalids.illegalReplay).toBeGreaterThan(0);
    const invalidGame = report.games.find(
      (game: { invalid: null | { reason: string } }) => game.invalid !== null,
    );
    expect(invalidGame.invalid).toMatchObject({ reason: "illegal-replay" });
  });

  it("limits held replay validation to moves and positions", () => {
    const root = temporaryDirectory();
    const baseline = writeBundle(root, "baseline.mjs", "B");
    const candidate = writeBundle(root, "candidate.mjs", "stateful-history-playfen");
    const output = path.join(root, "reports", "strength-history-drift.json");
    const result = run(strengthScript, [
      "--baseline",
      baseline,
      "--candidate",
      candidate,
      "--modes",
      "fast",
      "--variants",
      "Classic",
      "--max-plies",
      "2",
      "--game-driving",
      "held",
      "--out",
      output,
    ]);

    expect(result.status, result.stderr).toBe(0);
    expect(readJson(output).summary.invalids.total).toBe(0);
  });

  it("compares the final held position with the fresh replay", () => {
    const root = temporaryDirectory();
    const baseline = writeBundle(root, "baseline.mjs", "stateful-winner-playfen");
    const candidate = writeBundle(root, "candidate.mjs", "stateful-winner-playfen");
    const output = path.join(root, "reports", "strength-winner-drift.json");
    const result = run(strengthScript, [
      "--baseline",
      baseline,
      "--candidate",
      candidate,
      "--modes",
      "fast",
      "--variants",
      "Classic",
      "--max-plies",
      "2",
      "--game-driving",
      "held",
      "--out",
      output,
    ]);

    expect(result.status).toBe(1);
    expect(readJson(output).summary.invalids.illegalReplay).toBeGreaterThan(0);
  });

  it("rejects invalid held-game replay results", () => {
    const root = temporaryDirectory();
    const baseline = writeBundle(root, "baseline.mjs", "B");
    const candidate = writeBundle(root, "candidate.mjs", "invalid-held-playfen");
    const output = path.join(root, "reports", "strength-invalid-held.json");
    const result = run(strengthScript, [
      "--baseline",
      baseline,
      "--candidate",
      candidate,
      "--modes",
      "fast",
      "--variants",
      "Classic",
      "--max-plies",
      "2",
      "--game-driving",
      "held",
      "--out",
      output,
    ]);

    expect(result.status).toBe(1);
    expect(readJson(output).summary.invalids.illegalReplay).toBeGreaterThan(0);
  });

  it("rejects incorrect complete held-game replay payloads", () => {
    const root = temporaryDirectory();
    const baseline = writeBundle(root, "baseline.mjs", "B");
    const candidate = writeBundle(root, "candidate.mjs", "wrong-held-complete-payload");
    const output = path.join(root, "reports", "strength-wrong-held-payload.json");
    const result = run(strengthScript, [
      "--baseline",
      baseline,
      "--candidate",
      candidate,
      "--modes",
      "fast",
      "--variants",
      "Classic",
      "--max-plies",
      "2",
      "--game-driving",
      "held",
      "--out",
      output,
    ]);

    expect(result.status).toBe(1);
    expect(readJson(output).summary.invalids.illegalReplay).toBeGreaterThan(0);
  });

  it("rejects unknown game driving modes", () => {
    const root = temporaryDirectory();
    const baseline = writeBundle(root, "baseline.mjs", "B");
    const candidate = writeBundle(root, "candidate.mjs", "C");
    const result = run(strengthScript, [
      "--baseline",
      baseline,
      "--candidate",
      candidate,
      "--modes",
      "fast",
      "--variants",
      "Classic",
      "--game-driving",
      "sticky",
      "--out",
      path.join(root, "reports", "strength.json"),
    ]);
    expect(result.status).toBe(1);
    expect(result.stderr).toContain("--game-driving must be one of fresh, held");
  });

  it("reports capped strength games as inconclusive cutoffs", () => {
    const root = temporaryDirectory();
    const baseline = writeBundle(root, "baseline.mjs", "B");
    const candidate = writeBundle(root, "candidate.mjs", "C");
    const output = path.join(root, "cutoff.json");
    const result = run(strengthScript, [
      "--baseline",
      baseline,
      "--candidate",
      candidate,
      "--modes",
      "fast",
      "--variants",
      "Classic",
      "--max-plies",
      "1",
      "--out",
      output,
    ]);

    expect(result.status, result.stderr).toBe(1);
    expect(result.stderr).toContain("contained 2 cutoff games; report written");
    const report = readJson(output);
    expect(report.summary).toMatchObject({
      games: 2,
      validGames: 0,
      wins: 0,
      draws: 0,
      losses: 0,
      cutoffs: 2,
      score: null,
      invalids: { total: 0 },
      pairedUnits: {
        total: 1,
        inconclusive: 1,
        invalids: 0,
        cutoffs: 1,
      },
    });
    expect(report.games.map((game: { result: string }) => game.result)).toEqual([
      "cutoff",
      "cutoff",
    ]);
    expect(report.pairedUnits[0]).toMatchObject({
      outcome: "inconclusive",
      candidatePoints: null,
      inconclusiveReason: "cutoff",
    });
  });

  it("runs sorted continuation pairs and reports invalid state FENs", () => {
    const root = temporaryDirectory();
    const baseline = writeBundle(root, "baseline.mjs", "B");
    const candidate = writeBundle(root, "candidate.mjs", "C");
    const initialState = JSON.stringify({
      variant: "Classic",
      ply: 0,
      candidateColor: null,
    });
    const stateBank = path.join(root, "continuations.jsonl");
    fs.writeFileSync(
      stateBank,
      [
        { id: "z-state", fen: initialState },
        {
          id: "m-invalid",
          fen: "not-a-fen",
          variant: "UntrustedVariantLabel",
        },
        { id: "a-state", fen: initialState, variant: "Classic" },
      ]
        .map((state) => JSON.stringify(state))
        .join("\n") + "\n",
    );
    const output = path.join(root, "continuation-strength.json");
    const result = run(strengthScript, [
      "--baseline",
      baseline,
      "--candidate",
      candidate,
      "--states",
      stateBank,
      "--modes",
      "normal,fast",
      "--max-plies",
      "2",
      "--out",
      output,
    ]);

    expect(result.status, result.stderr).toBe(1);
    expect(result.stderr).toContain("report written");
    const report = readJson(output);
    expect(report.config).toEqual({
      modes: ["fast", "normal"],
      stateIds: ["a-state", "m-invalid", "z-state"],
      maxPlies: 2,
      gameDriving: "fresh",
      gamesPerPairedUnit: 2,
    });
    expect(report.states).toBe(3);
    expect(report.stateBank).toBe(stateBank);
    expect(report.stateBankSha256).toBe(fileSha256(stateBank));
    expect(
      report.rows.map((row: { mode: string; stateId: string; variant?: string }) => [
        row.mode,
        row.stateId,
        row.variant ?? null,
      ]),
    ).toEqual([
      ["fast", "a-state", "Classic"],
      ["fast", "m-invalid", "UntrustedVariantLabel"],
      ["fast", "z-state", null],
      ["normal", "a-state", "Classic"],
      ["normal", "m-invalid", "UntrustedVariantLabel"],
      ["normal", "z-state", null],
    ]);
    expect(report.modes[0]).toMatchObject({
      mode: "fast",
      games: 6,
      validGames: 4,
      wins: 4,
      draws: 0,
      losses: 0,
      cutoffs: 0,
      score: null,
      invalids: { total: 2, illegalReplay: 2 },
      pairedUnits: {
        total: 3,
        wins: 2,
        inconclusive: 1,
        invalids: 1,
      },
    });
    const invalidRow = report.rows.find(
      (row: { mode: string; stateId: string }) =>
        row.mode === "fast" && row.stateId === "m-invalid",
    );
    expect(invalidRow).toMatchObject({
      variant: "UntrustedVariantLabel",
      games: 2,
      validGames: 0,
      score: null,
      invalids: { total: 2, illegalReplay: 2 },
      pairedUnits: { total: 1, inconclusive: 1, invalids: 1 },
    });
    expect(
      report.games
        .filter(
          (game: { mode: string; stateId: string }) =>
            game.mode === "fast" && game.stateId === "m-invalid",
        )
        .map(
          (game: {
            candidateColor: string;
            result: string;
            invalid: { reason: string };
          }) => [game.candidateColor, game.result, game.invalid.reason],
        ),
    ).toEqual([
      ["white", "invalid", "illegal-replay"],
      ["black", "invalid", "illegal-replay"],
    ]);

    const exclusiveOutput = path.join(root, "exclusive.json");
    const exclusive = run(strengthScript, [
      "--baseline",
      baseline,
      "--candidate",
      candidate,
      "--states",
      stateBank,
      "--variants",
      "Classic",
      "--out",
      exclusiveOutput,
    ]);
    expect(exclusive.status).toBe(1);
    expect(exclusive.stderr).toContain(
      "--states and --variants are mutually exclusive",
    );
    expect(fs.existsSync(exclusiveOutput)).toBe(false);

    const mislabeledBank = path.join(root, "mislabeled.jsonl");
    fs.writeFileSync(
      mislabeledBank,
      `${JSON.stringify({
        id: "mislabeled",
        fen: initialState,
        variant: "MetadataOnly",
      })}\n`,
    );
    const mislabeledOutput = path.join(root, "mislabeled.json");
    const mislabeled = run(strengthScript, [
      "--baseline",
      baseline,
      "--candidate",
      candidate,
      "--states",
      mislabeledBank,
      "--modes",
      "fast",
      "--out",
      mislabeledOutput,
    ]);
    expect(mislabeled.status).toBe(1);
    expect(mislabeled.stderr).toContain(
      "variant metadata MetadataOnly does not match parsed variant Classic",
    );
    expect(fs.existsSync(mislabeledOutput)).toBe(false);
  });

  it("rejects terminal continuation states before strength games run", () => {
    const root = temporaryDirectory();
    const baseline = writeBundle(root, "baseline.mjs", "B");
    const candidate = writeBundle(root, "candidate.mjs", "C");
    const stateBank = path.join(root, "terminal.jsonl");
    fs.writeFileSync(
      stateBank,
      `${JSON.stringify({
        id: "already-finished",
        fen: JSON.stringify({
          variant: "Classic",
          ply: 2,
          candidateColor: "white",
        }),
        variant: "Classic",
      })}\n`,
    );
    const output = path.join(root, "terminal-strength.json");
    const result = run(strengthScript, [
      "--baseline",
      baseline,
      "--candidate",
      candidate,
      "--states",
      stateBank,
      "--modes",
      "fast",
      "--max-plies",
      "2",
      "--out",
      output,
    ]);

    expect(result.status).toBe(1);
    expect(result.stderr).toContain("state-bank state already-finished is terminal");
    expect(fs.existsSync(output)).toBe(false);
  });

  it("rejects terminal variant starts before strength games run", () => {
    const root = temporaryDirectory();
    const baseline = writeBundle(
      root,
      "terminal-baseline.mjs",
      "throw",
      false,
      false,
      false,
      true,
    );
    const candidate = writeBundle(
      root,
      "terminal-candidate.mjs",
      "throw",
      false,
      false,
      false,
      true,
    );
    const output = path.join(root, "terminal-variant-strength.json");
    const result = run(strengthScript, [
      "--baseline",
      baseline,
      "--candidate",
      candidate,
      "--variants",
      "Classic",
      "--modes",
      "fast",
      "--max-plies",
      "2",
      "--out",
      output,
    ]);

    expect(result.status).toBe(1);
    expect(result.stderr).toContain(
      "variant Classic is terminal and cannot be used for strength evidence",
    );
    expect(fs.existsSync(output)).toBe(false);
  });

  it("measures a sorted JSONL bank in repeated ABBA order", () => {
    const root = temporaryDirectory();
    const baseline = writeBundle(root, "baseline.mjs", "B");
    const candidate = writeBundle(root, "candidate.mjs", "C");
    const stateBank = path.join(root, "states.jsonl");
    const stateManifest = writeStateBankManifest(stateBank, baseline);
    const output = path.join(root, "performance.json");
    const result = run(performanceScript, [
      "--baseline",
      baseline,
      "--candidate",
      candidate,
      "--states",
      stateBank,
      "--state-manifest",
      stateManifest,
      "--repeat",
      "1",
      "--modes",
      "normal,fast",
      "--out",
      output,
    ]);

    expect(result.status, result.stderr).toBe(0);
    const report = readJson(output);
    expect(report.stateBankSha256).toBe(fileSha256(stateBank));
    expect(report.stateBankManifest).toBe(stateManifest);
    expect(report.stateBankManifestSha256).toBe(fileSha256(stateManifest));
    expect(report.config).toEqual({
      modes: ["fast", "normal"],
      repeat: 1,
      order: "ABBA",
      samplesPerBundlePerState: 2,
    });
    expect(report.states).toBe(64);
    const stateIds = fs
      .readFileSync(stateBank, "utf8")
      .trimEnd()
      .split("\n")
      .map((line) => (JSON.parse(line) as { id: string }).id)
      .sort();
    expect(
      report.rows.map((row: { mode: string; id: string }) => [row.mode, row.id]),
    ).toEqual(["fast", "normal"].flatMap((mode) => stateIds.map((id) => [mode, id])));
    for (const row of report.rows) {
      expect(row.callsPerBundle).toBe(2);
      expect(row.baseline).toMatchObject({
        count: 2,
        invalids: { total: 0 },
      });
      expect(row.candidate).toMatchObject({
        count: 2,
        invalids: { total: 0 },
      });
      expect(row.baseline.samplesMs).toHaveLength(2);
      expect(row.candidate.samplesMs).toHaveLength(2);
      expect(row.candidateBaselineRatio).toEqual({
        mean: expect.any(Number),
        median: expect.any(Number),
        p95: expect.any(Number),
        max: expect.any(Number),
      });
    }
    expect(report.summary).toMatchObject({
      states: 64,
      rows: 128,
      callsPerBundle: 256,
      baseline: { count: 256, invalids: { total: 0 } },
      candidate: { count: 256, invalids: { total: 0 } },
    });
  });

  it("requires a matching commit manifest for non-protected performance banks", () => {
    const root = temporaryDirectory();
    const baseline = writeBundle(root, "baseline.mjs", "B");
    const candidate = writeBundle(root, "candidate.mjs", "C");
    const stateBank = path.join(root, "states.jsonl");
    fs.writeFileSync(
      stateBank,
      `${JSON.stringify({
        id: "initial",
        fen: JSON.stringify({
          variant: "Classic",
          ply: 0,
          candidateColor: null,
        }),
      })}\n`,
    );

    const missingOutput = path.join(root, "missing-manifest.json");
    const missing = run(performanceScript, [
      "--baseline",
      baseline,
      "--candidate",
      candidate,
      "--states",
      stateBank,
      "--modes",
      "fast",
      "--repeat",
      "1",
      "--out",
      missingOutput,
    ]);
    expect(missing.status).toBe(1);
    expect(missing.stderr).toContain("--state-manifest is required");
    expect(fs.existsSync(missingOutput)).toBe(false);

    const stateManifest = writeStateBankManifest(stateBank, baseline);
    fs.writeFileSync(stateManifest, "{}\n");
    const mismatchedOutput = path.join(root, "mismatched-manifest.json");
    const mismatched = run(performanceScript, [
      "--baseline",
      baseline,
      "--candidate",
      candidate,
      "--states",
      stateBank,
      "--state-manifest",
      stateManifest,
      "--modes",
      "fast",
      "--repeat",
      "1",
      "--out",
      mismatchedOutput,
    ]);
    expect(mismatched.status).toBe(1);
    expect(mismatched.stderr).toContain(
      "performance state-bank manifest does not match state bank",
    );
    expect(fs.existsSync(mismatchedOutput)).toBe(false);

    const wrongSourceManifest = writeStateBankManifest(stateBank, candidate);
    const wrongSourceOutput = path.join(root, "wrong-source-manifest.json");
    const wrongSource = run(performanceScript, [
      "--baseline",
      baseline,
      "--candidate",
      candidate,
      "--states",
      stateBank,
      "--state-manifest",
      wrongSourceManifest,
      "--modes",
      "fast",
      "--repeat",
      "1",
      "--out",
      wrongSourceOutput,
    ]);
    expect(wrongSource.status).toBe(1);
    expect(wrongSource.stderr).toContain(
      "manifest source bundle does not match baseline bundle",
    );
    expect(fs.existsSync(wrongSourceOutput)).toBe(false);
  });

  it("accepts an immutable automove decision bank without a manifest", () => {
    const root = temporaryDirectory();
    const baseline = writeBundle(root, "baseline.mjs", "B");
    const candidate = writeBundle(root, "candidate.mjs", "C");
    const stateBank = path.join(
      repositoryRoot,
      "test-data/automove-decisions/v6/decisions.jsonl",
    );
    const output = path.join(root, "protected-bank.json");
    const result = run(performanceScript, [
      "--baseline",
      baseline,
      "--candidate",
      candidate,
      "--states",
      stateBank,
      "--modes",
      "fast",
      "--repeat",
      "1",
      "--out",
      output,
    ]);

    expect(result.status).toBe(1);
    expect(result.stderr).toContain("invalid calls; report written");
    expect(result.stderr).not.toContain("state-manifest");
    expect(readJson(output).stateBank).toBe(stateBank);
  });

  it("recognizes protected banks by opened-file identity", () => {
    const root = temporaryDirectory();
    const baseline = writeBundle(root, "baseline.mjs", "B");
    const candidate = writeBundle(root, "candidate.mjs", "C");
    const protectedBank = path.join(
      repositoryRoot,
      "test-data/automove-decisions/v6/decisions.jsonl",
    );
    const alias = path.join(root, "protected-alias.jsonl");
    fs.symlinkSync(protectedBank, alias);
    const output = path.join(root, "protected-alias-report.json");
    const result = run(performanceScript, [
      "--baseline",
      baseline,
      "--candidate",
      candidate,
      "--states",
      alias,
      "--modes",
      "fast",
      "--repeat",
      "1",
      "--out",
      output,
    ]);
    expect(result.status).toBe(1);
    expect(result.stderr).toContain("invalid calls; report written");
    expect(result.stderr).not.toContain("state-manifest");
  });

  it.runIf(process.platform !== "win32")(
    "rejects a FIFO bank before a path can be retargeted as protected",
    async () => {
      const root = temporaryDirectory();
      const fifo = path.join(root, "states.fifo");
      const alias = path.join(root, "states.jsonl");
      const protectedBank = path.join(
        repositoryRoot,
        "test-data/automove-decisions/v6/decisions.jsonl",
      );
      const makeFifo = spawnSync("mkfifo", [fifo], { encoding: "utf8" });
      expect(makeFifo.status, makeFifo.stderr).toBe(0);
      fs.symlinkSync(fifo, alias);
      const output = path.join(root, "fifo-report.json");
      const writerReady = path.join(root, "writer-ready");
      const baseline = writeBundle(root, "baseline.mjs", "B");
      const candidate = writeBundle(root, "candidate.mjs", "C");
      const writer = spawn(
        process.execPath,
        [
          "--input-type=module",
          "--eval",
          [
            'import { closeSync, openSync, symlinkSync, unlinkSync, writeFileSync } from "node:fs";',
            `writeFileSync(${JSON.stringify(writerReady)}, "");`,
            `const descriptor = openSync(${JSON.stringify(fifo)}, "w");`,
            `unlinkSync(${JSON.stringify(alias)});`,
            `symlinkSync(${JSON.stringify(protectedBank)}, ${JSON.stringify(alias)});`,
            `try { writeFileSync(descriptor, ${JSON.stringify(`${JSON.stringify({ id: "evil", fen: "evil" })}\n`)}); } catch {}`,
            "closeSync(descriptor);",
          ].join("\n"),
        ],
        { stdio: "ignore" },
      );
      await vi.waitFor(() => expect(fs.existsSync(writerReady)).toBe(true), {
        timeout: 2_000,
      });
      await new Promise((resolve) => setTimeout(resolve, 50));
      const result = spawnSync(
        process.execPath,
        [
          performanceScript,
          "--baseline",
          baseline,
          "--candidate",
          candidate,
          "--states",
          alias,
          "--modes",
          "fast",
          "--repeat",
          "1",
          "--out",
          output,
        ],
        {
          cwd: repositoryRoot,
          encoding: "utf8",
          timeout: 5_000,
          killSignal: "SIGKILL",
        },
      );
      if (writer.exitCode === null) {
        await Promise.race([
          once(writer, "exit"),
          new Promise((resolve) => setTimeout(resolve, 2_000)),
        ]);
      }
      if (writer.exitCode === null) writer.kill("SIGKILL");
      expect(result.error).toBeUndefined();
      expect(result.status).toBe(1);
      expect(result.stderr).toContain("not a regular file");
      expect(fs.realpathSync(alias)).toBe(protectedBank);
      expect(fs.existsSync(output)).toBe(false);
    },
  );

  it("fails closed without timing a fast invalid candidate", () => {
    const root = temporaryDirectory();
    const baseline = writeBundle(root, "baseline.mjs", "B");
    const candidate = writeBundle(root, "fast-invalid.mjs", "none");
    const stateBank = path.join(root, "states.jsonl");
    const stateManifest = writeStateBankManifest(stateBank, baseline);
    const output = path.join(root, "fast-invalid-performance.json");
    const result = run(performanceScript, [
      "--baseline",
      baseline,
      "--candidate",
      candidate,
      "--states",
      stateBank,
      "--state-manifest",
      stateManifest,
      "--repeat",
      "1",
      "--modes",
      "fast",
      "--out",
      output,
    ]);

    expect(result.status).toBe(1);
    expect(result.stderr).toContain("invalid calls; report written");
    expect(fs.existsSync(output)).toBe(true);
    const report = readJson(output);
    expect(report.rows[0]).toMatchObject({
      callsPerBundle: 2,
      baseline: { count: 2, invalids: { total: 0 } },
      candidate: {
        count: 0,
        samplesMs: [],
        invalids: { total: 2, noSuggestion: 2 },
      },
      candidateBaselineRatio: null,
    });
    expect(report.summary).toMatchObject({
      baseline: { count: 128, invalids: { total: 0 } },
      candidate: { count: 0, invalids: { total: 128, noSuggestion: 128 } },
      candidateBaselineRatio: null,
    });
  });

  it("classifies selector, payload, and preview failures without aborting", () => {
    const root = temporaryDirectory();
    const baseline = writeBundle(root, "baseline.mjs", "B");
    const stateBank = path.join(root, "states.jsonl");
    const stateManifest = writeStateBankManifest(stateBank, baseline);

    for (const invalidCase of [
      {
        strategy: "throw",
        output: "selector-error.json",
        counts: { selectorError: 2, sourceMutation: 0, illegalReplay: 0 },
      },
      {
        strategy: "mutating-throw",
        output: "mutating-selector-error.json",
        counts: { selectorError: 0, sourceMutation: 2, illegalReplay: 0 },
      },
      {
        strategy: "throwing-payload",
        output: "throwing-payload.json",
        counts: { selectorError: 0, sourceMutation: 0, illegalReplay: 2 },
      },
      {
        strategy: "mutating-invalid-payload",
        output: "mutating-invalid-payload.json",
        counts: { selectorError: 0, sourceMutation: 2, illegalReplay: 0 },
      },
      {
        strategy: "mutating-preview-throw",
        output: "mutating-preview-throw.json",
        counts: { selectorError: 0, sourceMutation: 2, illegalReplay: 0 },
      },
    ] as const) {
      const candidate = writeBundle(
        root,
        `${invalidCase.strategy}.mjs`,
        invalidCase.strategy,
      );
      const output = path.join(root, invalidCase.output);
      const result = run(performanceScript, [
        "--baseline",
        baseline,
        "--candidate",
        candidate,
        "--states",
        stateBank,
        "--state-manifest",
        stateManifest,
        "--repeat",
        "1",
        "--modes",
        "fast",
        "--out",
        output,
      ]);

      expect(result.status, result.stderr).toBe(1);
      expect(result.stderr).toContain("invalid calls; report written");
      expect(readJson(output).summary.candidate.invalids).toMatchObject({
        total: 128,
        selectorError: invalidCase.counts.selectorError * 64,
        sourceMutation: invalidCase.counts.sourceMutation * 64,
        illegalReplay: invalidCase.counts.illegalReplay * 64,
      });
      if (invalidCase.strategy === "mutating-preview-throw") {
        expect(readJson(output).summary.baseline.invalids).toMatchObject({
          total: 128,
          sourceMutation: 128,
        });
      }
    }
  }, 15_000);

  it("fails when a bundle changes while it is being loaded", async () => {
    const root = temporaryDirectory();
    const baseline = writeBundle(root, "baseline.mjs", "B");
    const candidate = writeBundle(root, "candidate.mjs", "C");
    const candidateSource = fs.readFileSync(candidate, "utf8");
    const marker = "candidate-load-started";
    fs.writeFileSync(
      candidate,
      `console.log(${JSON.stringify(marker)});
const pauseUntil = performance.now() + 300;
while (performance.now() < pauseUntil) {}

${candidateSource}`,
    );
    const output = path.join(root, "changed-bundle.json");
    const child = spawn(
      process.execPath,
      [
        performanceScript,
        "--baseline",
        baseline,
        "--candidate",
        candidate,
        "--states",
        path.join(root, "unused.jsonl"),
        "--repeat",
        "1",
        "--modes",
        "fast",
        "--out",
        output,
      ],
      { cwd: repositoryRoot, stdio: ["ignore", "pipe", "pipe"] },
    );
    child.stdout.setEncoding("utf8");
    child.stderr.setEncoding("utf8");
    let stdout = "";
    let stderr = "";
    let changed = false;
    child.stdout.on("data", (chunk: string) => {
      stdout += chunk;
      if (!changed && stdout.includes(marker)) {
        fs.appendFileSync(candidate, "\n");
        changed = true;
      }
    });
    child.stderr.on("data", (chunk: string) => {
      stderr += chunk;
    });
    const timeout = setTimeout(() => child.kill("SIGKILL"), 5_000);
    const [status] = (await once(child, "exit")) as [number | null];
    clearTimeout(timeout);

    expect(changed).toBe(true);
    expect(status).toBe(1);
    expect(stderr).toContain("candidate bundle changed while loading");
    expect(fs.existsSync(output)).toBe(false);
  });

  it("rejects bundles that delegate behavior to mutable local imports", () => {
    const root = temporaryDirectory();
    const implementation = writeBundle(root, "implementation.mjs", "B");
    const entry = path.join(root, "entry.mjs");
    const entryBytes = 'export { Game, GameVariant } from "./implementation.mjs";\n';
    fs.writeFileSync(entry, entryBytes);
    const output = path.join(root, "local-import.json");
    const result = run(strengthScript, [
      "--baseline",
      entry,
      "--candidate",
      implementation,
      "--modes",
      "fast",
      "--variants",
      "Classic",
      "--max-plies",
      "1",
      "--out",
      output,
    ]);
    expect(result.status).toBe(1);
    expect(result.stderr).toContain("not self-contained: ./implementation.mjs");
    expect(fs.readFileSync(entry, "utf8")).toBe(entryBytes);
    expect(fs.existsSync(output)).toBe(false);
  });

  it("blocks computed string-code loaders before they can import unhashed code", () => {
    const root = temporaryDirectory();
    const implementation = writeBundle(root, "implementation.mjs", "B");
    const implementationUrl = pathToFileURL(implementation).href;
    const importExpression = `import(${JSON.stringify(implementationUrl)})`;
    const loaders = [
      [
        "computed-function",
        `const capability = ["Fun", "ction"].join("");
const compile = globalThis[capability];
const load = compile(${JSON.stringify(`return ${importExpression}`)});
const dependency = await load();`,
      ],
      [
        "computed-constructor",
        `const capability = ["con", "structor"].join("");
const compile = (() => {})[capability];
const load = compile(${JSON.stringify(`return ${importExpression}`)});
const dependency = await load();`,
      ],
      [
        "computed-eval",
        `const capability = String.fromCharCode(101, 118, 97, 108);
const dependency = await (0, globalThis[capability])(${JSON.stringify(importExpression)});`,
      ],
    ] as const;

    for (const [name, loader] of loaders) {
      const entry = path.join(root, `${name}.mjs`);
      const output = path.join(root, `${name}.json`);
      fs.writeFileSync(
        entry,
        `${loader}
export const Game = dependency.Game;
export const GameVariant = dependency.GameVariant;
`,
      );
      const result = run(strengthScript, [
        "--baseline",
        entry,
        "--candidate",
        implementation,
        "--modes",
        "fast",
        "--variants",
        "Classic",
        "--max-plies",
        "1",
        "--out",
        output,
      ]);

      expect(result.status).toBe(1);
      expect(result.stderr).toContain(
        "Code generation from strings disallowed for this context",
      );
      expect(fs.existsSync(output)).toBe(false);
    }
  });

  it("blocks native module access before it can require unhashed code", () => {
    const root = temporaryDirectory();
    const candidate = writeBundle(root, "candidate.mjs", "B");
    const implementation = path.join(root, "implementation.cjs");
    const marker = path.join(root, "external-code-ran");
    const entry = path.join(root, "native-loader.mjs");
    const output = path.join(root, "native-loader.json");
    fs.writeFileSync(
      implementation,
      `require("node:fs").writeFileSync(${JSON.stringify(marker)}, "loaded");
module.exports = { Game: class Game {}, GameVariant: { Classic: "Classic" } };
`,
    );
    fs.writeFileSync(
      entry,
      `const moduleName = ["node", "module"].join(":");
const nativeModule = process.getBuiltinModule(moduleName);
const dependency = nativeModule.createRequire(${JSON.stringify(entry)})(${JSON.stringify(implementation)});
export const Game = dependency.Game;
export const GameVariant = dependency.GameVariant;
`,
    );

    const result = run(strengthScript, [
      "--baseline",
      entry,
      "--candidate",
      candidate,
      "--modes",
      "fast",
      "--variants",
      "Classic",
      "--max-plies",
      "1",
      "--out",
      output,
    ]);

    expect(result.status).toBe(1);
    expect(result.stderr).toContain("process is not defined");
    expect(fs.existsSync(marker)).toBe(false);
    expect(fs.existsSync(output)).toBe(false);
  });

  it("fails closed when bundle loading is called without the runtime guard", () => {
    const root = temporaryDirectory();
    const bundle = writeBundle(root, "bundle.mjs", "B");
    const optionsModule = pathToFileURL(
      path.join(repositoryRoot, "scripts/evidence/options.mjs"),
    ).href;
    const result = spawnSync(
      process.execPath,
      [
        "--experimental-vm-modules",
        "--input-type=module",
        "--eval",
        `import { DISALLOW_STRING_CODE_GENERATION_FLAG, loadPublicBundle } from ${JSON.stringify(optionsModule)}; process.execArgv.push(DISALLOW_STRING_CODE_GENERATION_FLAG); await loadPublicBundle(${JSON.stringify(bundle)}, "probe");`,
      ],
      { cwd: repositoryRoot, encoding: "utf8" },
    );

    expect(result.status).toBe(1);
    expect(result.stderr).toContain(
      "probe bundle execution requires --disallow-code-generation-from-strings",
    );
  });

  it("recognizes the runtime guard when NODE_OPTIONS enables it", () => {
    const root = temporaryDirectory();
    const bundle = writeBundle(root, "bundle.mjs", "B");
    const optionsModule = pathToFileURL(
      path.join(repositoryRoot, "scripts/evidence/options.mjs"),
    ).href;
    const result = spawnSync(
      process.execPath,
      [
        "--input-type=module",
        "--eval",
        `import { loadPublicBundle } from ${JSON.stringify(optionsModule)}; const bundle = await loadPublicBundle(${JSON.stringify(bundle)}, "probe"); console.log(bundle.sha256);`,
      ],
      {
        cwd: repositoryRoot,
        encoding: "utf8",
        env: {
          ...process.env,
          NODE_OPTIONS: [
            process.env["NODE_OPTIONS"],
            "--experimental-vm-modules",
            "--disallow-code-generation-from-strings",
          ]
            .filter(Boolean)
            .join(" "),
        },
      },
    );

    expect(result.status, result.stderr).toBe(0);
    expect(result.stdout.trim()).toBe(bytesSha256(fs.readFileSync(bundle)));
  });

  it("rejects numeric work limits that are not safe integers", () => {
    const root = temporaryDirectory();
    const baseline = writeBundle(root, "baseline.mjs", "B");
    const candidate = writeBundle(root, "candidate.mjs", "C");
    const stateBank = path.join(root, "states.jsonl");
    const stateManifest = writeStateBankManifest(stateBank, baseline);

    const performanceOutput = path.join(root, "unsafe-repeat.json");
    const performanceResult = run(performanceScript, [
      "--baseline",
      baseline,
      "--candidate",
      candidate,
      "--states",
      stateBank,
      "--state-manifest",
      stateManifest,
      "--repeat",
      "9007199254740993",
      "--modes",
      "fast",
      "--out",
      performanceOutput,
    ]);
    expect(performanceResult.status).toBe(1);
    expect(performanceResult.stderr).toContain(
      "--repeat must be a positive safe integer",
    );
    expect(fs.existsSync(performanceOutput)).toBe(false);

    const strengthOutput = path.join(root, "unsafe-max-plies.json");
    const strengthResult = run(strengthScript, [
      "--baseline",
      baseline,
      "--candidate",
      candidate,
      "--variants",
      "Classic",
      "--max-plies",
      "9007199254740993",
      "--out",
      strengthOutput,
    ]);
    expect(strengthResult.status).toBe(1);
    expect(strengthResult.stderr).toContain(
      "--max-plies must be a positive safe integer",
    );
    expect(fs.existsSync(strengthOutput)).toBe(false);
  });

  it("accepts practical work-limit boundaries and rejects larger values", () => {
    const root = temporaryDirectory();
    const baseline = writeBundle(root, "baseline.mjs", "B");
    const candidate = writeBundle(root, "candidate.mjs", "C");
    const stateBank = path.join(root, "states.jsonl");
    const stateManifest = writeStateBankManifest(stateBank, baseline);

    const boundaryPerformanceOutput = path.join(root, "boundary-repeat.json");
    const boundaryPerformance = run(performanceScript, [
      "--baseline",
      baseline,
      "--candidate",
      candidate,
      "--states",
      stateBank,
      "--state-manifest",
      stateManifest,
      "--repeat",
      "100",
      "--modes",
      "fast",
      "--out",
      boundaryPerformanceOutput,
    ]);
    expect(boundaryPerformance.status, boundaryPerformance.stderr).toBe(0);
    expect(readJson(boundaryPerformanceOutput).config.repeat).toBe(100);

    const excessivePerformanceOutput = path.join(root, "excessive-repeat.json");
    const excessivePerformance = run(performanceScript, [
      "--baseline",
      baseline,
      "--candidate",
      candidate,
      "--states",
      stateBank,
      "--repeat",
      "101",
      "--modes",
      "fast",
      "--out",
      excessivePerformanceOutput,
    ]);
    expect(excessivePerformance.status).toBe(1);
    expect(excessivePerformance.stderr).toContain("--repeat must be at most 100");
    expect(fs.existsSync(excessivePerformanceOutput)).toBe(false);

    const boundaryStrengthOutput = path.join(root, "boundary-max-plies.json");
    const boundaryStrength = run(strengthScript, [
      "--baseline",
      baseline,
      "--candidate",
      candidate,
      "--variants",
      "Classic",
      "--modes",
      "fast",
      "--max-plies",
      "1024",
      "--out",
      boundaryStrengthOutput,
    ]);
    expect(boundaryStrength.status, boundaryStrength.stderr).toBe(0);
    expect(readJson(boundaryStrengthOutput).config.maxPlies).toBe(1024);

    const excessiveStrengthOutput = path.join(root, "excessive-max-plies.json");
    const excessiveStrength = run(strengthScript, [
      "--baseline",
      baseline,
      "--candidate",
      candidate,
      "--variants",
      "Classic",
      "--modes",
      "fast",
      "--max-plies",
      "1025",
      "--out",
      excessiveStrengthOutput,
    ]);
    expect(excessiveStrength.status).toBe(1);
    expect(excessiveStrength.stderr).toContain("--max-plies must be at most 1024");
    expect(fs.existsSync(excessiveStrengthOutput)).toBe(false);
  });

  it("preflights report destinations before reading evidence inputs", () => {
    const root = temporaryDirectory();
    const missingBaseline = path.join(root, "missing-baseline.mjs");
    const missingCandidate = path.join(root, "missing-candidate.mjs");
    const missingStates = path.join(root, "missing-states.jsonl");
    const existingOutput = path.join(root, "existing.json");
    const existingBytes = Buffer.from("preserve this report\n");
    fs.writeFileSync(existingOutput, existingBytes);
    const forbiddenOutput = path.join(repositoryRoot, "package.json");
    const forbiddenBytes = fs.readFileSync(forbiddenOutput);
    const invocations = [
      {
        script: performanceScript,
        arguments: [
          "--baseline",
          missingBaseline,
          "--candidate",
          missingCandidate,
          "--states",
          missingStates,
          "--modes",
          "fast",
          "--repeat",
          "1",
        ],
      },
      {
        script: strengthScript,
        arguments: [
          "--baseline",
          missingBaseline,
          "--candidate",
          missingCandidate,
          "--variants",
          "Classic",
          "--modes",
          "fast",
          "--max-plies",
          "1",
        ],
      },
    ];

    for (const invocation of invocations) {
      const missing = run(invocation.script, invocation.arguments);
      expect(missing.status).toBe(1);
      expect(missing.stderr).toContain("missing required option: --out");

      const existing = run(invocation.script, [
        ...invocation.arguments,
        "--out",
        existingOutput,
      ]);
      expect(existing.status).toBe(1);
      expect(existing.stderr).toContain(
        "refusing to overwrite existing evidence report",
      );

      const forbidden = run(invocation.script, [
        ...invocation.arguments,
        "--out",
        forbiddenOutput,
      ]);
      expect(forbidden.status).toBe(1);
      expect(forbidden.stderr).toContain(
        "outside target/ or the OS temporary directory",
      );
    }

    expect(fs.existsSync(missingBaseline)).toBe(false);
    expect(fs.existsSync(missingCandidate)).toBe(false);
    expect(fs.existsSync(missingStates)).toBe(false);
    expect(fs.readFileSync(existingOutput)).toEqual(existingBytes);
    expect(fs.readFileSync(forbiddenOutput)).toEqual(forbiddenBytes);
  });

  it("refuses report destinations in arbitrary repository paths", () => {
    const root = temporaryDirectory();
    const baseline = writeBundle(root, "baseline.mjs", "B");
    const candidate = writeBundle(root, "candidate.mjs", "C");
    const configDestination = path.join(repositoryRoot, "package.json");
    const configBefore = fs.readFileSync(configDestination);
    const sourceDestination = path.join(
      repositoryRoot,
      "scripts",
      "automove-evidence-forbidden.json",
    );
    expect(fs.existsSync(sourceDestination)).toBe(false);

    for (const destination of [configDestination, sourceDestination]) {
      const result = run(strengthScript, [
        "--baseline",
        baseline,
        "--candidate",
        candidate,
        "--modes",
        "fast",
        "--variants",
        "Classic",
        "--max-plies",
        "1",
        "--out",
        destination,
      ]);
      expect(result.status).toBe(1);
      expect(result.stderr).toContain("outside target/ or the OS temporary directory");
    }

    expect(fs.readFileSync(configDestination)).toEqual(configBefore);
    expect(fs.existsSync(sourceDestination)).toBe(false);
  });

  it("refuses report paths that escape through directory symlinks", () => {
    const root = temporaryDirectory();
    const baseline = writeBundle(root, "baseline.mjs", "B");
    const candidate = writeBundle(root, "candidate.mjs", "C");
    const protectedDestination = path.join(
      repositoryRoot,
      "test-data",
      "automove-evidence-symlink-forbidden.json",
    );
    const sourceDestination = path.join(
      repositoryRoot,
      "scripts",
      "automove-evidence-symlink-forbidden.json",
    );
    expect(fs.existsSync(protectedDestination)).toBe(false);
    expect(fs.existsSync(sourceDestination)).toBe(false);
    const linkedTestData = path.join(root, "linked-test-data");
    const linkedScripts = path.join(root, "linked-scripts");
    fs.symlinkSync(path.join(repositoryRoot, "test-data"), linkedTestData, "dir");
    fs.symlinkSync(path.join(repositoryRoot, "scripts"), linkedScripts, "dir");

    for (const [output, message] of [
      [
        path.join(linkedTestData, path.basename(protectedDestination)),
        "under test-data/",
      ],
      [
        path.join(linkedScripts, path.basename(sourceDestination)),
        "outside target/ or the OS temporary directory",
      ],
    ] as const) {
      const result = run(strengthScript, [
        "--baseline",
        baseline,
        "--candidate",
        candidate,
        "--modes",
        "fast",
        "--variants",
        "Classic",
        "--max-plies",
        "1",
        "--out",
        output,
      ]);
      expect(result.status).toBe(1);
      expect(result.stderr).toContain(message);
    }

    expect(fs.existsSync(protectedDestination)).toBe(false);
    expect(fs.existsSync(sourceDestination)).toBe(false);
  });

  it("refuses to overwrite an existing report", () => {
    const root = temporaryDirectory();
    const baseline = writeBundle(root, "baseline.mjs", "B");
    const candidate = writeBundle(root, "candidate.mjs", "C");
    const output = path.join(root, "existing.json");
    const existing = Buffer.from("preserve this report\n");
    fs.writeFileSync(output, existing);

    const result = run(strengthScript, [
      "--baseline",
      baseline,
      "--candidate",
      candidate,
      "--modes",
      "fast",
      "--variants",
      "Classic",
      "--max-plies",
      "1",
      "--out",
      output,
    ]);

    expect(result.status).toBe(1);
    expect(result.stderr).toContain("refusing to overwrite existing evidence report");
    expect(fs.readFileSync(output)).toEqual(existing);
  });

  it("refuses to place a report in the immutable test-data tree", () => {
    const root = temporaryDirectory();
    const baseline = writeBundle(root, "baseline.mjs", "B");
    const candidate = writeBundle(root, "candidate.mjs", "C");
    const forbiddenOutput = path.join(
      repositoryRoot,
      "test-data",
      "automove-evidence-forbidden.json",
    );
    expect(fs.existsSync(forbiddenOutput)).toBe(false);
    const result = run(strengthScript, [
      "--baseline",
      baseline,
      "--candidate",
      candidate,
      "--modes",
      "fast",
      "--variants",
      "Classic",
      "--max-plies",
      "1",
      "--out",
      forbiddenOutput,
    ]);

    expect(result.status).toBe(1);
    expect(result.stderr).toContain("refusing to write");
    expect(fs.existsSync(forbiddenOutput)).toBe(false);
  });
});

function temporaryDirectory(): string {
  const directory = fs.mkdtempSync(
    path.join(os.tmpdir(), "mons-automove-evidence-test-"),
  );
  temporaryDirectories.push(directory);
  return directory;
}

function writeBundle(
  directory: string,
  name: string,
  strategy:
    | "B"
    | "C"
    | "Z"
    | "payload"
    | "none"
    | "throw"
    | "mutating-throw"
    | "throwing-payload"
    | "mutating-invalid-payload"
    | "mutating-preview-throw"
    | "stateful-playfen"
    | "stateful-history-playfen"
    | "stateful-winner-playfen"
    | "invalid-held-playfen"
    | "wrong-held-complete-payload",
  mutates = false,
  mutatesMetadata = false,
  invalidMetadata = false,
  terminalInitial = false,
): string {
  const filePath = path.join(directory, name);
  const suggestion =
    strategy === "none"
      ? "return undefined;"
      : strategy === "throw"
        ? 'throw new Error("synthetic selector failure");'
        : strategy === "mutating-throw"
          ? 'this.state.ply += 1; throw new Error("synthetic selector failure");'
          : strategy === "throwing-payload"
            ? 'return Object.defineProperty({}, "inputFen", { get() { throw new Error("synthetic payload failure"); } });'
            : strategy === "mutating-invalid-payload"
              ? "const game = this; return { get inputFen() { game.state.ply += 1; return null; }, inputs: [], events: [] };"
              : strategy === "payload"
                ? 'return { inputFen: "C", inputs: [{ kind: "synthetic", value: "C" }], events: [] };'
                : `${mutates ? "this.state.ply += 1;" : ""}
    ${mutatesMetadata ? "this.takebacks.push(this.toFen());" : ""}
    ${strategy === "invalid-held-playfen" ? "this.invalidHeldReplay = true;" : ""}
    ${strategy === "wrong-held-complete-payload" ? "this.wrongHeldCompletePayload = true;" : ""}
    const inputFen = ${JSON.stringify(
      strategy === "mutating-preview-throw" ||
        strategy === "stateful-playfen" ||
        strategy === "stateful-history-playfen" ||
        strategy === "stateful-winner-playfen" ||
        strategy === "invalid-held-playfen" ||
        strategy === "wrong-held-complete-payload"
        ? "C"
        : strategy,
    )};
    return {
      inputFen,
      inputs: [{ kind: "synthetic", value: inputFen }],
      events: eventsFor(inputFen),
    };`;
  fs.writeFileSync(
    filePath,
    `export const GameVariant = Object.freeze({ Classic: "Classic" });

function eventsFor(inputFen) {
  return [{ kind: "synthetic-event", inputFen }];
}

function inputFenFromInputs(inputs) {
  if (
    !Array.isArray(inputs) ||
    inputs.length !== 1 ||
    inputs[0]?.kind !== "synthetic" ||
    typeof inputs[0].value !== "string"
  ) return undefined;
  return inputs[0].value;
}

export class Game {
  constructor(options = {}) {
    this.state = {
      variant: options.variant ?? GameVariant.Classic,
      ply: ${terminalInitial ? 2 : 0},
      candidateColor: ${terminalInitial ? '"white"' : "null"},
    };
    this.takebacks = [];
    this.tracking = [];
  }

  static fromFen(fen) {
    try {
      const state = JSON.parse(fen);
      if (
        state.variant !== GameVariant.Classic ||
        !Number.isInteger(state.ply) ||
        ![null, "white", "black"].includes(state.candidateColor)
      ) return undefined;
      const game = new Game({ variant: state.variant });
      game.state = { ...state };
      return game.toFen() === fen ? game : undefined;
    } catch {
      return undefined;
    }
  }

  get activeColor() {
    return this.state.ply % 2 === 0 ? "white" : "black";
  }

  get variant() {
    return this.state.variant;
  }

  get turnNumber() {
    return this.state.ply;
  }

  get scores() {
    return { white: 0, black: 0 };
  }

  get potions() {
    return { white: 0, black: 0 };
  }

  get winner() {
    if (this.winnerOverride !== undefined) return this.winnerOverride;
    return this.state.ply >= 2 && this.state.candidateColor !== null
      ? this.state.candidateColor
      : undefined;
  }

  get historyVerified() {
    return ${invalidMetadata ? '"yes"' : "true"};
  }

  get takebackFens() {
    return [...this.takebacks];
  }

  get trackingEntries() {
    return this.tracking.map((entry) => ({
      ...entry,
      events: [...entry.events],
    }));
  }

  toFen() {
    return JSON.stringify(this.state);
  }

  availableMoveCounts() {
    return { monMoves: 1, manaMoves: 1, actions: 1, potions: 0 };
  }

  canTakeback() {
    return false;
  }

  preview(inputs) {
    ${strategy === "mutating-preview-throw" ? 'this.state.ply += 1; throw new Error("synthetic preview failure");' : ""}
    const inputFen = inputFenFromInputs(inputs);
    if (inputFen !== "B" && inputFen !== "C") {
      return { kind: "invalid", inputFen: inputFen ?? "" };
    }
    return { kind: "complete", inputFen, events: eventsFor(inputFen) };
  }

  suggestMove() {
    ${suggestion}
  }

  playFen(inputFen) {
    ${
      strategy === "stateful-playfen"
        ? "this.plays = (this.plays ?? 0) + 1; if (this.plays > 1) { this.state.ply += 1; }"
        : strategy === "stateful-history-playfen"
          ? 'this.plays = (this.plays ?? 0) + 1; if (this.plays > 1) { this.takebacks.push("drift"); }'
          : strategy === "stateful-winner-playfen"
            ? "this.plays = (this.plays ?? 0) + 1;"
            : ""
    }
    if (inputFen !== "B" && inputFen !== "C") {
      return { kind: "invalid", inputFen };
    }
    const candidateColor =
      inputFen === "C" ? this.activeColor : this.state.candidateColor;
    this.state = {
      ...this.state,
      ply: this.state.ply + 1,
      candidateColor,
    };
    ${strategy === "stateful-winner-playfen" ? 'if (this.plays > 1) this.winnerOverride = "white";' : ""}
    ${strategy === "invalid-held-playfen" ? 'if (this.invalidHeldReplay) return { kind: "invalid", inputFen };' : ""}
    ${strategy === "wrong-held-complete-payload" ? 'if (this.wrongHeldCompletePayload) return { kind: "complete", inputFen: "B", events: eventsFor(inputFen) };' : ""}
    return { kind: "complete", inputFen, events: eventsFor(inputFen) };
  }
}
`,
  );
  return filePath;
}

function run(script: string, arguments_: string[]): SpawnSyncReturns<string> {
  return spawnSync(process.execPath, [script, ...arguments_], {
    cwd: repositoryRoot,
    encoding: "utf8",
  });
}

function readJson(filePath: string) {
  return JSON.parse(fs.readFileSync(filePath, "utf8"));
}

function fileSha256(filePath: string): string {
  return bytesSha256(fs.readFileSync(filePath));
}

function writeStateBankManifest(stateBank: string, bundle: string): string {
  const name = path.basename(stateBank, path.extname(stateBank));
  return writePerformanceStateBankFixture(
    path.dirname(stateBank),
    name,
    fileSha256(bundle),
  ).manifest;
}

function bytesSha256(bytes: NodeJS.ArrayBufferView): string {
  return createHash("sha256").update(bytes).digest("hex");
}
