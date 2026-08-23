import { isDeepStrictEqual } from "node:util";

import { roundNumber } from "./metrics.mjs";

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

export function validateStateBankVariants(stateBank, firstBundle, secondBundle) {
  for (const state of stateBank.states) {
    if (state.variant === undefined && state.activeColor === undefined) continue;
    const sharedState = inspectSharedFen(firstBundle, secondBundle, state.fen);
    if (!sharedState.ok) continue;
    if (state.variant !== undefined && sharedState.variant !== state.variant) {
      throw new Error(
        `state ${state.id} variant metadata ${state.variant} does not match parsed variant ${sharedState.variant}`,
      );
    }
    if (
      state.activeColor !== undefined &&
      sharedState.activeColor !== state.activeColor
    ) {
      throw new Error(
        `state ${state.id} active-color metadata ${state.activeColor} does not match parsed active color ${sharedState.activeColor}`,
      );
    }
  }
}

export function runValidatedSuggestion(
  bundle,
  validationBundle,
  fen,
  mode,
  heldSource,
) {
  let source;
  let validator;
  let sourceBefore;
  let validatorBefore;
  try {
    source = heldSource === undefined ? bundle.Game.fromFen(fen) : heldSource;
    validator = validationBundle.Game.fromFen(fen);
    sourceBefore = inspectPublicGame(source);
    validatorBefore = inspectPublicGame(validator);
    const sharedSnapshotsMatch =
      sourceBefore.ok &&
      validatorBefore.ok &&
      (heldSource === undefined
        ? isDeepStrictEqual(sourceBefore.snapshot, validatorBefore.snapshot)
        : isDeepStrictEqual(
            positionSnapshot(sourceBefore.snapshot),
            positionSnapshot(validatorBefore.snapshot),
          ));
    if (
      source === undefined ||
      validator === undefined ||
      sourceBefore.fen !== fen ||
      validatorBefore.fen !== fen ||
      !sharedSnapshotsMatch
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
        !isDeepStrictEqual(sourceAfterPayload.snapshot, sourceBefore.snapshot) ||
        !isDeepStrictEqual(validatorAfterPayload.snapshot, validatorBefore.snapshot)
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
      !isDeepStrictEqual(validatorAfterPreview.snapshot, validatorBefore.snapshot)
    ) {
      return invalidSuggestion("source-mutation", elapsedMs, activeColor, inputFen);
    }
    if (
      !isMatchingCompleteResolution(sourcePreview, inputFen, expectedEvents) ||
      !isMatchingCompleteResolution(validationPreview, inputFen, expectedEvents)
    ) {
      return invalidSuggestion("illegal-replay", elapsedMs, activeColor, inputFen);
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
      return invalidSuggestion("illegal-replay", elapsedMs, activeColor, inputFen);
    }
    const replayResult = replay.playFen(inputFen);
    const validationResult = validationReplay.playFen(inputFen);
    const replayAfter = inspectPublicGame(replay);
    const validationReplayAfter = inspectPublicGame(validationReplay);
    if (
      !replayAfter.ok ||
      !validationReplayAfter.ok ||
      !isMatchingCompleteResolution(replayResult, inputFen, expectedEvents) ||
      !isMatchingCompleteResolution(validationResult, inputFen, expectedEvents) ||
      replayAfter.fen === fen ||
      !isDeepStrictEqual(replayAfter.snapshot, validationReplayAfter.snapshot)
    ) {
      return invalidSuggestion("illegal-replay", elapsedMs, activeColor, inputFen);
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
      return invalidSuggestion("source-mutation", elapsedMs, activeColor, inputFen);
    }
    return invalidSuggestion("illegal-replay", elapsedMs, activeColor, inputFen);
  }
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

function positionSnapshot(snapshot) {
  return {
    fen: snapshot.fen,
    variant: snapshot.variant,
    activeColor: snapshot.activeColor,
    turnNumber: snapshot.turnNumber,
    scores: snapshot.scores,
    potions: snapshot.potions,
    winner: snapshot.winner,
    availableMoveCounts: snapshot.availableMoveCounts,
  };
}

function isMatchingCompleteResolution(resolution, inputFen, events) {
  return (
    typeof resolution === "object" &&
    resolution !== null &&
    !Array.isArray(resolution) &&
    resolution.kind === "complete" &&
    resolution.inputFen === inputFen &&
    Array.isArray(resolution.events) &&
    isDeepStrictEqual(structuredClone(resolution.events), events)
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

export function createHeldGame(bundle, fen) {
  try {
    const game = bundle.Game.fromFen(fen);
    if (game === undefined || game === null) return undefined;
    const inspected = inspectPublicGame(game);
    if (!inspected.ok || inspected.fen !== fen) return undefined;
    return game;
  } catch {
    return undefined;
  }
}

export function advanceHeldGame(game, inputFen, expectedFen) {
  try {
    const result = game.playFen(inputFen);
    if (
      result === undefined ||
      result === null ||
      typeof result !== "object" ||
      result.kind === "invalid-input"
    ) {
      return false;
    }
    return game.toFen() === expectedFen;
  } catch {
    return false;
  }
}
