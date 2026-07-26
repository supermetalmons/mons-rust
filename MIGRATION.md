# Migrating from 0.2 to 0.3

Version 0.3 intentionally removes the Rust/Wasm compatibility layer. The new
API uses ordinary TypeScript classes, plain readonly objects, string
discriminants, camelCase names, and structured results. There is no CommonJS
entrypoint or compatibility shim.

## Import and construct a game

0.2:

```ts
import { GameVariant, Location, MonsGameModel } from "mons-rules";
const game = MonsGameModel.new(GameVariant.Classic);
```

0.3:

```ts
import {
  AutomovePreference,
  Game,
  GameVariant,
  Modifier,
  resolveMatch,
} from "mons-rules";
const game = new Game({ variant: GameVariant.Classic });
```

`GameVariant`, `Color`, `MonKind`, `Consumable`, and `Modifier` are now frozen
string-valued const objects with matching union types. Persist their string
values, not the old numeric ordinals.

CommonJS consumers must migrate to ESM:

Removed:

```js
const rules = require("mons-rules");
```

0.3:

```js
import * as rules from "mons-rules";
```

## Game method mapping

| 0.2                                       | 0.3                                                          |
| ----------------------------------------- | ------------------------------------------------------------ |
| `MonsGameModel.new(variant)`              | `new Game({ variant })`                                      |
| `MonsGameModel.from_fen(fen)`             | `Game.fromFen(fen)`                                          |
| `game.fen()`                              | `game.toFen()`                                               |
| `game.active_color()`                     | `game.activeColor`                                           |
| `game.turn_number()`                      | `game.turnNumber`                                            |
| `game.white_score()`                      | `game.scores[Color.White]`                                   |
| `game.black_score()`                      | `game.scores[Color.Black]`                                   |
| `game.winner_color()`                     | `game.winner`                                                |
| `game.is_moves_verified()`                | `game.historyVerified`                                       |
| `game.takeback_fens()`                    | `game.takebackFens`                                          |
| `game.verbose_tracking_entities()`        | `game.trackingEntries`                                       |
| `game.process_input_fen(fen)`             | `game.previewFen(fen)` or `game.playFen(fen)`                |
| `game.process_input(locations, modifier)` | `game.preview(inputs)` or `game.play(inputs)`                |
| `game.item(location)`                     | `game.itemAt(position)`                                      |
| `game.square(location)`                   | `game.squareAt(position)`                                    |
| `game.locations_with_content()`           | `game.contentPositions()`                                    |
| `game.can_takeback(color)`                | `game.canTakeback(color)`                                    |
| `game.takeback()`                         | `game.takeback()`                                            |
| `game.without_last_turn(fens)`            | `game.previousTurn(fens)`                                    |
| `game.verify_moves(white, black)`         | `game.verifyHistory({ white, black })`                       |
| `game.is_later_than(otherFen)`            | `game.isLaterThan(otherGame)`                                |
| `game.available_move_kinds()`             | `game.availableMoveCounts()`                                 |
| `game.smartAutomove(mode)`                | `game.suggestMove(AutomovePreference.Fast \| Normal \| Pro)` |
| `game.automove()`                         | `game.suggestMove(AutomovePreference.Random)`                |
| `game.clearTracking()`                    | `game.clearTracking()`                                       |

Every `suggestMove` call is source-pure. In 0.2, `game.automove()` applied its
random move to the game; in 0.3, apply a returned suggestion explicitly:

```ts
const suggestion = game.suggestMove(AutomovePreference.Random);
if (suggestion !== undefined) {
  game.play(suggestion.inputs);
}
```

`verifyHistory` takes arrays of per-turn input FEN strings. Convert the old
dash-separated arguments before calling it:

```ts
game.verifyHistory({
  white: oldWhiteMoves === "" ? [] : oldWhiteMoves.split("-"),
  black: oldBlackMoves === "" ? [] : oldBlackMoves.split("-"),
});
```

