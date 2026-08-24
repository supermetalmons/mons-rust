import { describe, expect, it, vi } from "vitest";

import { Board } from "../../src/engine/board/storage.js";
import {
  ACTIONS_PER_TURN,
  MANA_MOVES_PER_TURN,
  MONS_MOVES_PER_TURN,
  TARGET_SCORE,
} from "../../src/engine/board/config.js";
import { Color } from "../../src/engine/model/domain.js";
import {
  canMoveManaForCounts,
  canMoveMonForCounts,
  canUseActionForCounts,
  shouldAdvanceTurn,
  shouldAdvanceTurnForCounts,
  shouldSuggestRegularManaStartsFromScalars,
  winnerForScores,
} from "../../src/engine/rules/legality.js";
import { location } from "../../src/engine/board/geometry.js";

describe("scalar legality policy", () => {
  it("selects the winner with canonical White precedence", () => {
    expect(winnerForScores(0, 0)).toBeUndefined();
    expect(winnerForScores(TARGET_SCORE, 0)).toBe(Color.White);
    expect(winnerForScores(0, TARGET_SCORE)).toBe(Color.Black);
    expect(winnerForScores(TARGET_SCORE, TARGET_SCORE)).toBe(Color.White);
  });

  it("applies first-turn and counter gates to move availability", () => {
    expect(canMoveMonForCounts(MONS_MOVES_PER_TURN - 1)).toBe(true);
    expect(canMoveMonForCounts(MONS_MOVES_PER_TURN)).toBe(false);

    expect(canMoveManaForCounts(true, 0)).toBe(false);
    expect(canMoveManaForCounts(false, MANA_MOVES_PER_TURN - 1)).toBe(true);
    expect(canMoveManaForCounts(false, MANA_MOVES_PER_TURN)).toBe(false);

    expect(canUseActionForCounts(true, 0, 1)).toBe(false);
    expect(canUseActionForCounts(false, ACTIONS_PER_TURN - 1, 0)).toBe(true);
    expect(canUseActionForCounts(false, ACTIONS_PER_TURN, 0)).toBe(false);
    expect(canUseActionForCounts(false, ACTIONS_PER_TURN, 1)).toBe(true);
  });

  it.each([
    {
      name: "first turn",
      firstTurn: true,
      monsMoves: 0,
      manaMoves: 0,
      actionsUsed: 0,
      potions: 0,
      hasMonStart: false,
      includePotionAction: true,
      expected: false,
    },
    {
      name: "mana already moved",
      firstTurn: false,
      monsMoves: 0,
      manaMoves: MANA_MOVES_PER_TURN,
      actionsUsed: 0,
      potions: 0,
      hasMonStart: false,
      includePotionAction: true,
      expected: false,
    },
    {
      name: "no mon start",
      firstTurn: false,
      monsMoves: 0,
      manaMoves: 0,
      actionsUsed: 0,
      potions: 0,
      hasMonStart: false,
      includePotionAction: false,
      expected: true,
    },
    {
      name: "mon move available",
      firstTurn: false,
      monsMoves: MONS_MOVES_PER_TURN - 1,
      manaMoves: 0,
      actionsUsed: ACTIONS_PER_TURN,
      potions: 0,
      hasMonStart: true,
      includePotionAction: true,
      expected: false,
    },
    {
      name: "unused action",
      firstTurn: false,
      monsMoves: MONS_MOVES_PER_TURN,
      manaMoves: 0,
      actionsUsed: ACTIONS_PER_TURN - 1,
      potions: 0,
      hasMonStart: true,
      includePotionAction: true,
      expected: false,
    },
    {
      name: "spent action without potion",
      firstTurn: false,
      monsMoves: MONS_MOVES_PER_TURN,
      manaMoves: 0,
      actionsUsed: ACTIONS_PER_TURN,
      potions: 0,
      hasMonStart: true,
      includePotionAction: false,
      expected: true,
    },
    {
      name: "spent action with excluded potion action",
      firstTurn: false,
      monsMoves: MONS_MOVES_PER_TURN,
      manaMoves: 0,
      actionsUsed: ACTIONS_PER_TURN,
      potions: 1,
      hasMonStart: true,
      includePotionAction: false,
      expected: false,
    },
    {
      name: "spent action with included potion action",
      firstTurn: false,
      monsMoves: MONS_MOVES_PER_TURN,
      manaMoves: 0,
      actionsUsed: ACTIONS_PER_TURN,
      potions: 1,
      hasMonStart: true,
      includePotionAction: true,
      expected: true,
    },
  ])(
    "decides regular mana starts for $name",
    ({
      firstTurn,
      monsMoves,
      manaMoves,
      actionsUsed,
      potions,
      hasMonStart,
      includePotionAction,
      expected,
    }) => {
      expect(
        shouldSuggestRegularManaStartsFromScalars(
          firstTurn,
          monsMoves,
          manaMoves,
          actionsUsed,
          potions,
          hasMonStart,
          includePotionAction,
        ),
      ).toBe(expected);
    },
  );

  it.each([
    {
      name: "first turn with mon move",
      firstTurn: true,
      monsMoves: MONS_MOVES_PER_TURN - 1,
      manaMoves: 0,
      hasMana: false,
      expected: false,
    },
    {
      name: "first turn exhausted",
      firstTurn: true,
      monsMoves: MONS_MOVES_PER_TURN,
      manaMoves: 0,
      hasMana: true,
      expected: true,
    },
    {
      name: "later turn mana exhausted",
      firstTurn: false,
      monsMoves: 0,
      manaMoves: MANA_MOVES_PER_TURN,
      hasMana: true,
      expected: true,
    },
    {
      name: "later turn mon available",
      firstTurn: false,
      monsMoves: MONS_MOVES_PER_TURN - 1,
      manaMoves: 0,
      hasMana: false,
      expected: false,
    },
    {
      name: "later turn free mana remains",
      firstTurn: false,
      monsMoves: MONS_MOVES_PER_TURN,
      manaMoves: 0,
      hasMana: true,
      expected: false,
    },
    {
      name: "later turn no move remains",
      firstTurn: false,
      monsMoves: MONS_MOVES_PER_TURN,
      manaMoves: 0,
      hasMana: false,
      expected: true,
    },
  ])(
    "decides turn advancement for $name",
    ({ firstTurn, monsMoves, manaMoves, hasMana, expected }) => {
      expect(shouldAdvanceTurnForCounts(firstTurn, monsMoves, manaMoves, hasMana)).toBe(
        expected,
      );
    },
  );

  it.each([
    {
      name: "first turn with a mon move remaining",
      turnNumber: 1,
      monsMoves: MONS_MOVES_PER_TURN - 1,
      manaMoves: 0,
      hasMana: false,
      expected: false,
      scans: 0,
    },
    {
      name: "first turn with mon moves exhausted",
      turnNumber: 1,
      monsMoves: MONS_MOVES_PER_TURN,
      manaMoves: 0,
      hasMana: true,
      expected: true,
      scans: 0,
    },
    {
      name: "later turn with mana moves exhausted",
      turnNumber: 2,
      monsMoves: 0,
      manaMoves: MANA_MOVES_PER_TURN,
      hasMana: true,
      expected: true,
      scans: 0,
    },
    {
      name: "later turn with a mon move remaining",
      turnNumber: 2,
      monsMoves: MONS_MOVES_PER_TURN - 1,
      manaMoves: 0,
      hasMana: false,
      expected: false,
      scans: 0,
    },
    {
      name: "later turn with only free mana remaining",
      turnNumber: 2,
      monsMoves: MONS_MOVES_PER_TURN,
      manaMoves: 0,
      hasMana: true,
      expected: false,
      scans: 1,
    },
    {
      name: "later turn with no move remaining",
      turnNumber: 2,
      monsMoves: MONS_MOVES_PER_TURN,
      manaMoves: 0,
      hasMana: false,
      expected: true,
      scans: 1,
    },
  ])(
    "scans for mana only when needed on $name",
    ({ turnNumber, monsMoves, manaMoves, hasMana, expected, scans }) => {
      const board = new Board();
      const freeManaLocations = vi
        .spyOn(board, "allFreeRegularManaLocations")
        .mockReturnValue(hasMana ? [location(0, 0)] : []);

      expect(
        shouldAdvanceTurn({
          board,
          whiteScore: 0,
          blackScore: 0,
          activeColor: Color.White,
          actionsUsedCount: 0,
          manaMovesCount: manaMoves,
          monsMovesCount: monsMoves,
          whitePotionsCount: 0,
          blackPotionsCount: 0,
          turnNumber,
        }),
      ).toBe(expected);
      expect(freeManaLocations).toHaveBeenCalledTimes(scans);
      if (scans !== 0) {
        expect(freeManaLocations).toHaveBeenCalledWith(Color.White);
      }
    },
  );
});

