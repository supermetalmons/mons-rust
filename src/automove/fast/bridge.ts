import {
  Consumable,
  MON_KIND_IDS,
  Modifier,
  colorId,
  itemMana,
  itemMon,
  type Input,
  type Mana,
} from "../../engine/domain.js";
import type { MonsGame } from "../../engine/game.js";
import { BOARD_CELLS, fromLocationIndex } from "../../engine/geometry.js";
import {
  isFastWorkspaceAllocationFailure,
  rethrowFastWorkspaceAllocation,
} from "./allocation.js";
import {
  CONS_BOMB,
  CONS_BOTH,
  CONS_POTION,
  MANA_BLACK,
  MANA_SUPER,
  MANA_WHITE,
  fastMonCellFromMon,
  fastSquaresForVariant,
  makeConsumableCell,
  makeManaCell,
  monId,
} from "./board.js";
import {
  AUX_NONE,
  MOD_BOMB,
  MOD_POTION,
  MOVE_DEMON,
  MOVE_MON,
  MOVE_SPIRIT,
  moveAux,
  moveFrom,
  moveModifier,
  moveTo,
  moveType,
  type FastPosition,
} from "./position.js";

function consumableCode(consumable: Consumable): number {
  switch (consumable) {
    case Consumable.Bomb:
      return CONS_BOMB;
    case Consumable.Potion:
      return CONS_POTION;
    case Consumable.BombOrPotion:
      return CONS_BOTH;
  }
}

function manaCode(mana: Mana): number {
  if (mana.kind === "supermana") return MANA_SUPER;
  return colorId(mana.color) === 0 ? MANA_WHITE : MANA_BLACK;
}

const INT32_MAX = 0x7fff_ffff;

/** Returns false for unsupported states without modifying the destination. */
export function tryLoadPosition(
  position: FastPosition,
  game: MonsGame,
  maxDepth: number,
): boolean {
  if (!Number.isSafeInteger(maxDepth) || maxDepth < 0) return false;
  if (
    game.whitePotionsCount > INT32_MAX - maxDepth ||
    game.blackPotionsCount > INT32_MAX - maxDepth ||
    game.turnNumber > Number.MAX_SAFE_INTEGER - maxDepth
  ) {
    return false;
  }

  let seenMons: Uint8Array;
  try {
    seenMons = new Uint8Array(position.monLocations.length);
  } catch (error) {
    if (error instanceof RangeError) return false;
    throw error;
  }
  let manaCount = 0;
  for (const item of game.board.items) {
    if (item === undefined) continue;
    if (itemMana(item) !== undefined) {
      manaCount += 1;
      if (manaCount > position.manaIndices.length) return false;
    }
    const mon = itemMon(item);
    if (mon === undefined) continue;
    const id = monId(MON_KIND_IDS[mon.kind], colorId(mon.color));
    if (seenMons[id] !== 0) return false;
    seenMons[id] = 1;
  }

  try {
    loadPosition(position, game);
  } catch (error) {
    if (isFastWorkspaceAllocationFailure(error)) return false;
    throw error;
  }
  return true;
}

function loadPosition(position: FastPosition, game: MonsGame): void {
  let cells: Uint16Array;
  try {
    cells = new Uint16Array(BOARD_CELLS);
  } catch (error) {
    rethrowFastWorkspaceAllocation(error);
  }
  const items = game.board.items;
  for (let index = 0; index < BOARD_CELLS; index += 1) {
    const item = items[index];
    if (item === undefined) continue;
    switch (item.kind) {
      case "mon":
        cells[index] = fastMonCellFromMon(item.mon, 0, 0);
        break;
      case "mana":
        cells[index] = makeManaCell(manaCode(item.mana));
        break;
      case "mon-with-mana":
        cells[index] = fastMonCellFromMon(item.mon, manaCode(item.mana), 0);
        break;
      case "mon-with-consumable":
        cells[index] = fastMonCellFromMon(
          item.mon,
          0,
          consumableCode(item.consumable),
        );
        break;
      case "consumable":
        cells[index] = makeConsumableCell(consumableCode(item.consumable));
        break;
    }
  }
  position.reset({
    cells,
    squares: fastSquaresForVariant(game.variant()),
    whiteScore: game.whiteScore,
    blackScore: game.blackScore,
    active: colorId(game.activeColor),
    monsMoves: game.monsMovesCount,
    manaMoves: game.manaMovesCount,
    actionsUsed: game.actionsUsedCount,
    potions: [game.whitePotionsCount, game.blackPotionsCount],
    firstTurn: game.isFirstTurn(),
  });
}

function locationInput(index: number): Input {
  return {
    kind: "location",
    location: fromLocationIndex(index),
  };
}

function modifierInput(modifier: number): Input {
  return {
    kind: "modifier",
    modifier:
      modifier === MOD_BOMB ? Modifier.SelectBomb : Modifier.SelectPotion,
  };
}

export function moveToInputs(move: number): Input[] {
  const type = moveType(move);
  const inputs: Input[] = [
    locationInput(moveFrom(move)),
    locationInput(moveTo(move)),
  ];
  if (type === MOVE_SPIRIT) {
    inputs.push(locationInput(moveAux(move)));
  } else if (type === MOVE_DEMON && moveAux(move) !== AUX_NONE) {
    inputs.push(locationInput(moveAux(move)));
  }
  const modifier = moveModifier(move);
  if (modifier === MOD_BOMB || modifier === MOD_POTION) {
    if (type === MOVE_MON || type === MOVE_DEMON || type === MOVE_SPIRIT) {
      inputs.push(modifierInput(modifier));
    }
  }
  return inputs;
}
