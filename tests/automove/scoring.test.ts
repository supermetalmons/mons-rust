import { describe, expect, it } from "vitest";

import {
  Color,
  Consumable,
  MonKind,
  SUPERMANA,
  consumableItem,
  createMon,
  monItem,
  monWithManaItem,
  monWithConsumableItem,
  type Item,
} from "../../src/engine/domain.js";
import { GameVariant } from "../../src/engine/config.js";
import { parseGameFen } from "../../src/engine/fen.js";
import { MonsGame } from "../../src/engine/game.js";
import {
  BALANCED_DISTANCE_SCORING_WEIGHTS,
  DEFAULT_SCORING_WEIGHTS,
  RUNTIME_FAST_BOOLEAN_DRAINER_SCORING_WEIGHTS,
  ScoringEvalContext,
  defineScoringProfile,
  evaluatePreferabilityWithContext,
  evaluatePreferabilityWithWeightsAndExactPolicy,
  scoringProfileId,
  validateScoringProfile,
} from "../../src/automove/scoring.js";
import { createTestAutomoveExecutionContext } from "./execution-context.test-helper.js";
import {
  MAX_HEURISTIC_SCORE,
  MIN_HEURISTIC_SCORE,
} from "../../src/automove/score-math.js";

const SPIRIT_UTILITY_POINTS = 37;
const SPIRIT_UTILITY_WEIGHTS = defineScoringProfile({
  id: "test.spirit-utility",
  base: DEFAULT_SCORING_WEIGHTS,
  material: {
    faintedMon: 0,
    faintedCooldownStep: 0,
    activeMon: 0,
  },
  position: {
    monCloseToCenter: 0,
    spiritCloseToEnemy: 0,
  },
  threat: { spiritActionUtility: SPIRIT_UTILITY_POINTS },
});

function carrierGame(): MonsGame {
  const initial = new MonsGame(false, GameVariant.Classic);
  const state = parseGameFen(initial.fen());
  if (state === undefined) {
    throw new Error("initial game must have a parseable FEN");
  }
  const board = state.board.fork();
  const drainer = {
    kind: MonKind.Drainer,
    color: Color.White,
    cooldown: 0,
  } as const;
  board.delete(board.base(drainer));
  board.set({ i: 8, j: 5 }, monWithManaItem(drainer, SUPERMANA));
  return MonsGame.newSimulationState({
    ...state,
    board,
    whiteScore: 2,
    blackScore: 1,
    whitePotionsCount: 1,
    activeColor: Color.Black,
    monsMovesCount: 2,
    turnNumber: 5,
  });
}

function emptyClassicGame(): MonsGame {
  const game = new MonsGame(false, GameVariant.Classic);
  game.replaceBoardItems([]);
  return game;
}

function heuristicSpiritScore(spiritItem: Item, target?: Item): number {
  const execution = createTestAutomoveExecutionContext();
  const game = emptyClassicGame();
  game.replaceBoardItems(
    target === undefined
      ? [[{ i: 5, j: 5 }, spiritItem]]
      : [
          [{ i: 5, j: 5 }, spiritItem],
          [{ i: 3, j: 5 }, target],
        ],
  );
  return evaluatePreferabilityWithWeightsAndExactPolicy(
    execution,
    game,
    Color.White,
    SPIRIT_UTILITY_WEIGHTS,
    false,
  );
}

