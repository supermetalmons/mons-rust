#!/usr/bin/env node

import path from "node:path";
import { pathToFileURL } from "node:url";

import {
  addInvalid,
  advanceHeldGames,
  createHeldGame,
  emptyInvalidCounts,
  initialFen,
  inspectSharedFen,
  loadPublicBundle,
  mergeInvalidCounts,
  parseCliOptions,
  positiveIntegerOption,
  preflightJsonReportDestination,
  readStateBank,
  requiredOption,
  roundNumber,
  runValidatedSuggestion,
  selectModes,
  selectVariants,
  timingSummary,
  validateStateBankVariants,
  writeJsonReport,
} from "./automove-evidence-support.mjs";
import { runBundleExecutingCli } from "./evidence/options.mjs";

const helpText = `Usage: node scripts/run-automove-strength.mjs \\
  --baseline <bundle.mjs> --candidate <bundle.mjs> --out <report.json> \\
  [--modes fast,normal,pro] [--variants all|Classic,... | --states states.jsonl] \\
  [--max-plies 256] [--game-driving fresh|held]

Runs one mirrored color-swapped pair for every selected variant or state.
Game driving "fresh" rebuilds each acting game from the shared fen every ply;
"held" keeps one game per bundle alive across the whole game so engines may
reuse per-game state, while every ply is still cross-validated fresh.`;

const GAME_DRIVING_MODES = Object.freeze(["fresh", "held"]);

function gameDrivingOption(value) {
  if (value === undefined) return "fresh";
  if (!GAME_DRIVING_MODES.includes(value)) {
    throw new Error(`--game-driving must be one of ${GAME_DRIVING_MODES.join(", ")}`);
  }
  return value;
}

export async function runStrengthEvidence(configuration) {
  if (configuration.states !== undefined && configuration.variants !== undefined) {
    throw new Error("--states and --variants are mutually exclusive");
  }
  const baseline = await loadPublicBundle(configuration.baseline, "baseline");
  const candidate = await loadPublicBundle(configuration.candidate, "candidate");
  const modes = selectModes(configuration.modes);
  const stateBank =
    configuration.states === undefined ? null : readStateBank(configuration.states);
  const variants =
    stateBank === null
      ? selectVariants(configuration.variants, baseline, candidate)
      : [];
  if (stateBank !== null) {
    validateStateBankVariants(stateBank, baseline, candidate);
  }
  const gameDriving = gameDrivingOption(configuration.gameDriving);
  const maxPlies = configuration.maxPlies;
  const seeds =
    stateBank === null
      ? variants.map((variant) => ({
          fen: initialFen(baseline, variant),
          kind: "variant",
          key: variant,
          variant,
        }))
      : stateBank.states.map((state) => ({
          kind: "state",
          key: state.id,
          fen: state.fen,
          stateId: state.id,
          ...(state.variant === undefined ? {} : { variant: state.variant }),
        }));
  rejectTerminalStrengthSeeds(seeds, baseline, candidate);
  const groups = [];

  for (const mode of modes) {
    for (const seed of seeds) {
      const dimensions =
        seed.kind === "variant"
          ? { mode, variant: seed.variant }
          : {
              mode,
              stateId: seed.stateId,
              ...(seed.variant === undefined ? {} : { variant: seed.variant }),
            };
      const candidateWhite = playStrengthGame({
        baseline,
        candidate,
        candidateColor: "white",
        dimensions,
        gameDriving,
        id: `${mode}/${seed.key}/candidate-white`,
        maxPlies,
        mode,
        expectedVariant: seed.variant,
        startingFen: seed.fen,
      });
      const candidateBlack = playStrengthGame({
        baseline,
        candidate,
        candidateColor: "black",
        dimensions,
        gameDriving,
        id: `${mode}/${seed.key}/candidate-black`,
        maxPlies,
        mode,
        expectedVariant: seed.variant,
        startingFen: seed.fen,
      });
      groups.push({
        dimensions,
        records: [candidateWhite, candidateBlack],
        pairedUnit: summarizePair(
          `${mode}/${seed.key}`,
          dimensions,
          candidateWhite,
          candidateBlack,
        ),
      });
    }
  }

  const records = groups.flatMap((group) => group.records);
  const pairedUnits = groups.map((group) => group.pairedUnit);
  const rows = groups.map((group) =>
    summarizeGames(group.dimensions, group.records, [group.pairedUnit]),
  );
  const modeSummaries = modes.map((mode) =>
    summarizeGames(
      { mode },
      records.filter((record) => record.game.mode === mode),
      pairedUnits.filter((unit) => unit.mode === mode),
    ),
  );

  return {
    schemaVersion: 1,
    kind: "automove-strength",
    baseline: baseline.path,
    baselineSha256: baseline.sha256,
    candidate: candidate.path,
    candidateSha256: candidate.sha256,
    config:
      stateBank === null
        ? {
            modes,
            variants,
            maxPlies,
            gameDriving,
            gamesPerPairedUnit: 2,
          }
        : {
            modes,
            stateIds: stateBank.states.map((state) => state.id),
            maxPlies,
            gameDriving,
            gamesPerPairedUnit: 2,
          },
    ...(stateBank === null
      ? {}
      : {
          stateBank: stateBank.path,
          stateBankSha256: stateBank.sha256,
          states: stateBank.states.length,
        }),
    summary: summarizeGames({}, records, pairedUnits),
    modes: modeSummaries,
    rows,
    pairedUnits,
    games: records.map((record) => record.game),
  };
}

