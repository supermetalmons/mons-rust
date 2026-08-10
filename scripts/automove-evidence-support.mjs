import {
  lstatSync,
  mkdirSync,
  readFileSync,
  realpathSync,
  writeFileSync,
} from "node:fs";
import { createHash } from "node:crypto";
import { tmpdir } from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { isDeepStrictEqual } from "node:util";

export const AUTOMOVE_MODES = Object.freeze(["fast", "normal", "pro"]);
export const INVALID_REASONS = Object.freeze([
  "no-suggestion",
  "source-mutation",
  "selector-error",
  "illegal-replay",
]);

const repositoryRoot = path.resolve(import.meta.dirname, "..");
const protectedTestDataDirectory = path.join(repositoryRoot, "test-data");
const evidenceTargetDirectory = path.join(repositoryRoot, "target");
const temporaryDirectory = path.resolve(tmpdir());
const canonicalProtectedTestDataDirectory = realpathSync(
  protectedTestDataDirectory,
);
const canonicalEvidenceTargetDirectory = canonicalDestination(
  evidenceTargetDirectory,
);
const canonicalTemporaryDirectory = realpathSync(temporaryDirectory);
let importSequence = 0;

export function parseCliOptions(arguments_, allowedOptions) {
  const allowed = new Set([...allowedOptions, "help"]);
  const options = new Map();
  for (let index = 0; index < arguments_.length; index += 1) {
    const argument = arguments_[index];
    if (!argument?.startsWith("--")) {
      throw new Error(`unexpected positional argument: ${String(argument)}`);
    }
    const equalsIndex = argument.indexOf("=");
    const name = argument.slice(2, equalsIndex < 0 ? undefined : equalsIndex);
    if (!allowed.has(name)) {
      throw new Error(`unknown option: --${name}`);
    }
    if (options.has(name)) {
      throw new Error(`duplicate option: --${name}`);
    }
    if (name === "help") {
      if (equalsIndex >= 0) {
        throw new Error("--help does not accept a value");
      }
      options.set(name, true);
      continue;
    }
    const value =
      equalsIndex >= 0
        ? argument.slice(equalsIndex + 1)
        : arguments_[index + 1];
    if (value === undefined || value.startsWith("--")) {
      throw new Error(`missing value for --${name}`);
    }
    if (equalsIndex < 0) index += 1;
    options.set(name, value);
  }
  return options;
}

export function requiredOption(options, name) {
  const value = options.get(name);
  if (typeof value !== "string" || value.length === 0) {
    throw new Error(`missing required option: --${name}`);
  }
  return value;
}

export function positiveIntegerOption(options, name, defaultValue, maximum) {
  if (
    !Number.isSafeInteger(defaultValue) ||
    defaultValue < 1 ||
    !Number.isSafeInteger(maximum) ||
    maximum < defaultValue
  ) {
    throw new Error(`invalid numeric bounds for --${name}`);
  }
  const value = options.get(name);
  if (value === undefined) return defaultValue;
  if (typeof value !== "string" || !/^[1-9]\d*$/.test(value)) {
    throw new Error(`--${name} must be a positive safe integer`);
  }
  const parsed = Number(value);
  if (!Number.isSafeInteger(parsed)) {
    throw new Error(`--${name} must be a positive safe integer`);
  }
  if (parsed > maximum) {
    throw new Error(`--${name} must be at most ${maximum}`);
  }
  return parsed;
}

export function selectModes(value = "all") {
  const requested = parseList(value, "modes");
  if (requested.length === 1 && requested[0] === "all") {
    return [...AUTOMOVE_MODES];
  }
  const requestedSet = new Set(requested);
  for (const mode of requestedSet) {
    if (!AUTOMOVE_MODES.includes(mode)) {
      throw new Error(`unsupported automove mode: ${mode}`);
    }
  }
  return AUTOMOVE_MODES.filter((mode) => requestedSet.has(mode));
}

