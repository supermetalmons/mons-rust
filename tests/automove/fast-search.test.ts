import { readFileSync } from "node:fs";

import { describe, expect, it, vi } from "vitest";

import { AutomoveEngine } from "../../src/automove/runtime/engine.js";
import { AUTOMOVE_SELECTOR_BUDGET_MS } from "../../src/automove/core/deadline.js";
import { PRODUCTION_SELECTOR_BUDGET_MS } from "../../src/automove/runtime/deadline-selection.js";
import { suggestMove } from "../../src/automove/runtime/suggestion.js";
import { randomAutomove } from "../../src/automove/runtime/input-selection.js";
import { enumerateLegalTransitions } from "../../src/automove/transitions/enumerate.js";
import { moveToInputs, tryLoadPosition } from "../../src/automove/packed/bridge.js";
import { MAX_MOVES, generateMoves } from "../../src/automove/packed/moves.js";
import {
  FAST_MOVE_UNREPRESENTABLE,
  FastPosition,
  applyFastMove,
} from "../../src/automove/packed/state.js";
import {
  PACKED_SELECTION_PROFILES,
  FAST_SEARCH_TUNING,
  PRO_SEARCH_TUNING,
  STRATEGIC_SEARCH_TUNING,
  selectPackedFastSelection,
  selectProFastSelection,
} from "../../src/automove/packed/index.js";
import {
  DEFAULT_WEIGHTS,
  createEvalTables,
  normalizeEvalWeights,
} from "../../src/automove/packed/evaluation-weights.js";
import {
  FAST_WORKSPACE_ALLOCATION_FAILED,
  isFastWorkspaceAllocationFailure,
} from "../../src/automove/packed/allocation.js";
import { FastSearcher } from "../../src/automove/packed/search.js";
import {
  DEFAULT_TUNING,
  MAX_NONTERMINAL_SCORE,
  MAX_SEARCH_DEPTH,
  normalizeSearchLimits,
} from "../../src/automove/packed/search-tuning.js";
import { ALL_GAME_VARIANTS, GameVariant } from "../../src/engine/board/config.js";
import { inputArrayFen, parseInputArrayFen } from "../../src/engine/codec/input.js";
import { MonsGame } from "../../src/engine/game/mons-game.js";
import { BOARD_CELLS } from "../../src/engine/board/geometry.js";
import { expectFastPositionInvariants } from "./fast.test-helper.js";

type DecisionState = {
  readonly id: string;
  readonly fen: string;
};

const V4_DECISION_STATES: readonly DecisionState[] = readFileSync(
  new URL("../../test-data/automove-decisions/v4/decisions.jsonl", import.meta.url),
  "utf8",
)
  .trim()
  .split(/\r?\n/)
  .map((line) => {
    const state = JSON.parse(line) as Partial<DecisionState>;
    if (typeof state.id !== "string" || typeof state.fen !== "string") {
      throw new TypeError("automove decision state must contain id and fen");
    }
    return { id: state.id, fen: state.fen };
  });

const LATE_UNSUPPORTED_FEN =
  "0 0 b 0 1 5 0 0 2 y0xn10/n11/n02S0xn08/n11/n11/n02A0xE0xn01d0Bn05/n11/n11/n11/n11/n11";

function lateUnsupportedGame(): MonsGame {
  const game = MonsGame.fromFen(LATE_UNSUPPORTED_FEN, false);
  if (game === undefined) throw new Error("late-unsupported FEN must parse");
  return game;
}

function loadSearchRoot(searcher: FastSearcher, game: MonsGame): void {
  expect(tryLoadPosition(searcher.root, game, 40)).toBe(true);
}

function randomSource(seed: number) {
  let state = seed >>> 0 || 0x9e3779b9;
  return {
    nextUint32(): number {
      state ^= state << 13;
      state >>>= 0;
      state ^= state >>> 17;
      state ^= state << 5;
      state >>>= 0;
      return state;
    },
  };
}

function stateSignature(position: FastPosition): string {
  const parts: string[] = [];
  for (let index = 0; index < BOARD_CELLS; index += 1) {
    const cell = position.cells[index] ?? 0;
    if (cell !== 0) parts.push(`${index}:${cell}`);
  }
  parts.push(
    `w${position.whiteScore}`,
    `b${position.blackScore}`,
    `a${position.active}`,
    `m${position.monsMoves}`,
    `n${position.manaMoves}`,
    `u${position.actionsUsed}`,
    `p${position.potions[0]}/${position.potions[1]}`,
    `f${position.firstTurn ? 1 : 0}`,
    `l${[...position.monLocations].join("/")}`,
    `r${[...position.freeMana].join("/")}`,
    `c${position.manaCount}`,
    `i${[...position.manaIndices].slice(0, position.manaCount).join("/")}`,
    `h${position.hashLo}/${position.hashHi}`,
  );
  return parts.join(",");
}

