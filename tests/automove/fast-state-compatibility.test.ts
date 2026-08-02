import { describe, expect, it, vi } from "vitest";

import {
  loadPosition,
  moveToInputs,
  tryLoadPosition,
} from "../../src/automove/fast/bridge.js";
import {
  SQ_POOL,
  cellCooldown,
  fastSquaresForVariant,
} from "../../src/automove/fast/board.js";
import { MAX_MOVES, generateMoves } from "../../src/automove/fast/moves.js";
import {
  FastPosition,
  MOVE_MANA,
  applyFastMoveAndCheckRepresentability,
  moveFrom,
  moveTo,
  moveType,
} from "../../src/automove/fast/position.js";
import {
  Color,
  Consumable,
  Modifier,
  MonKind,
  createMon,
  consumableItem,
  manaItem,
  monItem,
  monWithConsumableItem,
  monWithManaItem,
  regularMana,
  type Input,
  type Item,
} from "../../src/engine/domain.js";
import {
  ALL_GAME_VARIANTS,
  DEFAULT_GAME_VARIANT,
} from "../../src/engine/config.js";
import { inputArrayFen } from "../../src/engine/fen.js";
import { MonsGame } from "../../src/engine/game.js";
import {
  BOARD_CELLS,
  BOARD_SIZE,
  location,
  locationIndex,
  type Location,
} from "../../src/engine/geometry.js";
import {
  expectFastPositionInvariants,
  fastPositionSnapshot,
  gameWith,
  locationInput,
  resetFastPosition,
} from "./fast.test-helper.js";

const INT32_MAX = 0x7fff_ffff;

const DAMAGING_ATTACKS = [
  {
    name: "mystic",
    actor: monItem(createMon(MonKind.Mystic, Color.White, 0)),
    to: location(3, 1),
  },
  {
    name: "demon",
    actor: monItem(createMon(MonKind.Demon, Color.White, 0)),
    to: location(3, 3),
  },
  {
    name: "bomb",
    actor: monWithConsumableItem(
      createMon(MonKind.Angel, Color.White, 0),
      Consumable.Bomb,
    ),
    to: location(3, 3),
  },
] as const;

function moveInputs(from: Location, to: Location): Input[] {
  return [locationInput(from), locationInput(to)];
}

function generatedMoves(game: MonsGame): Map<string, number> {
  const position = new FastPosition();
  loadPosition(position, game);
  expectFastPositionInvariants(position);
  const moves = new Int32Array(MAX_MOVES);
  const count = generateMoves(position, moves);
  const generated = new Map<string, number>();
  for (let index = 0; index < count; index += 1) {
    const move = moves[index];
    if (move !== undefined) {
      generated.set(inputArrayFen(moveToInputs(move)), move);
    }
  }
  return generated;
}

function manaEntries(count: number): (readonly [Location, Item])[] {
  return Array.from({ length: count }, (_, index) => [
    location(Math.trunc(index / BOARD_SIZE), index % BOARD_SIZE),
    manaItem(regularMana(index % 2 === 0 ? Color.White : Color.Black)),
  ]);
}

