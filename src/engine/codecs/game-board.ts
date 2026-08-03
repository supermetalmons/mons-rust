import { Board, MutableBoard } from "../board.js";
import {
  ACTIONS_PER_TURN,
  DEFAULT_GAME_VARIANT,
  GameVariant,
  gameVariantFromId,
  gameVariantId,
  MANA_MOVES_PER_TURN,
  MONS_MOVES_PER_TURN,
  TARGET_SCORE,
} from "../config.js";
import { Color, type Item } from "../domain.js";
import { BOARD_CELLS, BOARD_SIZE } from "../geometry.js";
import { colorFen, itemFen, parseItemFenAt } from "./domain-item.js";

const MAX_CANONICAL_SCORE = TARGET_SCORE + 1;
const EMPTY_RUN_FENS: readonly string[] = Object.freeze([
  "",
  "n01",
  "n02",
  "n03",
  "n04",
  "n05",
  "n06",
  "n07",
  "n08",
  "n09",
  "n10",
  "n11",
]);

/** The portion of game state serialized by the stable game FEN format. */
export type GameFenState = {
  board: Board;
  whiteScore: number;
  blackScore: number;
  activeColor: Color;
  actionsUsedCount: number;
  manaMovesCount: number;
  monsMovesCount: number;
  whitePotionsCount: number;
  blackPotionsCount: number;
  turnNumber: number;
};

export function boardFen(board: Board): string {
  let fen = "";
  let index = 0;
  for (let row = 0; row < BOARD_SIZE; row += 1) {
    if (row > 0) fen += "/";
    const rowEnd = index + BOARD_SIZE;
    let emptySpaceCount = 0;
    while (index < rowEnd) {
      const item = board.itemAtIndex(index);
      index += 1;
      if (item === undefined) {
        emptySpaceCount += 1;
        continue;
      }
      if (emptySpaceCount > 0) {
        fen += emptyRunFen(emptySpaceCount);
        emptySpaceCount = 0;
      }
      fen += itemFen(item);
    }
    if (emptySpaceCount > 0) {
      fen += emptyRunFen(emptySpaceCount);
    }
  }
  return fen;
}

export function parseBoardFen(
  fen: string,
  variant: GameVariant,
): Board | undefined {
  return parseBoardFenRange(fen, 0, fen.length, variant);
}

function parseBoardFenRange(
  fen: string,
  start: number,
  end: number,
  variant: GameVariant,
): Board | undefined {
  const items = new Array<Item | undefined>(BOARD_CELLS).fill(undefined);
  let characterIndex = start;
  for (let i = 0; i < BOARD_SIZE; i += 1) {
    let j = 0;
    let previousWasEmptyRun = false;
    while (characterIndex < end && fen.charCodeAt(characterIndex) !== 47) {
      if (fen.charCodeAt(characterIndex) === 110) {
        if (previousWasEmptyRun) return undefined;
        if (characterIndex + 2 >= end) return undefined;
        const tens = fen.charCodeAt(characterIndex + 1) - 48;
        const ones = fen.charCodeAt(characterIndex + 2) - 48;
        if (tens < 0 || tens > 9 || ones < 0 || ones > 9) return undefined;
        const count = tens * 10 + ones;
        if (count < 1 || j + count > BOARD_SIZE) return undefined;
        j += count;
        characterIndex += 3;
        previousWasEmptyRun = true;
        continue;
      }

      if (characterIndex + 2 >= end || j >= BOARD_SIZE) return undefined;
      const item = parseItemFenAt(fen, characterIndex);
      if (item === undefined) return undefined;
      items[i * BOARD_SIZE + j] = item;
      j += 1;
      characterIndex += 3;
      previousWasEmptyRun = false;
    }
    if (j !== BOARD_SIZE) return undefined;
    if (i < BOARD_SIZE - 1) {
      if (fen.charCodeAt(characterIndex) !== 47) return undefined;
      characterIndex += 1;
    }
  }
  if (characterIndex !== end) return undefined;

  return new MutableBoard(variant, items, undefined, true);
}

