import { beforeEach, describe, expect, it } from "vitest";

import { AutomoveEngine } from "../../src/automove/automove-engine.js";
import {
  SearchSession,
  type SearchControl,
} from "../../src/automove/deadline.js";
import {
  AutomoveCacheScope,
  createAutomoveExecutionContext,
  type BoundedAutomoveCache,
} from "../../src/automove/execution-context.js";

let session: SearchSession;
const TEST_RANDOM_SOURCE = Object.freeze({
  nextUint32: () => 0,
});

function mockClock(initialTime = 0): { set(time: number): void } {
  let currentTime = initialTime;
  session = new SearchSession({ clock: () => currentTime });
  return {
    set(time: number): void {
      currentTime = time;
    },
  };
}

describe("cooperative automove deadlines", () => {
  beforeEach(() => {
    session = new SearchSession({ clock: () => 0 });
    session.takePreviousTimeout();
  });

  it("has no cancellation or cache restriction outside a deadline", () => {
    expect(session.checkpoint()).toBe(false);
    expect(session.checkpointWithReserve(1_000)).toBe(false);
    expect(session.cancelled).toBe(false);
    expect(session.cacheWriteAllowed).toBe(true);
    expect(session.takePreviousTimeout()).toBe(false);
  });

  it("preserves the earlier outer deadline across ordinary nesting", () => {
    const clock = mockClock();
    const result = session.withDeadlineIfAbsent(50, () => {
      clock.set(10);
      const nested = session.withDeadlineIfAbsent(0, () => {
        expect(session.checkpoint()).toBe(false);
        return "nested";
      });
      expect(nested).toBe("nested");

      clock.set(50);
      expect(session.checkpoint()).toBe(true);
      clock.set(0);
      expect(session.checkpoint()).toBe(true);
      expect(session.cancelled).toBe(true);
      expect(session.cacheWriteAllowed).toBe(false);
      return "outer";
    });

    expect(result).toBe("outer");
    expect(session.cancelled).toBe(false);
    expect(session.takePreviousTimeout()).toBe(true);
    expect(session.takePreviousTimeout()).toBe(false);
  });

  it("restores a live outer deadline after a child deadline expires", () => {
    const clock = mockClock();
    session.withDeadlineIfAbsent(100, () => {
      clock.set(10);
      const child = session.withCooperativeSubdeadline(20, () => {
        clock.set(31);
        return "late";
      });

      expect(child).toBeUndefined();
      expect(session.cancelled).toBe(false);
      expect(session.checkpoint()).toBe(false);
      expect(session.cacheWriteAllowed).toBe(true);
    });
    expect(session.takePreviousTimeout()).toBe(false);
  });

  it("restores a live outer deadline when a nested operation throws", () => {
    const clock = mockClock();
    session.withDeadlineIfAbsent(100, () => {
      clock.set(10);
      expect(() =>
        session.withCooperativeSubdeadline(20, () => {
          clock.set(15);
          throw new Error("child failed");
        }),
      ).toThrow("child failed");

      expect(session.cancelled).toBe(false);
      expect(session.cacheWriteAllowed).toBe(true);
      clock.set(99);
      expect(session.checkpoint()).toBe(false);
    });
    expect(session.takePreviousTimeout()).toBe(false);
  });

  it("keeps an outer expiry sticky when it occurs inside a child", () => {
    const clock = mockClock();
    session.withDeadlineIfAbsent(20, () => {
      clock.set(5);
      const child = session.withCooperativeSubdeadline(100, () => {
        clock.set(20);
        return "late";
      });

      expect(child).toBeUndefined();
      expect(session.cancelled).toBe(true);
      expect(session.cacheWriteAllowed).toBe(false);
    });
    expect(session.takePreviousTimeout()).toBe(true);
  });

  it("restores the outer deadline after a child consumes its reserve", () => {
    const clock = mockClock();
    session.withDeadlineIfAbsent(100, () => {
      clock.set(10);
      const child = session.withCooperativeSubdeadline(40, () => {
        clock.set(35);
        expect(session.checkpointWithReserve(15)).toBe(true);
        return "reserved";
      });

      expect(child).toBeUndefined();
      expect(session.cancelled).toBe(false);
      expect(session.cacheWriteAllowed).toBe(true);
      clock.set(99);
      expect(session.checkpoint()).toBe(false);
    });
    expect(session.takePreviousTimeout()).toBe(false);
  });

  it("runs standalone child deadlines and suppresses late results", () => {
    const clock = mockClock();
    const completed = session.withCooperativeSubdeadline(10, () => {
      clock.set(9);
      return 7;
    });
    expect(completed).toBe(7);
    expect(session.takePreviousTimeout()).toBe(false);

    clock.set(20);
    const timedOut = session.withCooperativeSubdeadline(10, () => {
      clock.set(30);
      return 9;
    });
    expect(timedOut).toBeUndefined();
    expect(session.takePreviousTimeout()).toBe(true);
  });

  it("marks the deadline sticky when only the cleanup reserve remains", () => {
    const clock = mockClock();
    session.withDeadlineIfAbsent(10, () => {
      clock.set(4.999);
      expect(session.checkpointWithReserve(5)).toBe(false);
      clock.set(5);
      expect(session.checkpointWithReserve(5)).toBe(true);
      clock.set(0);
      expect(session.checkpointWithReserve(0)).toBe(true);
      expect(session.cacheWriteAllowed).toBe(false);
    });
    expect(session.takePreviousTimeout()).toBe(true);
  });

  it("keeps an earlier top-level timeout sticky across a later success", () => {
    const clock = mockClock();
    session.withDeadlineIfAbsent(10, () => {
      clock.set(10);
      expect(session.checkpoint()).toBe(true);
    });
    session.withDeadlineIfAbsent(10, () => {
      expect(session.checkpoint()).toBe(false);
    });

    expect(session.takePreviousTimeout()).toBe(true);
    expect(session.takePreviousTimeout()).toBe(false);
  });

  it("restores session state when an operation throws", () => {
    mockClock();
    expect(() =>
      session.withDeadlineIfAbsent(10, () => {
        throw new Error("operation failed");
      }),
    ).toThrow("operation failed");

    expect(session.checkpoint()).toBe(false);
    expect(session.cancelled).toBe(false);
    expect(session.cacheWriteAllowed).toBe(true);
    expect(session.takePreviousTimeout()).toBe(false);
  });

  it("implements the narrow search-control contract", () => {
    const control: SearchControl = session;
    expect(control.checkpoint()).toBe(false);
    expect(control.checkpointWithReserve(10)).toBe(false);
    expect(control.cancelled).toBe(false);
    expect(control.cacheWriteAllowed).toBe(true);
  });

  it("rejects a non-finite clock reading", () => {
    session = new SearchSession({ clock: () => Number.NaN });
    expect(() => session.withDeadlineIfAbsent(10, () => undefined)).toThrow(
      "monotonic clock must return a finite number",
    );
  });

  it("rejects negative and non-finite time budgets", () => {
    for (const invalid of [-1, Number.NaN, Number.POSITIVE_INFINITY]) {
      expect(() =>
        session.withDeadlineIfAbsent(invalid, () => undefined),
      ).toThrow("deadline budget must be a finite nonnegative number");
      expect(() =>
        session.withCooperativeSubdeadline(invalid, () => undefined),
      ).toThrow("subdeadline budget must be a finite nonnegative number");
      expect(() => session.checkpointWithReserve(invalid)).toThrow(
        "checkpoint reserve must be a finite nonnegative number",
      );
    }
  });

  it("expires a nonzero budget against the real monotonic clock", () => {
    const realSession = new SearchSession();
    let expired = false;

    realSession.withDeadlineIfAbsent(1, () => {
      const testGuardEnd = globalThis.performance.now() + 100;
      while (
        !realSession.checkpoint() &&
        globalThis.performance.now() < testGuardEnd
      ) {
        // Exercise the production clock until the cooperative check expires.
      }
      expired = realSession.cancelled;
    });

    expect(expired).toBe(true);
    expect(realSession.takePreviousTimeout()).toBe(true);
  });
});