function compareCanonicalPosition(
  game: MonsGame,
  label: string,
  engine: AutomoveEngine,
  buffer: Int32Array,
  packed: FastPosition,
  applied: FastPosition,
  expected: FastPosition,
): number {
  expect(tryLoadPosition(packed, game, 40)).toBe(true);
  expectFastPositionInvariants(packed, `${label}: packed load`);
  const count = generateMoves(packed, buffer);
  const generated: (readonly [string, number])[] = [];
  for (let index = 0; index < count; index += 1) {
    const move = buffer[index] ?? 0;
    generated.push([inputArrayFen(moveToInputs(move)), move]);
  }

  const reference: string[] = [];
  engine.run((execution) => {
    for (const transition of enumerateLegalTransitions(execution, game, 100_000)) {
      reference.push(inputArrayFen(transition.inputs));
    }
    return undefined;
  });

  const generatedInputs = generated.map(([inputFen]) => inputFen);
  expect(new Set(generatedInputs).size, `${label}: duplicate generated moves`).toBe(
    generatedInputs.length,
  );
  expect(new Set(reference).size, `${label}: duplicate canonical moves`).toBe(
    reference.length,
  );
  expect([...generatedInputs].sort(), label).toEqual([...reference].sort());

  let checkedMoves = 0;
  for (const [inputFen, move] of generated) {
    applied.copyFrom(packed);
    expectFastPositionInvariants(applied, `${label} ${inputFen}: before`);
    const representable = applyFastMove(applied, move) !== FAST_MOVE_UNREPRESENTABLE;
    const inputs = parseInputArrayFen(inputFen);
    expect(inputs).toBeDefined();
    if (inputs === undefined) continue;
    const clone = game.fork();
    expect(clone.processInput(inputs, false, false).kind).toBe("events");
    const canonicalRepresentable = tryLoadPosition(expected, clone, 1);
    expect(representable, `${label} ${inputFen}`).toBe(canonicalRepresentable);
    if (!representable) continue;
    expectFastPositionInvariants(applied, `${label} ${inputFen}: applied`);
    expectFastPositionInvariants(expected, `${label} ${inputFen}: canonical`);
    expect(stateSignature(applied), `${label} ${inputFen}`).toBe(
      stateSignature(expected),
    );
    checkedMoves += 1;
  }
  return checkedMoves;
}

