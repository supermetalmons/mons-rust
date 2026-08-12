import type { Input } from "../../engine/model/domain.js";
import type { MonsGame } from "../../engine/game/mons-game.js";
import type { AutomoveExecutionContext } from "../core/execution-context.js";
import {
  FAST_WORKSPACE_ALLOCATION_FAILED,
  isFastWorkspaceAllocationFailure,
} from "./allocation.js";
import { moveToInputs, tryLoadPosition } from "./bridge.js";
import { FastSearcher, type SearchLimits } from "./search.js";
import { DEFAULT_WEIGHTS } from "./evaluation.js";
import { FastPosition } from "./state.js";

type WeakFastSearcher = {
  deref(): FastSearcher | undefined;
};

type WeakRefGlobal = {
  readonly WeakRef?: new (target: FastSearcher) => WeakFastSearcher;
};

function systemWeakRefConstructor(): WeakRefGlobal["WeakRef"] {
  return (globalThis as unknown as WeakRefGlobal).WeakRef;
}

function tryCreateDefaultFastPosition():
  FastPosition | typeof FAST_WORKSPACE_ALLOCATION_FAILED {
  try {
    return new FastPosition();
  } catch (error) {
    if (error instanceof RangeError) return FAST_WORKSPACE_ALLOCATION_FAILED;
    throw error;
  }
}

function systemWeakFastSearcher(searcher: FastSearcher): WeakFastSearcher | undefined {
  const WeakRef = systemWeakRefConstructor();
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
    } catch (error) {
      if (isFastWorkspaceAllocationFailure(error)) {
        return FAST_WORKSPACE_ALLOCATION_FAILED;
      }
      throw error;
    } finally {
      this.#release(searcher);
    }
  }

  #release(searcher: FastSearcher): void {
    this.#idleFastSearcher ??= systemWeakFastSearcher(searcher);
  }
}

const DEFAULT_FAST_SEARCHER_POOL = new FastSearcherPool();

export const PACKED_SELECTION_PROFILES = Object.freeze({
  fast: Object.freeze({
    budgetMs: 16,
    limits: Object.freeze({
      maxDepth: 40,
      maxNodes: 30_000,
    }) satisfies SearchLimits,
  }),
  normal: Object.freeze({
    budgetMs: 75,
    limits: Object.freeze({
      maxDepth: 40,
      maxNodes: 150_000,
    }) satisfies SearchLimits,
  }),
  pro: Object.freeze({
    budgetMs: 460,
    limits: Object.freeze({
      maxDepth: 40,
      maxNodes: 2_000_000,
    }) satisfies SearchLimits,
  }),
});

type PackedSelectionMode = keyof typeof PACKED_SELECTION_PROFILES;

type FastSelectionResult =
  | {
      readonly kind: "supported";
      readonly inputs: Input[];
    }
  | {
      readonly kind: "unsupported";
      readonly fallbackInputs: Input[];
    };

export function selectPackedFastSelection(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  mode: PackedSelectionMode,
): FastSelectionResult {
  const profile = PACKED_SELECTION_PROFILES[mode];
  if (mode !== "pro" && systemWeakRefConstructor() === undefined) {
    return { kind: "unsupported", fallbackInputs: [] };
  }
  return execution.session.withUnrecordedDeadlineIfAbsent(
    profile.budgetMs,
    (): FastSelectionResult => {
      if (execution.session.checkpoint()) {
        return { kind: "supported", inputs: [] };
      }
      if (game.winnerColor() !== undefined) {
        return { kind: "supported", inputs: [] };
      }

      const endMs = execution.session.now() + profile.budgetMs;
      const deadlineReached = (): boolean => {
        if (execution.session.checkpoint()) return true;
        if (execution.session.now() < endMs) return false;
        execution.session.checkpoint();
        return true;
      };

      const position = tryCreateDefaultFastPosition();
      if (isFastWorkspaceAllocationFailure(position)) {
        return { kind: "unsupported", fallbackInputs: [] };
      }
      if (mode !== "pro" && deadlineReached()) {
        return { kind: "supported", inputs: [] };
      }
      if (!tryLoadPosition(position, game, profile.limits.maxDepth)) {
        return { kind: "unsupported", fallbackInputs: [] };
      }
      if (mode === "pro" ? execution.session.checkpoint() : deadlineReached()) {
        return { kind: "supported", inputs: [] };
      }

      const outcome = DEFAULT_FAST_SEARCHER_POOL.run((searcher) => {
        if (mode !== "pro" && deadlineReached()) return undefined;
        searcher.root.copyFrom(position);
        if (mode === "pro" ? execution.session.checkpoint() : deadlineReached()) {
          return undefined;
        }
        return searcher.search(profile.limits, deadlineReached, DEFAULT_WEIGHTS);
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
            outcome.depth > 0 && outcome.move !== 0 ? moveToInputs(outcome.move) : [],
        };
      }
      return {
        kind: "supported",
        inputs: outcome.move === 0 ? [] : moveToInputs(outcome.move),
      };
    },
  );
}

export function selectProFastSelection(
  execution: AutomoveExecutionContext,
  game: MonsGame,
): FastSelectionResult {
  return selectPackedFastSelection(execution, game, "pro");
}
