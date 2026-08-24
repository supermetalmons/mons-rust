import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { runInNewContext } from "node:vm";

import { build } from "esbuild";

import { expectedRuntimeExports, packageName } from "./config.mjs";
import { run } from "./support.mjs";

export async function assertPackageConsumer({
  expectedFiles,
  manifest,
  packageRoot,
  toolingRoot,
}) {
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
    const proSuggestion = proGame.suggestMove("pro");
    assert(proSuggestion !== undefined, "Pro produced no opening move");
    assert.equal(proGame.preview(proSuggestion.inputs).kind, "complete");
    assert.equal(proGame.toFen(), proSourceFen, "Pro mutated the source game");
    assert.throws(
      () => proGame.suggestMove("random"),
      /unsupported automove preference: random/,
    );
  `;
    fs.writeFileSync(path.join(consumerDirectory, "runtime.mjs"), runtimeSource);
    run(process.execPath, ["runtime.mjs"], { cwd: consumerDirectory });

    fs.writeFileSync(
      path.join(consumerDirectory, "consumer.ts"),
      `
      import {
        Color,
        Game,
        GameVariant,
        resolveMatch,
        type AvailableMoveCounts,
        type BoardItem,
        type Color as PlayerColor,
        type GameEvent,
        type Input,
        type InputResolution,
        type Mana,
        type Mon,
        type Position,
        type Square,
      } from ${JSON.stringify(packageName)};

      const game: Game = new Game({ variant: GameVariant.Classic });
      const color: PlayerColor = game.activeColor;
      const position: Position = { row: 10, column: 5 };
      const inputs: Input[] = [
        { kind: "position", position },
        { kind: "position", position: { row: 9, column: 4 } },
      ];
      const result = game.play(inputs);
      const resolution: InputResolution = game.preview([]);
      const counts: AvailableMoveCounts = game.availableMoveCounts();
      const item: BoardItem | undefined = game.itemAt(position);
      const square: Square = game.squareAt(position);
      const mana: Mana = { kind: "supermana" };
      const mon: Mon = {
        kind: "demon",
        color: Color.White,
        cooldown: 0,
      };
      const event: GameEvent | undefined =
        result.kind === "complete" ? result.events[0] : undefined;
      const submission = {
        white: { fen: game.toFen(), moves: [] },
        black: { fen: game.toFen(), moves: [] },
      };
      resolveMatch(submission);
      game.suggestMove("pro");
      game.canTakeback(Color.White);
      void color;
      void result;
      void resolution;
      void counts;
      void item;
      void square;
      void mana;
      void mon;
      void event;
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
        Game,
        GameVariant,
      } from ${JSON.stringify(packageName)};
      const game = new Game({ variant: GameVariant.Classic });
      const openingFen = game.toFen();
      const suggestion = game.suggestMove("pro");
      const packedSuggestion = game.suggestMove("fast");
      document.body.dataset["openingFen"] = openingFen;
      document.body.dataset["suggestion"] =
        suggestion?.inputFen ?? "";
      document.body.dataset["previewKind"] =
        suggestion === undefined ? "" : game.preview(suggestion.inputs).kind;
      document.body.dataset["packedSuggestion"] =
        packedSuggestion?.inputFen ?? "";
      document.body.dataset["packedPreviewKind"] =
        packedSuggestion === undefined
          ? ""
          : game.preview(packedSuggestion.inputs).kind;
      document.body.dataset["sourceFenAfter"] = game.toFen();
    `,
    );
    fs.writeFileSync(
      path.join(consumerDirectory, "worker.ts"),
      `
      import {
        Game,
        GameVariant,
      } from ${JSON.stringify(packageName)};
      self.onmessage = () => {
        const game = new Game({ variant: GameVariant.Classic });
        const openingFen = game.toFen();
        const suggestion = game.suggestMove("pro");
        const packedSuggestion = game.suggestMove("normal");
        postMessage({
          openingFen,
          packedPreviewKind:
            packedSuggestion === undefined
              ? ""
              : game.preview(packedSuggestion.inputs).kind,
          packedSuggestion: packedSuggestion?.inputFen ?? "",
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
      assert.equal(result.outputFiles.length, 1, `${entryPoint} bundle is missing`);
      return result.outputFiles[0].text;
    };

    const browserDataset = {};
    runInNewContext(await bundleForBrowser("browser.ts"), {
      document: { body: { dataset: browserDataset } },
      performance: { now: () => 0 },
      WeakRef: undefined,
    });
    assert.match(
      browserDataset.openingFen ?? "",
      /^0 0 w /u,
      "browser bundle did not initialize a game",
    );
    assert.equal(
      browserDataset.suggestion,
      "l10,5;l9,4",
      "browser Pro suggestion diverged from the current opening decision",
    );
    assert.equal(
      browserDataset.previewKind,
      "complete",
      "browser Pro suggestion was not applicable",
    );
    assert.notEqual(
      browserDataset.packedSuggestion,
      "",
      "browser Fast produced no opening move without WeakRef",
    );
    assert.equal(
      browserDataset.packedPreviewKind,
      "complete",
      "browser Fast suggestion without WeakRef was not applicable",
    );
    assert.equal(
      browserDataset.sourceFenAfter,
      browserDataset.openingFen,
      "browser Pro mutated the source game",
    );

    const workerScope = {};
    let workerMessage;
    runInNewContext(await bundleForBrowser("worker.ts"), {
      performance: { now: () => 0 },
      postMessage(value) {
        workerMessage = value;
      },
      self: workerScope,
      WeakRef: undefined,
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
      "worker Pro suggestion diverged from the current opening decision",
    );
    assert.equal(
      workerMessage?.previewKind,
      "complete",
      "worker Pro suggestion was not applicable",
    );
    assert.notEqual(
      workerMessage?.packedSuggestion,
      "",
      "worker Normal produced no opening move without WeakRef",
    );
    assert.equal(
      workerMessage?.packedPreviewKind,
      "complete",
      "worker Normal suggestion without WeakRef was not applicable",
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
}
