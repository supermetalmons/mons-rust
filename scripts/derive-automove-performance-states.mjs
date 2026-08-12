#!/usr/bin/env node

import { createHash } from "node:crypto";
import path from "node:path";
import { pathToFileURL } from "node:url";

import {
  loadPublicBundle,
  parseCliOptions,
  preflightJsonReportDestination,
  requiredOption,
} from "./automove-evidence-support.mjs";
import {
  createExclusiveEvidencePair,
  readRegularFileArtifact,
} from "./evidence/artifacts.mjs";
import { runBundleExecutingCli } from "./evidence/options.mjs";
import {
  performanceStateBankAlgorithm,
  performanceStateBankColors,
  performanceStateBankSource,
  performanceStatesPerColor,
} from "./evidence/performance-state-bank-schema.mjs";

const sourcePath = path.resolve(
  import.meta.dirname,
  "..",
  performanceStateBankSource.path,
);
const helpText = `Usage: node scripts/derive-automove-performance-states.mjs \\
  --bundle <bundle.mjs> --out <states.jsonl> [--manifest <manifest.json>]

The manifest defaults to <states.jsonl>.manifest.json. Outputs are restricted to
target/ or the OS temporary directory.`;

export async function deriveAutomovePerformanceStates(bundlePath) {
  const bundle = await loadPublicBundle(bundlePath, "public");
  if (typeof bundle.Game.fromFen !== "function") {
    throw new Error("public bundle Game does not expose fromFen");
  }
  const sourceArtifact = readRegularFileArtifact(
    sourcePath,
    "protected complete-games corpus",
  );
  const sourceBytes = sourceArtifact.bytes;
  const sourceSha256 = sourceArtifact.sha256;
  if (
    sourceBytes.byteLength !== performanceStateBankSource.bytes ||
    sourceSha256 !== performanceStateBankSource.sha256
  ) {
    throw new Error("protected complete-games corpus differs from the pinned source");
  }
  const sourceVariants = new Set();
  const candidates = [];
  let excludedTerminal = 0;
  let excludedTooShort = 0;
  let records = 0;

  for (const [index, line] of sourceBytes.toString("utf8").split(/\r?\n/u).entries()) {
    if (line.length === 0) continue;
    const sourceLine = index + 1;
    const record = parseSourceRecord(line, sourceLine);
    records += 1;
    sourceVariants.add(record.gameVariant);
    if (!bundle.variants.includes(record.gameVariant)) {
      throw new Error(
        `public bundle does not export source variant ${record.gameVariant}`,
      );
    }
    const midpointTurnIndex = Math.floor(record.turns.length / 2);
    if (midpointTurnIndex === 0 || midpointTurnIndex >= record.turns.length) {
      excludedTooShort += 1;
      continue;
    }
    const game = replayToMidpoint(bundle.Game, record, midpointTurnIndex, sourceLine);
    const winner = game.winner;
    if (winner !== undefined && !performanceStateBankColors.includes(winner)) {
      throw new Error(`public bundle produced an invalid winner on line ${sourceLine}`);
    }
    if (winner !== undefined) {
      excludedTerminal += 1;
      continue;
    }
    const fen = game.toFen();
    const activeColor = game.activeColor;
    if (
      typeof fen !== "string" ||
      fen.length === 0 ||
      !performanceStateBankColors.includes(activeColor) ||
      game.variant !== record.gameVariant
    ) {
      throw new Error(
        `public bundle produced an invalid midpoint on line ${sourceLine}`,
      );
    }
    assertFenRoundTrip(bundle.Game, fen, record.gameVariant, activeColor, sourceLine);
    candidates.push({
      activeColor,
      fen,
      midpointTurnIndex,
      sha256: sha256(fen),
      sourceLine,
      variant: record.gameVariant,
    });
  }

  const selected = selectStates(candidates, sourceVariants);
  const states = selected.map((candidate) => ({
    id: `midpoint-${candidate.sha256}-${String(candidate.sourceLine).padStart(10, "0")}`,
    fen: candidate.fen,
    variant: candidate.variant,
    activeColor: candidate.activeColor,
    sourceLine: candidate.sourceLine,
    midpointTurnIndex: candidate.midpointTurnIndex,
    stateSha256: candidate.sha256,
  }));
  const stateBank = `${states.map((state) => JSON.stringify(state)).join("\n")}\n`;
  const stateBankBytes = Buffer.from(stateBank, "utf8");
  const algorithmSha256 = sha256(JSON.stringify(performanceStateBankAlgorithm));
  const combinedSourceSha256 = sha256(
    JSON.stringify({
      bundleSha256: bundle.sha256,
      corpusSha256: sourceSha256,
    }),
  );
  const colorCounts = countByColor(selected);

  return {
    stateBank,
    manifest: {
      schemaVersion: 1,
      kind: "automove-performance-state-bank",
      algorithm: {
        ...performanceStateBankAlgorithm,
        sha256: algorithmSha256,
      },
      source: {
        sha256: combinedSourceSha256,
        corpus: {
          path: performanceStateBankSource.path,
          bytes: sourceBytes.byteLength,
          sha256: sourceSha256,
          records,
        },
        bundle: { sha256: bundle.sha256 },
      },
      candidates: {
        eligible: candidates.length,
        excludedTerminal,
        excludedTooShort,
      },
      selection: {
        states: selected.length,
        colors: colorCounts,
        variants: [...sourceVariants].sort(compareStrings),
      },
      output: {
        format: "jsonl",
        encoding: "UTF-8",
        trailingNewline: true,
        bytes: stateBankBytes.byteLength,
        sha256: sha256(stateBankBytes),
      },
    },
  };
}

