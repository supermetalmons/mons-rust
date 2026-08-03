import { AutomoveEngine } from "../automove/automove-engine.js";
import { suggestMove as suggestAutomove } from "../automove/runtime.js";
import type { Input as EngineInput } from "../engine/domain.js";
import {
  eventArrayFen,
  inputArrayFen,
  parseInputArrayFen,
} from "../engine/fen.js";
import { MonsGame } from "../engine/game.js";
import {
  availableMoveCountsFromEngine,
  fromEngineEvent,
  fromEngineInput,
  fromEngineItem,
  fromEngineOutput,
  fromEngineSquare,
  toEngineInput,
  toEnginePosition,
} from "./conversions.js";
import { replayInterleavedMoves } from "./replay.js";
import {
  AutomovePreference,
  Color,
  GameVariant,
  type AvailableMoveCounts,
  type AutomovePreference as AutomovePreferenceValue,
  type BoardItem,
  type Color as ColorValue,
  type GameVariant as GameVariantValue,
  type Input,
  type InputResolution,
  type MoveSuggestion,
  type MoveUsage,
  type PlayResult,
  type Position,
  type Square,
  type TrackingEntry,
} from "./types.js";

export type GameOptions = {
  readonly variant?: GameVariantValue;
};

export type MoveHistory = {
  readonly white: readonly string[];
  readonly black: readonly string[];
};

export class Game {
  #engine: MonsGame;
  readonly #automoveEngine = new AutomoveEngine();

  public constructor(options: GameOptions = {}) {
    const { variant = GameVariant.Classic } = options;
    if (!isGameVariant(variant)) {
      throw new TypeError(`unsupported game variant: ${String(variant)}`);
    }
    this.#engine = new MonsGame(true, variant);
  }

