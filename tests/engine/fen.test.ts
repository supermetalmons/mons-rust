import { describe, expect, it } from "vitest";

import { ALL_GAME_VARIANTS, GameVariant } from "../../src/engine/board/config.js";
import { Color, Modifier, MonKind } from "../../src/engine/model/domain.js";
import { monFen } from "../../src/engine/codec/domain-item.js";
import { gameFen, parseGameFen } from "../../src/engine/codec/game-board.js";
import {
  inputArrayFen,
  inputFen,
  parseInputArrayFen,
  parseInputFen,
} from "../../src/engine/codec/input.js";
import { eventArrayFen, outputFen } from "../../src/engine/codec/output-event.js";
import { MonsGame } from "../../src/engine/game/mons-game.js";

describe("strict FEN codecs", () => {
  it("accepts only complete items with gameplay-valid mon cooldowns", () => {
    const canonical = new MonsGame(false, GameVariant.Classic).fen();
    const cooldownTwo = canonical.replace("E0x", "E2x");
    expect(cooldownTwo).not.toBe(canonical);
    expect(parseGameFen(cooldownTwo)?.board.items).toContainEqual({
      kind: "mon",
      mon: { kind: MonKind.Demon, color: Color.White, cooldown: 2 },
    });

    for (const malformed of [
      canonical.replace("E0x", "E3x"),
      canonical.replace("D0x", "D0z"),
      canonical.replace("D0x", "D0"),
      canonical.replace("D0x", "D3x"),
    ]) {
      expect(malformed).not.toBe(canonical);
      expect(parseGameFen(malformed), malformed).toBeUndefined();
    }

    expect(() =>
      monFen({ kind: MonKind.Demon, color: Color.White, cooldown: 3 }),
    ).toThrow(RangeError);
  });

  it("accepts canonical game FEN and rejects alternate whitespace", () => {
    const canonical = new MonsGame(false, GameVariant.Classic).fen();
    expect(parseGameFen(canonical)).toBeDefined();

    for (const malformed of [
      ` ${canonical}`,
      `${canonical} `,
      canonical.replace(" ", "  "),
      canonical.replace(" ", "\t"),
      canonical.replace(" ", "\n"),
      canonical.replace(/^0/u, "-0"),
      `${canonical} 0`,
      `${canonical}💣`,
      canonical.replace("0 0 w 0 0 0", "0 0 w 2 0 0"),
      canonical.replace("0 0 w 0 0 0", "0 0 w 0 2 0"),
      canonical.replace("0 0 w 0 0 0", "0 0 w 0 0 6"),
      canonical.replace(" 1 ", " 0 "),
    ]) {
      expect(parseGameFen(malformed), malformed).toBeUndefined();
    }
  });

  it("round-trips canonical game FEN for every persisted variant", () => {
    for (const variant of ALL_GAME_VARIANTS) {
      const canonical = new MonsGame(false, variant).fen();
      const parsed = parseGameFen(canonical);

      expect(parsed, variant).toBeDefined();
      if (parsed !== undefined) {
        expect(gameFen(parsed)).toBe(canonical);
      }
    }
  });

  it("accepts safe counters and rejects unsafe numeric values", () => {
    const fields = new MonsGame(false, GameVariant.Classic).fen().split(" ");
    const withField = (index: number, value: string): string => {
      const changed = [...fields];
      changed[index] = value;
      return changed.join(" ");
    };

    expect(parseGameFen(withField(0, "7"))).toBeUndefined();
    for (const index of [6, 7, 8]) {
      expect(parseGameFen(withField(index, "1000001"))).toBeDefined();
      expect(parseGameFen(withField(index, "9007199254740991"))).toBeDefined();
      expect(parseGameFen(withField(index, "9007199254740992"))).toBeUndefined();
    }
    for (const index of [0, 1, 3, 4, 5, 6, 7, 8]) {
      expect(parseGameFen(withField(index, "00"))).toBeUndefined();
      expect(parseGameFen(withField(index, "01"))).toBeUndefined();
    }
  });

  it("round-trips the largest safe turn and rejects overflow atomically", () => {
    const fields = new MonsGame(false, GameVariant.Classic).fen().split(" ");
    fields[8] = String(Number.MAX_SAFE_INTEGER - 1);
    const game = MonsGame.fromFen(fields.join(" "), false);
    const whiteManaMove = parseInputArrayFen("l7,4;l6,4");
    expect(game).toBeDefined();
    expect(whiteManaMove).toBeDefined();
    if (game === undefined || whiteManaMove === undefined) return;

    const completed = game.processInput(whiteManaMove, false, false);
    expect(completed.kind).toBe("events");
    expect(game.turnNumber).toBe(Number.MAX_SAFE_INTEGER);
    expect(MonsGame.fromFen(game.fen(), false)?.fen()).toBe(game.fen());

    const beforeOverflow = game.fen();
    const blackManaMove = parseInputArrayFen("l3,4;l4,4");
    expect(blackManaMove).toBeDefined();
    if (blackManaMove === undefined) return;
    expect(game.processInput(blackManaMove.slice(0, 1), true, false)).toEqual({
      kind: "invalid-input",
    });
    const starts = game.processInput([], true, false);
    expect(starts.kind).toBe("locations-to-start-from");
    if (starts.kind === "locations-to-start-from") {
      expect(starts.locations).not.toContainEqual({ i: 3, j: 4 });
    }
    expect(game.processInput(blackManaMove, false, false)).toEqual({
      kind: "invalid-input",
    });
    expect(game.fen()).toBe(beforeOverflow);
  });

  it("round-trips every canonical coordinate input token", () => {
    for (let i = 0; i <= 10; i += 1) {
      for (let j = 0; j <= 10; j += 1) {
        const encoded = `l${i},${j}`;
        const parsed = parseInputFen(encoded);
        expect(parsed, encoded).toEqual({ kind: "location", location: { i, j } });
        if (parsed !== undefined) expect(inputFen(parsed)).toBe(encoded);
      }
    }
  });

  it("rejects noncanonical board run encodings", () => {
    const canonical = new MonsGame(false, GameVariant.Classic).fen();
    expect(parseGameFen(canonical.replace("n11", "n01n10"))).toBeUndefined();
  });

  it("requires exactly eleven complete board rows", () => {
    const canonical = new MonsGame(false).fen();
    const fields = canonical.split(" ");
    const boardCode = fields[9];
    expect(boardCode).toBeDefined();
    if (boardCode === undefined) return;

    for (const malformed of [
      boardCode.slice(0, -1),
      `${boardCode}/n11`,
      boardCode.replace("/n11/", "//n11/"),
      boardCode.replace("n11", "n1"),
      boardCode.replace("y0x", "z0x"),
      `${boardCode}é`,
    ]) {
      const malformedFields = [...fields];
      malformedFields[9] = malformed;
      expect(parseGameFen(malformedFields.join(" ")), malformed).toBeUndefined();
    }
  });

  it("rejects noncanonical input tokens and arrays", () => {
    expect(parseInputFen("z")).toEqual({ kind: "takeback" });
    for (const malformed of [
      "l",
      "l,",
      "l0",
      "l0,",
      "l,0",
      "l00,0",
      "l01,0",
      "l0,00",
      "l0,01",
      "l11,0",
      "l0,11",
      "l-1,0",
      "l0,-1",
      "l1.0,0",
      "l0,0,",
      "l0 0",
      "l0,0x",
      "l💣,0",
      "m",
      "mc",
      "mpp",
      "zanything",
      "Z",
    ]) {
      expect(parseInputFen(malformed), malformed).toBeUndefined();
    }

    expect(parseInputArrayFen("")).toEqual([]);
    expect(parseInputArrayFen("z;mp;mb;l10,10")).toHaveLength(4);
    expect(parseInputArrayFen("l10,3;l9,2")).toHaveLength(2);
    for (const malformed of [42, true, Symbol("input"), null, undefined]) {
      expect(() => parseInputArrayFen(malformed)).toThrow(TypeError);
    }
    for (const malformed of [
      ";",
      "z;",
      ";z",
      "z;;z",
      "z z",
      "z\t",
      "z；mp",
      "\ufeffz",
      "\ud800",
      "💣",
      "l10,3;invalid;l9,2",
      "l10,3;l9,2;l10,4;l9,3;z",
    ]) {
      expect(parseInputArrayFen(malformed), malformed).toBeUndefined();
    }
  });

  it("round-trips complete input arrays and preserves event/output ordering", () => {
    const encodedInputs = "l10,3;mp;z";
    const parsedInputs = parseInputArrayFen(encodedInputs);
    expect(parsedInputs).toEqual([
      { kind: "location", location: { i: 10, j: 3 } },
      { kind: "modifier", modifier: Modifier.SelectPotion },
      { kind: "takeback" },
    ]);
    expect(inputArrayFen(parsedInputs ?? [])).toBe(encodedInputs);

    const events = [
      { kind: "takeback" },
      { kind: "next-turn", color: Color.White },
    ] as const;
    expect(eventArrayFen(events)).toBe("z nt w");
    expect(outputFen({ kind: "events", events })).toBe("ent w/z");
  });
});