function parseSourceRecord(line, sourceLine) {
  let value;
  try {
    value = JSON.parse(line);
  } catch {
    throw new Error(`invalid JSON on complete-games line ${sourceLine}`);
  }
  if (
    value === null ||
    typeof value !== "object" ||
    Array.isArray(value) ||
    typeof value.gameVariant !== "string" ||
    value.gameVariant.length === 0 ||
    !Array.isArray(value.turns) ||
    value.turns.some(
      (turn) =>
        !Array.isArray(turn) ||
        turn.length === 0 ||
        turn.some((inputFen) => typeof inputFen !== "string" || inputFen.length === 0),
    )
  ) {
    throw new Error(`invalid complete-games record on line ${sourceLine}`);
  }
  return value;
}

function replayToMidpoint(Game, record, midpointTurnIndex, sourceLine) {
  let game;
  try {
    game = new Game({ variant: record.gameVariant });
    for (let turnIndex = 0; turnIndex < midpointTurnIndex; turnIndex += 1) {
      for (const inputFen of record.turns[turnIndex]) {
        const resolution = game.playFen(inputFen);
        if (
          resolution === null ||
          typeof resolution !== "object" ||
          resolution.kind !== "complete" ||
          resolution.inputFen !== inputFen
        ) {
          throw new Error("incomplete replay");
        }
      }
    }
  } catch {
    throw new Error(`public bundle could not replay complete-games line ${sourceLine}`);
  }
  return game;
}

function assertFenRoundTrip(Game, fen, variant, activeColor, sourceLine) {
  let roundTrip;
  try {
    roundTrip = Game.fromFen(fen);
  } catch {
    throw new Error(`public bundle could not parse midpoint on line ${sourceLine}`);
  }
  if (
    roundTrip === undefined ||
    roundTrip === null ||
    roundTrip.toFen() !== fen ||
    roundTrip.variant !== variant ||
    roundTrip.activeColor !== activeColor
  ) {
    throw new Error(
      `public bundle could not round-trip midpoint on line ${sourceLine}`,
    );
  }
}

function selectStates(candidates, sourceVariants) {
  const ordered = [...candidates].sort(compareCandidates);
  const variantSeeds = new Map();
  for (const candidate of ordered) {
    if (!variantSeeds.has(candidate.variant)) {
      variantSeeds.set(candidate.variant, candidate);
    }
  }
  for (const variant of sourceVariants) {
    if (!variantSeeds.has(variant)) {
      throw new Error(`no eligible midpoint candidate for variant ${variant}`);
    }
  }

  const selected = new Set(variantSeeds.values());
  for (const color of performanceStateBankColors) {
    let selectedForColor = [...selected].filter(
      (candidate) => candidate.activeColor === color,
    ).length;
    if (selectedForColor > performanceStatesPerColor) {
      throw new Error(`variant seeds exceed the ${color} state quota`);
    }
    for (const candidate of ordered) {
      if (selectedForColor === performanceStatesPerColor) break;
      if (candidate.activeColor !== color || selected.has(candidate)) continue;
      selected.add(candidate);
      selectedForColor += 1;
    }
    if (selectedForColor !== performanceStatesPerColor) {
      throw new Error(`not enough eligible ${color} midpoint candidates`);
    }
  }

  const result = ordered.filter((candidate) => selected.has(candidate));
  const colorCounts = countByColor(result);
  if (
    result.length !== performanceStatesPerColor * performanceStateBankColors.length ||
    performanceStateBankColors.some(
      (color) => colorCounts[color] !== performanceStatesPerColor,
    )
  ) {
    throw new Error("midpoint selection did not produce the required state bank");
  }
  return result;
}

function countByColor(candidates) {
  return Object.fromEntries(
    performanceStateBankColors.map((color) => [
      color,
      candidates.filter((candidate) => candidate.activeColor === color).length,
    ]),
  );
}

function compareCandidates(left, right) {
  return (
    compareStrings(left.sha256, right.sha256) || left.sourceLine - right.sourceLine
  );
}

function compareStrings(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

export async function main(arguments_ = process.argv.slice(2)) {
  const options = parseCliOptions(arguments_, ["bundle", "out", "manifest"]);
  if (options.has("help")) {
    console.log(helpText);
    return;
  }
  const bundlePath = requiredOption(options, "bundle");
  const requestedOutputPath = path.resolve(requiredOption(options, "out"));
  const requestedManifestPath = path.resolve(
    options.get("manifest") ?? `${requestedOutputPath}.manifest.json`,
  );
  if (requestedOutputPath === requestedManifestPath) {
    throw new Error("state-bank and manifest destinations must be different");
  }
  const outputPath = preflightJsonReportDestination(requestedOutputPath);
  const manifestPath = preflightJsonReportDestination(requestedManifestPath);
  const result = await deriveAutomovePerformanceStates(bundlePath);
  const [stateBankFile, manifestFile] = createExclusiveEvidencePair(
    outputPath,
    result.stateBank,
    manifestPath,
    `${JSON.stringify(result.manifest, null, 2)}\n`,
  );
  console.log(
    `automove performance states: ${result.manifest.selection.states} states -> ${stateBankFile.path}; manifest -> ${manifestFile.path}`,
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
