export const packageName = "mons-rules";
export const packageEntry = "./dist/mons-rules.js";
export const typesEntry = "./dist/entrypoints/mons-rules.d.ts";
export const publishedDistFiles = [
  "api/game.d.ts",
  "api/types.d.ts",
  "api/winner.d.ts",
  "entrypoints/mons-rules.d.ts",
  "mons-rules.js",
].sort();
export const expectedRuntimeExports = [
  "Color",
  "Consumable",
  "Game",
  "GameVariant",
  "Modifier",
  "MonKind",
  "resolveMatch",
].sort();
export const expectedTypeExports = [
  "AvailableMoveCounts",
  "BoardItem",
  "GameEvent",
  "Input",
  "InputResolution",
  "Mana",
  "Mon",
  "Position",
  "Square",
].sort();
