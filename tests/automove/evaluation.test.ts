import { describe, expect, it } from "vitest";

import { tryLoadPosition } from "../../src/automove/bridge.js";
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
} from "../../src/automove/board.js";
import { evaluateWithTables } from "../../src/automove/evaluation.js";
import {
  createEvalTables,
  normalizeEvalWeights,
  type EvalWeights,
} from "../../src/automove/evaluation-weights.js";
import { FastPosition } from "../../src/automove/state.js";
import { DEFAULT_GAME_VARIANT } from "../../src/engine/board/config.js";
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
} from "../../src/engine/model/domain.js";
import {
  BOARD_CELLS,
  BOARD_SIZE,
  location,
  locationIndex,
} from "../../src/engine/board/geometry.js";
import {
  expectFastPositionInvariants,
  gameWith,
  locationInput,
  resetFastPosition,
} from "./test-helper.js";

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
  manaStepQueue1: 0,
  manaStepQueue2: 0,
  manaStepQueue3: 0,
  manaStepQueue4: 0,
  manaStepQueue5: 0,
  manaStepWinThreat: 0,
  drainerTripTurn1: 0,
  drainerTripTurn2: 0,
  drainerTripTurn3: 0,
  drainerTripTurn4: 0,
  supermanaCarrier: 0,
  scoreShape10: 0,
  scoreShape20: 0,
  scoreShape21: 0,
  scoreShape30: 0,
  scoreShape31: 0,
  scoreShape32: 0,
  tripGradient: 0,
  raceHalfTurn: 0,
  threatMoverScaleSpare: 100,
  threatMoverScaleFew: 100,
  tripTwoPointScale: 100,
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

function makePosition(
  placements: readonly CellPlacement[],
  overrides: Partial<{
    whiteScore: number;
    blackScore: number;
    active: number;
    monsMoves: number;
    manaMoves: number;
  }> = {},
): FastPosition {
  const position = new FastPosition();
  const cells = new Uint16Array(BOARD_CELLS);
  for (const [row, column, cell] of placements) {
    cells[boardIndex([row, column])] = cell;
  }
  resetFastPosition(position, {
    cells,
    squares: fastSquaresForVariant(DEFAULT_GAME_VARIANT),
    ...overrides,
  });
  expectFastPositionInvariants(position);
  return position;
}

function weights(overrides: Partial<EvalWeights>): EvalWeights {
  return { ...ZERO_WEIGHTS, ...overrides };
}

function evaluate(position: FastPosition, values: EvalWeights): number {
  return evaluateWithTables(position, createEvalTables(normalizeEvalWeights(values)));
}

