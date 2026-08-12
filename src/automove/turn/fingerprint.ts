import { colorId } from "../../engine/model/domain.js";
import type { Color } from "../../api/types.js";
import type { MonsGame } from "../../engine/game/mons-game.js";
import { exactSearchStateHash } from "../exact/hash.js";
import {
  hash64FromNonnegativeInteger,
  hash64Mul,
  hash64Xor,
  type Hash64,
} from "../core/hash64.js";
import { scoringProfileId } from "../scoring/profile-validation.js";
import {
  FNV_OFFSET_BASIS,
  FNV_PRIME,
  TURN_ENGINE_MODE_CACHE_TAG,
  type TurnEngineConfig,
  type TurnEngineMode,
} from "./model.js";

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
    hash = hash64Mul(hash64Xor(hash, hash64FromNonnegativeInteger(value)), FNV_PRIME);
  }
  for (const codePoint of scoringProfileId(config.scoringWeights)) {
    hash = hash64Mul(
      hash64Xor(hash, hash64FromNonnegativeInteger(codePoint.codePointAt(0) ?? 0)),
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

export function cacheKey(game: MonsGame, config: TurnEngineConfig): TurnCacheKey {
  return cacheKeyForMode(game, config.mode, config);
}

type UtilityCacheKey = {
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
