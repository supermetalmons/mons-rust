import { createReadStream } from "node:fs";
import path from "node:path";
import { parseArgs } from "node:util";

import completeGamesManifest from "../../test-data/complete-games/v1/manifest.json" with { type: "json" };
import {
  ALL_GAME_VARIANTS,
  type GameVariant as VariantName,
} from "../engine/board/config.js";
import { parseInputArrayFen } from "../engine/codec/input.js";
import { MonsGame } from "../engine/game/mons-game.js";
import { forEachByteLine } from "./byte-lines.js";
import {
  decodeUtf8Strict,
  errorMessage,
  fail,
  isRecord,
  terminalEventMembershipError,
  type TerminalEventKind,
} from "./regression-support.js";

const EXPECTED_BYTES = completeGamesManifest.artifact.bytes;
const EXPECTED_SHA256 = completeGamesManifest.artifact.sha256;
const EXPECTED_GAME_COUNT = completeGamesManifest.statistics.recordCount;
const EXPECTED_TURN_COUNT = completeGamesManifest.statistics.turnCount;
const EXPECTED_INPUT_COUNT = completeGamesManifest.statistics.inputCount;
const PROGRESS_INTERVAL = 250;
const LOCATION_TOKEN = /^l(?:[0-9]|10),(?:[0-9]|10)$/u;
const MODIFIER_TOKEN = /^m(?:p|b)$/u;

const EXPECTED_VARIANT_COUNTS: Readonly<Record<VariantName, number>> =
  completeGamesManifest.statistics.variantGameCounts;
const VARIANT_NAMES = new Set<string>(ALL_GAME_VARIANTS);

type CompleteGameRecord = {
  readonly gameVariant: VariantName;
  readonly turns: readonly (readonly string[])[];
};

function isVariantName(value: string): value is VariantName {
  return VARIANT_NAMES.has(value);
}

function isCanonicalInputFen(inputFen: string): boolean {
  if (inputFen === "z") {
    return true;
  }
  return inputFen
    .split(";")
    .every((token) => LOCATION_TOKEN.test(token) || MODIFIER_TOKEN.test(token));
}

function parseRecord(raw: string, line: number): CompleteGameRecord {
  let parsed: unknown;
  try {
    parsed = JSON.parse(raw) as unknown;
  } catch (error) {
    fail(`line ${line} is not valid JSON: ${errorMessage(error)}`);
  }
  if (!isRecord(parsed)) {
    fail(`line ${line} must be a JSON object`);
  }
  const keys = Object.keys(parsed);
  if (keys.length !== 2 || keys[0] !== "gameVariant" || keys[1] !== "turns") {
    fail(
      `line ${line} keys must be ["gameVariant","turns"], got ${JSON.stringify(keys)}`,
    );
  }
  if (JSON.stringify(parsed) !== raw) {
    fail(`line ${line} is not canonical compact JSON`);
  }

  const variantValue = parsed["gameVariant"];
  const turnsValue = parsed["turns"];
  if (typeof variantValue !== "string" || !isVariantName(variantValue)) {
    fail(`line ${line} has unknown gameVariant ${JSON.stringify(variantValue)}`);
  }
  if (!Array.isArray(turnsValue) || turnsValue.length === 0) {
    fail(`line ${line} turns must be a non-empty array`);
  }

  const turns: string[][] = [];
  for (const [turnIndex, turnValue] of turnsValue.entries()) {
    if (!Array.isArray(turnValue) || turnValue.length === 0) {
      fail(`line ${line} turn ${turnIndex + 1} must be a non-empty array`);
    }
    const turn: string[] = [];
    for (const [inputIndex, inputValue] of turnValue.entries()) {
      if (
        typeof inputValue !== "string" ||
        inputValue.length === 0 ||
        !isCanonicalInputFen(inputValue)
      ) {
        fail(
          `line ${line} turn ${turnIndex + 1} input ${inputIndex + 1} is not canonical input FEN: ${JSON.stringify(inputValue)}`,
        );
      }
      turn.push(inputValue);
    }
    turns.push(turn);
  }
  return { gameVariant: variantValue, turns };
}

