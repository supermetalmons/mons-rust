import { describe, expect, it } from "vitest";

import { loadPosition, moveToInputs } from "../../src/automove/fast/bridge.js";
import { Z_SCALAR_HI, Z_SCALAR_LO } from "../../src/automove/fast/board.js";
import {
  DEFAULT_WEIGHTS,
  WIN_VALUE,
  normalizeEvalWeights,
  validateEvalWeights,
  type EvalWeights,
} from "../../src/automove/fast/evaluate.js";
import { MAX_MOVES } from "../../src/automove/fast/moves.js";
import { FastPosition } from "../../src/automove/fast/position.js";
import {
  FastSearcher,
  MAX_SEARCH_DEPTH,
  normalizeSearchLimits,
  scalarIndex,
  stateKeyHi,
  stateKeyLo,
  validateSearchLimits,
  type SearchTuning,
} from "../../src/automove/fast/search.js";
import {
  ACTIONS_PER_TURN,
  GameVariant,
  MANA_MOVES_PER_TURN,
  MONS_MOVES_PER_TURN,
} from "../../src/engine/config.js";
import { MonsGame } from "../../src/engine/game.js";
import { resetFastPosition } from "./fast.test-helper.js";

const TERMINAL_SCORE_BAND = WIN_VALUE - 48;
const MAX_NONTERMINAL_SCORE = WIN_VALUE - 49;
const CONFIGURED_SCALAR_STATE_COUNT =
  2 *
  (MONS_MOVES_PER_TURN + 1) *
  (MANA_MOVES_PER_TURN + 1) *
  (ACTIONS_PER_TURN + 1) *
  2 *
  4 *
  4;

const VALID_TUNING: SearchTuning = Object.freeze({
  lateMoveReduction: true,
  lateMoveIndex: 3,
  lateMoveDeepIndex: 8,
  moveCountPruning: true,
  moveCountDepth: 3,
  moveCountBase: 4,
  moveCountFactor: 5,
  futilityMargin: 900,
});

const NUMERIC_TUNING_FIELDS = [
  ["lateMoveIndex", MAX_MOVES],
  ["lateMoveDeepIndex", MAX_MOVES],
  ["moveCountDepth", MAX_SEARCH_DEPTH],
  ["moveCountBase", MAX_MOVES],
  ["moveCountFactor", MAX_MOVES],
  ["futilityMargin", MAX_NONTERMINAL_SCORE],
] as const satisfies readonly (readonly [keyof SearchTuning, number])[];

function tuningWith(key: keyof SearchTuning, value: unknown): SearchTuning {
  return { ...VALID_TUNING, [key]: value };
}

function weightsWith(key: keyof EvalWeights, value: unknown): EvalWeights {
  return { ...DEFAULT_WEIGHTS, [key]: value };
}

function openingSearch(): {
  readonly game: MonsGame;
  readonly searcher: FastSearcher;
} {
  const game = new MonsGame(true, GameVariant.Classic);
  const searcher = new FastSearcher();
  loadPosition(searcher.root, game);
  return { game, searcher };
}

function expectValidationError(
  operation: () => unknown,
  errorType: typeof TypeError | typeof RangeError,
  message: string,
): void {
  let thrown: unknown;
  try {
    operation();
  } catch (error) {
    thrown = error;
  }
  expect(thrown).toBeInstanceOf(errorType);
  expect((thrown as Error | undefined)?.message).toBe(message);
}

