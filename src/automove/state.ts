import { BOARD_CELLS } from "../engine/board/geometry.js";
import { ACTIONS_PER_TURN } from "../engine/board/config.js";
import { colorId } from "../engine/model/domain.js";
import { rethrowFastWorkspaceAllocation } from "./allocation.js";
import {
  canMoveManaForCounts,
  canMoveMonForCounts,
  canUseActionForCounts,
  shouldAdvanceTurnForCounts,
  shouldSuggestRegularManaStartsFromScalars,
  winnerForScores,
} from "../engine/rules/legality.js";
import {
  u8,
  COLOR_COUNT,
  CONS_BOMB,
  CONS_BOTH,
  KIND_ANGEL,
  MANA_SUPER,
  MON_BASE_INDEX,
  MON_ID_COUNT,
  MON_KIND_COUNT,
  OCC_CONS,
  OCC_MANA,
  OCC_MON,
  SUPERMANA_BASE_INDEX,
  cellConsumable,
  cellCooldown,
  cellHashHi,
  cellHashLo,
  cellMana,
  cellMonColor,
  cellMonKind,
  cellOccupancy,
  chebyshev,
  i32,
  KIND_DRAINER,
  makeManaCell,
  makeMonCell,
  manaScoreValue,
  monId,
  NEIGHBORS,
  squareIsPool,
  squareIsRegularForMovement,
  u16,
} from "./board.js";

export const MOVE_MON = 0;
export const MOVE_MANA = 1;
export const MOVE_MYSTIC = 2;
export const MOVE_DEMON = 3;
export const MOVE_SPIRIT = 4;
export const MOVE_BOMB = 5;

export const AUX_NONE = 127;
export const MOD_NONE = 0;
export const MOD_BOMB = 1;
export const MOD_POTION = 2;
export const MOD_IGNORED_STEP = 3;
export const FAST_MOVE_UNREPRESENTABLE = -2;

const PACKED_MANA_INDEX_CAPACITY = 16;

export function encodeMove(
  type: number,
  from: number,
  to: number,
  aux: number,
  modifier: number,
): number {
  return type | (from << 3) | (to << 10) | (aux << 17) | (modifier << 24);
}

export function moveType(move: number): number {
  return move & 7;
}

export function moveFrom(move: number): number {
  return (move >> 3) & 127;
}

export function moveTo(move: number): number {
  return (move >> 10) & 127;
}

export function moveAux(move: number): number {
  return (move >> 17) & 127;
}

export function moveModifier(move: number): number {
  return (move >> 24) & 3;
}

type FastArrayView = ArrayLike<number> & Iterable<number>;

export type FastPositionSnapshot = {
  readonly cells: FastArrayView;
  readonly squares: FastArrayView;
  readonly whiteScore: number;
  readonly blackScore: number;
  readonly active: number;
  readonly monsMoves: number;
  readonly manaMoves: number;
  readonly actionsUsed: number;
  readonly potions: FastArrayView;
  readonly firstTurn: boolean;
};

export type FastPosition = {
  readonly cells: FastArrayView;
  readonly squares: FastArrayView;
  readonly monLocations: FastArrayView;
  readonly freeMana: FastArrayView;
  readonly manaIndices: FastArrayView;
  readonly manaCount: number;
  readonly whiteScore: number;
  readonly blackScore: number;
  readonly active: number;
  readonly monsMoves: number;
  readonly manaMoves: number;
  readonly actionsUsed: number;
  readonly potions: FastArrayView;
  readonly firstTurn: boolean;
  readonly hashLo: number;
  readonly hashHi: number;
  copyFrom(other: FastPosition): void;
  reset(snapshot: FastPositionSnapshot): void;
};

function validateArrayLength(
  values: FastArrayView,
  expected: number,
  name: string,
): void {
  if (values.length !== expected) {
    throw new RangeError(`fast position ${name} length must be ${expected}`);
  }
}

