import type { Input } from "../../engine/model/domain.js";
import type { AutomoveExecutionContext } from "../core/execution-context.js";
import { Hash64Set, Hash64Table } from "../core/hash64.js";
import {
  TURN_ENGINE_CACHE_MAX_ENTRIES,
  type TurnOracleContext,
  type TurnPlan,
  type TurnUtility,
} from "./model.js";
import type { TurnCacheKey } from "./fingerprint.js";

const TURN_ENGINE_CACHES = Symbol("turn-engine-caches");

class TurnEngineCaches {
  public readonly continuation = new Hash64Table<readonly Input[]>(
    TURN_ENGINE_CACHE_MAX_ENTRIES,
  );
  public readonly oracle = new Hash64Table<TurnOracleContext>(
    TURN_ENGINE_CACHE_MAX_ENTRIES,
  );
  public readonly utility = new Hash64Table<TurnUtility>(TURN_ENGINE_CACHE_MAX_ENTRIES);
  public readonly bestPlan = new Hash64Table<TurnPlan>(TURN_ENGINE_CACHE_MAX_ENTRIES);
  public readonly noPlan = new Hash64Set(TURN_ENGINE_CACHE_MAX_ENTRIES);

  public readonly capacity = TURN_ENGINE_CACHE_MAX_ENTRIES * 5;

  public get size(): number {
    return (
      this.continuation.size +
      this.oracle.size +
      this.utility.size +
      this.bestPlan.size +
      this.noPlan.size
    );
  }

  public clear(): void {
    this.continuation.clear();
    this.oracle.clear();
    this.utility.clear();
    this.bestPlan.clear();
    this.noPlan.clear();
  }
}

export function turnEngineCaches(
  execution: AutomoveExecutionContext,
): TurnEngineCaches {
  return execution.caches.engine.getOrCreate(
    TURN_ENGINE_CACHES,
    () => new TurnEngineCaches(),
  );
}

export function turnCacheGet<V>(
  table: Hash64Table<V>,
  key: TurnCacheKey,
): V | undefined {
  return table.get(key.stateHash, key.modeTag, key.configFingerprint);
}

export function turnCacheHas(table: Hash64Set, key: TurnCacheKey): boolean {
  return table.has(key.stateHash, key.modeTag, key.configFingerprint);
}

export function turnCacheSet<V>(
  table: Hash64Table<V>,
  key: TurnCacheKey,
  value: V,
): void {
  table.set(key.stateHash, value, key.modeTag, key.configFingerprint);
}

export function turnCacheAdd(table: Hash64Set, key: TurnCacheKey): void {
  table.add(key.stateHash, key.modeTag, key.configFingerprint);
}

export function turnCacheDelete<V>(table: Hash64Table<V>, key: TurnCacheKey): void {
  table.delete(key.stateHash, key.modeTag, key.configFingerprint);
}

export function clearTurnEnginePlanCache(execution: AutomoveExecutionContext): void {
  turnEngineCaches(execution).clear();
}
