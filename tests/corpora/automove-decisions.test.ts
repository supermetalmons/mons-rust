import { readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import { afterAll, beforeAll, describe, expect, it, vi } from "vitest";

import {
  Game,
  type MoveSuggestion,
  type PlayResult,
} from "../../src/entrypoints/mons-rules.js";

const PREFERENCES = ["fast", "normal", "pro"] as const;
type Preference = (typeof PREFERENCES)[number];

type DecisionObservation = {
  readonly inputFen: string;
  readonly outputKind: number;
  readonly sourceFenAfterSmartAutomove: string;
  readonly replayOutputKind: number;
  readonly fenAfter: string;
};

type CorpusState = {
  readonly id: string;
  readonly fen: string;
  readonly decisions: Readonly<Record<Preference, DecisionObservation>>;
};

type CorpusManifest = {
  readonly fixedClockNowMs: number;
  readonly corpusFile: string;
  readonly decisionCount: number;
};

function archivedSuggestionKind(
  suggestion: MoveSuggestion | undefined,
): number {
  return suggestion === undefined ? 0 : 3;
}

function archivedPlayResultKind(result: PlayResult): number {
  return result.kind === "invalid" ? 0 : 3;
}

const corpusDirectory = fileURLToPath(
  new URL("../../test-data/automove-decisions/v5/", import.meta.url),
);
const manifest = JSON.parse(
  readFileSync(join(corpusDirectory, "manifest.json"), "utf8"),
) as CorpusManifest;
const states = readFileSync(join(corpusDirectory, manifest.corpusFile), "utf8")
  .trimEnd()
  .split("\n")
  .map((line) => JSON.parse(line) as CorpusState);

describe("automove decision corpus", () => {
  beforeAll(() => {
    vi.spyOn(globalThis.performance, "now").mockReturnValue(
      manifest.fixedClockNowMs,
    );
  });

  afterAll(() => {
    vi.restoreAllMocks();
  });

  it("replays all 39 decisions through the public API", () => {
    let decisionCount = 0;
    for (const state of states) {
      for (const preference of PREFERENCES) {
        decisionCount += 1;
        const expected = state.decisions[preference];
        const game = Game.fromFen(state.fen);
        expect(game, `${state.id} ${preference}: source FEN`).toBeDefined();
        if (game === undefined) continue;

        const before = game.toFen();
        const suggestion = game.suggestMove(preference);
        expect(
          archivedSuggestionKind(suggestion),
          `${state.id} ${preference}: output kind`,
        ).toBe(expected.outputKind);
        expect(suggestion?.inputFen, `${state.id} ${preference}: input`).toBe(
          expected.inputFen,
        );
        if (suggestion !== undefined) {
          expect(
            game.preview(suggestion.inputs),
            `${state.id} ${preference}: suggestion preview`,
          ).toEqual({
            kind: "complete",
            inputFen: suggestion.inputFen,
            events: suggestion.events,
          });
        }
        expect(game.toFen(), `${state.id} ${preference}: source mutation`).toBe(
          before,
        );
        expect(game.toFen(), `${state.id} ${preference}: source state`).toBe(
          expected.sourceFenAfterSmartAutomove,
        );

        const replay = Game.fromFen(state.fen);
        expect(replay, `${state.id} ${preference}: replay FEN`).toBeDefined();
        if (replay === undefined) continue;
        const replayOutput = replay.playFen(expected.inputFen);
        expect(
          archivedPlayResultKind(replayOutput),
          `${state.id} ${preference}: replay output kind`,
        ).toBe(expected.replayOutputKind);
        expect(replay.toFen(), `${state.id} ${preference}: replay state`).toBe(
          expected.fenAfter,
        );
      }
    }
    expect(decisionCount).toBe(manifest.decisionCount);
  }, 300_000);
});
