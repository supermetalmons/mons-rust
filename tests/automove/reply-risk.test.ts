import { describe, expect, it } from "vitest";

import {
  clearReplyRiskCache,
  rootReplyRiskSnapshot,
  selectedOverrideConfigKey,
} from "../../src/automove/reply-risk.js";
import { automoveConfigForGame } from "../../src/automove/selector-config.js";
import { DEFAULT_SCORING_WEIGHTS } from "../../src/automove/scoring.js";
import {
  TurnEngineMode,
  type TurnEngineConfig,
} from "../../src/automove/turn-engine.js";
import { GameVariant } from "../../src/engine/config.js";
import { Color } from "../../src/engine/domain.js";
import { MonsGame } from "../../src/engine/game.js";
import { createTestAutomoveExecutionContext } from "./execution-context.test-helper.js";

describe("reply-risk execution caches", () => {
  it("reuses snapshots only within the owning search session", () => {
    const firstExecution = createTestAutomoveExecutionContext();
    const secondExecution = createTestAutomoveExecutionContext();
    const game = new MonsGame(false, GameVariant.Classic);
    const config = automoveConfigForGame(game, "fast");

    const first = rootReplyRiskSnapshot(
      firstExecution,
      game,
      Color.White,
      config,
      4,
    );
    expect(
      rootReplyRiskSnapshot(firstExecution, game, Color.White, config, 4),
    ).toBe(first);
    expect(firstExecution.caches.session.entryCount).toBeGreaterThan(0);
    expect(secondExecution.caches.session.entryCount).toBe(0);

    const isolated = rootReplyRiskSnapshot(
      secondExecution,
      game,
      Color.White,
      config,
      4,
    );
    expect(isolated).toEqual(first);
    expect(isolated).not.toBe(first);

    const entriesBeforeClear = firstExecution.caches.session.entryCount;
    clearReplyRiskCache(firstExecution);
    expect(firstExecution.caches.session.entryCount).toBeLessThan(
      entriesBeforeClear,
    );
  });

  it("keys selected overrides by every behavior-changing planner switch", () => {
    const base = {
      mode: TurnEngineMode.Baseline,
      ownSeedCap: 8,
      ownBeam: 3,
      perNodeFamilyCap: 3,
      stepCap: 4,
      opponentSeedCap: 2,
      opponentBeam: 1,
      replySeedCap: 1,
      replyBeam: 1,
      expansionCap: 64,
      enableSpiritFamily: true,
      scoringWeights: DEFAULT_SCORING_WEIGHTS,
      enableLazyOracleScoreWindowProjection: false,
    } satisfies TurnEngineConfig;
    const keys = [
      selectedOverrideConfigKey(base, false, false),
      selectedOverrideConfigKey(
        { ...base, mode: TurnEngineMode.Production },
        false,
        false,
      ),
      selectedOverrideConfigKey(base, false, true),
      selectedOverrideConfigKey(
        { ...base, enableLazyOracleScoreWindowProjection: true },
        false,
        false,
      ),
      selectedOverrideConfigKey(base, true, false),
    ];

    expect(keys.every((key) => key !== undefined)).toBe(true);
    expect(new Set(keys.map((key) => `${key?.hi}:${key?.lo}`)).size).toBe(
      keys.length,
    );
  });
});
