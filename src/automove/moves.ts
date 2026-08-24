import { canMoveManaForCounts, canMoveMonForCounts } from "../engine/rules/legality.js";
import { TARGET_SCORE } from "../engine/board/config.js";
import {
  BOMB_TARGETS,
  CONS_BOMB,
  CONS_BOTH,
  POOL_DISTANCE,
  manaScoreValue,
  squareIsPool,
  DEMON_TARGETS,
  KIND_DEMON,
  KIND_DRAINER,
  KIND_MYSTIC,
  KIND_SPIRIT,
  MANA_SUPER,
  MON_KIND_COUNT,
  MYSTIC_TARGETS,
  NEIGHBORS,
  OCC_CONS,
  OCC_MANA,
  OCC_MON,
  SPIRIT_TARGETS,
  SQ_MON_BASE,
  SQ_SUPERMANA_BASE,
  cellConsumable,
  cellCooldown,
  cellMana,
  cellMonColor,
  cellMonKind,
  cellOccupancy,
  damagingAttackCanComplete,
  i32,
  midpoint,
  monId,
  squareMonBaseColor,
  squareMonBaseKind,
  squareIsRegularForMovement,
  u16,
  u8,
} from "./board.js";
import {
  AUX_NONE,
  MOD_BOMB,
  MOD_IGNORED_STEP,
  MOD_NONE,
  MOD_POTION,
  MOVE_BOMB,
  MOVE_DEMON,
  MOVE_MANA,
  MOVE_MON,
  MOVE_MYSTIC,
  MOVE_SPIRIT,
  awakeAngelGuards,
  canUseAction,
  encodeMove,
  manaMoveAllowed,
  manaStartsAllowed,
  type FastPosition,
} from "./state.js";

// Fixed supported workspace; overflow rejects the packed state instead of truncating.
export const MAX_MOVES = 704;

const RAW_START_OPTION_FLAG = 0x4000_0000;

function encodeGeneratedMoves(count: number, hadRawStartOption: boolean): number {
  return hadRawStartOption ? count + RAW_START_OPTION_FLAG : count;
}

function generatedMoveCount(result: number): number {
  return result >= RAW_START_OPTION_FLAG ? result - RAW_START_OPTION_FLAG : result;
}

function generatedMovesHadRawStartOption(result: number): boolean {
  return result >= RAW_START_OPTION_FLAG;
}

function isEnemyAwakeMon(cell: number, active: number): boolean {
  return (
    cellOccupancy(cell) === OCC_MON &&
    cellMonColor(cell) !== active &&
    cellCooldown(cell) === 0
  );
}

function damagingAttackKey(target: number): number {
  let key = 1 << 17;
  if (cellMana(target) !== 0) key += 1 << 20;
  if (cellMonKind(target) === KIND_DRAINER) key += 1 << 19;
  return key;
}

