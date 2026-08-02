#!/usr/bin/env node

import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { parseArgs } from "node:util";
import { runInNewContext } from "node:vm";

import { build } from "esbuild";

const toolingRoot = path.resolve(import.meta.dirname, "..");
const { positionals } = parseArgs({
  args: process.argv.slice(2),
  allowPositionals: true,
  options: {},
  strict: true,
});
assert(
  positionals.length <= 1,
  "usage: node scripts/check-package.mjs [package-directory]",
);

const packageRoot = path.resolve(positionals[0] ?? toolingRoot);
const packageName = "mons-rules";
const packageEntry = "./dist/mons-rules.js";
const typesEntry = "./dist/entrypoints/mons-rules.d.ts";
const publishedDistFiles = [
  "api/game.d.ts",
  "api/types.d.ts",
  "api/winner.d.ts",
  "entrypoints/mons-rules.d.ts",
  "mons-rules.js",
].sort();
const expectedRuntimeExports = [
  "AutomovePreference",
  "Color",
  "Consumable",
  "Game",
  "GameVariant",
  "Modifier",
  "MonKind",
  "resolveMatch",
].sort();

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, "utf8"));
}

function listFiles(directory, relativeTo = directory) {
  return fs
    .readdirSync(directory, { withFileTypes: true })
    .flatMap((entry) => {
      const entryPath = path.join(directory, entry.name);
      return entry.isDirectory()
        ? listFiles(entryPath, relativeTo)
        : [path.relative(relativeTo, entryPath).split(path.sep).join("/")];
    })
    .sort();
}

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    encoding: "utf8",
    ...options,
  });
  assert.equal(
    result.status,
    0,
    `${command} ${args.join(" ")} failed:\n${result.stdout}${result.stderr}`,
  );
  return result.stdout;
}

const manifest = readJson(path.join(packageRoot, "package.json"));
assert.equal(manifest.name, packageName, "package name changed");
assert.equal(manifest.type, "module", "package must be ESM");
assert.equal(manifest.main, packageEntry, "main entry changed");
assert.equal(manifest.types, typesEntry, "types entry changed");
assert.equal(manifest.sideEffects, false, "package must be side-effect free");
assert.equal(
  manifest.module,
  undefined,
  "duplicate module field must be absent",
);
assert.equal(
  manifest.browser,
  undefined,
  "duplicate browser field must be absent",
);
assert.deepEqual(
  manifest.exports,
  {
    ".": {
      types: typesEntry,
      import: packageEntry,
    },
  },
  "package must expose only its ESM and declaration entries",
);
for (const field of [
  "dependencies",
  "optionalDependencies",
  "peerDependencies",
  "peerDependenciesMeta",
  "bundleDependencies",
  "bundledDependencies",
]) {
  assert.equal(manifest[field], undefined, `${field} must not be published`);
}

const distRoot = path.join(packageRoot, "dist");
const distFiles = listFiles(distRoot);
assert(distFiles.includes("mons-rules.js"), "runtime bundle is missing");
assert(
  distFiles.includes("entrypoints/mons-rules.d.ts"),
  "generated declaration entry is missing",
);
for (const filePath of publishedDistFiles) {
  assert(
    distFiles.includes(filePath),
    `published artifact is missing: ${filePath}`,
  );
}
assert(
  distFiles.every(
    (filePath) =>
      filePath === "mons-rules.js" ||
      (filePath.endsWith(".d.ts") && !filePath.endsWith(".d.ts.map")),
  ),
  `dist contains an unexpected file: ${JSON.stringify(distFiles)}`,
);

const declarationText = publishedDistFiles
  .filter((filePath) => filePath.endsWith(".d.ts"))
  .map((filePath) => fs.readFileSync(path.join(distRoot, filePath), "utf8"))
  .join("\n");
