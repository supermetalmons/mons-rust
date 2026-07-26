import { describe, expect, it } from "vitest";

import { MutableBoard } from "../../src/engine/board.js";
import { GameVariant } from "../../src/engine/config.js";
import {
  Color,
  Consumable,
  Modifier,
  MonKind,
  NextInputKind,
  consumableItem,
  createMon,
  manaItem,
  monItem,
  monWithManaItem,
  regularMana,
  type Event,
  type Input,
  type Item,
  type NextInput,
  type Output,
} from "../../src/engine/domain.js";
import {
  gameFen,
  parseGameFen,
  type GameFenState,
} from "../../src/engine/fen.js";
import { MonsGame } from "../../src/engine/game.js";
import {
  BOARD_CELLS,
  location,
  locationIndex,
  type Location,
} from "../../src/engine/geometry.js";
import {
  RulesQueryCache,
  type InputStageResult,
} from "../../src/engine/query-cache.js";

function replaceItems(
  game: MonsGame,
  items: readonly (readonly [Location, Item])[],
): void {
  game.replaceBoardItems(items);
}

function withFenState(
  game: MonsGame,
  replacement: Partial<Omit<GameFenState, "board">>,
): MonsGame {
  const state = parseGameFen(game.fen());
  if (state === undefined) {
    throw new Error("test game must have a parseable FEN");
  }
  return MonsGame.newSimulationState({ ...state, ...replacement });
}

function demonAdditionalStepGame(): {
  readonly game: MonsGame;
  readonly pair: readonly Input[];
  readonly chain: readonly Input[];
} {
  const start = location(5, 3);
  const target = location(5, 5);
  const destination = location(5, 6);
  const demon = createMon(MonKind.Demon, Color.White, 0);
  const defender = createMon(MonKind.Mystic, Color.Black, 0);
  let game = new MonsGame(false, GameVariant.Classic);
  replaceItems(game, [
    [start, monItem(demon)],
    [target, monWithManaItem(defender, regularMana(Color.Black))],
  ]);
  game = withFenState(game, { turnNumber: 2 });

  const pair: readonly Input[] = [
    { kind: "location", location: start },
    { kind: "location", location: target },
  ];
  return {
    game,
    pair,
    chain: [...pair, { kind: "location", location: destination }],
  };
}

function demonConsumableSelectionGame(): {
  readonly game: MonsGame;
  readonly chain: readonly Input[];
} {
  const scenario = demonAdditionalStepGame();
  const destination = scenario.chain[2];
  if (destination?.kind !== "location") {
    throw new Error("demon scenario must end at a location");
  }
  scenario.game.replaceBoardItems([
    ...scenario.game.board.entries(),
    [destination.location, consumableItem(Consumable.BombOrPotion)],
  ]);
  return { game: scenario.game, chain: scenario.chain };
}

function expectSameQueryResult(
  warm: MonsGame,
  coldSource: MonsGame,
  input: readonly Input[],
): Output {
  const warmOutput = warm.processInput(input, true, false);
  const cold = coldSource.fork();
  cold.invalidateProcessInputCache();
  const coldOutput = cold.processInput(input, true, false);
  expect(warmOutput).toEqual(coldOutput);
  expect(warm.fen()).toBe(cold.fen());
  return warmOutput;
}

function mutableNextInputs(output: Output, stage: string): NextInput[] {
  expect(output.kind).toBe("next-input-options");
  if (output.kind !== "next-input-options") {
    throw new Error(`${stage} must produce next-input options`);
  }
  if (output.nextInputs.length === 0) {
    throw new Error(`${stage} must produce at least one next-input option`);
  }
  return output.nextInputs as NextInput[];
}

function mutableEvents(output: Output, stage: string): Event[] {
  expect(output.kind).toBe("events");
  if (output.kind !== "events") {
    throw new Error(`${stage} must produce events`);
  }
  if (output.events.length === 0) {
    throw new Error(`${stage} must produce at least one event`);
  }
  return output.events as Event[];
}

