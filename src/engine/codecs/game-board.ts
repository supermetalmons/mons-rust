import { Board } from "../board.js";
import {
  ACTIONS_PER_TURN,
  DEFAULT_GAME_VARIANT,
  GameVariant,
  gameVariantId,
  MANA_MOVES_PER_TURN,
  MONS_MOVES_PER_TURN,
  parseGameVariant,
  TARGET_SCORE,
} from "../config.js";
import { Color, type Item } from "../domain.js";
import { BOARD_CELLS, BOARD_SIZE } from "../geometry.js";
import {
  isAscii,
  parseNonnegativeInteger,
  splitGameFenFields,
} from "./common.js";
import {
  colorFen,
  itemFen,
  parseColorFen,
  parseItemFen,
} from "./domain-item.js";

const MAX_CANONICAL_SCORE = TARGET_SCORE + 1;

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
  const lines: string[] = [];
  for (let i = 0; i < BOARD_SIZE; i += 1) {
    let line = "";
    let emptySpaceCount = 0;
    for (let j = 0; j < BOARD_SIZE; j += 1) {
      const item = board.get({ i, j });
      if (item === undefined) {
        emptySpaceCount += 1;
        continue;
      }
      if (emptySpaceCount > 0) {
        line += emptyRunFen(emptySpaceCount);
        emptySpaceCount = 0;
      }
      line += itemFen(item);
    }
    if (emptySpaceCount > 0) {
      line += emptyRunFen(emptySpaceCount);
    }
    lines.push(line);
  }
  return lines.join("/");
}

export function parseBoardFen(
  fen: string,
  variant: GameVariant,
): Board | undefined {
  if (!isAscii(fen)) return undefined;
  const lines = fen.split("/");
  if (lines.length !== BOARD_SIZE) {
    return undefined;
  }

  const items: (Item | undefined)[] = Array.from(
    { length: BOARD_CELLS },
    () => undefined,
  );
  for (const [i, line] of lines.entries()) {
    let characterIndex = 0;
    let j = 0;
    let previousWasEmptyRun = false;
    while (characterIndex < line.length) {
      if (line[characterIndex] === "n") {
        if (previousWasEmptyRun) return undefined;
        const countCode = line.slice(characterIndex + 1, characterIndex + 3);
        if (!/^\d{2}$/u.test(countCode)) return undefined;
        const count = Number(countCode);
        if (count < 1 || j + count > BOARD_SIZE) return undefined;
        j += count;
        characterIndex += 3;
        previousWasEmptyRun = true;
        continue;
      }

      const itemCode = line.slice(characterIndex, characterIndex + 3);
      if (itemCode.length !== 3 || j >= BOARD_SIZE) return undefined;
      const item = parseItemFen(itemCode);
      if (item === undefined) return undefined;
      items[i * BOARD_SIZE + j] = item;
      j += 1;
      characterIndex += 3;
      previousWasEmptyRun = false;
    }
    if (j !== BOARD_SIZE) return undefined;
  }

  return Board.fromItems(items, variant);
}

export function gameFen(game: GameFenState): string {
  const fields = [
    game.whiteScore,
    game.blackScore,
    colorFen(game.activeColor),
    game.actionsUsedCount,
    game.manaMovesCount,
    game.monsMovesCount,
    game.whitePotionsCount,
    game.blackPotionsCount,
    game.turnNumber,
    boardFen(game.board),
  ].map(String);
  const variant = game.board.variant;
  if (variant !== DEFAULT_GAME_VARIANT) {
    fields.push(String(gameVariantId(variant)));
  }
  return fields.join(" ");
}

export function parseGameFen(fen: string): GameFenState | undefined {
  const fields = splitGameFenFields(fen);
  if (fields === undefined) return undefined;
  if (fields.length !== 10 && fields.length !== 11) {
    return undefined;
  }

  const variant =
    fields.length === 10
      ? DEFAULT_GAME_VARIANT
      : parseGameVariant(fields[10] ?? "");
  if (variant === undefined) {
    return undefined;
  }
  if (fields.length === 11 && variant === DEFAULT_GAME_VARIANT) {
    return undefined;
  }

  const boardCode = fields[9];
  const colorCode = fields[2];
  if (boardCode === undefined || colorCode === undefined) {
    return undefined;
  }
  const board = parseBoardFen(boardCode, variant);
  const activeColor = parseColorFen(colorCode);
  if (board === undefined || activeColor === undefined) {
    return undefined;
  }

  const numbers = [
    fields[0],
    fields[1],
    fields[3],
    fields[4],
    fields[5],
    fields[6],
    fields[7],
    fields[8],
  ].map((field) => parseNonnegativeInteger(field ?? ""));
  if (numbers.some((number) => number === undefined)) {
    return undefined;
  }
  const [
    whiteScore,
    blackScore,
    actionsUsedCount,
    manaMovesCount,
    monsMovesCount,
    whitePotionsCount,
    blackPotionsCount,
    turnNumber,
  ] = numbers;
  if (
    whiteScore === undefined ||
    blackScore === undefined ||
    actionsUsedCount === undefined ||
    manaMovesCount === undefined ||
    monsMovesCount === undefined ||
    whitePotionsCount === undefined ||
    blackPotionsCount === undefined ||
    turnNumber === undefined ||
    whiteScore > MAX_CANONICAL_SCORE ||
    blackScore > MAX_CANONICAL_SCORE ||
    actionsUsedCount > ACTIONS_PER_TURN ||
    manaMovesCount > MANA_MOVES_PER_TURN ||
    monsMovesCount > MONS_MOVES_PER_TURN ||
    turnNumber < 1
  ) {
    return undefined;
  }
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
  return `n${String(count).padStart(2, "0")}`;
}
