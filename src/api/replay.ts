import { Color, type Input } from "../engine/domain.js";
import { parseInputArrayFen } from "../engine/fen.js";
import type { MonsGame } from "../engine/game.js";

type ReplayProgress = {
  readonly whiteMovesProcessed: number;
  readonly blackMovesProcessed: number;
};

type InterleavedReplayResult = ReplayProgress & {
  readonly status: "complete" | "invalid-move" | "missing-move" | "stopped";
};

type AfterReplayMove = (game: MonsGame, progress: ReplayProgress) => boolean;

function parseReplayMove(move: string): Input[] | undefined {
  if (move === "") return undefined;
  return parseInputArrayFen(move);
}

/** Replay color-partitioned move histories in the game's active-color order. */
export function replayInterleavedMoves(
  game: MonsGame,
  whiteMoves: readonly string[],
  blackMoves: readonly string[],
  afterMove?: AfterReplayMove,
): InterleavedReplayResult {
  let whiteMovesProcessed = 0;
  let blackMovesProcessed = 0;

  while (
    whiteMovesProcessed < whiteMoves.length ||
    blackMovesProcessed < blackMoves.length
  ) {
    const whiteTurn = game.activeColor === Color.White;
    const moves = whiteTurn ? whiteMoves : blackMoves;
    const moveIndex = whiteTurn ? whiteMovesProcessed : blackMovesProcessed;
    const move = moves[moveIndex];
    if (move === undefined) {
      return {
        status: "missing-move",
        whiteMovesProcessed,
        blackMovesProcessed,
      };
    }

    const inputs = parseReplayMove(move);
    if (inputs === undefined) {
      return {
        status: "invalid-move",
        whiteMovesProcessed,
        blackMovesProcessed,
      };
    }
    const output = game.processInput(inputs, false, false);
    if (output.kind !== "events" || output.events.length === 0) {
      return {
        status: "invalid-move",
        whiteMovesProcessed,
        blackMovesProcessed,
      };
    }
    if (whiteTurn) {
      whiteMovesProcessed += 1;
    } else {
      blackMovesProcessed += 1;
    }

    const progress = { whiteMovesProcessed, blackMovesProcessed };
    if (afterMove?.(game, progress) === false) {
      return { status: "stopped", ...progress };
    }
  }

  return {
    status: "complete",
    whiteMovesProcessed,
    blackMovesProcessed,
  };
}
