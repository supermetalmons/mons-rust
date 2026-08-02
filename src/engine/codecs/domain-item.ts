import {
  Color,
  Consumable,
  SUPERMANA,
  consumableItem,
  createMon,
  manaItem,
  monItem,
  monWithConsumableItem,
  monWithManaItem,
  regularMana,
  type Item,
  type Mana,
  type Mon,
  MonKind,
} from "../domain.js";
import { isAscii } from "./common.js";

export function colorFen(color: Color): string {
  return color === Color.White ? "w" : "b";
}

export function parseColorFen(fen: string): Color | undefined {
  switch (fen) {
    case "w":
      return Color.White;
    case "b":
      return Color.Black;
    default:
      return undefined;
  }
}

export function monFen(mon: Mon): string {
  if (!Number.isInteger(mon.cooldown) || mon.cooldown < 0 || mon.cooldown > 2) {
    throw new RangeError("mon cooldown must be an integer from 0 through 2");
  }
  const kind = (() => {
    switch (mon.kind) {
      case MonKind.Demon:
        return "e";
      case MonKind.Drainer:
        return "d";
      case MonKind.Angel:
        return "a";
      case MonKind.Spirit:
        return "s";
      case MonKind.Mystic:
        return "y";
    }
  })();
  const colorKind = mon.color === Color.White ? kind.toUpperCase() : kind;
  return `${colorKind}${mon.cooldown}`;
}

export function parseMonFen(fen: string): Mon | undefined {
  if (!/^[A-Za-z][0-2]$/u.test(fen)) return undefined;
  const kindCharacter = fen[0];
  const cooldownCharacter = fen[1];
  if (kindCharacter === undefined || cooldownCharacter === undefined) {
    return undefined;
  }

  const kind = (() => {
    switch (kindCharacter.toLowerCase()) {
      case "e":
        return MonKind.Demon;
      case "d":
        return MonKind.Drainer;
      case "a":
        return MonKind.Angel;
      case "s":
        return MonKind.Spirit;
      case "y":
        return MonKind.Mystic;
      default:
        return undefined;
    }
  })();
  if (kind === undefined) {
    return undefined;
  }

  return createMon(
    kind,
    kindCharacter === kindCharacter.toUpperCase() ? Color.White : Color.Black,
    cooldownCharacter.charCodeAt(0) - 48,
  );
}

export function manaFen(mana: Mana): string {
  if (mana.kind === "supermana") {
    return "U";
  }
  return mana.color === Color.White ? "M" : "m";
}

function parseManaFen(fen: string): Mana | undefined {
  switch (fen) {
    case "M":
      return regularMana(Color.White);
    case "m":
      return regularMana(Color.Black);
    case "U":
      return SUPERMANA;
    default:
      return undefined;
  }
}

function consumableFen(consumable: Consumable): string {
  switch (consumable) {
    case Consumable.Potion:
      return "P";
    case Consumable.Bomb:
      return "B";
    case Consumable.BombOrPotion:
      return "Q";
  }
}

function parseConsumableFen(fen: string): Consumable | undefined {
  switch (fen) {
    case "P":
      return Consumable.Potion;
    case "B":
      return Consumable.Bomb;
    case "Q":
      return Consumable.BombOrPotion;
    default:
      return undefined;
  }
}

export function itemFen(item: Item): string {
  switch (item.kind) {
    case "mon":
      return `${monFen(item.mon)}x`;
    case "mana":
      return `xx${manaFen(item.mana)}`;
    case "mon-with-mana":
      return `${monFen(item.mon)}${manaFen(item.mana)}`;
    case "mon-with-consumable":
      return `${monFen(item.mon)}${consumableFen(item.consumable)}`;
    case "consumable":
      return `xx${consumableFen(item.consumable)}`;
  }
}

export function parseItemFen(fen: string): Item | undefined {
  if (fen.length !== 3 || !isAscii(fen)) return undefined;
  const monCode = fen.slice(0, 2);
  const contentCode = fen.slice(2);
  if (monCode === "xx") {
    const mana = parseManaFen(contentCode);
    if (mana !== undefined) {
      return manaItem(mana);
    }
    const consumable = parseConsumableFen(contentCode);
    return consumable === undefined ? undefined : consumableItem(consumable);
  }

  const mon = parseMonFen(monCode);
  if (mon === undefined) {
    return undefined;
  }
  const mana = parseManaFen(contentCode);
  if (mana !== undefined) {
    return monWithManaItem(mon, mana);
  }
  const consumable = parseConsumableFen(contentCode);
  if (consumable !== undefined) {
    return monWithConsumableItem(mon, consumable);
  }
  return contentCode === "x" ? monItem(mon) : undefined;
}
