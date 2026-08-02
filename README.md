# Mons rules engine

`mons-rules` is the dependency-free TypeScript rules engine for Super Metal
Mons. It is an ESM-only package for browsers, Web Workers, Node.js, and Firebase
Cloud Functions.

## Install

```sh
npm install mons-rules
```

## Use

```ts
import { AutomovePreference, Game, GameVariant, type Input } from "mons-rules";

const game = new Game({ variant: GameVariant.Classic });
const inputs: Input[] = [
  { kind: "position", position: { row: 10, column: 5 } },
  { kind: "position", position: { row: 9, column: 4 } },
];
const result = game.play(inputs);
const suggestion = game.suggestMove(AutomovePreference.Pro);
```

Use `preview` or `previewFen` to inspect inputs without mutating the game. Use
`play` or `playFen` to apply a complete legal move. Serialize games with
`toFen` and restore them with `Game.fromFen`.

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
