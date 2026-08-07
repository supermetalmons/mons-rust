import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

import { describe, expect, expectTypeOf, it } from "vitest";

import { replayInterleavedMoves } from "../../src/api/replay.js";
import type { InputAction } from "../../src/api/types.js";
import {
  GameVariant as EngineGameVariant,
  type GameVariant as EngineGameVariantValue,
} from "../../src/engine/config.js";
import {
  Color as EngineColor,
  Consumable as EngineConsumable,
  Modifier as EngineModifier,
  MonKind as EngineMonKind,
  type Color as EngineColorValue,
  type Consumable as EngineConsumableValue,
  type Mana as EngineMana,
  type Modifier as EngineModifierValue,
  type Mon as EngineMon,
  type MonKind as EngineMonKindValue,
  type NextInputKind,
  type Square as EngineSquare,
} from "../../src/engine/domain.js";
import { MonsGame } from "../../src/engine/game.js";
import {
  Color,
  Consumable,
  Game,
  GameVariant,
  Modifier,
  MonKind,
  resolveMatch,
  type Color as ColorValue,
  type Consumable as ConsumableValue,
  type GameVariant as GameVariantValue,
  type Mana,
  type Modifier as ModifierValue,
  type Mon,
  type MonKind as MonKindValue,
  type Square,
} from "../../src/entrypoints/mons-rules.js";

const WHITE_TURN = [
  "l10,5;l9,5",
  "l9,5;l8,5",
  "l8,5;l7,5",
  "l10,6;l9,6",
  "l9,6;l8,7",
] as const;

const BLACK_TURN = [
  "l0,5;l1,5",
  "l1,5;l2,5",
  "l2,5;l3,5",
  "l3,5;l4,5",
  "l4,5;l5,5",
  "l4,3;l3,2",
] as const;

type CompleteGame = {
  readonly gameVariant: string;
  readonly turns: readonly (readonly string[])[];
};

function firstCompleteGame(): CompleteGame {
  const corpusPath = fileURLToPath(
    new URL(
      "../../test-data/complete-games/v1/complete-games.jsonl",
      import.meta.url,
    ),
  );
  const firstLine = readFileSync(corpusPath, "utf8").split("\n", 1)[0];
  if (firstLine === undefined || firstLine.trim() === "") {
    throw new Error("complete-game corpus must contain at least one game");
  }
  return JSON.parse(firstLine) as CompleteGame;
}

function replayCompleteGame(completeGame: CompleteGame): {
  readonly game: Game;
  readonly moves: Record<ColorValue, string[]>;
} {
  const game = new Game({ variant: GameVariant.Classic });
  const moves: Record<ColorValue, string[]> = {
    [Color.White]: [],
    [Color.Black]: [],
  };
  for (const turn of completeGame.turns) {
    for (const move of turn) {
      moves[game.activeColor].push(move);
      expect(game.playFen(move).kind, move).toBe("complete");
    }
  }
  return { game, moves };
}

