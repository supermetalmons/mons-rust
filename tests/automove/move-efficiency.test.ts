import { afterEach, beforeEach, describe, expect, it } from "vitest";

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
} from "../../src/engine/model/domain.js";
import { GameVariant } from "../../src/engine/board/config.js";
import { MonsGame } from "../../src/engine/game/mons-game.js";
import { exactSearchStateHash } from "../../src/automove/exact/hash.js";
import {
  clearMoveEfficiencyCache,
  moveEfficiencyDeltaFromBeforeSnapshot,
  moveEfficiencySnapshotUncachedWithHash,
  moveEfficiencySnapshotWithHash,
} from "../../src/automove/root/move-efficiency.js";
import { applyInputsForSearchWithEvents } from "../../src/automove/transitions/simulation.js";
import { createTestAutomoveExecutionContext } from "./execution-context.test-helper.js";
import type { AutomoveExecutionContext } from "../../src/automove/core/execution-context.js";

const OPENING_MOVE: readonly Input[] = [
  { kind: "location", location: { i: 10, j: 5 } },
  { kind: "location", location: { i: 9, j: 4 } },
];

describe("move-efficiency snapshots", () => {
  let execution: AutomoveExecutionContext;

  beforeEach(() => {
    execution = createTestAutomoveExecutionContext();
    clearMoveEfficiencyCache(execution);
  });

  afterEach(() => {
    clearMoveEfficiencyCache(execution);
  });

  it("characterizes cache identity, cache tags, and uncached snapshots", () => {
    const game = new MonsGame(false, GameVariant.Classic);
    const hash = exactSearchStateHash(game);

    const approximate = moveEfficiencySnapshotWithHash(
      execution,
      game,
      Color.White,
      false,
      false,
      hash,
    );
    expect(
      moveEfficiencySnapshotWithHash(execution, game, Color.White, false, false, hash),
    ).toBe(approximate);

    const exact = moveEfficiencySnapshotWithHash(
      execution,
      game,
      Color.White,
      true,
      true,
      hash,
    );
    const opponent = moveEfficiencySnapshotWithHash(
      execution,
      game,
      Color.Black,
      false,
      false,
      hash,
    );
    const uncached = moveEfficiencySnapshotUncachedWithHash(
      execution,
      game,
      Color.White,
      false,
      false,
      hash,
    );

    expect(approximate).toEqual({
      myBestCarrierSteps: 15,
      opponentBestCarrierSteps: 15,
      myBestDrainerToManaSteps: 2,
      opponentBestDrainerToManaSteps: 2,
      myCarrierCount: 0,
      opponentCarrierCount: 0,
      mySpiritOnBase: true,
      opponentSpiritOnBase: true,
      mySpiritActionTargets: 0,
      opponentSpiritActionTargets: 0,
      mySameTurnScoreValue: 0,
      opponentSameTurnScoreValue: 0,
      mySameTurnOpponentManaScoreValue: 0,
      opponentSameTurnOpponentManaScoreValue: 0,
      mySafeSupermanaProgress: false,
      opponentSafeSupermanaProgress: false,
      mySafeOpponentManaProgress: false,
      opponentSafeOpponentManaProgress: false,
      mySafeSupermanaProgressSteps: 15,
      opponentSafeSupermanaProgressSteps: 15,
      mySafeOpponentManaProgressSteps: 15,
      opponentSafeOpponentManaProgressSteps: 15,
    });
    expect(exact).toEqual({
      ...approximate,
      myBestDrainerToManaSteps: 3,
      opponentBestDrainerToManaSteps: 3,
      opponentSpiritActionTargets: 6,
    });
    expect(opponent).toEqual(approximate);
    expect(exact).not.toBe(approximate);
    expect(opponent).not.toBe(approximate);
    expect(uncached).not.toBe(approximate);
    expect(uncached).toEqual(approximate);

    clearMoveEfficiencyCache(execution);
    const rebuilt = moveEfficiencySnapshotWithHash(
      execution,
      game,
      Color.White,
      false,
      false,
      hash,
    );
    expect(rebuilt).not.toBe(approximate);
    expect(rebuilt).toEqual(approximate);
  });

  it("uses each cooldown state's distinct hash for its after snapshot", () => {
    const awake = new MonsGame(false, GameVariant.Classic);
    const whiteSpirit = createMon(MonKind.Spirit, Color.White, 0);
    const spiritBase = awake.board.base(whiteSpirit);
    const cooling = awake.copy();
    const coolingBoard = cooling.board.fork();
    coolingBoard.set(spiritBase, monItem(createMon(MonKind.Spirit, Color.White, 1)));
    cooling.replaceBoardItems(coolingBoard.entries());

    const awakeHash = exactSearchStateHash(awake);
    const coolingHash = exactSearchStateHash(cooling);
    expect(coolingHash).not.toEqual(awakeHash);

    const before = moveEfficiencySnapshotUncachedWithHash(
      execution,
      awake,
      Color.White,
      false,
      false,
      awakeHash,
    );
    const options = {
      isRoot: false,
      applyBacktrackPenalty: false,
      applyRootManaHandoffGuard: false,
      rootBacktrackPenalty: 0,
      rootManaHandoffPenalty: 0,
      includeTacticalExact: false,
      includeStrategicExact: false,
    } as const;

    const awakeDelta = moveEfficiencyDeltaFromBeforeSnapshot(
      execution,
      awake,
      awake,
      Color.White,
      [],
      before,
      awakeHash,
      options,
    );
    const coolingDelta = moveEfficiencyDeltaFromBeforeSnapshot(
      execution,
      awake,
      cooling,
      Color.White,
      [],
      before,
      coolingHash,
      options,
    );

    expect(awakeDelta).toBe(0);
    expect(coolingDelta).toBe(90);
  });

  it("observes live carriers and consumed spirits without counting fainted carriers", () => {
    const game = new MonsGame(false, GameVariant.Classic);
    const whiteSpirit = createMon(MonKind.Spirit, Color.White, 0);
    const whiteSpiritBase = game.board.base(whiteSpirit);
    const whiteSpiritAway = { i: 5, j: 5 };
    const whiteCarrierNear = { i: 2, j: 2 };
    const whiteCarrierFar = { i: 4, j: 4 };
    const blackCarrier = { i: 8, j: 8 };
    const faintedWhiteCarrier = { i: 1, j: 1 };
    const board = game.board.fork();

    for (const location of [
      whiteSpiritAway,
      whiteCarrierNear,
      whiteCarrierFar,
      blackCarrier,
      faintedWhiteCarrier,
    ]) {
      board.delete(location);
    }
    board.delete(whiteSpiritBase);
    board.set(whiteSpiritAway, monWithConsumableItem(whiteSpirit, Consumable.Bomb));
    board.set(
      whiteCarrierNear,
      monWithManaItem(
        createMon(MonKind.Drainer, Color.White, 0),
        regularMana(Color.White),
      ),
    );
    board.set(
      whiteCarrierFar,
      monWithManaItem(
        createMon(MonKind.Angel, Color.White, 0),
        regularMana(Color.Black),
      ),
    );
    board.set(
      blackCarrier,
      monWithManaItem(
        createMon(MonKind.Mystic, Color.Black, 0),
        regularMana(Color.Black),
      ),
    );
    board.set(
      faintedWhiteCarrier,
      monWithManaItem(
        createMon(MonKind.Demon, Color.White, 2),
        regularMana(Color.White),
      ),
    );
    game.replaceBoardItems(board.entries());

    const snapshot = moveEfficiencySnapshotUncachedWithHash(
      execution,
      game,
      Color.White,
      false,
      false,
      exactSearchStateHash(game),
    );

    expect(snapshot.myCarrierCount).toBe(2);
    expect(snapshot.opponentCarrierCount).toBe(1);
    expect(snapshot.myBestCarrierSteps).toBe(2);
    expect(snapshot.opponentBestCarrierSteps).toBe(2);
    expect(snapshot.mySpiritOnBase).toBe(false);
    expect(snapshot.opponentSpiritOnBase).toBe(true);
  });

  it("characterizes the weighted delta for a real transition", () => {
    const game = new MonsGame(false, GameVariant.Classic);
    const applied = applyInputsForSearchWithEvents(game, OPENING_MOVE);
    expect(applied).toBeDefined();
    if (applied === undefined) return;

    const before = moveEfficiencySnapshotWithHash(
      execution,
      game,
      Color.White,
      true,
      true,
      exactSearchStateHash(game),
    );
    const delta = moveEfficiencyDeltaFromBeforeSnapshot(
      execution,
      game,
      applied.game,
      Color.White,
      applied.events,
      before,
      exactSearchStateHash(applied.game),
      {
        isRoot: true,
        applyBacktrackPenalty: true,
        applyRootManaHandoffGuard: true,
        rootBacktrackPenalty: 120,
        rootManaHandoffPenalty: 80,
        includeTacticalExact: true,
        includeStrategicExact: true,
      },
    );

    expect(delta).toBe(34);
    expect(game.fen()).toBe(new MonsGame(false, GameVariant.Classic).fen());
  });
});