function spiritDestinationOptionCount(
  position: FastPosition,
  target: number,
  destinationIndex: number,
): 0 | 1 | 2 {
  const destination = u16(position.cells, destinationIndex);
  const square = u8(position.squares, destinationIndex);
  const targetOccupancy = cellOccupancy(target);
  const targetIsMon = targetOccupancy === OCC_MON;
  const targetMana = cellMana(target);
  const targetConsumable = cellConsumable(target);
  const destinationOccupancy = destination === 0 ? -1 : cellOccupancy(destination);
  const destinationCarries =
    destinationOccupancy === OCC_MON &&
    (cellMana(destination) !== 0 || cellConsumable(destination) !== 0);

  if (destinationOccupancy === OCC_MON) {
    if (destinationCarries) {
      if (targetOccupancy !== OCC_CONS || targetConsumable !== CONS_BOTH) {
        return 0;
      }
    } else if (targetIsMon) {
      return 0;
    } else if (targetOccupancy === OCC_MANA) {
      if (
        cellMonKind(destination) !== KIND_DRAINER ||
        cellCooldown(destination) !== 0
      ) {
        return 0;
      }
    } else if (targetConsumable !== CONS_BOTH) {
      return 0;
    }
  } else if (destinationOccupancy === OCC_MANA) {
    if (
      !targetIsMon ||
      cellMonKind(target) !== KIND_DRAINER ||
      cellCooldown(target) !== 0
    ) {
      return 0;
    }
  } else if (destinationOccupancy === OCC_CONS) {
    if (cellConsumable(destination) !== CONS_BOTH || !targetIsMon) return 0;
  }

  if (square >= SQ_MON_BASE) {
    if (
      !targetIsMon ||
      squareMonBaseKind(square) !== cellMonKind(target) ||
      squareMonBaseColor(square) !== cellMonColor(target) ||
      targetMana !== 0 ||
      targetConsumable !== 0
    ) {
      return 0;
    }
  } else if (square === SQ_SUPERMANA_BASE) {
    const destinationHasSupermana =
      destinationOccupancy === OCC_MANA && cellMana(destination) === MANA_SUPER;
    const targetIsDrainer = targetIsMon && cellMonKind(target) === KIND_DRAINER;
    const allowed =
      targetMana === MANA_SUPER ||
      (targetMana === 0 &&
        targetIsDrainer &&
        (destination === 0 || destinationHasSupermana)) ||
      (targetIsDrainer && destinationHasSupermana);
    if (!allowed) return 0;
  }

  if (destination === 0) return 1;

  if (targetIsMon) {
    if (targetMana === 0 && targetConsumable === 0) {
      if (destinationOccupancy === OCC_CONS) {
        return cellConsumable(destination) === CONS_BOTH ? 2 : 0;
      }
      return 1;
    }
    if (targetMana !== 0) {
      if (destinationOccupancy === OCC_CONS) {
        return cellConsumable(destination) === CONS_BOTH ? 1 : 0;
      }
      return 1;
    }
    if (
      destinationOccupancy !== OCC_CONS ||
      cellConsumable(destination) !== CONS_BOTH
    ) {
      return 0;
    }
    return 1;
  }

  if (targetOccupancy === OCC_MANA) {
    if (destinationOccupancy !== OCC_MON || destinationCarries) return 0;
    return 1;
  }

  if (destinationOccupancy !== OCC_MON) return 0;
  if (!destinationCarries) return 2;
  return targetConsumable === CONS_BOTH ? 1 : 0;
}

