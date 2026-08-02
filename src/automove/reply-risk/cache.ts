import type { Color } from "../../engine/domain.js";
import { colorId } from "../../engine/domain.js";
import type { MonsGame } from "../../engine/game.js";
import type { AutomoveExecutionContext } from "../execution-context.js";
import { exactSearchStateHash } from "../exact.js";
import {
  Hash64Table,
  hash64,
  type Hash64,
  type Hash64Qualifier,
} from "../hash64.js";
import { scoringProfileId } from "../scoring.js";
import type { AutomoveConfig } from "../selector-types.js";
import {
  TurnEngineMode,
  type TurnEngineConfig,
  type TurnUtility,
} from "../turn-engine.js";
import type { RootReplyRiskSnapshot } from "./types.js";

const REPLY_RISK_SNAPSHOT_CACHE_MAX_ENTRIES = 4_096;

const REPLY_RISK_SESSION_CACHES = Symbol("reply-risk-session-caches");

class ReplyRiskSessionCaches {
  public readonly snapshots = new Hash64Table<RootReplyRiskSnapshot>(
    REPLY_RISK_SNAPSHOT_CACHE_MAX_ENTRIES,
  );
  public readonly spiritFollowupFloors = new Hash64Table<number>(
    REPLY_RISK_SNAPSHOT_CACHE_MAX_ENTRIES,
  );
  public readonly selectedOverrideUtilities = new Hash64Table<TurnUtility>(
    REPLY_RISK_SNAPSHOT_CACHE_MAX_ENTRIES,
  );

  public readonly capacity = REPLY_RISK_SNAPSHOT_CACHE_MAX_ENTRIES * 3;

  public get size(): number {
    return (
      this.snapshots.size +
      this.spiritFollowupFloors.size +
      this.selectedOverrideUtilities.size
    );
  }

  public clear(): void {
    this.snapshots.clear();
    this.spiritFollowupFloors.clear();
    this.selectedOverrideUtilities.clear();
  }
}

function replyRiskCaches(
  execution: AutomoveExecutionContext,
): ReplyRiskSessionCaches {
  return execution.caches.session.getOrCreate(
    REPLY_RISK_SESSION_CACHES,
    () => new ReplyRiskSessionCaches(),
  );
}

type ReplyRiskCacheKey = {
  readonly hash: Hash64;
  readonly tag: number;
  readonly qualifier: Hash64Qualifier;
  readonly secondary?: Hash64;
};

function cacheGet<V>(
  cache: Hash64Table<V>,
  key: ReplyRiskCacheKey,
): V | undefined {
  return cache.get(key.hash, key.tag, key.secondary, key.qualifier);
}

function cacheSet<V>(
  cache: Hash64Table<V>,
  key: ReplyRiskCacheKey,
  value: V,
): void {
  cache.set(key.hash, value, key.tag, key.secondary, key.qualifier);
}

export function replyRiskCacheKey(
  game: MonsGame,
  perspective: Color,
  replyLimit: number,
  config: AutomoveConfig,
): ReplyRiskCacheKey | undefined {
  const weights = config.evaluation.weights;
  if (!Number.isInteger(replyLimit) || replyLimit < 0 || replyLimit > 0xffff) {
    return undefined;
  }
  const tag = replyLimit + colorId(perspective) * 0x1_0000;
  return {
    hash: exactSearchStateHash(game),
    tag,
    qualifier: `string:${scoringProfileId(weights)}`,
  };
}

export function selectedOverrideConfigKey(
  config: TurnEngineConfig,
  enableSelectedFollowupProjection: boolean,
  enableSecondaryAnalysis: boolean,
): Hash64 | undefined {
  const fields: readonly (readonly [number, number])[] = [
    [config.ownSeedCap, 0xf],
    [config.ownBeam, 0x7],
    [config.perNodeFamilyCap, 0x7],
    [config.stepCap, 0x7],
    [config.opponentSeedCap, 0x7],
    [config.opponentBeam, 0x3],
    [config.replySeedCap, 0x3],
    [config.replyBeam, 0x3],
    [config.expansionCap, 0xff],
  ];
  if (
    fields.some(
      ([value, maximum]) =>
        !Number.isInteger(value) || value < 0 || value > maximum,
    )
  ) {
    return undefined;
  }
  const low =
    config.ownSeedCap |
    (config.ownBeam << 4) |
    (config.perNodeFamilyCap << 7) |
    (config.stepCap << 10) |
    (config.opponentSeedCap << 13) |
    (config.opponentBeam << 16) |
    (config.replySeedCap << 18) |
    (config.replyBeam << 20);
  const high =
    config.expansionCap |
    (Number(config.enableSpiritFamily) << 8) |
    (Number(enableSelectedFollowupProjection) << 9) |
    (Number(config.mode === TurnEngineMode.Production) << 10) |
    (Number(enableSecondaryAnalysis) << 11) |
    (Number(config.enableLazyOracleScoreWindowProjection) << 12);
  return hash64(high, low);
}

export function clearReplyRiskCache(execution: AutomoveExecutionContext): void {
  replyRiskCaches(execution).clear();
}

export function cachedReplyRiskSnapshot(
  execution: AutomoveExecutionContext,
  key: ReplyRiskCacheKey,
): RootReplyRiskSnapshot | undefined {
  return cacheGet(replyRiskCaches(execution).snapshots, key);
}

export function storeReplyRiskSnapshot(
  execution: AutomoveExecutionContext,
  key: ReplyRiskCacheKey,
  snapshot: RootReplyRiskSnapshot,
): void {
  cacheSet(replyRiskCaches(execution).snapshots, key, snapshot);
}

export function cachedSpiritFollowupFloor(
  execution: AutomoveExecutionContext,
  key: ReplyRiskCacheKey,
): number | undefined {
  return cacheGet(replyRiskCaches(execution).spiritFollowupFloors, key);
}

export function storeSpiritFollowupFloor(
  execution: AutomoveExecutionContext,
  key: ReplyRiskCacheKey,
  score: number,
): void {
  cacheSet(replyRiskCaches(execution).spiritFollowupFloors, key, score);
}

export function cachedSelectedOverrideUtility(
  execution: AutomoveExecutionContext,
  key: ReplyRiskCacheKey,
): TurnUtility | undefined {
  return cacheGet(replyRiskCaches(execution).selectedOverrideUtilities, key);
}

export function storeSelectedOverrideUtility(
  execution: AutomoveExecutionContext,
  key: ReplyRiskCacheKey,
  utility: TurnUtility,
): void {
  cacheSet(replyRiskCaches(execution).selectedOverrideUtilities, key, utility);
}
