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

export function colorFen(color: Color): string {
  return color === Color.White ? "w" : "b";
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
  return fen.length === 2 ? parseMonFenAt(fen, 0) : undefined;
}

function parseMonFenAt(fen: string, index: number): Mon | undefined {
  let color: Color;
  let kind: MonKind;
  switch (fen.charCodeAt(index)) {
    case 69:
      color = Color.White;
      kind = MonKind.Demon;
      break;
    case 101:
      color = Color.Black;
      kind = MonKind.Demon;
      break;
    case 68:
      color = Color.White;
      kind = MonKind.Drainer;
      break;
    case 100:
      color = Color.Black;
      kind = MonKind.Drainer;
      break;
    case 65:
      color = Color.White;
      kind = MonKind.Angel;
      break;
    case 97:
      color = Color.Black;
      kind = MonKind.Angel;
      break;
    case 83:
      color = Color.White;
      kind = MonKind.Spirit;
      break;
    case 115:
      color = Color.Black;
      kind = MonKind.Spirit;
      break;
    case 89:
      color = Color.White;
      kind = MonKind.Mystic;
      break;
    case 121:
      color = Color.Black;
      kind = MonKind.Mystic;
      break;
    default:
      return undefined;
  }
  const cooldownCode = fen.charCodeAt(index + 1);
  return cooldownCode < 48 || cooldownCode > 50
    ? undefined
    : createMon(kind, color, cooldownCode - 48);
}

export function manaFen(mana: Mana): string {
  if (mana.kind === "supermana") {
    return "U";
  }
  return mana.color === Color.White ? "M" : "m";
}

function parseManaCode(code: number): Mana | undefined {
  switch (code) {
    case 77:
      return regularMana(Color.White);
    case 109:
      return regularMana(Color.Black);
    case 85:
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

function parseConsumableCode(code: number): Consumable | undefined {
  switch (code) {
    case 80:
      return Consumable.Potion;
    case 66:
      return Consumable.Bomb;
    case 81:
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
  return fen.length === 3 ? parseItemFenAt(fen, 0) : undefined;
}

export function parseItemFenAt(fen: string, index: number): Item | undefined {
  const first = fen.charCodeAt(index);
  const content = fen.charCodeAt(index + 2);
  if (first === 120) {
    if (fen.charCodeAt(index + 1) !== 120) return undefined;
    const mana = parseManaCode(content);
    if (mana !== undefined) return manaItem(mana);
    const consumable = parseConsumableCode(content);
    return consumable === undefined ? undefined : consumableItem(consumable);
  }

  const mon = parseMonFenAt(fen, index);
  if (mon === undefined) return undefined;
  const mana = parseManaCode(content);
  if (mana !== undefined) {
    return monWithManaItem(mon, mana);
  }
  const consumable = parseConsumableCode(content);
  if (consumable !== undefined) {
    return monWithConsumableItem(mon, consumable);
  }
  return content === 120 ? monItem(mon) : undefined;
}