describe("interleaved move replay", () => {
  it("shares canonical public contracts with the engine", () => {
    expect(EngineColor).toBe(Color);
    expect(EngineConsumable).toBe(Consumable);
    expect(EngineModifier).toBe(Modifier);
    expect(EngineMonKind).toBe(MonKind);
    expect(EngineGameVariant).toBe(GameVariant);

    expectTypeOf<EngineColorValue>().toEqualTypeOf<ColorValue>();
    expectTypeOf<EngineConsumableValue>().toEqualTypeOf<ConsumableValue>();
    expectTypeOf<EngineModifierValue>().toEqualTypeOf<ModifierValue>();
    expectTypeOf<EngineMonKindValue>().toEqualTypeOf<MonKindValue>();
    expectTypeOf<EngineGameVariantValue>().toEqualTypeOf<GameVariantValue>();
    expectTypeOf<EngineMon>().toEqualTypeOf<Mon>();
    expectTypeOf<EngineMana>().toEqualTypeOf<Mana>();
    expectTypeOf<EngineSquare>().toEqualTypeOf<Square>();
    expectTypeOf<NextInputKind>().toEqualTypeOf<InputAction>();
  });

  it("consumes each color history in active-player order", () => {
    const game = new MonsGame(false, EngineGameVariant.Classic);
    const observedProgress: string[] = [];
    const result = replayInterleavedMoves(
      game,
      WHITE_TURN,
      BLACK_TURN,
      (_replayedGame, progress) => {
        observedProgress.push(
          `${progress.whiteMovesProcessed}:${progress.blackMovesProcessed}`,
        );
        return true;
      },
    );

    expect(result).toEqual({
      status: "complete",
      whiteMovesProcessed: WHITE_TURN.length,
      blackMovesProcessed: BLACK_TURN.length,
    });
    expect(observedProgress[observedProgress.length - 1]).toBe(
      `${WHITE_TURN.length}:${BLACK_TURN.length}`,
    );
    expect(game.activeColor).toBe(EngineColor.White);
  });

  it("reports strict replay failures without applying the invalid move", () => {
    for (const move of [
      "",
      "garbage",
      "l10,5",
      "l10,5;l0,0",
      "garbage;l10,5;l9,5",
      "zjunk",
      "l010,5;l9,5",
      `${WHITE_TURN[0]};l0,0;l0,1;l0,2`,
    ]) {
      const game = new MonsGame(false, EngineGameVariant.Classic);
      const initialFen = game.fen();
      let callbackCount = 0;
      const result = replayInterleavedMoves(game, [move], [], () => {
        callbackCount += 1;
        return true;
      });

      expect(result, move).toEqual({
        status: "invalid-move",
        whiteMovesProcessed: 0,
        blackMovesProcessed: 0,
      });
      expect(callbackCount, move).toBe(0);
      expect(game.fen(), move).toBe(initialFen);
    }
  });

  it("reports the consumed side when its history is exhausted", () => {
    const game = new MonsGame(false, EngineGameVariant.Classic);
    expect(
      replayInterleavedMoves(game, [WHITE_TURN[0]], [BLACK_TURN[0]]),
    ).toEqual({
      status: "missing-move",
      whiteMovesProcessed: 1,
      blackMovesProcessed: 0,
    });
  });
});

describe("Game history", () => {
  it("verifies typed histories and preserves metadata after failure", () => {
    const game = new Game();
    for (const move of WHITE_TURN) {
      expect(game.playFen(move).kind).toBe("complete");
    }
    for (const move of BLACK_TURN) {
      expect(game.playFen(move).kind).toBe("complete");
    }

    const beforeFen = game.toFen();
    expect(
      game.verifyHistory({
        white: WHITE_TURN,
        black: BLACK_TURN,
      }),
    ).toBe(true);
    const takebackBefore = game.takebackFens;
    const trackingBefore = game.trackingEntries;
    expect(takebackBefore.length).toBeGreaterThan(0);
    expect(trackingBefore.length).toBeGreaterThan(0);
    expect(
      trackingBefore.some(
        ({ eventsFen, fen }) => eventsFen !== "" && fen !== "",
      ),
    ).toBe(true);

    expect(
      game.verifyHistory({
        white: ["l10,4;l9,3"],
        black: BLACK_TURN,
      }),
    ).toBe(false);
    expect(game.toFen()).toBe(beforeFen);
    expect(game.historyVerified).toBe(true);
    expect(game.takebackFens).toEqual(takebackBefore);
    expect(game.trackingEntries).toEqual(trackingBefore);
  });

  it("supports empty history and derives a detached previous turn", () => {
    expect(
      new Game().verifyHistory({
        white: [],
        black: [],
      }),
    ).toBe(true);

    const game = new Game();
    expect(game.playFen("l10,3;l9,2").kind).toBe("complete");
    const firstFen = game.toFen();
    expect(game.playFen("l10,4;l9,3").kind).toBe("complete");
    const previous = game.previousTurn(game.takebackFens);
    expect(previous?.toFen()).toBe(firstFen);
    expect(previous?.canTakeback(previous.activeColor)).toBe(true);
    expect(previous?.takeback().kind).toBe("complete");
    expect(previous?.toFen()).not.toBe(firstFen);
    expect(game.trackingEntries.length).toBeGreaterThan(0);

    expect(game.previousTurn(["not-a-fen", game.toFen()])).toBeUndefined();
  });

  it("restores the first move from an empty pre-move takeback snapshot", () => {
    const game = new Game();
    const initialFen = game.toFen();
    const takebackFensBeforeMove = game.takebackFens;
    expect(takebackFensBeforeMove).toEqual([]);

    expect(game.playFen("l10,3;l9,2").kind).toBe("complete");
    const previous = game.previousTurn(takebackFensBeforeMove);

    expect(previous?.toFen()).toBe(initialFen);
    expect(previous?.takebackFens).toEqual([]);
  });

  it("applies legal takeback through the same atomic play path", () => {
    const game = new Game();
    expect(game.playFen("l10,3;l9,2").kind).toBe("complete");
    expect(game.playFen("l10,4;l9,3").kind).toBe("complete");
    const beforeTakeback = game.toFen();
    expect(game.canTakeback(Color.White)).toBe(true);
    expect(game.takeback()).toEqual({
      kind: "complete",
      inputFen: "z",
      events: [{ kind: "takeback" }],
    });
    expect(game.toFen()).not.toBe(beforeTakeback);
  });
});

