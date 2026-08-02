import { describe, expect, it, vi } from "vitest";

import { Board } from "../../src/engine/board.js";
import {
  ACTIONS_PER_TURN,
  MANA_MOVES_PER_TURN,
  MONS_MOVES_PER_TURN,
  TARGET_SCORE,
} from "../../src/engine/config.js";
import { Color } from "../../src/engine/domain.js";
import {
  canMoveManaForCounts,
  canMoveMonForCounts,
  canUseActionForCounts,
  shouldAdvanceTurn,
  shouldAdvanceTurnForCounts,
  shouldSuggestRegularManaStartsFromScalars,
  winnerForScores,
} from "../../src/engine/legality.js";
import { location } from "../../src/engine/geometry.js";

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
      expect(
        shouldAdvanceTurnForCounts(firstTurn, monsMoves, manaMoves, hasMana),
      ).toBe(expected);
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
      const findMana = vi
        .spyOn(board, "findMana")
        .mockReturnValue(hasMana ? location(0, 0) : undefined);

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
      expect(findMana).toHaveBeenCalledTimes(scans);
      if (scans !== 0) {
        expect(findMana).toHaveBeenCalledWith(Color.White);
      }
    },
  );
});
