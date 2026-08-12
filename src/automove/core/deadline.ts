import { AutomoveCacheScope } from "./cache-scope.js";

export const AUTOMOVE_SELECTOR_BUDGET_MS = 650;

/** A monotonic millisecond clock, such as `performance.now`. */
export type MonotonicClock = () => number;

/** The cancellation surface consumed by search and analysis code. */
export type SearchControl = {
  readonly cancelled: boolean;
  readonly cacheWriteAllowed: boolean;

  checkpoint(): boolean;
  checkpointWithReserve(reserveMs: number): boolean;
};

type SearchSessionOptions = {
  readonly clock?: MonotonicClock;
};

type ActiveDeadline = {
  endMs: number;
  timedOut: boolean;
  nesting: number;
  recordTimeout: boolean;
};

function systemMonotonicClock(): number {
  return globalThis.performance.now();
}

function validateDuration(durationMs: number, name: string): number {
  if (!Number.isFinite(durationMs) || durationMs < 0) {
    throw new RangeError(`${name} must be a finite nonnegative number`);
  }
  return durationMs;
}

/**
 * Owns one automove execution's cooperative deadline state.
 *
 * Keeping this state on an explicit object prevents independent engines and
 * tests from sharing timeout history or cache-write eligibility.
 */
export class SearchSession implements SearchControl {
  readonly #clock: MonotonicClock;
  readonly #caches = new AutomoveCacheScope("session");
  #activeDeadline: ActiveDeadline | undefined;
  #previousSelectionTimedOut = false;

  public constructor(options: SearchSessionOptions = {}) {
    this.#clock = options.clock ?? systemMonotonicClock;
  }

  public get caches(): AutomoveCacheScope {
    return this.#caches;
  }

  public now(): number {
    const currentTime = this.#clock();
    if (!Number.isFinite(currentTime)) {
      throw new RangeError("monotonic clock must return a finite number");
    }
    return currentTime;
  }

  public withDeadlineIfAbsent<T>(budgetMs: number, operation: () => T): T {
    return this.#runWithDeadlineIfAbsent(budgetMs, true, operation);
  }

  public withUnrecordedDeadlineIfAbsent<T>(budgetMs: number, operation: () => T): T {
    return this.#runWithDeadlineIfAbsent(budgetMs, false, operation);
  }

  public withCooperativeSubdeadline<T>(
    budgetMs: number,
    operation: () => T,
  ): T | undefined {
    const validatedBudget = validateDuration(budgetMs, "subdeadline budget");
    const outerDeadline = this.#activeDeadline;
    return outerDeadline === undefined
      ? this.#runWithStandaloneSubdeadline(validatedBudget, operation)
      : this.#runWithNestedSubdeadline(outerDeadline, validatedBudget, operation);
  }

  public checkpoint(): boolean {
    const deadline = this.#activeDeadline;
    if (deadline === undefined) return false;
    if (deadline.timedOut) return true;
    if (this.now() >= deadline.endMs) {
      deadline.timedOut = true;
      return true;
    }
    return false;
  }

  public checkpointWithReserve(reserveMs: number): boolean {
    const validatedReserve = validateDuration(reserveMs, "checkpoint reserve");
    const deadline = this.#activeDeadline;
    if (deadline === undefined) return false;
    if (deadline.timedOut) return true;
    if (this.now() + validatedReserve >= deadline.endMs) {
      deadline.timedOut = true;
      return true;
    }
    return false;
  }

  public get cancelled(): boolean {
    return this.#activeDeadline?.timedOut ?? false;
  }

  public get cacheWriteAllowed(): boolean {
    return !this.checkpoint();
  }

  public takePreviousTimeout(): boolean {
    if (this.#activeDeadline !== undefined) return false;
    const result = this.#previousSelectionTimedOut;
    this.#previousSelectionTimedOut = false;
    return result;
  }

  #runWithDeadlineIfAbsent<T>(
    budgetMs: number,
    recordTimeout: boolean,
    operation: () => T,
  ): T {
    this.#enterDeadline(validateDuration(budgetMs, "deadline budget"), recordTimeout);
    try {
      return operation();
    } finally {
      this.#exitDeadline();
    }
  }

  #enterDeadline(budgetMs: number, recordTimeout: boolean): void {
    if (this.#activeDeadline === undefined) {
      this.#activeDeadline = {
        endMs: this.now() + budgetMs,
        timedOut: false,
        nesting: 1,
        recordTimeout,
      };
      return;
    }
    this.#activeDeadline.recordTimeout ||= recordTimeout;
    this.#activeDeadline.nesting = Math.min(
      Number.MAX_SAFE_INTEGER,
      this.#activeDeadline.nesting + 1,
    );
  }

  #exitDeadline(): void {
    const deadline = this.#activeDeadline;
    if (deadline === undefined) return;
    if (deadline.nesting <= 1) {
      this.#previousSelectionTimedOut ||= deadline.timedOut && deadline.recordTimeout;
      this.#activeDeadline = undefined;
    } else {
      deadline.nesting -= 1;
    }
  }

  #runWithStandaloneSubdeadline<T>(
    budgetMs: number,
    operation: () => T,
  ): T | undefined {
    let completed: T | undefined;
    this.withDeadlineIfAbsent(budgetMs, () => {
      if (this.checkpoint()) return;
      const result = operation();
      if (!this.checkpoint()) completed = result;
    });
    return completed;
  }

  #restoreOuterDeadline(outer: ActiveDeadline): boolean {
    const currentTime = this.now();
    const activeDeadline = this.#activeDeadline;
    const childTimedOut =
      activeDeadline === undefined ||
      activeDeadline.timedOut ||
      currentTime >= activeDeadline.endMs;
    outer.recordTimeout ||= activeDeadline?.recordTimeout ?? false;
    if (currentTime >= outer.endMs) outer.timedOut = true;
    this.#activeDeadline = outer;
    return childTimedOut;
  }

  #runWithNestedSubdeadline<T>(
    outerDeadline: ActiveDeadline,
    budgetMs: number,
    operation: () => T,
  ): T | undefined {
    if (this.checkpoint()) return undefined;

    const outer = { ...outerDeadline };
    outerDeadline.endMs = Math.min(outerDeadline.endMs, this.now() + budgetMs);
    if (this.checkpoint()) {
      this.#restoreOuterDeadline(outer);
      return undefined;
    }

    let restored = false;
    try {
      const result = operation();
      const childTimedOut = this.#restoreOuterDeadline(outer);
      restored = true;
      return childTimedOut ? undefined : result;
    } finally {
      if (!restored) this.#restoreOuterDeadline(outer);
    }
  }
}