class MutableFastPosition implements FastPosition {
  public readonly cells = new Uint16Array(BOARD_CELLS);
  public squares: Uint8Array = new Uint8Array(BOARD_CELLS);
  public readonly monLocations = new Int32Array(MON_ID_COUNT).fill(-1);
  public readonly freeMana = new Int32Array(COLOR_COUNT);
  public readonly manaIndices = new Int32Array(PACKED_MANA_INDEX_CAPACITY);
  public manaCount = 0;
  public whiteScore = 0;
  public blackScore = 0;
  public active = 0;
  public monsMoves = 0;
  public manaMoves = 0;
  public actionsUsed = 0;
  public readonly potions = new Int32Array(COLOR_COUNT);
  public firstTurn = false;
  public hashLo = 0;
  public hashHi = 0;

  public copyFrom(other: FastPosition): void {
    const source = other as MutableFastPosition;
    this.cells.set(source.cells);
    this.squares = source.squares;
    const monLocations = this.monLocations;
    const sourceMonLocations = source.monLocations;
    for (let index = 0; index < MON_ID_COUNT; index += 1) {
      monLocations[index] = i32(sourceMonLocations, index);
    }
    this.freeMana[0] = i32(source.freeMana, 0);
    this.freeMana[1] = i32(source.freeMana, 1);
    const manaIndices = this.manaIndices;
    const sourceManaIndices = source.manaIndices;
    for (let index = 0; index < PACKED_MANA_INDEX_CAPACITY; index += 1) {
      manaIndices[index] = i32(sourceManaIndices, index);
    }
    this.manaCount = source.manaCount;
    this.whiteScore = source.whiteScore;
    this.blackScore = source.blackScore;
    this.active = source.active;
    this.monsMoves = source.monsMoves;
    this.manaMoves = source.manaMoves;
    this.actionsUsed = source.actionsUsed;
    this.potions[0] = i32(source.potions, 0);
    this.potions[1] = i32(source.potions, 1);
    this.firstTurn = source.firstTurn;
    this.hashLo = source.hashLo;
    this.hashHi = source.hashHi;
  }

  public reset(snapshot: FastPositionSnapshot): void {
    validateArrayLength(snapshot.cells, BOARD_CELLS, "cells");
    validateArrayLength(snapshot.squares, BOARD_CELLS, "squares");
    validateArrayLength(snapshot.potions, COLOR_COUNT, "potions");

    let squares: Uint8Array;
    try {
      squares = new Uint8Array(snapshot.squares);
    } catch (error) {
      rethrowFastWorkspaceAllocation(error);
    }
    let looseManaCount = 0;
    for (let index = 0; index < BOARD_CELLS; index += 1) {
      const cell = u16(snapshot.cells, index);
      if (cell === 0 || cellOccupancy(cell) !== OCC_MANA) continue;
      looseManaCount += 1;
      if (looseManaCount > this.manaIndices.length) {
        throw new RangeError("fast mana index capacity exceeded");
      }
    }

    const whiteScore = snapshot.whiteScore;
    const blackScore = snapshot.blackScore;
    const active = snapshot.active;
    const monsMoves = snapshot.monsMoves;
    const manaMoves = snapshot.manaMoves;
    const actionsUsed = snapshot.actionsUsed;
    const firstTurn = snapshot.firstTurn;

    this.cells.set(snapshot.cells);
    this.squares = squares;
    this.whiteScore = whiteScore;
    this.blackScore = blackScore;
    this.active = active;
    this.monsMoves = monsMoves;
    this.manaMoves = manaMoves;
    this.actionsUsed = actionsUsed;
    this.potions.set(snapshot.potions);
    this.firstTurn = firstTurn;
    this.rebuildDerived();
  }

