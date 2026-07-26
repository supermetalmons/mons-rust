import { createHash, type BinaryLike } from "node:crypto";
import { readFileSync } from "node:fs";
import path from "node:path";

import { describe, expect, it } from "vitest";

import { Color, Game, MonKind } from "../../src/entrypoints/mons-rules.js";

type ArtifactManifest = {
  readonly path: string;
  readonly recordCount: number;
  readonly bytes: number;
  readonly sha256: string;
  readonly orderedIdsSha256: string;
  readonly firstId: string;
  readonly lastId: string;
};

type EdgeManifest = {
  readonly schemaVersion: number;
  readonly corpusVersion: string;
  readonly constants: {
    readonly classicInitialFen: string;
    readonly parserWhitespaceCodePoints: readonly number[];
    readonly explicitNonWhitespaceCodePoints: readonly number[];
  };
  readonly statistics: {
    readonly recordCount: number;
    readonly matchingCaseCount: number;
    readonly approvedExceptionCount: number;
    readonly categoryCounts: Readonly<Record<string, number>>;
  };
  readonly artifacts: readonly ArtifactManifest[];
  readonly aggregate: {
    readonly artifactBytes: number;
    readonly recordCount: number;
    readonly orderedIdsSha256: string;
  };
};

type StoredEdgeRecord = {
  readonly id: string;
  readonly category: string;
  readonly operation:
    | "MonsGameModel.from_fen"
    | "MonsGameModel.item/square/remove_item"
    | "MonsGameModel.process_input_fen";
  readonly expectedRustWhitespace?: boolean;
  readonly inputSpec?: {
    readonly replaceAsciiFieldSeparatorsWithCodePoint?: number;
  };
  readonly inputFen?: string;
  readonly inputCodeUnits?: readonly number[];
  readonly constructorArgs?: {
    readonly iExpression: string;
    readonly jExpression: string;
  };
  readonly legacy: Readonly<Record<string, unknown>>;
  readonly typescriptPolicy: {
    readonly kind: "approved-exception" | "match-legacy";
  };
};

const corpusDirectory = path.resolve("test-data/compatibility-edge-cases/v1");
const manifest = JSON.parse(
  readFileSync(path.join(corpusDirectory, "manifest.json"), "utf8"),
) as EdgeManifest;

function sha256(value: BinaryLike): string {
  return createHash("sha256").update(value).digest("hex");
}

function readArtifact(artifact: ArtifactManifest): readonly StoredEdgeRecord[] {
  const bytes = readFileSync(path.resolve(artifact.path));
  expect(bytes.byteLength).toBe(artifact.bytes);
  expect(sha256(bytes)).toBe(artifact.sha256);

  const text = bytes.toString("utf8");
  expect(text.endsWith("\n")).toBe(true);
  const lines = text.slice(0, -1).split("\n");
  expect(lines).toHaveLength(artifact.recordCount);
  const records = lines.map((line) => {
    const record = JSON.parse(line) as StoredEdgeRecord;
    expect(JSON.stringify(record)).toBe(line);
    return record;
  });
  const ids = records.map(({ id }) => id);
  expect(ids[0]).toBe(artifact.firstId);
  expect(ids.at(-1)).toBe(artifact.lastId);
  expect(sha256(`${ids.join("\n")}\n`)).toBe(artifact.orderedIdsSha256);
  return records;
}

function archivalCategory(category: string): string {
  switch (category) {
    case "rust-whitespace":
      return "parser-whitespace";
    case "wasm-string-normalization":
      return "string-normalization";
    default:
      return category;
  }
}

describe("archived public API edge-case corpus", () => {
  const records = manifest.artifacts.flatMap(readArtifact);
  const recordsById = new Map(records.map((record) => [record.id, record]));

  function requiredRecord(id: string): StoredEdgeRecord {
    const record = recordsById.get(id);
    if (record === undefined) throw new Error(`missing archived record ${id}`);
    return record;
  }

  it("pins every v1 payload byte, record ID, and aggregate", () => {
    expect(manifest.schemaVersion).toBe(1);
    expect(manifest.corpusVersion).toBe("api-edge-cases-v1");
    expect(records).toHaveLength(manifest.statistics.recordCount);
    expect(records).toHaveLength(manifest.aggregate.recordCount);
    expect(
      manifest.artifacts.reduce((sum, artifact) => sum + artifact.bytes, 0),
    ).toBe(manifest.aggregate.artifactBytes);

    const ids = records.map(({ id }) => id);
    expect(new Set(ids).size).toBe(ids.length);
    expect(sha256(`${ids.join("\n")}\n`)).toBe(
      manifest.aggregate.orderedIdsSha256,
    );

    const categoryCounts: Record<string, number> = {};
    for (const record of records) {
      const category = archivalCategory(record.category);
      categoryCounts[category] = (categoryCounts[category] ?? 0) + 1;
    }
    expect(categoryCounts).toEqual(manifest.statistics.categoryCounts);
    expect(
      records.filter(
        ({ typescriptPolicy }) => typescriptPolicy.kind === "match-legacy",
      ),
    ).toHaveLength(manifest.statistics.matchingCaseCount);
    expect(
      records.filter(
        ({ typescriptPolicy }) =>
          typescriptPolicy.kind === "approved-exception",
      ),
    ).toHaveLength(manifest.statistics.approvedExceptionCount);
  });

  it("retains the archived parser-whitespace classification", () => {
    const whitespaceRecords = records.filter(
      ({ category }) => category === "rust-whitespace",
    );
    const codePoints = (expected: boolean): readonly (number | undefined)[] =>
      whitespaceRecords
        .filter(({ expectedRustWhitespace }) =>
          expected ? expectedRustWhitespace : expectedRustWhitespace === false,
        )
        .map(
          ({ inputSpec }) =>
            inputSpec?.replaceAsciiFieldSeparatorsWithCodePoint,
        );

    expect(codePoints(true)).toEqual(
      manifest.constants.parserWhitespaceCodePoints,
    );
    expect(codePoints(false)).toEqual(
      manifest.constants.explicitNonWhitespaceCodePoints,
    );
  });

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
    const preview = game.previewFen("l10,3;l9,2");
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
    const game = new Game();
    const before = game.toFen();

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
      expect(game.previewFen(input), id).toEqual({
        kind: "invalid",
        inputFen: input,
      });
      expect(game.toFen(), id).toBe(before);
    }

    for (const input of ["zjunk", "l10,3;garbage;l9,2", "l010,3;l9,2"]) {
      expect(game.previewFen(input), input).toEqual({
        kind: "invalid",
        inputFen: input,
      });
      expect(game.toFen(), input).toBe(before);
    }
  });
});
