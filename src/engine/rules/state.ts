import type { Board, MutableBoard } from "../board/storage.js";
import type { Color } from "../model/domain.js";

export type RulesState = {
  readonly board: Board;
  readonly whiteScore: number;
  readonly blackScore: number;
  readonly activeColor: Color;
  readonly actionsUsedCount: number;
  readonly manaMovesCount: number;
  readonly monsMovesCount: number;
  readonly whitePotionsCount: number;
  readonly blackPotionsCount: number;
  readonly turnNumber: number;
};

export type MutableRulesState = {
  board: MutableBoard;
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