describe("RulesQueryCache structural copies", () => {
  it("detaches start suggestions on insertion and lookup", () => {
    const cache = new RulesQueryCache();
    const originalLocation = { i: 4, j: 3 };
    const original: Output = {
      kind: "locations-to-start-from",
      locations: [originalLocation],
    };
    const expected: Output = {
      kind: "locations-to-start-from",
      locations: [{ i: 4, j: 3 }],
    };

    cache.setStartSuggestion("start", original);
    originalLocation.i = 99;
    expect(Reflect.set(original, "locations", [])).toBe(true);

    const firstHit = cache.getStartSuggestion("start");
    expect(firstHit).toEqual(expected);
    if (firstHit?.kind !== "locations-to-start-from") {
      throw new Error("start suggestion must contain locations");
    }
    const firstLocation = firstHit.locations[0];
    if (firstLocation === undefined) {
      throw new Error("start suggestion must contain one location");
    }
    expect(Reflect.set(firstLocation, "j", 99)).toBe(true);
    expect(Reflect.set(firstHit, "locations", [])).toBe(true);

    expect(cache.getStartSuggestion("start")).toEqual(expected);
  });

  it("detaches second-input options while preserving optional actor shape", () => {
    const cache = new RulesQueryCache();
    const originalLocation = { i: 5, j: 3 };
    const mutableActorMon = {
      kind: MonKind.Demon,
      color: Color.White,
      cooldown: 0,
    };
    const withActor: NextInput = {
      kind: NextInputKind.DemonAction,
      input: { kind: "location", location: originalLocation },
      actorMonItem: { kind: "mon", mon: mutableActorMon },
    };
    const withoutActor: NextInput = {
      kind: NextInputKind.SelectConsumable,
      input: { kind: "modifier", modifier: Modifier.SelectBomb },
    };
    const expected: readonly NextInput[] = [
      {
        kind: NextInputKind.DemonAction,
        input: { kind: "location", location: { i: 5, j: 3 } },
        actorMonItem: monItem(createMon(MonKind.Demon, Color.White, 0)),
      },
      {
        kind: NextInputKind.SelectConsumable,
        input: { kind: "modifier", modifier: Modifier.SelectBomb },
      },
    ];

    cache.setSecondInputOptions("second-options", [withActor, withoutActor]);
    originalLocation.i = 99;
    mutableActorMon.cooldown = 2;
    expect(Reflect.set(withActor, "input", { kind: "takeback" })).toBe(true);

    const firstHit = cache.getSecondInputOptions("second-options");
    expect(firstHit).toEqual(expected);
    if (firstHit === undefined) {
      throw new Error("second-input cache must contain options");
    }
    const firstOption = firstHit[0];
    const secondOption = firstHit[1];
    if (firstOption?.input.kind !== "location" || secondOption === undefined) {
      throw new Error("second-input cache must contain the expected options");
    }
    expect(Object.isFrozen(firstOption.actorMonItem)).toBe(true);
    expect(
      firstOption.actorMonItem?.kind === "mon" &&
        Object.isFrozen(firstOption.actorMonItem.mon),
    ).toBe(true);
    expect("actorMonItem" in secondOption).toBe(false);
    expect(Reflect.set(firstOption.input.location, "j", 99)).toBe(true);
    expect(Reflect.set(firstOption, "input", { kind: "takeback" })).toBe(true);
    firstHit.length = 0;

    expect(cache.getSecondInputOptions("second-options")).toEqual(expected);
  });

  it("detaches second- and third-stage events and options in both directions", () => {
    const cache = new RulesQueryCache();
    const originalFrom = { i: 4, j: 4 };
    const originalTo = { i: 5, j: 4 };
    const originalOptionLocation = { i: 6, j: 4 };
    const mutableMana = { kind: "regular", color: Color.White } as const;
    const originalEvent: Event = {
      kind: "mana-move",
      mana: mutableMana,
      from: originalFrom,
      to: originalTo,
    };
    const originalOption: NextInput = {
      kind: NextInputKind.ManaMove,
      input: { kind: "location", location: originalOptionLocation },
    };
    const originalResult: Exclude<InputStageResult, undefined> = [
      [originalEvent],
      [originalOption],
    ];
    const expected: Exclude<InputStageResult, undefined> = [
      [
        {
          kind: "mana-move",
          mana: regularMana(Color.White),
          from: { i: 4, j: 4 },
          to: { i: 5, j: 4 },
        },
      ],
      [
        {
          kind: NextInputKind.ManaMove,
          input: { kind: "location", location: { i: 6, j: 4 } },
        },
      ],
    ];

    cache.setSecondStage("second-stage", originalResult);
    cache.setThirdStage("third-stage", originalResult);
    originalFrom.i = 99;
    originalOptionLocation.i = 99;
    expect(Reflect.set(mutableMana, "color", Color.Black)).toBe(true);
    expect(Reflect.set(originalEvent, "to", { i: 99, j: 99 })).toBe(true);
    expect(Reflect.set(originalOption, "input", { kind: "takeback" })).toBe(
      true,
    );

    const stages = [
      {
        get: (): InputStageResult => cache.getSecondStage("second-stage"),
      },
      {
        get: (): InputStageResult => cache.getThirdStage("third-stage"),
      },
    ];
    for (const stage of stages) {
      const firstHit = stage.get();
      expect(firstHit).toEqual(expected);
      if (firstHit === undefined) {
        throw new Error("cached stage must be present");
      }
      const event = firstHit[0][0];
      const option = firstHit[1][0];
      if (event?.kind !== "mana-move" || option?.input.kind !== "location") {
        throw new Error("cached stage must contain the expected values");
      }
      expect(Object.isFrozen(event.mana)).toBe(true);
      expect(Reflect.set(event.from, "i", 99)).toBe(true);
      expect(Reflect.set(event, "to", { i: 99, j: 99 })).toBe(true);
      expect(Reflect.set(option.input.location, "i", 99)).toBe(true);
      expect(Reflect.set(option, "input", { kind: "takeback" })).toBe(true);

      expect(stage.get()).toEqual(expected);
    }

    cache.setSecondStage("undefined-second", undefined);
    cache.setThirdStage("undefined-third", undefined);
    expect(cache.hasSecondStage("undefined-second")).toBe(true);
    expect(cache.getSecondStage("undefined-second")).toBeUndefined();
    expect(cache.hasThirdStage("undefined-third")).toBe(true);
    expect(cache.getThirdStage("undefined-third")).toBeUndefined();
  });
});

