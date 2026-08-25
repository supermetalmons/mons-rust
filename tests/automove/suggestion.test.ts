import { describe, expect, it, vi } from "vitest";

import { FAST_WORKSPACE_ALLOCATION_FAILED } from "../../src/automove/allocation.js";
import { moveToInputs } from "../../src/automove/bridge.js";
import { FastSearcher } from "../../src/automove/search.js";
import {
  deterministicLegalFallbackInputs,
  suggestMove,
} from "../../src/automove/suggestion.js";
import { ALL_GAME_VARIANTS, GameVariant } from "../../src/engine/board/config.js";
import { inputArrayFen, parseInputArrayFen } from "../../src/engine/codec/input.js";
import { MonsGame } from "../../src/engine/game/mons-game.js";
import { MAX_INPUTS_PER_MOVE } from "../../src/engine/model/domain.js";

const BOMB_DEMON_FEN =
  "0 0 b 0 1 5 0 0 2 y0xn10/n11/n02S0xn08/n11/n11/n02A0xE0xn01d0Bn05/n11/n11/n11/n11/n11";

function expectLegalSourcePure(
  game: MonsGame,
  preference: "fast" | "normal" | "pro",
): string {
  const sourceFen = game.fen();
  const sourceHistory = [...game.takebackFens];
  const sourceTracking = [...game.verboseTrackingEntities];
  const suggestion = suggestMove(game, preference, () => 0);

  expect(suggestion.output.kind, preference).toBe("events");
  const inputs = parseInputArrayFen(suggestion.inputFen);
  expect(inputs, preference).toBeDefined();
  expect(inputs?.length, preference).toBeLessThanOrEqual(MAX_INPUTS_PER_MOVE);
  if (inputs !== undefined) {
    expect(game.fork().processInput(inputs, false, false), preference).toEqual(
      suggestion.output,
    );
  }
  expect(game.fen(), preference).toBe(sourceFen);
  expect(game.takebackFens, preference).toEqual(sourceHistory);
  expect(game.verboseTrackingEntities, preference).toEqual(sourceTracking);
  return suggestion.inputFen;
}

