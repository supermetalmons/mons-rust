import { describe, expect, it, vi } from "vitest";

import { AutomoveEngine } from "../../src/automove/automove-engine.js";
import { selectPackedFastSelection } from "../../src/automove/fast/index.js";
import { FastSearcher } from "../../src/automove/fast/search.js";
import {
  suggestMove,
  type AutomoveSuggestion,
} from "../../src/automove/runtime.js";
import {
  deterministicLegalFallbackInputs,
  randomAutomove,
  randomIndex,
} from "../../src/automove/runtime/input-selection.js";
import {
  PRODUCTION_FALLBACK_GUARDS,
  PRODUCTION_PRESELECTION_GUARDS,
} from "../../src/automove/runtime/production-policy.js";
import { selectSearchInputs } from "../../src/automove/runtime/search-selection.js";
import { automoveConfigForGame } from "../../src/automove/selector-config.js";
import { GameVariant } from "../../src/engine/config.js";
import { MAX_INPUTS_PER_MOVE } from "../../src/engine/domain.js";
import { inputArrayFen, parseInputArrayFen } from "../../src/engine/fen.js";
import { MonsGame } from "../../src/engine/game.js";

type StrategicPreference = Parameters<typeof suggestMove>[2];

const LATE_UNSUPPORTED_FEN =
  "0 0 b 0 1 5 0 0 2 y0xn10/n11/n02S0xn08/n11/n11/n02A0xE0xn01d0Bn05/n11/n11/n11/n11/n11";

function expectSourcePureSuggestion(
  preference: StrategicPreference,
  engine = new AutomoveEngine(),
): AutomoveSuggestion {
  const game = new MonsGame(true, GameVariant.Classic);
  const sourceFen = game.fen();
  const sourceHistory = [...game.takebackFens];
  const sourceTracking = [...game.verboseTrackingEntities];

  const suggestion = engine.run((execution) =>
    suggestMove(execution, game, preference),
  );

  expect(game.fen()).toBe(sourceFen);
  expect(game.takebackFens).toEqual(sourceHistory);
  expect(game.verboseTrackingEntities).toEqual(sourceTracking);
  expect(suggestion.output.kind).toBe("events");

  const inputs = parseInputArrayFen(suggestion.inputFen);
  expect(inputs).toBeDefined();
  if (inputs === undefined) return suggestion;
  const applied = game.copy().processInput(inputs, false, false);
  expect(applied).toEqual(suggestion.output);
  expect(game.fen()).toBe(sourceFen);
  return suggestion;
}

function expectSourcePureRandomFallback(engine = new AutomoveEngine()) {
  const game = new MonsGame(true, GameVariant.Classic);
  const sourceFen = game.fen();
  const config = automoveConfigForGame(game, "fast");
  const inputs = engine.run((execution) =>
    selectSearchInputs(execution, game, {
      ...config,
      search: { ...config.search, rootBranchLimit: 0 },
    }),
  );
  expect(game.fen()).toBe(sourceFen);
  expect(game.copy().processInput(inputs, false, false).kind).toBe("events");
  return inputs;
}

