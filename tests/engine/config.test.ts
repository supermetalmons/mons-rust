import { describe, expect, it } from "vitest";

import {
  ALL_GAME_VARIANTS,
  GAME_VARIANT_IDS,
  GameVariant,
  gameVariantFromId,
  manaBaseLocations,
} from "../../src/engine/board/config.js";
import {
  COLOR_IDS,
  Color,
  MODIFIER_RANK,
  MON_KIND_IDS,
  Modifier,
  MonKind,
} from "../../src/engine/model/domain.js";
import { boardFen } from "../../src/engine/codec/game-board.js";
import { isValidLocation } from "../../src/engine/board/geometry.js";
import { MonsGame } from "../../src/engine/game/mons-game.js";

describe("game variant configuration", () => {
  it("keeps the twelve persisted wire identities stable", () => {
    expect(Object.isFrozen(GAME_VARIANT_IDS)).toBe(true);
    expect(Object.isFrozen(COLOR_IDS)).toBe(true);
    expect(Object.isFrozen(MON_KIND_IDS)).toBe(true);
    expect(Object.isFrozen(MODIFIER_RANK)).toBe(true);
    expect(COLOR_IDS[Color.Black]).toBe(1);
    expect(MON_KIND_IDS[MonKind.Mystic]).toBe(4);
    expect(MODIFIER_RANK[Modifier.SelectBomb]).toBe(1);

    expect(ALL_GAME_VARIANTS).toEqual([
      GameVariant.Classic,
      GameVariant.SwappedManaRows,
      GameVariant.OffsetArcManaRows,
      GameVariant.CenterSpokeManaRows,
      GameVariant.AlternatingManaRows,
      GameVariant.InnerWedgeManaRows,
      GameVariant.OuterWedgeManaRows,
      GameVariant.BentCenterManaRows,
      GameVariant.OuterEdgeManaRows,
      GameVariant.SplitFlankManaRows,
      GameVariant.ForwardBridgeManaRows,
      GameVariant.CornerChainManaRows,
    ]);

    for (const [expectedId, variant] of ALL_GAME_VARIANTS.entries()) {
      expect(GAME_VARIANT_IDS[variant]).toBe(expectedId);
      expect(gameVariantFromId(expectedId)).toBe(variant);
    }
  });

  it("defines valid, distinct mana layouts that round-trip exactly", () => {
    const boardCodes = new Set<string>();
    for (const variant of ALL_GAME_VARIANTS) {
      for (const color of [Color.White, Color.Black]) {
        const locations = manaBaseLocations(variant, color);
        expect(locations).toHaveLength(5);
        expect(Object.isFrozen(locations)).toBe(true);
        expect(locations.every(Object.isFrozen)).toBe(true);
        expect(new Set(locations.map(({ i, j }) => `${i},${j}`)).size).toBe(5);
        expect(locations.every(isValidLocation)).toBe(true);
      }

      const source = new MonsGame(false, variant);
      const code = boardFen(source.board);
      const parsed = MonsGame.fromFen(source.fen(), false);
      expect(parsed).toBeDefined();
      if (parsed === undefined) {
        throw new Error(`generated board FEN failed to parse for ${variant}`);
      }
      expect(boardFen(parsed.board)).toBe(code);
      boardCodes.add(code);
    }
    expect(boardCodes.size).toBe(ALL_GAME_VARIANTS.length);
  });
});