function playStrengthGame({
  baseline,
  candidate,
  candidateColor,
  dimensions,
  gameDriving,
  id,
  maxPlies,
  mode,
  expectedVariant,
  startingFen,
}) {
  const baselineColor = oppositeColor(candidateColor);
  const samples = {
    white: [],
    black: [],
    baseline: [],
    candidate: [],
  };
  let fen = startingFen;
  let winner = null;
  let invalid = null;
  let plies = 0;

  let heldGames = null;
  if (gameDriving === "held") {
    const heldBaseline = createHeldGame(baseline, startingFen);
    const heldCandidate = createHeldGame(candidate, startingFen);
    if (heldBaseline === undefined || heldCandidate === undefined) {
      invalid = {
        reason: "illegal-replay",
        ply: 1,
        actor: null,
        color: null,
        fen,
        inputFen: null,
      };
    } else {
      heldGames = { baseline: heldBaseline, candidate: heldCandidate };
    }
  }

  while (plies < maxPlies && winner === null && invalid === null) {
    const sharedState = inspectSharedFen(baseline, candidate, fen);
    if (!sharedState.ok) {
      invalid = {
        reason: sharedState.reason,
        ply: plies + 1,
        actor: null,
        color: null,
        fen,
        inputFen: null,
      };
      break;
    }
    if (expectedVariant !== undefined && sharedState.variant !== expectedVariant) {
      invalid = {
        reason: "illegal-replay",
        ply: plies + 1,
        actor: null,
        color: null,
        fen,
        inputFen: null,
      };
      break;
    }
    if (sharedState.winner !== null) {
      winner = sharedState.winner;
      break;
    }
    const actor = sharedState.activeColor === candidateColor ? "candidate" : "baseline";
    const actorBundle = actor === "candidate" ? candidate : baseline;
    const validationBundle = actor === "candidate" ? baseline : candidate;
    const suggestion = runValidatedSuggestion(
      actorBundle,
      validationBundle,
      fen,
      mode,
      heldGames === null ? undefined : heldGames[actor],
    );
    if (suggestion.ok && typeof suggestion.elapsedMs === "number") {
      samples[sharedState.activeColor].push(suggestion.elapsedMs);
      samples[actor].push(suggestion.elapsedMs);
    }
    if (!suggestion.ok) {
      invalid = {
        reason: suggestion.reason,
        ply: plies + 1,
        actor,
        color: sharedState.activeColor,
        fen,
        inputFen: suggestion.inputFen,
      };
      break;
    }
    if (
      heldGames !== null &&
      !advanceHeldGames(
        heldGames.candidate,
        heldGames.baseline,
        suggestion.inputFen,
        suggestion.expectedEvents,
        suggestion.nextFen,
      )
    ) {
      invalid = {
        reason: "illegal-replay",
        ply: plies + 1,
        actor,
        color: sharedState.activeColor,
        fen,
        inputFen: suggestion.inputFen,
      };
      break;
    }
    fen = suggestion.nextFen;
    winner = suggestion.winner;
    plies += 1;
  }

  const result =
    invalid !== null
      ? "invalid"
      : winner === null
        ? "cutoff"
        : winner === candidateColor
          ? "win"
          : "loss";
  return {
    game: {
      id,
      ...dimensions,
      candidateColor,
      baselineColor,
      result,
      winner,
      plies,
      reachedMaxPlies: invalid === null && winner === null,
      invalid,
      finalFen: fen,
      timing: summarizeTimingGroups(samples),
    },
    samples,
  };
}

function summarizePair(id, dimensions, first, second) {
  const games = [first.game, second.game];
  const conclusive = games.every((game) =>
    ["win", "draw", "loss"].includes(game.result),
  );
  const candidatePoints = conclusive
    ? games.reduce((points, game) => points + resultPoints(game.result), 0)
    : null;
  const outcome =
    candidatePoints === null
      ? "inconclusive"
      : candidatePoints > 1
        ? "win"
        : candidatePoints < 1
          ? "loss"
          : "draw";
  return {
    id,
    ...dimensions,
    games: games.map((game) => game.id),
    outcome,
    candidatePoints,
    inconclusiveReason: games.some((game) => game.result === "invalid")
      ? "invalid"
      : games.some((game) => game.result === "cutoff")
        ? "cutoff"
        : null,
  };
}

