import { Board, MutableBoard } from "./board.js";
import {
  AvailableMoveKind,
  Color,
  Consumable,
  Modifier,
  MonKind,
  NextInputKind,
  MAX_INPUTS_PER_MOVE,
  inputEquals,
  inputChainKey,
  inputKey,
  isMonFainted,
  isSpiritTargetAllowed,
  itemConsumable,
  itemKey,
  itemMana,
  itemMon,
  otherColor,
  type Event,
  type Input,
  type Item,
  type NextInput,
  type Output,
  type Square,
} from "./domain.js";
import {
  ACTIONS_PER_TURN,
  DEFAULT_GAME_VARIANT,
  GameVariant,
  MANA_MOVES_PER_TURN,
  MONS_MOVES_PER_TURN,
} from "./config.js";
import { gameFen, parseGameFen, type GameFenState } from "./fen.js";
import {
  applyRulesEvents,
  canApplyRulesEvents,
  type MutableRulesState,
} from "./event-reducer.js";
import {
  BOARD_CELLS,
  bombReachableLocations,
  demonReachableLocations,
  locationBetween,
  locationDistance,
  locationEquals,
  locationIndex,
  mysticReachableLocations,
  nearbyLocations,
  spiritReachableLocations,
  type Location,
} from "./geometry.js";
import { RulesQueryCache, type InputStageResult } from "./query-cache.js";
import {
  GameHistory,
  type HistoryReplacement,
  type VerboseTrackingEntity,
} from "./history.js";
import {
  canPlayerMoveMana,
  canPlayerMoveMon,
  canPlayerUseAction,
  currentPlayerPotions,
  isFirstTurnState,
  shouldSuggestRegularManaStarts,
  spiritDestinationItemAllowed,
  winnerForState,
} from "./legality.js";

export type SuggestedStartInputOptions = {
  readonly includeManaStartsWithPotionAction: boolean;
};

const DEFAULT_SUGGESTED_START_INPUT_OPTIONS: SuggestedStartInputOptions =
  Object.freeze({
    includeManaStartsWithPotionAction: false,
  });

export const FOR_AUTOMOVE_START_INPUT_OPTIONS: SuggestedStartInputOptions =
  Object.freeze({
    includeManaStartsWithPotionAction: true,
  });

function nextInput(
  input: Input,
  kind: NextInputKind,
  actorMonItem?: Item,
): NextInput {
  const base = { input, kind };
  return actorMonItem === undefined ? base : { ...base, actorMonItem };
}

function regularSquareForMovement(square: Square): boolean {
  switch (square.kind) {
    case "regular":
    case "consumable-base":
    case "mana-base":
    case "mana-pool":
      return true;
    case "supermana-base":
    case "mon-base":
      return false;
  }
}

export class MonsGame {
  #state: MutableRulesState;
  #history: GameHistory;
  readonly #queryCache: RulesQueryCache;

