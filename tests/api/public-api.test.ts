import { describe, expect, expectTypeOf, it } from "vitest";

import * as api from "../../src/entrypoints/mons-rules.js";
import {
  Color,
  Consumable,
  Game,
  GameVariant,
  Modifier,
  MonKind,
  type AvailableMoveCounts,
  type BoardItem,
  type GameEvent,
  type Input,
  type InputResolution,
  type Mana,
  type Mon,
  type Position,
  type Square,
} from "../../src/entrypoints/mons-rules.js";

describe("public API", () => {
  it("exports only the TypeScript-native runtime surface", () => {
    expect(Object.keys(api).sort()).toEqual([
      "Color",
      "Consumable",
      "Game",
      "GameVariant",
      "Modifier",
      "MonKind",
      "resolveMatch",
    ]);

    expect(Color).toEqual({ White: "white", Black: "black" });
    expect(MonKind).toEqual({
      Demon: "demon",
      Drainer: "drainer",
      Angel: "angel",
      Spirit: "spirit",
      Mystic: "mystic",
    });
    expect(Consumable).toEqual({
      Potion: "potion",
      Bomb: "bomb",
      BombOrPotion: "bomb-or-potion",
    });
    expect(Modifier).toEqual({
      SelectPotion: "select-potion",
      SelectBomb: "select-bomb",
    });
    expect(GameVariant.Classic).toBe("Classic");
    for (const values of [Color, Consumable, GameVariant, Modifier, MonKind]) {
      expect(Object.isFrozen(values)).toBe(true);
    }
    expect(Game).not.toHaveProperty("new");
    const game = new Game();
    expect(game).not.toHaveProperty("free");
    expect(game).not.toHaveProperty("moveUsage");
    expect(game).not.toHaveProperty("previewFen");
    expect(game).not.toHaveProperty("clearTracking");
  });

  it("defaults only undefined variants and rejects an explicit null", () => {
    expect(new Game().variant).toBe(GameVariant.Classic);
    expect(
      new Game({
        variant: undefined,
      } as unknown as ConstructorParameters<typeof Game>[0]).variant,
    ).toBe(GameVariant.Classic);
    expect(
      () =>
        new Game({
          variant: null,
        } as unknown as ConstructorParameters<typeof Game>[0]),
    ).toThrow(new TypeError("unsupported game variant: null"));
  });

  it("previews without mutation and atomically applies complete input", () => {
    const game = new Game({ variant: GameVariant.Classic });
    const initialFen = game.toFen();

    const starts = game.preview([]);
    expect(starts).toEqual({
      kind: "awaiting-start",
      inputFen: "",
      positions: [
        { row: 10, column: 3 },
        { row: 10, column: 4 },
        { row: 10, column: 5 },
        { row: 10, column: 6 },
        { row: 10, column: 7 },
      ],
    });
    expect(game.toFen()).toBe(initialFen);

    const partial = game.preview([
      { kind: "position", position: { row: 10, column: 3 } },
    ]);
    expect(partial.kind).toBe("awaiting-input");
    expect(game.toFen()).toBe(initialFen);

    expect(game.play([{ kind: "position", position: { row: 10, column: 3 } }])).toEqual(
      { kind: "invalid", inputFen: "l10,3" },
    );
    expect(game.toFen()).toBe(initialFen);

    const applied = game.playFen("l10,3;l9,2");
    expect(applied.kind).toBe("complete");
    if (applied.kind !== "complete") {
      throw new Error("expected a complete move");
    }
    expect(applied.inputFen).toBe("l10,3;l9,2");
    expect(applied.events[0]?.kind).toBe("mon-move");
    expect(game.toFen()).not.toBe(initialFen);
    expect(game.activeColor).toBe(Color.White);
    expect(game.turnNumber).toBe(1);
    expect(game.availableMoveCounts().monMoves).toBe(4);
  });

  it("leaves all state untouched when a complete prefix has an invalid suffix", () => {
    const game = new Game();
    const before = {
      fen: game.toFen(),
      takebackFens: game.takebackFens,
      trackingEntries: game.trackingEntries,
    };
    const result = game.play([
      { kind: "position", position: { row: 10, column: 3 } },
      { kind: "position", position: { row: 9, column: 2 } },
      { kind: "position", position: { row: 0, column: 0 } },
    ]);

    expect(result.kind).toBe("invalid");
    expect(game.toFen()).toBe(before.fen);
    expect(game.takebackFens).toEqual(before.takebackFens);
    expect(game.trackingEntries).toEqual(before.trackingEntries);
  });

  it("returns detached board and event values", () => {
    const game = new Game();
    const result = game.playFen("l10,3;l9,2");
    expect(result.kind).toBe("complete");
    if (result.kind !== "complete") return;

    const item = game.itemAt({ row: 9, column: 2 });
    expect(item?.kind).toBe("mon");
    if (item?.kind !== "mon") return;
    const expectedCooldown = item.mon.cooldown;
    (item as { mon: { cooldown: number } }).mon.cooldown = 99;
    expect(game.itemAt({ row: 9, column: 2 })).toMatchObject({
      kind: "mon",
      mon: { cooldown: expectedCooldown },
    });

    const first = result.events[0];
    if (first?.kind === "mon-move") {
      (first as { to: { row: number } }).to.row = 0;
    }
    expect(game.itemAt({ row: 9, column: 2 })?.kind).toBe("mon");

    const scores = game.scores as Record<string, number>;
    const potions = game.potions as Record<string, number>;
    const takebackFens = game.takebackFens as string[];
    const trackingEntries = game.trackingEntries as unknown as {
      fen: string;
    }[];
    scores["white"] = 99;
    potions["black"] = 99;
    takebackFens.push("not-a-fen");
    if (trackingEntries[0] !== undefined) {
      trackingEntries[0].fen = "not-a-fen";
    }
    expect(game.scores[Color.White]).not.toBe(99);
    expect(game.potions[Color.Black]).not.toBe(99);
    expect(game.takebackFens).not.toContain("not-a-fen");
    expect(game.trackingEntries).not.toContainEqual(
      expect.objectContaining({ fen: "not-a-fen" }),
    );
  });

  it("uses strict all-or-nothing input parsing and strict positions", () => {
    const game = new Game();
    const initialFen = game.toFen();

    for (const malformed of [
      "l10,3;garbage;l9,2",
      "zjunk",
      "l010,3;l9,2",
      "l10,3;",
      "l10,3;mc",
      "l10,3;💣",
    ]) {
      expect(game.playFen(malformed), malformed).toEqual({
        kind: "invalid",
        inputFen: malformed,
      });
      expect(game.toFen(), malformed).toBe(initialFen);
    }

    expect(() => game.itemAt({ row: -1, column: 14 })).toThrow(RangeError);
    expect(() =>
      game.preview([{ kind: "position", position: { row: 0.5, column: 1 } }]),
    ).toThrow(RangeError);

    const completeMoveWithTrailingInput = [
      { kind: "position", position: { row: 10, column: 3 } },
      { kind: "position", position: { row: 9, column: 2 } },
      { kind: "position", position: { row: 10, column: 4 } },
      { kind: "position", position: { row: 9, column: 3 } },
      { kind: "takeback" },
    ] as const satisfies readonly Input[];
    expect(game.preview(completeMoveWithTrailingInput)).toEqual({
      kind: "invalid",
      inputFen: "l10,3;l9,2;l10,4;l9,3;z",
    });
    expect(game.toFen()).toBe(initialFen);
  });

  it("round-trips canonical FEN and exposes semantic snapshots", () => {
    const game = new Game({ variant: GameVariant.CornerChainManaRows });
    const restored = Game.fromFen(game.toFen());
    expect(restored?.variant).toBe(GameVariant.CornerChainManaRows);
    expect(restored?.scores).toEqual({ white: 0, black: 0 });
    expect(restored?.potions).toEqual({ white: 0, black: 0 });
    expect(restored?.winner).toBeUndefined();
    expect(restored?.contentPositions()).toEqual(game.contentPositions());
    expect(Game.fromFen(` ${game.toFen()}`)).toBeUndefined();
  });

  it("suggests a predicted move without mutating its source", () => {
    const game = new Game();
    const before = {
      fen: game.toFen(),
      takebackFens: game.takebackFens,
      trackingEntries: game.trackingEntries,
    };
    const suggestion = game.suggestMove("fast");

    expect(suggestion).toBeDefined();
    if (suggestion === undefined) return;
    expect(suggestion.inputFen).not.toBe("");
    expect(game.toFen()).toBe(before.fen);
    expect(game.takebackFens).toEqual(before.takebackFens);
    expect(game.trackingEntries).toEqual(before.trackingEntries);

    const preview = game.preview(suggestion.inputs);
    expect(preview).toEqual({
      kind: "complete",
      inputFen: suggestion.inputFen,
      events: suggestion.events,
    });
    expect(game.toFen()).toBe(before.fen);

    expect(() =>
      game.suggestMove("random" as Parameters<Game["suggestMove"]>[0]),
    ).toThrow(new TypeError("unsupported automove preference: random"));
  });

  it("provides discriminated result and event types", () => {
    const consumeResolution = (resolution: InputResolution): string => {
      switch (resolution.kind) {
        case "invalid":
          return resolution.inputFen;
        case "awaiting-start":
          return String(resolution.positions.length);
        case "awaiting-input":
          return String(resolution.options.length);
        case "complete":
          return String(resolution.events.length);
      }
    };
    const consumeEvent = (event: GameEvent): string => event.kind;
    const consumeMatch = (resolution: ReturnType<typeof api.resolveMatch>): string => {
      switch (resolution.kind) {
        case "ongoing":
        case "invalid":
          return resolution.kind;
        case "winner":
          return resolution.winner;
      }
    };
    const position: Position = { row: 10, column: 3 };

    expectTypeOf<Input>().toEqualTypeOf<
      | { readonly kind: "takeback" }
      | { readonly kind: "position"; readonly position: Position }
      | { readonly kind: "modifier"; readonly modifier: Modifier }
    >();
    expectTypeOf<
      NonNullable<ReturnType<Game["suggestMove"]>>["inputs"]
    >().toEqualTypeOf<readonly Input[]>();
    expectTypeOf(new Game().availableMoveCounts()).toEqualTypeOf<AvailableMoveCounts>();
    expectTypeOf(new Game().itemAt(position)).toEqualTypeOf<BoardItem | undefined>();
    expectTypeOf(new Game().squareAt(position)).toEqualTypeOf<Square>();
    expectTypeOf<Mana>().toExtend<
      { readonly kind: "supermana" } | { readonly kind: "regular" }
    >();
    expectTypeOf<Mon>().toHaveProperty("kind");
    expectTypeOf<readonly [number, number]>().not.toExtend<Position>();

    expect(consumeResolution(new Game().preview([]))).toBe("5");
    expect(consumeMatch({ kind: "winner", winner: Color.White })).toBe(Color.White);
    expect(position).toEqual({ row: 10, column: 3 });
    expect(
      consumeEvent({
        kind: "next-turn",
        color: Color.Black,
      }),
    ).toBe("next-turn");
  });
});