  private rebuildDerived(): void {
    this.monLocations.fill(-1);
    this.freeMana[0] = 0;
    this.freeMana[1] = 0;
    this.manaIndices.fill(0);
    this.manaCount = 0;
    let hashLo = 0;
    let hashHi = 0;
    for (let index = 0; index < BOARD_CELLS; index += 1) {
      const cell = u16(this.cells, index);
      if (cell === 0) continue;
      hashLo ^= cellHashLo(index, cell);
      hashHi ^= cellHashHi(index, cell);
      const occupancy = cellOccupancy(cell);
      if (occupancy === OCC_MON) {
        this.monLocations[monId(cellMonKind(cell), cellMonColor(cell))] = index;
      } else if (occupancy === OCC_MANA) {
        const mana = cellMana(cell);
        if (mana === 1 || mana === 2) {
          this.freeMana[mana - 1] = i32(this.freeMana, mana - 1) + 1;
        }
        this.manaIndices[this.manaCount] = index;
        this.manaCount += 1;
      }
    }
    this.hashLo = hashLo;
    this.hashHi = hashHi;
  }
}

export const FastPosition: new () => FastPosition = MutableFastPosition;

export function awakeAngelGuards(
  position: FastPosition,
  angelColor: number,
  target: number,
): boolean {
  const angelIndex = i32(position.monLocations, monId(KIND_ANGEL, angelColor));
  if (angelIndex < 0) return false;
  const angelCell = u16(position.cells, angelIndex);
  return (
    cellOccupancy(angelCell) === OCC_MON &&
    cellMonKind(angelCell) === KIND_ANGEL &&
    cellMonColor(angelCell) === angelColor &&
    cellCooldown(angelCell) === 0 &&
    chebyshev(angelIndex, target) === 1
  );
}

function removeManaIndex(position: MutableFastPosition, index: number): void {
  const count = position.manaCount;
  for (let slot = 0; slot < count; slot += 1) {
    if (i32(position.manaIndices, slot) !== index) continue;
    for (let next = slot + 1; next < count; next += 1) {
      position.manaIndices[next - 1] = i32(position.manaIndices, next);
    }
    position.manaIndices[count - 1] = 0;
    position.manaCount = count - 1;
    return;
  }
  throw new RangeError(`missing fast mana index: ${index}`);
}

function insertManaIndex(position: MutableFastPosition, index: number): void {
  const count = position.manaCount;
  if (count >= position.manaIndices.length) {
    throw new RangeError("fast mana index capacity exceeded");
  }
  let slot = count;
  while (slot > 0 && i32(position.manaIndices, slot - 1) > index) {
    position.manaIndices[slot] = i32(position.manaIndices, slot - 1);
    slot -= 1;
  }
  position.manaIndices[slot] = index;
  position.manaCount = count + 1;
}

function setCell(position: MutableFastPosition, index: number, value: number): void {
  const previous = u16(position.cells, index);
  if (previous === value) return;
  if (previous !== 0) {
    position.hashLo ^= cellHashLo(index, previous);
    position.hashHi ^= cellHashHi(index, previous);
    const previousOccupancy = cellOccupancy(previous);
    if (previousOccupancy === OCC_MON) {
      const previousId = monId(cellMonKind(previous), cellMonColor(previous));
      if (i32(position.monLocations, previousId) === index) {
        position.monLocations[previousId] = -1;
      }
    } else if (previousOccupancy === OCC_MANA) {
      const mana = cellMana(previous);
      if (mana === 1 || mana === 2) {
        position.freeMana[mana - 1] = i32(position.freeMana, mana - 1) - 1;
      }
      removeManaIndex(position, index);
    }
  }
  position.cells[index] = value;
  if (value !== 0) {
    position.hashLo ^= cellHashLo(index, value);
    position.hashHi ^= cellHashHi(index, value);
    const occupancy = cellOccupancy(value);
    if (occupancy === OCC_MON) {
      position.monLocations[monId(cellMonKind(value), cellMonColor(value))] = index;
    } else if (occupancy === OCC_MANA) {
      const mana = cellMana(value);
      if (mana === 1 || mana === 2) {
        position.freeMana[mana - 1] = i32(position.freeMana, mana - 1) + 1;
      }
      insertManaIndex(position, index);
    }
  }
}

function plainMonCell(cell: number): number {
  return makeMonCell(cellMonKind(cell), cellMonColor(cell), cellCooldown(cell), 0, 0);
}

function carryingMonCell(cell: number, mana: number, consumable: number): number {
  return makeMonCell(
    cellMonKind(cell),
    cellMonColor(cell),
    cellCooldown(cell),
    mana,
    consumable,
  );
}

