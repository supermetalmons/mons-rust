import { describe, expect, it } from "vitest";

import { tryLoadPosition } from "../../src/automove/fast/bridge.js";
import {
  CONS_BOMB,
  CONS_BOTH,
  CONS_POTION,
  KIND_DEMON,
  KIND_DRAINER,
  KIND_MYSTIC,
  MANA_BLACK,
  MANA_SUPER,
  MANA_WHITE,
  cellConsumable,
  cellCooldown,
  fastSquaresForVariant,
  makeManaCell,
  makeMonCell,
  manaScoreValue,
} from "../../src/automove/fast/board.js";
import {
  createEvalTables,
  evaluatePosition,
  evaluateWithTables,
  type EvalWeights,
} from "../../src/automove/fast/evaluate.js";
import { FastPosition } from "../../src/automove/fast/position.js";
import { DEFAULT_GAME_VARIANT } from "../../src/engine/config.js";
import {
  Color,
  Consumable,
  Modifier,
  MonKind,
  SUPERMANA,
  colorId,
  consumableItem,
  createMon,
  manaScore,
  manaItem,
  monItem,
  regularMana,
} from "../../src/engine/domain.js";
import {
  BOARD_CELLS,
  BOARD_SIZE,
  location,
  locationIndex,
} from "../../src/engine/geometry.js";
import {
  expectFastPositionInvariants,
  gameWith,
  locationInput,
  resetFastPosition,
} from "./fast.test-helper.js";

const ZERO_WEIGHTS = {
  scoreUnit: 0,
  potion: 0,
  bomb: 0,
  activeMon: 0,
  faintMon: 0,
  faintDrainer: 0,
  faintCooldownStep: 0,
  drainerCloseToMana: 0,
  drainerCloseToOwnPool: 0,
  drainerCloseToSupermana: 0,
  carrierCloseToPool: 0,
  carrierPointBonus: 0,
  carrierScoresThisTurn: 0,
  carrierScoresNextTurn: 0,
  winningCarrier: 0,
  drainerPickupScoresThisTurn: 0,
  manaToOwnerPool: 0,
  manaPointsAttraction: 0,
  manaDrainerControl: 0,
  supermanaDrainerControl: 0,
  monCloseToCenter: 0,
  spiritCloseToEnemy: 0,
  spiritOnOwnBase: 0,
  angelCloseToDrainer: 0,
  angelGuardingDrainer: 0,
  attackerCloseToEnemyDrainer: 0,
  drainerThreatImmediate: 0,
  drainerThreatWalk: 0,
  carrierThreatFactor: 0,
  manaToNearestPool: 0,
} satisfies EvalWeights;

type Coordinates = readonly [row: number, column: number];
type CellPlacement = readonly [row: number, column: number, cell: number];

const ATTACK_CASES = [
  {
    name: "Mystic",
    kind: KIND_MYSTIC,
    base: [0, 3] as const,
    actionOrigin: [4, 3] as const,
    target: [2, 5] as const,
  },
  {
    name: "Demon",
    kind: KIND_DEMON,
    base: [0, 7] as const,
    actionOrigin: [4, 7] as const,
    target: [2, 7] as const,
  },
] as const;

function boardIndex([row, column]: Coordinates): number {
  return row * BOARD_SIZE + column;
}

function makePosition(placements: readonly CellPlacement[]): FastPosition {
  const position = new FastPosition();
  const cells = new Uint16Array(BOARD_CELLS);
  for (const [row, column, cell] of placements) {
    cells[boardIndex([row, column])] = cell;
  }
  resetFastPosition(position, {
    cells,
    squares: fastSquaresForVariant(DEFAULT_GAME_VARIANT),
  });
  expectFastPositionInvariants(position);
  return position;
}

function weights(overrides: Partial<EvalWeights>): EvalWeights {
  return { ...ZERO_WEIGHTS, ...overrides };
}

function mon(
  kind: number,
  color: number,
  cooldown = 0,
  consumable = 0,
): number {
  return makeMonCell(kind, color, cooldown, 0, consumable);
}

