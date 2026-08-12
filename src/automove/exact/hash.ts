import { Board } from "../../engine/board/storage.js";
import type { GameVariant } from "../../api/types.js";
import { gameVariantId } from "../../engine/board/config.js";
import { Color, Consumable, type Mana, type Mon, MonKind } from "../../api/types.js";
import type { Item } from "../../engine/model/domain.js";
import { MonsGame } from "../../engine/game/mons-game.js";
import { locationIndex } from "../../engine/board/geometry.js";
import type { Hash64 } from "../core/hash64.js";

const SEARCH_SEED_HI = 0x6a09_e667;
const SEARCH_SEED_LO = 0xf3bc_c909;
const GOLDEN_ODD_HI = 0x9e37_79b1;
const GOLDEN_ODD_LO = 0x85eb_ca87;
const MIX_ODD_HI = 0x94d0_49bb;
const MIX_ODD_LO = 0x1331_11eb;
const SPLITMIX_INCREMENT_HI = 0x9e37_79b9;
const SPLITMIX_INCREMENT_LO = 0x7f4a_7c15;
const SPLITMIX_FIRST_HI = 0xbf58_476d;
const SPLITMIX_FIRST_LO = 0x1ce4_e5b9;
const SPLITMIX_SECOND_HI = 0x94d0_49bb;
const SPLITMIX_SECOND_LO = 0x1331_11eb;
const MON_COOLDOWN_SHIFT = 11;

type MutableHash64 = {
  hi: number;
  lo: number;
};

function multiplyInto(
  result: MutableHash64,
  leftHi: number,
  leftLo: number,
  rightHi: number,
  rightLo: number,
): void {
  const leftLoLow = leftLo & 0xffff;
  const leftLoHigh = leftLo >>> 16;
  const rightLoLow = rightLo & 0xffff;
  const rightLoHigh = rightLo >>> 16;
  const lowProduct = leftLoLow * rightLoLow;
  const middle =
    (lowProduct >>> 16) + leftLoHigh * rightLoLow + leftLoLow * rightLoHigh;
  result.lo = ((lowProduct & 0xffff) | ((middle & 0xffff) << 16)) >>> 0;
  result.hi =
    (leftLoHigh * rightLoHigh +
      Math.floor(middle / 0x1_0000) +
      (Math.imul(leftHi, rightLo) >>> 0) +
      (Math.imul(leftLo, rightHi) >>> 0)) >>>
    0;
}

function xorShiftRightInto(
  result: MutableHash64,
  hi: number,
  lo: number,
  shift: number,
): void {
  result.hi = (hi ^ (hi >>> shift)) >>> 0;
  result.lo = (lo ^ ((lo >>> shift) | (hi << (32 - shift)))) >>> 0;
}

function mixBoardHashInto(
  result: MutableHash64,
  valueHi: number,
  valueLo: number,
): void {
  xorShiftRightInto(result, valueHi, valueLo, 30);
  multiplyInto(result, result.hi, result.lo, SPLITMIX_FIRST_HI, SPLITMIX_FIRST_LO);
  xorShiftRightInto(result, result.hi, result.lo, 27);
  multiplyInto(result, result.hi, result.lo, SPLITMIX_SECOND_HI, SPLITMIX_SECOND_LO);
  xorShiftRightInto(result, result.hi, result.lo, 31);
}

function mixSearchHashInto(
  result: MutableHash64,
  valueHi: number,
  valueLo: number,
): void {
  const addedLo = (valueLo + SPLITMIX_INCREMENT_LO) >>> 0;
  const addedHi =
    (valueHi + SPLITMIX_INCREMENT_HI + (addedLo < valueLo >>> 0 ? 1 : 0)) >>> 0;
  xorShiftRightInto(result, addedHi, addedLo, 30);
  multiplyInto(result, result.hi, result.lo, SPLITMIX_FIRST_HI, SPLITMIX_FIRST_LO);
  xorShiftRightInto(result, result.hi, result.lo, 27);
  multiplyInto(result, result.hi, result.lo, SPLITMIX_SECOND_HI, SPLITMIX_SECOND_LO);
  xorShiftRightInto(result, result.hi, result.lo, 31);
}

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
  return mana.kind === "supermana" ? 2 : 1 | (exactHashColorBits(mana.color) << 4);
}

function exactSearchHashManaBits(mana: Mana): number {
  return mana.kind === "supermana" ? 0x20 : 0x10 | exactHashColorBits(mana.color);
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
      ((mon.cooldown & 0xff) << MON_COOLDOWN_SHIFT)) >>>
    0
  );
}