describe("packed automove suggestion", () => {
  it("keeps each preference legal and source-pure", () => {
    for (const preference of ["fast", "normal", "pro"] as const) {
      expectLegalSourcePure(new MonsGame(true, GameVariant.Classic), preference);
    }
  }, 60_000);

  it("supports every game variant", () => {
    for (const variant of ALL_GAME_VARIANTS) {
      expectLegalSourcePure(new MonsGame(true, variant), "fast");
    }
  }, 60_000);

  it("uses the packed node ceiling for each preference", () => {
    const search = vi.spyOn(FastSearcher.prototype, "search");
    try {
      for (const preference of ["fast", "normal", "pro"] as const) {
        expectLegalSourcePure(new MonsGame(true, GameVariant.Classic), preference);
      }
      expect(search.mock.calls.map(([limits]) => limits.maxNodes)).toEqual([
        38_400, 184_000, 2_000_000,
      ]);
      expect(
        search.mock.results.map((result) =>
          result.type === "return" ? result.value.nodes : undefined,
        ),
      ).toEqual([38_400, 184_000, 2_000_000]);
    } finally {
      search.mockRestore();
    }
  }, 60_000);

  it("uses fresh packed searchers and fixed decisions without WeakRef", () => {
    vi.stubGlobal("WeakRef", undefined);
    const search = vi.spyOn(FastSearcher.prototype, "search");
    try {
      const inputs = (["fast", "normal", "pro"] as const).map((preference) =>
        expectLegalSourcePure(new MonsGame(true, GameVariant.Classic), preference),
      );
      expect(inputs).toEqual(["l10,5;l9,4", "l10,5;l9,4", "l10,5;l9,4"]);
      expect(search).toHaveBeenCalledTimes(3);
      expect(new Set(search.mock.instances).size).toBe(3);
    } finally {
      search.mockRestore();
      vi.unstubAllGlobals();
    }
  }, 60_000);

  it("uses the deterministic fallback on timeout", () => {
    let reads = 0;
    const game = new MonsGame(false, GameVariant.Classic);
    const suggestion = suggestMove(game, "normal", () => (reads++ === 0 ? 0 : 1_000));

    expect(suggestion.inputFen).toBe("l10,3;l9,2");
    expect(suggestion.output.kind).toBe("events");
    expect(game.fen()).toBe(new MonsGame(false, GameVariant.Classic).fen());
  });

  it("retains partial search results after a cooperative timeout", () => {
    const originalSearch = Reflect.get(FastSearcher.prototype, "search");
    const partialInputFens: string[] = [];
    let now = 0;
    let searchEntries = 0;
    const search = vi
      .spyOn(FastSearcher.prototype, "search")
      .mockImplementation(function (
        this: FastSearcher,
        ...args: Parameters<FastSearcher["search"]>
      ) {
        searchEntries += 1;
        const [limits, checkTimeout, weights] = args;
        let advanced = false;
        const outcome = originalSearch.call(
          this,
          limits,
          () => {
            if (!advanced) {
              advanced = true;
              now = 1_000;
            }
            return checkTimeout();
          },
          weights,
        );
        partialInputFens.push(inputArrayFen(moveToInputs(outcome.move)));
        return outcome;
      });
    try {
      for (const preference of ["fast", "normal", "pro"] as const) {
        now = 0;
        const game = new MonsGame(true, GameVariant.Classic);
        const fallback = inputArrayFen(deterministicLegalFallbackInputs(game));
        const suggestion = suggestMove(game, preference, () => now);
        expect(suggestion.inputFen).toBe(partialInputFens[partialInputFens.length - 1]);
        expect(suggestion.inputFen).not.toBe(fallback);
      }
      expect(searchEntries).toBe(3);
      expect(partialInputFens.every((inputFen) => inputFen.length > 0)).toBe(true);
    } finally {
      search.mockRestore();
    }
  });

  it("uses the supported search result when bomb-fainted Demon replies are reachable", () => {
    const game = MonsGame.fromFen(BOMB_DEMON_FEN, true);
    expect(game).toBeDefined();
    if (game === undefined) return;
    const search = vi.spyOn(FastSearcher.prototype, "search");
    try {
      const suggestion = suggestMove(game, "pro", () => 0);
      expect(search).toHaveBeenCalledTimes(1);
      const result = search.mock.results[0];
      expect(result?.type).toBe("return");
      if (result?.type !== "return") return;
      expect(result.value.supported).toBe(true);
      expect(suggestion.inputFen).toBe(inputArrayFen(moveToInputs(result.value.move)));
    } finally {
      search.mockRestore();
    }
  }, 15_000);

  it("uses the deterministic fallback on workspace allocation failure", () => {
    const search = vi.spyOn(FastSearcher.prototype, "search").mockImplementation(() => {
      throw FAST_WORKSPACE_ALLOCATION_FAILED;
    });
    try {
      const game = new MonsGame(true, GameVariant.Classic);
      const expected = inputArrayFen(deterministicLegalFallbackInputs(game));
      expect(suggestMove(game, "fast", () => 0).inputFen).toBe(expected);
    } finally {
      search.mockRestore();
    }
  });

  it("uses the deterministic fallback when transposition allocation fails", () => {
    const game = new MonsGame(true, GameVariant.Classic);
    const expected = inputArrayFen(deterministicLegalFallbackInputs(game));
    const NativeInt32Array = globalThis.Int32Array;
    const capacity = 1 << 20;
    let failedAllocations = 0;
    const FailingInt32Array = function (value: unknown): Int32Array {
      if (value === capacity) {
        failedAllocations += 1;
        throw new RangeError("synthetic transposition allocation");
      }
      return Reflect.construct(NativeInt32Array, [value]) as Int32Array;
    } as unknown as Int32ArrayConstructor;
    vi.stubGlobal("WeakRef", undefined);
    vi.stubGlobal("Int32Array", FailingInt32Array);
    try {
      expect(suggestMove(game, "fast", () => 0).inputFen).toBe(expected);
      expect(failedAllocations).toBe(1);
    } finally {
      vi.unstubAllGlobals();
    }
  });

  it("does not hide non-allocation search failures", () => {
    const failure = new RangeError("synthetic search invariant");
    const search = vi.spyOn(FastSearcher.prototype, "search").mockImplementation(() => {
      throw failure;
    });
    try {
      expect(() =>
        suggestMove(new MonsGame(true, GameVariant.Classic), "fast", () => 0),
      ).toThrow(failure);
    } finally {
      search.mockRestore();
    }
  });

  it("finds the first completion at counter capacity", () => {
    const fields = new MonsGame(false, GameVariant.Classic).fen().split(" ");
    fields[8] = String(Number.MAX_SAFE_INTEGER - 1);
    const game = MonsGame.fromFen(fields.join(" "), false);
    expect(game).toBeDefined();
    if (game === undefined) return;
    const opening = parseInputArrayFen("l7,4;l6,4");
    expect(opening).toBeDefined();
    if (opening === undefined) return;
    expect(game.processInput(opening, false, false).kind).toBe("events");

    const fallback = deterministicLegalFallbackInputs(game);
    expect(inputArrayFen(fallback)).toBe("l0,3;l0,2");
    expect(game.fork().processInput(fallback, false, false).kind).toBe("events");
  });

  it("rejects non-finite injected clock values", () => {
    expect(() =>
      suggestMove(new MonsGame(true, GameVariant.Classic), "fast", () => NaN),
    ).toThrow("automove clock must return a finite number");
  });
});
