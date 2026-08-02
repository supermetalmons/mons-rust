import { describe, expect, it, vi } from "vitest";

import { AutomoveEngine } from "../../src/automove/automove-engine.js";
import { SearchSession } from "../../src/automove/deadline.js";
import {
  AutomoveCacheScope,
  createAutomoveExecutionContext,
} from "../../src/automove/execution-context.js";
import {
  FastSearcherPool,
  PRO_FAST_PROFILE,
  selectFastInputs,
  selectFastSelection,
  selectProFastInputs,
  selectProFastSelection,
  type FastProfile,
} from "../../src/automove/fast/index.js";
import {
  DEFAULT_WEIGHTS,
  type EvalWeights,
} from "../../src/automove/fast/evaluate.js";
import {
  PRODUCTION_SELECTOR_BUDGET_MS,
  selectProductionInputsWithDeadline,
} from "../../src/automove/runtime/deadline-selection.js";
import {
  FastSearcher,
  MAX_SEARCH_DEPTH,
  type SearchLimits,
  type SearchOutcome,
  type SearchTuning,
} from "../../src/automove/fast/search.js";
import { suggestMove } from "../../src/automove/runtime.js";
import { GameVariant } from "../../src/engine/config.js";
import type { Input } from "../../src/engine/domain.js";
import { inputArrayFen, parseInputArrayFen } from "../../src/engine/fen.js";
import { MonsGame } from "../../src/engine/game.js";
import { createTestAutomoveExecutionContext } from "./execution-context.test-helper.js";
import { TINY_PROFILE } from "./fast.test-helper.js";

const TEST_RANDOM_SOURCE = Object.freeze({
  nextUint32: () => 0,
});

const LATE_UNSUPPORTED_FEN =
  "0 0 b 0 1 5 0 0 2 y0xn10/n11/n02S0xn08/n11/n11/n02A0xE0xn01d0Bn05/n11/n11/n11/n11/n11";

function lateUnsupportedGame(): MonsGame {
  const game = MonsGame.fromFen(LATE_UNSUPPORTED_FEN, false);
  if (game === undefined) throw new Error("late-unsupported FEN must parse");
  return game;
}

function manualClock(initialTime = 0): {
  readonly read: () => number;
  set(time: number): void;
} {
  let currentTime = initialTime;
  return {
    read: () => currentTime,
    set(time: number): void {
      currentTime = time;
    },
  };
}

class AfterSearchFastSearcher extends FastSearcher {
  readonly #afterSearch: () => void;

  public constructor(afterSearch: () => void) {
    super();
    this.#afterSearch = afterSearch;
  }

  public override search(
    limits: SearchLimits,
    checkTimeout: () => boolean,
    weights: EvalWeights = DEFAULT_WEIGHTS,
  ): SearchOutcome {
    const outcome = super.search(limits, checkTimeout, weights);
    this.#afterSearch();
    return outcome;
  }
}