function addPotion(position: MutableFastPosition, color: number): void {
  position.potions[color] = i32(position.potions, color) + 1;
}

function collectConsumableAt(
  position: MutableFastPosition,
  mon: number,
  at: number,
  modifier: number,
): void {
  if (cellMana(mon) === 0 && cellConsumable(mon) === 0 && modifier === MOD_BOMB) {
    setCell(position, at, carryingMonCell(mon, 0, CONS_BOMB));
    return;
  }
  setCell(position, at, mon);
  addPotion(position, cellMonColor(mon));
}

function addScore(position: MutableFastPosition, amount: number): void {
  if (position.active === 0) position.whiteScore += amount;
  else position.blackScore += amount;
}

function consumeAction(position: MutableFastPosition): void {
  if (position.actionsUsed < ACTIONS_PER_TURN) {
    position.actionsUsed += 1;
    return;
  }
  position.potions[position.active] = i32(position.potions, position.active) - 1;
}

function faintMonAt(position: MutableFastPosition, cell: number): void {
  const kind = cellMonKind(cell);
  const color = cellMonColor(cell);
  const base = i32(MON_BASE_INDEX, monId(kind, color));
  setCell(position, base, makeMonCell(kind, color, 2, 0, 0));
}

function supermanaBackToBase(position: MutableFastPosition): void {
  const existing = u16(position.cells, SUPERMANA_BASE_INDEX);
  if (
    cellOccupancy(existing) === OCC_MON &&
    cellMana(existing) === 0 &&
    cellConsumable(existing) === 0
  ) {
    setCell(position, SUPERMANA_BASE_INDEX, carryingMonCell(existing, MANA_SUPER, 0));
  } else {
    setCell(position, SUPERMANA_BASE_INDEX, makeManaCell(MANA_SUPER));
  }
}

function releaseFaintedCarry(
  position: MutableFastPosition,
  target: number,
  at: number,
): void {
  const mana = cellMana(target);
  if (mana === MANA_SUPER) {
    supermanaBackToBase(position);
  } else if (mana !== 0) {
    setCell(position, at, makeManaCell(mana));
  }
  if (cellConsumable(target) === CONS_BOMB) {
    const targetBase = i32(
      MON_BASE_INDEX,
      monId(cellMonKind(target), cellMonColor(target)),
    );
    if (at !== targetBase) setCell(position, at, 0);
  }
}

function faintTargetAndReleaseCarry(
  position: MutableFastPosition,
  target: number,
  at: number,
): void {
  faintMonAt(position, target);
  releaseFaintedCarry(position, target, at);
}

function scoreCarriedManaAt(
  position: MutableFastPosition,
  at: number,
  mana: number,
): void {
  addScore(position, manaScoreValue(mana, position.active));
  const current = u16(position.cells, at);
  if (cellOccupancy(current) === OCC_MON) {
    setCell(position, at, plainMonCell(current));
  } else {
    setCell(position, at, 0);
  }
}

function applyMonMove(position: MutableFastPosition, move: number): void {
  const from = moveFrom(move);
  const to = moveTo(move);
  const start = u16(position.cells, from);
  const target = u16(position.cells, to);
  const startMana = cellMana(start);
  position.monsMoves += 1;
  setCell(position, from, 0);
  setCell(position, to, start);

  if (target !== 0) {
    const occupancy = cellOccupancy(target);
    if (occupancy === OCC_MANA) {
      if (startMana !== 0) setCell(position, from, makeManaCell(startMana));
      setCell(position, to, carryingMonCell(start, cellMana(target), 0));
    } else if (occupancy === OCC_CONS && cellConsumable(target) === CONS_BOTH) {
      collectConsumableAt(position, start, to, moveModifier(move));
    }
  }

  if (startMana !== 0 && squareIsPool(u8(position.squares, to))) {
    scoreCarriedManaAt(position, to, startMana);
  }
}

