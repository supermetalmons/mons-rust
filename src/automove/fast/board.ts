import {
  COLOR_IDS,
  MON_KIND_IDS,
  MonKind,
  colorId,
  manaScoreForOwnership,
  type Mon,
} from "../../engine/domain.js";
import {
  ALL_GAME_VARIANTS,
  MON_BASE_LOCATIONS,
  SUPERMANA_BASE,
  squaresForVariant,
} from "../../engine/config.js";
import type { GameVariant } from "../../api/types.js";
import { rethrowFastWorkspaceAllocation } from "./allocation.js";
import {
  bombReachableLocations,
  BOARD_CELLS,
  BOARD_CENTER_INDEX,
  BOARD_SIZE,
  demonReachableLocations,
  fromLocationIndex,
  locationIndex,
  mysticReachableLocations,
  nearbyLocations,
  spiritReachableLocations,
  type Location,
} from "../../engine/geometry.js";

export const COLOR_COUNT = Object.keys(COLOR_IDS).length;
export const MON_KIND_COUNT = Object.keys(MON_KIND_IDS).length;
export const MON_ID_COUNT = COLOR_COUNT * MON_KIND_COUNT;

export const OCC_MANA = 1;
export const OCC_CONS = 2;
export const OCC_MON = 3;

export const MANA_WHITE = 1;
export const MANA_BLACK = 2;
export const MANA_SUPER = 3;

export const CONS_BOMB = 1;
export const CONS_POTION = 2;
export const CONS_BOTH = 3;

export const KIND_DEMON = MON_KIND_IDS[MonKind.Demon];
export const KIND_DRAINER = MON_KIND_IDS[MonKind.Drainer];
export const KIND_ANGEL = MON_KIND_IDS[MonKind.Angel];
export const KIND_SPIRIT = MON_KIND_IDS[MonKind.Spirit];
export const KIND_MYSTIC = MON_KIND_IDS[MonKind.Mystic];

const SQ_REGULAR = 0;
const SQ_CONSUMABLE_BASE = 1;
export const SQ_SUPERMANA_BASE = 2;
const SQ_MANA_BASE = 3;
export const SQ_POOL = 5;
export const SQ_MON_BASE = 7;

export const SUPERMANA_BASE_INDEX = locationIndex(SUPERMANA_BASE);

export function cellOccupancy(cell: number): number {
  return cell & 3;
}

export function cellMonKind(cell: number): number {
  return (cell >> 2) & 7;
}

export function cellMonColor(cell: number): number {
  return (cell >> 5) & 1;
}

export function cellCooldown(cell: number): number {
  return (cell >> 6) & 3;
}

export function cellMana(cell: number): number {
  return (cell >> 8) & 3;
}

export function cellConsumable(cell: number): number {
  return (cell >> 10) & 3;
}

export function damagingAttackCanComplete(cell: number): boolean {
  const consumable = cellConsumable(cell);
  return consumable === 0 || consumable === CONS_BOMB;
}

export function makeMonCell(
  kind: number,
  color: number,
  cooldown: number,
  mana: number,
  consumable: number,
): number {
  return (
    OCC_MON |
    (kind << 2) |
    (color << 5) |
    (cooldown << 6) |
    (mana << 8) |
    (consumable << 10)
  );
}

export function makeManaCell(mana: number): number {
  return OCC_MANA | (mana << 8);
}

export function makeConsumableCell(consumable: number): number {
  return OCC_CONS | (consumable << 10);
}

export function manaScoreValue(mana: number, color: number): number {
  return manaScoreForOwnership(mana === MANA_SUPER, mana - 1 === color);
}

export function monId(kind: number, color: number): number {
  return kind * COLOR_COUNT + color;
}

export function squareIsRegularForMovement(square: number): boolean {
  return square !== SQ_SUPERMANA_BASE && square < SQ_MON_BASE;
}

export function squareIsPool(square: number): boolean {
  return square >= SQ_POOL && square < SQ_MON_BASE;
}

export function squareMonBaseKind(square: number): number {
  return Math.trunc((square - SQ_MON_BASE) / COLOR_COUNT);
}

export function squareMonBaseColor(square: number): number {
  return (square - SQ_MON_BASE) % COLOR_COUNT;
}

export function u8(array: ArrayLike<number>, index: number): number {
  return array[index] ?? 0;
}

export function u16(array: ArrayLike<number>, index: number): number {
  return array[index] ?? 0;
}

export function i32(array: ArrayLike<number>, index: number): number {
  return array[index] ?? 0;
}