function generateMonMoves(
  position: FastPosition,
  from: number,
  cell: number,
  out: Int32Array,
  start: number,
  keys: Int32Array,
): number {
  let count = start;
  let hadRawStartOption = false;
  const kind = cellMonKind(cell);
  const color = cellMonColor(cell);
  const carriedMana = cellMana(cell);
  const carriedConsumable = cellConsumable(cell);
  const active = position.active;
  const fromPoolDistance = u8(POOL_DISTANCE, from);
  const plainBase = kind === KIND_DRAINER ? 1024 : 0;
  const neighborStart = i32(NEIGHBORS.starts, from);
  const neighborCount = i32(NEIGHBORS.counts, from);
  for (let offset = 0; offset < neighborCount; offset += 1) {
    const to = i32(NEIGHBORS.list, neighborStart + offset);
    const target = u16(position.cells, to);
    const square = u8(position.squares, to);
    const targetOccupancy = target === 0 ? -1 : cellOccupancy(target);
    let allowed: boolean;
    if (carriedMana !== 0) {
      if (targetOccupancy === OCC_MON) allowed = false;
      else if (targetOccupancy === OCC_MANA || targetOccupancy === OCC_CONS) {
        allowed = true;
      } else {
        allowed =
          squareIsRegularForMovement(square) ||
          (square === SQ_SUPERMANA_BASE && carriedMana === MANA_SUPER);
      }
    } else if (carriedConsumable !== 0) {
      if (targetOccupancy === OCC_CONS) {
        allowed = true;
      } else if (targetOccupancy === -1) {
        allowed = squareIsRegularForMovement(square);
      } else allowed = false;
    } else {
      let itemAllows: boolean;
      if (targetOccupancy === OCC_MON) itemAllows = false;
      else if (targetOccupancy === OCC_MANA) itemAllows = kind === KIND_DRAINER;
      else itemAllows = true;
      if (!itemAllows) {
        allowed = false;
      } else if (square >= SQ_MON_BASE) {
        allowed =
          squareMonBaseKind(square) === kind && squareMonBaseColor(square) === color;
      } else if (square === SQ_SUPERMANA_BASE) {
        allowed =
          kind === KIND_DRAINER &&
          (target === 0 ||
            (targetOccupancy === OCC_MANA && cellMana(target) === MANA_SUPER));
      } else {
        allowed = true;
      }
    }
    if (!allowed) continue;
    hadRawStartOption = true;
    if (targetOccupancy === OCC_CONS && cellConsumable(target) !== CONS_BOTH) {
      continue;
    }
    let key = 0;
    if (carriedMana !== 0 && squareIsPool(square)) {
      key += (1 << 21) + manaScoreValue(carriedMana, active) * (1 << 18);
    }
    if (target !== 0) {
      if (targetOccupancy === OCC_MANA) {
        key += (1 << 16) + manaScoreValue(cellMana(target), active) * 512;
      } else if (targetOccupancy === OCC_CONS) {
        key += 1 << 14;
      }
    }
    if (carriedMana !== 0) {
      key += (fromPoolDistance - u8(POOL_DISTANCE, to)) * 2048 + 4096;
    } else {
      key += plainBase;
    }
    if (
      targetOccupancy === OCC_CONS &&
      cellConsumable(target) === CONS_BOTH &&
      carriedMana === 0 &&
      carriedConsumable === 0
    ) {
      keys[count] = key;
      out[count] = encodeMove(MOVE_MON, from, to, AUX_NONE, MOD_BOMB);
      count += 1;
      keys[count] = key;
      out[count] = encodeMove(MOVE_MON, from, to, AUX_NONE, MOD_POTION);
      count += 1;
    } else {
      keys[count] = key;
      out[count] = encodeMove(MOVE_MON, from, to, AUX_NONE, MOD_NONE);
      count += 1;
    }
  }
  return encodeGeneratedMoves(count, hadRawStartOption);
}

function generateMysticMoves(
  position: FastPosition,
  from: number,
  out: Int32Array,
  start: number,
  keys: Int32Array,
): number {
  const active = position.active;
  let count = start;
  let hadRawStartOption = false;
  const targetStart = i32(MYSTIC_TARGETS.starts, from);
  const targetCount = i32(MYSTIC_TARGETS.counts, from);
  for (let offset = 0; offset < targetCount; offset += 1) {
    const to = i32(MYSTIC_TARGETS.list, targetStart + offset);
    const target = u16(position.cells, to);
    if (!isEnemyAwakeMon(target, active)) continue;
    if (awakeAngelGuards(position, active ^ 1, to)) continue;
    hadRawStartOption = true;
    if (!damagingAttackCanComplete(target)) continue;
    keys[count] = damagingAttackKey(target);
    out[count] = encodeMove(MOVE_MYSTIC, from, to, AUX_NONE, MOD_NONE);
    count += 1;
  }
  return encodeGeneratedMoves(count, hadRawStartOption);
}