export function gameFen(game: GameFenState): string {
  const fen = `${game.whiteScore} ${game.blackScore} ${colorFen(game.activeColor)} ${game.actionsUsedCount} ${game.manaMovesCount} ${game.monsMovesCount} ${game.whitePotionsCount} ${game.blackPotionsCount} ${game.turnNumber} ${boardFen(game.board)}`;
  const variant = game.board.variant;
  return variant === DEFAULT_GAME_VARIANT
    ? fen
    : `${fen} ${gameVariantId(variant)}`;
}

export function parseGameFen(fen: string): GameFenState | undefined {
  let characterIndex = 0;
  const readInteger = (spaceTerminated: boolean): number => {
    const start = characterIndex;
    let parsed = 0;
    let leadingZero = false;
    while (characterIndex < fen.length) {
      const code = fen.charCodeAt(characterIndex);
      if (code === 32) break;
      if (code < 48 || code > 57) return -1;
      if (characterIndex === start) {
        leadingZero = code === 48;
      } else if (leadingZero) {
        return -1;
      }
      parsed = parsed * 10 + (code - 48);
      if (!Number.isSafeInteger(parsed)) return -1;
      characterIndex += 1;
    }
    if (characterIndex === start) return -1;
    if (spaceTerminated) {
      if (characterIndex >= fen.length) return -1;
      characterIndex += 1;
    } else if (characterIndex !== fen.length) {
      return -1;
    }
    return parsed;
  };

  const whiteScore = readInteger(true);
  const blackScore = readInteger(true);
  const colorCode = fen.charCodeAt(characterIndex);
  const activeColor =
    colorCode === 119
      ? Color.White
      : colorCode === 98
        ? Color.Black
        : undefined;
  if (activeColor === undefined || fen.charCodeAt(characterIndex + 1) !== 32) {
    return undefined;
  }
  characterIndex += 2;
  const actionsUsedCount = readInteger(true);
  const manaMovesCount = readInteger(true);
  const monsMovesCount = readInteger(true);
  const whitePotionsCount = readInteger(true);
  const blackPotionsCount = readInteger(true);
  const turnNumber = readInteger(true);
  if (
    whiteScore < 0 ||
    blackScore < 0 ||
    actionsUsedCount < 0 ||
    manaMovesCount < 0 ||
    monsMovesCount < 0 ||
    whitePotionsCount < 0 ||
    blackPotionsCount < 0 ||
    turnNumber < 0 ||
    whiteScore > MAX_CANONICAL_SCORE ||
    blackScore > MAX_CANONICAL_SCORE ||
    actionsUsedCount > ACTIONS_PER_TURN ||
    manaMovesCount > MANA_MOVES_PER_TURN ||
    monsMovesCount > MONS_MOVES_PER_TURN ||
    turnNumber < 1
  ) {
    return undefined;
  }

  const boardStart = characterIndex;
  while (characterIndex < fen.length && fen.charCodeAt(characterIndex) !== 32) {
    characterIndex += 1;
  }
  const boardEnd = characterIndex;
  let variant: GameVariant = DEFAULT_GAME_VARIANT;
  if (characterIndex < fen.length) {
    characterIndex += 1;
    const variantId = readInteger(false);
    const parsedVariant = gameVariantFromId(variantId);
    if (parsedVariant === undefined || parsedVariant === DEFAULT_GAME_VARIANT) {
      return undefined;
    }
    variant = parsedVariant;
  }
  const board = parseBoardFenRange(fen, boardStart, boardEnd, variant);
  if (board === undefined) return undefined;
  return {
    board,
    whiteScore,
    blackScore,
    activeColor,
    actionsUsedCount,
    manaMovesCount,
    monsMovesCount,
    whitePotionsCount,
    blackPotionsCount,
    turnNumber,
  };
}

function emptyRunFen(count: number): string {
  return EMPTY_RUN_FENS[count] ?? "";
}