function applyManaMove(position: MutableFastPosition, move: number): void {
  const from = moveFrom(move);
  const to = moveTo(move);
  const mana = cellMana(u16(position.cells, from));
  const target = u16(position.cells, to);
  position.manaMoves += 1;
  setCell(position, from, 0);
  setCell(position, to, makeManaCell(mana));
  if (target !== 0) {
    setCell(position, to, carryingMonCell(target, mana, 0));
  }
  if (squareIsPool(u8(position.squares, to))) {
    scoreCarriedManaAt(position, to, mana);
  }
}

function applyMysticAction(position: MutableFastPosition, move: number): void {
  const to = moveTo(move);
  const target = u16(position.cells, to);
  consumeAction(position);
  setCell(position, to, 0);
  faintTargetAndReleaseCarry(position, target, to);
}

function applyDemonAction(position: MutableFastPosition, move: number): boolean {
  const from = moveFrom(move);
  const to = moveTo(move);
  const modifier = moveModifier(move);
  const aux = modifier === MOD_IGNORED_STEP ? AUX_NONE : moveAux(move);
  const demon = u16(position.cells, from);
  const target = u16(position.cells, to);
  const demonBase = i32(MON_BASE_INDEX, monId(cellMonKind(demon), cellMonColor(demon)));
  const representable =
    cellConsumable(target) !== CONS_BOMB || aux === AUX_NONE || aux === demonBase;
  setCell(position, from, 0);
  if (aux === AUX_NONE) {
    setCell(position, to, plainMonCell(demon));
  } else {
    setCell(position, to, 0);
  }
  consumeAction(position);
  if (target !== 0) {
    faintTargetAndReleaseCarry(position, target, to);
    if (cellConsumable(target) === CONS_BOMB) {
      faintMonAt(position, demon);
    }
  }
  if (aux !== AUX_NONE) {
    if (modifier === MOD_BOMB) {
      setCell(position, aux, carryingMonCell(demon, 0, CONS_BOMB));
    } else {
      setCell(position, aux, plainMonCell(demon));
      if (modifier === MOD_POTION) {
        addPotion(position, cellMonColor(demon));
      }
    }
  }
  return representable;
}

function applyBombAttack(position: MutableFastPosition, move: number): void {
  const from = moveFrom(move);
  const to = moveTo(move);
  const attacker = u16(position.cells, from);
  const target = u16(position.cells, to);
  setCell(position, to, 0);
  setCell(position, from, plainMonCell(attacker));
  faintTargetAndReleaseCarry(position, target, to);
}

function applySpiritAction(position: MutableFastPosition, move: number): void {
  const to = moveTo(move);
  const aux = moveAux(move);
  const modifier = moveModifier(move);
  const target = u16(position.cells, to);
  const destination = u16(position.cells, aux);
  const targetMana = cellMana(target);
  consumeAction(position);
  setCell(position, to, 0);
  setCell(position, aux, target);

  if (destination !== 0) {
    const targetOccupancy = cellOccupancy(target);
    if (targetOccupancy === OCC_MON) {
      const destinationOccupancy = cellOccupancy(destination);
      if (destinationOccupancy === OCC_MANA) {
        if (targetMana !== 0) setCell(position, to, makeManaCell(targetMana));
        setCell(position, aux, carryingMonCell(target, cellMana(destination), 0));
      } else if (destinationOccupancy === OCC_CONS) {
        collectConsumableAt(position, target, aux, modifier);
      }
    } else if (targetOccupancy === OCC_MANA) {
      setCell(position, aux, carryingMonCell(destination, targetMana, 0));
    } else if (targetOccupancy === OCC_CONS) {
      collectConsumableAt(position, destination, aux, modifier);
    }
  }

  if (targetMana !== 0 && squareIsPool(u8(position.squares, aux))) {
    scoreCarriedManaAt(position, aux, targetMana);
  }
}

export function positionWinner(position: FastPosition): number {
  const winner = winnerForScores(position.whiteScore, position.blackScore);
  return winner === undefined ? -1 : colorId(winner);
}

export function canUseAction(position: FastPosition): boolean {
  return canUseActionForCounts(
    position.firstTurn,
    position.actionsUsed,
    i32(position.potions, position.active),
  );
}

