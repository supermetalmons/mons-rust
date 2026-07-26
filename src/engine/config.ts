import { GameVariant } from "../api/types.js";
import {
  Color,
  Consumable,
  MonKind,
  SUPERMANA,
  consumableItem,
  createMon,
  manaItem,
  monItem,
  regularMana,
  type Item,
  type Square,
} from "./domain.js";
import {
  BOARD_CELLS,
  BOARD_SIZE,
  assertValidLocation,
  location,
  locationIndex,
  type Location,
} from "./geometry.js";
export { GameVariant };

export const DEFAULT_GAME_VARIANT = GameVariant.Classic;
export const TARGET_SCORE = 5;
export const MONS_MOVES_PER_TURN = 5;
export const MANA_MOVES_PER_TURN = 1;
export const ACTIONS_PER_TURN = 1;

export const ALL_GAME_VARIANTS: readonly GameVariant[] = Object.freeze([
  GameVariant.Classic,
  GameVariant.SwappedManaRows,
  GameVariant.OffsetArcManaRows,
  GameVariant.CenterSpokeManaRows,
  GameVariant.AlternatingManaRows,
  GameVariant.InnerWedgeManaRows,
  GameVariant.OuterWedgeManaRows,
  GameVariant.BentCenterManaRows,
  GameVariant.OuterEdgeManaRows,
  GameVariant.SplitFlankManaRows,
  GameVariant.ForwardBridgeManaRows,
  GameVariant.CornerChainManaRows,
]);

export const GAME_VARIANT_IDS = Object.freeze({
  [GameVariant.Classic]: 0,
  [GameVariant.SwappedManaRows]: 1,
  [GameVariant.OffsetArcManaRows]: 2,
  [GameVariant.CenterSpokeManaRows]: 3,
  [GameVariant.AlternatingManaRows]: 4,
  [GameVariant.InnerWedgeManaRows]: 5,
  [GameVariant.OuterWedgeManaRows]: 6,
  [GameVariant.BentCenterManaRows]: 7,
  [GameVariant.OuterEdgeManaRows]: 8,
  [GameVariant.SplitFlankManaRows]: 9,
  [GameVariant.ForwardBridgeManaRows]: 10,
  [GameVariant.CornerChainManaRows]: 11,
} as const satisfies Readonly<Record<GameVariant, number>>);

const GAME_VARIANTS_BY_ID = new Map<number, GameVariant>(
  ALL_GAME_VARIANTS.map((variant) => [GAME_VARIANT_IDS[variant], variant]),
);

export function gameVariantId(variant: GameVariant): number {
  return GAME_VARIANT_IDS[variant];
}

type ManaLayout = Readonly<{
  [Color.White]: readonly Location[];
  [Color.Black]: readonly Location[];
}>;

function locations(
  ...pairs: readonly (readonly [number, number])[]
): readonly Location[] {
  return Object.freeze(pairs.map(([i, j]) => Object.freeze(location(i, j))));
}

function manaLayout(
  black: readonly Location[],
  white: readonly Location[],
): ManaLayout {
  return Object.freeze({
    [Color.Black]: black,
    [Color.White]: white,
  });
}

