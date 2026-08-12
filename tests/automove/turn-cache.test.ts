import { describe, expect, it } from "vitest";

import { AutomoveEngine } from "../../src/automove/runtime/engine.js";
import {
  turnCacheAdd,
  turnCacheHas,
  turnEngineCaches,
} from "../../src/automove/turn/cache.js";
import {
  cacheKey,
  configFingerprint,
  createUtilityCacheIdentity,
  utilityCacheKey,
} from "../../src/automove/turn/fingerprint.js";
import { hash64 } from "../../src/automove/core/hash64.js";
import {
  TURN_ENGINE_MODE_CACHE_TAG,
  TurnEngineMode,
  type TurnEngineConfig,
} from "../../src/automove/turn/model.js";
import { DEFAULT_SCORING_WEIGHTS } from "../../src/automove/scoring/presets.js";
import { Color } from "../../src/engine/model/domain.js";
import { MonsGame } from "../../src/engine/game/mons-game.js";

const BASE_CONFIG = Object.freeze({
  mode: TurnEngineMode.Baseline,
  ownSeedCap: 8,
  ownBeam: 4,
  perNodeFamilyCap: 3,
  stepCap: 5,
  opponentSeedCap: 4,
  opponentBeam: 2,
  replySeedCap: 2,
  replyBeam: 1,
  expansionCap: 96,
  enableSpiritFamily: true,
  scoringWeights: DEFAULT_SCORING_WEIGHTS,
  enableLazyOracleScoreWindowProjection: false,
} satisfies TurnEngineConfig);

describe("turn-engine cache identity", () => {
  it("keeps stable numeric identities for the renamed modes", () => {
    expect(TurnEngineMode).toEqual({
      Baseline: "baseline",
      Production: "production",
    });
    expect(TURN_ENGINE_MODE_CACHE_TAG).toEqual({
      baseline: 0,
      production: 1,
    });
  });

  it("separates warm entries when oracle projection policy changes", () => {
    const game = new MonsGame();
    const projectionConfig = Object.freeze({
      ...BASE_CONFIG,
      enableLazyOracleScoreWindowProjection: true,
    } satisfies TurnEngineConfig);

    expect(configFingerprint(BASE_CONFIG)).not.toEqual(
      configFingerprint(projectionConfig),
    );

    new AutomoveEngine().run((execution) => {
      const noPlanCache = turnEngineCaches(execution).noPlan;
      turnCacheAdd(noPlanCache, cacheKey(game, BASE_CONFIG));

      expect(turnCacheHas(noPlanCache, cacheKey(game, BASE_CONFIG))).toBe(true);
      expect(turnCacheHas(noPlanCache, cacheKey(game, projectionConfig))).toBe(false);
    });
  });

  it("separates stable utility identity from the evaluated state", () => {
    const startHash = hash64(11, 29);
    const firstState = hash64(31, 47);
    const secondState = hash64(53, 71);
    const whiteIdentity = createUtilityCacheIdentity(
      startHash,
      Color.White,
      BASE_CONFIG,
    );
    const blackIdentity = createUtilityCacheIdentity(
      startHash,
      Color.Black,
      BASE_CONFIG,
    );
    const projectionIdentity = createUtilityCacheIdentity(startHash, Color.White, {
      ...BASE_CONFIG,
      enableLazyOracleScoreWindowProjection: true,
    });

    expect(utilityCacheKey(firstState, whiteIdentity)).toEqual({
      stateHash: firstState,
      configFingerprint: whiteIdentity.configFingerprint,
      startTag: startHash.hi * 2,
      startLow: startHash.lo,
    });
    expect(utilityCacheKey(secondState, whiteIdentity)).toMatchObject({
      stateHash: secondState,
      configFingerprint: whiteIdentity.configFingerprint,
      startTag: whiteIdentity.startTag,
      startLow: whiteIdentity.startLow,
    });
    expect(blackIdentity.startTag).not.toBe(whiteIdentity.startTag);
    expect(projectionIdentity.configFingerprint).not.toEqual(
      whiteIdentity.configFingerprint,
    );
  });
});