export function selectVariants(value, baselineBundle, candidateBundle) {
  const requested = parseList(value ?? "all", "variants");
  const baselineVariants = baselineBundle.variants;
  const candidateVariants = new Set(candidateBundle.variants);
  const aliases = new Map(
    baselineBundle.variantEntries.flatMap(([key, item]) => [
      [key, item],
      [item, item],
    ]),
  );
  let variants;
  if (requested.length === 1 && requested[0] === "all") {
    variants = [...baselineVariants];
  } else {
    const requestedVariants = requested.map((item) => {
      const variant = aliases.get(item);
      if (variant === undefined) {
        throw new Error(`unsupported game variant: ${item}`);
      }
      return variant;
    });
    const requestedSet = new Set(requestedVariants);
    variants = baselineVariants.filter((variant) => requestedSet.has(variant));
  }
  for (const variant of variants) {
    if (!candidateVariants.has(variant)) {
      throw new Error(`candidate bundle does not export variant: ${variant}`);
    }
  }
  return variants;
}

export async function loadPublicBundle(filePath, role) {
  const absolutePath = path.resolve(filePath);
  const sourceBeforeImport = readFileSync(absolutePath);
  const sha256 = bytesSha256(sourceBeforeImport);
  const moduleUrl = pathToFileURL(absolutePath);
  importSequence += 1;
  moduleUrl.searchParams.set(
    "automoveEvidenceInstance",
    `${role}-${importSequence}`,
  );
  const namespace = await import(moduleUrl.href);
  let sourceAfterImport;
  try {
    sourceAfterImport = readFileSync(absolutePath);
  } catch {
    throw new Error(`${role} bundle changed while loading`);
  }
  if (!sourceBeforeImport.equals(sourceAfterImport)) {
    throw new Error(`${role} bundle changed while loading`);
  }
  if (typeof namespace.Game !== "function") {
    throw new Error(`${role} bundle does not export Game`);
  }
  if (
    namespace.GameVariant === null ||
    typeof namespace.GameVariant !== "object"
  ) {
    throw new Error(`${role} bundle does not export GameVariant`);
  }
  const variantEntries = Object.entries(namespace.GameVariant);
  if (
    variantEntries.length === 0 ||
    variantEntries.some(
      ([key, value]) => key.length === 0 || typeof value !== "string",
    )
  ) {
    throw new Error(`${role} bundle exports an invalid GameVariant`);
  }
  const variants = [...new Set(variantEntries.map(([, value]) => value))];
  return Object.freeze({
    Game: namespace.Game,
    path: absolutePath,
    sha256,
    variantEntries: Object.freeze(variantEntries),
    variants: Object.freeze(variants),
  });
}

export function initialFen(bundle, variant) {
  const game = new bundle.Game({ variant });
  const fen = game.toFen();
  if (typeof fen !== "string" || fen.length === 0) {
    throw new Error(`bundle produced an invalid initial FEN for ${variant}`);
  }
  return fen;
}

export function inspectSharedFen(firstBundle, secondBundle, fen) {
  try {
    const first = firstBundle.Game.fromFen(fen);
    const second = secondBundle.Game.fromFen(fen);
    const firstState = inspectPublicGame(first);
    const secondState = inspectPublicGame(second);
    if (
      first === undefined ||
      second === undefined ||
      !firstState.ok ||
      !secondState.ok ||
      firstState.fen !== fen ||
      secondState.fen !== fen ||
      !isDeepStrictEqual(firstState.snapshot, secondState.snapshot)
    ) {
      return { ok: false, reason: "illegal-replay" };
    }
    return {
      ok: true,
      activeColor: firstState.activeColor,
      variant: firstState.variant,
      winner: firstState.winner,
    };
  } catch {
    return { ok: false, reason: "illegal-replay" };
  }
}

export function validateStateBankVariants(
  stateBank,
  firstBundle,
  secondBundle,
) {
  for (const state of stateBank.states) {
    if (state.variant === undefined) continue;
    const sharedState = inspectSharedFen(firstBundle, secondBundle, state.fen);
    if (!sharedState.ok) continue;
    if (sharedState.variant !== state.variant) {
      throw new Error(
        `state ${state.id} variant metadata ${state.variant} does not match parsed variant ${sharedState.variant}`,
      );
    }
  }
}

