import type { AutomoveExecutionContext } from "./execution-context.js";
import { colorId, type Color, type Input } from "../engine/domain.js";
import type { MonsGame } from "../engine/game.js";
import { exactSearchStateHash } from "./exact.js";
import {
  Hash64Set,
  Hash64Table,
  hash64FromNonnegativeInteger,
  hash64Mul,
  hash64Xor,
  type Hash64,
} from "./hash64.js";
import { scoringProfileId } from "./scoring.js";
import {
  FNV_OFFSET_BASIS,
  FNV_PRIME,
  TURN_ENGINE_CACHE_MAX_ENTRIES,
  TURN_ENGINE_MODE_CACHE_TAG,
  type TurnEngineConfig,
  type TurnEngineMode,
  type TurnUtility,
  type TurnOracleContext,
  type TurnPlan,
} from "./turn-types.js";

const TURN_ENGINE_CACHES = Symbol("turn-engine-caches");

export class TurnEngineCaches {
  public readonly continuation = new Hash64Table<readonly Input[]>(
    TURN_ENGINE_CACHE_MAX_ENTRIES,
  );
  public readonly oracle = new Hash64Table<TurnOracleContext>(
    TURN_ENGINE_CACHE_MAX_ENTRIES,
  );
  public readonly utility = new Hash64Table<TurnUtility>(
    TURN_ENGINE_CACHE_MAX_ENTRIES,
  );
  public readonly bestPlan = new Hash64Table<TurnPlan>(
    TURN_ENGINE_CACHE_MAX_ENTRIES,
  );
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

export function configFingerprint(config: TurnEngineConfig): Hash64 {
  let hash = FNV_OFFSET_BASIS;
  const values = [
    config.ownSeedCap,
    config.ownBeam,
    config.perNodeFamilyCap,
    config.stepCap,
    config.opponentSeedCap,
    config.opponentBeam,
    config.replySeedCap,
    config.replyBeam,
    config.expansionCap,
    Number(config.enableSpiritFamily),
    TURN_ENGINE_MODE_CACHE_TAG[config.mode] + 1,
    Number(config.enableLazyOracleScoreWindowProjection),
  ];
  for (const value of values) {
    hash = hash64Mul(
      hash64Xor(hash, hash64FromNonnegativeInteger(value)),
      FNV_PRIME,
    );
  }
  for (const codePoint of scoringProfileId(config.scoringWeights)) {
    hash = hash64Mul(
      hash64Xor(
        hash,
        hash64FromNonnegativeInteger(codePoint.codePointAt(0) ?? 0),
      ),
      FNV_PRIME,
    );
  }
  return hash;
}

export type TurnCacheKey = {
  readonly stateHash: Hash64;
  readonly modeTag: number;
  readonly configFingerprint: Hash64;
};

export function cacheKeyForMode(
  game: MonsGame,
  mode: TurnEngineMode,
  config: TurnEngineConfig,
): TurnCacheKey {
  return {
    stateHash: exactSearchStateHash(game),
    modeTag: TURN_ENGINE_MODE_CACHE_TAG[mode],
    configFingerprint: configFingerprint(config),
  };
}

export function cacheKey(
  game: MonsGame,
  config: TurnEngineConfig,
): TurnCacheKey {
  return cacheKeyForMode(game, config.mode, config);
}

export type UtilityCacheKey = {
  readonly stateHash: Hash64;
  readonly configFingerprint: Hash64;
  readonly startTag: number;
  readonly startLow: number;
};

export type UtilityCacheIdentity = {
  readonly configFingerprint: Hash64;
  readonly startTag: number;
  readonly startLow: number;
};

export function createUtilityCacheIdentity(
  startHash: Hash64,
  perspective: Color,
  config: TurnEngineConfig,
): UtilityCacheIdentity {
  return {
    configFingerprint: configFingerprint(config),
    startTag: startHash.hi * 2 + colorId(perspective),
    startLow: startHash.lo,
  };
}

export function utilityCacheKey(
  stateHash: Hash64,
  identity: UtilityCacheIdentity,
): UtilityCacheKey {
  return {
    stateHash,
    configFingerprint: identity.configFingerprint,
    startTag: identity.startTag,
    startLow: identity.startLow,
  };
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

export function turnCacheDelete<V>(
  table: Hash64Table<V>,
  key: TurnCacheKey,
): void {
  table.delete(key.stateHash, key.modeTag, key.configFingerprint);
}

export function clearTurnEnginePlanCache(
  execution: AutomoveExecutionContext,
): void {
  turnEngineCaches(execution).clear();
}