for (const [label, pattern] of [
  ["model façade", /\bMonsGameModel\b/u],
  ["numeric model kind", /\b[A-Za-z]+ModelKind\b/u],
  ["manual lifecycle method", /\bfree\s*\(/u],
  ["Rust-style constructor", /\bstatic\s+new\s*\(/u],
  [
    "snake_case API",
    /\b(?:from_fen|process_input|active_color|turn_number|winner_color|can_takeback|verify_moves|smart_automove)\b/u,
  ],
]) {
  assert(!pattern.test(declarationText), `${label} leaked into declarations`);
}

const expectedFiles = [
  "LICENSE",
  "README.md",
  "package.json",
  ...publishedDistFiles.map((filePath) => `dist/${filePath}`),
].sort();
const temporaryRoot = fs.mkdtempSync(
  path.join(os.tmpdir(), "mons-rules-package-check-"),
);

try {
  const packDirectory = path.join(temporaryRoot, "pack");
  fs.mkdirSync(packDirectory);
  const reports = JSON.parse(
    run("npm", ["pack", "--json", "--pack-destination", packDirectory], {
      cwd: packageRoot,
    }),
  );
  assert.equal(reports.length, 1, "npm pack must produce exactly one archive");

  const report = reports[0];
  assert.equal(report.name, packageName, "packed package name changed");
  assert.equal(report.version, manifest.version, "packed version changed");
  assert.deepEqual(
    report.files.map(({ path: filePath }) => filePath).sort(),
    expectedFiles,
    "npm tar surface differs from the built package",
  );
  assert(report.size <= 250_000, `packed size ${report.size} exceeds 250000`);
  assert(
    report.unpackedSize <= 1_000_000,
    `unpacked size ${report.unpackedSize} exceeds 1000000`,
  );

  const archivePath = path.join(packDirectory, report.filename);
  const consumerDirectory = path.join(temporaryRoot, "consumer");
  fs.mkdirSync(consumerDirectory);
  fs.writeFileSync(
    path.join(consumerDirectory, "package.json"),
    `${JSON.stringify({ name: "package-smoke", private: true, type: "module" }, null, 2)}\n`,
  );
  run(
    "npm",
    [
      "install",
      "--ignore-scripts",
      "--no-audit",
      "--no-fund",
      "--package-lock=false",
      archivePath,
    ],
    { cwd: consumerDirectory },
  );

  const runtimeSource = `
    import assert from "node:assert/strict";
    import {
      AutomovePreference,
      Color,
      Game,
      GameVariant,
      resolveMatch,
    } from ${JSON.stringify(packageName)};
    import * as api from ${JSON.stringify(packageName)};

    assert.deepEqual(Object.keys(api).sort(), ${JSON.stringify(expectedRuntimeExports)});
    assert.equal(Game.name, "Game");
    assert.equal(resolveMatch.name, "resolveMatch");
    assert.equal(Color.White, "white");
    assert.equal(GameVariant.Classic, "Classic");

    const game = new Game({ variant: GameVariant.Classic });
    const openingFen = game.toFen();
    const output = game.playFen("l10,5;l9,4");
    assert.equal(output.kind, "complete");
    assert(output.events.length > 0, "representative move emitted no events");
    assert.notEqual(game.toFen(), openingFen, "representative move did not update the game");
    assert.equal(Game.fromFen(game.toFen())?.toFen(), game.toFen());
    assert.deepEqual(
      resolveMatch({
        white: { fen: openingFen, moves: [] },
        black: { fen: openingFen, moves: [] },
      }),
      { kind: "ongoing" },
    );

    const proGame = new Game({ variant: GameVariant.Classic });
    const proSourceFen = proGame.toFen();
    const proSuggestion = proGame.suggestMove(AutomovePreference.Pro);
    assert(proSuggestion !== undefined, "Pro produced no opening move");
    assert.equal(proGame.preview(proSuggestion.inputs).kind, "complete");
    assert.equal(proGame.toFen(), proSourceFen, "Pro mutated the source game");
  `;
  fs.writeFileSync(path.join(consumerDirectory, "runtime.mjs"), runtimeSource);
  run(process.execPath, ["runtime.mjs"], { cwd: consumerDirectory });

  fs.writeFileSync(
    path.join(consumerDirectory, "consumer.ts"),
    `
      import {
        AutomovePreference,
        Color,
        Game,
        GameVariant,
        resolveMatch,
        type Color as PlayerColor,
        type Input,
        type MatchSubmission,
        type PlayResult,
        type Position,
      } from ${JSON.stringify(packageName)};

      const game: Game = new Game({ variant: GameVariant.Classic });
      const color: PlayerColor = game.activeColor;
      const position: Position = { row: 10, column: 5 };
      const inputs: Input[] = [
        { kind: "position", position },
        { kind: "position", position: { row: 9, column: 4 } },
      ];
      const result: PlayResult = game.play(inputs);
      const submission: MatchSubmission = {
        white: { fen: game.toFen(), moves: [] },
        black: { fen: game.toFen(), moves: [] },
      };
      resolveMatch(submission);
      game.suggestMove(AutomovePreference.Pro);
      game.canTakeback(Color.White);
      void color;
      void result;
    `,
  );
  fs.writeFileSync(
    path.join(consumerDirectory, "tsconfig.json"),
    `${JSON.stringify(
      {
        compilerOptions: {
          lib: ["ES2020", "DOM"],
          module: "NodeNext",
          moduleResolution: "NodeNext",
          noEmit: true,
          strict: true,
          target: "ES2020",
        },
        files: ["consumer.ts"],
      },
      null,
      2,
    )}\n`,
  );
  run(
    process.execPath,
    [
      path.join(toolingRoot, "node_modules", "typescript", "bin", "tsc"),
      "-p",
      "tsconfig.json",
    ],
    { cwd: consumerDirectory },
  );

  fs.writeFileSync(
    path.join(consumerDirectory, "commonjs.cjs"),
    `require(${JSON.stringify(packageName)});\n`,
  );
  const commonJsResult = spawnSync(process.execPath, ["commonjs.cjs"], {
    cwd: consumerDirectory,
    encoding: "utf8",
  });
  assert.notEqual(
    commonJsResult.status,
    0,
    "CommonJS require unexpectedly loaded the ESM-only package",
  );
  assert.match(
    commonJsResult.stderr,
    /ERR_PACKAGE_PATH_NOT_EXPORTED|No "exports" main defined/u,
    "CommonJS require failed for an unexpected reason",
  );

  fs.writeFileSync(
    path.join(consumerDirectory, "browser.ts"),
    `
      import {
        AutomovePreference,
        Game,
        GameVariant,
      } from ${JSON.stringify(packageName)};
      const game = new Game({ variant: GameVariant.Classic });
      const openingFen = game.toFen();
      const suggestion = game.suggestMove(AutomovePreference.Pro);
      document.body.dataset["openingFen"] = openingFen;
      document.body.dataset["suggestion"] =
        suggestion?.inputFen ?? "";
      document.body.dataset["previewKind"] =
        suggestion === undefined ? "" : game.preview(suggestion.inputs).kind;
      document.body.dataset["sourceFenAfter"] = game.toFen();
    `,
  );
  fs.writeFileSync(
    path.join(consumerDirectory, "worker.ts"),
    `
      import {
        AutomovePreference,
        Game,
        GameVariant,
      } from ${JSON.stringify(packageName)};
      self.onmessage = () => {
        const game = new Game({ variant: GameVariant.Classic });
        const openingFen = game.toFen();
        const suggestion = game.suggestMove(AutomovePreference.Pro);
        postMessage({
          openingFen,
          previewKind:
            suggestion === undefined ? "" : game.preview(suggestion.inputs).kind,
          sourceFenAfter: game.toFen(),
          suggestion: suggestion?.inputFen ?? "",
        });
      };
    `,
  );
  const bundleForBrowser = async (entryPoint) => {
    const result = await build({
      absWorkingDir: consumerDirectory,
      entryPoints: [entryPoint],
      bundle: true,
      format: "iife",
      logLevel: "silent",
      platform: "browser",
      target: "es2020",
      write: false,
    });
    assert.equal(
      result.outputFiles.length,
      1,
      `${entryPoint} bundle is missing`,
    );
    return result.outputFiles[0].text;
  };
  const deterministicCrypto = {
    getRandomValues(values) {
      values.fill(0);
      return values;
    },
  };

  const browserDataset = {};
  runInNewContext(await bundleForBrowser("browser.ts"), {
    crypto: deterministicCrypto,
    document: { body: { dataset: browserDataset } },
    performance: { now: () => 0 },
  });
  assert.match(
    browserDataset.openingFen ?? "",
    /^0 0 w /u,
    "browser bundle did not initialize a game",
  );
  assert.equal(
    browserDataset.suggestion,
    "l10,5;l9,4",
    "browser Pro suggestion diverged from the v4 opening decision",
  );
  assert.equal(
    browserDataset.previewKind,
    "complete",
    "browser Pro suggestion was not applicable",
  );
  assert.equal(
    browserDataset.sourceFenAfter,
    browserDataset.openingFen,
    "browser Pro mutated the source game",
  );

  const workerScope = {};
  let workerMessage;
  runInNewContext(await bundleForBrowser("worker.ts"), {
    crypto: deterministicCrypto,
    performance: { now: () => 0 },
    postMessage(value) {
      workerMessage = value;
    },
    self: workerScope,
  });
  assert.equal(
    typeof workerScope.onmessage,
    "function",
    "worker bundle did not install its message handler",
  );
  workerScope.onmessage();
  assert.match(
    workerMessage?.openingFen ?? "",
    /^0 0 w /u,
    "worker bundle did not initialize a game",
  );
  assert.equal(
    workerMessage?.suggestion,
    "l10,5;l9,4",
    "worker Pro suggestion diverged from the v4 opening decision",
  );
  assert.equal(
    workerMessage?.previewKind,
    "complete",
    "worker Pro suggestion was not applicable",
  );
  assert.equal(
    workerMessage?.sourceFenAfter,
    workerMessage?.openingFen,
    "worker Pro mutated the source game",
  );

  console.log(
    `mons-rules ESM package passed: packed=${report.size} unpacked=${report.unpackedSize}`,
  );
} finally {
  fs.rmSync(temporaryRoot, { recursive: true, force: true });
}