export function runValidatedSuggestion(bundle, validationBundle, fen, mode) {
  let source;
  let validator;
  let sourceBefore;
  let validatorBefore;
  try {
    source = bundle.Game.fromFen(fen);
    validator = validationBundle.Game.fromFen(fen);
    sourceBefore = inspectPublicGame(source);
    validatorBefore = inspectPublicGame(validator);
    if (
      source === undefined ||
      validator === undefined ||
      !sourceBefore.ok ||
      !validatorBefore.ok ||
      sourceBefore.fen !== fen ||
      validatorBefore.fen !== fen ||
      !isDeepStrictEqual(sourceBefore.snapshot, validatorBefore.snapshot)
    ) {
      return invalidSuggestion("illegal-replay");
    }
  } catch {
    return invalidSuggestion("illegal-replay");
  }

  const activeColor = sourceBefore.activeColor;
  let suggestion;
  const started = process.hrtime.bigint();
  try {
    suggestion = source.suggestMove(mode);
  } catch {
    const elapsedMs = elapsedMilliseconds(started);
    const sourceAfterError = inspectPublicGame(source);
    if (
      !sourceAfterError.ok ||
      !isDeepStrictEqual(sourceAfterError.snapshot, sourceBefore.snapshot)
    ) {
      return invalidSuggestion("source-mutation", elapsedMs, activeColor);
    }
    return invalidSuggestion("selector-error", elapsedMs, activeColor);
  }
  const elapsedMs = elapsedMilliseconds(started);

  try {
    const sourceAfterSuggestion = inspectPublicGame(source);
    if (
      !sourceAfterSuggestion.ok ||
      !isDeepStrictEqual(sourceAfterSuggestion.snapshot, sourceBefore.snapshot)
    ) {
      return invalidSuggestion("source-mutation", elapsedMs, activeColor);
    }
  } catch {
    return invalidSuggestion("source-mutation", elapsedMs, activeColor);
  }
  let inputFen = null;
  try {
    if (suggestion === undefined || suggestion === null) {
      return invalidSuggestion("no-suggestion", elapsedMs, activeColor);
    }
    if (typeof suggestion !== "object" || Array.isArray(suggestion)) {
      return invalidSuggestion("illegal-replay", elapsedMs, activeColor);
    }
    const suggestionInputFen = suggestion.inputFen;
    const suggestionInputs = suggestion.inputs;
    const suggestionEvents = suggestion.events;
    if (
      typeof suggestionInputFen !== "string" ||
      suggestionInputFen.length === 0 ||
      !Array.isArray(suggestionInputs) ||
      !Array.isArray(suggestionEvents)
    ) {
      const sourceAfterPayload = inspectPublicGame(source);
      const validatorAfterPayload = inspectPublicGame(validator);
      if (
        !sourceAfterPayload.ok ||
        !validatorAfterPayload.ok ||
        !isDeepStrictEqual(
          sourceAfterPayload.snapshot,
          sourceBefore.snapshot,
        ) ||
        !isDeepStrictEqual(
          validatorAfterPayload.snapshot,
          validatorBefore.snapshot,
        )
      ) {
        return invalidSuggestion(
          "source-mutation",
          elapsedMs,
          activeColor,
          typeof suggestionInputFen === "string" ? suggestionInputFen : null,
        );
      }
      return invalidSuggestion("illegal-replay", elapsedMs, activeColor);
    }
    inputFen = suggestionInputFen;
    const sourceInputs = structuredClone(suggestionInputs);
    const validationInputs = structuredClone(suggestionInputs);
    const expectedEvents = structuredClone(suggestionEvents);
    const sourcePreview = source.preview(sourceInputs);
    const validationPreview = validator.preview(validationInputs);
    const sourceAfterPreview = inspectPublicGame(source);
    const validatorAfterPreview = inspectPublicGame(validator);
    if (
      !sourceAfterPreview.ok ||
      !validatorAfterPreview.ok ||
      !isDeepStrictEqual(sourceAfterPreview.snapshot, sourceBefore.snapshot) ||
      !isDeepStrictEqual(
        validatorAfterPreview.snapshot,
        validatorBefore.snapshot,
      )
    ) {
      return invalidSuggestion(
        "source-mutation",
        elapsedMs,
        activeColor,
        inputFen,
      );
    }
    if (
      !isMatchingCompleteResolution(sourcePreview, inputFen, expectedEvents) ||
      !isMatchingCompleteResolution(validationPreview, inputFen, expectedEvents)
    ) {
      return invalidSuggestion(
        "illegal-replay",
        elapsedMs,
        activeColor,
        inputFen,
      );
    }

    const replay = bundle.Game.fromFen(fen);
    const validationReplay = validationBundle.Game.fromFen(fen);
    const replayBefore = inspectPublicGame(replay);
    const validationReplayBefore = inspectPublicGame(validationReplay);
    if (
      replay === undefined ||
      validationReplay === undefined ||
      !replayBefore.ok ||
      !validationReplayBefore.ok ||
      replayBefore.fen !== fen ||
      validationReplayBefore.fen !== fen
    ) {
      return invalidSuggestion(
        "illegal-replay",
        elapsedMs,
        activeColor,
        inputFen,
      );
    }
    const replayResult = replay.playFen(inputFen);
    const validationResult = validationReplay.playFen(inputFen);
    const replayAfter = inspectPublicGame(replay);
    const validationReplayAfter = inspectPublicGame(validationReplay);
    if (
      !replayAfter.ok ||
      !validationReplayAfter.ok ||
      !isMatchingCompleteResolution(replayResult, inputFen, expectedEvents) ||
      !isMatchingCompleteResolution(
        validationResult,
        inputFen,
        expectedEvents,
      ) ||
      replayAfter.fen === fen ||
      !isDeepStrictEqual(replayAfter.snapshot, validationReplayAfter.snapshot)
    ) {
      return invalidSuggestion(
        "illegal-replay",
        elapsedMs,
        activeColor,
        inputFen,
      );
    }
    return {
      ok: true,
      activeColor,
      elapsedMs,
      inputFen,
      nextFen: replayAfter.fen,
      winner: replayAfter.winner,
    };
  } catch {
    const sourceAfterError = inspectPublicGame(source);
    const validatorAfterError = inspectPublicGame(validator);
    if (
      !sourceAfterError.ok ||
      !validatorAfterError.ok ||
      !isDeepStrictEqual(sourceAfterError.snapshot, sourceBefore.snapshot) ||
      !isDeepStrictEqual(validatorAfterError.snapshot, validatorBefore.snapshot)
    ) {
      return invalidSuggestion(
        "source-mutation",
        elapsedMs,
        activeColor,
        inputFen,
      );
    }
    return invalidSuggestion(
      "illegal-replay",
      elapsedMs,
      activeColor,
      inputFen,
    );
  }
}

