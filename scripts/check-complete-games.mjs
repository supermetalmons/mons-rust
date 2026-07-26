#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { join } from "node:path";

const runner = join(import.meta.dirname, "run-complete-games.mjs");
const result = spawnSync(
  process.execPath,
  [runner, "--check-only", ...process.argv.slice(2)],
  { stdio: "inherit" },
);

if (result.error !== undefined) {
  throw result.error;
}
if (result.signal !== null) {
  console.error(`complete games check terminated by ${result.signal}`);
  process.exitCode = 1;
} else {
  process.exitCode = result.status ?? 1;
}
