import { describe, expect, it } from "vitest";

import { moveToInputs, tryLoadPosition } from "../../src/automove/bridge.js";
import {
  COLOR_COUNT,
  MANA_BLACK,
  MANA_WHITE,
  MON_KIND_COUNT,
  cellCooldown,
  makeManaCell,
  makeMonCell,
} from "../../src/automove/board.js";
import { MAX_MOVES, generateMoves } from "../../src/automove/moves.js";
import {
  FAST_MOVE_UNREPRESENTABLE,
  FastPosition,
  applyFastMove,
} from "../../src/automove/state.js";
import {
  Color,
  Consumable,
  MonKind,
  createMon,
  isMonFainted,
  itemMon,
  manaItem,
  monItem,
  monWithConsumableItem,
  monWithManaItem,
  regularMana,
  type Input,
  type Item,
} from "../../src/engine/model/domain.js";
import { inputArrayFen } from "../../src/engine/codec/input.js";
import { MonsGame } from "../../src/engine/game/mons-game.js";
import {
  BOARD_CELLS,
  BOARD_SIZE,
  location,
  locationIndex,
  type Location,
} from "../../src/engine/board/geometry.js";
import {
  expectFastPositionInvariants,
  fastPositionSnapshot,
  gameWith,
  locationInput,
} from "./test-helper.js";

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

function loadedPosition(game: MonsGame, maxDepth = 40): FastPosition {
  const position = new FastPosition();
  expect(tryLoadPosition(position, game, maxDepth)).toBe(true);
  expectFastPositionInvariants(position);
  return position;
}

