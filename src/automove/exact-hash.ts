import { Board } from "../engine/board.js";
import { type GameVariant, gameVariantId } from "../engine/config.js";
import {
  Color,
  Consumable,
  type Item,
  type Mana,
  type Mon,
  MonKind,
} from "../engine/domain.js";
import { MonsGame } from "../engine/game.js";
import { locationIndex } from "../engine/geometry.js";
import {
  hash64,
  type Hash64,
  hash64Add,
  hash64FromNonnegativeInteger,
  hash64FromLowWord,
  hash64Mul,
  hash64RotateLeft,
  hash64ShiftRight,
  hash64Xor,
} from "./hash64.js";

const SEARCH_SEED = hash64(0x6a09_e667, 0xf3bc_c909);
const GOLDEN_ODD = hash64(0x9e37_79b1, 0x85eb_ca87);
const MIX_ODD = hash64(0x94d0_49bb, 0x1331_11eb);
const SPLITMIX_INCREMENT = hash64(0x9e37_79b9, 0x7f4a_7c15);
const SPLITMIX_FIRST = hash64(0xbf58_476d, 0x1ce4_e5b9);
const SPLITMIX_SECOND = hash64(0x94d0_49bb, 0x1331_11eb);

function exactHashColorBits(color: Color): number {
  return color === Color.White ? 1 : 2;
}

function exactHashMonKindBits(kind: MonKind): number {
  switch (kind) {
    case MonKind.Demon:
      return 1;
    case MonKind.Drainer:
      return 2;
    case MonKind.Angel:
      return 3;
    case MonKind.Spirit:
      return 4;
    case MonKind.Mystic:
      return 5;
  }
}

export function exactHashManaBits(mana: Mana): number {
  return mana.kind === "supermana"
    ? 2
    : 1 | (exactHashColorBits(mana.color) << 4);
}

function exactSearchHashManaBits(mana: Mana): number {
  return mana.kind === "supermana"
    ? 0x20
    : 0x10 | exactHashColorBits(mana.color);
}

function exactHashConsumableBits(consumable: Consumable): number {
  switch (consumable) {
    case Consumable.Bomb:
      return 1;
    case Consumable.Potion:
      return 2;
    case Consumable.BombOrPotion:
      return 3;
  }
}

function exactSearchHashConsumableBits(consumable: Consumable): number {
  switch (consumable) {
    case Consumable.Potion:
      return 1;
    case Consumable.Bomb:
      return 2;
    case Consumable.BombOrPotion:
      return 3;
  }
}

function exactHashMonBits(mon: Mon): number {
  return (
    (exactHashMonKindBits(mon.kind) |
      (exactHashColorBits(mon.color) << 4) |
      ((mon.cooldown & 0xff) << 8)) >>>
    0
  );
}

function exactHashItem(item: Item): Hash64 {
  let bits: number;
  switch (item.kind) {
    case "mon":
      bits = 0x100 | exactHashMonBits(item.mon);
      break;
    case "mana":
      bits = 0x200 | exactHashManaBits(item.mana);
      break;
    case "mon-with-mana":
      bits =
        0x300 |
        exactHashMonBits(item.mon) |
        (exactHashManaBits(item.mana) << 16);
      break;
    case "mon-with-consumable":
      bits =
        0x400 |
        exactHashMonBits(item.mon) |
        (exactHashConsumableBits(item.consumable) << 16);
      break;
    case "consumable":
      bits = 0x500 | exactHashConsumableBits(item.consumable);
      break;
  }
  return hash64FromLowWord(bits >>> 0);
}

