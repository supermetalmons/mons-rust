import { describe, expect, it } from "vitest";

import { GameVariant } from "../../src/engine/board/config.js";
import { Color, regularMana, type Input } from "../../src/engine/model/domain.js";
import { MonsGame } from "../../src/engine/game/mons-game.js";
import { hash64 } from "../../src/automove/core/hash64.js";
import { compareRankedEvaluatedRootIndices } from "../../src/automove/root/evaluated-ordering.js";
import { type EvaluatedRoot } from "../../src/automove/root/types.js";
import {
  compareRankedChildren,
  type RankedChild,
} from "../../src/automove/search/ordering.js";
import type { MoveClassFlags } from "../../src/automove/root/types.js";
import {
  EMPTY_PACKAGE_META,
  TurnPlanFamily,
  createTurnUtility,
  type TurnAction,
  type TurnPlan,
  type TurnUtility,
} from "../../src/automove/turn/model.js";
import {
  compareActionKeys,
  compareTurnUtilities,
  turnEngineComparePlans,
} from "../../src/automove/turn/ordering.js";

const QUIET_CLASSES: MoveClassFlags = Object.freeze({
  immediateScore: false,
  drainerAttack: false,
  drainerSafetyRecover: false,
  carrierProgress: false,
  material: false,
  quiet: true,
});

function evaluatedRoot(overrides: Partial<EvaluatedRoot> = {}): EvaluatedRoot {
  return {
    rootRank: 0,
    inputs: [],
    game: new MonsGame(false, GameVariant.Classic),
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
    safeSupermanaProgressSteps: 0,
    safeOpponentManaProgressSteps: 0,
    scorePathBestSteps: 0,
    sameTurnScoreWindowValue: 0,
    spiritSetupGain: 0,
    spiritSameTurnScoreSetupNow: false,
    spiritOwnManaSetupNow: false,
    supermanaProgress: false,
    opponentManaProgress: false,
    policyPriority: 0,
    classes: QUIET_CLASSES,
    heuristic: 0,
    events: [],
    stateHash: hash64(0, 0),
    score: 0,
    nodesAfter: 0,
    ...overrides,
  };
}

function rankedChild(
  hashLow: number,
  overrides: Partial<RankedChild> = {},
): RankedChild {
  return {
    game: new MonsGame(false, GameVariant.Classic),
    hash: hash64(0, hashLow),
    heuristic: 0,
    orderingEfficiency: 0,
    tacticalExtensionTrigger: false,
    quietReductionCandidate: true,
    classes: QUIET_CLASSES,
    ...overrides,
  };
}

function plan(utility: TurnUtility, overrides: Partial<TurnPlan> = {}): TurnPlan {
  return {
    actions: [],
    compiledChunks: [],
    endGame: new MonsGame(false, GameVariant.Classic),
    utility,
    headUtility: utility,
    headFamily: TurnPlanFamily.ManaTempo,
    goalFamily: TurnPlanFamily.ManaTempo,
    packageMeta: EMPTY_PACKAGE_META,
    ...overrides,
  };
}

describe("automove ordering contracts", () => {
  it("preserves the opposite search-sort and turn-quality comparator polarities", () => {
    const strongerSearchChild = rankedChild(1, { heuristic: 20 });
    const weakerSearchChild = rankedChild(2, { heuristic: 10 });
    expect(
      compareRankedChildren(strongerSearchChild, weakerSearchChild, true),
    ).toBeLessThan(0);
    expect(
      compareRankedChildren(weakerSearchChild, strongerSearchChild, false),
    ).toBeLessThan(0);

    const strongerUtility = createTurnUtility({ winState: 1 });
    const weakerUtility = createTurnUtility({ evalScore: 10_000 });
    expect(compareTurnUtilities(strongerUtility, weakerUtility)).toBeGreaterThan(0);
    expect(
      turnEngineComparePlans(plan(strongerUtility), plan(weakerUtility)),
    ).toBeGreaterThan(0);
  });

  it("uses the source root index only after score and tactical ties", () => {
    const roots = [
      evaluatedRoot(),
      evaluatedRoot(),
      evaluatedRoot({ winsImmediately: true }),
    ];
    const indices = [1, 0, 2];

    indices.sort((left, right) =>
      compareRankedEvaluatedRootIndices(roots, left, right),
    );

    expect(indices).toEqual([2, 0, 1]);
    expect(compareRankedEvaluatedRootIndices(roots, 0, 1)).toBeLessThan(0);
    expect(compareRankedEvaluatedRootIndices(roots, 1, 0)).toBeGreaterThan(0);
  });

  it("keeps action tags and locations in deterministic ascending key order", () => {
    const actions: TurnAction[] = [
      {
        kind: "safety-retreat",
        actor: { i: 6, j: 0 },
        to: { i: 6, j: 1 },
      },
      {
        kind: "score-carry",
        actor: { i: 5, j: 0 },
        wanted: regularMana(Color.White),
        step: { i: 5, j: 1 },
      },
      { kind: "move-mana", from: { i: 4, j: 0 }, to: { i: 4, j: 1 } },
      { kind: "bomb", actor: { i: 3, j: 0 }, target: { i: 3, j: 1 } },
      {
        kind: "spirit-shift",
        actor: { i: 2, j: 0 },
        target: { i: 2, j: 1 },
        destination: { i: 2, j: 2 },
      },
      { kind: "attack", actor: { i: 1, j: 0 }, target: { i: 1, j: 1 } },
      { kind: "walk", actor: { i: 0, j: 0 }, to: { i: 0, j: 1 } },
    ];

    actions.sort(compareActionKeys);

    expect(actions.map(({ kind }) => kind)).toEqual([
      "walk",
      "attack",
      "spirit-shift",
      "bomb",
      "move-mana",
      "score-carry",
      "safety-retreat",
    ]);
  });

  it("prefers plan family precedence, then fewer actions, then input chains", () => {
    const utility = createTurnUtility();
    const higherFamily = plan(utility, {
      goalFamily: TurnPlanFamily.ImmediateScore,
    });
    const lowerFamily = plan(utility, {
      goalFamily: TurnPlanFamily.ManaTempo,
    });
    expect(turnEngineComparePlans(higherFamily, lowerFamily)).toBeGreaterThan(0);

    const walk: TurnAction = {
      kind: "walk",
      actor: { i: 0, j: 0 },
      to: { i: 0, j: 1 },
    };
    expect(
      turnEngineComparePlans(plan(utility), plan(utility, { actions: [walk] })),
    ).toBeGreaterThan(0);

    const lowerInput: Input = {
      kind: "location",
      location: { i: 0, j: 0 },
    };
    const higherInput: Input = {
      kind: "location",
      location: { i: 1, j: 0 },
    };
    expect(
      turnEngineComparePlans(
        plan(utility, { compiledChunks: [[higherInput]] }),
        plan(utility, { compiledChunks: [[lowerInput]] }),
      ),
    ).toBeGreaterThan(0);
  });
});