function exactHashItemBits(item: Item): number {
  let bits: number;
  switch (item.kind) {
    case "mon":
      bits = 0x100 | exactHashMonBits(item.mon);
      break;
    case "mana":
      bits = 0x200 | exactHashManaBits(item.mana);
      break;
    case "mon-with-mana":
      bits = 0x300 | exactHashMonBits(item.mon) | (exactHashManaBits(item.mana) << 16);
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
  return bits >>> 0;
}

function exactSearchHashItemBits(item: Item): number {
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
        0x300 | exactHashMonBits(item.mon) | (exactSearchHashManaBits(item.mana) << 16);
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
  return bits >>> 0;
}

function mixSearchFieldInto(result: MutableHash64, value: number, salt: number): void {
  mixSearchHashInto(result, Math.floor(value / 0x1_0000_0000), (value ^ salt) >>> 0);
}

function exactBoardVariantHashInto(result: MutableHash64, variant: GameVariant): void {
  const value = gameVariantId(variant);
  const valueLo = value >>> 0;
  const lo = (valueLo + 0x85a3_08d3) >>> 0;
  const hi =
    (Math.floor(value / 0x1_0000_0000) + 0x243f_6a88 + (lo < valueLo ? 1 : 0)) >>> 0;
  mixBoardHashInto(result, hi, lo);
}

export function exactBoardHash(board: Board): Hash64 {
  const result: MutableHash64 = { hi: 0, lo: 0 };
  exactBoardVariantHashInto(result, board.variant);
  let stateHi = (SEARCH_SEED_HI ^ result.hi) >>> 0;
  let stateLo = (SEARCH_SEED_LO ^ result.lo) >>> 0;
  for (const [location, item] of board.entries()) {
    multiplyInto(result, 0, locationIndex(location) + 1, GOLDEN_ODD_HI, GOLDEN_ODD_LO);
    const indexHi = result.hi;
    const indexLo = result.lo;
    multiplyInto(result, 0, exactHashItemBits(item), MIX_ODD_HI, MIX_ODD_LO);
    mixBoardHashInto(result, indexHi ^ result.hi, indexLo ^ result.lo);
    stateHi = (stateHi ^ result.hi) >>> 0;
    stateLo = (stateLo ^ result.lo) >>> 0;
  }
  result.hi = stateHi;
  result.lo = stateLo;
  return result;
}

export function exactSearchStateHash(game: MonsGame): Hash64 {
  const result: MutableHash64 = { hi: 0, lo: 0 };
  let stateHi = SEARCH_SEED_HI;
  let stateLo = SEARCH_SEED_LO;
  for (const [location, item] of game.board.entries()) {
    multiplyInto(result, 0, locationIndex(location) + 1, GOLDEN_ODD_HI, GOLDEN_ODD_LO);
    mixSearchHashInto(result, result.hi, result.lo ^ exactSearchHashItemBits(item));
    stateHi = (stateHi ^ result.hi) >>> 0;
    stateLo = (stateLo ^ result.lo) >>> 0;
    const rotatedHi = ((stateHi << 17) | (stateLo >>> 15)) >>> 0;
    const rotatedLo = ((stateLo << 17) | (stateHi >>> 15)) >>> 0;
    multiplyInto(result, rotatedHi, rotatedLo, MIX_ODD_HI, MIX_ODD_LO);
    stateHi = result.hi;
    stateLo = result.lo;
  }

  mixSearchFieldInto(result, game.whiteScore, 0x11);
  stateHi = (stateHi ^ result.hi) >>> 0;
  stateLo = (stateLo ^ result.lo) >>> 0;
  mixSearchFieldInto(result, game.blackScore, 0x23);
  stateHi = (stateHi ^ result.hi) >>> 0;
  stateLo = (stateLo ^ result.lo) >>> 0;
  mixSearchFieldInto(result, exactHashColorBits(game.activeColor), 0x35);
  stateHi = (stateHi ^ result.hi) >>> 0;
  stateLo = (stateLo ^ result.lo) >>> 0;
  mixSearchFieldInto(result, game.actionsUsedCount, 0x47);
  stateHi = (stateHi ^ result.hi) >>> 0;
  stateLo = (stateLo ^ result.lo) >>> 0;
  mixSearchFieldInto(result, game.manaMovesCount, 0x59);
  stateHi = (stateHi ^ result.hi) >>> 0;
  stateLo = (stateLo ^ result.lo) >>> 0;
  mixSearchFieldInto(result, game.monsMovesCount, 0x6b);
  stateHi = (stateHi ^ result.hi) >>> 0;
  stateLo = (stateLo ^ result.lo) >>> 0;
  mixSearchFieldInto(result, game.whitePotionsCount, 0x7d);
  stateHi = (stateHi ^ result.hi) >>> 0;
  stateLo = (stateLo ^ result.lo) >>> 0;
  mixSearchFieldInto(result, game.blackPotionsCount, 0x8f);
  stateHi = (stateHi ^ result.hi) >>> 0;
  stateLo = (stateLo ^ result.lo) >>> 0;
  mixSearchFieldInto(result, game.turnNumber, 0xa1);
  stateHi = (stateHi ^ result.hi) >>> 0;
  stateLo = (stateLo ^ result.lo) >>> 0;
  mixSearchFieldInto(result, gameVariantId(game.variant()), 0xb3);
  stateHi = (stateHi ^ result.hi) >>> 0;
  stateLo = (stateLo ^ result.lo) >>> 0;
  mixSearchHashInto(result, stateHi, stateLo);
  return result;
}