function exactSearchHashItem(item: Item): Hash64 {
  let bits: number;
  switch (item.kind) {
    case "mon":
      bits = 0x100 | exactHashMonBits(item.mon);
      break;
    case "mana":
      bits = 0x200 | exactSearchHashManaBits(item.mana);
      break;
    case "mon-with-mana":
      bits =
        0x300 |
        exactHashMonBits(item.mon) |
        (exactSearchHashManaBits(item.mana) << 16);
      break;
    case "mon-with-consumable":
      bits =
        0x400 |
        exactHashMonBits(item.mon) |
        (exactSearchHashConsumableBits(item.consumable) << 16);
      break;
    case "consumable":
      bits = 0x500 | exactSearchHashConsumableBits(item.consumable);
      break;
  }
  return hash64FromLowWord(bits >>> 0);
}

function mixBoardHash(value: Hash64): Hash64 {
  let mixed = value;
  mixed = hash64Xor(mixed, hash64ShiftRight(mixed, 30));
  mixed = hash64Mul(mixed, SPLITMIX_FIRST);
  mixed = hash64Xor(mixed, hash64ShiftRight(mixed, 27));
  mixed = hash64Mul(mixed, SPLITMIX_SECOND);
  return hash64Xor(mixed, hash64ShiftRight(mixed, 31));
}

function mixSearchHash(value: Hash64): Hash64 {
  let mixed = hash64Add(value, SPLITMIX_INCREMENT);
  mixed = hash64Mul(
    hash64Xor(mixed, hash64ShiftRight(mixed, 30)),
    SPLITMIX_FIRST,
  );
  mixed = hash64Mul(
    hash64Xor(mixed, hash64ShiftRight(mixed, 27)),
    SPLITMIX_SECOND,
  );
  return hash64Xor(mixed, hash64ShiftRight(mixed, 31));
}

function exactBoardEntryHash(index: number, item: Item): Hash64 {
  return mixBoardHash(
    hash64Xor(
      hash64Mul(hash64FromLowWord(index + 1), GOLDEN_ODD),
      hash64Mul(exactHashItem(item), MIX_ODD),
    ),
  );
}

function exactBoardVariantHash(variant: GameVariant): Hash64 {
  return mixBoardHash(
    hash64Add(
      hash64FromNonnegativeInteger(gameVariantId(variant)),
      hash64(0x243f_6a88, 0x85a3_08d3),
    ),
  );
}

export function exactBoardHash(board: Board): Hash64 {
  let state = hash64Xor(SEARCH_SEED, exactBoardVariantHash(board.variant));
  for (const [location, item] of board.entries()) {
    state = hash64Xor(
      state,
      exactBoardEntryHash(locationIndex(location), item),
    );
  }
  return state;
}

export function exactSearchStateHash(game: MonsGame): Hash64 {
  let state = SEARCH_SEED;
  for (const [location, item] of game.board.entries()) {
    const entry = hash64Xor(
      hash64Mul(hash64FromLowWord(locationIndex(location) + 1), GOLDEN_ODD),
      exactSearchHashItem(item),
    );
    state = hash64Xor(state, mixSearchHash(entry));
    state = hash64Mul(hash64RotateLeft(state, 17), MIX_ODD);
  }

  const fields: readonly (readonly [Hash64, number])[] = [
    [hash64FromNonnegativeInteger(game.whiteScore), 0x11],
    [hash64FromNonnegativeInteger(game.blackScore), 0x23],
    [hash64FromLowWord(exactHashColorBits(game.activeColor)), 0x35],
    [hash64FromNonnegativeInteger(game.actionsUsedCount), 0x47],
    [hash64FromNonnegativeInteger(game.manaMovesCount), 0x59],
    [hash64FromNonnegativeInteger(game.monsMovesCount), 0x6b],
    [hash64FromNonnegativeInteger(game.whitePotionsCount), 0x7d],
    [hash64FromNonnegativeInteger(game.blackPotionsCount), 0x8f],
    [hash64FromNonnegativeInteger(game.turnNumber), 0xa1],
    [hash64FromNonnegativeInteger(gameVariantId(game.variant())), 0xb3],
  ];
  for (const [value, salt] of fields) {
    state = hash64Xor(
      state,
      mixSearchHash(hash64Xor(value, hash64FromLowWord(salt))),
    );
  }
  return mixSearchHash(state);
}