export function at<Value>(items: readonly Value[], index: number): Value {
  const value = items[index];
  if (value === undefined) {
    throw new RangeError("automove search slot index out of range");
  }
  return value;
}

export function fastMonCellFromMon(
  mon: Mon,
  mana: number,
  consumable: number,
): number {
  return makeMonCell(
    MON_KIND_IDS[mon.kind],
    colorId(mon.color),
    mon.cooldown,
    mana,
    consumable,
  );
}

function buildSquares(variant: GameVariant): Uint8Array {
  const squares = new Uint8Array(BOARD_CELLS);
  const source = squaresForVariant(variant);
  for (let index = 0; index < BOARD_CELLS; index += 1) {
    const square = source[index];
    if (square === undefined) continue;
    switch (square.kind) {
      case "regular":
        squares[index] = SQ_REGULAR;
        break;
      case "consumable-base":
        squares[index] = SQ_CONSUMABLE_BASE;
        break;
      case "supermana-base":
        squares[index] = SQ_SUPERMANA_BASE;
        break;
      case "mana-base":
        squares[index] = SQ_MANA_BASE + colorId(square.color);
        break;
      case "mana-pool":
        squares[index] = SQ_POOL + colorId(square.color);
        break;
      case "mon-base":
        squares[index] =
          SQ_MON_BASE +
          monId(MON_KIND_IDS[square.monKind], colorId(square.color));
        break;
    }
  }
  return squares;
}

const SQUARES_BY_VARIANT = new Map<GameVariant, Uint8Array>(
  ALL_GAME_VARIANTS.map((variant) => [variant, buildSquares(variant)]),
);

export function fastSquaresForVariant(variant: GameVariant): Uint8Array {
  const squares = SQUARES_BY_VARIANT.get(variant);
  if (squares === undefined) {
    throw new TypeError(`unsupported game variant: ${variant}`);
  }
  try {
    return squares.slice();
  } catch (error) {
    rethrowFastWorkspaceAllocation(error);
  }
}

export const MON_BASE_INDEX = (() => {
  const table = new Int32Array(MON_ID_COUNT).fill(-1);
  const squares = fastSquaresForVariant(at(ALL_GAME_VARIANTS, 0));
  for (const location of MON_BASE_LOCATIONS) {
    const index = locationIndex(location);
    const square = u8(squares, index);
    if (square >= SQ_MON_BASE) {
      table[square - SQ_MON_BASE] = index;
    }
  }
  return table;
})();

type OffsetTable = {
  readonly list: Int32Array;
  readonly starts: Int32Array;
  readonly counts: Int32Array;
};

function buildOffsetTable(
  reachableLocations: (origin: Location) => readonly Location[],
): OffsetTable {
  const offsets: number[] = [];
  const counts = new Int32Array(BOARD_CELLS);
  const starts = new Int32Array(BOARD_CELLS);
  for (let index = 0; index < BOARD_CELLS; index += 1) {
    starts[index] = offsets.length;
    const targets = reachableLocations(fromLocationIndex(index))
      .map(locationIndex)
      .sort((left, right) => left - right);
    offsets.push(...targets);
    counts[index] = targets.length;
  }
  return {
    list: new Int32Array(offsets),
    starts,
    counts,
  };
}

export const NEIGHBORS = buildOffsetTable(nearbyLocations);
export const MYSTIC_TARGETS = buildOffsetTable(mysticReachableLocations);
export const DEMON_TARGETS = buildOffsetTable(demonReachableLocations);
export const SPIRIT_TARGETS = buildOffsetTable(spiritReachableLocations);
export const BOMB_TARGETS = buildOffsetTable(bombReachableLocations);

const CHEBYSHEV = (() => {
  const table = new Uint8Array(BOARD_CELLS * BOARD_CELLS);
  for (let a = 0; a < BOARD_CELLS; a += 1) {
    const rowA = Math.trunc(a / BOARD_SIZE);
    const columnA = a % BOARD_SIZE;
    for (let b = 0; b < BOARD_CELLS; b += 1) {
      const rowB = Math.trunc(b / BOARD_SIZE);
      const columnB = b % BOARD_SIZE;
      table[a * BOARD_CELLS + b] = Math.max(
        Math.abs(rowA - rowB),
        Math.abs(columnA - columnB),
      );
    }
  }
  return table;
})();

export function chebyshev(from: number, to: number): number {
  return u8(CHEBYSHEV, from * BOARD_CELLS + to);
}

