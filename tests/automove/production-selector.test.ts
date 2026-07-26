import { describe, expect, it, vi } from "vitest";

import { GameVariant } from "../../src/engine/config.js";
import { Color, inputChainsEqual } from "../../src/engine/domain.js";
import { parseGameFen } from "../../src/engine/fen.js";
import { MonsGame } from "../../src/engine/game.js";
import type { ReplyRiskHooks } from "../../src/automove/reply-risk.js";
import { rankRootCandidates } from "../../src/automove/root-candidates.js";
import { rootFamily } from "../../src/automove/root-family.js";
import type { EvaluatedRoot } from "../../src/automove/search.js";
import {
  automoveConfigForGame,
  patchAutomoveConfig,
  withProductionPlanner,
} from "../../src/automove/selector-config.js";
import { acceptTurnEngineHeadAfterSearch } from "../../src/automove/production-selector.js";
import type { AutomoveConfig } from "../../src/automove/selector-types.js";
import {
  EMPTY_TURN_UTILITY,
  TurnPlanFamily,
  createTurnUtility,
  type TurnPlan,
} from "../../src/automove/turn-engine.js";
import { createTestAutomoveExecutionContext } from "./execution-context.test-helper.js";

type RootPair = {
  readonly candidate: EvaluatedRoot;
  readonly selected: EvaluatedRoot;
};

type EvaluateTurnEngineRootUtility = NonNullable<
  ReplyRiskHooks["evaluateTurnEngineRootUtility"]
>;

const ZERO_UTILITY = EMPTY_TURN_UTILITY;
const DOMINATING_UTILITY = createTurnUtility({ denyGain: 1 });

function productionConfig(game: MonsGame) {
  const base = automoveConfigForGame(game, "pro");
  const config = patchAutomoveConfig(withProductionPlanner(base), {
    planner: {
      secondaryAnalysis: false,
      selectedFollowupProjection: false,
    },
  });
  const evaluateTurnEngineRootUtility = vi.fn<EvaluateTurnEngineRootUtility>(
    () => ZERO_UTILITY,
  );
  const hooks = Object.freeze({
    evaluateTurnEngineRootUtility,
  }) satisfies ReplyRiskHooks;
  return { config, hooks, evaluateTurnEngineRootUtility };
}

function rootPair(game: MonsGame, config: AutomoveConfig): RootPair {
  const roots = rankRootCandidates(
    createTestAutomoveExecutionContext(),
    game,
    game.activeColor,
    config,
  ).slice(0, 2);
  const selectedRoot = roots[0];
  const candidateRoot = roots[1];
  if (selectedRoot === undefined || candidateRoot === undefined) {
    throw new Error("production acceptance tests require two legal roots");
  }
  return {
    candidate: {
      ...candidateRoot,
      score: candidateRoot.heuristic,
      nodesAfter: 0,
    },
    selected: {
      ...selectedRoot,
      score: selectedRoot.heuristic,
      nodesAfter: 0,
    },
  };
}

function neutralRoot(root: EvaluatedRoot): EvaluatedRoot {
  return {
    ...root,
    score: 0,
    efficiency: 0,
    winsImmediately: false,
    attacksOpponentDrainer: false,
    ownDrainerVulnerable: false,
    ownDrainerWalkVulnerable: false,
    spiritDevelopment: false,
    keepsAwakeSpiritOnBase: false,
    manaHandoffToOpponent: false,
    hasRoundtrip: false,
    scoresSupermanaThisTurn: false,
    scoresOpponentManaThisTurn: false,
    safeSupermanaPickupNow: false,
    safeOpponentManaPickupNow: false,
    safeSupermanaProgressSteps: 99,
    safeOpponentManaProgressSteps: 99,
    scorePathBestSteps: 99,
    sameTurnScoreWindowValue: 0,
    spiritSetupGain: 0,
    spiritSameTurnScoreSetupNow: false,
    spiritOwnManaSetupNow: false,
    supermanaProgress: false,
    opponentManaProgress: false,
    policyPriority: 0,
    classes: {
      immediateScore: false,
      drainerAttack: false,
      drainerSafetyRecover: false,
      carrierProgress: false,
      material: false,
      quiet: true,
    },
  };
}

