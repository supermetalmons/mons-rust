import type { Input } from "../../engine/domain.js";
import type { MonsGame } from "../../engine/game.js";
import type { AutomoveExecutionContext } from "../execution-context.js";
import {
  FAST_WORKSPACE_ALLOCATION_FAILED,
  isFastWorkspaceAllocationFailure,
} from "./allocation.js";
import { moveToInputs, tryLoadPosition } from "./bridge.js";
import { FastSearcher, type SearchLimits } from "./search.js";
import { DEFAULT_WEIGHTS } from "./evaluate.js";
import { FastPosition } from "./position.js";

type WeakFastSearcher = {
  deref(): FastSearcher | undefined;
};

type WeakRefGlobal = {
  readonly WeakRef?: new (target: FastSearcher) => WeakFastSearcher;
};

function tryCreateDefaultFastPosition():
  FastPosition | typeof FAST_WORKSPACE_ALLOCATION_FAILED {
  try {
    return new FastPosition();
  } catch (error) {
    if (error instanceof RangeError) return FAST_WORKSPACE_ALLOCATION_FAILED;
    throw error;
  }
}

function systemWeakFastSearcher(
  searcher: FastSearcher,
): WeakFastSearcher | undefined {
  const WeakRef = (globalThis as unknown as WeakRefGlobal).WeakRef;
  return WeakRef === undefined ? undefined : new WeakRef(searcher);
}

class FastSearcherPool {
  #idleFastSearcher: WeakFastSearcher | undefined;

  public run<Result>(
    operation: (searcher: FastSearcher) => Result,
  ): Result | typeof FAST_WORKSPACE_ALLOCATION_FAILED {
    let searcher = this.#idleFastSearcher?.deref();
    if (searcher === undefined) {
      try {
        searcher = new FastSearcher();
      } catch (error) {
        if (error instanceof RangeError) {
          return FAST_WORKSPACE_ALLOCATION_FAILED;
        }
        throw error;
      }
    }
    this.#idleFastSearcher = undefined;
    try {
      return operation(searcher);
    } finally {
      this.#release(searcher);
    }
  }

  #release(searcher: FastSearcher): void {
    this.#idleFastSearcher ??= systemWeakFastSearcher(searcher);
  }
}

const DEFAULT_FAST_SEARCHER_POOL = new FastSearcherPool();

const PRO_FAST_BUDGET_MS = 460;
const PRO_FAST_LIMITS: SearchLimits = Object.freeze({
  maxDepth: 40,
  maxNodes: 2_000_000,
});

type FastSelectionResult =
  | {
      readonly kind: "supported";
      readonly inputs: Input[];
    }
  | {
      readonly kind: "unsupported";
      readonly fallbackInputs: Input[];
    };

export function selectProFastSelection(
  execution: AutomoveExecutionContext,
  game: MonsGame,
): FastSelectionResult {
  return execution.session.withUnrecordedDeadlineIfAbsent(
    PRO_FAST_BUDGET_MS,
    (): FastSelectionResult => {
      if (execution.session.checkpoint()) {
        return { kind: "supported", inputs: [] };
      }
      if (game.winnerColor() !== undefined) {
        return { kind: "supported", inputs: [] };
      }

      const endMs = execution.session.now() + PRO_FAST_BUDGET_MS;

      const position = tryCreateDefaultFastPosition();
      if (isFastWorkspaceAllocationFailure(position)) {
        return { kind: "unsupported", fallbackInputs: [] };
      }
      if (!tryLoadPosition(position, game, PRO_FAST_LIMITS.maxDepth)) {
        return { kind: "unsupported", fallbackInputs: [] };
      }
      if (execution.session.checkpoint()) {
        return { kind: "supported", inputs: [] };
      }

      const outcome = DEFAULT_FAST_SEARCHER_POOL.run((searcher) => {
        searcher.root.copyFrom(position);
        if (execution.session.checkpoint()) return undefined;
        return searcher.search(
          PRO_FAST_LIMITS,
          () =>
            execution.session.checkpoint() || execution.session.now() >= endMs,
          DEFAULT_WEIGHTS,
        );
      });
      if (isFastWorkspaceAllocationFailure(outcome)) {
        return { kind: "unsupported", fallbackInputs: [] };
      }
      if (outcome === undefined) {
        return { kind: "supported", inputs: [] };
      }
      execution.session.checkpoint();
      if (!outcome.supported) {
        return {
          kind: "unsupported",
          fallbackInputs:
            outcome.depth > 0 && outcome.move !== 0
              ? moveToInputs(outcome.move)
              : [],
        };
      }
      return {
        kind: "supported",
        inputs: outcome.move === 0 ? [] : moveToInputs(outcome.move),
      };
    },
  );
}
