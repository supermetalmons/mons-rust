import { AutomoveCacheScope } from "../core/cache-scope.js";
import { SearchSession, type MonotonicClock } from "../core/deadline.js";
import {
  createAutomoveExecutionContext,
  type AutomoveExecutionContext,
  type AutomoveRandomSource,
} from "../core/execution-context.js";

type AutomoveEngineOptions = {
  readonly clock?: MonotonicClock;
  readonly randomSource?: AutomoveRandomSource;
};

const CRYPTO_RANDOM_SOURCE: AutomoveRandomSource = Object.freeze({
  nextUint32(): number {
    const value = new Uint32Array(1);
    globalThis.crypto.getRandomValues(value);
    return value[0] ?? 0;
  },
});

/**
 * Owns cross-call automove state and creates isolated state for each execution.
 */
export class AutomoveEngine {
  readonly #clock: MonotonicClock | undefined;
  readonly #randomSource: AutomoveRandomSource;
  readonly #engineCaches = new AutomoveCacheScope("engine");
  #clearCachesBeforeNextRun = false;

  public constructor(options: AutomoveEngineOptions = {}) {
    this.#clock = options.clock;
    this.#randomSource = options.randomSource ?? CRYPTO_RANDOM_SOURCE;
  }

  public run<Result>(operation: (context: AutomoveExecutionContext) => Result): Result {
    if (this.#clearCachesBeforeNextRun) {
      this.#engineCaches.clear();
      this.#clearCachesBeforeNextRun = false;
    }
    const session = new SearchSession(
      this.#clock === undefined ? {} : { clock: this.#clock },
    );
    const context = createAutomoveExecutionContext(
      session,
      this.#engineCaches,
      this.#randomSource,
    );
    try {
      return operation(context);
    } finally {
      this.#clearCachesBeforeNextRun = session.takePreviousTimeout();
      context.caches.session.clear();
    }
  }

  public clearCaches(): void {
    this.#clearCachesBeforeNextRun = false;
    this.#engineCaches.clear();
  }
}