describe("blocked mandatory mana move", () => {
  const preStuckFen =
    "4 2 w 1 0 4 0 0 15 n05d1xn05/n11/n02xxmn08/n04a0xxxms0xn01Y0xn02/n04xxUxxMxxmn04/y0xn03E0xn01e0xn04/n11/n11/n04S0xn06/n09A0xn01/n10D0x 1";

  it("advances the turn when the last mon move leaves every free mana unmovable", async () => {
    const { MonsGame } = await import("../../src/engine/game/mons-game.js");
    const { Game } = await import("../../src/api/game.js");
    const { inputArrayFen } = await import("../../src/engine/codec/input.js");
    const { tryLoadPosition, moveToInputs } =
      await import("../../src/automove/bridge.js");
    const { FastPosition } = await import("../../src/automove/state.js");
    const { generateMoves, MAX_MOVES } = await import("../../src/automove/moves.js");
    const { i32 } = await import("../../src/automove/board.js");

    const engine = MonsGame.fromFen(preStuckFen, false);
    if (engine === undefined) throw new Error("pre-stuck FEN must load");
    const position = new FastPosition();
    expect(tryLoadPosition(position, engine, 40)).toBe(true);
    const buffer = new Int32Array(MAX_MOVES);
    const count = generateMoves(position, buffer);
    expect(count).toBeGreaterThan(0);

    let advanced = 0;
    let held = 0;
    for (let index = 0; index < count; index += 1) {
      const inputFen = inputArrayFen(moveToInputs(i32(buffer, index)));
      const game = Game.fromFen(preStuckFen);
      if (game === undefined) throw new Error("pre-stuck FEN must load");
      const result = game.playFen(inputFen);
      if (result.kind !== "complete") continue;
      if (game.activeColor === Color.Black) {
        advanced += 1;
        if (advanced === 1) {
          expect(game.suggestMove("fast")).toBeDefined();
        }
      } else {
        held += 1;
        if (held === 1) {
          expect(game.suggestMove("fast")).toBeDefined();
        }
      }
    }
    expect(advanced).toBeGreaterThan(0);
    expect(held).toBeGreaterThan(0);
    expect(advanced + held).toBe(count);
  }, 30000);
});