describe("packed-state search lifecycle", () => {
  it("does not retain the fast workspace in the engine cache", () => {
    const engine = new AutomoveEngine({ clock: () => 0 });
    const game = new MonsGame(true, GameVariant.Classic);

    engine.run((execution) => {
      const inputs = selectFastInputs(execution, game, TINY_PROFILE);
      expect(inputs).toBeDefined();
      expect(execution.caches.session.cacheCount).toBe(0);
      expect(execution.caches.engine.cacheCount).toBe(0);
    });

    engine.run((execution) => {
      expect(execution.caches.engine.cacheCount).toBe(0);
    });
  });

  it("runs unpooled searches when weak retention is disabled", () => {
    let createdSearchers = 0;
    const pool = new FastSearcherPool({
      createSearcher: () => {
        createdSearchers += 1;
        return new FastSearcher();
      },
      weakRefFactory: false,
    });
    const engine = new AutomoveEngine({ clock: () => 0 });
    const game = new MonsGame(true, GameVariant.Classic);
    for (let attempt = 0; attempt < 2; attempt += 1) {
      const inputs = engine.run((execution) =>
        selectFastInputs(execution, game, TINY_PROFILE, pool),
      );
      expect(inputs).toBeDefined();
      expect(inputs?.length).toBeGreaterThan(0);
    }
    expect(createdSearchers).toBe(2);
  });

  it("keeps nested fast selections independent and applicable", () => {
    const nestedEngine = new AutomoveEngine({ clock: () => 0 });
    const outerGame = new MonsGame(true, GameVariant.Classic);
    const nestedGame = new MonsGame(true, GameVariant.Classic);
    expect(
      nestedGame.processInput(
        [
          { kind: "location", location: { i: 10, j: 3 } },
          { kind: "location", location: { i: 9, j: 2 } },
        ],
        false,
        false,
      ).kind,
    ).toBe("events");
    const outerProfile: FastProfile = {
      ...TINY_PROFILE,
      maxDepth: 2,
      maxNodes: 100,
    };
    const nestedProfile: FastProfile = {
      ...TINY_PROFILE,
      maxNodes: 10,
    };
    let nestedInputs: Input[] | undefined;
    let nested = false;
    let createdSearchers = 0;
    const pool = new FastSearcherPool({
      createSearcher: () => {
        createdSearchers += 1;
        return createdSearchers === 1
          ? new AfterSearchFastSearcher(() => {
              if (nested) return;
              nested = true;
              nestedInputs = nestedEngine.run((nestedExecution) =>
                selectFastInputs(
                  nestedExecution,
                  nestedGame,
                  nestedProfile,
                  pool,
                ),
              );
            })
          : new FastSearcher();
      },
      weakRefFactory: (searcher) => ({
        deref: () => searcher,
      }),
    });
    const outerEngine = new AutomoveEngine({ clock: () => 0 });
    const outerInputs = outerEngine.run((execution) => {
      const inputs = selectFastInputs(execution, outerGame, outerProfile, pool);
      expect(execution.caches.session.cacheCount).toBe(0);
      expect(execution.caches.engine.cacheCount).toBe(0);
      return inputs;
    });

    const expectedNestedEngine = new AutomoveEngine({ clock: () => 0 });
    const expectedNestedInputs = expectedNestedEngine.run((execution) =>
      selectFastInputs(execution, nestedGame, nestedProfile),
    );
    const expectedOuterEngine = new AutomoveEngine({ clock: () => 0 });
    const expectedOuterInputs = expectedOuterEngine.run((execution) =>
      selectFastInputs(execution, outerGame, outerProfile),
    );

    expect(nested).toBe(true);
    expect(createdSearchers).toBe(2);
    expect(outerInputs).toBeDefined();
    expect(nestedInputs).toBeDefined();
    expect(outerInputs).toEqual(expectedOuterInputs);
    expect(nestedInputs).toEqual(expectedNestedInputs);
    if (outerInputs === undefined || nestedInputs === undefined) return;
    expect(outerGame.fork().processInput(outerInputs, false, false).kind).toBe(
      "events",
    );
    expect(
      nestedGame.fork().processInput(nestedInputs, false, false).kind,
    ).toBe("events");
    outerEngine.run((execution) => {
      expect(execution.caches.session.cacheCount).toBe(0);
      expect(execution.caches.engine.cacheCount).toBe(0);
    });
    nestedEngine.run((execution) => {
      expect(execution.caches.session.cacheCount).toBe(0);
      expect(execution.caches.engine.cacheCount).toBe(0);
    });
  });

  it("honors an already-expired enclosing deadline", () => {
    let now = 0;
    const context = createTestAutomoveExecutionContext(() => now);
    const { session } = context;
    const game = new MonsGame(true, GameVariant.Classic);

    const inputs = session.withDeadlineIfAbsent(1, () => {
      now = 1;
      const selected = selectFastInputs(context, game, TINY_PROFILE);
      expect(session.cancelled).toBe(true);
      expect(context.caches.session.cacheCount).toBe(0);
      expect(context.caches.engine.cacheCount).toBe(0);
      return selected;
    });

    expect(inputs).toEqual([]);
    expect(session.takePreviousTimeout()).toBe(true);
  });

  it("does not record an owned fast deadline while retaining its result", () => {
    const clock = manualClock();
    let searchCompleted = false;
    const pool = new FastSearcherPool({
      createSearcher: () =>
        new AfterSearchFastSearcher(() => {
          searchCompleted = true;
          clock.set(TINY_PROFILE.budgetMs);
        }),
      weakRefFactory: false,
    });
    const context = createTestAutomoveExecutionContext(clock.read);
    const { session } = context;
    const game = new MonsGame(true, GameVariant.Classic);
    const inputs = selectFastInputs(context, game, TINY_PROFILE, pool);

    expect(searchCompleted).toBe(true);
    expect(inputs).toBeDefined();
    expect(inputs?.length).toBeGreaterThan(0);
    expect(session.takePreviousTimeout()).toBe(false);
  });

  it("preserves unrelated engine caches after an owned fast timeout", () => {
    const clock = manualClock();
    let searchCompleted = false;
    const pool = new FastSearcherPool({
      createSearcher: () =>
        new AfterSearchFastSearcher(() => {
          searchCompleted = true;
          clock.set(TINY_PROFILE.budgetMs);
        }),
      weakRefFactory: false,
    });
    const engine = new AutomoveEngine({ clock: clock.read });
    const game = new MonsGame(true, GameVariant.Classic);
    let unrelatedSize = 1;
    const unrelatedCache = {
      capacity: 1,
      get size(): number {
        return unrelatedSize;
      },
      clear(): void {
        unrelatedSize = 0;
      },
    };

    engine.run((execution) => {
      execution.caches.engine.own(unrelatedCache);
      expect(
        selectFastInputs(execution, game, TINY_PROFILE, pool),
      ).toBeDefined();
      expect(searchCompleted).toBe(true);
      expect(execution.caches.engine.cacheCount).toBe(1);
    });

    engine.run((execution) => {
      expect(unrelatedSize).toBe(1);
      expect(execution.caches.engine.cacheCount).toBe(1);
    });
  });

  it("uses one stable snapshot of an accessor-backed fast profile", () => {
    const expectedEngine = new AutomoveEngine({ clock: () => 0 });
    const actualEngine = new AutomoveEngine({ clock: () => 0 });
    const game = new MonsGame(true, GameVariant.Classic);
    const expectedTuning: SearchTuning = {
      lateMoveReduction: true,
      lateMoveIndex: 3,
      lateMoveDeepIndex: 8,
      moveCountPruning: true,
      moveCountDepth: 3,
      moveCountBase: 4,
      moveCountFactor: 5,
      futilityMargin: 900,
    };
    const expectedProfile: FastProfile = {
      ...TINY_PROFILE,
      maxDepth: 2,
      maxNodes: 2_048,
      tuning: expectedTuning,
    };
    const expected = expectedEngine.run((execution) =>
      selectFastSelection(execution, game, expectedProfile),
    );
    let budgetReads = 0;
    let maxDepthReads = 0;
    let maxNodesReads = 0;
    let tuningReads = 0;
    let weightsReads = 0;
    let scoreUnitReads = 0;
    const weights = new Proxy(
      { ...DEFAULT_WEIGHTS },
      {
        get(target, property, receiver) {
          if (property !== "scoreUnit") {
            return Reflect.get(target, property, receiver);
          }
          scoreUnitReads += 1;
          return scoreUnitReads === 1 ? target.scoreUnit : Number.NaN;
        },
      },
    );
    const profile: FastProfile = {
      get budgetMs(): number {
        budgetReads += 1;
        return budgetReads === 1 ? expectedProfile.budgetMs : Number.NaN;
      },
      get maxDepth(): number {
        maxDepthReads += 1;
        return maxDepthReads === 1 ? expectedProfile.maxDepth : Number.NaN;
      },
      get maxNodes(): number {
        maxNodesReads += 1;
        return maxNodesReads === 1 ? expectedProfile.maxNodes : Number.NaN;
      },
      get tuning(): SearchTuning {
        tuningReads += 1;
        if (tuningReads !== 1) throw new Error("tuning read more than once");
        return expectedTuning;
      },
      get weights() {
        weightsReads += 1;
        if (weightsReads !== 1) throw new Error("weights read more than once");
        return weights;
      },
    };

    const actual = actualEngine.run((execution) =>
      selectFastSelection(execution, game, profile),
    );

    expect(actual).toEqual(expected);
    expect(budgetReads).toBe(1);
    expect(maxDepthReads).toBe(1);
    expect(maxNodesReads).toBe(1);
    expect(tuningReads).toBe(1);
    expect(weightsReads).toBe(1);
    expect(scoreUnitReads).toBe(1);
  });

  it("rejects invalid profiles before observing time", () => {
    let clockReads = 0;
    const context = createTestAutomoveExecutionContext(() => {
      clockReads += 1;
      return 0;
    });
    const { session } = context;
    const game = new MonsGame(true);

    expect(() =>
      selectFastInputs(context, game, {
        ...TINY_PROFILE,
        maxNodes: Number.NaN,
      }),
    ).toThrow(new RangeError("maxNodes must be a nonnegative safe integer"));
    expect(() =>
      selectFastInputs(context, game, {
        ...TINY_PROFILE,
        weights: { ...DEFAULT_WEIGHTS, scoreUnit: Number.NaN },
      }),
    ).toThrow(
      new RangeError(
        "weights.scoreUnit must be a safe integer from -1000000 through 1000000",
      ),
    );
    expect(() =>
      selectFastInputs(context, game, {
        ...TINY_PROFILE,
        tuning: {
          lateMoveReduction: true,
          lateMoveIndex: 3,
          lateMoveDeepIndex: 8,
          moveCountPruning: true,
          moveCountDepth: Number.NaN,
          moveCountBase: 4,
          moveCountFactor: 5,
          futilityMargin: 900,
        },
      }),
    ).toThrow(
      new RangeError(
        `tuning.moveCountDepth must be a safe integer from 0 through ${MAX_SEARCH_DEPTH}`,
      ),
    );

    expect(clockReads).toBe(0);
    expect(context.caches.session.cacheCount).toBe(0);
    expect(context.caches.engine.cacheCount).toBe(0);
    expect(session.takePreviousTimeout()).toBe(false);
  });

  it("rejects invalid limits before observing an expired deadline", () => {
    const context = createTestAutomoveExecutionContext();
    const { session } = context;
    const game = new MonsGame(true, GameVariant.Classic);

    session.withDeadlineIfAbsent(0, () => {
      expect(() =>
        selectFastSelection(context, game, {
          ...TINY_PROFILE,
          maxDepth: Number.NaN,
        }),
      ).toThrow(
        new RangeError(
          `maxDepth must be a safe integer from 0 through ${MAX_SEARCH_DEPTH}`,
        ),
      );
    });

    expect(session.takePreviousTimeout()).toBe(false);
  });

  it("returns no terminal move before search", () => {
    const fields = new MonsGame(true, GameVariant.Classic).fen().split(" ");
    fields[0] = "5";
    fields[8] = String(Number.MAX_SAFE_INTEGER);
    const game = MonsGame.fromFen(fields.join(" "), true);
    expect(game).toBeDefined();
    if (game === undefined) return;

    const engine = new AutomoveEngine({ clock: () => 0 });
    const sourceFen = game.fen();
    const fast = engine.run((execution) => {
      const selected = selectFastInputs(execution, game, TINY_PROFILE);
      expect(execution.caches.session.cacheCount).toBe(0);
      expect(execution.caches.engine.cacheCount).toBe(0);
      return selected;
    });
    expect(fast).toEqual([]);

    const suggestion = engine.run((execution) =>
      suggestMove(execution, game, "pro"),
    );
    expect(suggestion).toEqual({
      output: { kind: "invalid-input" },
      inputFen: "",
    });
    expect(game.fen()).toBe(sourceFen);
  });

  it("does not mistake a coarse monotonic clock for a frozen clock", () => {
    let clockReads = 0;
    let observedMs = 0;
    const engine = new AutomoveEngine({
      clock: () => {
        observedMs = clockReads >= 600 ? TINY_PROFILE.budgetMs : 0;
        clockReads += 1;
        return observedMs;
      },
    });
    const game = new MonsGame(true, GameVariant.Classic);
    const selection = engine.run((execution) =>
      selectFastSelection(execution, game, {
        ...TINY_PROFILE,
        maxDepth: 40,
        maxNodes: 300_000,
      }),
    );

    expect(selection.kind).toBe("supported");
    if (selection.kind !== "supported") return;
    expect(selection.inputs.length).toBeGreaterThan(0);
    expect(observedMs).toBe(TINY_PROFILE.budgetMs);
    expect(clockReads).toBeGreaterThanOrEqual(600);
  }, 30_000);

  it("uses the node ceiling when the clock never advances", () => {
    let clockReads = 0;
    const frozenProfile: FastProfile = {
      ...PRO_FAST_PROFILE,
      maxNodes: 262_144,
    };
    const engine = new AutomoveEngine({
      clock: () => {
        clockReads += 1;
        return 0;
      },
    });
    const game = new MonsGame(true, GameVariant.Classic);
    const inputs = engine.run((execution) =>
      selectFastInputs(execution, game, frozenProfile),
    );

    expect(inputs).toBeDefined();
    expect(inputs?.length).toBeGreaterThan(0);
    expect(clockReads).toBeGreaterThan(1_000);
    expect(clockReads).toBeLessThanOrEqual(
      Math.ceil(frozenProfile.maxNodes / 512) * 2 + 16,
    );
  }, 30_000);

  it("retains a completed root move when a later position is unsupported", () => {
    const game = lateUnsupportedGame();
    const engine = new AutomoveEngine({ clock: () => 0 });
    const profile: FastProfile = {
      ...PRO_FAST_PROFILE,
      maxDepth: 2,
    };
    const selection = engine.run((execution) =>
      selectFastSelection(execution, game, profile),
    );

    expect(selection.kind).toBe("unsupported");
    if (selection.kind !== "unsupported") return;
    expect(inputArrayFen(selection.fallbackInputs)).toBe("l5,5;l5,3");
    expect(
      game.fork().processInput(selection.fallbackInputs, false, false).kind,
    ).toBe("events");
    expect(
      engine.run((execution) => selectFastInputs(execution, game, profile)),
    ).toBeUndefined();
  });

  it("returns a legal timed fallback for unsupported Pro states", () => {
    const fields = new MonsGame(false, GameVariant.Classic).fen().split(" ");
    fields[8] = String(Number.MAX_SAFE_INTEGER);
    const game = MonsGame.fromFen(fields.join(" "), false);
    expect(game).toBeDefined();
    if (game === undefined) return;

    const sourceFen = game.fen();
    const engine = new AutomoveEngine({ clock: () => 0 });
    const detailed = engine.run((execution) =>
      selectProFastSelection(execution, game),
    );
    expect(detailed).toEqual({
      kind: "unsupported",
      fallbackInputs: [],
    });
    const fast = engine.run((execution) => {
      const selected = selectProFastInputs(execution, game);
      expect(execution.caches.session.cacheCount).toBe(0);
      expect(execution.caches.engine.cacheCount).toBe(0);
      return selected;
    });
    expect(fast).toBeUndefined();

    const runtimeEngineCaches = new AutomoveCacheScope("engine");
    const runtimeSession = new SearchSession({
      clock: () => (runtimeEngineCaches.cacheCount > 0 ? 1_000 : 0),
    });
    const runtimeContext = createAutomoveExecutionContext(
      runtimeSession,
      runtimeEngineCaches,
      TEST_RANDOM_SOURCE,
    );
    const suggestion = suggestMove(runtimeContext, game, "pro");
    expect(runtimeContext.caches.engine.cacheCount).toBeGreaterThan(0);
    expect(runtimeContext.caches.session.cacheCount).toBeGreaterThan(0);
    expect(runtimeSession.takePreviousTimeout()).toBe(true);
    expect(suggestion.output.kind).toBe("events");
    expect(game.fen()).toBe(sourceFen);

    const inputs = parseInputArrayFen(suggestion.inputFen);
    expect(inputs).toBeDefined();
    if (inputs === undefined) return;
    expect(game.fork().processInput(inputs, false, false).kind).toBe("events");
  });

  it("completes canonical Pro selection after packed selection is unsupported", () => {
    const context = createTestAutomoveExecutionContext(() => 0);
    const game = new MonsGame(true, GameVariant.Classic);
    const sourceFen = game.fen();
    let packedSelections = 0;

    const inputs = selectProductionInputsWithDeadline(context, game, () => {
      packedSelections += 1;
      return { kind: "unsupported", fallbackInputs: [] };
    });

    expect(packedSelections).toBe(1);
    expect(inputArrayFen(inputs)).toBe("l10,7;l9,8");
    expect(context.caches.engine.cacheCount).toBeGreaterThan(0);
    expect(context.caches.session.cacheCount).toBeGreaterThan(0);
    expect(context.session.takePreviousTimeout()).toBe(false);
    expect(game.fen()).toBe(sourceFen);
    expect(game.fork().processInput(inputs, false, false).kind).toBe("events");
  });

  it("uses the retained packed move when the deadline expires after search", () => {
    const clock = manualClock();
    let searchCompleted = false;
    const pool = new FastSearcherPool({
      createSearcher: () =>
        new AfterSearchFastSearcher(() => {
          searchCompleted = true;
          clock.set(PRODUCTION_SELECTOR_BUDGET_MS);
        }),
      weakRefFactory: false,
    });
    const context = createTestAutomoveExecutionContext(clock.read);
    const { session } = context;
    const game = lateUnsupportedGame();
    const sourceFen = game.fen();

    const inputs = selectProductionInputsWithDeadline(
      context,
      game,
      (execution, source) => selectProFastSelection(execution, source, pool),
    );

    expect(searchCompleted).toBe(true);
    expect(inputArrayFen(inputs)).toBe("l5,5;l5,3");
    expect(session.takePreviousTimeout()).toBe(true);
    expect(game.fen()).toBe(sourceFen);
    expect(
      MonsGame.fromFen(sourceFen, false)?.processInput(inputs, false, false)
        .kind,
    ).toBe("events");
  });

  it("returns a retained packed move without building an unused emergency", () => {
    const clock = manualClock();
    const pool = new FastSearcherPool({ weakRefFactory: false });
    const context = createTestAutomoveExecutionContext(clock.read);
    const { session } = context;
    const game = lateUnsupportedGame();
    const sourceFen = game.fen();
    const fork = vi.spyOn(game, "fork");
    let packedSelectionCompleted = false;

    const inputs = selectProductionInputsWithDeadline(
      context,
      game,
      (execution, source) => {
        const selection = selectProFastSelection(execution, source, pool);
        packedSelectionCompleted = true;
        clock.set(PRODUCTION_SELECTOR_BUDGET_MS);
        return selection;
      },
    );

    expect(packedSelectionCompleted).toBe(true);
    expect(inputArrayFen(inputs)).toBe("l5,5;l5,3");
    expect(fork).not.toHaveBeenCalled();
    expect(session.takePreviousTimeout()).toBe(true);
    expect(game.fen()).toBe(sourceFen);
    expect(
      MonsGame.fromFen(sourceFen, false)?.processInput(inputs, false, false)
        .kind,
    ).toBe("events");
  });

  it("falls back before the shared Pro deadline is renewed", () => {
    const engineCaches = new AutomoveCacheScope("engine");
    const session = new SearchSession({
      clock: () => {
        if (session.caches.cacheCount > 0) {
          return PRODUCTION_SELECTOR_BUDGET_MS;
        }
        return engineCaches.cacheCount > 0 ? PRO_FAST_PROFILE.budgetMs : 0;
      },
    });
    const context = createAutomoveExecutionContext(
      session,
      engineCaches,
      TEST_RANDOM_SOURCE,
    );
    const game = lateUnsupportedGame();
    const sourceFen = game.fen();

    const suggestion = suggestMove(context, game, "pro");

    expect(suggestion.output.kind).toBe("events");
    expect(suggestion.inputFen).toBe("l5,5;l5,3");
    expect(session.takePreviousTimeout()).toBe(true);
    expect(context.caches.engine.cacheCount).toBeGreaterThan(0);
    expect(context.caches.session.cacheCount).toBeGreaterThan(0);
    expect(game.fen()).toBe(sourceFen);
  });
});