describe("packed-state automove search", () => {
  it("uses the audited mode budgets and fixed-clock node ceilings", () => {
    expect(STRATEGIC_SEARCH_TUNING).toEqual({
      lateMoveReduction: true,
      lateMoveIndex: 2,
      lateMoveDeepIndex: 6,
      moveCountPruning: true,
      moveCountDepth: 3,
      moveCountBase: 7,
      moveCountFactor: 7,
      futilityMargin: 900,
      aspirationDelta: 600,
      aspirationMinDepth: 2,
      winsNextTurnThreat: 0,
    });
    expect(FAST_SEARCH_TUNING).toEqual({
      lateMoveReduction: true,
      lateMoveIndex: 2,
      lateMoveDeepIndex: 6,
      moveCountPruning: true,
      moveCountDepth: 3,
      moveCountBase: 7,
      moveCountFactor: 7,
      futilityMargin: 900,
      aspirationDelta: 600,
      aspirationMinDepth: 2,
      winsNextTurnThreat: 2500,
    });
    expect(PRO_SEARCH_TUNING).toEqual({
      lateMoveReduction: true,
      lateMoveIndex: 3,
      lateMoveDeepIndex: 8,
      moveCountPruning: true,
      moveCountDepth: 3,
      moveCountBase: 4,
      moveCountFactor: 5,
      futilityMargin: 900,
      aspirationDelta: 0,
      aspirationMinDepth: 3,
      winsNextTurnThreat: 0,
    });
    expect(PACKED_SELECTION_PROFILES).toEqual({
      fast: {
        budgetMs: 16,
        limits: {
          maxDepth: 40,
          maxNodes: 38_400,
          tuning: FAST_SEARCH_TUNING,
        },
      },
      normal: {
        budgetMs: 75,
        limits: {
          maxDepth: 40,
          maxNodes: 184_000,
          tuning: STRATEGIC_SEARCH_TUNING,
        },
      },
      pro: {
        budgetMs: 460,
        limits: {
          maxDepth: 40,
          maxNodes: 2_000_000,
          tuning: PRO_SEARCH_TUNING,
        },
      },
    });
    expect(PACKED_SELECTION_PROFILES.fast.limits.tuning).toBe(FAST_SEARCH_TUNING);
    expect(PACKED_SELECTION_PROFILES.normal.limits.tuning).toBe(
      STRATEGIC_SEARCH_TUNING,
    );
    expect(PACKED_SELECTION_PROFILES.pro.limits.tuning).toBe(PRO_SEARCH_TUNING);
    expect(Object.isFrozen(PACKED_SELECTION_PROFILES)).toBe(true);
    expect(Object.isFrozen(STRATEGIC_SEARCH_TUNING)).toBe(true);
    expect(
      Object.values(PACKED_SELECTION_PROFILES).every(
        (profile) => Object.isFrozen(profile) && Object.isFrozen(profile.limits),
      ),
    ).toBe(true);
    let clockReads = 0;
    const engine = new AutomoveEngine({
      clock: () => {
        clockReads += 1;
        return 0;
      },
    });
    const search = vi.spyOn(FastSearcher.prototype, "search");
    try {
      for (const mode of ["fast", "normal", "pro"] as const) {
        const game = new MonsGame(true, GameVariant.Classic);
        const selection = engine.run((execution) =>
          selectPackedFastSelection(execution, game, mode),
        );
        expect(selection.kind, mode).toBe("supported");
        if (selection.kind !== "supported") continue;
        expect(selection.inputs.length, mode).toBeGreaterThan(0);
        expect(
          game.fork().processInput(selection.inputs, false, false).kind,
          mode,
        ).toBe("events");
      }
      expect(search.mock.calls.map(([limits]) => limits.maxNodes)).toEqual([
        38_400, 184_000, 2_000_000,
      ]);
      expect(search.mock.calls.map(([limits]) => limits)).toEqual([
        { maxDepth: 40, maxNodes: 38_400, tuning: FAST_SEARCH_TUNING },
        { maxDepth: 40, maxNodes: 184_000, tuning: STRATEGIC_SEARCH_TUNING },
        { maxDepth: 40, maxNodes: 2_000_000, tuning: PRO_SEARCH_TUNING },
      ]);
      expect(search.mock.calls[0]?.[0]).toBe(PACKED_SELECTION_PROFILES.fast.limits);
      expect(search.mock.calls[1]?.[0]).toBe(PACKED_SELECTION_PROFILES.normal.limits);
      expect(search.mock.calls[2]?.[0]).toBe(PACKED_SELECTION_PROFILES.pro.limits);
      expect(
        search.mock.results.map((result) =>
          result.type === "return" ? result.value.nodes : undefined,
        ),
      ).toEqual([38_400, 184_000, 2_000_000]);
    } finally {
      search.mockRestore();
    }
    expect(clockReads).toBeGreaterThan(0);
    expect(clockReads).toBeLessThanOrEqual(10_000);
  }, 60_000);

  it("declines Fast and Normal without WeakRef while preserving Pro", () => {
    vi.stubGlobal("WeakRef", undefined);
    const search = vi.spyOn(FastSearcher.prototype, "search");
    try {
      const engine = new AutomoveEngine({ clock: () => 0 });
      for (const mode of ["fast", "normal"] as const) {
        const game = new MonsGame(true, GameVariant.Classic);
        expect(
          engine.run((execution) => selectPackedFastSelection(execution, game, mode)),
          mode,
        ).toEqual({ kind: "unsupported", fallbackInputs: [] });
      }

      const game = new MonsGame(true, GameVariant.Classic);
      const pro = engine.run((execution) =>
        selectPackedFastSelection(execution, game, "pro"),
      );
      expect(pro.kind).toBe("supported");
      if (pro.kind === "supported") {
        expect(pro.inputs.length).toBeGreaterThan(0);
        expect(game.fork().processInput(pro.inputs, false, false).kind).toBe("events");
      }
      expect(search.mock.calls.map(([limits]) => limits.maxNodes)).toEqual([2_000_000]);
    } finally {
      search.mockRestore();
      vi.unstubAllGlobals();
    }
  }, 60_000);

  it("does not start search after a nested profile deadline expires", () => {
    const budgetMs = PACKED_SELECTION_PROFILES.fast.budgetMs;
    let clockReads = 0;
    const engine = new AutomoveEngine({
      clock: () => {
        clockReads += 1;
        return clockReads <= 3 ? 0 : budgetMs;
      },
    });
    const game = new MonsGame(true, GameVariant.Classic);
    const sourceFen = game.fen();
    const search = vi.spyOn(FastSearcher.prototype, "search");
    try {
      const result = engine.run((execution) => {
        const selection = execution.session.withDeadlineIfAbsent(
          AUTOMOVE_SELECTOR_BUDGET_MS,
          () => selectPackedFastSelection(execution, game, "fast"),
        );
        return {
          selection,
          recordedTimeout: execution.session.takePreviousTimeout(),
        };
      });

      expect(result).toEqual({
        selection: { kind: "supported", inputs: [] },
        recordedTimeout: false,
      });
      expect(search).not.toHaveBeenCalled();
      expect(game.fen()).toBe(sourceFen);
    } finally {
      search.mockRestore();
    }
    expect(clockReads).toBeGreaterThan(3);
  });

  it("records an outer timeout crossed while checking a profile", () => {
    const profileBudgetMs = PACKED_SELECTION_PROFILES.fast.budgetMs;
    let clockReads = 0;
    const engine = new AutomoveEngine({
      clock: () => {
        clockReads += 1;
        if (clockReads <= 3) return 0;
        return clockReads === 4 ? profileBudgetMs : AUTOMOVE_SELECTOR_BUDGET_MS;
      },
    });
    const game = new MonsGame(true, GameVariant.Classic);
    const search = vi.spyOn(FastSearcher.prototype, "search");
    try {
      const result = engine.run((execution) => {
        const selection = execution.session.withDeadlineIfAbsent(
          AUTOMOVE_SELECTOR_BUDGET_MS,
          () => selectPackedFastSelection(execution, game, "fast"),
        );
        return {
          selection,
          recordedTimeout: execution.session.takePreviousTimeout(),
        };
      });

      expect(result).toEqual({
        selection: { kind: "supported", inputs: [] },
        recordedTimeout: true,
      });
      expect(search).not.toHaveBeenCalled();
    } finally {
      search.mockRestore();
    }
  });

  it("preserves Pro setup behavior until the outer deadline", () => {
    const profileBudgetMs = PACKED_SELECTION_PROFILES.pro.budgetMs;
    let clockReads = 0;
    const engine = new AutomoveEngine({
      clock: () => {
        clockReads += 1;
        return clockReads <= 3 ? 0 : profileBudgetMs;
      },
    });
    const game = new MonsGame(true, GameVariant.Classic);
    const search = vi.spyOn(FastSearcher.prototype, "search").mockReturnValue({
      move: 0,
      score: 0,
      depth: 0,
      nodes: 0,
      supported: true,
    });
    try {
      const result = engine.run((execution) => {
        const selection = execution.session.withDeadlineIfAbsent(
          PRODUCTION_SELECTOR_BUDGET_MS,
          () => selectPackedFastSelection(execution, game, "pro"),
        );
        return {
          selection,
          recordedTimeout: execution.session.takePreviousTimeout(),
        };
      });

      expect(result).toEqual({
        selection: { kind: "supported", inputs: [] },
        recordedTimeout: false,
      });
      expect(search).toHaveBeenCalledOnce();
    } finally {
      search.mockRestore();
    }
    expect(clockReads).toBeGreaterThan(3);
  });

  it("falls back only for evaluation workspace allocation failures", () => {
    const FailingFloat64Array = function (): Float64Array {
      throw new RangeError("synthetic evaluation allocation");
    } as unknown as Float64ArrayConstructor;
    vi.stubGlobal("Float64Array", FailingFloat64Array);
    try {
      expect(() =>
        createEvalTables(normalizeEvalWeights({ ...DEFAULT_WEIGHTS })),
      ).toThrow(FAST_WORKSPACE_ALLOCATION_FAILED);
      let branded: unknown;
      try {
        createEvalTables(normalizeEvalWeights({ ...DEFAULT_WEIGHTS }));
      } catch (error) {
        branded = error;
      }
      expect(isFastWorkspaceAllocationFailure(branded)).toBe(true);
    } finally {
      vi.unstubAllGlobals();
    }

    const allocation = vi
      .spyOn(FastSearcher.prototype, "search")
      .mockImplementation(() => {
        throw FAST_WORKSPACE_ALLOCATION_FAILED;
      });
    try {
      for (const mode of ["fast", "normal", "pro"] as const) {
        const game = new MonsGame(true, GameVariant.Classic);
        const selection = new AutomoveEngine({ clock: () => 0 }).run((execution) =>
          selectPackedFastSelection(execution, game, mode),
        );
        expect(selection, mode).toEqual({
          kind: "unsupported",
          fallbackInputs: [],
        });
      }
      for (const mode of ["fast", "normal"] as const) {
        const game = new MonsGame(true, GameVariant.Classic);
        const sourceFen = game.fen();
        const suggestion = new AutomoveEngine({ clock: () => 0 }).run((execution) =>
          suggestMove(execution, game, mode),
        );
        expect(suggestion.output.kind, mode).toBe("events");
        const inputs = parseInputArrayFen(suggestion.inputFen);
        expect(inputs, mode).toBeDefined();
        if (inputs !== undefined) {
          expect(game.fork().processInput(inputs, false, false).kind, mode).toBe(
            "events",
          );
        }
        expect(game.fen(), mode).toBe(sourceFen);
      }
    } finally {
      allocation.mockRestore();
    }

    const failure = new RangeError("synthetic search invariant");
    const search = vi.spyOn(FastSearcher.prototype, "search").mockImplementation(() => {
      throw failure;
    });
    try {
      const game = new MonsGame(true, GameVariant.Classic);
      expect(() =>
        new AutomoveEngine({ clock: () => 0 }).run((execution) =>
          selectPackedFastSelection(execution, game, "fast"),
        ),
      ).toThrow(failure);
    } finally {
      search.mockRestore();
    }
  });

  it("matches engine move generation and transitions on every variant", () => {
    const buffer = new Int32Array(MAX_MOVES);
    const packed = new FastPosition();
    const applied = new FastPosition();
    const expected = new FastPosition();
    let checkedPositions = 0;
    let checkedMoves = 0;

    for (const variant of ALL_GAME_VARIANTS) {
      const game = new MonsGame(true, variant);
      const engine = new AutomoveEngine({ randomSource: randomSource(17) });
      for (let ply = 0; ply < 40; ply += 1) {
        if (game.winnerColor() !== undefined) break;

        checkedMoves += compareCanonicalPosition(
          game,
          `${variant} ply ${ply}`,
          engine,
          buffer,
          packed,
          applied,
          expected,
        );
        checkedPositions += 1;

        const inputFen = engine.run((execution) => {
          const suggestion = randomAutomove(execution, game.fork());
          return suggestion.output.kind === "events" ? suggestion.inputFen : undefined;
        });
        if (inputFen === undefined) break;
        const inputs = parseInputArrayFen(inputFen);
        if (inputs === undefined) break;
        if (game.processInput(inputs, false, false).kind !== "events") break;
      }
    }

    const corpusEngine = new AutomoveEngine({ randomSource: randomSource(29) });
    for (const state of V4_DECISION_STATES) {
      const game = MonsGame.fromFen(state.fen, true);
      expect(game, state.id).toBeDefined();
      if (game === undefined) continue;
      checkedMoves += compareCanonicalPosition(
        game,
        `automove-decisions/v4 ${state.id}`,
        corpusEngine,
        buffer,
        packed,
        applied,
        expected,
      );
      checkedPositions += 1;
    }

    expect(checkedPositions).toBeGreaterThan(400);
    expect(checkedMoves).toBeGreaterThan(10_000);
  }, 120_000);

  it("verifies reduced fail-highs at full depth", () => {
    const game = MonsGame.fromFen(
      "0 0 b 0 0 3 0 0 2 n03y0xs0xn06/n06d0xa0xe0xn02/n11/n04xxmn01xxmn04/n04xxmxxmxxmn04/xxQn04xxUn04xxQ/n04xxMxxMxxMn04/n04xxMn01xxMn04/n03E0xn07/n04A0xn01D0xn04/n06S0xY0xn03 5",
      true,
    );
    expect(game).toBeDefined();
    if (game === undefined) return;

    const searcher = new FastSearcher();
    loadSearchRoot(searcher, game);
    const outcome = searcher.search(
      {
        maxDepth: 4,
        maxNodes: 1_000_000,
        tuning: {
          lateMoveReduction: true,
          lateMoveIndex: 3,
          lateMoveDeepIndex: 8,
          moveCountPruning: true,
          moveCountDepth: 3,
          moveCountBase: 4,
          moveCountFactor: 5,
          futilityMargin: 900,
          aspirationDelta: 0,
          aspirationMinDepth: 3,
          winsNextTurnThreat: 0,
        },
      },
      () => false,
    );

    expect(outcome.depth).toBe(4);
    expect(outcome.score).toBe(2007);
    expect(outcome.supported).toBe(true);
    expect(inputArrayFen(moveToInputs(outcome.move))).toBe("l0,4;l1,4");
  });

  it("does not accept a selectively derived terminal score as proven", () => {
    const game = MonsGame.fromFen(
      "0 3 b 0 0 0 0 0 100 n11/n02xxmy0xn03xxUn01xxmn01/n01xxmn02s0xxxmn05/n04a0xn06/n05A0xn05/n11/n01d0xn05xxMn03/n06xxMn01xxMn02/xxMn03S0xD0xn04e0x/n03E0xn06Y0x/n11",
      true,
    );
    expect(game).toBeDefined();
    if (game === undefined) return;

    const searcher = new FastSearcher();
    loadSearchRoot(searcher, game);
    const outcome = searcher.search({ maxDepth: 8, maxNodes: 120_000 }, () => false);

    expect(outcome).toEqual({
      move: 16_725_528,
      score: 999_997,
      depth: 6,
      nodes: 120_000,
      supported: true,
    });
    expect(
      game.fork().processInput(moveToInputs(outcome.move), false, false).kind,
    ).toBe("events");
  });

  it("still stops at depth one for a genuine immediate win", () => {
    const game = MonsGame.fromFen(
      "0 3 b 0 0 0 0 0 108 n01d0Mn09/n06e0xn02xxmn01/s0xn03y0xn01xxMn04/n10a0x/n04A0xn03xxmn02/n05xxUn05/n02xxMn05Y0xn02/n02D0Mn01xxMn03S0xn02/n11/n02E0xn08/n11",
      true,
    );
    expect(game).toBeDefined();
    if (game === undefined) return;

    const winningColor = game.activeColor;
    const searcher = new FastSearcher();
    loadSearchRoot(searcher, game);
    const outcome = searcher.search({ maxDepth: 8, maxNodes: 120_000 }, () => false);

    expect(outcome).toEqual({
      move: 1_204,
      score: 1_000_000,
      depth: 1,
      nodes: 62,
      supported: true,
    });
    const replay = game.fork();
    expect(replay.processInput(moveToInputs(outcome.move), false, false).kind).toBe(
      "events",
    );
    expect(replay.winnerColor()).toBe(winningColor);
  });

  it("enforces the node ceiling and samples timeout every 512 nodes", () => {
    const game = new MonsGame(true, GameVariant.Classic);
    const cappedSearcher = new FastSearcher();
    loadSearchRoot(cappedSearcher, game);
    const capped = cappedSearcher.search({ maxDepth: 40, maxNodes: 1 }, () => false);

    expect(capped.nodes).toBe(1);
    expect(capped.depth).toBe(0);
    expect(capped.move).not.toBe(0);
    expect(game.fork().processInput(moveToInputs(capped.move), false, false).kind).toBe(
      "events",
    );

    const sampledSearcher = new FastSearcher();
    loadSearchRoot(sampledSearcher, game);
    sampledSearcher.search({ maxDepth: 2, maxNodes: 10_000 }, () => false);
    loadSearchRoot(sampledSearcher, game);
    let timeoutChecks = 0;
    const sampled = sampledSearcher.search({ maxDepth: 40, maxNodes: 10_000 }, () => {
      timeoutChecks += 1;
      return true;
    });

    expect(timeoutChecks).toBe(1);
    expect(sampled.nodes).toBe(512);
    expect(sampled.move).not.toBe(0);
    expect(
      game.fork().processInput(moveToInputs(sampled.move), false, false).kind,
    ).toBe("events");
  });

  it("honors custom weights when reusing a searcher", () => {
    const game = new MonsGame(true, GameVariant.Classic);
    const zeroWeights = normalizeEvalWeights(
      Object.fromEntries(Object.keys(DEFAULT_WEIGHTS).map((key) => [key, 0])),
    );
    const limits = { maxDepth: 3, maxNodes: 100_000 };

    const reused = new FastSearcher();
    loadSearchRoot(reused, game);
    reused.search(limits, () => false);
    loadSearchRoot(reused, game);
    const reusedOutcome = reused.search(limits, () => false, zeroWeights);

    const fresh = new FastSearcher();
    loadSearchRoot(fresh, game);
    const freshOutcome = fresh.search(limits, () => false, zeroWeights);

    expect(reusedOutcome).toEqual(freshOutcome);
  });

  it("does not allocate leaf-evaluation cache arrays", () => {
    const NativeInt32Array = globalThis.Int32Array;
    const allocationLengths: number[] = [];
    const TrackingInt32Array = new Proxy(NativeInt32Array, {
      construct(target, argumentsList, newTarget) {
        const length = argumentsList[0];
        if (typeof length === "number") allocationLengths.push(length);
        return Reflect.construct(target, argumentsList, newTarget);
      },
    });

    vi.stubGlobal("Int32Array", TrackingInt32Array);
    try {
      new FastSearcher();
    } finally {
      vi.unstubAllGlobals();
    }

    expect(allocationLengths).not.toContain(1 << 14);
  });

  it("validates limits and skips search work for terminal roots", () => {
    const normalized = normalizeSearchLimits({
      maxDepth: MAX_SEARCH_DEPTH,
      maxNodes: 0,
    });
    expect(normalized.maxDepth).toBe(MAX_SEARCH_DEPTH);
    expect(normalized.maxNodes).toBe(0);
    expect(Object.isFrozen(normalized)).toBe(true);

    for (const maxDepth of [-1, 0.5, MAX_SEARCH_DEPTH + 1, Number.NaN]) {
      expect(() => normalizeSearchLimits({ maxDepth, maxNodes: 0 })).toThrow(
        RangeError,
      );
    }
    for (const maxNodes of [-1, 0.5, Number.NaN]) {
      expect(() => normalizeSearchLimits({ maxDepth: 0, maxNodes })).toThrow(
        RangeError,
      );
    }
    expect(() => normalizeSearchLimits(null)).toThrow(TypeError);

    const fields = new MonsGame(true, GameVariant.Classic).fen().split(" ");
    fields[0] = "5";
    const terminal = MonsGame.fromFen(fields.join(" "), true);
    expect(terminal).toBeDefined();
    if (terminal === undefined) return;
    const searcher = new FastSearcher();
    loadSearchRoot(searcher, terminal);
    let timeoutChecks = 0;
    expect(
      searcher.search(
        { maxDepth: MAX_SEARCH_DEPTH, maxNodes: Number.MAX_SAFE_INTEGER },
        () => {
          timeoutChecks += 1;
          return false;
        },
      ),
    ).toEqual({
      move: 0,
      score: 0,
      depth: 0,
      nodes: 0,
      supported: true,
    });
    expect(timeoutChecks).toBe(0);
  });

  it("uses five fixed table arrays and checks time between allocations", () => {
    const game = new MonsGame(true, GameVariant.Classic);
    const NativeInt32Array = globalThis.Int32Array;
    const capacity = 1 << 20;

    const fullSearcher = new FastSearcher();
    loadSearchRoot(fullSearcher, game);
    const fullAllocations: number[] = [];
    const TrackingInt32Array = function (value: unknown): Int32Array {
      if (value === capacity) fullAllocations.push(value);
      return Reflect.construct(NativeInt32Array, [value]) as Int32Array;
    } as unknown as Int32ArrayConstructor;
    vi.stubGlobal("Int32Array", TrackingInt32Array);
    try {
      expect(
        fullSearcher.search({ maxDepth: 2, maxNodes: 100_000 }, () => false).depth,
      ).toBe(2);
    } finally {
      vi.unstubAllGlobals();
    }
    expect(fullAllocations).toEqual(Array.from({ length: 5 }, () => capacity));

    const timedSearcher = new FastSearcher();
    loadSearchRoot(timedSearcher, game);
    const timedAllocations: number[] = [];
    let timeoutChecks = 0;
    const TimedInt32Array = function (value: unknown): Int32Array {
      if (value === capacity) timedAllocations.push(value);
      return Reflect.construct(NativeInt32Array, [value]) as Int32Array;
    } as unknown as Int32ArrayConstructor;
    vi.stubGlobal("Int32Array", TimedInt32Array);
    let timedDepth = -1;
    try {
      timedDepth = timedSearcher.search({ maxDepth: 2, maxNodes: 100_000 }, () => {
        timeoutChecks += 1;
        return timeoutChecks === 3;
      }).depth;
    } finally {
      vi.unstubAllGlobals();
    }
    expect(timedDepth).toBe(1);
    expect(timedAllocations).toEqual([capacity, capacity]);
  });

  it("falls back on table RangeErrors and propagates other failures", () => {
    const game = new MonsGame(true, GameVariant.Classic);
    const capacity = 1 << 20;
    const NativeInt32Array = globalThis.Int32Array;
    const rangeSearcher = new FastSearcher();
    loadSearchRoot(rangeSearcher, game);
    const FailingInt32Array = function (value: unknown): Int32Array {
      if (value === capacity) throw new RangeError("synthetic allocation");
      return Reflect.construct(NativeInt32Array, [value]) as Int32Array;
    } as unknown as Int32ArrayConstructor;
    vi.stubGlobal("Int32Array", FailingInt32Array);
    let rangeOutcome;
    try {
      rangeOutcome = rangeSearcher.search(
        { maxDepth: 2, maxNodes: 100_000 },
        () => false,
      );
    } finally {
      vi.unstubAllGlobals();
    }
    expect(rangeOutcome.depth).toBe(1);
    expect(rangeOutcome.move).not.toBe(0);

    const failure = new TypeError("synthetic allocation");
    const throwingSearcher = new FastSearcher();
    loadSearchRoot(throwingSearcher, game);
    const ThrowingInt32Array = function (value: unknown): Int32Array {
      if (value === capacity) throw failure;
      return Reflect.construct(NativeInt32Array, [value]) as Int32Array;
    } as unknown as Int32ArrayConstructor;
    vi.stubGlobal("Int32Array", ThrowingInt32Array);
    try {
      expect(() =>
        throwingSearcher.search({ maxDepth: 2, maxNodes: 100_000 }, () => false),
      ).toThrow(failure);
    } finally {
      vi.unstubAllGlobals();
    }
  });

  it("selects an applicable Pro move for every variant opening", () => {
    let now = 0;
    const engine = new AutomoveEngine({ clock: () => now++ });
    for (const variant of ALL_GAME_VARIANTS) {
      const game = new MonsGame(true, variant);
      const selection = engine.run((execution) =>
        selectProFastSelection(execution, game),
      );
      expect(selection.kind, variant).toBe("supported");
      if (selection.kind !== "supported") continue;
      expect(selection.inputs.length, variant).toBeGreaterThan(0);
      expect(
        game.fork().processInput(selection.inputs, false, false).kind,
        variant,
      ).toBe("events");
    }
  }, 60_000);

  it("handles coarse, frozen, and nested selection clocks", () => {
    let coarseReads = 0;
    const coarseEngine = new AutomoveEngine({
      clock: () => {
        coarseReads += 1;
        return coarseReads >= 600 ? 460 : 0;
      },
    });
    const coarseGame = new MonsGame(true, GameVariant.Classic);
    const coarse = coarseEngine.run((execution) =>
      selectProFastSelection(execution, coarseGame),
    );
    expect(coarse.kind).toBe("supported");
    if (coarse.kind === "supported") {
      expect(coarse.inputs.length).toBeGreaterThan(0);
      expect(coarseGame.fork().processInput(coarse.inputs, false, false).kind).toBe(
        "events",
      );
    }
    expect(coarseReads).toBeGreaterThanOrEqual(600);

    const game = new MonsGame(true, ALL_GAME_VARIANTS[0]);
    let clockReads = 0;
    const engine = new AutomoveEngine({
      clock: () => {
        clockReads += 1;
        return 0;
      },
    });
    const frozen = engine.run((execution) => selectProFastSelection(execution, game));
    expect(frozen.kind).toBe("supported");
    if (frozen.kind === "supported") {
      expect(frozen.inputs.length).toBeGreaterThan(0);
    }
    expect(clockReads).toBeGreaterThan(0);
    expect(clockReads).toBeLessThanOrEqual(10_000);

    const nestedGame = new MonsGame(true, GameVariant.Classic);
    let nestedReads = 0;
    const nestedEngine = new AutomoveEngine({
      clock: () => {
        nestedReads += 1;
        return nestedReads >= 30 ? 460 : 0;
      },
    });
    let outerReads = 0;
    let nesting = false;
    let nestedSelection: ReturnType<typeof selectProFastSelection> | undefined;
    const outerEngine = new AutomoveEngine({
      clock: () => {
        outerReads += 1;
        if (outerReads === 20 && !nesting) {
          nesting = true;
          nestedSelection = nestedEngine.run((execution) =>
            selectProFastSelection(execution, nestedGame),
          );
        }
        return outerReads >= 30 ? 460 : 0;
      },
    });
    const outerGame = new MonsGame(true, GameVariant.Classic);
    const outerSelection = outerEngine.run((execution) =>
      selectProFastSelection(execution, outerGame),
    );
    expect(nestedSelection?.kind).toBe("supported");
    expect(outerSelection.kind).toBe("supported");
    if (nestedSelection?.kind === "supported") {
      expect(
        nestedGame.fork().processInput(nestedSelection.inputs, false, false).kind,
      ).toBe("events");
    }
    if (outerSelection.kind === "supported") {
      expect(
        outerGame.fork().processInput(outerSelection.inputs, false, false).kind,
      ).toBe("events");
    }
  }, 60_000);

  it("retains an applicable root move when a later position is unsupported", () => {
    const game = lateUnsupportedGame();
    const sourceFen = game.fen();
    const engine = new AutomoveEngine({ clock: () => 0 });
    const selection = engine.run((execution) =>
      selectProFastSelection(execution, game),
    );

    expect(selection.kind).toBe("unsupported");
    if (selection.kind !== "unsupported") return;
    expect(inputArrayFen(selection.fallbackInputs)).toBe("l5,5;l5,3");
    expect(game.fork().processInput(selection.fallbackInputs, false, false).kind).toBe(
      "events",
    );
    expect(game.fen()).toBe(sourceFen);
  });

  it("returns no Pro move for terminal games", () => {
    const fields = new MonsGame(true, GameVariant.Classic).fen().split(" ");
    fields[0] = "5";
    fields[8] = String(Number.MAX_SAFE_INTEGER);
    const game = MonsGame.fromFen(fields.join(" "), true);
    expect(game).toBeDefined();
    if (game === undefined) return;
    const sourceFen = game.fen();
    const engine = new AutomoveEngine({ clock: () => 0 });

    expect(engine.run((execution) => selectProFastSelection(execution, game))).toEqual({
      kind: "supported",
      inputs: [],
    });
    expect(engine.run((execution) => suggestMove(execution, game, "pro"))).toEqual({
      output: { kind: "invalid-input" },
      inputFen: "",
    });
    expect(game.fen()).toBe(sourceFen);
  });
});

