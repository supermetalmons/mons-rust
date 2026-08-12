import { Board } from "../../engine/board/storage.js";
import { Color, Consumable, MonKind, type Mana, type Mon } from "../../api/types.js";
import {
  colorId,
  isMonFainted,
  itemMon,
  manaScore,
  otherColor,
  type Item,
} from "../../engine/model/domain.js";
import {
  BOARD_CELLS,
  MAX_LOCATION_INDEX,
  locationIndex,
  type Location,
} from "../../engine/board/geometry.js";
import { MonsGame } from "../../engine/game/mons-game.js";
import {
  AttackReachSummary,
  attackReachSummary,
  attackReachSummaryForTargets,
  attackReachSummaryTargetLocations,
  canAttackTargetOnBoardWithHash,
} from "../exact/attack-reach.js";
import { drainerImmediateThreats } from "../exact/drainer-safety.js";
import { exactBoardHash } from "../exact/hash.js";
import { exactStrategicAnalysis } from "../exact/strategic.js";
import type { ExactStrategicAnalysis } from "../exact/types.js";
import type { AutomoveExecutionContext } from "../core/execution-context.js";
import type { Hash64 } from "../core/hash64.js";

type ScoringDangerSource = {
  readonly location: Location;
  readonly heuristicThreat: boolean;
  readonly exactActionThreat: boolean;
  readonly exactBombThreat: boolean;
};

type ScoringManaEntry = {
  readonly location: Location;
  readonly mana: Mana;
  readonly scoreSteps: number;
};

type ScoringBoardSummary = {
  readonly manaEntries: ScoringManaEntry[];
  readonly liveManaCarriers: readonly [ScoringManaEntry[], ScoringManaEntry[]];
  readonly liveMonLocations: readonly [Location[], Location[]];
  readonly drainerTargetLocations: readonly [Location[], Location[]];
  readonly liveDrainerLocations: readonly [Location[], Location[]];
  readonly liveAngelLocations: readonly [Location[], Location[]];
  readonly dangerSources: readonly [ScoringDangerSource[], ScoringDangerSource[]];
  readonly looseConsumableLocations: Location[];
  readonly regularManaMoveScores: [number, number];
  readonly regularManaScorePathSteps: [number | undefined, number | undefined];
};

export type ManaPathSnapshot = {
  readonly candidates: ScoringManaEntry[];
  readonly regularManaMoveScores: readonly [number, number];
};

type ScoringEvalOptions = {
  readonly allowExactStrategic: boolean;
  readonly useAttackReachSummary?: boolean;
  readonly narrowAttackReachTargets?: boolean;
  readonly narrowAttackReachToDrainers?: boolean;
};

function scoringDangerSourceFlags(
  item: Item,
  mon: Mon,
): readonly [boolean, boolean, boolean] {
  const heuristicThreat =
    item.kind !== "mon-with-mana" &&
    (mon.kind === MonKind.Mystic ||
      mon.kind === MonKind.Demon ||
      item.kind === "mon-with-consumable");
  const exactActionThreat = mon.kind === MonKind.Mystic || mon.kind === MonKind.Demon;
  const exactBombThreat =
    item.kind === "mon-with-consumable" && item.consumable === Consumable.Bomb;
  return [heuristicThreat, exactActionThreat, exactBombThreat];
}

function distanceToAnyClosestPool(location: Location): number {
  return (
    Math.max(
      Math.min(location.i, Math.abs(MAX_LOCATION_INDEX - location.i)),
      Math.min(location.j, Math.abs(MAX_LOCATION_INDEX - location.j)),
    ) + 1
  );
}

export function colorSlot(color: Color): 0 | 1 {
  return color === Color.White ? 0 : 1;
}