  static #fromEngine(engine: MonsGame): Game {
    const game = new Game({ variant: engine.variant() });
    game.#engine = engine;
    return game;
  }

  public static fromFen(fen: string): Game | undefined {
    const engine = MonsGame.fromFen(fen, true);
    if (engine?.fen() !== fen) {
      return undefined;
    }
    return Game.#fromEngine(engine);
  }

  public get variant(): GameVariantValue {
    return this.#engine.variant();
  }

  public get activeColor(): ColorValue {
    return this.#engine.activeColor;
  }

  public get turnNumber(): number {
    return this.#engine.turnNumber;
  }

  public get scores(): Readonly<Record<ColorValue, number>> {
    return {
      [Color.White]: this.#engine.whiteScore,
      [Color.Black]: this.#engine.blackScore,
    };
  }

  public get potions(): Readonly<Record<ColorValue, number>> {
    return {
      [Color.White]: this.#engine.whitePotionsCount,
      [Color.Black]: this.#engine.blackPotionsCount,
    };
  }

  public get moveUsage(): MoveUsage {
    return {
      monMoves: this.#engine.monsMovesCount,
      manaMoves: this.#engine.manaMovesCount,
      actions: this.#engine.actionsUsedCount,
    };
  }

  public get winner(): ColorValue | undefined {
    return this.#engine.winnerColor();
  }

  public get historyVerified(): boolean {
    return this.#engine.isMovesVerified;
  }

  public get takebackFens(): readonly string[] {
    return [...this.#engine.takebackFens];
  }

  public get trackingEntries(): readonly TrackingEntry[] {
    return this.#engine.verboseTrackingEntities.map((entry) => ({
      fen: entry.fen,
      color: entry.color,
      events: entry.events.map(fromEngineEvent),
      eventsFen: eventArrayFen(entry.events),
    }));
  }

  public toFen(): string {
    return this.#engine.fen();
  }

  public preview(inputs: readonly Input[]): InputResolution {
    const engineInputs = inputs.map(toEngineInput);
    return this.#previewEngineInputs(engineInputs, inputArrayFen(engineInputs));
  }

  public previewFen(inputFen: string): InputResolution {
    const engineInputs = parseInputArrayFen(inputFen);
    if (engineInputs === undefined) return { kind: "invalid", inputFen };
    return this.#previewEngineInputs(engineInputs, inputArrayFen(engineInputs));
  }

  public play(inputs: readonly Input[]): PlayResult {
    const engineInputs = inputs.map(toEngineInput);
    return this.#playEngineInputs(engineInputs, inputArrayFen(engineInputs));
  }

  public playFen(inputFen: string): PlayResult {
    const engineInputs = parseInputArrayFen(inputFen);
    if (engineInputs === undefined) return { kind: "invalid", inputFen };
    return this.#playEngineInputs(engineInputs, inputArrayFen(engineInputs));
  }

  #previewEngineInputs(
    inputs: readonly EngineInput[],
    inputFen: string,
  ): InputResolution {
    const simulation =
      inputs.length === 1 && inputs[0]?.kind === "takeback"
        ? this.#engine.copy()
        : this.#engine.fork();
    return fromEngineOutput(
      simulation.processInput(inputs, false, false),
      inputFen,
    );
  }

  #playEngineInputs(
    inputs: readonly EngineInput[],
    inputFen: string,
  ): PlayResult {
    const result = fromEngineOutput(
      this.#engine.processInput(inputs, false, false),
      inputFen,
    );
    return result.kind === "complete" ? result : { kind: "invalid", inputFen };
  }

  public itemAt(position: Position): BoardItem | undefined {
    const item = this.#engine.board.get(toEnginePosition(position));
    return item === undefined ? undefined : fromEngineItem(item);
  }

  public squareAt(position: Position): Square {
    return fromEngineSquare(
      this.#engine.board.squareAt(toEnginePosition(position)),
    );
  }

  public contentPositions(): readonly Position[] {
    const positions = new Map<string, Position>();
    for (const [position] of this.#engine.board.entries()) {
      positions.set(positionKey(position.i, position.j), {
        row: position.i,
        column: position.j,
      });
    }
    for (const position of this.#engine.board.allMonsBases()) {
      positions.set(positionKey(position.i, position.j), {
        row: position.i,
        column: position.j,
      });
    }
    return [...positions.values()].sort(
      (left, right) => left.row - right.row || left.column - right.column,
    );
  }

  public canTakeback(color: ColorValue): boolean {
    if (!isColor(color)) {
      throw new TypeError(`unsupported color: ${String(color)}`);
    }
    return this.#engine.canTakeback(color);
  }

  public takeback(): PlayResult {
    return this.play([{ kind: "takeback" }]);
  }

  public previousTurn(takebackFens: readonly string[]): Game | undefined {
    const trackingEntries = this.#engine.verboseTrackingEntities;
    if (trackingEntries.length <= 1) {
      return undefined;
    }

    const tracking = trackingEntries.slice(0, -1);
    const latest = tracking[tracking.length - 1];
    const restored = MonsGame.fromFen(latest?.fen ?? this.#engine.fen(), true);
    if (restored === undefined) {
      return undefined;
    }
    const restoredTakebackIndex = takebackFens.lastIndexOf(restored.fen());
    if (takebackFens.length > 0 && restoredTakebackIndex < 0) {
      return undefined;
    }
    const restoredTakebackFens =
      restoredTakebackIndex < 0
        ? []
        : takebackFens.slice(0, restoredTakebackIndex + 1);
    if (
      restoredTakebackFens.some((fen) => {
        const historyGame = MonsGame.fromFen(fen, false);
        return (
          historyGame?.fen() !== fen ||
          historyGame.variant() !== restored.variant()
        );
      })
    ) {
      return undefined;
    }
    restored.replaceHistory({
      takebackFens: restoredTakebackFens,
      trackingEntries: tracking,
      verboseTrackingEnabled: true,
      movesVerified: this.#engine.isMovesVerified,
    });
    return Game.#fromEngine(restored);
  }

  public verifyHistory(history: MoveHistory): boolean {
    const verification = new MonsGame(true, this.#engine.variant());
    const replay = replayInterleavedMoves(
      verification,
      history.white,
      history.black,
    );
    if (
      replay.status !== "complete" ||
      verification.fen() !== this.#engine.fen()
    ) {
      return false;
    }

    this.#engine.replaceHistory({
      takebackFens: verification.takebackFens,
      trackingEntries: verification.verboseTrackingEntities,
      movesVerified: true,
    });
    return true;
  }

  public isLaterThan(other: Game): boolean {
    return this.#engine.isLaterThan(other.#engine);
  }

  public availableMoveCounts(): AvailableMoveCounts {
    return availableMoveCountsFromEngine(this.#engine.availableMoveKinds());
  }

  public suggestMove(
    preference: AutomovePreferenceValue,
  ): MoveSuggestion | undefined {
    if (!isAutomovePreference(preference)) {
      throw new TypeError(
        `unsupported automove preference: ${String(preference)}`,
      );
    }
    const suggestion = this.#automoveEngine.run((execution) =>
      suggestAutomove(execution, this.#engine, preference),
    );
    if (suggestion.output.kind !== "events") {
      return undefined;
    }
    const engineInputs = parseInputArrayFen(suggestion.inputFen);
    if (engineInputs === undefined) {
      return undefined;
    }
    return {
      inputs: engineInputs.map(fromEngineInput),
      inputFen: suggestion.inputFen,
      events: suggestion.output.events.map(fromEngineEvent),
    };
  }

  public clearTracking(): void {
    this.#engine.clearTracking();
  }
}

function isColor(value: unknown): value is ColorValue {
  return value === Color.White || value === Color.Black;
}

function isGameVariant(value: unknown): value is GameVariantValue {
  return (Object.values(GameVariant) as readonly unknown[]).includes(value);
}

function isAutomovePreference(
  value: unknown,
): value is AutomovePreferenceValue {
  return (Object.values(AutomovePreference) as readonly unknown[]).includes(
    value,
  );
}

function positionKey(row: number, column: number): string {
  return `${row},${column}`;
}