describe("packed-state search limits", () => {
  it("enforces maxNodes as an exact work-unit ceiling", () => {
    const { game, searcher } = openingSearch();
    const outcome = searcher.search({ maxDepth: 40, maxNodes: 1 }, () => false);

    expect(outcome.nodes).toBe(1);
    expect(outcome.depth).toBe(0);
    expect(outcome.supported).toBe(true);
    expect(outcome.move).not.toBe(0);
    expect(
      game.fork().processInput(moveToInputs(outcome.move), false, false).kind,
    ).toBe("events");
  });

  it("samples timeout after exactly 512 counted work units", () => {
    const { game, searcher } = openingSearch();
    searcher.search({ maxDepth: 2, maxNodes: 10_000 }, () => false);
    loadPosition(searcher.root, game);
    let timeoutChecks = 0;
    const outcome = searcher.search({ maxDepth: 40, maxNodes: 10_000 }, () => {
      timeoutChecks += 1;
      return true;
    });

    expect(timeoutChecks).toBe(1);
    expect(outcome.nodes).toBe(512);
    expect(outcome.move).not.toBe(0);
    expect(
      game.fork().processInput(moveToInputs(outcome.move), false, false).kind,
    ).toBe("events");
  });

  it("keeps high nonterminal evaluations outside the terminal score band", () => {
    const fields = new MonsGame(true, GameVariant.Classic).fen().split(" ");
    fields[6] = "5000";
    const game = MonsGame.fromFen(fields.join(" "), true);
    expect(game).toBeDefined();
    if (game === undefined) return;

    const searcher = new FastSearcher();
    loadPosition(searcher.root, game);
    const outcome = searcher.search(
      { maxDepth: 2, maxNodes: 1_000_000 },
      () => false,
    );

    expect(outcome.depth).toBe(2);
    expect(Math.abs(outcome.score)).toBeLessThan(TERMINAL_SCORE_BAND);
    expect(outcome.move).not.toBe(0);
  });

  it("rejects unsafe limits and accepts the exact depth ceiling", () => {
    const { searcher } = openingSearch();

    for (const maxDepth of [
      -1,
      0.5,
      MAX_SEARCH_DEPTH + 1,
      Number.NaN,
      Number.POSITIVE_INFINITY,
    ]) {
      expectValidationError(
        () => searcher.search({ maxDepth, maxNodes: 0 }, () => false),
        RangeError,
        `maxDepth must be a safe integer from 0 through ${MAX_SEARCH_DEPTH}`,
      );
    }
    for (const maxNodes of [
      -1,
      0.5,
      Number.NaN,
      Number.POSITIVE_INFINITY,
      Number.MAX_SAFE_INTEGER + 1,
    ]) {
      expectValidationError(
        () => searcher.search({ maxDepth: 0, maxNodes }, () => false),
        RangeError,
        "maxNodes must be a nonnegative safe integer",
      );
    }

    const outcome = searcher.search(
      { maxDepth: MAX_SEARCH_DEPTH, maxNodes: 0 },
      () => false,
    );
    expect(outcome.depth).toBe(0);
    expect(outcome.move).not.toBe(0);
    expect(outcome.supported).toBe(true);
  });

  it("validates every evaluation weight and tolerates extra fields", () => {
    const keys = Object.keys(DEFAULT_WEIGHTS) as (keyof EvalWeights)[];
    expect(keys).toHaveLength(29);

    for (const key of keys) {
      expect(() =>
        validateEvalWeights(weightsWith(key, -1_000_000)),
      ).not.toThrow();
      expect(() =>
        validateEvalWeights(weightsWith(key, 1_000_000)),
      ).not.toThrow();
      for (const value of [
        -1_000_001,
        1_000_001,
        0.5,
        Number.NaN,
        Number.POSITIVE_INFINITY,
        Number.MAX_SAFE_INTEGER + 1,
      ]) {
        expectValidationError(
          () => validateEvalWeights(weightsWith(key, value)),
          RangeError,
          `weights.${key} must be a safe integer from -1000000 through 1000000`,
        );
      }
      expectValidationError(
        () => validateEvalWeights(weightsWith(key, undefined)),
        TypeError,
        `weights.${key} must be a number`,
      );
      expectValidationError(
        () => validateEvalWeights(weightsWith(key, "1")),
        TypeError,
        `weights.${key} must be a number`,
      );
    }

    for (const value of [undefined, null, [], 1, "weights"]) {
      expectValidationError(
        () => validateEvalWeights(value),
        TypeError,
        "fast evaluation weights must be an object",
      );
    }
    expect(() =>
      validateEvalWeights({ ...DEFAULT_WEIGHTS, futureWeight: 1 }),
    ).not.toThrow();
  });

  it("validates every search tuning field at its engine-derived bounds", () => {
    for (const key of [
      "lateMoveReduction",
      "moveCountPruning",
    ] as const satisfies readonly (keyof SearchTuning)[]) {
      for (const value of [true, false]) {
        expect(() =>
          validateSearchLimits({
            maxDepth: 0,
            maxNodes: 0,
            tuning: tuningWith(key, value),
          }),
        ).not.toThrow();
      }
      expectValidationError(
        () =>
          validateSearchLimits({
            maxDepth: 0,
            maxNodes: 0,
            tuning: tuningWith(key, 1),
          }),
        TypeError,
        `tuning.${key} must be a boolean`,
      );
    }

    for (const [key, maximum] of NUMERIC_TUNING_FIELDS) {
      for (const value of [0, maximum]) {
        expect(() =>
          validateSearchLimits({
            maxDepth: 0,
            maxNodes: 0,
            tuning: tuningWith(key, value),
          }),
        ).not.toThrow();
      }
      for (const value of [
        -1,
        maximum + 1,
        0.5,
        Number.NaN,
        Number.POSITIVE_INFINITY,
        Number.MAX_SAFE_INTEGER + 1,
      ]) {
        expectValidationError(
          () =>
            validateSearchLimits({
              maxDepth: 0,
              maxNodes: 0,
              tuning: tuningWith(key, value),
            }),
          RangeError,
          `tuning.${key} must be a safe integer from 0 through ${maximum}`,
        );
      }
      expectValidationError(
        () =>
          validateSearchLimits({
            maxDepth: 0,
            maxNodes: 0,
            tuning: tuningWith(key, undefined),
          }),
        TypeError,
        `tuning.${key} must be a number`,
      );
    }

    for (const tuning of [null, [], 1, "tuning"]) {
      expectValidationError(
        () =>
          validateSearchLimits({
            maxDepth: 0,
            maxNodes: 0,
            tuning: tuning as unknown as SearchTuning,
          }),
        TypeError,
        "fast search tuning must be an object",
      );
    }
    expect(() =>
      validateSearchLimits({
        maxDepth: 0,
        maxNodes: 0,
        tuning: {
          ...VALID_TUNING,
          futureOption: true,
        } as SearchTuning,
      }),
    ).not.toThrow();
  });

  it("normalizes accessor-backed weights and limits from one read", () => {
    const weightKeys = Object.keys(DEFAULT_WEIGHTS) as (keyof EvalWeights)[];
    const weightReads = new Map<keyof EvalWeights, number>();
    const weights = new Proxy(
      { ...DEFAULT_WEIGHTS },
      {
        get(target, property, receiver) {
          if (
            typeof property === "string" &&
            weightKeys.includes(property as keyof EvalWeights)
          ) {
            const key = property as keyof EvalWeights;
            const reads = (weightReads.get(key) ?? 0) + 1;
            weightReads.set(key, reads);
            return reads === 1
              ? Reflect.get(target, property, receiver)
              : Number.NaN;
          }
          return Reflect.get(target, property, receiver);
        },
      },
    );

    const normalizedWeights = normalizeEvalWeights(weights);
    expect(normalizedWeights).toEqual(DEFAULT_WEIGHTS);
    expect(Object.is(normalizedWeights, weights)).toBe(false);
    expect(Object.isFrozen(normalizedWeights)).toBe(true);
    for (const key of weightKeys) {
      expect(weightReads.get(key), key).toBe(1);
    }

    let maxDepthReads = 0;
    let maxNodesReads = 0;
    let tuningReads = 0;
    const tuningKeys = Object.keys(VALID_TUNING) as (keyof SearchTuning)[];
    const tuningFieldReads = new Map<keyof SearchTuning, number>();
    const tuning = new Proxy(
      { ...VALID_TUNING },
      {
        get(target, property, receiver) {
          if (
            typeof property === "string" &&
            tuningKeys.includes(property as keyof SearchTuning)
          ) {
            const key = property as keyof SearchTuning;
            const reads = (tuningFieldReads.get(key) ?? 0) + 1;
            tuningFieldReads.set(key, reads);
            return reads === 1
              ? Reflect.get(target, property, receiver)
              : undefined;
          }
          return Reflect.get(target, property, receiver);
        },
      },
    );
    const limits = {
      get maxDepth(): number {
        maxDepthReads += 1;
        return maxDepthReads === 1 ? 4 : Number.NaN;
      },
      get maxNodes(): number {
        maxNodesReads += 1;
        return maxNodesReads === 1 ? 2_048 : Number.NaN;
      },
      get tuning(): SearchTuning {
        tuningReads += 1;
        if (tuningReads !== 1) throw new Error("tuning read more than once");
        return tuning;
      },
    };

    const normalizedLimits = normalizeSearchLimits(limits);
    expect(normalizedLimits).toEqual({
      maxDepth: 4,
      maxNodes: 2_048,
      tuning: VALID_TUNING,
    });
    expect(Object.isFrozen(normalizedLimits)).toBe(true);
    expect(Object.isFrozen(normalizedLimits.tuning)).toBe(true);
    expect(maxDepthReads).toBe(1);
    expect(maxNodesReads).toBe(1);
    expect(tuningReads).toBe(1);
    for (const key of tuningKeys) {
      expect(tuningFieldReads.get(key), key).toBe(1);
    }
  });

  it("rejects invalid tuning and weights before direct search work", () => {
    const { searcher } = openingSearch();
    let timeoutChecks = 0;
    const checkTimeout = (): boolean => {
      timeoutChecks += 1;
      return false;
    };

    expectValidationError(
      () =>
        searcher.search(
          { maxDepth: 1, maxNodes: 1 },
          checkTimeout,
          weightsWith("scoreUnit", Number.NaN),
        ),
      RangeError,
      "weights.scoreUnit must be a safe integer from -1000000 through 1000000",
    );
    expectValidationError(
      () =>
        searcher.search(
          {
            maxDepth: 1,
            maxNodes: 1,
            tuning: tuningWith("moveCountDepth", Number.NaN),
          },
          checkTimeout,
        ),
      RangeError,
      `tuning.moveCountDepth must be a safe integer from 0 through ${MAX_SEARCH_DEPTH}`,
    );
    expect(timeoutChecks).toBe(0);
  });

  it("searches from one snapshot of accessor-backed configuration", () => {
    const control = openingSearch().searcher.search(
      { maxDepth: 4, maxNodes: 2_048, tuning: VALID_TUNING },
      () => false,
      DEFAULT_WEIGHTS,
    );
    const { searcher } = openingSearch();
    let maxDepthReads = 0;
    let maxNodesReads = 0;
    let tuningReads = 0;
    let moveCountDepthReads = 0;
    let scoreUnitReads = 0;
    const tuning = new Proxy(VALID_TUNING, {
      get(target, property, receiver) {
        if (property !== "moveCountDepth") {
          return Reflect.get(target, property, receiver);
        }
        moveCountDepthReads += 1;
        return moveCountDepthReads === 1 ? target.moveCountDepth : Number.NaN;
      },
    });
    const limits = {
      get maxDepth(): number {
        maxDepthReads += 1;
        return maxDepthReads === 1 ? 4 : Number.NaN;
      },
      get maxNodes(): number {
        maxNodesReads += 1;
        return maxNodesReads === 1 ? 2_048 : Number.NaN;
      },
      get tuning(): SearchTuning {
        tuningReads += 1;
        if (tuningReads !== 1) throw new Error("tuning read more than once");
        return tuning;
      },
    };
    const weights = new Proxy(DEFAULT_WEIGHTS, {
      get(target, property, receiver) {
        if (property !== "scoreUnit") {
          return Reflect.get(target, property, receiver);
        }
        scoreUnitReads += 1;
        return scoreUnitReads === 1 ? target.scoreUnit : Number.NaN;
      },
    });

    const outcome = searcher.search(limits, () => false, weights);
    expect(outcome).toEqual(control);
    expect(maxDepthReads).toBe(1);
    expect(maxNodesReads).toBe(1);
    expect(tuningReads).toBe(1);
    expect(moveCountDepthReads).toBe(1);
    expect(scoreUnitReads).toBe(1);
  });

  it("keeps direct search configuration stable across timeout callbacks", () => {
    const control = openingSearch().searcher.search(
      { maxDepth: 40, maxNodes: 1_024, tuning: VALID_TUNING },
      () => false,
      DEFAULT_WEIGHTS,
    );
    const { searcher } = openingSearch();
    const tuning = { ...VALID_TUNING };
    const limits = { maxDepth: 40, maxNodes: 1_024, tuning };
    const weights = { ...DEFAULT_WEIGHTS };
    let mutated = false;

    const outcome = searcher.search(
      limits,
      () => {
        if (!mutated) {
          mutated = true;
          limits.maxDepth = 0;
          limits.maxNodes = 0;
          tuning.moveCountDepth = 0;
          weights.scoreUnit = Number.NaN;
        }
        return false;
      },
      weights,
    );

    expect(mutated).toBe(true);
    expect(outcome).toEqual(control);
  });

  it("maps every configured scalar state to a unique contiguous index", () => {
    const position = new FastPosition();
    const indices = new Set<number>();

    for (let active = 0; active < 2; active += 1) {
      for (
        let monsMoves = 0;
        monsMoves <= MONS_MOVES_PER_TURN;
        monsMoves += 1
      ) {
        for (
          let manaMoves = 0;
          manaMoves <= MANA_MOVES_PER_TURN;
          manaMoves += 1
        ) {
          for (
            let actionsUsed = 0;
            actionsUsed <= ACTIONS_PER_TURN;
            actionsUsed += 1
          ) {
            for (const firstTurn of [false, true]) {
              for (let whitePotions = 0; whitePotions < 4; whitePotions += 1) {
                for (
                  let blackPotions = 0;
                  blackPotions < 4;
                  blackPotions += 1
                ) {
                  resetFastPosition(position, {
                    active,
                    monsMoves,
                    manaMoves,
                    actionsUsed,
                    firstTurn,
                    potions: [whitePotions, blackPotions],
                  });
                  const index = scalarIndex(position);
                  indices.add(index);
                }
              }
            }
          }
        }
      }
    }

    expect(indices.size).toBe(CONFIGURED_SCALAR_STATE_COUNT);
    expect([...indices].sort((a, b) => a - b)).toEqual(
      Array.from(
        { length: CONFIGURED_SCALAR_STATE_COUNT },
        (_, index) => index,
      ),
    );
    expect(CONFIGURED_SCALAR_STATE_COUNT).toBeLessThanOrEqual(
      Z_SCALAR_LO.length,
    );
    expect(Z_SCALAR_LO.length).toBe(Z_SCALAR_HI.length);
  });

  it("preserves the current scalar hash distribution", () => {
    const position = new FastPosition();
    expect(scalarIndex(position)).toBe(0);

    resetFastPosition(position, { active: 1 });
    expect(scalarIndex(position)).toBe(1);
    resetFastPosition(position, { active: 0, monsMoves: 1 });
    expect(scalarIndex(position)).toBe(2);
    resetFastPosition(position, { monsMoves: 0, manaMoves: 1 });
    expect(scalarIndex(position)).toBe(12);
    resetFastPosition(position, { manaMoves: 0, actionsUsed: 1 });
    expect(scalarIndex(position)).toBe(24);
    resetFastPosition(position, { actionsUsed: 0, firstTurn: true });
    expect(scalarIndex(position)).toBe(48);
    resetFastPosition(position, { firstTurn: false, potions: [1, 0] });
    expect(scalarIndex(position)).toBe(96);
    resetFastPosition(position, { potions: [0, 1] });
    expect(scalarIndex(position)).toBe(384);

    resetFastPosition(position, {
      active: 1,
      monsMoves: MONS_MOVES_PER_TURN,
      manaMoves: MANA_MOVES_PER_TURN,
      actionsUsed: ACTIONS_PER_TURN,
      firstTurn: true,
      potions: [3, 3],
    });
    expect(scalarIndex(position)).toBe(1535);
  });

  it("does not search terminal roots", () => {
    const fields = new MonsGame(true, GameVariant.Classic).fen().split(" ");
    fields[0] = "5";
    const game = MonsGame.fromFen(fields.join(" "), true);
    expect(game).toBeDefined();
    if (game === undefined) return;

    const searcher = new FastSearcher();
    loadPosition(searcher.root, game);
    let timeoutChecks = 0;
    const outcome = searcher.search(
      { maxDepth: MAX_SEARCH_DEPTH, maxNodes: Number.MAX_SAFE_INTEGER },
      () => {
        timeoutChecks += 1;
        return false;
      },
    );

    expect(outcome).toEqual({
      move: 0,
      score: 0,
      depth: 0,
      nodes: 0,
      supported: true,
    });
    expect(timeoutChecks).toBe(0);
    expect(searcher.size).toBe(0);
  });

  it("separates exact potion counts within the shared scalar bucket", () => {
    const first = new FastPosition();
    const second = new FastPosition();
    resetFastPosition(first, { potions: [3, 0] });
    resetFastPosition(second, { potions: [4, 0] });

    expect(scalarIndex(first)).toBe(scalarIndex(second));
    expect(stateKeyLo(first, scalarIndex(first))).not.toBe(
      stateKeyLo(second, scalarIndex(second)),
    );
    expect(stateKeyHi(first, scalarIndex(first))).toBe(
      stateKeyHi(second, scalarIndex(second)),
    );

    resetFastPosition(first, { potions: [0, 3] });
    resetFastPosition(second, { potions: [0, 4] });

    expect(scalarIndex(first)).toBe(scalarIndex(second));
    expect(stateKeyLo(first, scalarIndex(first))).toBe(
      stateKeyLo(second, scalarIndex(second)),
    );
    expect(stateKeyHi(first, scalarIndex(first))).not.toBe(
      stateKeyHi(second, scalarIndex(second)),
    );
  });
});