function neutralRootPair(game: MonsGame, config: AutomoveConfig): RootPair {
  const pair = rootPair(game, config);
  return {
    candidate: neutralRoot(pair.candidate),
    selected: neutralRoot(pair.selected),
  };
}

function planFor(candidate: EvaluatedRoot): TurnPlan {
  const family = rootFamily(candidate);
  return {
    actions: [],
    compiledChunks: [candidate.inputs],
    endGame: candidate.game,
    utility: EMPTY_TURN_UTILITY,
    headUtility: EMPTY_TURN_UTILITY,
    headFamily: family,
    goalFamily: family,
    packageMeta: {
      scoreGain: 0,
      denyGain: 0,
      drainerSafetyDelta: 0,
      spiritOnlySetup: false,
      endsNonnegativeDrainerSafety: true,
      opponentImmediateWindowAfter: 0,
    },
  };
}

function spiritPlanFor(candidate: EvaluatedRoot): TurnPlan {
  return {
    ...planFor(candidate),
    utility: DOMINATING_UTILITY,
    headUtility: DOMINATING_UTILITY,
    headFamily: TurnPlanFamily.SpiritImpact,
    goalFamily: TurnPlanFamily.SpiritImpact,
  };
}

function spiritFixture(
  turnNumber: number,
  options: {
    readonly candidateOwnManaSetup?: boolean;
    readonly selectedSpiritDevelopment?: boolean;
  } = {},
) {
  const initial = new MonsGame(false, GameVariant.Classic);
  const state = parseGameFen(initial.fen());
  if (state === undefined) {
    throw new Error("initial game must have a parseable FEN");
  }
  const game = MonsGame.newSimulationState({ ...state, turnNumber });
  const { config, hooks, evaluateTurnEngineRootUtility } =
    productionConfig(game);
  const pair = neutralRootPair(game, config);
  const candidate: EvaluatedRoot = {
    ...pair.candidate,
    spiritDevelopment: true,
    spiritOwnManaSetupNow: options.candidateOwnManaSetup ?? false,
  };
  const selected: EvaluatedRoot = {
    ...pair.selected,
    spiritDevelopment: options.selectedSpiritDevelopment ?? false,
  };
  evaluateTurnEngineRootUtility.mockClear();
  return {
    game,
    config,
    hooks,
    evaluateTurnEngineRootUtility,
    candidate,
    selected,
    plan: spiritPlanFor(candidate),
  };
}

function expectSpiritDecisionWithoutMutation(
  fixture: ReturnType<typeof spiritFixture>,
  expected: boolean,
  expectedForkCount?: number,
): void {
  const {
    game,
    config,
    hooks,
    evaluateTurnEngineRootUtility,
    candidate,
    selected,
    plan,
  } = fixture;
  const sourceFen = game.fen();
  const candidateFen = candidate.game.fen();
  const selectedFen = selected.game.fen();
  const fork =
    expectedForkCount === undefined ? undefined : vi.spyOn(game, "fork");

  expect(
    acceptTurnEngineHeadAfterSearch(
      createTestAutomoveExecutionContext(),
      game,
      Color.White,
      config,
      [candidate, selected],
      selected.inputs,
      plan,
      hooks,
    ),
  ).toBe(expected);

  expect(evaluateTurnEngineRootUtility).toHaveBeenCalledTimes(2);
  expect(evaluateTurnEngineRootUtility.mock.calls[0]?.[1]).toBe(selected);
  expect(evaluateTurnEngineRootUtility.mock.calls[1]?.[1]).toBe(candidate);
  if (expectedForkCount !== undefined) {
    expect(fork).toHaveBeenCalledTimes(expectedForkCount);
  }
  expect(game.fen()).toBe(sourceFen);
  expect(candidate.game.fen()).toBe(candidateFen);
  expect(selected.game.fen()).toBe(selectedFen);
}

