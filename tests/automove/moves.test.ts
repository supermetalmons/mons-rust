import { describe, expect, it } from "vitest";

import {
  COLOR_COUNT,
  CONS_BOMB,
  CONS_BOTH,
  CONS_POTION,
  KIND_ANGEL,
  KIND_DEMON,
  MANA_BLACK,
  MANA_WHITE,
  fastSquaresForVariant,
  makeConsumableCell,
  makeManaCell,
  makeMonCell,
} from "../../src/automove/board.js";
import { MAX_MOVES, generateMoves } from "../../src/automove/moves.js";
import {
  AUX_NONE,
  MOD_BOMB,
  MOD_NONE,
  MOD_POTION,
  MOVE_MON,
  FastPosition,
  encodeMove,
} from "../../src/automove/state.js";
import { DEFAULT_GAME_VARIANT } from "../../src/engine/board/config.js";
import { BOARD_CELLS, BOARD_SIZE } from "../../src/engine/board/geometry.js";
import { resetFastPosition } from "./test-helper.js";

type Placement = readonly [row: number, column: number, cell: number];
type GoldenMove = readonly [move: number, key: number];

function index(row: number, column: number): number {
  return row * BOARD_SIZE + column;
}

function mon(kind: number, mana = 0, consumable = 0, color = 0): number {
  return makeMonCell(kind, color, 0, mana, consumable);
}

function packed(placements: readonly Placement[]): FastPosition {
  const cells = new Uint16Array(BOARD_CELLS);
  for (const [row, column, cell] of placements) {
    cells[index(row, column)] = cell;
  }
  const position = new FastPosition();
  resetFastPosition(position, {
    cells,
    squares: fastSquaresForVariant(DEFAULT_GAME_VARIANT),
    whiteScore: 0,
    blackScore: 0,
    active: 0,
    monsMoves: 0,
    manaMoves: 0,
    actionsUsed: 0,
    potions: new Int32Array(COLOR_COUNT),
    firstTurn: false,
  });
  return position;
}

function move(
  from: readonly [number, number],
  to: readonly [number, number],
  key: number,
  modifier = MOD_NONE,
): GoldenMove {
  return [
    encodeMove(
      MOVE_MON,
      index(from[0], from[1]),
      index(to[0], to[1]),
      AUX_NONE,
      modifier,
    ),
    key,
  ];
}

const GOLDEN_CASES = [
  {
    name: "plain mon preserves occupancy, modifier, and own-base ordering",
    placements: [
      [9, 4, mon(KIND_ANGEL)],
      [8, 3, makeConsumableCell(CONS_BOTH)],
      [8, 5, makeManaCell(MANA_WHITE)],
      [9, 5, mon(KIND_DEMON, 0, 0, 1)],
    ] as const,
    expected: [
      move([9, 4], [8, 3], 1 << 14, MOD_BOMB),
      move([9, 4], [8, 3], 1 << 14, MOD_POTION),
      move([9, 4], [8, 4], 0),
      move([9, 4], [9, 3], 0),
      move([9, 4], [10, 4], 0),
    ],
  },
  {
    name: "mana carrier preserves pool, pickup, and distance keys",
    placements: [
      [1, 1, mon(KIND_ANGEL, MANA_WHITE)],
      [0, 1, makeManaCell(MANA_BLACK)],
      [1, 0, makeConsumableCell(CONS_BOTH)],
      [2, 2, mon(KIND_DEMON, 0, 0, 1)],
    ] as const,
    expected: [
      move([1, 1], [0, 0], 2_365_440),
      move([1, 1], [0, 1], 70_656),
      move([1, 1], [0, 2], 2_048),
      move([1, 1], [1, 0], 20_480),
      move([1, 1], [1, 2], 2_048),
      move([1, 1], [2, 0], 2_048),
      move([1, 1], [2, 1], 2_048),
    ],
  },
  {
    name: "consumable carrier preserves fixed-consumable suppression",
    placements: [
      [5, 1, mon(KIND_ANGEL, 0, CONS_POTION)],
      [4, 1, makeConsumableCell(CONS_BOTH)],
      [4, 2, makeConsumableCell(CONS_BOMB)],
      [5, 2, makeManaCell(MANA_WHITE)],
      [6, 2, mon(KIND_DEMON, 0, 0, 1)],
    ] as const,
    expected: [
      move([5, 1], [4, 0], 0),
      move([5, 1], [4, 1], 1 << 14),
      move([5, 1], [5, 0], 0),
      move([5, 1], [6, 0], 0),
      move([5, 1], [6, 1], 0),
    ],
  },
] as const;

describe("specialized packed move generation", () => {
  it.each(GOLDEN_CASES)("$name", ({ placements, expected }) => {
    const moves = new Int32Array(MAX_MOVES);
    const keys = new Int32Array(MAX_MOVES);
    const count = generateMoves(packed(placements), moves, keys);

    expect(
      Array.from({ length: count }, (_, slot) => [moves[slot], keys[slot]]),
    ).toEqual(expected);
  });

  it("suppresses mana starts when a carrier has only raw fixed-consumable starts", () => {
    const fixedConsumables = [
      [4, 4],
      [4, 5],
      [4, 6],
      [5, 4],
      [5, 6],
      [6, 4],
      [6, 5],
      [6, 6],
    ] as const;
    const position = packed([
      [5, 5, mon(KIND_ANGEL, MANA_WHITE)],
      [1, 1, makeManaCell(MANA_WHITE)],
      ...fixedConsumables.map(
        ([row, column]) => [row, column, makeConsumableCell(CONS_BOMB)] as const,
      ),
    ]);
    const moves = new Int32Array(MAX_MOVES);
    const keys = new Int32Array(MAX_MOVES);

    expect(position.freeMana[0]).toBe(1);
    expect(generateMoves(position, moves, keys)).toBe(0);
    expect([...moves.subarray(0, 1)]).toEqual([0]);
    expect([...keys.subarray(0, 1)]).toEqual([0]);
  });
});