describe("MonsGame staged-input caches", () => {
  it("returns identical values from warm and cold second- and third-stage caches", () => {
    const { game, pair, chain } = demonAdditionalStepGame();
    const warm = game.fork();

    expectSameQueryResult(warm, game, []);
    expectSameQueryResult(warm, game, pair.slice(0, 1));
    expectSameQueryResult(warm, game, pair);
    expectSameQueryResult(warm, game, chain);

    // Repeat the staged queries so both stage caches serve hits.
    expectSameQueryResult(warm, game, pair);
    expectSameQueryResult(warm, game, chain);
  });

  it("applies a move from warmed stage caches identically to a cold state", () => {
    const { game, chain } = demonAdditionalStepGame();
    const warm = game.fork();
    const cold = game.fork();
    cold.invalidateProcessInputCache();

    mutableEvents(
      warm.processInput(chain, true, false),
      "warmed third-stage query",
    );

    const warmOutput = warm.processInput(chain, false, false);
    const coldOutput = cold.processInput(chain, false, false);

    expect(warmOutput).toEqual(coldOutput);
    expect(warm.fen()).toBe(cold.fen());
    expect(warm.fen()).not.toBe(game.fen());
  });

  it("returns independent second- and third-stage result arrays", () => {
    const { game, pair, chain } = demonAdditionalStepGame();
    const warm = game.fork();
    const expectedPair = game.fork().processInput(pair, true, false);
    const expectedChain = game.fork().processInput(chain, true, false);

    const pairOutput = warm.processInput(pair, true, false);
    const pairOptions = mutableNextInputs(pairOutput, "second stage");
    pairOptions.length = 0;
    expect(warm.processInput(pair, true, false)).toEqual(expectedPair);

    const chainOutput = warm.processInput(chain, true, false);
    const chainEvents = mutableEvents(chainOutput, "third stage");
    chainEvents.length = 0;
    expect(warm.processInput(chain, true, false)).toEqual(expectedChain);
  });

  it("returns an independent third-stage option array", () => {
    const { game, chain } = demonConsumableSelectionGame();
    const warm = game.fork();
    const expected = game.fork().processInput(chain, true, false);

    const output = warm.processInput(chain, true, false);
    const options = mutableNextInputs(output, "third-stage consumable");
    options.length = 0;

    expect(warm.processInput(chain, true, false)).toEqual(expected);
  });

  it("does not advertise a consumable choice that would overflow potions", () => {
    const scenario = demonConsumableSelectionGame();
    const game = withFenState(scenario.game, {
      whitePotionsCount: Number.MAX_SAFE_INTEGER,
    });

    const output = game.processInput(scenario.chain, true, false);
    expect(output.kind).toBe("next-input-options");
    if (output.kind !== "next-input-options") return;
    expect(output.nextInputs.map((option) => option.input)).toEqual([
      { kind: "modifier", modifier: Modifier.SelectBomb },
    ]);

    const before = game.fen();
    expect(
      game.processInput(
        [
          ...scenario.chain,
          { kind: "modifier", modifier: Modifier.SelectPotion },
        ],
        false,
        false,
      ),
    ).toEqual({ kind: "invalid-input" });
    expect(game.fen()).toBe(before);
  });
});