describe("turn-engine head acceptance", () => {
  it("accepts a plan whose head is already the selected root", () => {
    const game = new MonsGame(false, GameVariant.Classic);
    const { config, hooks } = productionConfig(game);
    const { selected } = rootPair(game, config);

    expect(
      acceptTurnEngineHeadAfterSearch(
        createTestAutomoveExecutionContext(),
        game,
        Color.White,
        config,
        [selected],
        selected.inputs,
        planFor(selected),
        hooks,
      ),
    ).toBe(true);
  });

  it("does not replace an immediate win with a non-winning head", () => {
    const game = new MonsGame(false, GameVariant.Classic);
    const { config, hooks } = productionConfig(game);
    const pair = rootPair(game, config);
    const candidate = { ...pair.candidate, winsImmediately: false };
    const selected = { ...pair.selected, winsImmediately: true };

    expect(
      acceptTurnEngineHeadAfterSearch(
        createTestAutomoveExecutionContext(),
        game,
        Color.White,
        config,
        [candidate, selected],
        selected.inputs,
        planFor(candidate),
        hooks,
      ),
    ).toBe(false);
  });

  it("rejects a macro head that does not dominate the selected root", () => {
    const game = new MonsGame(false, GameVariant.Classic);
    const { config, hooks } = productionConfig(game);
    const pair = rootPair(game, config);
    const sourceFen = game.fen();
    const candidateFens = [pair.candidate.game.fen(), pair.selected.game.fen()];
    expect(inputChainsEqual(pair.candidate.inputs, pair.selected.inputs)).toBe(
      false,
    );

    const accepted = acceptTurnEngineHeadAfterSearch(
      createTestAutomoveExecutionContext(),
      game,
      Color.White,
      config,
      [pair.candidate, pair.selected],
      pair.selected.inputs,
      planFor(pair.candidate),
      hooks,
    );

    expect(accepted).toBe(false);
    expect(game.fen()).toBe(sourceFen);
    expect([pair.candidate.game.fen(), pair.selected.game.fen()]).toEqual(
      candidateFens,
    );
  });

  it("rejects a turn-three spirit setup at the ordered safe-mana guard", () => {
    const fixture = spiritFixture(3, { candidateOwnManaSetup: true });

    expect(fixture.config.planner.secondaryAnalysis).toBe(false);
    expect(fixture.config.planner.selectedFollowupProjection).toBe(false);
    expect(fixture.game.playerCanUseAction()).toBe(true);
    expect(fixture.game.playerCanMoveMana()).toBe(true);
    expect(rootFamily(fixture.candidate)).toBe(TurnPlanFamily.SpiritImpact);
    expect(rootFamily(fixture.selected)).toBe(TurnPlanFamily.ManaTempo);

    expectSpiritDecisionWithoutMutation(fixture, false, 0);
  });

  it("accepts a turn-four production spirit-development head", () => {
    const fixture = spiritFixture(4);

    expect(rootFamily(fixture.candidate)).toBe(TurnPlanFamily.SpiritImpact);
    expect(rootFamily(fixture.selected)).toBe(TurnPlanFamily.ManaTempo);
    expect(fixture.candidate.game.playerCanMoveMon()).toBe(true);

    expectSpiritDecisionWithoutMutation(fixture, true);
  });

  it("rejects a turn-four spirit head that regresses its plain sibling", () => {
    const fixture = spiritFixture(4, {
      selectedSpiritDevelopment: true,
    });

    expect(rootFamily(fixture.candidate)).toBe(TurnPlanFamily.SpiritImpact);
    expect(rootFamily(fixture.selected)).toBe(TurnPlanFamily.SpiritImpact);
    expect(fixture.candidate.game.playerCanMoveMon()).toBe(true);

    expectSpiritDecisionWithoutMutation(fixture, false);
  });
});