export function midpoint(from: number, to: number): number {
  const rowFrom = Math.trunc(from / BOARD_SIZE);
  const columnFrom = from % BOARD_SIZE;
  const rowTo = Math.trunc(to / BOARD_SIZE);
  const columnTo = to % BOARD_SIZE;
  return (
    Math.trunc((rowFrom + rowTo) / 2) * BOARD_SIZE +
    Math.trunc((columnFrom + columnTo) / 2)
  );
}

const POOL_INDICES = (() => {
  const list: number[] = [];
  const squares = fastSquaresForVariant(at(ALL_GAME_VARIANTS, 0));
  for (let index = 0; index < BOARD_CELLS; index += 1) {
    if (squareIsPool(u8(squares, index))) list.push(index);
  }
  return new Int32Array(list);
})();

export const OWN_POOL_DISTANCE = (() => {
  const table = new Uint8Array(BOARD_CELLS * COLOR_COUNT);
  for (let index = 0; index < BOARD_CELLS; index += 1) {
    const row = Math.trunc(index / BOARD_SIZE);
    const column = index % BOARD_SIZE;
    const columnDistance = Math.min(column, BOARD_SIZE - 1 - column);
    for (let color = 0; color < COLOR_COUNT; color += 1) {
      const poolRow = color === 0 ? BOARD_SIZE - 1 : 0;
      const rowDistance = Math.abs(poolRow - row);
      table[color * BOARD_CELLS + index] =
        rowDistance > columnDistance ? rowDistance : columnDistance;
    }
  }
  return table;
})();

export const CENTER_ROW_DISTANCE = (() => {
  const table = new Uint8Array(BOARD_CELLS);
  for (let index = 0; index < BOARD_CELLS; index += 1) {
    table[index] = Math.abs(
      BOARD_CENTER_INDEX - Math.trunc(index / BOARD_SIZE),
    );
  }
  return table;
})();

export const POOL_DISTANCE = (() => {
  const table = new Uint8Array(BOARD_CELLS);
  for (let index = 0; index < BOARD_CELLS; index += 1) {
    let best = 99;
    for (let poolIndex = 0; poolIndex < POOL_INDICES.length; poolIndex += 1) {
      const distance = chebyshev(index, i32(POOL_INDICES, poolIndex));
      if (distance < best) best = distance;
    }
    table[index] = best;
  }
  return table;
})();

function xorshift(seed: number): () => number {
  let state = seed >>> 0;
  return () => {
    state ^= state << 13;
    state >>>= 0;
    state ^= state >>> 17;
    state ^= state << 5;
    state >>>= 0;
    return state | 0;
  };
}

function randomTable(next: () => number, size: number): Int32Array {
  const table = new Int32Array(size);
  for (let index = 0; index < size; index += 1) table[index] = next();
  return table;
}

const RANDOM = xorshift(0x2f6e2b1);

const MON_COOLDOWN_STATES = 3;
const MON_SLOTS = 1 + MON_ID_COUNT * MON_COOLDOWN_STATES;
const Z_MON_LO = randomTable(RANDOM, BOARD_CELLS * MON_SLOTS);
const Z_MON_HI = randomTable(RANDOM, BOARD_CELLS * MON_SLOTS);
const Z_MANA_LO = randomTable(RANDOM, BOARD_CELLS * 4);
const Z_MANA_HI = randomTable(RANDOM, BOARD_CELLS * 4);
const Z_CONS_LO = randomTable(RANDOM, BOARD_CELLS * 4);
const Z_CONS_HI = randomTable(RANDOM, BOARD_CELLS * 4);
const Z_SCALAR_SIZE = 2048;
export const Z_SCALAR_LO = randomTable(RANDOM, Z_SCALAR_SIZE);
export const Z_SCALAR_HI = randomTable(RANDOM, Z_SCALAR_SIZE);

function monSlot(cell: number): number {
  if (cellOccupancy(cell) !== OCC_MON) return 0;
  return (
    1 +
    monId(cellMonKind(cell), cellMonColor(cell)) * MON_COOLDOWN_STATES +
    cellCooldown(cell)
  );
}

export function cellHashLo(index: number, cell: number): number {
  if (cell === 0) return 0;
  return (
    i32(Z_MON_LO, index * MON_SLOTS + monSlot(cell)) ^
    i32(Z_MANA_LO, index * 4 + cellMana(cell)) ^
    i32(Z_CONS_LO, index * 4 + cellConsumable(cell))
  );
}

export function cellHashHi(index: number, cell: number): number {
  if (cell === 0) return 0;
  return (
    i32(Z_MON_HI, index * MON_SLOTS + monSlot(cell)) ^
    i32(Z_MANA_HI, index * 4 + cellMana(cell)) ^
    i32(Z_CONS_HI, index * 4 + cellConsumable(cell))
  );
}