describe("scoring evaluation", () => {
  it("bounds extreme valid counters below terminal scores", () => {
    const execution = createTestAutomoveExecutionContext();
    const initial = new MonsGame(false, GameVariant.Classic);
    const state = parseGameFen(initial.fen());
    if (state === undefined) {
      throw new Error("initial game must have a parseable FEN");
    }
    const game = MonsGame.newSimulationState({
      ...state,
      whitePotionsCount: Number.MAX_SAFE_INTEGER,
    });

    expect(
      evaluatePreferabilityWithWeightsAndExactPolicy(
        execution,
        game,
        Color.White,
        DEFAULT_SCORING_WEIGHTS,
        false,
      ),
    ).toBe(MAX_HEURISTIC_SCORE);
    expect(
      evaluatePreferabilityWithWeightsAndExactPolicy(
        execution,
        game,
        Color.Black,
        DEFAULT_SCORING_WEIGHTS,
        false,
      ),
    ).toBe(MIN_HEURISTIC_SCORE);
  });

  it("characterizes heuristic and exact-policy scores", () => {
    const execution = createTestAutomoveExecutionContext();
    const initial = new MonsGame(false, GameVariant.Classic);
    const carrier = carrierGame();
    const observations = [
      evaluatePreferabilityWithWeightsAndExactPolicy(
        execution,
        initial,
        Color.White,
        BALANCED_DISTANCE_SCORING_WEIGHTS,
        false,
      ),
      evaluatePreferabilityWithWeightsAndExactPolicy(
        execution,
        initial,
        Color.Black,
        BALANCED_DISTANCE_SCORING_WEIGHTS,
        true,
      ),
      evaluatePreferabilityWithWeightsAndExactPolicy(
        execution,
        carrier,
        Color.White,
        RUNTIME_FAST_BOOLEAN_DRAINER_SCORING_WEIGHTS,
        false,
      ),
      evaluatePreferabilityWithWeightsAndExactPolicy(
        execution,
        carrier,
        Color.White,
        RUNTIME_FAST_BOOLEAN_DRAINER_SCORING_WEIGHTS,
        true,
      ),
      evaluatePreferabilityWithWeightsAndExactPolicy(
        execution,
        carrier,
        Color.Black,
        RUNTIME_FAST_BOOLEAN_DRAINER_SCORING_WEIGHTS,
        true,
      ),
    ];

    expect(observations).toEqual([940, 940, 944_071, 946_013, -944_630]);
  });

  it.each([true, false])(
    "reuses context-owned state with exact policy %s",
    (allowExactStrategic) => {
      const execution = createTestAutomoveExecutionContext();
      const game = carrierGame();
      const context = new ScoringEvalContext(execution, game, {
        allowExactStrategic,
      });
      expect(context.game).toBe(game);
      expect(context.board).toBe(game.board);
      expect(context.allowExactStrategic).toBe(allowExactStrategic);
      expect(context.boardSummary()).toBe(context.boardSummary());
      expect(context.manaPathSnapshot()).toBe(context.manaPathSnapshot());
      expect(context.exactAnalysis()).toBe(context.exactAnalysis());
      if (!allowExactStrategic) {
        expect(context.exactAnalysis()).toBeUndefined();
      }

      const withContext = evaluatePreferabilityWithContext(
        context,
        Color.White,
        RUNTIME_FAST_BOOLEAN_DRAINER_SCORING_WEIGHTS,
      );
      expect(withContext).toBe(
        evaluatePreferabilityWithWeightsAndExactPolicy(
          execution,
          game,
          Color.White,
          RUNTIME_FAST_BOOLEAN_DRAINER_SCORING_WEIGHTS,
          allowExactStrategic,
        ),
      );
    },
  );

  it.each([
    ["plain spirit", monItem(createMon(MonKind.Spirit, Color.White, 0))],
    [
      "spirit with a consumable",
      monWithConsumableItem(
        createMon(MonKind.Spirit, Color.White, 0),
        Consumable.Potion,
      ),
    ],
    [
      "spirit with mana",
      monWithManaItem(createMon(MonKind.Spirit, Color.White, 0), SUPERMANA),
    ],
  ])(
    "counts only eligible reachable targets for a %s",
    (_description, spiritItem) => {
      const baseline = heuristicSpiritScore(spiritItem);
      const liveMon = monItem(createMon(MonKind.Demon, Color.Black, 0));
      const faintedMon = monItem(createMon(MonKind.Demon, Color.Black, 2));
      const looseConsumable = consumableItem(Consumable.Bomb);

      expect(heuristicSpiritScore(spiritItem, liveMon) - baseline).toBe(
        SPIRIT_UTILITY_POINTS,
      );
      expect(heuristicSpiritScore(spiritItem, faintedMon) - baseline).toBe(0);
      expect(heuristicSpiritScore(spiritItem, looseConsumable) - baseline).toBe(
        SPIRIT_UTILITY_POINTS,
      );
    },
  );

  it("keeps exported scoring profiles frozen singletons", () => {
    expect(Object.isFrozen(BALANCED_DISTANCE_SCORING_WEIGHTS)).toBe(true);
    expect(Object.isFrozen(BALANCED_DISTANCE_SCORING_WEIGHTS.position)).toBe(
      true,
    );
    expect(Object.isFrozen(RUNTIME_FAST_BOOLEAN_DRAINER_SCORING_WEIGHTS)).toBe(
      true,
    );
    expect(scoringProfileId(BALANCED_DISTANCE_SCORING_WEIGHTS)).toBe(
      "balanced-distance",
    );
  });

  it("rejects unsafe custom scoring profiles", () => {
    expect(() =>
      defineScoringProfile({
        id: "test.unsafe",
        base: DEFAULT_SCORING_WEIGHTS,
        material: { confirmedScore: 10_001 },
      }),
    ).toThrow(RangeError);

    expect(() =>
      defineScoringProfile({
        id: "Not a stable id",
        base: DEFAULT_SCORING_WEIGHTS,
      }),
    ).toThrow(RangeError);

    defineScoringProfile({
      id: "test.unique-profile-id",
      base: DEFAULT_SCORING_WEIGHTS,
    });
    expect(() =>
      defineScoringProfile({
        id: "test.unique-profile-id",
        base: DEFAULT_SCORING_WEIGHTS,
        material: { confirmedScore: 999 },
      }),
    ).toThrow(/already registered/u);
  });

  it("requires exact scoring section shapes at runtime", () => {
    const incompleteMaterial: {
      -readonly [
        Key in keyof typeof DEFAULT_SCORING_WEIGHTS.material
      ]?: (typeof DEFAULT_SCORING_WEIGHTS.material)[Key];
    } = { ...DEFAULT_SCORING_WEIGHTS.material };
    delete incompleteMaterial.activeMon;
    expect(() =>
      validateScoringProfile({
        ...DEFAULT_SCORING_WEIGHTS,
        material: incompleteMaterial,
      }),
    ).toThrow(/must contain exactly/u);

    expect(() =>
      validateScoringProfile({
        ...DEFAULT_SCORING_WEIGHTS,
        formula: {
          ...DEFAULT_SCORING_WEIGHTS.formula,
          unexpectedFlag: true,
        },
      }),
    ).toThrow(/must contain exactly/u);

    expect(() =>
      validateScoringProfile({
        ...DEFAULT_SCORING_WEIGHTS,
        material: {
          ...DEFAULT_SCORING_WEIGHTS.material,
          confirmedScore: 999,
        },
      }),
    ).toThrow(/already registered/u);
  });
});