export function emptyInvalidCounts() {
  return {
    total: 0,
    noSuggestion: 0,
    sourceMutation: 0,
    selectorError: 0,
    illegalReplay: 0,
  };
}

export function addInvalid(counts, reason) {
  counts.total += 1;
  if (reason === "no-suggestion") counts.noSuggestion += 1;
  if (reason === "source-mutation") counts.sourceMutation += 1;
  if (reason === "selector-error") counts.selectorError += 1;
  if (reason === "illegal-replay") counts.illegalReplay += 1;
}

export function mergeInvalidCounts(items) {
  const merged = emptyInvalidCounts();
  for (const item of items) {
    merged.total += item.total;
    merged.noSuggestion += item.noSuggestion;
    merged.sourceMutation += item.sourceMutation;
    merged.selectorError += item.selectorError;
    merged.illegalReplay += item.illegalReplay;
  }
  return merged;
}

export function timingSummary(values) {
  const sorted = values.map(roundNumber).sort((left, right) => left - right);
  if (sorted.length === 0) {
    return {
      count: 0,
      totalMs: 0,
      meanMs: null,
      medianMs: null,
      p95Ms: null,
      maxMs: null,
    };
  }
  const total = sorted.reduce((sum, value) => sum + value, 0);
  const middle = Math.floor(sorted.length / 2);
  const median =
    sorted.length % 2 === 0
      ? (sorted[middle - 1] + sorted[middle]) / 2
      : sorted[middle];
  const p95Index = Math.max(0, Math.ceil(sorted.length * 0.95) - 1);
  return {
    count: sorted.length,
    totalMs: roundNumber(total),
    meanMs: roundNumber(total / sorted.length),
    medianMs: roundNumber(median),
    p95Ms: sorted[p95Index],
    maxMs: sorted[sorted.length - 1],
  };
}

export function timingRatios(candidate, baseline) {
  return {
    mean: safeRatio(candidate.meanMs, baseline.meanMs),
    median: safeRatio(candidate.medianMs, baseline.medianMs),
    p95: safeRatio(candidate.p95Ms, baseline.p95Ms),
    max: safeRatio(candidate.maxMs, baseline.maxMs),
  };
}

export function roundNumber(value) {
  return Math.round(value * 1_000_000) / 1_000_000;
}