function summarizeGames(dimensions, records, pairedUnits) {
  const games = records.map((record) => record.game);
  const wins = games.filter((game) => game.result === "win").length;
  const draws = games.filter((game) => game.result === "draw").length;
  const losses = games.filter((game) => game.result === "loss").length;
  const cutoffs = games.filter((game) => game.result === "cutoff").length;
  const validGames = wins + draws + losses;
  const points = wins + draws * 0.5;
  const invalids = emptyInvalidCounts();
  for (const game of games) {
    if (game.invalid !== null) addInvalid(invalids, game.invalid.reason);
  }
  const pairCounts = {
    total: pairedUnits.length,
    wins: pairedUnits.filter((unit) => unit.outcome === "win").length,
    draws: pairedUnits.filter((unit) => unit.outcome === "draw").length,
    losses: pairedUnits.filter((unit) => unit.outcome === "loss").length,
    inconclusive: pairedUnits.filter((unit) => unit.outcome === "inconclusive").length,
    invalids: pairedUnits.filter((unit) => unit.inconclusiveReason === "invalid")
      .length,
    cutoffs: pairedUnits.filter((unit) => unit.inconclusiveReason === "cutoff").length,
  };
  const samples = records.reduce(
    (combined, record) => {
      for (const key of Object.keys(combined)) {
        combined[key].push(...record.samples[key]);
      }
      return combined;
    },
    { white: [], black: [], baseline: [], candidate: [] },
  );
  return {
    ...dimensions,
    games: games.length,
    validGames,
    wins,
    draws,
    losses,
    cutoffs,
    points,
    score:
      invalids.total > 0 || cutoffs > 0 || validGames === 0
        ? null
        : roundNumber(points / validGames),
    invalids: mergeInvalidCounts([invalids]),
    pairedUnits: pairCounts,
    timing: summarizeTimingGroups(samples),
  };
}

function summarizeTimingGroups(samples) {
  return {
    white: timingSummary(samples.white),
    black: timingSummary(samples.black),
    baseline: timingSummary(samples.baseline),
    candidate: timingSummary(samples.candidate),
  };
}

function resultPoints(result) {
  if (result === "win") return 1;
  if (result === "draw") return 0.5;
  return 0;
}

function oppositeColor(color) {
  return color === "white" ? "black" : "white";
}

function rejectTerminalStrengthSeeds(seeds, baseline, candidate) {
  for (const seed of seeds) {
    const sharedState = inspectSharedFen(baseline, candidate, seed.fen);
    if (sharedState.ok && sharedState.winner !== null) {
      const description =
        seed.kind === "state"
          ? `state-bank state ${seed.stateId}`
          : `variant ${seed.variant}`;
      throw new Error(
        `${description} is terminal and cannot be used for strength evidence`,
      );
    }
  }
}

export async function main(arguments_ = process.argv.slice(2)) {
  const options = parseCliOptions(arguments_, [
    "baseline",
    "candidate",
    "modes",
    "variants",
    "states",
    "max-plies",
    "game-driving",
    "out",
  ]);
  if (options.has("help")) {
    console.log(helpText);
    return;
  }
  if (options.has("states") && options.has("variants")) {
    throw new Error("--states and --variants are mutually exclusive");
  }
  const configuration = {
    baseline: requiredOption(options, "baseline"),
    candidate: requiredOption(options, "candidate"),
    modes: options.get("modes"),
    variants: options.get("variants"),
    states: options.get("states"),
    maxPlies: positiveIntegerOption(options, "max-plies", 256, 1024),
    gameDriving: gameDrivingOption(options.get("game-driving")),
  };
  const outputPath = preflightJsonReportDestination(requiredOption(options, "out"));
  const report = await runStrengthEvidence(configuration);
  writeJsonReport(outputPath, report);
  console.log(
    `automove strength: ${report.summary.games} games, ` +
      `${report.summary.pairedUnits.total} paired units, ` +
      `${report.summary.invalids.total} invalids, ` +
      `${report.summary.cutoffs} cutoffs -> ${outputPath}`,
  );
  const failures = [];
  if (report.summary.invalids.total > 0) {
    failures.push(`${report.summary.invalids.total} invalid games`);
  }
  if (report.summary.cutoffs > 0) {
    failures.push(`${report.summary.cutoffs} cutoff games`);
  }
  if (failures.length > 0) {
    throw new Error(
      `automove strength contained ${failures.join(" and ")}; report written to ${outputPath}`,
    );
  }
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
