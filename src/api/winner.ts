import { MonsGame } from "../engine/game/mons-game.js";
import { Color as EngineColor } from "../engine/model/domain.js";
import { replayInterleavedMoves } from "./replay.js";
import { Color, type MatchResolution, type MatchSubmission } from "./types.js";

/** Validate two independently submitted end states and move histories. */
export function resolveMatch(submission: MatchSubmission): MatchResolution {
  const whiteGame = submittedGame(submission.white.fen);
  const blackGame = submittedGame(submission.black.fen);

  if (whiteGame === undefined || blackGame === undefined) {
    if (whiteGame === undefined && blackGame === undefined) {
      return { kind: "invalid" };
    }
    return {
      kind: "winner",
      winner: whiteGame === undefined ? Color.Black : Color.White,
    };
  }
  if (whiteGame.variant() !== blackGame.variant()) {
    return { kind: "invalid" };
  }

  const normalizedWhiteFen = whiteGame.fen();
  const normalizedBlackFen = blackGame.fen();
  if (whiteGame.winnerColor() === undefined && blackGame.winnerColor() === undefined) {
    return { kind: "ongoing" };
  }

  const replay = new MonsGame(false, whiteGame.variant());
  let resolution: MatchResolution | undefined;
  replayInterleavedMoves(
    replay,
    submission.white.moves,
    submission.black.moves,
    (replayedGame, { whiteMovesProcessed, blackMovesProcessed }) => {
      const winner = replayedGame.winnerColor();
      if (winner === undefined) return true;

      const submittedEveryWinnerMove =
        winner === EngineColor.White
          ? whiteMovesProcessed === submission.white.moves.length
          : blackMovesProcessed === submission.black.moves.length;
      const submittedWinnerFen =
        winner === EngineColor.White ? normalizedWhiteFen : normalizedBlackFen;
      resolution =
        submittedEveryWinnerMove && submittedWinnerFen === replayedGame.fen()
          ? { kind: "winner", winner }
          : { kind: "invalid" };
      return false;
    },
  );
  return resolution ?? { kind: "invalid" };
}

function submittedGame(fen: string): MonsGame | undefined {
  const game = MonsGame.fromFen(fen, false);
  return game?.fen() === fen ? game : undefined;
}