function generateDemonMoves(
  position: FastPosition,
  from: number,
  out: Int32Array,
  start: number,
  keys: Int32Array,
): number {
  const active = position.active;
  let count = start;
  let hadRawStartOption = false;
  const kind = KIND_DEMON;
  const color = active;
  const targetStart = i32(DEMON_TARGETS.starts, from);
  const targetCount = i32(DEMON_TARGETS.counts, from);
  for (let offset = 0; offset < targetCount; offset += 1) {
    const to = i32(DEMON_TARGETS.list, targetStart + offset);
    const target = u16(position.cells, to);
    if (!isEnemyAwakeMon(target, active)) continue;
    if (awakeAngelGuards(position, active ^ 1, to)) continue;
    const between = midpoint(from, to);
    if (u16(position.cells, between) !== 0) continue;
    const betweenSquare = u8(position.squares, between);
    if (betweenSquare === SQ_SUPERMANA_BASE || betweenSquare >= SQ_MON_BASE) {
      continue;
    }
    hadRawStartOption = true;
    if (!damagingAttackCanComplete(target)) continue;
    const key = damagingAttackKey(target);
    const targetSquare = u8(position.squares, to);
    const targetMana = cellMana(target);
    const requiresStep =
      (targetMana !== 0 && targetMana !== MANA_SUPER) ||
      targetSquare === SQ_SUPERMANA_BASE ||
      targetSquare >= SQ_MON_BASE;
    if (!requiresStep) {
      keys[count] = key;
      out[count] = encodeMove(MOVE_DEMON, from, to, AUX_NONE, MOD_NONE);
      count += 1;
      continue;
    }
    const before = count;
    const stepStart = i32(NEIGHBORS.starts, to);
    const stepCount = i32(NEIGHBORS.counts, to);
    for (let stepOffset = 0; stepOffset < stepCount; stepOffset += 1) {
      const step = i32(NEIGHBORS.list, stepStart + stepOffset);
      const stepCell = u16(position.cells, step);
      const stepOccupancy = stepCell === 0 ? -1 : cellOccupancy(stepCell);
      if (stepOccupancy !== -1 && stepOccupancy !== OCC_CONS) continue;
      const stepSquare = u8(position.squares, step);
      const allowed =
        squareIsRegularForMovement(stepSquare) ||
        (stepSquare >= SQ_MON_BASE &&
          squareMonBaseKind(stepSquare) === kind &&
          squareMonBaseColor(stepSquare) === color);
      if (!allowed) continue;
      if (stepOccupancy === OCC_CONS) {
        if (cellConsumable(stepCell) === CONS_BOTH) {
          keys[count] = key;
          out[count] = encodeMove(MOVE_DEMON, from, to, step, MOD_BOMB);
          count += 1;
          keys[count] = key;
          out[count] = encodeMove(MOVE_DEMON, from, to, step, MOD_POTION);
          count += 1;
        } else {
          keys[count] = key;
          out[count] = encodeMove(MOVE_DEMON, from, to, step, MOD_IGNORED_STEP);
          count += 1;
        }
      } else {
        keys[count] = key;
        out[count] = encodeMove(MOVE_DEMON, from, to, step, MOD_NONE);
        count += 1;
      }
    }
    if (count === before) {
      keys[count] = key;
      out[count] = encodeMove(MOVE_DEMON, from, to, AUX_NONE, MOD_NONE);
      count += 1;
    }
  }
  return encodeGeneratedMoves(count, hadRawStartOption);
}