describe("fast packed-state compatibility", () => {
  it("returns independently owned square layouts", () => {
    const first = fastSquaresForVariant(DEFAULT_GAME_VARIANT);
    const second = fastSquaresForVariant(DEFAULT_GAME_VARIANT);
    const expected = [...second];

    expect(first).not.toBe(second);
    first.fill(0xff);

    expect([...second]).toEqual(expected);
    expect([...fastSquaresForVariant(DEFAULT_GAME_VARIANT)]).toEqual(expected);
  });

  it("keeps pool and mon-base topology fixed across current variants", () => {
    const fixedTopology = (squares: Uint8Array): number[] =>
      Array.from(squares, (square) => (square >= SQ_POOL ? square : 0));
    const expected = fixedTopology(fastSquaresForVariant(DEFAULT_GAME_VARIANT));

    for (const variant of ALL_GAME_VARIANTS) {
      expect(fixedTopology(fastSquaresForVariant(variant)), variant).toEqual(
        expected,
      );
    }
  });

  it("owns reset topology and shares it across controlled copies", () => {
    const sourceSquares = fastSquaresForVariant(DEFAULT_GAME_VARIANT);
    const position = new FastPosition();
    resetFastPosition(position, { squares: sourceSquares });
    const expectedSquares = [...position.squares];

    sourceSquares.fill(0xff);
    expect([...position.squares]).toEqual(expectedSquares);

    const copy = new FastPosition();
    copy.copyFrom(position);
    expect(copy.squares).toBe(position.squares);
    expectFastPositionInvariants(position);
    expectFastPositionInvariants(copy);
  });

  it("rejects malformed reset snapshots before mutation", () => {
    const position = new FastPosition();
    loadPosition(position, new MonsGame(false));
    const before = fastPositionSnapshot(position);
    const squaresBefore = position.squares;

    for (const [name, overrides] of [
      ["cells", { cells: before.cells.slice(1) }],
      ["squares", { squares: before.squares.slice(1) }],
      ["potions", { potions: before.potions.slice(1) }],
    ] as const) {
      expect(() => position.reset({ ...before, ...overrides }), name).toThrow(
        RangeError,
      );
      expect(fastPositionSnapshot(position), name).toEqual(before);
      expect(position.squares, name).toBe(squaresBefore);
      expectFastPositionInvariants(position, name);
    }
  });

  it("rejects move buffers that cannot hold the generated moves", () => {
    const position = new FastPosition();
    loadPosition(position, new MonsGame(true));
    const required = generateMoves(position, new Int32Array(MAX_MOVES));

    expect(required).toBeGreaterThan(0);
    expect(() => generateMoves(position, new Int32Array(required - 1))).toThrow(
      "fast move buffer capacity exceeded",
    );
  });

  it("rejects unsupported states before mutating the destination", () => {
    const position = new FastPosition();
    expect(tryLoadPosition(position, new MonsGame(false), 1)).toBe(true);
    expectFastPositionInvariants(position);
    const before = fastPositionSnapshot(position);
    const squaresBefore = position.squares;
    const duplicateMon = createMon(MonKind.Demon, Color.White, 0);
    const duplicateGame = gameWith([
      [location(5, 3), monItem(duplicateMon)],
      [location(5, 4), monWithConsumableItem(duplicateMon, Consumable.Bomb)],
    ]);
    const tooMuchManaGame = gameWith(
      manaEntries(position.manaIndices.length + 1),
    );
    const tooMuchTotalManaGame = gameWith([
      ...manaEntries(position.manaIndices.length),
      [
        location(5, 5),
        monWithManaItem(
          createMon(MonKind.Drainer, Color.White, 0),
          regularMana(Color.White),
        ),
      ],
    ]);
    const whitePotionGame = gameWith([], {
      whitePotionsCount: INT32_MAX - 1,
    });
    const blackPotionGame = gameWith([], {
      blackPotionsCount: INT32_MAX - 1,
    });
    const turnGame = gameWith([], {
      turnNumber: Number.MAX_SAFE_INTEGER - 1,
    });

    for (const [name, game, maxDepth] of [
      ["negative depth", new MonsGame(false), -1],
      ["fractional depth", new MonsGame(false), 1.5],
      ["duplicate mon", duplicateGame, 1],
      ["too much mana", tooMuchManaGame, 1],
      ["too much loose and carried mana", tooMuchTotalManaGame, 1],
      ["white potion headroom", whitePotionGame, 2],
      ["black potion headroom", blackPotionGame, 2],
      ["turn headroom", turnGame, 2],
    ] as const) {
      expect(tryLoadPosition(position, game, maxDepth), name).toBe(false);
      expect(fastPositionSnapshot(position), name).toEqual(before);
      expect(position.squares, name).toBe(squaresBefore);
      expectFastPositionInvariants(position, name);
    }
  });

  it("fails soft on bridge workspace allocation without mutation", () => {
    const game = new MonsGame(false, DEFAULT_GAME_VARIANT);
    const position = new FastPosition();
    loadPosition(position, game);
    const before = fastPositionSnapshot(position);
    const squaresBefore = position.squares;
    const NativeUint16Array = globalThis.Uint16Array;
    const FailingUint16Array = function (value: unknown): Uint16Array {
      if (value === BOARD_CELLS) {
        throw new RangeError("synthetic bridge allocation failure");
      }
      return Reflect.construct(NativeUint16Array, [value]) as Uint16Array;
    } as unknown as Uint16ArrayConstructor;

    vi.stubGlobal("Uint16Array", FailingUint16Array);
    try {
      expect(tryLoadPosition(position, game, 1)).toBe(false);
    } finally {
      vi.unstubAllGlobals();
    }

    expect(fastPositionSnapshot(position)).toEqual(before);
    expect(position.squares).toBe(squaresBefore);
  });

  it("does not misclassify position invariant errors as allocation failures", () => {
    const failure = new RangeError("synthetic position invariant failure");
    const position = new FastPosition();
    const rejectingPosition = new Proxy(position, {
      get(target, property, receiver) {
        if (property === "reset") {
          return () => {
            throw failure;
          };
        }
        return Reflect.get(target, property, receiver);
      },
    });

    expect(() =>
      tryLoadPosition(
        rejectingPosition,
        new MonsGame(false, DEFAULT_GAME_VARIANT),
        1,
      ),
    ).toThrow(failure);
  });

  it("accepts exact packed counter and mana boundaries", () => {
    const maxDepth = 2;
    const game = gameWith(manaEntries(16), {
      whitePotionsCount: INT32_MAX - maxDepth,
      blackPotionsCount: INT32_MAX - maxDepth,
      turnNumber: Number.MAX_SAFE_INTEGER - maxDepth,
    });
    const position = new FastPosition();

    expect(position.manaIndices.length).toBe(16);
    expect(tryLoadPosition(position, game, maxDepth)).toBe(true);
    expect(position.manaCount).toBe(16);
    expect(position.potions[0]).toBe(INT32_MAX - maxDepth);
    expect(position.potions[1]).toBe(INT32_MAX - maxDepth);
    expectFastPositionInvariants(position);
  });

  it("rejects direct loads that exceed the loose mana index capacity", () => {
    const position = new FastPosition();
    loadPosition(position, new MonsGame(false));
    const before = fastPositionSnapshot(position);
    const squaresBefore = position.squares;

    expect(() =>
      loadPosition(
        position,
        gameWith(manaEntries(position.manaIndices.length + 1)),
      ),
    ).toThrow("fast mana index capacity exceeded");
    expect(fastPositionSnapshot(position)).toEqual(before);
    expect(position.squares).toBe(squaresBefore);
    expectFastPositionInvariants(position);
  });

  it("rejects concrete loose consumables as mon-move destinations", () => {
    const from = location(5, 3);
    const to = location(5, 4);
    const mon = createMon(MonKind.Demon, Color.White, 0);
    const actors = [
      ["plain", monItem(mon)],
      ["mana carrier", monWithManaItem(mon, regularMana(Color.White))],
      ["bomb carrier", monWithConsumableItem(mon, Consumable.Bomb)],
    ] as const;

    for (const [actorName, actor] of actors) {
      for (const consumable of [Consumable.Bomb, Consumable.Potion]) {
        const game = gameWith([
          [from, actor],
          [to, consumableItem(consumable)],
        ]);
        const inputs = moveInputs(from, to);
        const inputFen = inputArrayFen(inputs);

        expect(
          game.fork().processInput(inputs, false, false).kind,
          `${actorName} ${consumable}`,
        ).toBe("invalid-input");
        expect(
          generatedMoves(game).has(inputFen),
          `${actorName} ${consumable}`,
        ).toBe(false);
      }
    }
  });

  it("uses raw mon destinations when deciding whether mana may start", () => {
    const from = location(10, 3);
    const concreteDestination = location(9, 2);
    const manaAt = location(8, 8);
    const game = gameWith([
      [from, monItem(createMon(MonKind.Demon, Color.White, 0))],
      [concreteDestination, consumableItem(Consumable.Potion)],
      [location(9, 3), monItem(createMon(MonKind.Angel, Color.Black, 0))],
      [location(9, 4), monItem(createMon(MonKind.Demon, Color.Black, 0))],
      [location(10, 2), monItem(createMon(MonKind.Mystic, Color.Black, 0))],
      [location(10, 4), monItem(createMon(MonKind.Spirit, Color.Black, 0))],
      [manaAt, manaItem(regularMana(Color.White))],
    ]);
    const position = new FastPosition();
    loadPosition(position, game);
    expectFastPositionInvariants(position);
    const count = generateMoves(position, new Int32Array(MAX_MOVES));

    expect(count).toBe(0);

    const root = game.processInput([], false, false);
    expect(root.kind).toBe("locations-to-start-from");
    if (root.kind !== "locations-to-start-from") return;
    expect(root.locations).toContainEqual(from);
    expect(root.locations).not.toContainEqual(manaAt);
  });

  it("uses raw attack targets when deciding whether mana may start", () => {
    const from = location(5, 3);
    const targetAt = location(3, 1);
    const manaAt = location(8, 8);
    const game = gameWith(
      [
        [from, monItem(createMon(MonKind.Mystic, Color.White, 0))],
        [
          targetAt,
          monWithConsumableItem(
            createMon(MonKind.Drainer, Color.Black, 0),
            Consumable.Potion,
          ),
        ],
        [manaAt, manaItem(regularMana(Color.White))],
      ],
      { monsMovesCount: 5 },
    );
    const position = new FastPosition();
    loadPosition(position, game);
    const count = generateMoves(position, new Int32Array(MAX_MOVES));

    expect(count).toBe(0);

    const root = game.processInput([], false, false);
    expect(root.kind).toBe("locations-to-start-from");
    if (root.kind !== "locations-to-start-from") return;
    expect(root.locations).toContainEqual(from);
    expect(root.locations).not.toContainEqual(manaAt);
  });

  it("keeps loose mana indices sorted through pickup and relocation", () => {
    const drainerAt = location(5, 3);
    const pickupAt = location(5, 4);
    const firstManaAt = location(8, 8);
    const secondManaAt = location(9, 9);
    const game = gameWith(
      [
        [drainerAt, monItem(createMon(MonKind.Drainer, Color.White, 0))],
        [pickupAt, manaItem(regularMana(Color.White))],
        [firstManaAt, manaItem(regularMana(Color.White))],
        [secondManaAt, manaItem(regularMana(Color.White))],
      ],
      {
        actionsUsedCount: 1,
        monsMovesCount: 4,
      },
    );
    const position = new FastPosition();
    loadPosition(position, game);
    const pickupMove = generatedMoves(game).get(
      inputArrayFen(moveInputs(drainerAt, pickupAt)),
    );
    expect(pickupMove).toBeDefined();
    if (pickupMove === undefined) return;

    expect(applyFastMoveAndCheckRepresentability(position, pickupMove)).toBe(
      true,
    );
    expectFastPositionInvariants(position);
    expect([...position.manaIndices].slice(0, position.manaCount)).toEqual([
      locationIndex(firstManaAt),
      locationIndex(secondManaAt),
    ]);

    const moves = new Int32Array(MAX_MOVES);
    const count = generateMoves(position, moves);
    const origins: number[] = [];
    const destinationsByOrigin = new Map<number, number[]>();
    let relocationMove = 0;
    for (let slot = 0; slot < count; slot += 1) {
      const move = moves[slot] ?? 0;
      if (moveType(move) !== MOVE_MANA) continue;
      const origin = moveFrom(move);
      if (origins[origins.length - 1] !== origin) origins.push(origin);
      const destinations = destinationsByOrigin.get(origin) ?? [];
      destinations.push(moveTo(move));
      destinationsByOrigin.set(origin, destinations);
      if (relocationMove === 0) relocationMove = move;
    }

    expect(origins).toEqual([
      locationIndex(firstManaAt),
      locationIndex(secondManaAt),
    ]);
    const canonical = game.fork();
    expect(
      canonical.processInput(moveInputs(drainerAt, pickupAt), false, false)
        .kind,
    ).toBe("events");
    for (const manaAt of [firstManaAt, secondManaAt]) {
      const output = canonical
        .fork()
        .processInput([locationInput(manaAt)], false, false);
      expect(output.kind).toBe("next-input-options");
      if (output.kind !== "next-input-options") continue;
      expect(destinationsByOrigin.get(locationIndex(manaAt))).toEqual(
        output.nextInputs.map((option) => {
          if (option.input.kind !== "location") {
            throw new TypeError("mana destination must be a location");
          }
          return locationIndex(option.input.location);
        }),
      );
    }
    expect(relocationMove).not.toBe(0);
    expect(
      applyFastMoveAndCheckRepresentability(position, relocationMove),
    ).toBe(true);
    expectFastPositionInvariants(position);
    expect([...position.manaIndices].slice(0, position.manaCount)).toEqual(
      [moveTo(relocationMove), locationIndex(secondManaAt)].sort(
        (left, right) => left - right,
      ),
    );
  });

  it("preserves BombOrPotion collection for plain mons and carriers", () => {
    const from = location(5, 3);
    const to = location(5, 4);
    const mon = createMon(MonKind.Demon, Color.White, 0);
    const plainGame = gameWith([
      [from, monItem(mon)],
      [to, consumableItem(Consumable.BombOrPotion)],
    ]);
    const baseInputs = moveInputs(from, to);
    const plainMoves = generatedMoves(plainGame);

    for (const modifier of [Modifier.SelectBomb, Modifier.SelectPotion]) {
      const inputs: Input[] = [...baseInputs, { kind: "modifier", modifier }];
      expect(plainMoves.has(inputArrayFen(inputs)), modifier).toBe(true);
      expect(
        plainGame.fork().processInput(inputs, false, false).kind,
        modifier,
      ).toBe("events");
    }

    for (const [name, actor] of [
      ["mana carrier", monWithManaItem(mon, regularMana(Color.White))],
      ["bomb carrier", monWithConsumableItem(mon, Consumable.Bomb)],
    ] as const) {
      const game = gameWith([
        [from, actor],
        [to, consumableItem(Consumable.BombOrPotion)],
      ]);
      expect(generatedMoves(game).has(inputArrayFen(baseInputs)), name).toBe(
        true,
      );
      expect(
        game.fork().processInput(baseInputs, false, false).kind,
        name,
      ).toBe("events");
    }
  });

  it("preserves concrete Demon step inputs while ignoring their step event", () => {
    const from = location(5, 3);
    const targetAt = location(5, 5);
    const stepAt = location(5, 6);
    const target = monItem(createMon(MonKind.Drainer, Color.Black, 0));

    for (const consumable of [Consumable.Bomb, Consumable.Potion]) {
      const game = gameWith([
        [from, monItem(createMon(MonKind.Demon, Color.White, 0))],
        [targetAt, target],
        [stepAt, consumableItem(consumable)],
      ]);
      const inputs = [
        locationInput(from),
        locationInput(targetAt),
        locationInput(stepAt),
      ];
      const move = generatedMoves(game).get(inputArrayFen(inputs));
      expect(move, consumable).toBeDefined();
      if (move === undefined) continue;

      const canonical = game.fork();
      const output = canonical.processInput(inputs, false, false);
      expect(output.kind, consumable).toBe("events");
      if (output.kind !== "events") continue;
      expect(
        output.events.some((event) => event.kind === "demon-additional-step"),
        consumable,
      ).toBe(false);

      const position = new FastPosition();
      loadPosition(position, game);
      expect(
        applyFastMoveAndCheckRepresentability(position, move),
        consumable,
      ).toBe(true);
      expectFastPositionInvariants(position, consumable);
      const expected = new FastPosition();
      loadPosition(expected, canonical);
      expectFastPositionInvariants(expected, consumable);
      expect(fastPositionSnapshot(position), consumable).toEqual(
        fastPositionSnapshot(expected),
      );
    }
  });

  it("reports Demon Bomb-step successors that cannot stay packed", () => {
    const from = location(5, 3);
    const targetAt = location(5, 5);
    const stepAt = location(5, 6);
    const game = gameWith([
      [from, monItem(createMon(MonKind.Demon, Color.White, 0))],
      [
        targetAt,
        monWithConsumableItem(
          createMon(MonKind.Drainer, Color.Black, 0),
          Consumable.Bomb,
        ),
      ],
    ]);
    const inputs = [
      locationInput(from),
      locationInput(targetAt),
      locationInput(stepAt),
    ];
    const move = generatedMoves(game).get(inputArrayFen(inputs));
    expect(move).toBeDefined();
    if (move === undefined) return;

    const position = new FastPosition();
    loadPosition(position, game);
    expect(applyFastMoveAndCheckRepresentability(position, move)).toBe(false);

    const canonical = game.fork();
    expect(canonical.processInput(inputs, false, false).kind).toBe("events");
    const expected = new FastPosition();
    loadPosition(expected, canonical);
    expect([...position.cells]).toEqual([...expected.cells]);
    expect(position.hashLo).toBe(expected.hashLo);
    expect(position.hashHi).toBe(expected.hashHi);
    expect(tryLoadPosition(new FastPosition(), canonical, 1)).toBe(false);
  });

  it("keeps a Demon Bomb step onto its own base representable", () => {
    const from = location(8, 4);
    const targetAt = location(10, 4);
    const demonBase = location(10, 3);
    const game = gameWith([
      [from, monItem(createMon(MonKind.Demon, Color.White, 0))],
      [
        targetAt,
        monWithConsumableItem(
          createMon(MonKind.Drainer, Color.Black, 0),
          Consumable.Bomb,
        ),
      ],
    ]);
    const inputs = [
      locationInput(from),
      locationInput(targetAt),
      locationInput(demonBase),
    ];
    const move = generatedMoves(game).get(inputArrayFen(inputs));
    expect(move).toBeDefined();
    if (move === undefined) return;

    const position = new FastPosition();
    loadPosition(position, game);
    expect(applyFastMoveAndCheckRepresentability(position, move)).toBe(true);
    expectFastPositionInvariants(position);

    const canonical = game.fork();
    expect(canonical.processInput(inputs, false, false).kind).toBe("events");
    const expected = new FastPosition();
    expect(tryLoadPosition(expected, canonical, 1)).toBe(true);
    expectFastPositionInvariants(expected);
    expect(fastPositionSnapshot(position)).toEqual(
      fastPositionSnapshot(expected),
    );
  });

  it("rejects damaging attacks against Potion and BombOrPotion carriers", () => {
    const from = location(5, 3);
    const targetMon = createMon(MonKind.Drainer, Color.Black, 0);

    for (const attack of DAMAGING_ATTACKS) {
      for (const consumable of [Consumable.Potion, Consumable.BombOrPotion]) {
        const game = gameWith([
          [from, attack.actor],
          [attack.to, monWithConsumableItem(targetMon, consumable)],
        ]);
        const inputs = moveInputs(from, attack.to);
        const label = `${attack.name} ${consumable}`;

        expect(game.fork().processInput(inputs, false, false).kind, label).toBe(
          "invalid-input",
        );
        expect(generatedMoves(game).has(inputArrayFen(inputs)), label).toBe(
          false,
        );
      }
    }
  });

  it("preserves damaging attacks against Bomb carriers", () => {
    const from = location(5, 3);
    const targetMon = createMon(MonKind.Drainer, Color.Black, 0);

    for (const attack of DAMAGING_ATTACKS) {
      const game = gameWith([
        [from, attack.actor],
        [attack.to, monWithConsumableItem(targetMon, Consumable.Bomb)],
      ]);
      const inputs = moveInputs(from, attack.to);

      expect(generatedMoves(game).has(inputArrayFen(inputs)), attack.name).toBe(
        true,
      );
      expect(
        game.fork().processInput(inputs, false, false).kind,
        attack.name,
      ).toBe("events");
    }
  });

  it("decrements cooldown only for plain fainted mons after turn advance", () => {
    const from = location(5, 3);
    const to = location(5, 4);
    const plainAt = location(2, 2);
    const manaCarrierAt = location(2, 3);
    const consumableCarrierAt = location(2, 4);
    const game = gameWith(
      [
        [from, monItem(createMon(MonKind.Demon, Color.White, 0))],
        [plainAt, monItem(createMon(MonKind.Drainer, Color.Black, 2))],
        [
          manaCarrierAt,
          monWithManaItem(
            createMon(MonKind.Angel, Color.Black, 2),
            regularMana(Color.Black),
          ),
        ],
        [
          consumableCarrierAt,
          monWithConsumableItem(
            createMon(MonKind.Spirit, Color.Black, 2),
            Consumable.Bomb,
          ),
        ],
      ],
      { manaMovesCount: 1 },
    );
    const inputs = moveInputs(from, to);
    const position = new FastPosition();
    expect(tryLoadPosition(position, game, 1)).toBe(true);
    expectFastPositionInvariants(position);
    const move = generatedMoves(game).get(inputArrayFen(inputs));
    expect(move).toBeDefined();
    if (move === undefined) return;

    expect(applyFastMoveAndCheckRepresentability(position, move)).toBe(true);
    expectFastPositionInvariants(position);

    expect(cellCooldown(position.cells[locationIndex(plainAt)] ?? 0)).toBe(1);
    expect(
      cellCooldown(position.cells[locationIndex(manaCarrierAt)] ?? 0),
    ).toBe(2);
    expect(
      cellCooldown(position.cells[locationIndex(consumableCarrierAt)] ?? 0),
    ).toBe(2);

    const canonical = game.fork();
    expect(canonical.processInput(inputs, false, false).kind).toBe("events");
    const expected = new FastPosition();
    loadPosition(expected, canonical);
    expectFastPositionInvariants(expected);
    expect(fastPositionSnapshot(position)).toEqual(
      fastPositionSnapshot(expected),
    );
  });

  it("removes a mon overwritten at a fainted target's home base", () => {
    const from = location(5, 3);
    const targetAt = location(3, 1);
    const targetBase = location(0, 5);
    const game = gameWith([
      [from, monItem(createMon(MonKind.Mystic, Color.White, 0))],
      [targetAt, monItem(createMon(MonKind.Drainer, Color.Black, 0))],
      [targetBase, monItem(createMon(MonKind.Angel, Color.Black, 0))],
    ]);
    const inputs = moveInputs(from, targetAt);
    const position = new FastPosition();
    expect(tryLoadPosition(position, game, 1)).toBe(true);
    expectFastPositionInvariants(position);
    const move = generatedMoves(game).get(inputArrayFen(inputs));
    expect(move).toBeDefined();
    if (move === undefined) return;

    expect(applyFastMoveAndCheckRepresentability(position, move)).toBe(true);
    expectFastPositionInvariants(position);

    const canonical = game.fork();
    expect(canonical.processInput(inputs, false, false).kind).toBe("events");
    const expected = new FastPosition();
    loadPosition(expected, canonical);
    expectFastPositionInvariants(expected);
    expect(fastPositionSnapshot(position)).toEqual(
      fastPositionSnapshot(expected),
    );
  });
});