function scoringBoardSummary(board: Board): ScoringBoardSummary {
  const summary: ScoringBoardSummary = {
    manaEntries: [],
    liveManaCarriers: [[], []],
    liveMonLocations: [[], []],
    drainerTargetLocations: [[], []],
    liveDrainerLocations: [[], []],
    liveAngelLocations: [[], []],
    dangerSources: [[], []],
    looseConsumableLocations: [],
    regularManaMoveScores: [0, 0],
    regularManaScorePathSteps: [undefined, undefined],
  };

  for (const [location, item] of board.entries()) {
    switch (item.kind) {
      case "mana": {
        const scoreSteps = distanceToAnyClosestPool(location) - 1;
        summary.manaEntries.push({ location, mana: item.mana, scoreSteps });
        if (item.mana.kind === "regular") {
          const slot = colorSlot(item.mana.color);
          const candidateSteps = scoreSteps + 1;
          const current = summary.regularManaScorePathSteps[slot];
          summary.regularManaScorePathSteps[slot] =
            current === undefined ? candidateSteps : Math.min(current, candidateSteps);
          if (scoreSteps <= 1) {
            summary.regularManaMoveScores[slot] = manaScore(item.mana, item.mana.color);
          }
        }
        break;
      }
      case "mon":
      case "mon-with-mana":
      case "mon-with-consumable": {
        const mon = item.mon;
        const slot = colorSlot(mon.color);
        if (mon.kind === MonKind.Drainer) {
          summary.drainerTargetLocations[slot].push(location);
        }
        if (isMonFainted(mon)) break;
        if (item.kind === "mon-with-mana") {
          summary.liveManaCarriers[slot].push({
            location,
            mana: item.mana,
            scoreSteps: distanceToAnyClosestPool(location) - 1,
          });
        }
        summary.liveMonLocations[slot].push(location);
        if (mon.kind === MonKind.Drainer) {
          summary.liveDrainerLocations[slot].push(location);
        }
        if (mon.kind === MonKind.Angel) {
          summary.liveAngelLocations[slot].push(location);
        }
        const [heuristicThreat, exactActionThreat, exactBombThreat] =
          scoringDangerSourceFlags(item, mon);
        if (heuristicThreat || exactActionThreat || exactBombThreat) {
          summary.dangerSources[slot].push({
            location,
            heuristicThreat,
            exactActionThreat,
            exactBombThreat,
          });
        }
        break;
      }
      case "consumable":
        summary.looseConsumableLocations.push(location);
        break;
    }
  }
  return summary;
}

function manaPathSnapshot(summary: ScoringBoardSummary): ManaPathSnapshot {
  return {
    candidates: summary.manaEntries.slice(),
    regularManaMoveScores: [
      summary.regularManaMoveScores[0],
      summary.regularManaMoveScores[1],
    ],
  };
}

export class ScoringEvalContext {
  readonly #execution: AutomoveExecutionContext;
  readonly #game: MonsGame;
  readonly #board: Board;
  readonly #allowExactStrategic: boolean;
  readonly #enableAttackReachSummary: boolean;
  readonly #enableAttackReachTargetNarrowing: boolean;
  readonly #enableAttackReachDrainerTargetNarrowing: boolean;
  #boardHash: Hash64 | undefined;
  #boardSummary: ScoringBoardSummary | undefined;
  #manaPathSnapshot: ManaPathSnapshot | undefined;
  #exactAnalysis: ExactStrategicAnalysis | undefined;
  #attackReachTargets: readonly [Location[], Location[]] | undefined;
  #firstDrainerThreatMemoIndex = -1;
  #firstDrainerThreatMemo: readonly [number, number] | undefined;
  #secondDrainerThreatMemoIndex = -1;
  #secondDrainerThreatMemo: readonly [number, number] | undefined;
  #drainerThreatMemoMap: Map<number, readonly [number, number]> | undefined;
  #attackReachSummaryMemo: Map<number, AttackReachSummary> | undefined;

  public constructor(
    execution: AutomoveExecutionContext,
    game: MonsGame,
    options: ScoringEvalOptions,
  ) {
    this.#execution = execution;
    this.#game = game;
    this.#board = game.board;
    this.#allowExactStrategic = options.allowExactStrategic;
    this.#enableAttackReachSummary = options.useAttackReachSummary ?? false;
    this.#enableAttackReachTargetNarrowing = options.narrowAttackReachTargets ?? false;
    this.#enableAttackReachDrainerTargetNarrowing =
      options.narrowAttackReachToDrainers ?? false;
  }

  public get boardHash(): Hash64 {
    this.#boardHash ??= exactBoardHash(this.#board);
    return this.#boardHash;
  }

  public get execution(): AutomoveExecutionContext {
    return this.#execution;
  }

  public get game(): MonsGame {
    return this.#game;
  }

  public get board(): Board {
    return this.#board;
  }

  public get allowExactStrategic(): boolean {
    return this.#allowExactStrategic;
  }

  public boardSummary(): ScoringBoardSummary {
    this.#boardSummary ??= scoringBoardSummary(this.#board);
    return this.#boardSummary;
  }

  public manaPathSnapshot(): ManaPathSnapshot {
    this.#manaPathSnapshot ??= manaPathSnapshot(this.boardSummary());
    return this.#manaPathSnapshot;
  }

  public exactAnalysis(): ExactStrategicAnalysis | undefined {
    if (!this.#allowExactStrategic) return undefined;
    this.#exactAnalysis ??= exactStrategicAnalysis(this.#execution, this.#game);
    return this.#exactAnalysis;
  }

  #targets(targetColor: Color): readonly Location[] {
    this.#attackReachTargets ??= [
      attackReachSummaryTargetLocations(this.#board, Color.White),
      attackReachSummaryTargetLocations(this.#board, Color.Black),
    ];
    return this.#attackReachTargets[colorSlot(targetColor)];
  }

  #drainerTargets(targetColor: Color): readonly Location[] {
    return this.boardSummary().drainerTargetLocations[colorSlot(targetColor)];
  }