const MANA_LAYOUTS = Object.freeze({
  [GameVariant.Classic]: manaLayout(
    locations([3, 4], [3, 6], [4, 3], [4, 5], [4, 7]),
    locations([7, 4], [7, 6], [6, 3], [6, 5], [6, 7]),
  ),
  [GameVariant.SwappedManaRows]: manaLayout(
    locations([3, 3], [3, 5], [3, 7], [4, 4], [4, 6]),
    locations([7, 3], [7, 5], [7, 7], [6, 4], [6, 6]),
  ),
  [GameVariant.OffsetArcManaRows]: manaLayout(
    locations([3, 4], [3, 6], [4, 2], [4, 5], [4, 8]),
    locations([6, 2], [6, 5], [6, 8], [7, 4], [7, 6]),
  ),
  [GameVariant.CenterSpokeManaRows]: manaLayout(
    locations([3, 5], [4, 2], [4, 4], [4, 6], [4, 8]),
    locations([6, 2], [6, 4], [6, 6], [6, 8], [7, 5]),
  ),
  [GameVariant.AlternatingManaRows]: manaLayout(
    locations([4, 1], [4, 3], [4, 5], [4, 7], [4, 9]),
    locations([6, 1], [6, 3], [6, 5], [6, 7], [6, 9]),
  ),
  [GameVariant.InnerWedgeManaRows]: manaLayout(
    locations([3, 4], [3, 6], [4, 4], [4, 5], [4, 6]),
    locations([6, 4], [6, 5], [6, 6], [7, 4], [7, 6]),
  ),
  [GameVariant.OuterWedgeManaRows]: manaLayout(
    locations([3, 4], [3, 5], [3, 6], [4, 3], [4, 7]),
    locations([6, 3], [6, 7], [7, 4], [7, 5], [7, 6]),
  ),
  [GameVariant.BentCenterManaRows]: manaLayout(
    locations([3, 5], [4, 4], [4, 5], [4, 6], [5, 3]),
    locations([5, 7], [6, 4], [6, 5], [6, 6], [7, 5]),
  ),
  [GameVariant.OuterEdgeManaRows]: manaLayout(
    locations([4, 0], [4, 1], [4, 9], [4, 10], [5, 1]),
    locations([5, 9], [6, 0], [6, 1], [6, 9], [6, 10]),
  ),
  [GameVariant.SplitFlankManaRows]: manaLayout(
    locations([3, 4], [3, 6], [4, 2], [4, 3], [5, 2]),
    locations([5, 8], [6, 7], [6, 8], [7, 4], [7, 6]),
  ),
  [GameVariant.ForwardBridgeManaRows]: manaLayout(
    locations([3, 4], [3, 6], [4, 3], [4, 5], [5, 4]),
    locations([5, 6], [6, 5], [6, 7], [7, 4], [7, 6]),
  ),
  [GameVariant.CornerChainManaRows]: manaLayout(
    locations([3, 5], [3, 6], [4, 6], [4, 7], [5, 7]),
    locations([5, 3], [6, 3], [6, 4], [7, 4], [7, 5]),
  ),
} satisfies Readonly<Record<GameVariant, ManaLayout>>);

const REGULAR_SQUARE: Square = Object.freeze({ kind: "regular" });
const CONSUMABLE_BASE_SQUARE: Square = Object.freeze({
  kind: "consumable-base",
});
const SUPERMANA_BASE_SQUARE: Square = Object.freeze({ kind: "supermana-base" });

export const SUPERMANA_BASE: Location = Object.freeze(location(5, 5));

export const MON_BASE_LOCATIONS: readonly Location[] = locations(
  [0, 3],
  [0, 4],
  [0, 5],
  [0, 6],
  [0, 7],
  [10, 3],
  [10, 4],
  [10, 5],
  [10, 6],
  [10, 7],
);

export function gameVariantFromId(id: number): GameVariant | undefined {
  return Number.isSafeInteger(id) ? GAME_VARIANTS_BY_ID.get(id) : undefined;
}

export function parseGameVariant(value: string): GameVariant | undefined {
  if (!/^(?:0|[1-9]\d*)$/u.test(value)) return undefined;
  return gameVariantFromId(Number(value));
}

export function manaBaseLocations(
  variant: GameVariant,
  color: Color,
): readonly Location[] {
  return MANA_LAYOUTS[variant][color];
}