function generatedMoves(game: MonsGame): Map<string, number> {
  const position = loadedPosition(game);
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

function expectRepresentable(position: FastPosition, move: number): void {
  expect(applyFastMove(position, move)).not.toBe(FAST_MOVE_UNREPRESENTABLE);
  expectFastPositionInvariants(position);
}

function expectPackedStorageLayout(position: FastPosition): void {
  const cells = position.cells as Uint16Array;
  const views = [
    cells,
    position.monLocations as Int32Array,
    position.freeMana as Int32Array,
    position.manaIndices as Int32Array,
    position.potions as Int32Array,
  ];
  const cellEnd = BOARD_CELLS * Uint16Array.BYTES_PER_ELEMENT;
  const monOffset =
    Math.ceil(cellEnd / Int32Array.BYTES_PER_ELEMENT) * Int32Array.BYTES_PER_ELEMENT;
  const freeManaOffset =
    monOffset + COLOR_COUNT * MON_KIND_COUNT * Int32Array.BYTES_PER_ELEMENT;
  const manaIndicesOffset = freeManaOffset + COLOR_COUNT * Int32Array.BYTES_PER_ELEMENT;
  const potionsOffset =
    manaIndicesOffset + position.manaIndices.length * Int32Array.BYTES_PER_ELEMENT;
  const storageEnd = potionsOffset + COLOR_COUNT * Int32Array.BYTES_PER_ELEMENT;

  expect(views.every((view) => view.buffer === cells.buffer)).toBe(true);
  expect(
    views.map((view) => [view.byteOffset, view.byteOffset + view.byteLength]),
  ).toEqual([
    [0, cellEnd],
    [monOffset, freeManaOffset],
    [freeManaOffset, manaIndicesOffset],
    [manaIndicesOffset, potionsOffset],
    [potionsOffset, storageEnd],
  ]);
  expect(views.every((view) => view.byteOffset % view.BYTES_PER_ELEMENT === 0)).toBe(
    true,
  );
  for (let index = 1; index < views.length; index += 1) {
    const previous = views[index - 1];
    const current = views[index];
    if (previous === undefined || current === undefined) {
      throw new RangeError("packed view layout is incomplete");
    }
    expect(previous.byteOffset + previous.byteLength).toBeLessThanOrEqual(
      current.byteOffset,
    );
  }
  expect(cells.buffer.byteLength).toBe(storageEnd);
}

describe("fast packed-state compatibility", () => {
  it("copies every packed position field into independent storage", () => {
    const source = new FastPosition();
    const cells = new Uint16Array(BOARD_CELLS);
    for (let kind = 0; kind < MON_KIND_COUNT; kind += 1) {
      for (let color = 0; color < COLOR_COUNT; color += 1) {
        const index = kind * COLOR_COUNT + color;
        cells[index] = makeMonCell(kind, color, index % 3, 0, 0);
      }
    }
    for (let index = 0; index < source.manaIndices.length; index += 1) {
      cells[20 + index] = makeManaCell(index % 2 === 0 ? MANA_WHITE : MANA_BLACK);
    }
    source.reset({
      cells,
      squares: new Uint8Array(BOARD_CELLS).fill(1),
      whiteScore: 3,
      blackScore: 2,
      active: 1,
      monsMoves: 1,
      manaMoves: 1,
      actionsUsed: 1,
      potions: new Int32Array([7, 11]),
      firstTurn: true,
    });
    const expected = fastPositionSnapshot(source);
    const destination = new FastPosition();

    destination.copyFrom(source);

    expect(fastPositionSnapshot(destination)).toEqual(expected);
    expectPackedStorageLayout(source);
    expectPackedStorageLayout(destination);
    expect((destination.cells as Uint16Array).buffer).not.toBe(
      (source.cells as Uint16Array).buffer,
    );
    expect(destination.cells).not.toBe(source.cells);
    expect(destination.monLocations).not.toBe(source.monLocations);
    expect(destination.freeMana).not.toBe(source.freeMana);
    expect(destination.manaIndices).not.toBe(source.manaIndices);
    expect(destination.potions).not.toBe(source.potions);
    expect(destination.squares).toBe(source.squares);

    source.reset({
      cells: new Uint16Array(BOARD_CELLS),
      squares: new Uint8Array(BOARD_CELLS),
      whiteScore: 0,
      blackScore: 0,
      active: 0,
      monsMoves: 0,
      manaMoves: 0,
      actionsUsed: 0,
      potions: new Int32Array(COLOR_COUNT),
      firstTurn: false,
    });
    expect(fastPositionSnapshot(destination)).toEqual(expected);
  });

  it("rejects unsupported loads without mutating the destination", () => {
    const position = loadedPosition(new MonsGame(false), 1);
    const before = fastPositionSnapshot(position);
    const squaresBefore = position.squares;
    const duplicateMon = createMon(MonKind.Demon, Color.White, 0);
    const unsupported = [
      ["negative depth", new MonsGame(false), -1],
      ["fractional depth", new MonsGame(false), 1.5],
      [
        "duplicate mon",
        gameWith([
          [location(5, 3), monItem(duplicateMon)],
          [location(5, 4), monWithConsumableItem(duplicateMon, Consumable.Bomb)],
        ]),
        1,
      ],
      ["loose mana capacity", gameWith(manaEntries(17)), 1],
      [
        "total mana capacity",
        gameWith([
          ...manaEntries(16),
          [
            location(5, 5),
            monWithManaItem(
              createMon(MonKind.Drainer, Color.White, 0),
              regularMana(Color.White),
            ),
          ],
        ]),
        1,
      ],
      ["white potion headroom", gameWith([], { whitePotionsCount: INT32_MAX - 1 }), 2],
      ["black potion headroom", gameWith([], { blackPotionsCount: INT32_MAX - 1 }), 2],
      ["turn headroom", gameWith([], { turnNumber: Number.MAX_SAFE_INTEGER - 1 }), 2],
    ] as const;

    for (const [name, game, maxDepth] of unsupported) {
      expect(tryLoadPosition(position, game, maxDepth), name).toBe(false);
      expect(fastPositionSnapshot(position), name).toEqual(before);
      expect(position.squares, name).toBe(squaresBefore);
    }
  });

  it("accepts exact packed counter and mana boundaries", () => {
    const maxDepth = 2;
    const game = gameWith(manaEntries(16), {
      whitePotionsCount: INT32_MAX - maxDepth,
      blackPotionsCount: INT32_MAX - maxDepth,
      turnNumber: Number.MAX_SAFE_INTEGER - maxDepth,
    });
    const position = loadedPosition(game, maxDepth);

    expect(position.manaIndices.length).toBe(16);
    expect(position.manaCount).toBe(16);
    expect(position.potions[0]).toBe(INT32_MAX - maxDepth);
    expect(position.potions[1]).toBe(INT32_MAX - maxDepth);
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
        [stepAt, { kind: "consumable", consumable }],
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

      const position = loadedPosition(game);
      expectRepresentable(position, move);
      expect(fastPositionSnapshot(position), consumable).toEqual(
        fastPositionSnapshot(loadedPosition(canonical)),
      );
    }
  });

  it.each([
    {
      name: "supermana base",
      from: location(5, 3),
      targetAt: location(5, 5),
      formerStepAt: location(5, 6),
    },
    {
      name: "defender's own mon base",
      from: location(2, 5),
      targetAt: location(0, 5),
      formerStepAt: location(1, 5),
    },
  ])("cancels a bomb-fainted Demon step from the $name", (testCase) => {
    const demon = createMon(MonKind.Demon, Color.White, 0);
    const defender = createMon(MonKind.Drainer, Color.Black, 0);
    const game = gameWith([
      [testCase.from, monItem(demon)],
      [testCase.targetAt, monWithConsumableItem(defender, Consumable.Bomb)],
    ]);
    const inputs = [locationInput(testCase.from), locationInput(testCase.targetAt)];
    const formerInputs = [...inputs, locationInput(testCase.formerStepAt)];

    const moves = generatedMoves(game);
    const move = moves.get(inputArrayFen(inputs));
    expect(move).toBeDefined();
    expect(moves.has(inputArrayFen(formerInputs))).toBe(false);
    if (move === undefined) return;

    expect(game.fork().processInput(formerInputs, false, false).kind).toBe(
      "invalid-input",
    );
    const canonical = game.fork();
    const output = canonical.processInput(inputs, false, false);
    expect(output.kind).toBe("events");
    if (output.kind !== "events") return;
    expect(output.events.slice(0, 4).map((event) => event.kind)).toEqual([
      "demon-action",
      "mon-fainted",
      "bomb-explosion",
      "mon-fainted",
    ]);
    expect(output.events.some((event) => event.kind === "demon-additional-step")).toBe(
      false,
    );

    const demonAtBase = canonical.board.get(canonical.board.base(demon));
    const defenderAtBase = canonical.board.get(canonical.board.base(defender));
    const demonAtBaseMon = demonAtBase === undefined ? undefined : itemMon(demonAtBase);
    const defenderAtBaseMon =
      defenderAtBase === undefined ? undefined : itemMon(defenderAtBase);
    expect(demonAtBaseMon === undefined ? false : isMonFainted(demonAtBaseMon)).toBe(
      true,
    );
    expect(
      defenderAtBaseMon === undefined ? false : isMonFainted(defenderAtBaseMon),
    ).toBe(true);
    expect(
      canonical.board.items.filter((item) => {
        const mon = item === undefined ? undefined : itemMon(item);
        return mon?.kind === demon.kind && mon.color === demon.color;
      }),
    ).toHaveLength(1);

    const position = loadedPosition(game);
    expectRepresentable(position, move);
    expect(fastPositionSnapshot(position)).toEqual(
      fastPositionSnapshot(loadedPosition(canonical, 1)),
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
        expect(generatedMoves(game).has(inputArrayFen(inputs)), label).toBe(false);
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

      expect(generatedMoves(game).has(inputArrayFen(inputs)), attack.name).toBe(true);
      expect(game.fork().processInput(inputs, false, false).kind, attack.name).toBe(
        "events",
      );
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
    const move = generatedMoves(game).get(inputArrayFen(inputs));
    expect(move).toBeDefined();
    if (move === undefined) return;

    const position = loadedPosition(game, 1);
    expectRepresentable(position, move);
    expect(cellCooldown(position.cells[locationIndex(plainAt)] ?? 0)).toBe(1);
    expect(cellCooldown(position.cells[locationIndex(manaCarrierAt)] ?? 0)).toBe(2);
    expect(cellCooldown(position.cells[locationIndex(consumableCarrierAt)] ?? 0)).toBe(
      2,
    );

    const canonical = game.fork();
    expect(canonical.processInput(inputs, false, false).kind).toBe("events");
    expect(fastPositionSnapshot(position)).toEqual(
      fastPositionSnapshot(loadedPosition(canonical, 1)),
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
    const move = generatedMoves(game).get(inputArrayFen(inputs));
    expect(move).toBeDefined();
    if (move === undefined) return;

    const position = loadedPosition(game, 1);
    expectRepresentable(position, move);
    const canonical = game.fork();
    expect(canonical.processInput(inputs, false, false).kind).toBe("events");
    expect(fastPositionSnapshot(position)).toEqual(
      fastPositionSnapshot(loadedPosition(canonical, 1)),
    );
  });
});
