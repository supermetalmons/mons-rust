import { createHash } from "node:crypto";
import { createReadStream } from "node:fs";
import { parseArgs } from "node:util";
import { createGunzip } from "node:zlib";

import rulesManifest from "../../test-data/rules-regressions.manifest.json" with { type: "json" };
import { MonsGame } from "../engine/game.js";
import { outputFen, parseInputArrayFen } from "../engine/fen.js";
import { forEachByteLine } from "./byte-lines.js";
import {
  decodeUtf8Strict,
  errorMessage,
  fail,
  parseCanonicalRuleTestCase,
  type RuleTestCase,
} from "./regression-support.js";

const EXPECTED_CASE_COUNT = rulesManifest.artifact.caseCount;
const EXPECTED_COMPRESSED_BYTES = rulesManifest.artifact.compressedBytes;
const EXPECTED_COMPRESSED_SHA256 = rulesManifest.artifact.compressedSha256;
const EXPECTED_UNCOMPRESSED_BYTES = rulesManifest.artifact.uncompressedBytes;
const EXPECTED_UNCOMPRESSED_SHA256 = rulesManifest.artifact.uncompressedSha256;
const PROGRESS_INTERVAL = 100_000;

function replayCase(line: number, testCase: RuleTestCase): void {
  const game = MonsGame.fromFen(testCase.fenBefore, false);
  if (game === undefined) {
    fail(
      `fixture line ${line} has an invalid fenBefore:\n` +
        `fenBefore: ${testCase.fenBefore}\ninputFen:  ${testCase.inputFen}`,
    );
  }

  let actualOutputFen: string;
  try {
    const inputs = parseInputArrayFen(testCase.inputFen);
    if (inputs === undefined) {
      fail(`fixture line ${line} has invalid inputFen: ${testCase.inputFen}`);
    }
    const output = game.processInput(inputs, false, false);
    actualOutputFen = outputFen(output);
  } catch (error) {
    fail(
      `fixture line ${line} engine error: ${errorMessage(error)}\n` +
        `fenBefore: ${testCase.fenBefore}\ninputFen:  ${testCase.inputFen}`,
    );
  }
  const actualFenAfter = game.fen();

  if (
    actualOutputFen !== testCase.outputFen ||
    actualFenAfter !== testCase.fenAfter
  ) {
    fail(
      `fixture line ${line} rules mismatch:\n` +
        `fenBefore:         ${testCase.fenBefore}\n` +
        `inputFen:          ${testCase.inputFen}\n` +
        `expected output:   ${testCase.outputFen}\n` +
        `actual output:     ${actualOutputFen}\n` +
        `expected fenAfter: ${testCase.fenAfter}\n` +
        `actual fenAfter:   ${actualFenAfter}`,
    );
  }
}

async function run(): Promise<void> {
  parseArgs({
    args: process.argv.slice(2),
    allowPositionals: false,
    options: {},
    strict: true,
  });
  const corpusPath = process.env["MONS_RULES_CORPUS_PATH"];
  if (corpusPath === undefined || corpusPath === "") {
    fail("MONS_RULES_CORPUS_PATH is not set");
  }

  let previousRaw: Buffer | undefined;
  let count = 0;
  const compressed = createReadStream(corpusPath);
  const compressedHash = createHash("sha256");
  let compressedBytes = 0;
  compressed.on("data", (chunk: string | Buffer) => {
    const bytes = typeof chunk === "string" ? Buffer.from(chunk) : chunk;
    compressedBytes += bytes.byteLength;
    compressedHash.update(bytes);
  });
  const gunzip = createGunzip();
  compressed.on("error", (error) => gunzip.destroy(error));
  compressed.pipe(gunzip);

  const summary = await forEachByteLine(gunzip, (rawBytes, line) => {
    count = line;
    if (line > EXPECTED_CASE_COUNT) {
      fail(`fixture count exceeds ${EXPECTED_CASE_COUNT} at line ${line}`);
    }
    if (rawBytes.length === 0) {
      fail(`fixture line ${line} is empty`);
    }

    if (previousRaw !== undefined) {
      const ordering = Buffer.compare(rawBytes, previousRaw);
      if (ordering < 0) {
        fail(
          `out-of-order transition at fixture line ${line}:\n` +
            `previous: ${previousRaw.toString("utf8")}\n` +
            `current:  ${rawBytes.toString("utf8")}`,
        );
      }
      if (ordering === 0) {
        fail(`duplicate transition at fixture line ${line}`);
      }
    }

    let raw: string;
    try {
      raw = decodeUtf8Strict(rawBytes);
    } catch (error) {
      fail(`fixture line ${line} is not valid UTF-8: ${errorMessage(error)}`);
    }
    let testCase: RuleTestCase;
    try {
      testCase = parseCanonicalRuleTestCase(raw);
    } catch (error) {
      fail(`invalid fixture JSON at line ${line}: ${errorMessage(error)}`);
    }
    replayCase(line, testCase);
    previousRaw = Buffer.from(rawBytes);

    if (line % PROGRESS_INTERVAL === 0) {
      console.error(
        `progress: ${line}/${EXPECTED_CASE_COUNT} canonical rules transitions passed`,
      );
    }
  });

  const actualCompressedSha256 = compressedHash.digest("hex");
  if (compressedBytes !== EXPECTED_COMPRESSED_BYTES) {
    fail(
      `compressed byte count mismatch: expected ${EXPECTED_COMPRESSED_BYTES}, got ${compressedBytes}`,
    );
  }
  if (actualCompressedSha256 !== EXPECTED_COMPRESSED_SHA256) {
    fail(
      `compressed SHA-256 mismatch: expected ${EXPECTED_COMPRESSED_SHA256}, got ${actualCompressedSha256}`,
    );
  }

  if (!summary.endsWithLf) {
    fail("rules corpus must end with an LF");
  }
  if (summary.containsCarriageReturn) {
    fail("rules corpus must use LF line endings without carriage returns");
  }
  if (summary.bytes !== EXPECTED_UNCOMPRESSED_BYTES) {
    fail(
      `uncompressed byte count mismatch: expected ${EXPECTED_UNCOMPRESSED_BYTES}, got ${summary.bytes}`,
    );
  }
  if (summary.sha256 !== EXPECTED_UNCOMPRESSED_SHA256) {
    fail(
      `uncompressed SHA-256 mismatch: expected ${EXPECTED_UNCOMPRESSED_SHA256}, got ${summary.sha256}`,
    );
  }
  if (
    count !== EXPECTED_CASE_COUNT ||
    summary.lineCount !== EXPECTED_CASE_COUNT
  ) {
    fail(
      `fixture count mismatch: expected ${EXPECTED_CASE_COUNT}, read ${summary.lineCount}`,
    );
  }
  console.log(`ok: ${EXPECTED_CASE_COUNT} canonical rules transitions passed`);
}

void run().catch((error: unknown) => {
  console.error(`error: ${errorMessage(error)}`);
  process.exitCode = 1;
});
