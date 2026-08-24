import { describe, expect, it } from "vitest";

import { moveToInputs, tryLoadPosition } from "../../src/automove/bridge.js";
import { cellCooldown } from "../../src/automove/board.js";
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

describe("fast packed-state compatibility", () => {
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

    const position = loadedPosition(game);
    expect(applyFastMove(position, move)).toBe(FAST_MOVE_UNREPRESENTABLE);
    const canonical = game.fork();
    expect(canonical.processInput(inputs, false, false).kind).toBe("events");
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

    const position = loadedPosition(game);
    expectRepresentable(position, move);
    const canonical = game.fork();
    expect(canonical.processInput(inputs, false, false).kind).toBe("events");
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