describe("MonsGame serialized state copying", () => {
  it("keeps board, tracking, and history ownership explicit across copies", () => {
    const initial = new MonsGame(false, GameVariant.OffsetArcManaRows);
    const removedAt = location(0, 3);
    const state = parseGameFen(initial.fen());
    if (state === undefined) {
      throw new Error("initial game must have a parseable FEN");
    }
    const board = state.board.fork();
    board.delete(removedAt);
    const source = MonsGame.fromFen(
      gameFen({
        ...state,
        board,
        whiteScore: 3,
        blackScore: 2,
        activeColor: Color.Black,
        actionsUsedCount: 1,
        manaMovesCount: 1,
        monsMovesCount: 4,
        whitePotionsCount: 2,
        blackPotionsCount: 3,
        turnNumber: 8,
      }),
      true,
    );
    if (source === undefined) {
      throw new Error("custom state must produce a valid game");
    }
    source.replaceHistory({
      takebackFens: ["before", source.fen()],
      movesVerified: true,
      trackingEntries: [
        {
          fen: source.fen(),
          color: source.activeColor,
          events: [
            {
              kind: "mana-move",
              mana: regularMana(Color.Black),
              from: location(3, 4),
              to: location(4, 4),
            },
          ],
        },
      ],
    });
    expect(source.canTakeback(source.activeColor)).toBe(true);

    const copy = source.copy();
    expect(copy.fen()).toBe(source.fen());
    expect(copy.variant()).toBe(source.variant());
    expect(copy.withVerboseTracking).toBe(true);
    expect(copy.isMovesVerified).toBe(true);
    expect(copy.takebackFens).toEqual(source.takebackFens);
    expect(copy.takebackFens).not.toBe(source.takebackFens);
    expect(copy.verboseTrackingEntities).toEqual(
      source.verboseTrackingEntities,
    );
    expect(copy.verboseTrackingEntities).not.toBe(
      source.verboseTrackingEntities,
    );
    expect(copy.verboseTrackingEntities[0]?.events).not.toBe(
      source.verboseTrackingEntities[0]?.events,
    );
    copy.replaceBoardItems([
      ...copy.board.entries(),
      [removedAt, manaItem(regularMana(Color.White))],
    ]);
    expect(source.board.get(removedAt)).toBeUndefined();

    const simulation = source.fork();
    expect(simulation.fen()).toBe(source.fen());
    expect(simulation.variant()).toBe(source.variant());
    expect(simulation.isMovesVerified).toBe(source.isMovesVerified);
    expect(simulation.withVerboseTracking).toBe(false);
    expect(simulation.takebackFens).toEqual([]);
    expect(simulation.verboseTrackingEntities).toEqual([]);
    expect(simulation.canTakeback(source.activeColor)).toBe(false);
    simulation.replaceBoardItems([
      ...simulation.board.entries(),
      [removedAt, manaItem(regularMana(Color.White))],
    ]);
    expect(source.board.get(removedAt)).toBeUndefined();

    const parsedState = parseGameFen(source.fen());
    expect(parsedState).toBeDefined();
    if (parsedState === undefined) {
      return;
    }
    const hydratedSimulation = MonsGame.newSimulationState(parsedState);
    expect(hydratedSimulation.fen()).toBe(source.fen());
    expect(hydratedSimulation.canTakeback(source.activeColor)).toBe(false);

    const restored = MonsGame.fromFen(source.fen(), true);
    expect(restored).toBeDefined();
    expect(restored?.fen()).toBe(source.fen());
    expect(restored?.withVerboseTracking).toBe(true);
    expect(restored?.takebackFens).toEqual([]);
    expect(restored?.verboseTrackingEntities).toEqual([]);
    expect(restored?.isMovesVerified).toBe(false);
  });
});