  #attackReachSummary(
    attackerColor: Color,
    targetColor: Color,
    remainingMoves: number,
    canUseAction: boolean,
    drainerTargetsOnly: boolean,
  ): AttackReachSummary {
    const tag =
      Number.isInteger(remainingMoves) &&
      remainingMoves >= 0 &&
      remainingMoves <= 0xffff
        ? remainingMoves * 16 +
          colorId(attackerColor) +
          colorId(targetColor) * 2 +
          Number(canUseAction) * 4 +
          Number(drainerTargetsOnly) * 8
        : undefined;
    if (tag !== undefined) {
      const cached = this.#attackReachSummaryMemo?.get(tag);
      if (cached !== undefined) return cached;
    }
    let summary: AttackReachSummary;
    if (drainerTargetsOnly) {
      summary = attackReachSummaryForTargets(
        this.#execution,
        this.#board,
        attackerColor,
        remainingMoves,
        canUseAction,
        this.#drainerTargets(targetColor),
      );
    } else if (this.#enableAttackReachTargetNarrowing) {
      summary = attackReachSummaryForTargets(
        this.#execution,
        this.#board,
        attackerColor,
        remainingMoves,
        canUseAction,
        this.#targets(targetColor),
      );
    } else {
      summary = attackReachSummary(
        this.#execution,
        this.#board,
        attackerColor,
        targetColor,
        remainingMoves,
        canUseAction,
      );
    }
    if (tag !== undefined) {
      (this.#attackReachSummaryMemo ??= new Map()).set(tag, summary);
    }
    return summary;
  }

  public drainerImmediateThreats(
    color: Color,
    location: Location,
  ): readonly [number, number] {
    if (this.#enableAttackReachSummary) {
      return this.#attackReachSummary(
        otherColor(color),
        color,
        0,
        true,
        this.#enableAttackReachDrainerTargetNarrowing,
      ).immediateThreats(location);
    }
    const memoIndex =
      locationIndex(location) + (color === Color.Black ? BOARD_CELLS : 0);
    if (this.#drainerThreatMemoMap !== undefined) {
      const cached = this.#drainerThreatMemoMap.get(memoIndex);
      if (cached !== undefined) return cached;
    } else {
      if (
        memoIndex === this.#firstDrainerThreatMemoIndex &&
        this.#firstDrainerThreatMemo !== undefined
      ) {
        return this.#firstDrainerThreatMemo;
      }
      if (
        memoIndex === this.#secondDrainerThreatMemoIndex &&
        this.#secondDrainerThreatMemo !== undefined
      ) {
        return this.#secondDrainerThreatMemo;
      }
    }
    const threats = drainerImmediateThreats(
      this.#execution,
      this.#board,
      color,
      location,
    );
    if (this.#drainerThreatMemoMap !== undefined) {
      this.#drainerThreatMemoMap.set(memoIndex, threats);
    } else if (this.#firstDrainerThreatMemoIndex < 0) {
      this.#firstDrainerThreatMemoIndex = memoIndex;
      this.#firstDrainerThreatMemo = threats;
    } else if (this.#secondDrainerThreatMemoIndex < 0) {
      this.#secondDrainerThreatMemoIndex = memoIndex;
      this.#secondDrainerThreatMemo = threats;
    } else {
      const memo = new Map<number, readonly [number, number]>();
      if (this.#firstDrainerThreatMemo !== undefined) {
        memo.set(this.#firstDrainerThreatMemoIndex, this.#firstDrainerThreatMemo);
      }
      if (this.#secondDrainerThreatMemo !== undefined) {
        memo.set(this.#secondDrainerThreatMemoIndex, this.#secondDrainerThreatMemo);
      }
      memo.set(memoIndex, threats);
      this.#drainerThreatMemoMap = memo;
      this.#firstDrainerThreatMemo = undefined;
      this.#secondDrainerThreatMemo = undefined;
    }
    return threats;
  }

  public canAttackTargetOnBoard(
    attackerColor: Color,
    targetColor: Color,
    target: Location,
    remainingMoves: number,
    canUseAction: boolean,
  ): boolean {
    if (this.#enableAttackReachSummary) {
      const targetItem = this.#board.get(target);
      const targetMon = targetItem === undefined ? undefined : itemMon(targetItem);
      const drainerOnly =
        this.#enableAttackReachDrainerTargetNarrowing &&
        targetMon?.color === targetColor &&
        targetMon.kind === MonKind.Drainer;
      if (drainerOnly) {
        return this.#attackReachSummary(
          attackerColor,
          targetColor,
          remainingMoves,
          canUseAction,
          true,
        ).canAttackTarget(target);
      }
      if (!this.#enableAttackReachDrainerTargetNarrowing) {
        return this.#attackReachSummary(
          attackerColor,
          targetColor,
          remainingMoves,
          canUseAction,
          false,
        ).canAttackTarget(target);
      }
    }
    return canAttackTargetOnBoardWithHash(
      this.#execution,
      this.#board,
      this.boardHash,
      attackerColor,
      targetColor,
      target,
      remainingMoves,
      canUseAction,
    );
  }
}