// A spirit push earns the tactical exemption from reduction and pruning only when it does
// something a quiet mon step cannot: carry a mana-bearing item toward a pool, or hand mana to
// an own awake drainer. Sideways and outward pushes are the bulk of the tactical class.
function generateSpiritMoves(
  position: FastPosition,
  from: number,
  out: Int32Array,
  start: number,
  keys: Int32Array,
): number {
  let count = start;
  let hadRawStartOption = false;
  const active = position.active;
  const targetStart = i32(SPIRIT_TARGETS.starts, from);
  const targetCount = i32(SPIRIT_TARGETS.counts, from);
  for (let offset = 0; offset < targetCount; offset += 1) {
    const to = i32(SPIRIT_TARGETS.list, targetStart + offset);
    const target = u16(position.cells, to);
    if (target === 0) continue;
    if (cellOccupancy(target) === OCC_MON && cellCooldown(target) !== 0) {
      continue;
    }
    hadRawStartOption = true;
    const targetManaForKey = cellMana(target);
    const targetIsLooseMana = cellOccupancy(target) === OCC_MANA;
    const targetPoolDistance = u8(POOL_DISTANCE, to);
    const manaKeyBonus =
      (1 << 21) + manaScoreValue(targetManaForKey, active) * (1 << 18);
    const destinationStart = i32(NEIGHBORS.starts, to);
    const destinationCount = i32(NEIGHBORS.counts, to);
    for (let index = 0; index < destinationCount; index += 1) {
      const destination = i32(NEIGHBORS.list, destinationStart + index);
      const optionCount = spiritDestinationOptionCount(position, target, destination);
      if (optionCount === 0) continue;
      let key = 0;
      if (targetManaForKey !== 0 && squareIsPool(u8(position.squares, destination))) {
        key += (1 << 13) + manaKeyBonus;
      } else if (
        targetIsLooseMana &&
        advancesMana(position, destination, targetPoolDistance, active)
      ) {
        key += (1 << 13) + (1 << 15);
      }
      if (optionCount === 2) {
        keys[count] = key;
        out[count] = encodeMove(MOVE_SPIRIT, from, to, destination, MOD_BOMB);
        count += 1;
        keys[count] = key;
        out[count] = encodeMove(MOVE_SPIRIT, from, to, destination, MOD_POTION);
        count += 1;
      } else {
        keys[count] = key;
        out[count] = encodeMove(MOVE_SPIRIT, from, to, destination, MOD_NONE);
        count += 1;
      }
    }
  }
  return encodeGeneratedMoves(count, hadRawStartOption);
}

function advancesMana(
  position: FastPosition,
  destination: number,
  fromPoolDistance: number,
  active: number,
): boolean {
  if (u8(POOL_DISTANCE, destination) < fromPoolDistance) return true;
  const cell = u16(position.cells, destination);
  return (
    cellOccupancy(cell) === OCC_MON &&
    cellMonKind(cell) === KIND_DRAINER &&
    cellMonColor(cell) === active &&
    cellCooldown(cell) === 0
  );
}

function generateBombMoves(
  position: FastPosition,
  from: number,
  out: Int32Array,
  start: number,
  keys: Int32Array,
): number {
  const active = position.active;
  let count = start;
  let hadRawStartOption = false;
  const targetStart = i32(BOMB_TARGETS.starts, from);
  const targetCount = i32(BOMB_TARGETS.counts, from);
  for (let offset = 0; offset < targetCount; offset += 1) {
    const to = i32(BOMB_TARGETS.list, targetStart + offset);
    const target = u16(position.cells, to);
    if (!isEnemyAwakeMon(target, active)) continue;
    hadRawStartOption = true;
    if (!damagingAttackCanComplete(target)) continue;
    keys[count] = damagingAttackKey(target);
    out[count] = encodeMove(MOVE_BOMB, from, to, AUX_NONE, MOD_NONE);
    count += 1;
  }
  return encodeGeneratedMoves(count, hadRawStartOption);
}

function generateManaMoves(
  position: FastPosition,
  from: number,
  out: Int32Array,
  start: number,
  keys: Int32Array,
): number {
  let count = start;
  const neighborStart = i32(NEIGHBORS.starts, from);
  const neighborCount = i32(NEIGHBORS.counts, from);
  for (let offset = 0; offset < neighborCount; offset += 1) {
    const to = i32(NEIGHBORS.list, neighborStart + offset);
    const square = u8(position.squares, to);
    if (!manaMoveAllowed(position, to)) continue;
    keys[count] = squareIsPool(square) ? (1 << 21) + (1 << 18) : 16;
    out[count] = encodeMove(MOVE_MANA, from, to, AUX_NONE, MOD_NONE);
    count += 1;
  }
  return count;
}

function generateWinningManaMoves(
  position: FastPosition,
  from: number,
  out: Int32Array,
  start: number,
  keys: Int32Array,
): number {
  let count = start;
  if (u8(POOL_DISTANCE, from) !== 1) return count;
  const neighborStart = i32(NEIGHBORS.starts, from);
  const neighborCount = i32(NEIGHBORS.counts, from);
  for (let offset = 0; offset < neighborCount; offset += 1) {
    const to = i32(NEIGHBORS.list, neighborStart + offset);
    const square = u8(position.squares, to);
    if (!squareIsPool(square) || !manaMoveAllowed(position, to)) continue;
    keys[count] = (1 << 21) + (1 << 18);
    out[count] = encodeMove(MOVE_MANA, from, to, AUX_NONE, MOD_NONE);
    count += 1;
  }
  return count;
}