function boundedCache(initialSize: number, capacity = 8): BoundedAutomoveCache {
  let size = initialSize;
  return {
    capacity,
    get size(): number {
      return size;
    },
    clear(): void {
      size = 0;
    },
  };
}

describe("automove execution context", () => {
  it("owns bounded caches according to their explicit lifetime", () => {
    const engineCaches = new AutomoveCacheScope("engine");
    const context = createAutomoveExecutionContext(
      new SearchSession({ clock: () => 0 }),
      engineCaches,
      TEST_RANDOM_SOURCE,
    );
    const sessionCache = context.caches.session.own(boundedCache(2));
    const engineCache = context.caches.engine.own(boundedCache(3));

    expect(context.caches.session.lifetime).toBe("session");
    expect(context.caches.engine.lifetime).toBe("engine");
    expect(context.caches.session.entryCount).toBe(2);
    expect(context.caches.engine.entryCount).toBe(3);

    context.caches.session.clear();
    expect(sessionCache.size).toBe(0);
    expect(engineCache.size).toBe(3);
  });

  it("rejects invalid cache containers and cache-scope wiring", () => {
    const scope = new AutomoveCacheScope("session");
    expect(() => scope.own(boundedCache(2, 1))).toThrow(
      "automove cache size must be within its capacity",
    );
    expect(() =>
      createAutomoveExecutionContext(
        new SearchSession({ clock: () => 0 }),
        scope,
        TEST_RANDOM_SOURCE,
      ),
    ).toThrow("execution context requires an engine cache scope");
  });

  it("creates a fresh session per engine run and preserves engine caches", () => {
    let currentTime = 0;
    const engine = new AutomoveEngine({ clock: () => currentTime });
    const engineCache = boundedCache(4);
    const firstSessionCache = boundedCache(2);
    let firstSession: SearchSession | undefined;
    let firstEngineScope: AutomoveCacheScope | undefined;

    const firstResult = engine.run((context) => {
      firstSession = context.session;
      firstEngineScope = context.caches.engine;
      context.caches.engine.own(engineCache);
      context.caches.session.own(firstSessionCache);
      return context.session.withDeadlineIfAbsent(10, () => {
        currentTime = 9;
        return 7;
      });
    });

    expect(firstResult).toBe(7);
    expect(firstSessionCache.size).toBe(0);
    expect(engineCache.size).toBe(4);

    engine.run((context) => {
      expect(context.session).not.toBe(firstSession);
      expect(context.caches.engine).toBe(firstEngineScope);
      expect(context.caches.engine.entryCount).toBe(4);
    });

    engine.clearCaches();
    expect(engineCache.size).toBe(0);
  });

  it("invalidates engine caches before the run after a timeout", () => {
    let currentTime = 0;
    const engine = new AutomoveEngine({ clock: () => currentTime });
    const engineCache = boundedCache(4);

    engine.run((context) => {
      context.caches.engine.own(engineCache);
      context.session.withDeadlineIfAbsent(10, () => {
        currentTime = 10;
        expect(context.session.checkpoint()).toBe(true);
      });
    });

    expect(engineCache.size).toBe(4);
    engine.run((context) => {
      expect(context.caches.engine.entryCount).toBe(0);
    });
    expect(engineCache.size).toBe(0);
  });

  it("invalidates engine caches after a timeout followed by a successful phase", () => {
    let currentTime = 0;
    const engine = new AutomoveEngine({ clock: () => currentTime });
    const engineCache = boundedCache(4);

    engine.run((context) => {
      context.caches.engine.own(engineCache);
      context.session.withDeadlineIfAbsent(10, () => {
        currentTime = 10;
        expect(context.session.checkpoint()).toBe(true);
      });
      context.session.withDeadlineIfAbsent(10, () => {
        expect(context.session.checkpoint()).toBe(false);
      });
    });

    engine.run((context) => {
      expect(context.caches.engine.entryCount).toBe(0);
    });
    expect(engineCache.size).toBe(0);
  });
});
