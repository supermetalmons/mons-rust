import { describe, expect, it } from "vitest";

import { moveToInputs, tryLoadPosition } from "../../src/automove/bridge.js";
import { FastSearcher } from "../../src/automove/search.js";
import {
  DEFAULT_TUNING,
  MAX_SEARCH_DEPTH,
  memoizedSearchLimits,
  normalizeSearchLimits,
} from "../../src/automove/search-tuning.js";
import {
  FAST_SEARCH_TUNING,
  PACKED_SELECTION_PROFILES,
  PRO_SEARCH_TUNING,
  STRATEGIC_SEARCH_TUNING,
} from "../../src/automove/selector.js";
import { GameVariant } from "../../src/engine/board/config.js";
import { inputArrayFen } from "../../src/engine/codec/input.js";
import { MonsGame } from "../../src/engine/game/mons-game.js";

const BOMB_DEMON_FEN =
  "0 0 b 0 1 5 0 0 2 y0xn10/n11/n02S0xn08/n11/n11/n02A0xE0xn01d0Bn05/n11/n11/n11/n11/n11";

function loadedSearcher(game = new MonsGame(true, GameVariant.Classic)): FastSearcher {
  const searcher = new FastSearcher();
  expect(tryLoadPosition(searcher.root, game, 40)).toBe(true);
  return searcher;
}

describe("packed automove search", () => {
  it("keeps the audited budgets, node ceilings, and tuning", () => {
    expect(PACKED_SELECTION_PROFILES).toEqual({
      fast: {
        budgetMs: 50,
        limits: {
          maxDepth: 40,
          maxNodes: 38_400,
          tuning: FAST_SEARCH_TUNING,
        },
      },
      normal: {
        budgetMs: 150,
        limits: {
          maxDepth: 40,
          maxNodes: 184_000,
          tuning: STRATEGIC_SEARCH_TUNING,
        },
      },
      pro: {
        budgetMs: 650,
        limits: {
          maxDepth: 40,
          maxNodes: 2_000_000,
          tuning: PRO_SEARCH_TUNING,
        },
      },
    });
    expect(Object.isFrozen(PACKED_SELECTION_PROFILES)).toBe(true);
    expect(
      Object.values(PACKED_SELECTION_PROFILES).every(
        (profile) => Object.isFrozen(profile) && Object.isFrozen(profile.limits),
      ),
    ).toBe(true);
    expect([
      FAST_SEARCH_TUNING.winsNextTurnThreat,
      STRATEGIC_SEARCH_TUNING.winsNextTurnThreat,
      PRO_SEARCH_TUNING.winsNextTurnThreat,
    ]).toEqual([2_500, 0, 0]);
  });

  it("selects a legal root move at each fixed node ceiling", () => {
    for (const preference of ["fast", "normal", "pro"] as const) {
      const game = new MonsGame(true, GameVariant.Classic);
      const searcher = loadedSearcher(game);
      const outcome = searcher.search(
        PACKED_SELECTION_PROFILES[preference].limits,
        () => false,
      );
      expect(outcome.supported, preference).toBe(true);
      expect(outcome.move, preference).not.toBe(0);
      expect(outcome.nodes, preference).toBe(
        PACKED_SELECTION_PROFILES[preference].limits.maxNodes,
      );
      expect(
        game.fork().processInput(moveToInputs(outcome.move), false, false).kind,
        preference,
      ).toBe("events");
    }
  }, 60_000);

  it("checks the cooperative timeout every 2,048 nodes", () => {
    const game = new MonsGame(true, GameVariant.Classic);
    const searcher = loadedSearcher(game);
    searcher.search({ maxDepth: 2, maxNodes: 100_000 }, () => false);

    let checks = 0;
    const outcome = searcher.search({ maxDepth: 40, maxNodes: 100_000 }, () => {
      checks += 1;
      return checks === 3;
    });

    expect(checks).toBe(3);
    expect(outcome.nodes).toBe(3 * 2_048);
    expect(outcome.move).not.toBe(0);
    expect(
      game.fork().processInput(moveToInputs(outcome.move), false, false).kind,
    ).toBe("events");
  });

  it("keeps bomb-fainted Demon replies representable", () => {
    const game = MonsGame.fromFen(BOMB_DEMON_FEN, false);
    expect(game).toBeDefined();
    if (game === undefined) return;
    const outcome = loadedSearcher(game).search(
      PACKED_SELECTION_PROFILES.pro.limits,
      () => false,
    );

    expect(outcome.supported).toBe(true);
    expect(outcome.move).not.toBe(0);
    expect(outcome.depth).toBeGreaterThan(0);
    expect(inputArrayFen(moveToInputs(outcome.move))).toBe("l0,0;l2,2");
    expect(
      game.fork().processInput(moveToInputs(outcome.move), false, false).kind,
    ).toBe("events");
  }, 15_000);

  it("normalizes limits without changing frozen profiles", () => {
    const limits = PACKED_SELECTION_PROFILES.fast.limits;
    const first = memoizedSearchLimits(limits);
    expect(memoizedSearchLimits(limits)).toBe(first);
    expect(first).toEqual(limits);
    expect(normalizeSearchLimits({ maxDepth: 0, maxNodes: 0 })).toEqual({
      maxDepth: 0,
      maxNodes: 0,
      tuning: DEFAULT_TUNING,
    });
  });

  it.each([
    [null, TypeError],
    [{ maxDepth: -1, maxNodes: 1 }, RangeError],
    [{ maxDepth: MAX_SEARCH_DEPTH + 1, maxNodes: 1 }, RangeError],
    [{ maxDepth: 1, maxNodes: -1 }, RangeError],
    [{ maxDepth: 1, maxNodes: 1, tuning: {} }, TypeError],
  ])("rejects invalid search limits %#", (limits, error) => {
    expect(() => normalizeSearchLimits(limits)).toThrow(error);
  });
});