export function readStateBank(filePath) {
  const absolutePath = path.resolve(filePath);
  const sourceBytes = readFileSync(absolutePath);
  const source = sourceBytes.toString("utf8");
  const states = [];
  const ids = new Set();
  for (const [index, line] of source.split(/\r?\n/u).entries()) {
    if (line.trim().length === 0) continue;
    let value;
    try {
      value = JSON.parse(line);
    } catch {
      throw new Error(`invalid JSON on state-bank line ${index + 1}`);
    }
    if (
      value === null ||
      typeof value !== "object" ||
      typeof value.id !== "string" ||
      value.id.length === 0 ||
      typeof value.fen !== "string" ||
      value.fen.length === 0
    ) {
      throw new Error(`invalid state on state-bank line ${index + 1}`);
    }
    if (
      value.variant !== undefined &&
      (typeof value.variant !== "string" || value.variant.length === 0)
    ) {
      throw new Error(
        `invalid variant metadata on state-bank line ${index + 1}`,
      );
    }
    if (ids.has(value.id)) {
      throw new Error(`duplicate state-bank id: ${value.id}`);
    }
    ids.add(value.id);
    states.push(
      Object.freeze({
        id: value.id,
        fen: value.fen,
        ...(value.variant === undefined ? {} : { variant: value.variant }),
      }),
    );
  }
  if (states.length === 0) {
    throw new Error("state bank is empty");
  }
  states.sort((left, right) => compareStrings(left.id, right.id));
  return {
    path: absolutePath,
    sha256: bytesSha256(sourceBytes),
    states,
  };
}

export function preflightJsonReportDestination(filePath) {
  const absolutePath = path.resolve(filePath);
  validateReportDestination(absolutePath);
  mkdirSync(path.dirname(absolutePath), { recursive: true });
  validateReportDestination(absolutePath);
  if (pathEntryExists(absolutePath)) {
    throw existingReportError(absolutePath);
  }
  return absolutePath;
}

export function writeJsonReport(filePath, report) {
  const absolutePath = preflightJsonReportDestination(filePath);
  try {
    writeFileSync(absolutePath, `${JSON.stringify(report, null, 2)}\n`, {
      encoding: "utf8",
      flag: "wx",
    });
  } catch (error) {
    if (error instanceof Error && "code" in error && error.code === "EEXIST") {
      throw existingReportError(absolutePath);
    }
    throw error;
  }
  return absolutePath;
}

function existingReportError(filePath) {
  return new Error(
    `refusing to overwrite existing evidence report: ${filePath}`,
  );
}

function validateReportDestination(filePath) {
  const destination = canonicalDestination(filePath);
  if (
    isWithin(filePath, protectedTestDataDirectory) ||
    isWithin(destination, canonicalProtectedTestDataDirectory)
  ) {
    throw new Error("refusing to write an evidence report under test-data/");
  }
  const isTargetDestination =
    isWithin(filePath, evidenceTargetDirectory) &&
    isWithin(destination, canonicalEvidenceTargetDirectory);
  const isTemporaryDestination =
    (isWithin(filePath, temporaryDirectory) ||
      isWithin(filePath, canonicalTemporaryDirectory)) &&
    isWithin(destination, canonicalTemporaryDirectory);
  if (!isTargetDestination && !isTemporaryDestination) {
    throw new Error(
      "refusing to write an evidence report outside target/ or the OS temporary directory",
    );
  }
}

function parseList(value, optionName) {
  const items = value.split(",").map((item) => item.trim());
  if (items.length === 0 || items.some((item) => item.length === 0)) {
    throw new Error(`--${optionName} must be a non-empty comma-separated list`);
  }
  if (new Set(items).size !== items.length) {
    throw new Error(`--${optionName} contains duplicate values`);
  }
  if (items.includes("all") && items.length !== 1) {
    throw new Error(`--${optionName}=all cannot be combined with other values`);
  }
  return items;
}

function invalidSuggestion(
  reason,
  elapsedMs = null,
  activeColor = null,
  inputFen = null,
) {
  return { ok: false, reason, elapsedMs, activeColor, inputFen };
}