function replayGame(record: CompleteGameRecord, line: number): void {
  const game = new MonsGame(false, record.gameVariant);

  for (const [turnIndex, turn] of record.turns.entries()) {
    const lastTurn = turnIndex === record.turns.length - 1;
    for (const [inputIndex, inputFen] of turn.entries()) {
      const lastInput = inputIndex === turn.length - 1;
      const before = game.fen();
      const parsedInputs = parseInputArrayFen(inputFen);
      if (parsedInputs === undefined) {
        fail(
          `line ${line} turn ${turnIndex + 1} input ${inputIndex + 1} did not parse completely: ${inputFen}`,
        );
      }

      let output;
      try {
        output = game.processInput(parsedInputs, false, false);
      } catch (error) {
        fail(
          `line ${line} turn ${turnIndex + 1} input ${inputIndex + 1} threw: ${errorMessage(error)}\n` +
            `fenBefore: ${before}\ninputFen: ${inputFen}`,
        );
      }
      if (output.kind !== "events" || output.events.length === 0) {
        fail(
          `line ${line} turn ${turnIndex + 1} input ${inputIndex + 1} is not a resolved legal input ` +
            `(output ${output.kind})\nfenBefore: ${before}\ninputFen: ${inputFen}`,
        );
      }

      const expectedTerminal: TerminalEventKind | undefined = lastInput
        ? lastTurn
          ? "game-over"
          : "next-turn"
        : undefined;
      const terminalError = terminalEventMembershipError(
        output.events,
        expectedTerminal,
      );
      if (terminalError !== undefined) {
        fail(
          `line ${line} turn ${turnIndex + 1} input ${inputIndex + 1}: ${terminalError}\n` +
            `fenBefore: ${before}\ninputFen: ${inputFen}\n` +
            `eventKinds: ${output.events.map((event) => event.kind).join(",")}`,
        );
      }
    }
  }
}

function parseOptions(argv: readonly string[]): {
  readonly checkOnly: boolean;
  readonly corpusRoot: string;
} {
  const { values } = parseArgs({
    args: argv,
    allowPositionals: false,
    options: {
      "check-only": { type: "boolean" },
      help: { type: "boolean", short: "h" },
      root: { type: "string" },
    },
    strict: true,
  });
  if (values.help === true) {
    console.log(
      "usage: node scripts/run-complete-games.mjs [--check-only] [--root <corpus-directory>]",
    );
    process.exit(0);
  }

  const defaultCorpusRoot = path.resolve(
    import.meta.dirname,
    "..",
    "..",
    path.dirname(completeGamesManifest.artifact.path),
  );
  return {
    checkOnly: values["check-only"] ?? false,
    corpusRoot:
      values.root === undefined ? defaultCorpusRoot : path.resolve(values.root),
  };
}

async function run(): Promise<void> {
  const { checkOnly, corpusRoot } = parseOptions(process.argv.slice(2));
  const corpusPath = path.join(
    corpusRoot,
    path.basename(completeGamesManifest.artifact.path),
  );
  const variantCounts = Object.fromEntries(
    ALL_GAME_VARIANTS.map((name) => [name, 0]),
  ) as Record<VariantName, number>;
  let gameCount = 0;
  let turnCount = 0;
  let inputCount = 0;

  const summary = await forEachByteLine(
    createReadStream(corpusPath),
    (rawBytes, line) => {
      if (rawBytes.length === 0) {
        fail(`line ${line} is empty`);
      }
      let raw: string;
      try {
        raw = decodeUtf8Strict(rawBytes);
      } catch (error) {
        fail(`line ${line} is not valid UTF-8: ${errorMessage(error)}`);
      }
      const record = parseRecord(raw, line);
      gameCount += 1;
      variantCounts[record.gameVariant] += 1;
      turnCount += record.turns.length;
      inputCount += record.turns.reduce((total, turn) => total + turn.length, 0);
      if (!checkOnly) {
        replayGame(record, line);
      }

      if (!checkOnly && gameCount % PROGRESS_INTERVAL === 0) {
        console.error(
          `progress: ${gameCount}/${EXPECTED_GAME_COUNT} complete games replayed`,
        );
      }
    },
  );

  if (!summary.endsWithLf || summary.containsCarriageReturn) {
    fail("complete games corpus must use LF and end with exactly one LF");
  }
  if (summary.bytes !== EXPECTED_BYTES) {
    fail(`corpus byte count: expected ${EXPECTED_BYTES}, got ${summary.bytes}`);
  }
  if (summary.sha256 !== EXPECTED_SHA256) {
    fail(`corpus SHA-256: expected ${EXPECTED_SHA256}, got ${summary.sha256}`);
  }
  if (gameCount !== EXPECTED_GAME_COUNT || summary.lineCount !== gameCount) {
    fail(`game count: expected ${EXPECTED_GAME_COUNT}, got ${gameCount}`);
  }
  if (turnCount !== EXPECTED_TURN_COUNT) {
    fail(`turn count: expected ${EXPECTED_TURN_COUNT}, got ${turnCount}`);
  }
  if (inputCount !== EXPECTED_INPUT_COUNT) {
    fail(`input count: expected ${EXPECTED_INPUT_COUNT}, got ${inputCount}`);
  }
  for (const variant of ALL_GAME_VARIANTS) {
    const actual = variantCounts[variant];
    const expected = EXPECTED_VARIANT_COUNTS[variant];
    if (actual !== expected) {
      fail(`${variant} game count: expected ${expected}, got ${actual}`);
    }
  }

  const action = checkOnly ? "corpus check" : "replay";
  console.log(
    `complete games ${action} passed: ${gameCount} games, ${turnCount} turns, ${inputCount} inputs across ${ALL_GAME_VARIANTS.length} variants`,
  );
}

void run().catch((error: unknown) => {
  console.error(`complete games check failed: ${errorMessage(error)}`);
  process.exitCode = 1;
});
