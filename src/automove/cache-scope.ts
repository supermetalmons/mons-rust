type AutomoveCacheLifetime = "session" | "engine";

/**
 * Minimal lifecycle shared by the bounded hash tables used by automove.
 * Cache implementations retain their specialized lookup APIs.
 */
export type BoundedAutomoveCache = {
  readonly capacity: number;
  readonly size: number;

  clear(): void;
};

function assertBoundedCache(cache: BoundedAutomoveCache): void {
  if (!Number.isSafeInteger(cache.capacity) || cache.capacity < 1) {
    throw new RangeError("automove cache capacity must be a positive integer");
  }
  if (
    !Number.isSafeInteger(cache.size) ||
    cache.size < 0 ||
    cache.size > cache.capacity
  ) {
    throw new RangeError("automove cache size must be within its capacity");
  }
}

/** Owns and clears caches that share one explicit lifetime. */
export class AutomoveCacheScope {
  readonly #caches = new Set<BoundedAutomoveCache>();
  readonly #keyedCaches = new Map<symbol, BoundedAutomoveCache>();

  public constructor(public readonly lifetime: AutomoveCacheLifetime) {}

  public get cacheCount(): number {
    return this.#caches.size;
  }

  public get entryCount(): number {
    let entries = 0;
    for (const cache of this.#caches) entries += cache.size;
    return entries;
  }

  public own<Cache extends BoundedAutomoveCache>(cache: Cache): Cache {
    assertBoundedCache(cache);
    this.#caches.add(cache);
    return cache;
  }

  public getOrCreate<Cache extends BoundedAutomoveCache>(
    key: symbol,
    create: () => Cache,
  ): Cache {
    const existing = this.#keyedCaches.get(key);
    if (existing !== undefined) return existing as Cache;
    const cache = this.own(create());
    this.#keyedCaches.set(key, cache);
    return cache;
  }

  public clear(): void {
    for (const cache of this.#caches) cache.clear();
  }
}