describe("resolveMatch", () => {
  it("returns typed outcomes for ongoing, invalid, and one-invalid states", () => {
    const initial = new Game().toFen();
    expect(
      resolveMatch({
        white: { fen: initial, moves: [] },
        black: { fen: initial, moves: [] },
      }),
    ).toEqual({ kind: "ongoing" });
    expect(
      resolveMatch({
        white: { fen: "invalid", moves: [] },
        black: { fen: "also-invalid", moves: [] },
      }),
    ).toEqual({ kind: "invalid" });
    expect(
      resolveMatch({
        white: { fen: "invalid", moves: [] },
        black: { fen: initial, moves: [] },
      }),
    ).toEqual({ kind: "winner", winner: Color.Black });

    const otherVariant = new Game({
      variant: GameVariant.SwappedManaRows,
    }).toFen();
    expect(
      resolveMatch({
        white: { fen: initial, moves: [] },
        black: { fen: otherVariant, moves: [] },
      }),
    ).toEqual({ kind: "invalid" });
  });

  it("validates a terminal complete-game submission", () => {
    const completeGame = firstCompleteGame();
    expect(completeGame.gameVariant).toBe("Classic");

    const { game, moves } = replayCompleteGame(completeGame);
    const winningColor = game.winner;
    expect(winningColor).toBeDefined();
    if (winningColor === undefined) {
      throw new Error("complete game must have a winner");
    }

    const initialFen = new Game().toFen();
    const finalFen = game.toFen();
    const submittedFens =
      winningColor === Color.White
        ? { white: finalFen, black: initialFen }
        : { white: initialFen, black: finalFen };
    const expected = { kind: "winner", winner: winningColor } as const;

    expect(
      resolveMatch({
        white: { fen: submittedFens.white, moves: moves.white },
        black: { fen: submittedFens.black, moves: moves.black },
      }),
    ).toEqual(expected);
    expect(
      resolveMatch({
        white: { fen: submittedFens.black, moves: moves.white },
        black: { fen: submittedFens.white, moves: moves.black },
      }),
    ).toEqual({ kind: "invalid" });

    const invalidConsumed = {
      white: [...moves.white],
      black: [...moves.black],
    };
    invalidConsumed[winningColor].splice(-1, 0, "garbage");
    expect(
      resolveMatch({
        white: { fen: submittedFens.white, moves: invalidConsumed.white },
        black: { fen: submittedFens.black, moves: invalidConsumed.black },
      }),
    ).toEqual({ kind: "invalid" });

    const trailingLosing = {
      white: [...moves.white],
      black: [...moves.black],
    };
    const losingColor =
      winningColor === Color.White ? Color.Black : Color.White;
    trailingLosing[losingColor].push("garbage");
    expect(
      resolveMatch({
        white: { fen: submittedFens.white, moves: trailingLosing.white },
        black: { fen: submittedFens.black, moves: trailingLosing.black },
      }),
    ).toEqual(expected);
  });
});