function mon(kind: number, color: number, cooldown = 0, consumable = 0): number {
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
  const withoutWhiteValue = evaluate(withoutWhite, controlWeights);

  expect(evaluate(withFaintedWhite, controlWeights)).toBe(withoutWhiteValue);
  expect(evaluate(withAwakeWhite, controlWeights)).not.toBe(withoutWhiteValue);
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
        expect(manaScoreValue(packedMana, playerId)).toBe(expectedScores[playerId]);
      }
    }
  });

  it("ignores fainted drainers in loose-mana control", () => {
    expectFaintedDrainerIgnored(MANA_WHITE, [9, 5], weights({ manaDrainerControl: 1 }));
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
      const targetCell: CellPlacement = [target[0], target[1], mon(KIND_DRAINER, 0)];
      const onBase = makePosition([targetCell, [base[0], base[1], mon(kind, 1)]]);
      const atActionOrigin = makePosition([
        targetCell,
        [actionOrigin[0], actionOrigin[1], mon(kind, 1)],
      ]);

      expect(evaluate(onBase, threatWeights)).toBe(0);
      expect(evaluate(atActionOrigin, threatWeights)).toBe(-1);
    },
  );

  it("keeps bomb attacks from mon bases immediate", () => {
    const position = makePosition([
      [0, 3, mon(KIND_MYSTIC, 1, 0, CONS_BOMB)],
      [2, 5, mon(KIND_DRAINER, 0)],
    ]);

    expect(evaluate(position, weights({ drainerThreatImmediate: 1 }))).toBe(-1);
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

    expect(evaluate(onMidpoint, threatWeights)).toBe(-5);
    expect(evaluate(blockedMidpoint, threatWeights)).toBe(-4);
  });

  it("keeps a Demon's own prohibited midpoint unavailable", () => {
    const position = makePosition([
      [4, 5, mon(KIND_DRAINER, 0)],
      [5, 5, mon(KIND_DEMON, 1)],
    ]);

    expect(evaluate(position, weights({ drainerThreatWalk: 5 }))).toBe(-4);
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

        expect(evaluate(position, threatWeights)).toBe(0);
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

        expect(evaluate(position, threatWeights)).toBe(0);
      }
      const bombCarrier = makePosition([
        [target[0], target[1], mon(KIND_DRAINER, 0, 0, CONS_BOMB)],
        [actionOrigin[0], actionOrigin[1], mon(kind, 1)],
      ]);

      expect(evaluate(bombCarrier, threatWeights)).toBe(-1);
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

    expect(evaluate(tooFarFromPool, pickupWeights)).toBe(0);
    expect(evaluate(reachablePool, pickupWeights)).toBe(1);
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

    expect(evaluate(emptyHanded, pickupWeights)).toBe(1);
    expect(evaluate(carryingPotion, pickupWeights)).toBe(0);
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
      evaluate(
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
      game.processInput([locationInput(carrierAt), locationInput(pool)], false, false)
        .kind,
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
      evaluate(
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
    const blackManaNearWhitePool = makePosition([[10, 1, makeManaCell(MANA_BLACK)]]);
    expect(evaluate(blackManaNearWhitePool, nearestPool)).toBe(-400);

    const ownPool = weights({ manaToOwnerPool: 800 });
    expect(evaluate(blackManaNearWhitePool, ownPool)).toBe(-72);
  });

  it("ignores supermana when drawing mana toward a pool", () => {
    const nearestPool = weights({ manaToNearestPool: 800 });
    const superNearPool = makePosition([[10, 1, makeManaCell(MANA_SUPER)]]);

    expect(evaluate(superNearPool, nearestPool)).toBe(0);
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
      evaluate(position, custom),
    );
  });

  it("rejects weights outside the validated range", () => {
    const position = makePosition([[10, 1, makeManaCell(MANA_BLACK)]]);

    expect(() =>
      evaluate(position, weights({ manaToNearestPool: 3_000_000_000 })),
    ).toThrow(RangeError);
  });

  it("prices the free mana step as a queue position with a cliff near the pool", () => {
    const queue = weights({
      manaStepQueue1: 3_600,
      manaStepQueue2: 2_500,
      manaStepQueue3: 1_050,
      manaStepQueue4: 800,
      manaStepQueue5: 800,
    });

    // Column c on row 10 is exactly c steps from the white pool at (10, 0).
    const expected = [3_600, 3_600, 2_500, 1_050, 800, 800];
    for (let distance = 1; distance <= 5; distance += 1) {
      const position = makePosition([[10, distance, makeManaCell(MANA_WHITE)]]);
      expect(evaluate(position, queue), `distance ${distance}`).toBe(
        expected[distance] ?? 0,
      );
    }

    const black = makePosition([[10, 1, makeManaCell(MANA_BLACK)]]);
    expect(evaluate(black, queue)).toBe(-3_600);
    const supermana = makePosition([[10, 1, makeManaCell(MANA_SUPER)]]);
    expect(evaluate(supermana, queue)).toBe(0);

    const combined = weights({ manaStepQueue2: 2_500, manaToNearestPool: 800 });
    expect(evaluate(makePosition([[10, 2, makeManaCell(MANA_WHITE)]]), combined)).toBe(
      2_500 + Math.trunc(800 / 3),
    );
  });

  it("buckets the fused drainer trip by the turns it still needs", () => {
    const trip = weights({
      drainerTripTurn1: 4_200,
      drainerTripTurn2: 2_200,
      drainerTripTurn3: 1_000,
      drainerTripTurn4: 400,
    });

    // Pick-up distance plus the mana's pool distance, against the mover's remaining steps.
    const withinTurn = makePosition([
      [10, 4, mon(KIND_DRAINER, 0)],
      [10, 1, makeManaCell(MANA_WHITE)],
    ]);
    expect(evaluate(withinTurn, trip)).toBe(4_200);

    const oneTurnShort = makePosition([
      [5, 5, mon(KIND_DRAINER, 0)],
      [10, 1, makeManaCell(MANA_WHITE)],
    ]);
    expect(evaluate(oneTurnShort, trip)).toBe(2_200);

    const spentMoves = makePosition(
      [
        [10, 4, mon(KIND_DRAINER, 0)],
        [10, 1, makeManaCell(MANA_WHITE)],
      ],
      { monsMoves: 4 },
    );
    expect(evaluate(spentMoves, trip)).toBe(2_200);

    const carrying = makePosition([
      [10, 2, makeMonCell(KIND_DRAINER, 0, 0, MANA_WHITE, 0)],
    ]);
    expect(evaluate(carrying, trip)).toBe(4_200);
  });

  it("grades the fused drainer trip inside a turn bucket by its remaining steps", () => {
    const gradient = weights({ tripGradient: 100 });

    // Three steps to the mana plus one from the mana to the pool.
    const trip = makePosition([
      [10, 4, mon(KIND_DRAINER, 0)],
      [10, 1, makeManaCell(MANA_WHITE)],
    ]);
    expect(evaluate(trip, gradient)).toBe(-400);

    const closer = makePosition([
      [10, 2, mon(KIND_DRAINER, 0)],
      [10, 1, makeManaCell(MANA_WHITE)],
    ]);
    expect(evaluate(closer, gradient)).toBe(-200);

    // With no mana to fetch the trip is unreachable, and the gradient saturates.
    const noMana = makePosition([[10, 4, mon(KIND_DRAINER, 0)]]);
    expect(evaluate(noMana, gradient)).toBe(-1200);
  });

  it("prices the tempo lead in half turns once one side is close to the target", () => {
    const race = weights({ scoreUnit: 12_000, raceHalfTurn: 100 });
    const carrier: CellPlacement = [
      10,
      2,
      makeMonCell(KIND_DRAINER, 0, 0, MANA_WHITE, 0),
    ];

    // White delivers next turn, Black has no channel at all, and White is to move: the lead
    // saturates the table.
    const late = makePosition([carrier], { whiteScore: 3 });
    expect(evaluate(late, race)).toBe(3 * 12_000 + 600);

    // The same board with the target still four points away is not yet a race.
    const early = makePosition([carrier]);
    expect(evaluate(early, race)).toBe(0);

    // The point that opens the band has to stay worth more than the correction it switches on.
    expect(() =>
      createEvalTables(weights({ scoreUnit: 12_000, raceHalfTurn: 2_100 })),
    ).toThrow(RangeError);
  });

  it("discounts a threat the threatened side can still answer", () => {
    const threat = weights({
      drainerThreatImmediate: 1_000,
      threatMoverScaleSpare: 25,
    });
    const placements: readonly CellPlacement[] = [
      [2, 5, mon(KIND_DRAINER, 0)],
      [4, 3, mon(KIND_MYSTIC, 1)],
    ];

    // White holds sub-moves, so the attack has to survive White's own turn first.
    expect(evaluate(makePosition(placements), threat)).toBe(-250);
    expect(evaluate(makePosition(placements, { monsMoves: 5 }), threat)).toBe(-1_000);
    expect(evaluate(makePosition(placements, { active: 1 }), threat)).toBe(-1_000);
  });

  it("weighs a two-point trip against the shortest one", () => {
    const trip = {
      drainerTripTurn1: 4_200,
      drainerTripTurn2: 2_200,
      tripGradient: 100,
    };
    // Own mana four fused steps away against the other side's mana six away: the near item is
    // worth one point, the far one two.
    const placements: readonly CellPlacement[] = [
      [10, 4, mon(KIND_DRAINER, 0)],
      [10, 1, makeManaCell(MANA_WHITE)],
      [10, 7, makeManaCell(MANA_BLACK)],
    ];
    const position = makePosition(placements);

    // At the neutral scale both plans are priced by the same table, so the shorter one wins.
    expect(evaluate(position, weights({ ...trip, tripTwoPointScale: 100 }))).toBe(
      4_200 - 400,
    );

    // Scaled up, the two-point trip is worth its extra two steps and the turn they cost.
    expect(evaluate(position, weights({ ...trip, tripTwoPointScale: 290 }))).toBe(
      Math.trunc((2_200 * 290) / 100) - 600,
    );

    const onlyTwoPoint = makePosition([
      [10, 4, mon(KIND_DRAINER, 0)],
      [10, 1, makeManaCell(MANA_BLACK)],
    ]);
    expect(evaluate(onlyTwoPoint, weights({ ...trip, tripTwoPointScale: 290 }))).toBe(
      Math.trunc((4_200 * 290) / 100) - 400,
    );
    expect(evaluate(onlyTwoPoint, weights({ ...trip, tripTwoPointScale: 50 }))).toBe(
      Math.trunc((4_200 * 50) / 100) - 400,
    );
  });

  it("marks a side that needs one point and owns mana within mana-step reach", () => {
    const threat = weights({ manaStepWinThreat: 8_000 });
    const placements: readonly CellPlacement[] = [[10, 2, makeManaCell(MANA_WHITE)]];

    expect(evaluate(makePosition(placements, { whiteScore: 4 }), threat)).toBe(8_000);
    expect(evaluate(makePosition(placements, { whiteScore: 3 }), threat)).toBe(0);
    expect(
      evaluate(
        makePosition([[10, 3, makeManaCell(MANA_WHITE)]], { whiteScore: 4 }),
        threat,
      ),
    ).toBe(0);
    expect(
      evaluate(
        makePosition([[10, 2, makeManaCell(MANA_BLACK)]], { blackScore: 4 }),
        threat,
      ),
    ).toBe(-8_000);
  });

  it("counts a scoring mana move onto a drainer occupying the pool", () => {
    const position = makePosition(
      [
        [0, 0, mon(KIND_DRAINER, 0)],
        [0, 1, makeManaCell(MANA_BLACK)],
        [5, 5, mon(KIND_DRAINER, 1, 0, CONS_BOMB)],
      ],
      { blackScore: 4, active: 0 },
    );

    expect(evaluateWithTables(position, createEvalTables(weights({})), 2_500)).toBe(
      -2_500,
    );
  });

  it("keeps equal-point pickup windows independent of board order", () => {
    const tables = createEvalTables(weights({}));
    const first = makePosition(
      [
        [2, 2, mon(KIND_DRAINER, 1)],
        [0, 1, makeManaCell(MANA_BLACK)],
        [1, 2, makeManaCell(MANA_BLACK)],
      ],
      { blackScore: 3, active: 0 },
    );
    const rotated = makePosition(
      [
        [8, 8, mon(KIND_DRAINER, 1)],
        [9, 8, makeManaCell(MANA_BLACK)],
        [10, 9, makeManaCell(MANA_BLACK)],
      ],
      { blackScore: 3, active: 0 },
    );

    expect(evaluateWithTables(first, tables, 2_500)).toBe(-2_500);
    expect(evaluateWithTables(rotated, tables, 2_500)).toBe(-2_500);
  });

  it("separates the carried supermana from an enemy regular mana of equal point value", () => {
    const carrier = weights({ supermanaCarrier: 1_200 });

    expect(
      evaluate(
        makePosition([[5, 5, makeMonCell(KIND_DRAINER, 0, 0, MANA_SUPER, 0)]]),
        carrier,
      ),
    ).toBe(1_200);
    expect(
      evaluate(
        makePosition([[5, 5, makeMonCell(KIND_DRAINER, 0, 0, MANA_BLACK, 0)]]),
        carrier,
      ),
    ).toBe(0);
    expect(
      evaluate(
        makePosition([[5, 5, makeMonCell(KIND_DRAINER, 1, 0, MANA_SUPER, 0)]]),
        carrier,
      ),
    ).toBe(-1_200);
  });

  it("builds the drainer trip table from turn buckets", () => {
    const tables = createEvalTables(
      weights({
        drainerTripTurn1: 4_200,
        drainerTripTurn2: 2_200,
        drainerTripTurn3: 1_000,
        drainerTripTurn4: 400,
      }),
    );
    const expected = [4_200, 2_200, 1_000, 400];
    for (let excess = 0; excess < tables.drainerTrip.length; excess += 1) {
      const turns = 1 + Math.ceil(excess / 5);
      expect(tables.drainerTrip[excess], `excess ${excess}`).toBe(
        expected[Math.min(turns, 4) - 1],
      );
    }
  });

  it("prices a lead by the score pair, antisymmetrically", () => {
    const shape = weights({
      scoreUnit: 12_000,
      scoreShape10: -6_824,
      scoreShape20: -6_978,
      scoreShape21: -3_085,
      scoreShape30: -8_259,
      scoreShape31: -4_275,
      scoreShape32: -1_052,
    });
    const empty: readonly CellPlacement[] = [];
    const cases = [
      [1, 0, -6_824],
      [2, 0, -6_978],
      [2, 1, -3_085],
      [3, 0, -8_259],
      [3, 1, -4_275],
      [3, 2, -1_052],
    ] as const;

    for (const [white, black, expected] of cases) {
      const lead = (white - black) * 12_000 + expected;
      expect(
        evaluate(makePosition(empty, { whiteScore: white, blackScore: black }), shape),
        `${white}-${black}`,
      ).toBe(lead);
      expect(
        evaluate(makePosition(empty, { whiteScore: black, blackScore: white }), shape),
        `${black}-${white}`,
      ).toBe(-lead);
    }

    for (const score of [0, 1, 2, 3]) {
      expect(
        evaluate(makePosition(empty, { whiteScore: score, blackScore: score }), shape),
        `${score}-${score}`,
      ).toBe(0);
    }
  });

  it("keeps a scored point monotone against the score-shape correction", () => {
    const tables = createEvalTables(
      weights({
        scoreUnit: 12_000,
        scoreShape10: -6_824,
        scoreShape20: -6_978,
        scoreShape21: -3_085,
        scoreShape30: -8_259,
        scoreShape31: -4_275,
        scoreShape32: -1_052,
      }),
    );
    const stride = Math.round(Math.sqrt(tables.scoreShape.length));
    for (let own = 0; own + 1 < stride; own += 1) {
      for (let other = 0; other < stride; other += 1) {
        const gain =
          12_000 +
          (tables.scoreShape[(own + 1) * stride + other] ?? 0) -
          (tables.scoreShape[own * stride + other] ?? 0);
        expect(gain, `${own}->${own + 1} against ${other}`).toBeGreaterThan(0);
      }
    }

    expect(() =>
      createEvalTables(weights({ scoreUnit: 12_000, scoreShape10: -20_000 })),
    ).toThrow(RangeError);
  });

  it("rejects derived evaluation values outside their table ranges", () => {
    const fitting = createEvalTables(
      weights({
        drainerThreatImmediate: 1_000_000,
        carrierThreatFactor: 2_147,
        drainerTripTurn1: 1_000_000,
        tripTwoPointScale: 214_748,
      }),
    );
    expect(fitting.threatImmediate[3]).toBe(2_147_000_000);
    expect(fitting.drainerTripTwoPoint[0]).toBe(2_147_480_000);

    expect(() =>
      createEvalTables(
        weights({
          drainerThreatImmediate: 1_000_000,
          carrierThreatFactor: 1_000_000,
        }),
      ),
    ).toThrow(/immediate threat derived value/);

    expect(() =>
      createEvalTables(
        weights({
          drainerTripTurn1: 1_000_000,
          tripTwoPointScale: 1_000_000,
        }),
      ),
    ).toThrow(/two-point trip derived value/);

    expect(() =>
      createEvalTables(
        weights({
          drainerThreatWalk: 999_999,
          carrierThreatFactor: 999_999,
          threatMoverScaleSpare: 999_999,
        }),
      ),
    ).toThrow(/walking threat derived value/);

    expect(() =>
      createEvalTables(
        weights({
          drainerThreatWalk: 849_326,
          carrierThreatFactor: 933_139,
          threatMoverScaleSpare: 709_311,
        }),
      ),
    ).toThrow(/walking threat derived value/);
  });
});