`isLaterThan` now requires a validated `Game` instead of silently accepting an
invalid FEN:

```ts
const other = Game.fromFen(otherFen);
const later = other === undefined ? undefined : game.isLaterThan(other);
```

## Inputs and positions

`Location` and `Modifier` arguments are replaced by plain `Input` values:

```ts
// 0.2
game.process_input([new Location(10, 5), new Location(9, 4)]);

// 0.3
game.play([
  { kind: "position", position: { row: 10, column: 5 } },
  { kind: "position", position: { row: 9, column: 4 } },
]);
```

Coordinate fields changed from `i`/`j` to `row`/`column`. Potion and bomb
selection are explicit input values:

```ts
{ kind: "modifier", modifier: Modifier.SelectPotion }
{ kind: "modifier", modifier: Modifier.SelectBomb }
```

`Modifier.Cancel` has no replacement. Discard the partial input sequence in
the caller instead of sending a cancellation token.

`preview` and `previewFen` never mutate the game. `play` and `playFen` mutate
only when the complete sequence is legal; incomplete or invalid sequences
return `{ kind: "invalid", inputFen }`.

## Outputs, events, and board values

`OutputModel` and `OutputModelKind` are replaced by the `InputResolution`
discriminated union:

| 0.2 kind               | 0.3 result                                        |
| ---------------------- | ------------------------------------------------- |
| `InvalidInput`         | `{ kind: "invalid", inputFen }`                   |
| `LocationsToStartFrom` | `{ kind: "awaiting-start", inputFen, positions }` |
| `NextInputOptions`     | `{ kind: "awaiting-input", inputFen, options }`   |
| `Events`               | `{ kind: "complete", inputFen, events }`          |

Narrow on `kind` instead of calling wrapper accessors:

```ts
const resolution = game.previewFen("l10,5");
if (resolution.kind === "awaiting-input") {
  for (const option of resolution.options) {
    console.log(option.input, option.action);
  }
}
```

`EventModel`, `ItemModel`, `ManaModel`, `Mon`, `SquareModel`,
`NextInputModel`, and `VerboseTrackingEntityModel` are now plain readonly
objects. Their numeric `*Kind` enums were replaced by string `kind`
discriminants such as `"mon-move"`, `"regular"`, `"mon"`, and
`"next-turn"`. Event coordinates use descriptive fields including `from`,
`to`, `at`, and `by`.

Wasm lifecycle methods such as `free()` and mutable wrapper setters no longer
exist. Construct application values as object literals and treat values
returned by the engine as readonly.

## Removed engine-only operations

The following compatibility methods exposed mutable engine internals and have
no public replacement:

- `remove_item`
- `setVerboseTracking`
- `newForSimulation`
- `fromFenForSimulation`

Use `preview` for non-mutating simulation. `inactive_player_items_counters()`
is replaced by the color-keyed `game.potions` property; select the inactive
color explicitly.

## Match resolution

The positional `winner` function and its `""`, `"w"`, `"b"`, and `"x"`
return codes are replaced by `resolveMatch`:

```ts
const resolution = resolveMatch({
  white: { fen: whiteFen, moves: whiteMoves },
  black: { fen: blackFen, moves: blackMoves },
});

switch (resolution.kind) {
  case "ongoing":
    break;
  case "winner":
    console.log(resolution.winner);
    break;
  case "invalid":
    break;
}
```

Move histories are arrays rather than dash-separated strings.

## Validation differences

The 0.2 compatibility layer emulated Wasm coercions, including numeric enum
coercion, integer wrapping, permissive string normalization, and no-op
`free()` methods. Version 0.3 validates the TypeScript domain directly:

- invalid FEN and input FEN return `undefined` or an `"invalid"` result;
- unsupported variants, colors, and automove preferences throw `TypeError`;
- positions must use in-bounds integer `row` and `column` values;
- non-ASCII or non-canonical wire values are rejected rather than normalized.

Update callers to validate external data before constructing typed values.
