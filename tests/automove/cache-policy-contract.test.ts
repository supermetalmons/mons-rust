import { describe, expect, it } from "vitest";

import { MonsGame } from "../../src/engine/game/mons-game.js";
import {
  BALANCED_DISTANCE_SCORING_WEIGHTS,
  DEFAULT_SCORING_WEIGHTS,
} from "../../src/automove/scoring/presets.js";
import { Hash64Table } from "../../src/automove/core/hash64.js";
import { turnCacheGet, turnCacheSet } from "../../src/automove/turn/cache.js";
import { cacheKey, configFingerprint } from "../../src/automove/turn/fingerprint.js";
import {
  TURN_PLAN_FAMILY_CACHE_TAG,
  TURN_PLAN_FAMILY_PRIORITY_ORDER,
  TurnEngineMode,
  TurnPlanFamily,
  type TurnEngineConfig,
} from "../../src/automove/turn/model.js";
import { familyRank } from "../../src/automove/turn/ordering.js";

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

function config(overrides: Partial<TurnEngineConfig>): TurnEngineConfig {
  return { ...BASE_CONFIG, ...overrides };
}

describe("automove cache and policy contracts", () => {
  it("includes every turn-engine configuration axis in cache identity", () => {
    const variants: readonly (readonly [string, TurnEngineConfig])[] = [
      ["mode", config({ mode: TurnEngineMode.Production })],
      ["ownSeedCap", config({ ownSeedCap: 9 })],
      ["ownBeam", config({ ownBeam: 5 })],
      ["perNodeFamilyCap", config({ perNodeFamilyCap: 4 })],
      ["stepCap", config({ stepCap: 6 })],
      ["opponentSeedCap", config({ opponentSeedCap: 5 })],
      ["opponentBeam", config({ opponentBeam: 3 })],
      ["replySeedCap", config({ replySeedCap: 3 })],
      ["replyBeam", config({ replyBeam: 2 })],
      ["expansionCap", config({ expansionCap: 97 })],
      ["enableSpiritFamily", config({ enableSpiritFamily: false })],
      ["scoringWeights", config({ scoringWeights: BALANCED_DISTANCE_SCORING_WEIGHTS })],
      [
        "enableLazyOracleScoreWindowProjection",
        config({ enableLazyOracleScoreWindowProjection: true }),
      ],
    ];
    const baseline = configFingerprint(BASE_CONFIG);
    const fingerprints = new Set([`${baseline.hi}:${baseline.lo}`]);

    expect(configFingerprint({ ...BASE_CONFIG })).toEqual(baseline);
    for (const [label, variant] of variants) {
      const fingerprint = configFingerprint(variant);
      expect(fingerprint, label).not.toEqual(baseline);
      fingerprints.add(`${fingerprint.hi}:${fingerprint.lo}`);
    }
    expect(fingerprints.size).toBe(variants.length + 1);

    const game = new MonsGame();
    const table = new Hash64Table<string>(variants.length + 1);
    turnCacheSet(table, cacheKey(game, BASE_CONFIG), "baseline");
    expect(turnCacheGet(table, cacheKey(game, { ...BASE_CONFIG }))).toBe("baseline");
    for (const [label, variant] of variants) {
      expect(turnCacheGet(table, cacheKey(game, variant)), label).toBeUndefined();
    }
  });

  it("pins turn family precedence separately from persistent cache tags", () => {
    expect(TURN_PLAN_FAMILY_PRIORITY_ORDER).toEqual([
      TurnPlanFamily.ImmediateScore,
      TurnPlanFamily.DenyOpponentWindow,
      TurnPlanFamily.DrainerKill,
      TurnPlanFamily.DrainerSafetyRecovery,
      TurnPlanFamily.SpiritImpact,
      TurnPlanFamily.SafeSupermanaProgress,
      TurnPlanFamily.SafeOpponentManaProgress,
      TurnPlanFamily.ManaTempo,
    ]);
    expect(TURN_PLAN_FAMILY_PRIORITY_ORDER.map((family) => familyRank(family))).toEqual(
      [0, 1, 2, 3, 4, 5, 6, 7],
    );
    expect(TURN_PLAN_FAMILY_CACHE_TAG).toEqual({
      [TurnPlanFamily.ImmediateScore]: 1,
      [TurnPlanFamily.DenyOpponentWindow]: 2,
      [TurnPlanFamily.DrainerKill]: 3,
      [TurnPlanFamily.SafeSupermanaProgress]: 4,
      [TurnPlanFamily.SafeOpponentManaProgress]: 5,
      [TurnPlanFamily.DrainerSafetyRecovery]: 6,
      [TurnPlanFamily.SpiritImpact]: 7,
      [TurnPlanFamily.ManaTempo]: 8,
    });
  });
});