const FALLBACK_KEYS = new Int32Array(MAX_MOVES);

export function generateMoves(
  position: FastPosition,
  out: Int32Array,
  keys: Int32Array = FALLBACK_KEYS,
): number {
  const active = position.active;
  const canMove = canMoveMonForCounts(position.monsMoves);
  const actionAvailable = canUseAction(position);
  let count = 0;
  let rawMonStartCount = 0;

  for (let kind = 0; kind < MON_KIND_COUNT; kind += 1) {
    const from = i32(position.monLocations, monId(kind, active));
    if (from < 0) continue;
    const cell = u16(position.cells, from);
    if (cellOccupancy(cell) !== OCC_MON) continue;
    let hasRawStartOption = false;
    const carriedMana = cellMana(cell);
    const carriedConsumable = cellConsumable(cell);
    const carriesItem = carriedMana !== 0 || carriedConsumable !== 0;
    if (!carriesItem && cellCooldown(cell) !== 0) continue;

    if (canMove) {
      const result = generateMonMoves(position, from, cell, out, count, keys);
      count = generatedMoveCount(result);
      hasRawStartOption ||= generatedMovesHadRawStartOption(result);
    }

    if (carriesItem) {
      if (carriedConsumable === CONS_BOMB) {
        const result = generateBombMoves(position, from, out, count, keys);
        count = generatedMoveCount(result);
        hasRawStartOption ||= generatedMovesHadRawStartOption(result);
      }
    } else {
      const square = u8(position.squares, from);
      if (square < SQ_MON_BASE && actionAvailable) {
        if (kind === KIND_MYSTIC) {
          const result = generateMysticMoves(position, from, out, count, keys);
          count = generatedMoveCount(result);
          hasRawStartOption ||= generatedMovesHadRawStartOption(result);
        } else if (kind === KIND_DEMON) {
          const result = generateDemonMoves(position, from, out, count, keys);
          count = generatedMoveCount(result);
          hasRawStartOption ||= generatedMovesHadRawStartOption(result);
        } else if (kind === KIND_SPIRIT) {
          const result = generateSpiritMoves(position, from, out, count, keys);
          count = generatedMoveCount(result);
          hasRawStartOption ||= generatedMovesHadRawStartOption(result);
        }
      }
    }

    if (hasRawStartOption) rawMonStartCount += 1;
  }

  if (manaStartsAllowed(position, rawMonStartCount)) {
    const wanted = active + 1;
    for (let slot = 0; slot < position.manaCount; slot += 1) {
      const index = i32(position.manaIndices, slot);
      const cell = u16(position.cells, index);
      if (cellOccupancy(cell) !== OCC_MANA) continue;
      if (cellMana(cell) !== wanted) continue;
      count = generateManaMoves(position, index, out, count, keys);
    }
  } else if (
    i32(position.freeMana, active) !== 0 &&
    canMoveManaForCounts(position.firstTurn, position.manaMoves) &&
    (active === 0 ? position.whiteScore : position.blackScore) + 1 >= TARGET_SCORE
  ) {
    const wanted = active + 1;
    for (let slot = 0; slot < position.manaCount; slot += 1) {
      const index = i32(position.manaIndices, slot);
      const cell = u16(position.cells, index);
      if (cellOccupancy(cell) !== OCC_MANA) continue;
      if (cellMana(cell) !== wanted) continue;
      count = generateWinningManaMoves(position, index, out, count, keys);
    }
  }

  if (count > out.length) {
    throw new RangeError(
      `fast move buffer capacity exceeded: generated ${count} moves for ${out.length} slots`,
    );
  }
  return count;
}
