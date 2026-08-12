#!/usr/bin/env node

import assert from "node:assert/strict";
import path from "node:path";
import { parseArgs } from "node:util";

import { checkPackage } from "./package-check/index.mjs";

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
await checkPackage(packageRoot, toolingRoot);