function inspectPublicGame(game) {
  try {
    if (typeof game !== "object" || game === null || Array.isArray(game)) {
      return { ok: false };
    }
    const fen = game.toFen();
    const variant = game.variant;
    const activeColor = game.activeColor;
    const turnNumber = game.turnNumber;
    const scores = game.scores;
    const potions = game.potions;
    const winnerValue = game.winner;
    const historyVerified = game.historyVerified;
    const takebackFens = game.takebackFens;
    const trackingEntries = game.trackingEntries;
    const availableMoveCounts = game.availableMoveCounts();
    const canTakeback = {
      white: game.canTakeback("white"),
      black: game.canTakeback("black"),
    };
    if (
      typeof fen !== "string" ||
      fen.length === 0 ||
      typeof variant !== "string" ||
      variant.length === 0 ||
      !isColor(activeColor) ||
      !isNonnegativeSafeInteger(turnNumber) ||
      !isColorRecord(scores) ||
      !isColorRecord(potions) ||
      (winnerValue !== undefined && !isColor(winnerValue)) ||
      typeof historyVerified !== "boolean" ||
      !Array.isArray(takebackFens) ||
      takebackFens.some((item) => typeof item !== "string") ||
      !isTrackingEntries(trackingEntries) ||
      !isAvailableMoveCounts(availableMoveCounts) ||
      typeof canTakeback.white !== "boolean" ||
      typeof canTakeback.black !== "boolean"
    ) {
      return { ok: false };
    }
    const winner = winnerValue ?? null;
    const snapshot = structuredClone({
      fen,
      variant,
      activeColor,
      turnNumber,
      scores: { white: scores.white, black: scores.black },
      potions: { white: potions.white, black: potions.black },
      winner,
      historyVerified,
      takebackFens,
      trackingEntries,
      availableMoveCounts,
      canTakeback,
    });
    return { ok: true, activeColor, fen, snapshot, variant, winner };
  } catch {
    return { ok: false };
  }
}

function isMatchingCompleteResolution(resolution, inputFen, events) {
  return (
    typeof resolution === "object" &&
    resolution !== null &&
    !Array.isArray(resolution) &&
    resolution.kind === "complete" &&
    resolution.inputFen === inputFen &&
    Array.isArray(resolution.events) &&
    isDeepStrictEqual(resolution.events, events)
  );
}

function isColorRecord(value) {
  return (
    typeof value === "object" &&
    value !== null &&
    !Array.isArray(value) &&
    isNonnegativeSafeInteger(value.white) &&
    isNonnegativeSafeInteger(value.black)
  );
}

function isAvailableMoveCounts(value) {
  return (
    typeof value === "object" &&
    value !== null &&
    !Array.isArray(value) &&
    isNonnegativeSafeInteger(value.monMoves) &&
    isNonnegativeSafeInteger(value.manaMoves) &&
    isNonnegativeSafeInteger(value.actions) &&
    isNonnegativeSafeInteger(value.potions)
  );
}

function isTrackingEntries(value) {
  return (
    Array.isArray(value) &&
    value.every(
      (entry) =>
        typeof entry === "object" &&
        entry !== null &&
        !Array.isArray(entry) &&
        typeof entry.fen === "string" &&
        isColor(entry.color) &&
        Array.isArray(entry.events) &&
        typeof entry.eventsFen === "string",
    )
  );
}

function isNonnegativeSafeInteger(value) {
  return Number.isSafeInteger(value) && value >= 0;
}

function elapsedMilliseconds(started) {
  return roundNumber(Number(process.hrtime.bigint() - started) / 1_000_000);
}

function isColor(value) {
  return value === "white" || value === "black";
}

function safeRatio(numerator, denominator) {
  if (
    typeof numerator !== "number" ||
    typeof denominator !== "number" ||
    denominator === 0
  ) {
    return null;
  }
  return roundNumber(numerator / denominator);
}

function isWithin(filePath, directoryPath) {
  const relative = path.relative(directoryPath, filePath);
  return (
    relative === "" ||
    (!relative.startsWith("..") && !path.isAbsolute(relative))
  );
}

function canonicalDestination(filePath) {
  const suffix = [];
  let existingPath = filePath;
  while (!pathEntryExists(existingPath)) {
    const parent = path.dirname(existingPath);
    if (parent === existingPath) break;
    suffix.unshift(path.basename(existingPath));
    existingPath = parent;
  }
  return path.join(realpathSync(existingPath), ...suffix);
}

function pathEntryExists(filePath) {
  try {
    lstatSync(filePath);
    return true;
  } catch (error) {
    if (error instanceof Error && "code" in error && error.code === "ENOENT") {
      return false;
    }
    throw error;
  }
}

function compareStrings(left, right) {
  if (left < right) return -1;
  if (left > right) return 1;
  return 0;
}

function bytesSha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}