export function manaStartsAllowed(
  position: FastPosition,
  rawMonStartCount: number,
): boolean {
  if (i32(position.freeMana, position.active) === 0) return false;
  return shouldSuggestRegularManaStartsFromScalars(
    position.firstTurn,
    position.monsMoves,
    position.manaMoves,
    position.actionsUsed,
    i32(position.potions, position.active),
    rawMonStartCount !== 0,
    true,
  );
}

export function manaMoveAllowed(position: FastPosition, destination: number): boolean {
  if (!squareIsRegularForMovement(u8(position.squares, destination))) return false;
  const target = u16(position.cells, destination);
  return (
    target === 0 ||
    (cellOccupancy(target) === OCC_MON &&
      cellMonKind(target) === KIND_DRAINER &&
      cellMana(target) === 0 &&
      cellConsumable(target) === 0)
  );
}

function hasMovableFreeRegularMana(position: FastPosition): boolean {
  const mana = position.active + 1;
  for (let slot = 0; slot < position.manaCount; slot += 1) {
    const index = i32(position.manaIndices, slot);
    const cell = u16(position.cells, index);
    if (cellOccupancy(cell) !== OCC_MANA || cellMana(cell) !== mana) continue;
    const start = i32(NEIGHBORS.starts, index);
    const count = i32(NEIGHBORS.counts, index);
    for (let offset = 0; offset < count; offset += 1) {
      const destination = i32(NEIGHBORS.list, start + offset);
      if (manaMoveAllowed(position, destination)) return true;
    }
  }
  return false;
}

function shouldAdvanceTurn(position: FastPosition): boolean {
  const needsFreeManaCheck =
    !position.firstTurn &&
    canMoveManaForCounts(position.firstTurn, position.manaMoves) &&
    !canMoveMonForCounts(position.monsMoves);
  const movableFreeRegularMana =
    !needsFreeManaCheck ||
    (i32(position.freeMana, position.active) !== 0 &&
      hasMovableFreeRegularMana(position));
  return shouldAdvanceTurnForCounts(
    position.firstTurn,
    position.monsMoves,
    position.manaMoves,
    movableFreeRegularMana,
  );
}

function advanceTurn(position: MutableFastPosition): void {
  position.active ^= 1;
  position.firstTurn = false;
  position.actionsUsed = 0;
  position.manaMoves = 0;
  position.monsMoves = 0;
  const active = position.active;
  for (let kind = 0; kind < MON_KIND_COUNT; kind += 1) {
    const index = i32(position.monLocations, monId(kind, active));
    if (index < 0) continue;
    const cell = u16(position.cells, index);
    const cooldown = cellCooldown(cell);
    if (cooldown === 0 || cellMana(cell) !== 0 || cellConsumable(cell) !== 0) {
      continue;
    }
    setCell(
      position,
      index,
      makeMonCell(
        cellMonKind(cell),
        cellMonColor(cell),
        cooldown - 1,
        cellMana(cell),
        cellConsumable(cell),
      ),
    );
  }
}

export function applyFastMove(position: FastPosition, move: number): number {
  const mutablePosition = position as MutableFastPosition;
  const type = moveType(move);
  let representable = true;
  switch (type) {
    case MOVE_MON:
      applyMonMove(mutablePosition, move);
      break;
    case MOVE_MANA:
      applyManaMove(mutablePosition, move);
      break;
    case MOVE_MYSTIC:
      applyMysticAction(mutablePosition, move);
      break;
    case MOVE_DEMON:
      representable = applyDemonAction(mutablePosition, move);
      break;
    case MOVE_SPIRIT:
      applySpiritAction(mutablePosition, move);
      break;
    case MOVE_BOMB:
      applyBombAttack(mutablePosition, move);
      break;
    default:
      throw new RangeError(`unsupported fast move type: ${type}`);
  }
  const winner = positionWinner(mutablePosition);
  if (winner === -1 && shouldAdvanceTurn(mutablePosition)) {
    advanceTurn(mutablePosition);
  }
  return representable ? winner : FAST_MOVE_UNREPRESENTABLE;
}
