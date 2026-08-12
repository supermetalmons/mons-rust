#!/usr/bin/env node

import { createHash } from "node:crypto";
import { readFile, readdir } from "node:fs/promises";
import { posix, resolve } from "node:path";

const repositoryRoot = resolve(import.meta.dirname, "..");

const protectedFiles = Object.freeze({
  "test-data/rules-regressions.jsonl.gz":
    "02942e8107a3de160cfa1bf99dc6d1bcc070c94ba4aca650cb0c67530ee2e280",
  "test-data/automove-decisions/v1/README.md":
    "b163876a3763d6904d720ac0f0468eaae01554252ab40233b6965c59d071161a",
  "test-data/automove-decisions/v1/decisions.jsonl":
    "0ca2d5e5c619f24da155da5b70c20707f963a47235e74a0857eed128ab429f67",
  "test-data/automove-decisions/v1/internal-selector-observations.txt":
    "f6e52b3e4cf112e6e36f36776e7d587f491b46d169ff0df9fa1070651a04e324",
  "test-data/automove-decisions/v1/manifest.json":
    "70f9aae306523880b82874543517bc407b0897d16f344385da8ba4d31ccfaade",
  "test-data/automove-decisions/v4/README.md":
    "5fbd0d59891e3bd0823b60c30381fcb388c056f5e4351ce7f85c447b5ebc8f68",
  "test-data/automove-decisions/v4/decisions.jsonl":
    "96bc554ccd66d398ee79f031383a9dfadc96924df89c94a770714351fb22d02d",
  "test-data/automove-decisions/v4/manifest.json":
    "b9b9c74dfa1054265d9ae3865ac0cc7ba25ef633613f1898b51f237c5a6b717d",
  "test-data/automove-decisions/v5/README.md":
    "9002732736bd38b414130d92995cbc8e6f4b675d758ba033dc9ad4ef1ea05608",
  "test-data/automove-decisions/v5/decisions.jsonl":
    "3c791d84e230c967d2530fc1f1828020ded06cf44d8e5f6696bafd23e1672836",
  "test-data/automove-decisions/v5/manifest.json":
    "b84f009152dcce737e470dc6c7d2615277fb0859d3168aeffd08a26c7d17659d",
  "test-data/automove-decisions/v6/README.md":
    "e55750af545c7e36b8b325bda769095c2e3f11819237ad81f42496285ee1586a",
  "test-data/automove-decisions/v6/decisions.jsonl":
    "32799d75bebfe49494770af1657ff232208244081f7eafe9adaa606b1a251ee1",
  "test-data/automove-decisions/v6/manifest.json":
    "927b750a1e0cefdca99352aa956543770d6c9ef6a991256b9235971c4821a01b",
  "test-data/compatibility-edge-cases/v1/coordinate-cases.jsonl":
    "bc5f2b96f4755cd3c4e2d45f8b5b1753d56c27a7947d572c6956db3e85ddad66",
  "test-data/compatibility-edge-cases/v1/fen-cases.jsonl":
    "c4e2ad67d54bdb83e4e0f0310783e8f2043d82f9f93448ef9e885b1e54529304",
  "test-data/compatibility-edge-cases/v1/manifest.json":
    "c2926d59ed23d766b17d9a95cf169f5298d0db2c6b9ed19ab5ffb2a08b6d2b3f",
  "test-data/compatibility-edge-cases/v1/string-cases.jsonl":
    "70d1adc75a3d3b0ec014ff79cf08c95948c98117adc53f2304f497394de2ef3f",
  "test-data/complete-games/v1/README.md":
    "53e6d140aa928a7468f4271cd62fd2f569d45cce7955f27e6bf40bfe1cd61a39",
  "test-data/complete-games/v1/complete-games.jsonl":
    "5bc194f15516a9c275807415910c95b2e62ce63df9e575ac93e1dd93013197eb",
  "test-data/complete-games/v1/manifest.json":
    "7002d2fab95f311cc27c34ff588f5d1d94685ffd5b6212936375fe86f7527fe9",
});

const protectedDirectories = Object.freeze([
  "test-data/automove-decisions/v1",
  "test-data/automove-decisions/v4",
  "test-data/automove-decisions/v5",
  "test-data/automove-decisions/v6",
  "test-data/compatibility-edge-cases/v1",
  "test-data/complete-games/v1",
]);

async function listFiles(relativeDirectory) {
  const files = [];
  const directories = [relativeDirectory];
  while (directories.length > 0) {
    const directory = directories.pop();
    const entries = await readdir(resolve(repositoryRoot, directory), {
      withFileTypes: true,
    });
    for (const entry of entries) {
      const relativePath = posix.join(directory, entry.name);
      if (entry.isDirectory()) {
        directories.push(relativePath);
      } else {
        files.push(relativePath);
      }
    }
  }
  return files.sort();
}

const failures = [];

for (const directory of protectedDirectories) {
  const expected = Object.keys(protectedFiles)
    .filter((relativePath) => relativePath.startsWith(`${directory}/`))
    .sort();
  const actual = await listFiles(directory);
  if (
    actual.length !== expected.length ||
    actual.some((relativePath, index) => relativePath !== expected[index])
  ) {
    failures.push(
      `${directory}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`,
    );
  }
}

for (const [relativePath, expectedHash] of Object.entries(protectedFiles)) {
  const bytes = await readFile(resolve(repositoryRoot, relativePath));
  const actualHash = createHash("sha256").update(bytes).digest("hex");
  if (actualHash !== expectedHash) {
    failures.push(`${relativePath}: expected ${expectedHash}, got ${actualHash}`);
  }
}

if (failures.length > 0) {
  throw new Error(`Protected corpus integrity check failed:\n${failures.join("\n")}`);
}

console.log(
  `Protected corpus integrity check passed (${Object.keys(protectedFiles).length} files).`,
);
