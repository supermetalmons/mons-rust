import {
  Color,
  MonKind,
  immutableItem,
  isMonFainted,
  itemEquals,
  itemMon,
  type Item,
  type Mon,
  type Square,
} from "./domain.js";
import {
  DEFAULT_GAME_VARIANT,
  GameVariant,
  MON_BASE_LOCATIONS,
  SUPERMANA_BASE,
  initialItemsForVariant,
  monBase,
  squareAtForVariant,
} from "./config.js";
import {
  ALL_LOCATIONS,
  BOARD_CELLS,
  assertValidLocation,
  fromLocationIndex,
  isValidLocation,
  locationIndex,
  type Location,
} from "./geometry.js";

type BoardEntry = readonly [Location, Item];

type BoardStorage = {
  readonly itemSlots: (Item | undefined)[];
  occupiedEntries: readonly BoardEntry[] | undefined;
};

export class Board {
  protected readonly storage: BoardStorage;

  public constructor(
    public readonly variant: GameVariant = DEFAULT_GAME_VARIANT,
    items?: readonly (Item | undefined)[],
    sharedStorage?: BoardStorage,
    itemsAreImmutable = false,
  ) {
    if (sharedStorage !== undefined) {
      this.storage = sharedStorage;
      return;
    }
    const initialItems = items ?? initialItemsForVariant(variant);
    if (initialItems.length !== BOARD_CELLS) {
      throw new RangeError(`board requires exactly ${BOARD_CELLS} item slots`);
    }
    this.storage = {
      itemSlots:
        items === undefined || itemsAreImmutable
          ? [...initialItems]
          : initialItems.map((item) =>
              item === undefined ? undefined : immutableItem(item),
            ),
      occupiedEntries: undefined,
    };
  }

  public get items(): readonly (Item | undefined)[] {
    return [...this.storage.itemSlots];
  }

  public static fromItems(
    items: readonly (Item | undefined)[],
    variant: GameVariant,
  ): MutableBoard {
    return new MutableBoard(variant, items);
  }

  /** Create an independently mutable board with shared immutable values. */
  public fork(): MutableBoard {
    const fork = new MutableBoard(
      this.variant,
      this.storage.itemSlots,
      undefined,
      true,
    );
    fork.storage.occupiedEntries = this.getOccupiedEntries();
    return fork;
  }

  public get(at: Location): Item | undefined {
    if (!isValidLocation(at)) {
      return undefined;
    }
    return this.storage.itemSlots[locationIndex(at)];
  }

  public squareAt(at: Location): Square {
    assertValidLocation(at);
    return squareAtForVariant(at, this.variant);
  }

  public allMonsBases(): readonly Location[] {
    return MON_BASE_LOCATIONS;
  }

  public supermanaBase(): Location {
    return SUPERMANA_BASE;
  }

  public allMonsLocations(color: Color): Location[] {
    const result: Location[] = [];
    for (let index = 0; index < BOARD_CELLS; index += 1) {
      const item = this.storage.itemSlots[index];
      if (item !== undefined && itemMon(item)?.color === color) {
        result.push(fromLocationIndex(index));
      }
    }
    return result;
  }

  public allFreeRegularManaLocations(color: Color): Location[] {
    const result: Location[] = [];
    for (let index = 0; index < BOARD_CELLS; index += 1) {
      const item = this.storage.itemSlots[index];
      if (
        item?.kind === "mana" &&
        item.mana.kind === "regular" &&
        item.mana.color === color
      ) {
        result.push(fromLocationIndex(index));
      }
    }
    return result;
  }

  public base(mon: Mon): Location {
    return monBase(mon.kind, mon.color);
  }

  public faintedMonsLocations(color: Color): Location[] {
    const result: Location[] = [];
    for (let index = 0; index < BOARD_CELLS; index += 1) {
      const item = this.storage.itemSlots[index];
      if (
        item?.kind === "mon" &&
        item.mon.color === color &&
        isMonFainted(item.mon)
      ) {
        result.push(fromLocationIndex(index));
      }
    }
    return result;
  }

  public findMana(color: Color): Location | undefined {
    for (let index = 0; index < BOARD_CELLS; index += 1) {
      const item = this.storage.itemSlots[index];
      if (
        item?.kind === "mana" &&
        item.mana.kind === "regular" &&
        item.mana.color === color
      ) {
        return fromLocationIndex(index);
      }
    }
    return undefined;
  }

  public findAwakeAngel(color: Color): Location | undefined {
    for (let index = 0; index < BOARD_CELLS; index += 1) {
      const item = this.storage.itemSlots[index];
      const mon = item === undefined ? undefined : itemMon(item);
      if (
        mon?.color === color &&
        mon.kind === MonKind.Angel &&
        !isMonFainted(mon)
      ) {
        return fromLocationIndex(index);
      }
    }
    return undefined;
  }

  /** @internal Compare board storage without allocating public snapshots. */
  public itemsEqual(other: Board): boolean {
    for (let index = 0; index < BOARD_CELLS; index += 1) {
      const leftItem = this.storage.itemSlots[index];
      const rightItem = other.storage.itemSlots[index];
      if (leftItem === undefined || rightItem === undefined) {
        if (leftItem !== rightItem) {
          return false;
        }
      } else if (!itemEquals(leftItem, rightItem)) {
        return false;
      }
    }
    return true;
  }

  private getOccupiedEntries(): readonly BoardEntry[] {
    if (this.storage.occupiedEntries !== undefined) {
      return this.storage.occupiedEntries;
    }
    const occupiedEntries: BoardEntry[] = [];
    for (let index = 0; index < BOARD_CELLS; index += 1) {
      const item = this.storage.itemSlots[index];
      if (item !== undefined) {
        const at = ALL_LOCATIONS[index];
        if (at !== undefined) {
          occupiedEntries.push([at, item]);
        }
      }
    }
    this.storage.occupiedEntries = occupiedEntries;
    return occupiedEntries;
  }

  /** Iterate a snapshot of the board's occupied locations in row-major order. */
  public entries(): IterableIterator<BoardEntry> {
    return this.getOccupiedEntries().values();
  }
}

/**
 * Mutable board storage for rules reducers and isolated simulations.
 *
 * Games expose {@link readonlyView} instead, so mutations cannot bypass their
 * revisioned query-cache and history boundaries.
 */
export class MutableBoard extends Board {
  #readonlyView: Board | undefined;

  public delete(at: Location): void {
    assertValidLocation(at);
    this.storage.itemSlots[locationIndex(at)] = undefined;
    this.storage.occupiedEntries = undefined;
  }

  public set(at: Location, item: Item): void {
    assertValidLocation(at);
    this.storage.itemSlots[locationIndex(at)] = immutableItem(item);
    this.storage.occupiedEntries = undefined;
  }

  /** A stable, live view that intentionally has no mutating methods. */
  public readonlyView(): Board {
    this.#readonlyView ??= new Board(this.variant, undefined, this.storage);
    return this.#readonlyView;
  }
}

export function boardEquals(left: Board, right: Board): boolean {
  if (left.variant !== right.variant) {
    return false;
  }
  return left.itemsEqual(right);
}
