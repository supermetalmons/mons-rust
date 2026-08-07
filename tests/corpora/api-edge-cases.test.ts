import { readFileSync } from "node:fs";
import path from "node:path";

import { describe, expect, it } from "vitest";

import { Color, Game, MonKind } from "../../src/entrypoints/mons-rules.js";

type ArtifactManifest = {
  readonly path: string;
};

type EdgeManifest = {
  readonly constants: {
    readonly classicInitialFen: string;
  };
  readonly artifacts: readonly ArtifactManifest[];
};

type StoredEdgeRecord = {
  readonly id: string;
  readonly operation:
    | "MonsGameModel.from_fen"
    | "MonsGameModel.item/square/remove_item"
    | "MonsGameModel.process_input_fen";
  readonly inputFen?: string;
  readonly inputCodeUnits?: readonly number[];
};

const corpusDirectory = path.resolve("test-data/compatibility-edge-cases/v1");
const manifest = JSON.parse(
  readFileSync(path.join(corpusDirectory, "manifest.json"), "utf8"),
) as EdgeManifest;

function readArtifact(artifact: ArtifactManifest): readonly StoredEdgeRecord[] {
  return readFileSync(path.resolve(artifact.path), "utf8")
    .trimEnd()
    .split("\n")
    .map((line) => JSON.parse(line) as StoredEdgeRecord);
}

describe("archived public API edge-case corpus", () => {
  const records = manifest.artifacts.flatMap(readArtifact);
  const recordsById = new Map(records.map((record) => [record.id, record]));

  function requiredRecord(id: string): StoredEdgeRecord {
    const record = recordsById.get(id);
    if (record === undefined) throw new Error(`missing archived record ${id}`);
    return record;
  }

  it("covers canonical FEN, board values, and legal input semantics", () => {
    expect(requiredRecord("valid-occupied").operation).toBe(
      "MonsGameModel.item/square/remove_item",
    );
    const game = Game.fromFen(manifest.constants.classicInitialFen);
    expect(game).toBeDefined();
    if (game === undefined) return;

    expect(game.toFen()).toBe(manifest.constants.classicInitialFen);
    expect(game.itemAt({ row: 0, column: 3 })).toEqual({
      kind: "mon",
      mon: {
        kind: MonKind.Mystic,
        color: Color.Black,
        cooldown: 0,
      },
    });
    expect(game.squareAt({ row: 0, column: 3 })).toEqual({
      kind: "mon-base",
      monKind: MonKind.Mystic,
      color: Color.Black,
    });

    const before = game.toFen();
    const preview = game.preview([
      { kind: "position", position: { row: 10, column: 3 } },
      { kind: "position", position: { row: 9, column: 2 } },
    ]);
    expect(preview.kind).toBe("complete");
    expect(game.toFen()).toBe(before);
    expect(game.playFen("l10,3;l9,2").kind).toBe("complete");
    expect(game.toFen()).not.toBe(before);
  });

  it("strictly rejects representative permissive Rust FEN cases", () => {
    for (const id of [
      "wrong-row-count",
      "empty-first-row",
      "cross-row-11",
      "row0-run99-alias",
      "invalid-item-after-oob",
      "ascii-ascii-two-byte",
      "valid-pair-four-byte",
    ]) {
      const record = requiredRecord(id);
      expect(record.inputFen, id).toBeDefined();
      expect(Game.fromFen(record.inputFen ?? ""), id).toBeUndefined();
    }

    for (const codePoint of [9, 160, 8232]) {
      const fen = manifest.constants.classicInitialFen.replaceAll(
        " ",
        String.fromCodePoint(codePoint),
      );
      expect(Game.fromFen(fen), `U+${codePoint.toString(16)}`).toBeUndefined();
    }
  });

  it("rejects wrapped, fractional, and non-finite coordinates", () => {
    const game = new Game();
    for (const position of [
      { row: 0.9, column: 3.9 },
      { row: -1, column: 14 },
      { row: 0, column: 37 },
      { row: Number.NaN, column: Number.POSITIVE_INFINITY },
    ]) {
      expect(() => game.itemAt(position), JSON.stringify(position)).toThrow(
        RangeError,
      );
      expect(() => game.squareAt(position), JSON.stringify(position)).toThrow(
        RangeError,
      );
    }
  });

  it("rejects malformed input text without normalization or mutation", () => {
    for (const id of [
      "normalization-bom",
      "normalization-valid-pair",
      "normalization-lone-high",
      "normalization-bom-then-high",
    ]) {
      const codeUnits = requiredRecord(id).inputCodeUnits;
      if (codeUnits === undefined) {
        throw new Error(`${id} is missing archived input code units`);
      }
      const input = String.fromCharCode(...codeUnits);
      const game = new Game();
      const before = game.toFen();
      expect(game.playFen(input), id).toEqual({
        kind: "invalid",
        inputFen: input,
      });
      expect(game.toFen(), id).toBe(before);
    }

    for (const input of ["zjunk", "l10,3;garbage;l9,2", "l010,3;l9,2"]) {
      const game = new Game();
      const before = game.toFen();
      expect(game.playFen(input), input).toEqual({
        kind: "invalid",
        inputFen: input,
      });
      expect(game.toFen(), input).toBe(before);
    }
  });
});
