import { afterEach, describe, expect, it, vi } from "vitest";

import {
  FastSearcherPool,
  selectFastSelection,
} from "../../src/automove/fast/index.js";
import { FastSearcher } from "../../src/automove/fast/search.js";
import { GameVariant } from "../../src/engine/config.js";
import { MonsGame } from "../../src/engine/game.js";
import { createTestAutomoveExecutionContext } from "./execution-context.test-helper.js";
import { TINY_PROFILE } from "./fast.test-helper.js";

const POSITION_MODULE = "../../src/automove/fast/position.js";
const SEARCH_MODULE = "../../src/automove/fast/search.js";

function openingSelection(pool?: FastSearcherPool) {
  return selectFastSelection(
    createTestAutomoveExecutionContext(),
    new MonsGame(true, GameVariant.Classic),
    TINY_PROFILE,
    pool,
  );
}

afterEach(() => {
  vi.doUnmock(POSITION_MODULE);
  vi.doUnmock(SEARCH_MODULE);
  vi.resetModules();
});

describe("fast workspace pool failures", () => {
  it("routes default position allocation failure to unsupported", async () => {
    const failure = new RangeError("synthetic position allocation failure");
    vi.resetModules();
    vi.doMock(POSITION_MODULE, async () => {
      const actual =
        await vi.importActual<
          typeof import("../../src/automove/fast/position.js")
        >(POSITION_MODULE);
      function FailingFastPosition(): never {
        throw failure;
      }
      return {
        ...actual,
        FastPosition: FailingFastPosition,
      };
    });

    const { selectFastSelection: selectWithAllocationFailure } =
      await import("../../src/automove/fast/index.js");
    expect(
      selectWithAllocationFailure(
        createTestAutomoveExecutionContext(),
        new MonsGame(true, GameVariant.Classic),
        TINY_PROFILE,
      ),
    ).toEqual({ kind: "unsupported", fallbackInputs: [] });
  });

  it("routes default searcher allocation failure to unsupported", async () => {
    const failure = new RangeError("synthetic searcher allocation failure");
    vi.resetModules();
    vi.doMock(SEARCH_MODULE, async () => {
      const actual =
        await vi.importActual<
          typeof import("../../src/automove/fast/search.js")
        >(SEARCH_MODULE);
      function FailingFastSearcher(): never {
        throw failure;
      }
      return {
        ...actual,
        FastSearcher: FailingFastSearcher,
      };
    });

    const { selectFastSelection: selectWithAllocationFailure } =
      await import("../../src/automove/fast/index.js");
    expect(
      selectWithAllocationFailure(
        createTestAutomoveExecutionContext(),
        new MonsGame(true, GameVariant.Classic),
        TINY_PROFILE,
      ),
    ).toEqual({ kind: "unsupported", fallbackInputs: [] });
  });

  it("propagates RangeErrors from custom searcher factories", () => {
    const failure = new RangeError("synthetic custom factory failure");
    const pool = new FastSearcherPool({
      createSearcher: () => {
        throw failure;
      },
      weakRefFactory: false,
    });

    expect(() => openingSelection(pool)).toThrow(failure);
  });

  it("reads and classifies a custom searcher factory once", () => {
    const failure = new RangeError("synthetic accessor factory failure");
    let reads = 0;
    const pool = new FastSearcherPool({
      get createSearcher() {
        reads += 1;
        return reads === 1
          ? () => {
              throw failure;
            }
          : () => new FastSearcher({ transpositionCapacity: 1 });
      },
      weakRefFactory: false,
    });

    expect(reads).toBe(1);
    expect(() => openingSelection(pool)).toThrow(failure);
    expect(reads).toBe(1);
  });

  it("rejects non-function searcher factories", () => {
    expect(
      () =>
        new FastSearcherPool({
          createSearcher: null as unknown as () => FastSearcher,
        }),
    ).toThrow(new TypeError("createSearcher must be a function"));
  });

  it("propagates RangeErrors from search operations", () => {
    const failure = new RangeError("synthetic search failure");
    class FailingSearcher extends FastSearcher {
      public override search(): never {
        throw failure;
      }
    }
    const pool = new FastSearcherPool({
      createSearcher: () => new FailingSearcher(),
      weakRefFactory: false,
    });

    expect(() => openingSelection(pool)).toThrow(failure);
  });

  it("rejects asynchronous operations without repooling their searcher", async () => {
    const retained: FastSearcher[] = [];
    let creations = 0;
    let pending: Promise<void> | undefined;
    const pool = new FastSearcherPool({
      createSearcher: () => {
        creations += 1;
        return new FastSearcher({ transpositionCapacity: 1 });
      },
      weakRefFactory: (searcher) => {
        retained.push(searcher);
        return { deref: () => searcher };
      },
    });

    const asynchronousOperation = (() => {
      pending = Promise.resolve();
      return pending;
    }) as () => unknown;
    expect(() => pool.runSynchronous(asynchronousOperation)).toThrow(
      new TypeError("FastSearcherPool operations must be synchronous"),
    );
    expect(creations).toBe(1);
    expect(retained).toEqual([]);

    const second = pool.runSynchronous((searcher) => searcher);
    expect(second).toBeInstanceOf(FastSearcher);
    expect(creations).toBe(2);
    expect(retained).toEqual([second]);
    await pending;
  });
});
