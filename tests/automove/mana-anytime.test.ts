import { describe, expect, it } from "vitest";

import { moveToInputs, tryLoadPosition } from "../../src/automove/bridge.js";
import { MAX_MOVES, generateMoves } from "../../src/automove/moves.js";
import {
  FastPosition,
  MOVE_MANA,
  applyFastMove,
  moveType,
} from "../../src/automove/state.js";
import { TARGET_SCORE } from "../../src/engine/board/config.js";
import { location } from "../../src/engine/board/geometry.js";
import { inputArrayFen } from "../../src/engine/codec/input.js";
import {
  Color,
  MonKind,
  createMon,
  manaItem,
  monItem,
  regularMana,
} from "../../src/engine/model/domain.js";
import { gameWith } from "./test-helper.js";

function generated(position: FastPosition): readonly number[] {
  const buffer = new Int32Array(MAX_MOVES);
  const keys = new Int32Array(MAX_MOVES);
  const count = generateMoves(position, buffer, keys);
  return [...buffer.slice(0, count)];
}

describe("winning mana moves before the mandatory mon moves are spent", () => {
  it("offers the early scoring move only when it wins immediately", () => {
    const manaAt = location(1, 1);
    const poolAt = location(0, 0);
    const scoringInput = inputArrayFen([
      { kind: "location", location: manaAt },
      { kind: "location", location: poolAt },
    ]);

    for (const whiteScore of [TARGET_SCORE - 2, TARGET_SCORE - 1]) {
      const game = gameWith(
        [
          [location(5, 5), monItem(createMon(MonKind.Drainer, Color.White, 0))],
          [location(8, 8), monItem(createMon(MonKind.Drainer, Color.Black, 0))],
          [manaAt, manaItem(regularMana(Color.White))],
        ],
        { whiteScore, activeColor: Color.White },
      );
      const packed = new FastPosition();
      expect(tryLoadPosition(packed, game, 40)).toBe(true);

      const moves = generated(packed);
      const inputFens = moves.map((move) => inputArrayFen(moveToInputs(move)));
      expect(inputFens.includes(scoringInput)).toBe(whiteScore === TARGET_SCORE - 1);
      if (whiteScore !== TARGET_SCORE - 1) continue;

      const move = moves[inputFens.indexOf(scoringInput)] ?? 0;
      expect(moveType(move)).toBe(MOVE_MANA);
      const applied = new FastPosition();
      applied.copyFrom(packed);
      expect(applyFastMove(applied, move)).toBe(0);
      expect(applied.whiteScore).toBe(TARGET_SCORE);

      const clone = game.fork();
      expect(clone.processInput(moveToInputs(move), false, false).kind).toBe("events");
      expect(clone.winnerColor()).toBe(Color.White);
    }
  });
});
