import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

export const performanceStateBankAlgorithm = Object.freeze({
  id: "complete-games-v1-midpoint-balanced-v1",
  midpointTurnIndex: "floor(turns.length / 2)",
  tooShort: "midpointTurnIndex === 0 || midpointTurnIndex >= turns.length",
  terminal: "winner is defined after replaying turns before the midpoint",
  candidateSha256: "SHA-256 of the UTF-8 public FEN",
  variantSeed: "lowest candidate SHA-256 per variant, with source line as tie-break",
  colorFill: "variant seeds followed by candidates in SHA-256 and source-line order",
  outputOrder: "candidate SHA-256, then source line",
  statesPerColor: 32,
});

export type PerformanceState = {
  id: string;
  fen: string;
  variant: string;
  activeColor: "white" | "black";
  sourceLine: number;
  midpointTurnIndex: number;
  stateSha256: string;
};

export function writePerformanceStateBankFixture(
  directory: string,
  name = "states",
  bundleSha256 = "a".repeat(64),
): {
  manifest: string;
  manifestValue: Record<string, unknown>;
  stateBank: string;
  states: PerformanceState[];
} {
  const states = Array.from({ length: 64 }, (_, index) => {
    const activeColor = index < 32 ? "white" : "black";
    const fen = JSON.stringify({
      variant: "Classic",
      ply: activeColor === "white" ? 0 : 1,
      candidateColor: null,
      nonce: index,
    });
    const stateSha256 = sha256(fen);
    const sourceLine = index + 1;
    return {
      id: `midpoint-${stateSha256}-${String(sourceLine).padStart(10, "0")}`,
      fen,
      variant: "Classic",
      activeColor,
      sourceLine,
      midpointTurnIndex: 1,
      stateSha256,
    } satisfies PerformanceState;
  }).sort(
    (left, right) =>
      compareStrings(left.stateSha256, right.stateSha256) ||
      left.sourceLine - right.sourceLine,
  );
  const stateBankContent = `${states.map((state) => JSON.stringify(state)).join("\n")}\n`;
  const stateBankBytes = Buffer.from(stateBankContent);
  const stateBank = path.join(directory, `${name}.jsonl`);
  const manifest = path.join(directory, `${name}.manifest.json`);
  const manifestValue = {
    schemaVersion: 1,
    kind: "automove-performance-state-bank",
    algorithm: {
      ...performanceStateBankAlgorithm,
      sha256: sha256(JSON.stringify(performanceStateBankAlgorithm)),
    },
    source: {
      sha256: sha256(
        JSON.stringify({
          bundleSha256,
          corpusSha256:
            "5bc194f15516a9c275807415910c95b2e62ce63df9e575ac93e1dd93013197eb",
        }),
      ),
      corpus: {
        path: "test-data/complete-games/v1/complete-games.jsonl",
        bytes: 2_273_026,
        sha256: "5bc194f15516a9c275807415910c95b2e62ce63df9e575ac93e1dd93013197eb",
        records: 1_527,
      },
      bundle: { sha256: bundleSha256 },
    },
    candidates: { eligible: 1_527, excludedTerminal: 0, excludedTooShort: 0 },
    selection: {
      states: states.length,
      colors: { white: 32, black: 32 },
      variants: ["Classic"],
    },
    output: {
      format: "jsonl",
      encoding: "UTF-8",
      trailingNewline: true,
      bytes: stateBankBytes.byteLength,
      sha256: sha256(stateBankBytes),
    },
  };
  fs.writeFileSync(stateBank, stateBankBytes);
  fs.writeFileSync(manifest, `${JSON.stringify(manifestValue)}\n`);
  return { manifest, manifestValue, stateBank, states };
}

function sha256(value: NodeJS.ArrayBufferView | string): string {
  return createHash("sha256").update(value).digest("hex");
}

function compareStrings(left: string, right: string): number {
  return left < right ? -1 : left > right ? 1 : 0;
}
