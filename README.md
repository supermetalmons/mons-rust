# Mons rules engine

Dependency-free TypeScript rules for Super Metal Mons. The ESM-only package
runs in browsers, Web Workers, Node.js, and Firebase Cloud Functions.

## Install

```sh
npm install mons-rules
```

## Use

```ts
import { Game, GameVariant, type Input } from "mons-rules";

const game = new Game({ variant: GameVariant.Classic });
const inputs: Input[] = [
  { kind: "position", position: { row: 10, column: 5 } },
  { kind: "position", position: { row: 9, column: 4 } },
];
const result = game.play(inputs);
const suggestion = game.suggestMove("pro");
```

Use `preview` to inspect typed inputs without mutation. Use `play` or `playFen`
to apply a complete legal move. Serialize with `toFen` and restore with
`Game.fromFen`.

Fast and Normal suggestions use cooperative time-bounded search. The exact move
can vary with runtime, JIT, cache, and garbage-collection timing. Suggestions
remain legal and do not mutate the source game. Environments without `WeakRef`
use the canonical Fast and Normal selectors instead of the packed search path.

Published JavaScript targets ES2020 and uses Web-standard `performance` and
`crypto` globals. Node.js 22.13 through 22.x, or Node.js 24 or newer, is
required for Node consumers and repository tooling.

## Validate

```sh
npm ci --engine-strict
npm run check
```

## Release

```sh
npm run bump
npm run publish -- --check-only
npm run publish
```