  public constructor(
    withVerboseTracking = false,
    variant: GameVariant = DEFAULT_GAME_VARIANT,
    board?: Board,
  ) {
    const ownedBoard =
      board === undefined ? new MutableBoard(variant) : board.fork();
    this.#state = {
      board: ownedBoard,
      whiteScore: 0,
      blackScore: 0,
      activeColor: Color.White,
      actionsUsedCount: 0,
      manaMovesCount: 0,
      monsMovesCount: 0,
      whitePotionsCount: 0,
      blackPotionsCount: 0,
      turnNumber: 1,
    };
    this.#history = new GameHistory(withVerboseTracking);
    this.#queryCache = new RulesQueryCache();
  }

  public get board(): Board {
    return this.#state.board.readonlyView();
  }

  public get whiteScore(): number {
    return this.#state.whiteScore;
  }

  public get blackScore(): number {
    return this.#state.blackScore;
  }

  public get activeColor(): Color {
    return this.#state.activeColor;
  }

  public get actionsUsedCount(): number {
    return this.#state.actionsUsedCount;
  }

  public get manaMovesCount(): number {
    return this.#state.manaMovesCount;
  }

  public get monsMovesCount(): number {
    return this.#state.monsMovesCount;
  }

  public get whitePotionsCount(): number {
    return this.#state.whitePotionsCount;
  }

  public get blackPotionsCount(): number {
    return this.#state.blackPotionsCount;
  }

  public get turnNumber(): number {
    return this.#state.turnNumber;
  }

  public get takebackFens(): readonly string[] {
    return this.#history.takebackFens;
  }

  public get isMovesVerified(): boolean {
    return this.#history.movesVerified;
  }

  public get withVerboseTracking(): boolean {
    return this.#history.verboseTrackingEnabled;
  }

  public get verboseTrackingEntities(): readonly VerboseTrackingEntity[] {
    return this.#history.trackingEntries;
  }

  public replaceHistory(replacement: HistoryReplacement): void {
    this.#history.replace(replacement);
  }

  /**
   * Copy only the scalar state represented by game FEN. Callers keep board,
   * history, tracking, and cache ownership explicit.
   */
  #copyFenFieldsFrom(state: GameFenState): void {
    this.#state.whiteScore = state.whiteScore;
    this.#state.blackScore = state.blackScore;
    this.#state.activeColor = state.activeColor;
    this.#state.actionsUsedCount = state.actionsUsedCount;
    this.#state.manaMovesCount = state.manaMovesCount;
    this.#state.monsMovesCount = state.monsMovesCount;
    this.#state.whitePotionsCount = state.whitePotionsCount;
    this.#state.blackPotionsCount = state.blackPotionsCount;
    this.#state.turnNumber = state.turnNumber;
  }

  static #fromBoard(withVerboseTracking: boolean, board: Board): MonsGame {
    return new MonsGame(withVerboseTracking, board.variant, board);
  }

  public static newSimulationState(state: GameFenState): MonsGame {
    const game = MonsGame.#fromBoard(false, state.board);
    game.#copyFenFieldsFrom(state);
    game.#history.setTakebackTracking(false);
    return game;
  }

  public static fromFen(
    fen: string,
    withVerboseTracking = false,
  ): MonsGame | undefined {
    const state = parseGameFen(fen);
    if (state === undefined) {
      return undefined;
    }
    const game = MonsGame.#fromBoard(withVerboseTracking, state.board);
    game.#copyFenFieldsFrom(state);
    game.#history.resetForLoadedFen();
    return game;
  }

  public fen(): string {
    return gameFen(this);
  }

  public copy(): MonsGame {
    const game = MonsGame.#fromBoard(this.withVerboseTracking, this.board);
    game.#copyFenFieldsFrom(this);
    game.#history = this.#history.copy();
    return game;
  }

  public fork(): MonsGame {
    const simulation = MonsGame.#fromBoard(false, this.board);
    simulation.#copyFenFieldsFrom(this);
    simulation.#history = this.#history.fork();
    return simulation;
  }

  public variant(): GameVariant {
    return this.board.variant;
  }

  public replaceBoardItems(items: Iterable<readonly [Location, Item]>): void {
    const itemArray: (Item | undefined)[] = Array.from(
      { length: BOARD_CELLS },
      () => undefined,
    );
    for (const [at, item] of items) {
      itemArray[locationIndex(at)] = item;
    }
    const board = Board.fromItems(itemArray, this.variant());
    this.#state.board = board;
    this.#history.resetForBoardReplacement();
    this.invalidateProcessInputCache();
  }

  public setTakebackHistoryTracking(enabled: boolean): void {
    this.#history.setTakebackTracking(enabled);
    this.invalidateProcessInputCache();
  }

  public invalidateProcessInputCache(): void {
    this.#queryCache.invalidate();
  }

  public setVerboseTracking(enabled: boolean): void {
    this.#history.setVerboseTracking(enabled);
  }

  public clearTracking(): void {
    this.#history.clearTracking();
    this.invalidateProcessInputCache();
  }

  #updateWith(otherGame: MonsGame): void {
    const board = otherGame.board.fork();
    this.#state.board = board;
    this.#copyFenFieldsFrom(otherGame);
    this.invalidateProcessInputCache();
  }

  public canTakeback(color: Color): boolean {
    return this.#history.canTakeback(this.activeColor, color);
  }

  public processInput(
    input: readonly Input[],
    doNotApplyEvents: boolean,
    oneOptionEnough: boolean,
  ): Output {
    return this.processInputWithStartOptions(
      input,
      doNotApplyEvents,
      oneOptionEnough,
      undefined,
    );
  }

  public processInputWithStartOptions(
    input: readonly Input[],
    doNotApplyEvents: boolean,
    oneOptionEnough: boolean,
    suggestedStartOptions: SuggestedStartInputOptions | undefined,
  ): Output {
    return this.#processInputInternal(
      input,
      doNotApplyEvents,
      oneOptionEnough,
      suggestedStartOptions ?? DEFAULT_SUGGESTED_START_INPUT_OPTIONS,
    );
  }

  /**
   * Resolve only the input grammar, without recursively filtering completions
   * for numeric-capacity failures. Automove's emergency fallback uses this to
   * walk candidates in stable order and stop at the first applicable leaf.
   */
  public inspectInputGrammar(
    input: readonly Input[],
    suggestedStartOptions: SuggestedStartInputOptions,
  ): Output {
    return this.#processInputInternal(
      input,
      true,
      false,
      suggestedStartOptions,
      false,
    );
  }

  #processInputInternal(
    input: readonly Input[],
    doNotApplyEvents: boolean,
    oneOptionEnough: boolean,
    suggestedStartOptions: SuggestedStartInputOptions,
    filterCounterCapacityOptions = true,
  ): Output {
    if (
      input.length > MAX_INPUTS_PER_MOVE ||
      this.winnerColor() !== undefined
    ) {
      return { kind: "invalid-input" };
    }
    if (input.length === 0) {
      const key = `${filterCounterCapacityOptions ? 1 : 0}:${
        suggestedStartOptions.includeManaStartsWithPotionAction ? 1 : 0
      }`;
      const cached = this.#queryCache.getStartSuggestion(key);
      if (cached !== undefined) {
        return cached;
      }
      const output = this.#suggestedInputToStartWith(
        suggestedStartOptions,
        filterCounterCapacityOptions,
      );
      this.#queryCache.setStartSuggestion(key, output);
      return output;
    }

    const firstInput = input[0];
    if (input.length === 1 && firstInput?.kind === "takeback") {
      const prepared = this.#history.prepareTakeback(
        this.activeColor,
        this.activeColor,
      );
      if (prepared === undefined) {
        return { kind: "invalid-input" };
      }
      const previousGame = MonsGame.fromFen(prepared.previousFen, false);
      if (previousGame === undefined) return { kind: "invalid-input" };
      if (doNotApplyEvents) {
        return { kind: "events", events: [{ kind: "takeback" }] };
      }
      this.#updateWith(previousGame);
      this.#history.commitTakeback(prepared);
      this.invalidateProcessInputCache();
      return { kind: "events", events: [{ kind: "takeback" }] };
    }

    if (firstInput?.kind !== "location") {
      return { kind: "invalid-input" };
    }
    const startLocation = firstInput.location;
    const boardStartItem = this.board.get(startLocation);
    if (boardStartItem === undefined) {
      return { kind: "invalid-input" };
    }
    const startItem = boardStartItem;
    const specificSecondInput = input[1];
    const filterCounterCapacity =
      filterCounterCapacityOptions && this.#requiresCounterCapacityFiltering();
    const secondInputOptions = this.#secondInputOptions(
      startLocation,
      startItem,
      oneOptionEnough && !filterCounterCapacity,
      specificSecondInput,
    );

    if (specificSecondInput === undefined) {
      const applicableOptions = this.#applicableInputOptions(
        input,
        secondInputOptions,
        suggestedStartOptions,
        filterCounterCapacity,
        oneOptionEnough,
      );
      return applicableOptions.length === 0
        ? { kind: "invalid-input" }
        : { kind: "next-input-options", nextInputs: applicableOptions };
    }
    if (specificSecondInput.kind !== "location") {
      return { kind: "invalid-input" };
    }
    const targetLocation = specificSecondInput.location;
    const secondOption = secondInputOptions.find((option) =>
      inputEquals(option.input, specificSecondInput),
    );
    if (secondOption === undefined) {
      return { kind: "invalid-input" };
    }

    const specificThirdInput = input[2];
    const secondResult = this.#processSecondInput(
      secondOption.kind,
      startItem,
      startLocation,
      targetLocation,
    );
    const events = secondResult?.[0] ?? [];
    const thirdInputOptions = secondResult?.[1] ?? [];

    if (specificThirdInput === undefined) {
      if (thirdInputOptions.length !== 0) {
        const applicableOptions = this.#applicableInputOptions(
          input,
          thirdInputOptions,
          suggestedStartOptions,
          filterCounterCapacity,
          oneOptionEnough,
        );
        return applicableOptions.length === 0
          ? { kind: "invalid-input" }
          : { kind: "next-input-options", nextInputs: applicableOptions };
      }
      if (events.length !== 0) {
        return this.#resolveEvents(events, doNotApplyEvents);
      }
      return { kind: "invalid-input" };
    }

    const thirdInput = thirdInputOptions.find((option) =>
      inputEquals(option.input, specificThirdInput),
    );
    if (thirdInput === undefined) {
      return { kind: "invalid-input" };
    }
    const specificFourthInput = input[3];
    const thirdResult = this.#processThirdInput(
      thirdInput,
      startItem,
      startLocation,
      targetLocation,
    );
    events.push(...(thirdResult?.[0] ?? []));
    const fourthInputOptions = thirdResult?.[1] ?? [];

    if (specificFourthInput === undefined) {
      if (fourthInputOptions.length !== 0) {
        const applicableOptions = this.#applicableInputOptions(
          input,
          fourthInputOptions,
          suggestedStartOptions,
          filterCounterCapacity,
          oneOptionEnough,
        );
        return applicableOptions.length === 0
          ? { kind: "invalid-input" }
          : { kind: "next-input-options", nextInputs: applicableOptions };
      }
      if (events.length !== 0) {
        return this.#resolveEvents(events, doNotApplyEvents);
      }
      return { kind: "invalid-input" };
    }

    if (
      specificFourthInput.kind !== "modifier" ||
      thirdInput.input.kind !== "location"
    ) {
      return { kind: "invalid-input" };
    }
    const fourthInput = fourthInputOptions.find((option) =>
      inputEquals(option.input, specificFourthInput),
    );
    const actorMonItem = fourthInput?.actorMonItem;
    const actorMon =
      actorMonItem === undefined ? undefined : itemMon(actorMonItem);
    if (actorMonItem === undefined || actorMon === undefined) {
      return { kind: "invalid-input" };
    }
    switch (specificFourthInput.modifier) {
      case Modifier.SelectBomb:
        events.push({
          kind: "pickup-bomb",
          by: actorMon,
          at: thirdInput.input.location,
        });
        break;
      case Modifier.SelectPotion:
        events.push({
          kind: "pickup-potion",
          by: actorMonItem,
          at: thirdInput.input.location,
        });
        break;
    }
    return this.#resolveEvents(events, doNotApplyEvents);
  }

  #requiresCounterCapacityFiltering(): boolean {
    return (
      this.turnNumber === Number.MAX_SAFE_INTEGER ||
      this.whitePotionsCount === Number.MAX_SAFE_INTEGER ||
      this.blackPotionsCount === Number.MAX_SAFE_INTEGER
    );
  }

  #applicableInputOptions(
    prefix: readonly Input[],
    options: NextInput[],
    suggestedStartOptions: SuggestedStartInputOptions,
    required: boolean,
    oneOptionEnough: boolean,
  ): NextInput[] {
    if (!required) return options;
    const applicable = (option: NextInput): boolean =>
      this.#hasApplicableCompletion(
        [...prefix, option.input],
        suggestedStartOptions,
      );
    if (!oneOptionEnough) return options.filter(applicable);
    const first = options.find(applicable);
    return first === undefined ? [] : [first];
  }

  #hasApplicableCompletion(
    inputs: readonly Input[],
    suggestedStartOptions: SuggestedStartInputOptions,
  ): boolean {
    const cacheKey = `${suggestedStartOptions.includeManaStartsWithPotionAction ? 1 : 0}|${inputChainKey(inputs)}`;
    const cached = this.#queryCache.getCompletionViability(cacheKey);
    if (cached !== undefined) return cached;

    const output = this.#processInputInternal(
      inputs,
      true,
      false,
      suggestedStartOptions,
      false,
    );
    const applicable = (() => {
      switch (output.kind) {
        case "invalid-input":
          return false;
        case "events":
          return canApplyRulesEvents(this.#state, output.events);
        case "locations-to-start-from":
          return output.locations.some((at) =>
            this.#hasApplicableCompletion(
              [...inputs, { kind: "location", location: at }],
              suggestedStartOptions,
            ),
          );
        case "next-input-options":
          return output.nextInputs.some((option) =>
            this.#hasApplicableCompletion(
              [...inputs, option.input],
              suggestedStartOptions,
            ),
          );
      }
    })();
    this.#queryCache.setCompletionViability(cacheKey, applicable);
    return applicable;
  }

  #suggestedInputToStartWith(
    suggestedStartOptions: SuggestedStartInputOptions,
    filterCounterCapacityOptions: boolean,
  ): Output {
    const suggestedLocations: Location[] = [];
    const seenLocations = Array.from({ length: BOARD_CELLS }, () => false);

    for (const at of this.board.allMonsLocations(this.activeColor)) {
      const output = this.#processInputInternal(
        [{ kind: "location", location: at }],
        true,
        true,
        suggestedStartOptions,
        filterCounterCapacityOptions,
      );
      if (
        output.kind === "next-input-options" &&
        output.nextInputs.length !== 0
      ) {
        const index = locationIndex(at);
        if (!seenLocations[index]) {
          seenLocations[index] = true;
          suggestedLocations.push(at);
        }
      }
    }

    const shouldAddRegularManaStarts = shouldSuggestRegularManaStarts(
      this,
      suggestedLocations.length !== 0,
      suggestedStartOptions.includeManaStartsWithPotionAction,
    );

    if (shouldAddRegularManaStarts) {
      for (const at of this.board.allFreeRegularManaLocations(
        this.activeColor,
      )) {
        const output = this.#processInputInternal(
          [{ kind: "location", location: at }],
          true,
          true,
          suggestedStartOptions,
          filterCounterCapacityOptions,
        );
        if (
          output.kind === "next-input-options" &&
          output.nextInputs.length !== 0
        ) {
          const index = locationIndex(at);
          if (!seenLocations[index]) {
            seenLocations[index] = true;
            suggestedLocations.push(at);
          }
        }
      }
    }

    return suggestedLocations.length === 0
      ? { kind: "invalid-input" }
      : { kind: "locations-to-start-from", locations: suggestedLocations };
  }

  #secondInputOptions(
    startLocation: Location,
    startItem: Item,
    onlyOne: boolean,
    specificNext: Input | undefined,
  ): NextInput[] {
    const cacheKey = `${locationIndex(startLocation)}|${itemKey(startItem)}|${onlyOne ? 1 : 0}|${
      specificNext === undefined ? "" : inputKey(specificNext)
    }`;
    const cached = this.#queryCache.getSecondInputOptions(cacheKey);
    if (cached !== undefined) {
      return cached;
    }

    const specificLocation =
      specificNext?.kind === "location" ? specificNext.location : undefined;
    const opponentsAngelLocation = this.board.findAwakeAngel(
      otherColor(this.activeColor),
    );
    const startSquare = this.board.squareAt(startLocation);
    const options: NextInput[] = [];

    switch (startItem.kind) {
      case "mon": {
        const mon = startItem.mon;
        if (mon.color !== this.activeColor || isMonFainted(mon)) {
          break;
        }
        if (this.playerCanMoveMon()) {
          options.push(
            ...this.nextInputsFromLocations(
              nearbyLocations(startLocation),
              NextInputKind.MonMove,
              onlyOne,
              specificNext === undefined
                ? undefined
                : specificNext.kind === "location"
                  ? specificNext.location
                  : startLocation,
              (at) => {
                const item = this.board.get(at);
                const square = this.board.squareAt(at);
                let itemAllows: boolean;
                switch (item?.kind) {
                  case "mon":
                  case "mon-with-mana":
                  case "mon-with-consumable":
                    itemAllows = false;
                    break;
                  case "mana":
                    itemAllows = mon.kind === MonKind.Drainer;
                    break;
                  case "consumable":
                  case undefined:
                    itemAllows = true;
                    break;
                }
                if (!itemAllows) {
                  return false;
                }
                switch (square.kind) {
                  case "regular":
                  case "consumable-base":
                  case "mana-base":
                  case "mana-pool":
                    return true;
                  case "supermana-base":
                    return (
                      mon.kind === MonKind.Drainer &&
                      (item === undefined ||
                        (item.kind === "mana" &&
                          item.mana.kind === "supermana"))
                    );
                  case "mon-base":
                    return (
                      square.monKind === mon.kind && square.color === mon.color
                    );
                }
              },
            ),
          );
        }

        if (startSquare.kind !== "mon-base" && this.playerCanUseAction()) {
          switch (mon.kind) {
            case MonKind.Angel:
            case MonKind.Drainer:
              break;
            case MonKind.Mystic:
              options.push(
                ...this.nextInputsFromLocations(
                  mysticReachableLocations(startLocation),
                  NextInputKind.MysticAction,
                  onlyOne,
                  specificLocation,
                  (at) => {
                    const item = this.board.get(at);
                    if (
                      item === undefined ||
                      MonsGame.#isLocationGuardedByAngelLocation(
                        opponentsAngelLocation,
                        at,
                      )
                    ) {
                      return false;
                    }
                    const targetMon = itemMon(item);
                    return (
                      targetMon !== undefined &&
                      mon.color !== targetMon.color &&
                      !isMonFainted(targetMon)
                    );
                  },
                ),
              );
              break;
            case MonKind.Demon:
              options.push(
                ...this.nextInputsFromLocations(
                  demonReachableLocations(startLocation),
                  NextInputKind.DemonAction,
                  onlyOne,
                  specificLocation,
                  (at) => {
                    const item = this.board.get(at);
                    const between = locationBetween(startLocation, at);
                    const betweenSquare = this.board.squareAt(between);
                    if (
                      item === undefined ||
                      MonsGame.#isLocationGuardedByAngelLocation(
                        opponentsAngelLocation,
                        at,
                      ) ||
                      this.board.get(between) !== undefined ||
                      betweenSquare.kind === "supermana-base" ||
                      betweenSquare.kind === "mon-base"
                    ) {
                      return false;
                    }
                    const targetMon = itemMon(item);
                    return (
                      targetMon !== undefined &&
                      mon.color !== targetMon.color &&
                      !isMonFainted(targetMon)
                    );
                  },
                ),
              );
              break;
            case MonKind.Spirit:
              options.push(
                ...this.nextInputsFromLocations(
                  spiritReachableLocations(startLocation),
                  NextInputKind.SpiritTargetCapture,
                  onlyOne,
                  specificLocation,
                  (at) => {
                    const item = this.board.get(at);
                    if (item === undefined) {
                      return false;
                    }
                    return isSpiritTargetAllowed(item);
                  },
                ),
              );
              break;
          }
        }
        break;
      }
      case "mana":
        if (
          startItem.mana.kind === "regular" &&
          startItem.mana.color === this.activeColor &&
          this.playerCanMoveMana()
        ) {
          options.push(
            ...this.nextInputsFromLocations(
              nearbyLocations(startLocation),
              NextInputKind.ManaMove,
              onlyOne,
              specificLocation,
              (at) => {
                const item = this.board.get(at);
                const square = this.board.squareAt(at);
                if (item?.kind === "mon") {
                  return (
                    regularSquareForMovement(square) &&
                    item.mon.kind === MonKind.Drainer
                  );
                }
                if (item !== undefined) {
                  return false;
                }
                return regularSquareForMovement(square);
              },
            ),
          );
        }
        break;
      case "mon-with-mana":
        if (
          startItem.mon.color === this.activeColor &&
          this.playerCanMoveMon()
        ) {
          options.push(
            ...this.nextInputsFromLocations(
              nearbyLocations(startLocation),
              NextInputKind.MonMove,
              onlyOne,
              specificLocation,
              (at) => {
                const item = this.board.get(at);
                const square = this.board.squareAt(at);
                switch (item?.kind) {
                  case "mon":
                  case "mon-with-mana":
                  case "mon-with-consumable":
                    return false;
                  case "mana":
                  case "consumable":
                    return true;
                  case undefined:
                    if (regularSquareForMovement(square)) {
                      return true;
                    }
                    return (
                      square.kind === "supermana-base" &&
                      startItem.mana.kind === "supermana"
                    );
                }
              },
            ),
          );
        }
        break;
      case "mon-with-consumable": {
        const mon = startItem.mon;
        if (mon.color !== this.activeColor) {
          break;
        }
        if (this.playerCanMoveMon()) {
          options.push(
            ...this.nextInputsFromLocations(
              nearbyLocations(startLocation),
              NextInputKind.MonMove,
              onlyOne,
              specificLocation,
              (at) => {
                const item = this.board.get(at);
                const square = this.board.squareAt(at);
                switch (item?.kind) {
                  case "mon":
                  case "mana":
                  case "mon-with-mana":
                  case "mon-with-consumable":
                    return false;
                  case "consumable":
                    return true;
                  case undefined:
                    return regularSquareForMovement(square);
                }
              },
            ),
          );
        }
        if (startItem.consumable === Consumable.Bomb) {
          options.push(
            ...this.nextInputsFromLocations(
              bombReachableLocations(startLocation),
              NextInputKind.BombAttack,
              onlyOne,
              specificLocation,
              (at) => {
                const item = this.board.get(at);
                const targetMon =
                  item === undefined ? undefined : itemMon(item);
                return (
                  targetMon !== undefined &&
                  mon.color !== targetMon.color &&
                  !isMonFainted(targetMon)
                );
              },
            ),
          );
        }
        break;
      }
      case "consumable":
        break;
    }

    this.#queryCache.setSecondInputOptions(cacheKey, options);
    return options;
  }

  #processSecondInput(
    kind: NextInputKind,
    startItem: Item,
    startLocation: Location,
    targetLocation: Location,
  ): InputStageResult {
    const cacheKey = `${kind}|${itemKey(startItem)}|${locationIndex(startLocation)}|${locationIndex(
      targetLocation,
    )}`;
    if (this.#queryCache.hasSecondStage(cacheKey)) {
      return this.#queryCache.getSecondStage(cacheKey);
    }
    const computed = this.#processSecondInputUncached(
      kind,
      startItem,
      startLocation,
      targetLocation,
    );
    this.#queryCache.setSecondStage(cacheKey, computed);
    return computed;
  }

  #processSecondInputUncached(
    kind: NextInputKind,
    startItem: Item,
    startLocation: Location,
    targetLocation: Location,
  ): InputStageResult {
    const thirdInputOptions: NextInput[] = [];
    const events: Event[] = [];
    const targetSquare = this.board.squareAt(targetLocation);
    const targetItem = this.board.get(targetLocation);

    switch (kind) {
      case NextInputKind.MonMove: {
        const movingMon = itemMon(startItem);
        if (movingMon === undefined) {
          return undefined;
        }
        events.push({
          kind: "mon-move",
          item: startItem,
          from: startLocation,
          to: targetLocation,
        });

        if (targetItem !== undefined) {
          switch (targetItem.kind) {
            case "mon":
            case "mon-with-mana":
            case "mon-with-consumable":
              return undefined;
            case "mana": {
              const startMana = itemMana(startItem);
              if (startMana !== undefined) {
                events.push({
                  kind: "mana-dropped",
                  mana: startMana,
                  at: startLocation,
                });
              }
              events.push({
                kind: "pickup-mana",
                mana: targetItem.mana,
                by: movingMon,
                at: targetLocation,
              });
              break;
            }
            case "consumable":
              switch (targetItem.consumable) {
                case Consumable.Bomb:
                case Consumable.Potion:
                  return undefined;
                case Consumable.BombOrPotion:
                  if (
                    itemConsumable(startItem) !== undefined ||
                    itemMana(startItem) !== undefined
                  ) {
                    events.push({
                      kind: "pickup-potion",
                      by: startItem,
                      at: targetLocation,
                    });
                  } else {
                    thirdInputOptions.push(
                      nextInput(
                        { kind: "modifier", modifier: Modifier.SelectBomb },
                        NextInputKind.SelectConsumable,
                        startItem,
                      ),
                    );
                    thirdInputOptions.push(
                      nextInput(
                        { kind: "modifier", modifier: Modifier.SelectPotion },
                        NextInputKind.SelectConsumable,
                        startItem,
                      ),
                    );
                  }
                  break;
              }
              break;
          }
        }

        if (targetSquare.kind === "mana-pool") {
          const manaInHand = itemMana(startItem);
          if (manaInHand !== undefined) {
            events.push({
              kind: "mana-scored",
              mana: manaInHand,
              at: targetLocation,
            });
          }
        }
        break;
      }
      case NextInputKind.ManaMove: {
        if (startItem.kind !== "mana") {
          return undefined;
        }
        const mana = startItem.mana;
        events.push({
          kind: "mana-move",
          mana: mana,
          from: startLocation,
          to: targetLocation,
        });
        if (targetItem !== undefined) {
          if (targetItem.kind !== "mon") {
            return undefined;
          }
          events.push({
            kind: "pickup-mana",
            mana: mana,
            by: targetItem.mon,
            at: targetLocation,
          });
        }
        switch (targetSquare.kind) {
          case "regular":
          case "mana-base":
          case "consumable-base":
            break;
          case "mana-pool":
            events.push({
              kind: "mana-scored",
              mana: mana,
              at: targetLocation,
            });
            break;
          case "mon-base":
          case "supermana-base":
            return undefined;
        }
        break;
      }
      case NextInputKind.MysticAction: {
        if (startItem.kind !== "mon") {
          return undefined;
        }
        events.push({
          kind: "mystic-action",
          mystic: startItem.mon,
          from: startLocation,
          to: targetLocation,
        });
        if (targetItem !== undefined) {
          const targetMon = itemMon(targetItem);
          if (targetMon === undefined) {
            return undefined;
          }
          events.push({
            kind: "mon-fainted",
            mon: targetMon,
            from: targetLocation,
            to: this.board.base(targetMon),
          });
          if (targetItem.kind === "mon-with-mana") {
            if (targetItem.mana.kind === "regular") {
              events.push({
                kind: "mana-dropped",
                mana: targetItem.mana,
                at: targetLocation,
              });
            } else {
              events.push({
                kind: "supermana-back-to-base",
                from: targetLocation,
                to: this.board.supermanaBase(),
              });
            }
          }
          if (targetItem.kind === "mon-with-consumable") {
            switch (targetItem.consumable) {
              case Consumable.Bomb:
                events.push({
                  kind: "bomb-explosion",
                  at: targetLocation,
                });
                break;
              case Consumable.Potion:
              case Consumable.BombOrPotion:
                return undefined;
            }
          }
        }
        break;
      }
      case NextInputKind.DemonAction: {
        if (startItem.kind !== "mon") {
          return undefined;
        }
        const startMon = startItem.mon;
        events.push({
          kind: "demon-action",
          demon: startMon,
          from: startLocation,
          to: targetLocation,
        });
        let requiresAdditionalStep = false;
        if (targetItem !== undefined) {
          const targetMon = itemMon(targetItem);
          if (targetMon === undefined) {
            return undefined;
          }
          events.push({
            kind: "mon-fainted",
            mon: targetMon,
            from: targetLocation,
            to: this.board.base(targetMon),
          });
          if (targetItem.kind === "mon-with-mana") {
            if (targetItem.mana.kind === "regular") {
              requiresAdditionalStep = true;
              events.push({
                kind: "mana-dropped",
                mana: targetItem.mana,
                at: targetLocation,
              });
            } else {
              events.push({
                kind: "supermana-back-to-base",
                from: targetLocation,
                to: this.board.supermanaBase(),
              });
            }
          }
          if (targetItem.kind === "mon-with-consumable") {
            switch (targetItem.consumable) {
              case Consumable.Bomb:
                events.push({
                  kind: "bomb-explosion",
                  at: targetLocation,
                });
                events.push({
                  kind: "mon-fainted",
                  mon: startMon,
                  from: targetLocation,
                  to: this.board.base(startMon),
                });
                break;
              case Consumable.Potion:
              case Consumable.BombOrPotion:
                return undefined;
            }
          }
        }
        if (
          targetSquare.kind === "supermana-base" ||
          targetSquare.kind === "mon-base"
        ) {
          requiresAdditionalStep = true;
        }
        if (requiresAdditionalStep) {
          for (const at of nearbyLocations(targetLocation)) {
            const item = this.board.get(at);
            const square = this.board.squareAt(at);
            if (item !== undefined && item.kind !== "consumable") {
              continue;
            }
            if (regularSquareForMovement(square)) {
              thirdInputOptions.push(
                nextInput(
                  { kind: "location", location: at },
                  NextInputKind.DemonAdditionalStep,
                ),
              );
            } else if (
              square.kind === "mon-base" &&
              square.monKind === startMon.kind &&
              square.color === startMon.color
            ) {
              thirdInputOptions.push(
                nextInput(
                  { kind: "location", location: at },
                  NextInputKind.DemonAdditionalStep,
                ),
              );
            }
          }
        }
        break;
      }
      case NextInputKind.SpiritTargetCapture: {
        if (targetItem === undefined) {
          return undefined;
        }
        const targetMon = itemMon(targetItem);
        const targetMana = itemMana(targetItem);
        thirdInputOptions.push(
          ...this.nextInputsFromLocations(
            nearbyLocations(targetLocation),
            NextInputKind.SpiritTargetMove,
            false,
            undefined,
            (at) => {
              const destinationItem = this.board.get(at);
              const destinationSquare = this.board.squareAt(at);
              if (!spiritDestinationItemAllowed(targetItem, destinationItem)) {
                return false;
              }
              switch (destinationSquare.kind) {
                case "regular":
                case "consumable-base":
                case "mana-base":
                case "mana-pool":
                  return true;
                case "supermana-base": {
                  const destinationHasSupermana =
                    destinationItem?.kind === "mana" &&
                    destinationItem.mana.kind === "supermana";
                  return (
                    targetMana?.kind === "supermana" ||
                    (targetMana === undefined &&
                      targetMon?.kind === MonKind.Drainer &&
                      (destinationItem === undefined ||
                        destinationHasSupermana)) ||
                    (targetMon?.kind === MonKind.Drainer &&
                      destinationHasSupermana)
                  );
                }
                case "mon-base":
                  return (
                    targetMon?.kind === destinationSquare.monKind &&
                    targetMon.color === destinationSquare.color &&
                    targetMana === undefined &&
                    itemConsumable(targetItem) === undefined
                  );
              }
            },
          ),
        );
        break;
      }
      case NextInputKind.BombAttack: {
        const startMon = itemMon(startItem);
        if (startMon === undefined) {
          return undefined;
        }
        events.push({
          kind: "bomb-attack",
          by: startMon,
          from: startLocation,
          to: targetLocation,
        });
        if (targetItem !== undefined) {
          const targetMon = itemMon(targetItem);
          if (targetMon === undefined) {
            return undefined;
          }
          events.push({
            kind: "mon-fainted",
            mon: targetMon,
            from: targetLocation,
            to: this.board.base(targetMon),
          });
          if (targetItem.kind === "mon-with-mana") {
            if (targetItem.mana.kind === "regular") {
              events.push({
                kind: "mana-dropped",
                mana: targetItem.mana,
                at: targetLocation,
              });
            } else {
              events.push({
                kind: "supermana-back-to-base",
                from: targetLocation,
                to: this.board.supermanaBase(),
              });
            }
          }
          if (targetItem.kind === "mon-with-consumable") {
            switch (targetItem.consumable) {
              case Consumable.Bomb:
                events.push({
                  kind: "bomb-explosion",
                  at: targetLocation,
                });
                break;
              case Consumable.Potion:
              case Consumable.BombOrPotion:
                return undefined;
            }
          }
        }
        break;
      }
      case NextInputKind.DemonAdditionalStep:
      case NextInputKind.SpiritTargetMove:
      case NextInputKind.SelectConsumable:
        break;
    }

    return [events, thirdInputOptions];
  }

  #processThirdInput(
    thirdInput: NextInput,
    startItem: Item,
    startLocation: Location,
    targetLocation: Location,
  ): InputStageResult {
    const cacheKey = `${inputKey(thirdInput.input)}|${thirdInput.kind}|${
      thirdInput.actorMonItem === undefined
        ? ""
        : itemKey(thirdInput.actorMonItem)
    }|${itemKey(startItem)}|${locationIndex(startLocation)}|${locationIndex(targetLocation)}`;
    if (this.#queryCache.hasThirdStage(cacheKey)) {
      return this.#queryCache.getThirdStage(cacheKey);
    }
    const computed = this.#processThirdInputUncached(
      thirdInput,
      startItem,
      startLocation,
      targetLocation,
    );
    this.#queryCache.setThirdStage(cacheKey, computed);
    return computed;
  }

  #processThirdInputUncached(
    thirdInput: NextInput,
    startItem: Item,
    startLocation: Location,
    targetLocation: Location,
  ): InputStageResult {
    const targetItem = this.board.get(targetLocation);
    const fourthInputOptions: NextInput[] = [];
    const events: Event[] = [];

    switch (thirdInput.kind) {
      case NextInputKind.SpiritTargetMove: {
        if (thirdInput.input.kind !== "location" || targetItem === undefined) {
          return undefined;
        }
        const destinationLocation = thirdInput.input.location;
        const destinationItem = this.board.get(destinationLocation);
        const destinationSquare = this.board.squareAt(destinationLocation);
        events.push({
          kind: "spirit-target-move",
          item: targetItem,
          from: targetLocation,
          to: destinationLocation,
          by: startLocation,
        });

        if (destinationItem !== undefined) {
          switch (targetItem.kind) {
            case "mon":
              switch (destinationItem.kind) {
                case "mon":
                case "mon-with-mana":
                case "mon-with-consumable":
                  return undefined;
                case "mana":
                  events.push({
                    kind: "pickup-mana",
                    mana: destinationItem.mana,
                    by: targetItem.mon,
                    at: destinationLocation,
                  });
                  break;
                case "consumable":
                  switch (destinationItem.consumable) {
                    case Consumable.Potion:
                    case Consumable.Bomb:
                      return undefined;
                    case Consumable.BombOrPotion:
                      fourthInputOptions.push(
                        nextInput(
                          { kind: "modifier", modifier: Modifier.SelectBomb },
                          NextInputKind.SelectConsumable,
                          targetItem,
                        ),
                      );
                      fourthInputOptions.push(
                        nextInput(
                          { kind: "modifier", modifier: Modifier.SelectPotion },
                          NextInputKind.SelectConsumable,
                          targetItem,
                        ),
                      );
                      break;
                  }
                  break;
              }
              break;
            case "mana":
              if (destinationItem.kind !== "mon") {
                return undefined;
              }
              events.push({
                kind: "pickup-mana",
                mana: targetItem.mana,
                by: destinationItem.mon,
                at: destinationLocation,
              });
              break;
            case "mon-with-mana":
              switch (destinationItem.kind) {
                case "mon":
                case "mon-with-mana":
                case "mon-with-consumable":
                  return undefined;
                case "mana":
                  events.push({
                    kind: "mana-dropped",
                    mana: targetItem.mana,
                    at: targetLocation,
                  });
                  events.push({
                    kind: "pickup-mana",
                    mana: destinationItem.mana,
                    by: targetItem.mon,
                    at: destinationLocation,
                  });
                  break;
                case "consumable":
                  switch (destinationItem.consumable) {
                    case Consumable.Potion:
                    case Consumable.Bomb:
                      return undefined;
                    case Consumable.BombOrPotion:
                      events.push({
                        kind: "pickup-potion",
                        by: targetItem,
                        at: destinationLocation,
                      });
                      break;
                  }
                  break;
              }
              break;
            case "mon-with-consumable":
              if (
                destinationItem.kind !== "consumable" ||
                destinationItem.consumable !== Consumable.BombOrPotion
              ) {
                return undefined;
              }
              events.push({
                kind: "pickup-potion",
                by: targetItem,
                at: destinationLocation,
              });
              break;
            case "consumable":
              switch (destinationItem.kind) {
                case "mana":
                case "consumable":
                  return undefined;
                case "mon":
                  fourthInputOptions.push(
                    nextInput(
                      { kind: "modifier", modifier: Modifier.SelectBomb },
                      NextInputKind.SelectConsumable,
                      destinationItem,
                    ),
                  );
                  fourthInputOptions.push(
                    nextInput(
                      { kind: "modifier", modifier: Modifier.SelectPotion },
                      NextInputKind.SelectConsumable,
                      destinationItem,
                    ),
                  );
                  break;
                case "mon-with-mana":
                case "mon-with-consumable":
                  switch (targetItem.consumable) {
                    case Consumable.Potion:
                    case Consumable.Bomb:
                      return undefined;
                    case Consumable.BombOrPotion:
                      events.push({
                        kind: "pickup-potion",
                        by: destinationItem,
                        at: destinationLocation,
                      });
                      break;
                  }
                  break;
              }
              break;
          }
        }

        if (destinationSquare.kind === "mana-pool") {
          const mana = itemMana(targetItem);
          if (mana !== undefined) {
            events.push({
              kind: "mana-scored",
              mana: mana,
              at: destinationLocation,
            });
          }
        }
        break;
      }
      case NextInputKind.DemonAdditionalStep: {
        if (thirdInput.input.kind !== "location") {
          return undefined;
        }
        const demon = itemMon(startItem);
        if (demon === undefined) {
          return undefined;
        }
        const destinationLocation = thirdInput.input.location;
        events.push({
          kind: "demon-additional-step",
          demon: demon,
          from: targetLocation,
          to: destinationLocation,
        });
        const destinationItem = this.board.get(destinationLocation);
        if (destinationItem?.kind === "consumable") {
          switch (destinationItem.consumable) {
            case Consumable.Potion:
            case Consumable.Bomb:
              return undefined;
            case Consumable.BombOrPotion:
              fourthInputOptions.push(
                nextInput(
                  { kind: "modifier", modifier: Modifier.SelectBomb },
                  NextInputKind.SelectConsumable,
                  startItem,
                ),
              );
              fourthInputOptions.push(
                nextInput(
                  { kind: "modifier", modifier: Modifier.SelectPotion },
                  NextInputKind.SelectConsumable,
                  startItem,
                ),
              );
              break;
          }
        }
        break;
      }
      case NextInputKind.SelectConsumable: {
        if (thirdInput.input.kind !== "modifier") {
          return undefined;
        }
        const mon = itemMon(startItem);
        if (mon === undefined) {
          return undefined;
        }
        switch (thirdInput.input.modifier) {
          case Modifier.SelectBomb:
            events.push({
              kind: "pickup-bomb",
              by: mon,
              at: targetLocation,
            });
            break;
          case Modifier.SelectPotion:
            events.push({
              kind: "pickup-potion",
              by: startItem,
              at: targetLocation,
            });
            break;
        }
        break;
      }
      case NextInputKind.MonMove:
      case NextInputKind.ManaMove:
      case NextInputKind.MysticAction:
      case NextInputKind.DemonAction:
      case NextInputKind.SpiritTargetCapture:
      case NextInputKind.BombAttack:
        return undefined;
    }

    return [events, fourthInputOptions];
  }

  #resolveEvents(events: readonly Event[], doNotApplyEvents: boolean): Output {
    if (doNotApplyEvents) {
      return { kind: "events", events };
    }
    const applied = this.applyAndAddResultingEvents(events);
    return applied === undefined
      ? { kind: "invalid-input" }
      : { kind: "events", events: applied };
  }

  public applyAndAddResultingEvents(
    events: readonly Event[],
  ): Event[] | undefined {
    if (!canApplyRulesEvents(this.#state, events)) {
      return undefined;
    }
    this.invalidateProcessInputCache();
    const snapshotFen = (): string => this.fen();
    this.#history.beginEventApplication(snapshotFen, this.activeColor);
    const reduction = applyRulesEvents(this.#state, events);
    this.#history.completeEventApplication({
      snapshotFen,
      color: this.activeColor,
      events: reduction.events,
      turnAdvanced: reduction.turnAdvanced,
      winner: reduction.winner,
    });
    return reduction.events;
  }

  static #isLocationGuardedByAngelLocation(
    angelLocation: Location | undefined,
    targetLocation: Location,
  ): boolean {
    return (
      angelLocation !== undefined &&
      locationDistance(angelLocation, targetLocation) === 1
    );
  }

  public nextInputsFromLocations(
    locations: readonly Location[],
    kind: NextInputKind,
    onlyOne: boolean,
    specific: Location | undefined,
    filter: (location: Location) => boolean,
  ): NextInput[] {
    if (specific !== undefined) {
      if (
        locations.some((candidate) => locationEquals(candidate, specific)) &&
        filter(specific)
      ) {
        return [nextInput({ kind: "location", location: specific }, kind)];
      }
      return [];
    }
    if (onlyOne) {
      const one = locations.find(filter);
      return one === undefined
        ? []
        : [nextInput({ kind: "location", location: one }, kind)];
    }
    return locations
      .filter(filter)
      .map((at) => nextInput({ kind: "location", location: at }, kind));
  }

  public availableMoveKinds(): Map<AvailableMoveKind, number> {
    const moves = new Map<AvailableMoveKind, number>();
    moves.set(
      AvailableMoveKind.MonMove,
      MONS_MOVES_PER_TURN - this.monsMovesCount,
    );
    moves.set(AvailableMoveKind.Action, 0);
    moves.set(AvailableMoveKind.Potion, 0);
    moves.set(AvailableMoveKind.ManaMove, 0);
    if (this.turnNumber === 1) {
      return moves;
    }
    moves.set(
      AvailableMoveKind.Action,
      ACTIONS_PER_TURN - this.actionsUsedCount,
    );
    moves.set(AvailableMoveKind.Potion, this.playerPotionsCount());
    moves.set(
      AvailableMoveKind.ManaMove,
      MANA_MOVES_PER_TURN - this.manaMovesCount,
    );
    return moves;
  }

  public winnerColor(): Color | undefined {
    return winnerForState(this);
  }

  public isLaterThan(game: MonsGame): boolean {
    if (this.variant() !== game.variant()) {
      return false;
    }
    if (this.turnNumber > game.turnNumber) {
      return true;
    }
    if (this.turnNumber !== game.turnNumber) {
      return false;
    }
    return (
      this.playerPotionsCount() < game.playerPotionsCount() ||
      this.actionsUsedCount > game.actionsUsedCount ||
      this.manaMovesCount > game.manaMovesCount ||
      this.monsMovesCount > game.monsMovesCount ||
      this.board.faintedMonsLocations(otherColor(this.activeColor)).length >
        game.board.faintedMonsLocations(otherColor(game.activeColor)).length
    );
  }

  public isFirstTurn(): boolean {
    return isFirstTurnState(this);
  }

  public playerPotionsCount(): number {
    return currentPlayerPotions(this);
  }

  public playerCanMoveMon(): boolean {
    return canPlayerMoveMon(this);
  }

  public playerCanMoveMana(): boolean {
    return canPlayerMoveMana(this);
  }

  public playerCanUseAction(): boolean {
    return canPlayerUseAction(this);
  }
}
