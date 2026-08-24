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

const preview = game.preview(inputs);
if (preview.kind === "complete") {
  game.play(inputs);
}

const suggestion = game.suggestMove("pro");
if (suggestion !== undefined) {
  game.play(suggestion.inputs);
}

const restored = Game.fromFen(game.toFen());
```

`preview` resolves typed inputs without mutation. `play` and `playFen` apply
only complete legal moves. `toFen` serializes canonical game state and
`Game.fromFen` restores it.

Fast, Normal, and Pro automove search is cooperatively time-bounded. Exact
moves can vary with runtime timing, but returned suggestions are complete,
legal, and do not mutate the source game.

Published JavaScript targets ES2022 and automove uses the Web-standard
`performance.now()` clock. Node.js 22.13 through 22.x, or Node.js 24 or newer,
is required for Node consumers and repository tooling.

## Validate

```sh
npm run check
```

Maintainer boundaries and immutable-data rules are in
[docs/architecture.md on GitHub](https://github.com/supermetalmons/rules/blob/main/docs/architecture.md).

## Release

```sh
npm run bump
npm run publish -- --check-only
npm run publish
```