function expectFaintedDrainerIgnored(
  mana: number,
  manaAt: Coordinates,
  controlWeights: EvalWeights,
): void {
  const blackDrainer: CellPlacement = [1, 5, mon(KIND_DRAINER, 1)];
  const looseMana: CellPlacement = [manaAt[0], manaAt[1], makeManaCell(mana)];
  const withoutWhite = makePosition([blackDrainer, looseMana]);
  const withFaintedWhite = makePosition([
    blackDrainer,
    looseMana,
    [10, 5, mon(KIND_DRAINER, 0, 1)],
  ]);
  const withAwakeWhite = makePosition([
    blackDrainer,
    looseMana,
    [10, 5, mon(KIND_DRAINER, 0)],
  ]);
  const withoutWhiteValue = evaluatePosition(withoutWhite, controlWeights);

  expect(evaluatePosition(withFaintedWhite, controlWeights)).toBe(
    withoutWhiteValue,
  );
  expect(evaluatePosition(withAwakeWhite, controlWeights)).not.toBe(
    withoutWhiteValue,
  );
}

describe("fast position evaluation", () => {
  it("matches canonical mana scoring for every mana and player", () => {
    const cases = [
      [regularMana(Color.White), MANA_WHITE, [1, 2]],
      [regularMana(Color.Black), MANA_BLACK, [2, 1]],
      [SUPERMANA, MANA_SUPER, [2, 2]],
    ] as const;
    const players = [Color.White, Color.Black] as const;

    for (const [mana, packedMana, expectedScores] of cases) {
      for (const player of players) {
        const playerId = colorId(player);
        expect(manaScore(mana, player)).toBe(expectedScores[playerId]);
        expect(manaScoreValue(packedMana, playerId)).toBe(
          expectedScores[playerId],
        );
      }
    }
  });

  it("ignores fainted drainers in loose-mana control", () => {
    expectFaintedDrainerIgnored(
      MANA_WHITE,
      [9, 5],
      weights({ manaDrainerControl: 1 }),
    );
    expectFaintedDrainerIgnored(
      MANA_SUPER,
      [5, 5],
      weights({ supermanaDrainerControl: 1 }),
    );
  });

  it.each(ATTACK_CASES)(
    "does not treat a $name on its base as an immediate action threat",
    ({ kind, base, actionOrigin, target }) => {
      const threatWeights = weights({ drainerThreatImmediate: 1 });
      const targetCell: CellPlacement = [
        target[0],
        target[1],
        mon(KIND_DRAINER, 0),
      ];
      const onBase = makePosition([
        targetCell,
        [base[0], base[1], mon(kind, 1)],
      ]);
      const atActionOrigin = makePosition([
        targetCell,
        [actionOrigin[0], actionOrigin[1], mon(kind, 1)],
      ]);

      expect(evaluatePosition(onBase, threatWeights)).toBe(0);
      expect(evaluatePosition(atActionOrigin, threatWeights)).toBe(-1);
    },
  );

  it("keeps bomb attacks from mon bases immediate", () => {
    const position = makePosition([
      [0, 3, mon(KIND_MYSTIC, 1, 0, CONS_BOMB)],
      [2, 5, mon(KIND_DRAINER, 0)],
    ]);

    expect(
      evaluatePosition(position, weights({ drainerThreatImmediate: 1 })),
    ).toBe(-1);
  });

  it("lets a Demon vacate its own attack midpoint while approaching an origin", () => {
    const target: CellPlacement = [2, 7, mon(KIND_DRAINER, 0)];
    const onMidpoint = makePosition([target, [3, 7, mon(KIND_DEMON, 1)]]);
    const blockedMidpoint = makePosition([
      target,
      [4, 6, mon(KIND_DEMON, 1)],
      [3, 7, makeManaCell(MANA_WHITE)],
    ]);
    const threatWeights = weights({ drainerThreatWalk: 5 });

    expect(evaluatePosition(onMidpoint, threatWeights)).toBe(-5);
    expect(evaluatePosition(blockedMidpoint, threatWeights)).toBe(-4);
  });

  it("keeps a Demon's own prohibited midpoint unavailable", () => {
    const position = makePosition([
      [4, 5, mon(KIND_DRAINER, 0)],
      [5, 5, mon(KIND_DEMON, 1)],
    ]);

    expect(evaluatePosition(position, weights({ drainerThreatWalk: 5 }))).toBe(
      -4,
    );
  });

  it.each(ATTACK_CASES)(
    "does not grant intrinsic $name actions to non-Bomb consumable carriers",
    ({ kind, actionOrigin, target }) => {
      const threatWeights = weights({ drainerThreatImmediate: 1 });
      for (const consumable of [CONS_POTION, CONS_BOTH]) {
        const position = makePosition([
          [target[0], target[1], mon(KIND_DRAINER, 0)],
          [actionOrigin[0], actionOrigin[1], mon(kind, 1, 0, consumable)],
        ]);

        expect(evaluatePosition(position, threatWeights)).toBe(0);
      }
    },
  );

  it.each(ATTACK_CASES)(
    "does not treat non-Bomb consumable carriers as $name attack targets",
    ({ kind, actionOrigin, target }) => {
      const threatWeights = weights({ drainerThreatImmediate: 1 });
      for (const consumable of [CONS_POTION, CONS_BOTH]) {
        const position = makePosition([
          [target[0], target[1], mon(KIND_DRAINER, 0, 0, consumable)],
          [actionOrigin[0], actionOrigin[1], mon(kind, 1)],
        ]);

        expect(evaluatePosition(position, threatWeights)).toBe(0);
      }
      const bombCarrier = makePosition([
        [target[0], target[1], mon(KIND_DRAINER, 0, 0, CONS_BOMB)],
        [actionOrigin[0], actionOrigin[1], mon(kind, 1)],
      ]);

      expect(evaluatePosition(bombCarrier, threatWeights)).toBe(-1);
    },
  );

  it("requires the pickup and pool route to fit in the remaining moves", () => {
    const tooFarFromPool = makePosition([
      [5, 3, mon(KIND_DRAINER, 0)],
      [5, 4, makeManaCell(MANA_WHITE)],
    ]);
    resetFastPosition(tooFarFromPool, { monsMoves: 2 });
    const reachablePool = makePosition([
      [8, 1, mon(KIND_DRAINER, 0)],
      [9, 0, makeManaCell(MANA_WHITE)],
    ]);
    resetFastPosition(reachablePool, { monsMoves: 3 });
    const pickupWeights = weights({ drainerPickupScoresThisTurn: 1 });

    expect(evaluatePosition(tooFarFromPool, pickupWeights)).toBe(0);
    expect(evaluatePosition(reachablePool, pickupWeights)).toBe(1);
  });

  it("requires an empty-handed drainer for the pickup bonus", () => {
    const emptyHanded = makePosition([
      [8, 1, mon(KIND_DRAINER, 0)],
      [9, 0, makeManaCell(MANA_WHITE)],
    ]);
    resetFastPosition(emptyHanded, { monsMoves: 3 });
    const carryingPotion = makePosition([
      [8, 1, mon(KIND_DRAINER, 0, 0, CONS_POTION)],
      [9, 0, makeManaCell(MANA_WHITE)],
    ]);
    resetFastPosition(carryingPotion, { monsMoves: 3 });
    const pickupWeights = weights({ drainerPickupScoresThisTurn: 1 });

    expect(evaluatePosition(emptyHanded, pickupWeights)).toBe(1);
    expect(evaluatePosition(carryingPotion, pickupWeights)).toBe(0);
  });

  it("values a canonically reachable fainted mana carrier that can score", () => {
    const manaFrom = location(2, 1);
    const carrierAt = location(1, 1);
    const pool = location(0, 0);
    const game = gameWith(
      [
        [manaFrom, manaItem(regularMana(Color.White))],
        [carrierAt, monItem(createMon(MonKind.Drainer, Color.Black, 2))],
      ],
      { blackScore: 3 },
    );

    expect(
      game.processInput(
        [locationInput(manaFrom), locationInput(carrierAt)],
        false,
        false,
      ).kind,
    ).toBe("events");
    expect(game.activeColor).toBe(Color.Black);

    const position = new FastPosition();
    expect(tryLoadPosition(position, game, 40)).toBe(true);
    expect(
      evaluatePosition(
        position,
        weights({
          faintDrainer: 10,
          carrierPointBonus: 3,
          carrierScoresThisTurn: 5,
          winningCarrier: 7,
        }),
      ),
    ).toBe(-13);

    expect(
      game.processInput(
        [locationInput(carrierAt), locationInput(pool)],
        false,
        false,
      ).kind,
    ).toBe("events");
    expect(game.blackScore).toBe(5);
  });

  it("values and detects a canonically reachable fainted Bomb carrier", () => {
    const spiritAt = location(5, 3);
    const consumableAt = location(5, 5);
    const carrierAt = location(5, 6);
    const targetAt = location(5, 8);
    const game = gameWith([
      [spiritAt, monItem(createMon(MonKind.Spirit, Color.White, 0))],
      [consumableAt, consumableItem(Consumable.BombOrPotion)],
      [carrierAt, monItem(createMon(MonKind.Demon, Color.White, 2))],
      [targetAt, monItem(createMon(MonKind.Drainer, Color.Black, 0))],
    ]);

    expect(
      game.processInput(
        [
          locationInput(spiritAt),
          locationInput(consumableAt),
          locationInput(carrierAt),
          { kind: "modifier", modifier: Modifier.SelectBomb },
        ],
        false,
        false,
      ).kind,
    ).toBe("events");

    const position = new FastPosition();
    expect(tryLoadPosition(position, game, 40)).toBe(true);
    const carrier = position.cells[locationIndex(carrierAt)] ?? 0;
    expect(cellCooldown(carrier)).toBe(2);
    expect(cellConsumable(carrier)).toBe(CONS_BOMB);
    expect(
      evaluatePosition(
        position,
        weights({
          faintMon: 10,
          bomb: 3,
          drainerThreatImmediate: 5,
        }),
      ),
    ).toBe(-2);

    expect(
      game.processInput(
        [locationInput(carrierAt), locationInput(targetAt)],
        false,
        false,
      ).kind,
    ).toBe("events");
  });

  it("draws loose mana toward the nearest pool, not the owner's pool", () => {
    const nearestPool = weights({ manaToNearestPool: 800 });

    // (10, 1) is one step from the white pool at (10, 0) and ten from either black pool.
    const blackManaNearWhitePool = makePosition([
      [10, 1, makeManaCell(MANA_BLACK)],
    ]);
    expect(evaluatePosition(blackManaNearWhitePool, nearestPool)).toBe(-400);

    const ownPool = weights({ manaToOwnerPool: 800 });
    expect(evaluatePosition(blackManaNearWhitePool, ownPool)).toBe(-72);
  });

  it("ignores supermana when drawing mana toward a pool", () => {
    const nearestPool = weights({ manaToNearestPool: 800 });
    const superNearPool = makePosition([[10, 1, makeManaCell(MANA_SUPER)]]);

    expect(evaluatePosition(superNearPool, nearestPool)).toBe(0);
  });

  it("builds distance tables that reproduce the inline division", () => {
    for (const weight of [0, 1, 7, -7, 350, -1_000_000, 1_000_000]) {
      const tables = createEvalTables(weights({ manaToNearestPool: weight }));
      for (
        let distance = 0;
        distance < tables.manaToNearestPool.length;
        distance += 1
      ) {
        // The table stores Int32, so -0 from truncation lands as 0.
        expect(tables.manaToNearestPool[distance]).toBe(
          Math.trunc(weight / (distance + 1)) | 0,
        );
      }
    }
  });

  it("evaluates identically through prebuilt tables", () => {
    const position = makePosition([
      [10, 1, makeManaCell(MANA_BLACK)],
      [5, 4, makeManaCell(MANA_WHITE)],
      [6, 5, mon(KIND_DRAINER, 0)],
      [4, 5, mon(KIND_DRAINER, 1)],
      [3, 3, mon(KIND_MYSTIC, 1)],
    ]);
    const custom = weights({
      manaToNearestPool: 800,
      manaToOwnerPool: 170,
      manaPointsAttraction: 350,
      carrierCloseToPool: 1600,
      drainerCloseToMana: 330,
      drainerThreatWalk: 240,
      carrierThreatFactor: 2,
    });

    expect(evaluateWithTables(position, createEvalTables(custom))).toBe(
      evaluatePosition(position, custom),
    );
  });

  it("rejects weights outside the validated range", () => {
    const position = makePosition([[10, 1, makeManaCell(MANA_BLACK)]]);

    expect(() =>
      evaluatePosition(position, weights({ manaToNearestPool: 3_000_000_000 })),
    ).toThrow(RangeError);
  });
});
