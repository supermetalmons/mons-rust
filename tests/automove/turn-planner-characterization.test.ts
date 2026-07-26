import { describe, expect, it } from "vitest";

import { AutomoveEngine } from "../../src/automove/automove-engine.js";
import { automoveConfigForGame } from "../../src/automove/selector-config.js";
import { turnEngineConfigForGame } from "../../src/automove/production-selector.js";
import {
  EMPTY_TURN_UTILITY,
  clearTurnEnginePlanCache,
  createTurnUtility,
  discoverTurnOpportunities,
  turnEngineCandidatePlan,
  turnEngineCandidatePlanLive,
  utilityHasNonnegativeDenyGain,
  utilityHasScoreDeltaForce,
  utilityImprovesNonScoreOverrideAxes,
  utilityPassesOverrideGuard,
  utilityStrictlyDominatesOverrideAxes,
  utilitySupportsFamilyFallback,
  utilitySupportsPrimaryAxesEvalTolerance,
  utilitySupportsTemporaryRiskRecovery,
  type TurnAction,
  type TurnEngineConfig,
  type TurnOpportunity,
  type TurnPlan,
} from "../../src/automove/turn-engine.js";
import { GameVariant } from "../../src/engine/config.js";
import { inputArrayFen } from "../../src/engine/fen.js";
import { MonsGame } from "../../src/engine/game.js";

function plannerConfig(game: MonsGame): TurnEngineConfig {
  const base = turnEngineConfigForGame(
    game,
    automoveConfigForGame(game, "fast"),
  );
  return {
    ...base,
    ownSeedCap: 12,
    ownBeam: 3,
    perNodeFamilyCap: 2,
    stepCap: 2,
    opponentSeedCap: 4,
    opponentBeam: 2,
    replySeedCap: 3,
    replyBeam: 2,
    expansionCap: 48,
  };
}

function actionKey(action: TurnAction): string {
  switch (action.kind) {
    case "walk":
    case "safety-retreat":
      return `${action.kind}:${action.actor.i},${action.actor.j}>${action.to.i},${action.to.j}`;
    case "attack":
    case "bomb":
      return `${action.kind}:${action.actor.i},${action.actor.j}>${action.target.i},${action.target.j}`;
    case "spirit-shift":
      return `${action.kind}:${action.actor.i},${action.actor.j}>${action.target.i},${action.target.j}>${action.destination.i},${action.destination.j}`;
    case "move-mana":
      return `${action.kind}:${action.from.i},${action.from.j}>${action.to.i},${action.to.j}`;
    case "score-carry":
      return `${action.kind}:${action.actor.i},${action.actor.j}>${action.step.i},${action.step.j}`;
  }
}

function opportunitySummary(opportunity: TurnOpportunity): string {
  return `${opportunity.family}|${opportunity.kind}|${actionKey(opportunity.action)}`;
}

function planSummary(plan: TurnPlan | undefined): unknown {
  if (plan === undefined) return undefined;
  return {
    actions: plan.actions.map(actionKey),
    chunks: plan.compiledChunks.map(inputArrayFen),
    headFamily: plan.headFamily,
    goalFamily: plan.goalFamily,
    utility: [
      plan.utility.winState,
      plan.utility.avoidImmediateLoss,
      plan.utility.scoreDelta,
      plan.utility.denyGain,
      plan.utility.drainerAttack,
      plan.utility.drainerSafety,
      plan.utility.evalScore,
    ],
  };
}

const EXPECTED_OPENING_OPPORTUNITIES = [
  "safe-supermana-progress|safe-supermana-progress|walk:10,5>9,4",
  "safe-opponent-mana-progress|safe-opponent-mana-progress|walk:10,5>9,5",
  "mana-tempo|mana-tempo|walk:10,5>9,6",
] as const;

const EXPECTED_OPENING_PLAN = {
  actions: ["walk:10,5>9,4", "walk:9,4>8,5"],
  chunks: ["l10,5;l9,4", "l9,4;l8,5"],
  headFamily: "safe-supermana-progress",
  goalFamily: "safe-supermana-progress",
  utility: [0, 1, 616, 0, 0, 2, 941],
} as const;

describe("turn planner characterization", () => {
  it("uses plain utility data with pure comparison helpers", () => {
    const incumbent = createTurnUtility();
    const candidate = createTurnUtility({
      avoidImmediateLoss: 1,
      scoreDelta: 220,
      denyGain: 1,
      drainerSafety: 1,
    });

    expect(Object.getPrototypeOf(candidate)).toBe(Object.prototype);
    expect(Object.isFrozen(EMPTY_TURN_UTILITY)).toBe(true);
    expect(createTurnUtility()).not.toBe(EMPTY_TURN_UTILITY);
    expect(utilityHasNonnegativeDenyGain(candidate)).toBe(true);
    expect(utilitySupportsTemporaryRiskRecovery(candidate)).toBe(true);
    expect(utilityStrictlyDominatesOverrideAxes(candidate, incumbent)).toBe(
      true,
    );
    expect(utilityPassesOverrideGuard(candidate, incumbent)).toBe(true);
    expect(utilitySupportsFamilyFallback(candidate, incumbent)).toBe(true);
    expect(utilityImprovesNonScoreOverrideAxes(candidate, incumbent)).toBe(
      true,
    );
    expect(utilityHasScoreDeltaForce(candidate, incumbent, 220)).toBe(true);
    expect(
      utilitySupportsPrimaryAxesEvalTolerance(candidate, incumbent, 0),
    ).toBe(true);
  });

  it("pins opportunity ordering and the selected plan for a stable opening state", () => {
    const game = new MonsGame(false, GameVariant.Classic);
    const config = plannerConfig(game);
    const engine = new AutomoveEngine({ clock: () => 0 });
    const opportunities = engine.run((execution) =>
      discoverTurnOpportunities(execution, game, game.activeColor, config, 12),
    );
    const plan = engine.run((execution) =>
      turnEngineCandidatePlanLive(execution, game, game.activeColor, config),
    );

    expect(opportunities.map(opportunitySummary)).toEqual(
      EXPECTED_OPENING_OPPORTUNITIES,
    );
    expect(planSummary(plan)).toEqual(EXPECTED_OPENING_PLAN);
  });

  it("returns the same plan with a cold, warm, and rebuilt cache", () => {
    const game = new MonsGame(false, GameVariant.Classic);
    const config = plannerConfig(game);
    const engine = new AutomoveEngine({ clock: () => 0 });

    engine.run(clearTurnEnginePlanCache);
    let engineEntriesAfterCold = 0;
    const cold = planSummary(
      engine.run((execution) => {
        const plan = turnEngineCandidatePlan(
          execution,
          game.copy(),
          game.activeColor,
          config,
        );
        engineEntriesAfterCold = execution.caches.engine.entryCount;
        return plan;
      }),
    );
    expect(engineEntriesAfterCold).toBeGreaterThan(0);
    const warm = planSummary(
      engine.run((execution) =>
        turnEngineCandidatePlan(
          execution,
          game.copy(),
          game.activeColor,
          config,
        ),
      ),
    );
    engine.run(clearTurnEnginePlanCache);
    const rebuilt = planSummary(
      engine.run((execution) =>
        turnEngineCandidatePlan(
          execution,
          game.copy(),
          game.activeColor,
          config,
        ),
      ),
    );

    expect(cold).toEqual(EXPECTED_OPENING_PLAN);
    expect(warm).toEqual(cold);
    expect(rebuilt).toEqual(cold);
  });
});
