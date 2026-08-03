import { describe, expect, it } from "vitest";

import {
  clearExactStateAnalysisCache,
  defaultColorSummary,
  exactBoardHash,
  exactOwnDrainerSafetyScoreWithHash,
  exactOpportunityContext,
  exactSearchStateHash,
  exactStrategicAnalysis,
  exactTurnSummary,
} from "../../src/automove/exact.js";
import { applyInputsForSearchWithEvents } from "../../src/automove/transitions.js";
import { GameVariant } from "../../src/engine/config.js";
import {
  Color,
  Consumable,
  MonKind,
  createMon,
  monItem,
  monWithConsumableItem,
  monWithManaItem,
  regularMana,
  type Input,
  type Item,
} from "../../src/engine/domain.js";
import { MonsGame } from "../../src/engine/game.js";
import { createTestAutomoveExecutionContext } from "./execution-context.test-helper.js";

const OPENING_MOVE: readonly Input[] = [
  { kind: "location", location: { i: 10, j: 5 } },
  { kind: "location", location: { i: 9, j: 4 } },
];

describe("exact analysis execution state", () => {
  it("keeps cooldown states distinct for every mon item shape", () => {
    const itemFactories: readonly ((cooldown: number) => Item)[] = [
      (cooldown) => monItem(createMon(MonKind.Drainer, Color.White, cooldown)),
      (cooldown) =>
        monWithManaItem(
          createMon(MonKind.Drainer, Color.White, cooldown),
          regularMana(Color.Black),
        ),
      (cooldown) =>
        monWithConsumableItem(
          createMon(MonKind.Drainer, Color.White, cooldown),
          Consumable.Potion,
        ),
    ];
    const boardHashes = new Set<string>();
    const searchHashes = new Set<string>();

    for (const itemFactory of itemFactories) {
      const itemBoardHashes = new Set<string>();
      const itemSearchHashes = new Set<string>();
      for (let cooldown = 0; cooldown <= 2; cooldown += 1) {
        const game = new MonsGame(false, GameVariant.Classic);
        game.replaceBoardItems([[{ i: 5, j: 5 }, itemFactory(cooldown)]]);
        const boardHash = exactBoardHash(game.board);
        const searchHash = exactSearchStateHash(game);
        itemBoardHashes.add(`${boardHash.hi}:${boardHash.lo}`);
        itemSearchHashes.add(`${searchHash.hi}:${searchHash.lo}`);
        boardHashes.add(`${boardHash.hi}:${boardHash.lo}`);
        searchHashes.add(`${searchHash.hi}:${searchHash.lo}`);
      }
      expect(itemBoardHashes.size).toBe(3);
      expect(itemSearchHashes.size).toBe(3);
    }

    expect(boardHashes.size).toBe(9);
    expect(searchHashes.size).toBe(9);
  });

  it("keeps awake and cooling Drainer safety isolated in a shared cache", () => {
    const awake = new MonsGame(false, GameVariant.Classic);
    const whiteDrainer = createMon(MonKind.Drainer, Color.White, 0);
    const drainerBase = awake.board.base(whiteDrainer);
    const cooling = awake.copy();
    const coolingBoard = cooling.board.fork();
    coolingBoard.set(
      drainerBase,
      monItem(createMon(MonKind.Drainer, Color.White, 1)),
    );
    cooling.replaceBoardItems(coolingBoard.entries());

    const safety = (
      execution: ReturnType<typeof createTestAutomoveExecutionContext>,
      game: MonsGame,
    ): number =>
      exactOwnDrainerSafetyScoreWithHash(
        execution,
        game.board,
        exactBoardHash(game.board),
        Color.White,
      );

    expect(safety(createTestAutomoveExecutionContext(), awake)).toBe(2);
    expect(safety(createTestAutomoveExecutionContext(), cooling)).toBe(0);

    const awakeFirst = createTestAutomoveExecutionContext();
    expect(safety(awakeFirst, awake)).toBe(2);
    expect(safety(awakeFirst, cooling)).toBe(0);

    const coolingFirst = createTestAutomoveExecutionContext();
    expect(safety(coolingFirst, cooling)).toBe(0);
    expect(safety(coolingFirst, awake)).toBe(2);
  });

  it("preserves the exact board and search hashes for stable game states", () => {
    const execution = createTestAutomoveExecutionContext();
    const game = new MonsGame(false, GameVariant.Classic);
    const sourceFen = game.fen();

    expect(exactBoardHash(game.board)).toEqual({
      hi: 979_734_162,
      lo: 159_047_913,
    });
    expect(exactSearchStateHash(game)).toEqual({
      hi: 4_060_751_631,
      lo: 1_740_825_244,
    });

    const applied = applyInputsForSearchWithEvents(game, OPENING_MOVE);
    expect(applied).toBeDefined();
    if (applied === undefined) return;

    expect(exactBoardHash(applied.game.board)).toEqual({
      hi: 451_172_277,
      lo: 1_334_539_599,
    });
    expect(exactSearchStateHash(applied.game)).toEqual({
      hi: 2_129_505_380,
      lo: 545_061_926,
    });
    expect(exactTurnSummary(execution, applied.game, Color.White)).toEqual({
      canAttackOpponentDrainer: false,
      safeSupermanaProgress: false,
      safeSupermanaProgressSteps: undefined,
      safeOpponentManaProgress: false,
      safeOpponentManaProgressSteps: undefined,
      spiritAssistedSupermanaProgress: false,
      spiritAssistedOpponentManaProgress: false,
      spiritAssistedScore: false,
      spiritAssistedDenial: false,
      sameTurnScoreWindowValue: 0,
      scorePathBestSteps: 6,
    });
    expect(game.fen()).toBe(sourceFen);
  });

  it("preserves the opening exact opportunity summary", () => {
    const execution = createTestAutomoveExecutionContext();
    const game = new MonsGame(false, GameVariant.Classic);

    expect(exactOpportunityContext(execution, game, Color.White)).toEqual({
      budget: {
        remainingMonMoves: 5,
        canUseAction: false,
        canMoveMana: false,
      },
      turn: {
        safeSupermanaProgress: false,
        safeSupermanaProgressSteps: undefined,
        safeOpponentManaProgress: false,
        safeOpponentManaProgressSteps: undefined,
        spiritAssistedScore: false,
        spiritAssistedScoreValue: 0,
        spiritAssistedDenial: false,
        spiritAssistedDenialValue: 0,
        sameTurnScoreWindowValue: 0,
      },
      delta: {
        sameTurnScoreWindowValue: 0,
        spiritGain: 0,
        opponentWindowDenyGain: 0,
        drainerAttackAvailable: false,
        drainerSafety: 2,
        safeSupermanaProgressSteps: undefined,
        safeOpponentManaProgressSteps: undefined,
      },
      opponentCanWinImmediately: false,
    });
  });

  it("keeps exact caches isolated by session and clearable in place", () => {
    const firstExecution = createTestAutomoveExecutionContext();
    const secondExecution = createTestAutomoveExecutionContext();
    const game = new MonsGame(false, GameVariant.Classic);

    const first = exactStrategicAnalysis(firstExecution, game);
    expect(exactStrategicAnalysis(firstExecution, game)).toBe(first);
    expect(firstExecution.caches.session.entryCount).toBeGreaterThan(0);
    expect(secondExecution.caches.session.entryCount).toBe(0);

    const isolated = exactStrategicAnalysis(secondExecution, game);
    expect(isolated).toEqual(first);
    expect(isolated).not.toBe(first);
    expect(secondExecution.caches.session.entryCount).toBeGreaterThan(0);

    clearExactStateAnalysisCache(firstExecution);
    expect(firstExecution.caches.session.entryCount).toBe(0);
    const rebuilt = exactStrategicAnalysis(firstExecution, game);
    expect(rebuilt).toEqual(first);
    expect(rebuilt).not.toBe(first);
  });

  it("returns the neutral exact analysis without caching after timeout", () => {
    const execution = createTestAutomoveExecutionContext();
    const game = new MonsGame(false, GameVariant.Classic);

    const analysis = execution.session.withDeadlineIfAbsent(5, () =>
      exactStrategicAnalysis(execution, game),
    );

    expect(analysis.colorSummary(Color.White)).toEqual(defaultColorSummary());
    expect(analysis.colorSummary(Color.Black)).toEqual(defaultColorSummary());
    expect(execution.caches.session.entryCount).toBe(0);
    expect(execution.session.takePreviousTimeout()).toBe(true);
  });
});
