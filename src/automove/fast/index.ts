import type { Input } from "../../engine/domain.js";
import type { MonsGame } from "../../engine/game.js";
import type { AutomoveExecutionContext } from "../execution-context.js";
import {
  FAST_WORKSPACE_ALLOCATION_FAILED,
  isFastWorkspaceAllocationFailure,
} from "./allocation.js";
import { moveToInputs, tryLoadPosition } from "./bridge.js";
import {
  FastSearcher,
  normalizeSearchLimits,
  type SearchLimits,
} from "./search.js";
import {
  DEFAULT_WEIGHTS,
  normalizeEvalWeights,
  type EvalWeights,
} from "./evaluate.js";
import { FastPosition } from "./position.js";

type WeakFastSearcher = {
  deref(): FastSearcher | undefined;
};

type WeakRefGlobal = {
  readonly WeakRef?: new (target: FastSearcher) => WeakFastSearcher;
};

type SynchronousResult<Result> = [
  Extract<Result, PromiseLike<unknown>>,
] extends [never]
  ? Result
  : never;

function tryCreateDefaultFastPosition():
  FastPosition | typeof FAST_WORKSPACE_ALLOCATION_FAILED {
  try {
    return new FastPosition();
  } catch (error) {
    if (error instanceof RangeError) return FAST_WORKSPACE_ALLOCATION_FAILED;
    throw error;
  }
}

function isPromiseLike(value: unknown): value is PromiseLike<unknown> {
  return (
    ((typeof value === "object" && value !== null) ||
      typeof value === "function") &&
    "then" in value &&
    typeof value.then === "function"
  );
}

export type FastSearcherPoolOptions = {
  readonly createSearcher?: () => FastSearcher;
  readonly weakRefFactory?:
    false | ((target: FastSearcher) => WeakFastSearcher);
};

function systemWeakFastSearcher(
  searcher: FastSearcher,
): WeakFastSearcher | undefined {
  const WeakRef = (globalThis as unknown as WeakRefGlobal).WeakRef;
  return WeakRef === undefined ? undefined : new WeakRef(searcher);
}

export class FastSearcherPool {
  readonly #createSearcher: () => FastSearcher;
  readonly #usesDefaultSearcherFactory: boolean;
  readonly #weakRefFactory: FastSearcherPoolOptions["weakRefFactory"];
  #idleFastSearcher: WeakFastSearcher | undefined;

  public constructor(options: FastSearcherPoolOptions = {}) {
    const createSearcher = options.createSearcher;
    if (createSearcher !== undefined && typeof createSearcher !== "function") {
      throw new TypeError("createSearcher must be a function");
    }
    this.#usesDefaultSearcherFactory = createSearcher === undefined;
    this.#createSearcher = createSearcher ?? (() => new FastSearcher());
    this.#weakRefFactory = options.weakRefFactory;
  }

  public runSynchronous<Result>(
    operation: (searcher: FastSearcher) => SynchronousResult<Result>,
  ): Result | typeof FAST_WORKSPACE_ALLOCATION_FAILED {
    let searcher = this.#idleFastSearcher?.deref();
    if (searcher === undefined) {
      if (!this.#usesDefaultSearcherFactory) {
        searcher = this.#createSearcher();
      } else {
        try {
          searcher = this.#createSearcher();
        } catch (error) {
          if (error instanceof RangeError) {
            return FAST_WORKSPACE_ALLOCATION_FAILED;
          }
          throw error;
        }
      }
    }
    this.#idleFastSearcher = undefined;
    let result: Result;
    try {
      result = operation(searcher);
    } catch (error) {
      this.#release(searcher);
      throw error;
    }
    if (isPromiseLike(result)) {
      void Promise.resolve(result).catch(() => undefined);
      throw new TypeError("FastSearcherPool operations must be synchronous");
    }
    this.#release(searcher);
    return result;
  }

  #release(searcher: FastSearcher): void {
    if (this.#weakRefFactory !== false) {
      this.#idleFastSearcher ??=
        this.#weakRefFactory === undefined
          ? systemWeakFastSearcher(searcher)
          : this.#weakRefFactory(searcher);
    }
  }
}

const DEFAULT_FAST_SEARCHER_POOL = new FastSearcherPool();

export type FastProfile = SearchLimits & {
  readonly budgetMs: number;
  readonly weights: EvalWeights;
};

/** Unsupported lets Pro continue canonically, using any completed root as fallback. */
export type FastSelectionResult =
  | {
      readonly kind: "supported";
      readonly inputs: Input[];
    }
  | {
      readonly kind: "unsupported";
      readonly fallbackInputs: Input[];
    };

export function selectFastSelection(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  profile: FastProfile,
  pool: FastSearcherPool = DEFAULT_FAST_SEARCHER_POOL,
): FastSelectionResult {
  const limits = normalizeSearchLimits(profile);
  const weights = normalizeEvalWeights(profile.weights);
  const budgetMs = profile.budgetMs;
  return execution.session.withUnrecordedDeadlineIfAbsent(
    budgetMs,
    (): FastSelectionResult => {
      if (execution.session.checkpoint()) {
        return { kind: "supported", inputs: [] };
      }
      if (game.winnerColor() !== undefined) {
        return { kind: "supported", inputs: [] };
      }

      const endMs = execution.session.now() + budgetMs;

      const position = tryCreateDefaultFastPosition();
      if (isFastWorkspaceAllocationFailure(position)) {
        return { kind: "unsupported", fallbackInputs: [] };
      }
      if (!tryLoadPosition(position, game, limits.maxDepth)) {
        return { kind: "unsupported", fallbackInputs: [] };
      }
      if (execution.session.checkpoint()) {
        return { kind: "supported", inputs: [] };
      }

      const outcome = pool.runSynchronous((searcher) => {
        searcher.root.copyFrom(position);
        if (execution.session.checkpoint()) return undefined;
        return searcher.search(
          limits,
          () =>
            execution.session.checkpoint() || execution.session.now() >= endMs,
          weights,
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

export function selectFastInputs(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  profile: FastProfile,
  pool: FastSearcherPool = DEFAULT_FAST_SEARCHER_POOL,
): Input[] | undefined {
  const selection = selectFastSelection(execution, game, profile, pool);
  return selection.kind === "supported" ? selection.inputs : undefined;
}

/**
 * Production Pro budget. The flat per-decision budget keeps the average move
 * time at or below the previous production selector while the node ceiling
 * bounds work when the host clock is frozen.
 */
export const PRO_FAST_PROFILE: FastProfile = Object.freeze({
  budgetMs: 460,
  maxDepth: 40,
  maxNodes: 2_000_000,
  weights: DEFAULT_WEIGHTS,
});

export function selectProFastSelection(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  pool: FastSearcherPool = DEFAULT_FAST_SEARCHER_POOL,
): FastSelectionResult {
  return selectFastSelection(execution, game, PRO_FAST_PROFILE, pool);
}

export function selectProFastInputs(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  pool: FastSearcherPool = DEFAULT_FAST_SEARCHER_POOL,
): Input[] | undefined {
  return selectFastInputs(execution, game, PRO_FAST_PROFILE, pool);
}