describe("packed search tuning levers", () => {
  const LEVER_FEN =
    "0 0 b 0 0 3 0 0 2 n03y0xs0xn06/n06d0xa0xe0xn02/n11/n04xxmn01xxmn04/n04xxmxxmxxmn04/xxQn04xxUn04xxQ/n04xxMxxMxxMn04/n04xxMn01xxMn04/n03E0xn07/n04A0xn01D0xn04/n06S0xY0xn03 5";
  const BASE_LIMITS = { maxDepth: 4, maxNodes: 1_000_000 };
  const BASE_TUNING = {
    lateMoveReduction: true,
    lateMoveIndex: 3,
    lateMoveDeepIndex: 8,
    moveCountPruning: true,
    moveCountDepth: 3,
    moveCountBase: 4,
    moveCountFactor: 5,
    futilityMargin: 900,
    aspirationDelta: 0,
    aspirationMinDepth: 3,
    winsNextTurnThreat: 0,
  };

  function leverGame(): MonsGame {
    const game = MonsGame.fromFen(LEVER_FEN, true);
    if (game === undefined) throw new Error("lever fen must parse");
    return game;
  }

  function searchWith(searcher: FastSearcher, game: MonsGame, limits: object) {
    loadSearchRoot(searcher, game);
    return searcher.search(
      limits as Parameters<FastSearcher["search"]>[0],
      () => false,
      DEFAULT_WEIGHTS,
    );
  }

  it("treats an oversized aspiration window as a full-width search", () => {
    const game = leverGame();
    const plain = searchWith(new FastSearcher(), game, BASE_LIMITS);
    const huge = searchWith(new FastSearcher(), game, {
      ...BASE_LIMITS,
      tuning: {
        ...BASE_TUNING,
        aspirationDelta: MAX_NONTERMINAL_SCORE,
        aspirationMinDepth: 2,
      },
    });
    expect(huge).toEqual(plain);
  });

  it("converges aspiration re-searches to the full-width score", () => {
    const game = leverGame();
    // Futility pruning tests the static evaluation against the window's own alpha, so an
    // aspiration search and a full-width search do not prune the same nodes and their scores
    // are not required to agree. Without that lever the two must return the same value.
    const exact = { ...STRATEGIC_SEARCH_TUNING, futilityMargin: 0 };
    const plain = searchWith(new FastSearcher(), game, {
      ...BASE_LIMITS,
      tuning: { ...exact, aspirationDelta: 0 },
    });
    const aspired = searchWith(new FastSearcher(), game, {
      ...BASE_LIMITS,
      tuning: exact,
    });
    expect(aspired.score).toBe(plain.score);
    expect(aspired.depth).toBe(plain.depth);

    // With the shipped tuning the surviving invariant is the decision, not the score.
    const tuned = {
      ...BASE_LIMITS,
      tuning: { ...STRATEGIC_SEARCH_TUNING },
    };
    const shipped = searchWith(new FastSearcher(), game, tuned);
    const shippedPlain = searchWith(new FastSearcher(), game, {
      ...BASE_LIMITS,
      tuning: { ...STRATEGIC_SEARCH_TUNING, aspirationDelta: 0 },
    });
    expect(shipped.move).toBe(shippedPlain.move);
    expect(shipped.depth).toBe(shippedPlain.depth);
    expect(shipped.supported).toBe(true);
    expect(
      game.fork().processInput(moveToInputs(shipped.move), false, false).kind,
    ).toBe("events");
    const repeat = searchWith(new FastSearcher(), game, tuned);
    expect(repeat).toEqual(shipped);
  });

  it("keeps aspiration searches deterministic under node ceilings", () => {
    const game = leverGame();
    for (const budget of [6_000, 12_000, 30_000, 150_000]) {
      const limits = {
        maxDepth: 40,
        maxNodes: budget,
        tuning: { ...STRATEGIC_SEARCH_TUNING },
      };
      const first = searchWith(new FastSearcher(), game, limits);
      const second = searchWith(new FastSearcher(), game, limits);
      expect(second).toEqual(first);
      expect(first.supported).toBe(true);
      expect(first.nodes).toBeLessThanOrEqual(budget);
      expect(
        game.fork().processInput(moveToInputs(first.move), false, false).kind,
      ).toBe("events");
    }
  });

  it("defaults and validates profile-varying tuning keys", () => {
    const normalized = normalizeSearchLimits({
      maxDepth: 4,
      maxNodes: 100,
      tuning: BASE_TUNING,
    });
    expect(normalized.tuning).toEqual(DEFAULT_TUNING);
    expect(normalizeSearchLimits({ maxDepth: 4, maxNodes: 100 }).tuning).toBe(
      DEFAULT_TUNING,
    );
    expect(() =>
      normalizeSearchLimits({
        maxDepth: 4,
        maxNodes: 100,
        tuning: { ...BASE_TUNING, aspirationDelta: true },
      }),
    ).toThrow(TypeError);
    expect(() =>
      normalizeSearchLimits({
        maxDepth: 4,
        maxNodes: 100,
        tuning: { ...BASE_TUNING, aspirationDelta: -1 },
      }),
    ).toThrow(RangeError);
    expect(() =>
      normalizeSearchLimits({
        maxDepth: 4,
        maxNodes: 100,
        tuning: { ...BASE_TUNING, aspirationMinDepth: 99 },
      }),
    ).toThrow(RangeError);
  });
});
