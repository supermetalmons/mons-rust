import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

import ts from "typescript";

import {
  expectedTypeExports,
  packageEntry,
  packageName,
  publishedDistFiles,
  typesEntry,
} from "./config.mjs";
import { listFiles, readJson } from "./support.mjs";

export function assertPackageContract(packageRoot) {
  const manifest = readJson(path.join(packageRoot, "package.json"));
  assert.equal(manifest.name, packageName, "package name changed");
  assert.equal(manifest.type, "module", "package must be ESM");
  assert.equal(manifest.main, packageEntry, "main entry changed");
  assert.equal(manifest.types, typesEntry, "types entry changed");
  assert.equal(manifest.sideEffects, false, "package must be side-effect free");
  assert.equal(manifest.module, undefined, "duplicate module field must be absent");
  assert.equal(manifest.browser, undefined, "duplicate browser field must be absent");
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
    assert(distFiles.includes(filePath), `published artifact is missing: ${filePath}`);
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
  const entryDeclarationText = fs.readFileSync(
    path.join(distRoot, "entrypoints/mons-rules.d.ts"),
    "utf8",
  );
  const entryDeclaration = ts.createSourceFile(
    "mons-rules.d.ts",
    entryDeclarationText,
    ts.ScriptTarget.Latest,
    true,
    ts.ScriptKind.TS,
  );
  const actualTypeExports = entryDeclaration.statements
    .flatMap((statement) => {
      assert(
        ts.isExportDeclaration(statement),
        "package entry declaration must contain only re-exports",
      );
      assert(
        statement.exportClause !== undefined &&
          ts.isNamedExports(statement.exportClause),
        "package entry declaration must use named exports",
      );
      return statement.exportClause.elements
        .filter((element) => statement.isTypeOnly || element.isTypeOnly)
        .map((element) => element.name.text);
    })
    .sort();
  assert.deepEqual(
    actualTypeExports,
    expectedTypeExports,
    "package type exports changed",
  );
  for (const [label, pattern] of [
    ["model façade", /\bMonsGameModel\b/u],
    ["numeric model kind", /\b[A-Za-z]+ModelKind\b/u],
    ["manual lifecycle method", /\bfree\s*\(/u],
    ["Rust-style constructor", /\bstatic\s+new\s*\(/u],
    ["removed automove preference", /\bAutomovePreference\b/u],
    ["removed move usage", /\bmoveUsage\b/u],
    ["removed FEN preview", /\bpreviewFen\b/u],
    ["removed tracking reset", /\bclearTracking\b/u],
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
  return { expectedFiles, manifest };
}