function buildSquares(variant: GameVariant): readonly Square[] {
  const squares = Array.from<Square>({ length: BOARD_CELLS }).fill(
    REGULAR_SQUARE,
  );
  const put = (at: Location, square: Square): void => {
    squares[locationIndex(at)] = Object.freeze(square);
  };

  put(location(0, 0), { kind: "mana-pool", color: Color.Black });
  put(location(0, 10), { kind: "mana-pool", color: Color.Black });
  put(location(10, 0), { kind: "mana-pool", color: Color.White });
  put(location(10, 10), { kind: "mana-pool", color: Color.White });

  put(location(0, 3), {
    kind: "mon-base",
    monKind: MonKind.Mystic,
    color: Color.Black,
  });
  put(location(0, 4), {
    kind: "mon-base",
    monKind: MonKind.Spirit,
    color: Color.Black,
  });
  put(location(0, 5), {
    kind: "mon-base",
    monKind: MonKind.Drainer,
    color: Color.Black,
  });
  put(location(0, 6), {
    kind: "mon-base",
    monKind: MonKind.Angel,
    color: Color.Black,
  });
  put(location(0, 7), {
    kind: "mon-base",
    monKind: MonKind.Demon,
    color: Color.Black,
  });

  put(location(10, 3), {
    kind: "mon-base",
    monKind: MonKind.Demon,
    color: Color.White,
  });
  put(location(10, 4), {
    kind: "mon-base",
    monKind: MonKind.Angel,
    color: Color.White,
  });
  put(location(10, 5), {
    kind: "mon-base",
    monKind: MonKind.Drainer,
    color: Color.White,
  });
  put(location(10, 6), {
    kind: "mon-base",
    monKind: MonKind.Spirit,
    color: Color.White,
  });
  put(location(10, 7), {
    kind: "mon-base",
    monKind: MonKind.Mystic,
    color: Color.White,
  });

  for (const at of manaBaseLocations(variant, Color.Black)) {
    put(at, { kind: "mana-base", color: Color.Black });
  }
  for (const at of manaBaseLocations(variant, Color.White)) {
    put(at, { kind: "mana-base", color: Color.White });
  }

  put(location(5, 0), CONSUMABLE_BASE_SQUARE);
  put(location(5, 10), CONSUMABLE_BASE_SQUARE);
  put(SUPERMANA_BASE, SUPERMANA_BASE_SQUARE);
  return Object.freeze(squares);
}

const SQUARES_BY_VARIANT = new Map<GameVariant, readonly Square[]>(
  ALL_GAME_VARIANTS.map((variant) => [variant, buildSquares(variant)]),
);

export function squaresForVariant(variant: GameVariant): readonly Square[] {
  const squares = SQUARES_BY_VARIANT.get(variant);
  if (squares === undefined) {
    throw new TypeError(`unsupported game variant: ${variant}`);
  }
  return squares;
}

export function squareAtForVariant(at: Location, variant: GameVariant): Square {
  assertValidLocation(at);
  return squaresForVariant(variant)[locationIndex(at)] ?? REGULAR_SQUARE;
}

export function squareAt(at: Location): Square {
  return squareAtForVariant(at, DEFAULT_GAME_VARIANT);
}

function initialItemForSquare(square: Square): Item | undefined {
  switch (square.kind) {
    case "mon-base":
      return monItem(createMon(square.monKind, square.color, 0));
    case "mana-base":
      return manaItem(regularMana(square.color));
    case "supermana-base":
      return manaItem(SUPERMANA);
    case "consumable-base":
      return consumableItem(Consumable.BombOrPotion);
    case "regular":
    case "mana-pool":
      return undefined;
  }
}

function buildInitialItems(
  variant: GameVariant,
): readonly (Item | undefined)[] {
  return Object.freeze(squaresForVariant(variant).map(initialItemForSquare));
}

const INITIAL_ITEMS_BY_VARIANT = new Map<
  GameVariant,
  readonly (Item | undefined)[]
>(ALL_GAME_VARIANTS.map((variant) => [variant, buildInitialItems(variant)]));

/** A new slot array containing immutable initial board values. */
export function initialItemsForVariant(
  variant: GameVariant,
): (Item | undefined)[] {
  const initial = INITIAL_ITEMS_BY_VARIANT.get(variant);
  if (initial === undefined) {
    throw new TypeError(`unsupported game variant: ${variant}`);
  }
  return [...initial];
}

export function monBase(kind: MonKind, color: Color): Location {
  for (const at of MON_BASE_LOCATIONS) {
    const square = squareAt(at);
    if (
      square.kind === "mon-base" &&
      square.monKind === kind &&
      square.color === color
    ) {
      return at;
    }
  }
  throw new Error("Expected at least one base for the given mon");
}

export { BOARD_SIZE };