describe("production automove runtime", () => {
  it("keeps strategic suggestions and the internal fallback source-pure", () => {
    const strategic = expectSourcePureSuggestion("fast");
    const random = expectSourcePureRandomFallback();

    expect(strategic.inputFen).not.toBe("");
    expect(random).not.toEqual([]);
  });

  it("routes public Fast and Normal through their packed node ceilings", () => {
    const engine = new AutomoveEngine({ clock: () => 0 });
    const search = vi.spyOn(FastSearcher.prototype, "search");
    try {
      expect(expectSourcePureSuggestion("fast", engine).output.kind).toBe(
        "events",
      );
      expect(expectSourcePureSuggestion("normal", engine).output.kind).toBe(
        "events",
      );
      expect(search.mock.calls.map(([limits]) => limits.maxNodes)).toEqual([
        30_000, 150_000,
      ]);
      expect(
        search.mock.results.map((result) =>
          result.type === "return" ? result.value.nodes : undefined,
        ),
      ).toEqual([30_000, 150_000]);
    } finally {
      search.mockRestore();
    }
  });

  it("falls back to the canonical selector for unsupported packed states", () => {
    for (const preference of ["fast", "normal"] as const) {
      const game = MonsGame.fromFen(LATE_UNSUPPORTED_FEN, true);
      expect(game, preference).toBeDefined();
      if (game === undefined) continue;
      const sourceFen = game.fen();
      const engineOptions = {
        clock: () => 0,
        randomSource: { nextUint32: () => 0 },
      };
      const packed = new AutomoveEngine(engineOptions).run((execution) =>
        selectPackedFastSelection(execution, game, preference),
      );
      expect(packed.kind, preference).toBe("unsupported");
      if (packed.kind === "unsupported") {
        expect(
          game.fork().processInput(packed.fallbackInputs, false, false).kind,
          preference,
        ).toBe("events");
      }

      const config = automoveConfigForGame(game, preference);
      const canonical = new AutomoveEngine(engineOptions).run((execution) =>
        selectSearchInputs(execution, game, config),
      );
      expect(canonical.length, preference).toBeGreaterThan(0);
      const suggestion = new AutomoveEngine(engineOptions).run((execution) =>
        suggestMove(execution, game, preference),
      );

      expect(suggestion.output.kind, preference).toBe("events");
      expect(suggestion.inputFen, preference).toBe(inputArrayFen(canonical));
      expect(game.fen(), preference).toBe(sourceFen);
    }
  });

  it("uses the canonical Fast and Normal selectors without WeakRef", () => {
    vi.stubGlobal("WeakRef", undefined);
    const search = vi.spyOn(FastSearcher.prototype, "search");
    try {
      for (const preference of ["fast", "normal"] as const) {
        const game = new MonsGame(true, GameVariant.Classic);
        const sourceFen = game.fen();
        const sourceHistory = [...game.takebackFens];
        const sourceTracking = [...game.verboseTrackingEntities];
        const engineOptions = {
          clock: () => 0,
          randomSource: { nextUint32: () => 0 },
        };
        const config = automoveConfigForGame(game, preference);
        const canonical = new AutomoveEngine(engineOptions).run((execution) =>
          selectSearchInputs(execution, game, config),
        );
        const suggestion = new AutomoveEngine(engineOptions).run((execution) =>
          suggestMove(execution, game, preference),
        );

        expect(suggestion.inputFen, preference).toBe(inputArrayFen(canonical));
        expect(suggestion.output.kind, preference).toBe("events");
        expect(
          game.fork().processInput(canonical, false, false),
          preference,
        ).toEqual(suggestion.output);
        expect(game.fen(), preference).toBe(sourceFen);
        expect(game.takebackFens, preference).toEqual(sourceHistory);
        expect(game.verboseTrackingEntities, preference).toEqual(
          sourceTracking,
        );
      }
      expect(search).not.toHaveBeenCalled();
    } finally {
      search.mockRestore();
      vi.unstubAllGlobals();
    }
  });

  it("draws random suggestions from the engine-owned uint32 source", () => {
    let draws = 0;
    const engine = new AutomoveEngine({
      randomSource: {
        nextUint32(): number {
          draws += 1;
          return 0;
        },
      },
    });

    const inputs = expectSourcePureRandomFallback(engine);

    expect(draws).toBeGreaterThan(0);
    expect(inputs).not.toEqual([]);
  });

  it("rejects the incomplete uint32 tail before mapping an index", () => {
    const draws = [0xffff_ffff, 0];
    const engine = new AutomoveEngine({
      randomSource: {
        nextUint32(): number {
          const value = draws.shift();
          if (value === undefined) throw new Error("unexpected random draw");
          return value;
        },
      },
    });

    expect(engine.run((execution) => randomIndex(execution, 3))).toBe(0);
    expect(draws).toEqual([]);
  });

  it("rejects values outside the injected random source's uint32 contract", () => {
    for (const invalid of [-1, 0x1_0000_0000, Number.NaN]) {
      const engine = new AutomoveEngine({
        randomSource: { nextUint32: () => invalid },
      });
      const game = new MonsGame(false, GameVariant.Classic);

      expect(() =>
        engine.run((execution) => randomAutomove(execution, game.fork())),
      ).toThrow("automove random source must return a uint32");
    }
  });

  it("returns the deterministic legal fallback when the fixed clock expires", () => {
    let clockReads = 0;
    const engine = new AutomoveEngine({
      clock: () => (clockReads++ === 0 ? 0 : 1_000),
    });
    const timedOutGame = new MonsGame(false, GameVariant.Classic);
    const sourceFen = timedOutGame.fen();

    const fallback = engine.run((execution) =>
      suggestMove(execution, timedOutGame, "normal"),
    );

    expect(timedOutGame.fen()).toBe(sourceFen);
    expect(fallback).toEqual({
      inputFen: "l10,3;l9,2",
      output: {
        kind: "events",
        events: [
          {
            kind: "mon-move",
            item: {
              kind: "mon",
              mon: { kind: "demon", color: "white", cooldown: 0 },
            },
            from: { i: 10, j: 3 },
            to: { i: 9, j: 2 },
          },
        ],
      },
    });

    const recovered = expectSourcePureSuggestion("fast", engine);
    expect(recovered.output.kind).toBe("events");
  });

  it("short-circuits to the first applicable fallback at counter capacity", () => {
    const fields = new MonsGame(false, GameVariant.Classic).fen().split(" ");
    fields[8] = String(Number.MAX_SAFE_INTEGER - 1);
    const game = MonsGame.fromFen(fields.join(" "), false);
    expect(game).toBeDefined();
    if (game === undefined) return;

    const openingManaMove = parseInputArrayFen("l7,4;l6,4");
    expect(openingManaMove).toBeDefined();
    if (openingManaMove === undefined) return;
    expect(game.processInput(openingManaMove, false, false).kind).toBe(
      "events",
    );
    expect(game.turnNumber).toBe(Number.MAX_SAFE_INTEGER);

    const fallback = deterministicLegalFallbackInputs(game);
    expect(inputArrayFen(fallback)).toBe("l0,3;l0,2");
    expect(game.fork().processInput(fallback, false, false).kind).toBe(
      "events",
    );
  });

  it("keeps every generated input chain within the engine codec limit", () => {
    expect(expectSourcePureRandomFallback().length).toBeLessThanOrEqual(
      MAX_INPUTS_PER_MOVE,
    );
    for (const preference of ["fast", "normal", "pro"] as const) {
      const suggestion = expectSourcePureSuggestion(preference);
      const inputs = parseInputArrayFen(suggestion.inputFen);
      expect(inputs).toBeDefined();
      expect(inputs?.length).toBeLessThanOrEqual(MAX_INPUTS_PER_MOVE);
    }
  });

  it("selects the same strategic move with cold and warm engine caches", () => {
    const newEngine = (): AutomoveEngine => {
      let now = 0;
      return new AutomoveEngine({
        clock: () => now++,
        randomSource: { nextUint32: () => 0 },
      });
    };
    for (const preference of ["normal", "pro"] as const) {
      const game = new MonsGame(true, GameVariant.Classic);
      const warmEngine = newEngine();
      const first = warmEngine.run((execution) =>
        suggestMove(execution, game, preference),
      );
      const warm = warmEngine.run((execution) =>
        suggestMove(execution, game, preference),
      );
      const cold = newEngine().run((execution) =>
        suggestMove(execution, game, preference),
      );

      expect(warm).toEqual(first);
      expect(cold).toEqual(first);
    }
  });

  it("publishes stable production guard IDs in evaluation order", () => {
    expect(PRODUCTION_PRESELECTION_GUARDS.map(({ id }) => id)).toEqual([
      "early-white-fallback",
      "score-window-tactical-fallback",
      "unconditional-black-fallback",
    ]);
    expect(PRODUCTION_FALLBACK_GUARDS.map(({ id }) => id)).toEqual([
      "white-early-baseline-fallback",
      "white-nonnegative-deny-fallback",
      "white-negative-deny-fallback",
      "white-confirm-baseline-tiebreak",
      "white-confirm-baseline-better",
      "late-black-fallback",
    ]);
    expect(Object.isFrozen(PRODUCTION_PRESELECTION_GUARDS)).toBe(true);
    expect(Object.isFrozen(PRODUCTION_FALLBACK_GUARDS)).toBe(true);
    expect(PRODUCTION_PRESELECTION_GUARDS.every(Object.isFrozen)).toBe(true);
    expect(PRODUCTION_FALLBACK_GUARDS.every(Object.isFrozen)).toBe(true);
  });
});
