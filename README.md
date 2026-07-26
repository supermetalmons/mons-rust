# Mons rules engine

`mons-rules` is the dependency-free TypeScript rules engine for Super Metal
Mons. The package is an ES module targeting ES2020 and runs in browsers, Web
Workers, Node.js, and Firebase Cloud Functions.

Version 0.3 introduces an intentionally smaller, idiomatic TypeScript API.
Existing 0.2 users should follow [MIGRATION.md](./MIGRATION.md).

## Install

```sh
npm install mons-rules
```

`mons-rules` is ESM-only:

```ts
import { AutomovePreference, Game, GameVariant, type Input } from "mons-rules";

const game = new Game({ variant: GameVariant.Classic });

const inputs: Input[] = [
  { kind: "position", position: { row: 10, column: 5 } },
  { kind: "position", position: { row: 9, column: 4 } },
];
const result = game.play(inputs);

if (result.kind === "complete") {
  console.log(result.events);
}

const suggestion = game.suggestMove(AutomovePreference.Normal);
console.log(suggestion?.inputFen);
```

FEN helpers are available when a wire-format boundary is more convenient:

```ts
const game = new Game();
const result = game.playFen("l10,5;l9,4");
const restored = Game.fromFen(game.toFen());
```

Use `preview` or `previewFen` to inspect a partial input sequence without
mutating the game. Use `play` or `playFen` to apply a complete legal move.
Results, events, board items, positions, and squares are plain discriminated
objects rather than mutable façade classes.

Published JavaScript uses Web-standard `performance` and `crypto` globals.
Node.js 22.13 through 22.x, or Node.js 24 or newer, is required for Node
consumers and repository tooling.

## Validation

Install dependencies and run the complete local gate:

```sh
npm ci --engine-strict
npm run check
```

The check streams and replays 699,994 canonical rules transitions without
unpacking the compressed corpus. It also validates the public API,
deterministic automove decisions, and 1,527 complete real-player games
containing 25,185 turns and 169,480 inputs across all 12 variants.

Run `node ./scripts/check-complete-games.mjs` to validate the immutable public
corpus without replaying it. Run `npm run test:complete-games` for the full
engine replay.

## Release

Run `npm run bump` to increment the patch version in `package.json` and
`package-lock.json`, then commit the release change. Run
`npm run publish -- --check-only` to validate the unpublished version and
perform an npm dry run. Run `npm run publish` from a clean worktree to publish
that version to `latest`.

A real publish uses the transient `mons-npm-publish-lock` tag on `origin`, or
the shared remote named by `MONS_PUBLISH_LOCK_REMOTE`, to serialize releases
across hosts. The script prints lease-protected recovery instructions if
cleanup fails.