describe("MonsGame board ownership", () => {
  it("exposes a live readonly view and independently mutable forks", () => {
    const game = new MonsGame(false, GameVariant.Classic);
    const boardView = game.board;
    const removedAt = location(0, 3);

    expect("set" in boardView).toBe(false);
    expect("delete" in boardView).toBe(false);

    const fork = boardView.fork();
    expect(typeof fork.set).toBe("function");
    expect(typeof fork.delete).toBe("function");
    fork.delete(removedAt);

    expect(fork.get(removedAt)).toBeUndefined();
    expect(boardView.get(removedAt)).toBeDefined();
    expect(game.board).toBe(boardView);

    const movedFrom = location(10, 5);
    const movedTo = location(9, 4);
    expect(
      game.processInput(
        [
          { kind: "location", location: movedFrom },
          { kind: "location", location: movedTo },
        ],
        false,
        false,
      ).kind,
    ).toBe("events");
    expect(boardView.get(movedFrom)).toBeUndefined();
    expect(boardView.get(movedTo)).toBeDefined();
    expect(game.board).toBe(boardView);
  });

  it("forks a supplied board before warming any game query caches", () => {
    const supplied = new MutableBoard(GameVariant.Classic);
    const removedAt = location(10, 3);
    const game = new MonsGame(false, GameVariant.Classic, supplied);
    const fenBefore = game.fen();
    const suggestionBefore = game.processInput([], true, false);

    supplied.delete(removedAt);

    expect(game.fen()).toBe(fenBefore);
    expect(game.board.get(removedAt)).toBeDefined();
    expect(game.processInput([], true, false)).toEqual(suggestionBefore);
  });

  it("normalizes supplied item values before sharing them between forks", () => {
    const at = location(10, 5);
    const mutableMon = {
      kind: MonKind.Drainer,
      color: Color.White,
      cooldown: 0,
    };
    const mutableItem: Item = { kind: "mon", mon: mutableMon };
    const slots = Array.from(
      { length: BOARD_CELLS },
      (): Item | undefined => undefined,
    );
    slots[locationIndex(at)] = mutableItem;
    const supplied = MutableBoard.fromItems(slots, GameVariant.Classic);
    const game = new MonsGame(false, GameVariant.Classic, supplied);
    const fenBefore = game.fen();
    const stored = game.board.get(at);

    expect(Object.isFrozen(stored)).toBe(true);
    expect(stored?.kind === "mon" && Object.isFrozen(stored.mon)).toBe(true);
    expect(Reflect.set(mutableMon, "cooldown", 2)).toBe(true);
    expect(
      stored?.kind === "mon" && Reflect.set(stored.mon, "cooldown", 2),
    ).toBe(false);
    expect(game.fen()).toBe(fenBefore);
    expect(game.board.get(at)).toEqual(
      monItem(createMon(MonKind.Drainer, Color.White, 0)),
    );
  });
});
