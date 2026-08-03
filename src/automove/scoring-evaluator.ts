import { Board } from "../engine/board.js";
import type { AutomoveExecutionContext } from "./execution-context.js";
import {
  Color,
  Consumable,
  MonKind,
  colorId,
  isMonFainted,
  isSpiritTargetAllowed,
  itemMon,
  manaScore,
  otherColor,
  type Item,
  type Mana,
  type Mon,
} from "../engine/domain.js";
import {
  BOARD_CENTER_INDEX,
  BOARD_CELLS,
  BOARD_SIZE,
  MAX_LOCATION_INDEX,
  locationDistance,
  locationEquals,
  locationIndex,
  spiritReachableLocations,
  type Location,
} from "../engine/geometry.js";
import {
  MON_BASE_LOCATIONS,
  MONS_MOVES_PER_TURN,
  TARGET_SCORE,
} from "../engine/config.js";
import { MonsGame } from "../engine/game.js";
import {
  clampHeuristicScore,
  saturatingScoreAdd,
  saturatingScoreMultiply,
} from "./score-math.js";
import type { ScoringWeights } from "./scoring-profile.js";
import {
  AttackReachSummary,
  attackReachSummary,
  attackReachSummaryForTargets,
  attackReachSummaryTargetLocations,
  canAttackTargetOnBoardWithHash,
  drainerImmediateThreats,
  exactBoardHash,
  exactStrategicAnalysis,
  isDrainerUnderImmediateThreat,
  isDrainerUnderWalkThreatWithHash,
  type ExactColorSummary,
  type ExactSpiritSummary,
  type ExactStrategicAnalysis,
} from "./exact.js";
import type { Hash64 } from "./hash64.js";

const PROTECTED_HIGH_VALUE_CARRIER_SUPERMANA_SCALE_BP = 2_500;
const PROTECTED_HIGH_VALUE_CARRIER_OPPONENT_MANA_SCALE_BP = 2_500;
const PROTECTED_HIGH_VALUE_CARRIER_VIRTUAL_SCORE_BP_MAX = 9_200;
const PROTECTED_HIGH_VALUE_CARRIER_OPPONENT_SCORE_MARGIN = 2;
const MON_BASE_INDICES = new Set(MON_BASE_LOCATIONS.map(locationIndex));

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

type ScoringManaCarrierEntry = ScoringManaEntry;

type ScoringBoardSummary = {
  readonly manaEntries: ScoringManaEntry[];
  readonly liveManaCarriers: readonly [
    ScoringManaCarrierEntry[],
    ScoringManaCarrierEntry[],
  ];
  readonly liveMonLocations: readonly [Location[], Location[]];
  readonly drainerTargetLocations: readonly [Location[], Location[]];
  readonly liveDrainerLocations: readonly [Location[], Location[]];
  readonly liveAngelLocations: readonly [Location[], Location[]];
  readonly dangerSources: readonly [
    ScoringDangerSource[],
    ScoringDangerSource[],
  ];
  readonly looseConsumableLocations: Location[];
  readonly regularManaMoveScores: [number, number];
  readonly regularManaScorePathSteps: [number | undefined, number | undefined];
};

type DrainerSafetySnapshot = {
  readonly riskDanger: number;
  readonly minMana: number;
  readonly angelNearby: boolean;
  readonly exactDangerThreat: boolean;
  readonly walkThreat: boolean;
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
  const exactActionThreat =
    mon.kind === MonKind.Mystic || mon.kind === MonKind.Demon;
  const exactBombThreat =
    item.kind === "mon-with-consumable" && item.consumable === Consumable.Bomb;
  return [heuristicThreat, exactActionThreat, exactBombThreat];
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
            current === undefined
              ? candidateSteps
              : Math.min(current, candidateSteps);
          if (scoreSteps <= 1) {
            summary.regularManaMoveScores[slot] = manaScore(
              item.mana,
              item.mana.color,
            );
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
        if (isMonFainted(mon)) {
          break;
        }
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

function exactSafe(snapshot: DrainerSafetySnapshot): boolean {
  return !snapshot.exactDangerThreat && !snapshot.walkThreat;
}

function guardedAgainstExactAttack(snapshot: DrainerSafetySnapshot): boolean {
  return snapshot.angelNearby && !snapshot.exactDangerThreat;
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
    this.#enableAttackReachTargetNarrowing =
      options.narrowAttackReachTargets ?? false;
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
    if (!this.#allowExactStrategic) {
      return undefined;
    }
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
      if (cached !== undefined) {
        return cached;
      }
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
      if (cached !== undefined) {
        return cached;
      }
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
        memo.set(
          this.#firstDrainerThreatMemoIndex,
          this.#firstDrainerThreatMemo,
        );
      }
      if (this.#secondDrainerThreatMemo !== undefined) {
        memo.set(
          this.#secondDrainerThreatMemoIndex,
          this.#secondDrainerThreatMemo,
        );
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
      const targetMon =
        targetItem === undefined ? undefined : itemMon(targetItem);
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

export function evaluatePreferabilityWithWeightsAndExactPolicy(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  color: Color,
  weights: ScoringWeights,
  allowExactStrategic: boolean,
): number {
  const context = new ScoringEvalContext(execution, game, {
    allowExactStrategic,
  });
  return evaluatePreferabilityWithContext(context, color, weights);
}

export function evaluatePreferabilityWithContext(
  context: ScoringEvalContext,
  color: Color,
  weights: ScoringWeights,
): number {
  const game = context.game;
  const useHeuristicFormula = context.allowExactStrategic
    ? weights.formula.useHeuristicFormula
    : true;
  const includeRegularManaMoveWindows =
    weights.formula.includeRegularManaMoveWindows && !useHeuristicFormula;
  const includeMatchPointWindow =
    weights.formula.includeMatchPointWindow && !useHeuristicFormula;
  const nextTurnWindowScaleBp = Math.min(
    20_000,
    Math.max(0, Math.trunc(weights.formula.nextTurnWindowScaleBp)),
  );
  const supermanaBase = game.board.supermanaBase();
  const remainingMonMovesForActive = Math.max(
    0,
    MONS_MOVES_PER_TURN - game.monsMovesCount,
  );
  const exactAnalysis = useHeuristicFormula
    ? undefined
    : context.exactAnalysis();
  const myExactSummary = exactAnalysis?.colorSummary(color);
  const opponentExactSummary = exactAnalysis?.colorSummary(otherColor(color));
  const myScoreNow = color === Color.White ? game.whiteScore : game.blackScore;
  const opponentScoreNow =
    color === Color.White ? game.blackScore : game.whiteScore;

  const scoreDifference =
    color === Color.White
      ? game.whiteScore - game.blackScore
      : game.blackScore - game.whiteScore;
  const potionDifference =
    color === Color.White
      ? game.whitePotionsCount - game.blackPotionsCount
      : game.blackPotionsCount - game.whitePotionsCount;
  let score =
    scoreDifference * weights.material.confirmedScore +
    potionDifference * weights.material.hasConsumable;
  if (weights.formula.doubleConfirmedScore) {
    score = score * weights.material.confirmedScore;
  }

  score = scorePreferabilityBoardItems(
    color,
    weights,
    context,
    useHeuristicFormula,
    supermanaBase,
    remainingMonMovesForActive,
    exactAnalysis,
    myExactSummary,
    opponentExactSummary,
    score,
  );
  score = scorePreferabilityRaceWindows(
    color,
    weights,
    context,
    useHeuristicFormula,
    includeRegularManaMoveWindows,
    myExactSummary,
    opponentExactSummary,
    score,
  );
  return clampHeuristicScore(
    scorePreferabilityImmediateWindows(
      color,
      weights,
      context,
      useHeuristicFormula,
      includeRegularManaMoveWindows,
      includeMatchPointWindow,
      nextTurnWindowScaleBp,
      remainingMonMovesForActive,
      myExactSummary,
      opponentExactSummary,
      myScoreNow,
      opponentScoreNow,
      score,
    ),
  );
}

function scorePreferabilityBoardItems(
  color: Color,
  weights: ScoringWeights,
  context: ScoringEvalContext,
  useHeuristicFormula: boolean,
  supermanaBase: Location,
  remainingMonMovesForActive: number,
  exactAnalysis: ExactStrategicAnalysis | undefined,
  myExactSummary: ExactColorSummary | undefined,
  opponentExactSummary: ExactColorSummary | undefined,
  initialScore: number,
): number {
  const game = context.game;
  let score = initialScore;
  const addScore = (value: number): void => {
    score = score + value;
  };
  const addSigned = (multiplier: number, value: number): void => {
    addScore(multiplier * value);
  };
  const addSignedRatio = (
    multiplier: number,
    value: number,
    divisor: number,
  ): void => {
    addScore(Math.trunc((multiplier * value) / divisor));
  };

  const evaluateDrainer = (
    mon: Mon,
    location: Location,
    multiplier: number,
    includeHeuristicPickupPath: boolean,
  ): void => {
    const safety = drainerSafetySnapshotWithContext(
      mon.color,
      location,
      useHeuristicFormula,
      weights.threat.drainerWalkThreatBoolean !== 0,
      context,
    );
    addSignedRatio(
      multiplier,
      weights.position.drainerCloseToMana,
      safety.minMana,
    );
    addSignedRatio(
      multiplier,
      weights.position.drainerCloseToOwnPool,
      distanceToClosestPool(location, mon.color),
    );
    addSignedRatio(
      multiplier,
      weights.position.drainerCloseToSupermana,
      distanceToLocation(location, supermanaBase),
    );
    if (!guardedAgainstExactAttack(safety)) {
      addSignedRatio(
        multiplier,
        weights.position.drainerAtRisk,
        safety.riskDanger,
      );
    } else {
      addSigned(multiplier, weights.position.angelGuardingDrainer);
    }

    if (includeHeuristicPickupPath || !useHeuristicFormula) {
      const path = useHeuristicFormula
        ? bestDrainerPickupPathWithSnapshot(
            context.manaPathSnapshot(),
            mon.color,
            location,
          )
        : exactAnalysis?.colorSummary(mon.color).bestDrainerPickup;
      if (path !== undefined) {
        const pathSteps = "pathSteps" in path ? path.pathSteps : path[0];
        const totalMoves =
          "totalMoves" in path ? path.totalMoves : pathSteps + 1;
        const manaValue = "manaValue" in path ? path.manaValue : path[1];
        addSignedRatio(
          multiplier,
          weights.threat.drainerBestManaPath * manaValue,
          pathSteps + 1,
        );
        if (
          mon.color === game.activeColor &&
          totalMoves <= remainingMonMovesForActive
        ) {
          addSigned(
            multiplier,
            weights.threat.drainerPickupScoreThisTurn * manaValue,
          );
        }
      }
    }

    if (weights.threat.drainerImmediateThreat !== 0) {
      const [actionThreats, bombThreats] = drainerImmediateThreatsWithContext(
        mon.color,
        location,
        context,
      );
      const immediateThreats = safety.angelNearby
        ? bombThreats
        : actionThreats + bombThreats;
      if (immediateThreats > 0) {
        addSigned(
          multiplier,
          weights.threat.drainerImmediateThreat * immediateThreats,
        );
      }
    }

    const evaluateDanger =
      weights.threat.drainerDangerBoolean !== 0 ||
      weights.threat.drainerWalkThreatBoolean !== 0;
    const underDangerThreat = evaluateDanger && safety.exactDangerThreat;
    if (weights.threat.drainerDangerBoolean !== 0 && underDangerThreat) {
      addSigned(multiplier, weights.threat.drainerDangerBoolean);
      if (multiplier === -1) {
        addScore(weights.threat.opponentDrainerAttackBonus);
      }
    }
    if (
      weights.threat.drainerWalkThreatBoolean !== 0 &&
      !underDangerThreat &&
      safety.walkThreat
    ) {
      addSigned(multiplier, weights.threat.drainerWalkThreatBoolean);
    }
  };

  const evaluateSpirit = (
    mon: Mon,
    location: Location,
    multiplier: number,
  ): void => {
    const enemyDistance = nearestEnemyMonDistanceWithContext(
      mon.color,
      location,
      context,
    );
    addSignedRatio(
      multiplier,
      weights.position.spiritCloseToEnemy,
      enemyDistance,
    );
    addSigned(
      -multiplier,
      spiritOnOwnBasePenalty(
        game.board,
        mon,
        location,
        weights.position.spiritOnOwnBasePenalty,
      ),
    );
    const utilityCap = useHeuristicFormula ? 4 : 6;
    let utility: number;
    let pressureBonus: number;
    if (useHeuristicFormula) {
      utility = heuristicSpiritActionUtility(game.board, location);
      pressureBonus = 0;
    } else {
      const spirit = exactSummaryForScoring(
        requireExactSummary(myExactSummary),
        requireExactSummary(opponentExactSummary),
        mon.color,
        color,
      ).spirit;
      utility = spirit.utility;
      pressureBonus = exactSpiritPressureBonus(spirit, weights);
    }
    addSigned(
      multiplier,
      weights.threat.spiritActionUtility * Math.min(utility, utilityCap),
    );
    addSigned(multiplier, pressureBonus);
  };

  for (const [location, item] of game.board.entries()) {
    switch (item.kind) {
      case "mon": {
        const mon = item.mon;
        const multiplier = mon.color === color ? 1 : -1;
        if (isMonFainted(mon)) {
          addSigned(
            multiplier,
            mon.kind === MonKind.Drainer
              ? weights.material.faintedDrainer
              : weights.material.faintedMon,
          );
          addSigned(
            multiplier,
            weights.material.faintedCooldownStep * mon.cooldown,
          );
        } else if (mon.kind === MonKind.Drainer) {
          evaluateDrainer(mon, location, multiplier, true);
        } else if (mon.kind === MonKind.Spirit) {
          evaluateSpirit(mon, location, multiplier);
        } else if (mon.kind === MonKind.Angel) {
          addSignedRatio(
            multiplier,
            weights.position.angelCloseToFriendlyDrainer,
            nearestFriendlyDrainerDistanceWithContext(
              mon.color,
              location,
              context,
            ),
          );
        } else {
          addSignedRatio(
            multiplier,
            weights.position.monCloseToCenter,
            distanceToCenter(location),
          );
        }
        if (
          weights.threat.attackerCloseToOpponentDrainer !== 0 &&
          !isMonFainted(mon) &&
          (mon.kind === MonKind.Demon || mon.kind === MonKind.Mystic)
        ) {
          addSignedRatio(
            multiplier,
            weights.threat.attackerCloseToOpponentDrainer,
            nearestFriendlyDrainerDistanceWithContext(
              otherColor(mon.color),
              location,
              context,
            ),
          );
        }
        if (!MON_BASE_INDICES.has(locationIndex(location))) {
          addSigned(multiplier, weights.material.activeMon);
        }
        break;
      }
      case "mon-with-consumable": {
        const mon = item.mon;
        const multiplier = mon.color === color ? 1 : -1;
        addSigned(multiplier, weights.material.hasConsumable);
        if (mon.kind === MonKind.Drainer) {
          evaluateDrainer(mon, location, multiplier, false);
        } else if (mon.kind === MonKind.Spirit) {
          evaluateSpirit(mon, location, multiplier);
        } else if (mon.kind === MonKind.Angel) {
          addSignedRatio(
            multiplier,
            weights.position.angelCloseToFriendlyDrainer,
            nearestFriendlyDrainerDistanceWithContext(
              mon.color,
              location,
              context,
            ),
          );
        } else {
          addSignedRatio(
            multiplier,
            weights.position.monCloseToCenter,
            distanceToCenter(location),
          );
        }
        if (
          weights.threat.attackerCloseToOpponentDrainer !== 0 &&
          !isMonFainted(mon)
        ) {
          const isAttacker =
            mon.kind === MonKind.Demon ||
            mon.kind === MonKind.Mystic ||
            item.consumable === Consumable.Bomb;
          if (isAttacker) {
            addSignedRatio(
              multiplier,
              weights.threat.attackerCloseToOpponentDrainer,
              nearestFriendlyDrainerDistanceWithContext(
                otherColor(mon.color),
                location,
                context,
              ),
            );
          }
        }
        if (
          !useHeuristicFormula &&
          !MON_BASE_INDICES.has(locationIndex(location))
        ) {
          addSigned(multiplier, weights.material.activeMon);
        }
        break;
      }
      case "mana": {
        addScore(
          Math.trunc(
            weights.position.manaCloseToSamePool /
              distanceToClosestPool(location, color),
          ),
        );
        let manaBonus: number;
        if (item.mana.kind === "regular") {
          const manaColor = item.mana.color;
          const ownerMultiplier = manaColor === color ? 1 : -1;
          const ownerPoolDistance = distanceToClosestPool(location, manaColor);
          const ownerDrainerDistance =
            nearestFriendlyDrainerDistanceWithContext(
              manaColor,
              location,
              context,
            );
          const enemyDrainerDistance =
            nearestFriendlyDrainerDistanceWithContext(
              otherColor(manaColor),
              location,
              context,
            );
          const drainerControl = Math.min(
            4,
            Math.max(-4, enemyDrainerDistance - ownerDrainerDistance),
          );
          manaBonus =
            ownerMultiplier *
            (Math.trunc(
              weights.mana.regularManaToOwnerPool / ownerPoolDistance,
            ) +
              weights.mana.regularManaDrainerControl * drainerControl);
          if (!useHeuristicFormula && manaColor === otherColor(color)) {
            manaBonus =
              manaBonus + weights.mana.opponentManaDenial * -drainerControl;
          }
        } else {
          const myDrainerDistance = nearestFriendlyDrainerDistanceWithContext(
            color,
            location,
            context,
          );
          const enemyDrainerDistance =
            nearestFriendlyDrainerDistanceWithContext(
              otherColor(color),
              location,
              context,
            );
          const drainerControl = Math.min(
            4,
            Math.max(-4, enemyDrainerDistance - myDrainerDistance),
          );
          manaBonus =
            weights.mana.supermanaDrainerControl * drainerControl +
            (useHeuristicFormula
              ? 0
              : weights.mana.supermanaRaceControl * drainerControl);
        }
        addScore(manaBonus);
        break;
      }
      case "mon-with-mana": {
        const { mon, mana } = item;
        const multiplier = mon.color === color ? 1 : -1;
        const nearestPoolDistance = distanceToAnyClosestPool(location);
        const manaExtra =
          mana.kind === "supermana"
            ? weights.position.extraForSupermana
            : mana.color === color
              ? 0
              : weights.position.extraForOpponentsMana;
        addSigned(multiplier, weights.position.drainerHoldingMana);
        addSignedRatio(
          multiplier,
          weights.position.monWithManaCloseToAnyPool + manaExtra,
          nearestPoolDistance,
        );
        if (nearestPoolDistance <= 2) {
          const immediateBonus =
            mana.kind === "supermana"
              ? weights.mana.manaCarrierOneStepFromPool +
                weights.mana.supermanaCarrierOneStepFromPoolExtra
              : weights.mana.manaCarrierOneStepFromPool;
          addSigned(multiplier, immediateBonus);
          const carrierScore =
            mon.color === Color.White ? game.whiteScore : game.blackScore;
          if (carrierScore + manaScore(mana, mon.color) >= TARGET_SCORE) {
            addSigned(multiplier, weights.mana.immediateWinningCarrier);
          }
        }

        const carriesHighValueMana =
          !useHeuristicFormula &&
          mon.kind === MonKind.Drainer &&
          (mana.kind === "supermana" || mana.color !== mon.color);
        const safety = drainerSafetySnapshotWithContext(
          mon.color,
          location,
          useHeuristicFormula,
          weights.threat.manaCarrierWalkThreatBoolean !== 0 ||
            carriesHighValueMana,
          context,
        );
        addSignedRatio(
          multiplier,
          weights.mana.manaCarrierAtRisk,
          safety.riskDanger,
        );
        if (guardedAgainstExactAttack(safety)) {
          addSigned(multiplier, weights.mana.manaCarrierGuarded);
        }
        if (
          !useHeuristicFormula &&
          mon.kind === MonKind.Drainer &&
          carriesHighValueMana
        ) {
          let virtualScoreBp: number;
          if (mana.kind === "supermana") {
            virtualScoreBp = saturatingScoreMultiply(
              weights.mana.supermanaRaceControl,
              PROTECTED_HIGH_VALUE_CARRIER_SUPERMANA_SCALE_BP,
            );
          } else if (mana.color !== mon.color) {
            virtualScoreBp = saturatingScoreMultiply(
              weights.mana.opponentManaDenial,
              PROTECTED_HIGH_VALUE_CARRIER_OPPONENT_MANA_SCALE_BP,
            );
          } else {
            virtualScoreBp = 0;
          }
          virtualScoreBp = Math.min(
            PROTECTED_HIGH_VALUE_CARRIER_VIRTUAL_SCORE_BP_MAX,
            Math.max(0, virtualScoreBp),
          );
          const opponentScore =
            mon.color === Color.White ? game.blackScore : game.whiteScore;
          const opponentScoreLimit = Math.max(
            0,
            TARGET_SCORE - PROTECTED_HIGH_VALUE_CARRIER_OPPONENT_SCORE_MARGIN,
          );
          if (
            virtualScoreBp > 0 &&
            exactSafe(safety) &&
            opponentScore <= opponentScoreLimit
          ) {
            const virtualTwoPointScore = saturatingScoreMultiply(
              weights.material.confirmedScore,
              2,
            );
            addSigned(
              multiplier,
              scaleByBp(virtualTwoPointScore, virtualScoreBp),
            );
          }
        }
        if (mon.color === game.activeColor) {
          const poolSteps = nearestPoolDistance - 1;
          if (poolSteps <= remainingMonMovesForActive) {
            addSigned(multiplier, weights.threat.manaCarrierScoreThisTurn);
          }
        }
        if (mon.kind === MonKind.Drainer) {
          addSignedRatio(
            multiplier,
            weights.position.drainerCloseToOwnPool,
            distanceToClosestPool(location, mon.color),
          );
          const [actionThreats, bombThreats] =
            drainerImmediateThreatsWithContext(mon.color, location, context);
          const immediateThreats = safety.angelNearby
            ? bombThreats
            : actionThreats + bombThreats;
          if (immediateThreats > 0) {
            addSigned(
              multiplier,
              weights.threat.drainerImmediateThreat * immediateThreats,
            );
          }
          const evaluateDanger =
            weights.threat.manaCarrierDangerBoolean !== 0 ||
            weights.threat.manaCarrierWalkThreatBoolean !== 0;
          const underDangerThreat = evaluateDanger && safety.exactDangerThreat;
          if (
            weights.threat.manaCarrierDangerBoolean !== 0 &&
            underDangerThreat
          ) {
            addSigned(multiplier, weights.threat.manaCarrierDangerBoolean);
            if (multiplier === -1) {
              addScore(weights.threat.opponentDrainerAttackBonus);
            }
          }
          if (
            weights.threat.manaCarrierWalkThreatBoolean !== 0 &&
            !underDangerThreat &&
            safety.walkThreat
          ) {
            addSigned(multiplier, weights.threat.manaCarrierWalkThreatBoolean);
          }
        } else if (mon.kind === MonKind.Spirit) {
          addSigned(
            -multiplier,
            spiritOnOwnBasePenalty(
              game.board,
              mon,
              location,
              weights.position.spiritOnOwnBasePenalty,
            ),
          );
          const utilityCap = useHeuristicFormula ? 4 : 6;
          let utility: number;
          let pressureBonus: number;
          if (useHeuristicFormula) {
            utility = heuristicSpiritActionUtility(game.board, location);
            pressureBonus = 0;
          } else {
            const spirit = exactSummaryForScoring(
              requireExactSummary(myExactSummary),
              requireExactSummary(opponentExactSummary),
              mon.color,
              color,
            ).spirit;
            utility = spirit.utility;
            pressureBonus = exactSpiritPressureBonus(spirit, weights);
          }
          addSigned(
            multiplier,
            weights.threat.spiritActionUtility * Math.min(utility, utilityCap),
          );
          addSigned(multiplier, pressureBonus);
        }
        if (
          !useHeuristicFormula &&
          !MON_BASE_INDICES.has(locationIndex(location))
        ) {
          addSigned(multiplier, weights.material.activeMon);
        }
        break;
      }
      case "consumable":
        break;
    }
  }

  return score;
}

function scorePreferabilityRaceWindows(
  color: Color,
  weights: ScoringWeights,
  context: ScoringEvalContext,
  useHeuristicFormula: boolean,
  includeRegularManaMoveWindows: boolean,
  myExactSummary: ExactColorSummary | undefined,
  opponentExactSummary: ExactColorSummary | undefined,
  initialScore: number,
): number {
  let score = initialScore;
  const myScorePathWindow = useHeuristicFormula
    ? scorePathWindowToAnyPoolForContext(
        context,
        color,
        false,
        includeRegularManaMoveWindows,
      )
    : exactScorePathWindowForContext(
        context,
        color,
        requireExactSummary(myExactSummary),
        includeRegularManaMoveWindows,
      );
  const opponentScorePathWindow = useHeuristicFormula
    ? scorePathWindowToAnyPoolForContext(
        context,
        otherColor(color),
        false,
        includeRegularManaMoveWindows,
      )
    : exactScorePathWindowForContext(
        context,
        otherColor(color),
        requireExactSummary(opponentExactSummary),
        includeRegularManaMoveWindows,
      );
  if (myScorePathWindow.bestSteps !== undefined) {
    score =
      score +
      scaleByBp(
        Math.trunc(
          weights.race.scoreRacePathProgress /
            Math.max(1, myScorePathWindow.bestSteps),
        ),
        10_000,
      );
    if (!useHeuristicFormula) {
      score =
        score +
        scaleByBp(
          Math.trunc(
            (weights.race.scoreRaceMultiPath *
              myScorePathWindow.multiPressure) /
              100,
          ),
          10_000,
        );
    }
  }
  if (opponentScorePathWindow.bestSteps !== undefined) {
    score =
      score +
      -scaleByBp(
        Math.trunc(
          weights.race.opponentScoreRacePathProgress /
            Math.max(1, opponentScorePathWindow.bestSteps),
        ),
        10_000,
      );
    if (!useHeuristicFormula) {
      score =
        score +
        -scaleByBp(
          Math.trunc(
            (weights.race.opponentScoreRaceMultiPath *
              opponentScorePathWindow.multiPressure) /
              100,
          ),
          10_000,
        );
    }
  }

  return score;
}

function scorePreferabilityImmediateWindows(
  color: Color,
  weights: ScoringWeights,
  context: ScoringEvalContext,
  useHeuristicFormula: boolean,
  includeRegularManaMoveWindows: boolean,
  includeMatchPointWindow: boolean,
  nextTurnWindowScaleBp: number,
  remainingMonMovesForActive: number,
  myExactSummary: ExactColorSummary | undefined,
  opponentExactSummary: ExactColorSummary | undefined,
  myScoreNow: number,
  opponentScoreNow: number,
  initialScore: number,
): number {
  const game = context.game;
  let score = initialScore;
  if (game.activeColor === color) {
    const immediateWindow = useHeuristicFormula
      ? immediateScoreWindowSummaryForContext(
          context,
          color,
          remainingMonMovesForActive,
          false,
          includeRegularManaMoveWindows,
          includeRegularManaMoveWindows && game.playerCanMoveMana(),
        )
      : exactImmediateScoreWindowForContext(
          context,
          color,
          requireExactSummary(myExactSummary),
          includeRegularManaMoveWindows && game.playerCanMoveMana(),
        );
    score =
      score +
      scaleByBp(
        weights.race.immediateScoreWindow * immediateWindow.bestScore,
        10_000,
      );
    if (!useHeuristicFormula) {
      score =
        score +
        scaleByBp(
          Math.trunc(
            (weights.race.immediateScoreMultiWindow *
              immediateWindow.multiPressure) /
              100,
          ),
          10_000,
        );
      const opponentNextTurnWindow = exactImmediateScoreWindowForContext(
        context,
        otherColor(color),
        requireExactSummary(opponentExactSummary),
        includeRegularManaMoveWindows,
      );
      score =
        score +
        -scaleByBp(
          Math.trunc(
            (weights.race.opponentImmediateScoreWindow *
              opponentNextTurnWindow.bestScore *
              nextTurnWindowScaleBp) /
              10_000,
          ),
          10_000,
        );
      score =
        score +
        -scaleByBp(
          Math.trunc(
            (weights.race.opponentImmediateScoreMultiWindow *
              opponentNextTurnWindow.multiPressure *
              nextTurnWindowScaleBp) /
              1_000_000,
          ),
          10_000,
        );
      if (includeMatchPointWindow) {
        if (myScoreNow + immediateWindow.bestScore >= TARGET_SCORE) {
          score = score + weights.mana.immediateWinningCarrier;
        }
        if (
          opponentScoreNow + opponentNextTurnWindow.bestScore >=
          TARGET_SCORE
        ) {
          score = score + -weights.mana.immediateWinningCarrier;
        }
      }
    }
  } else {
    const opponentImmediateWindow = useHeuristicFormula
      ? immediateScoreWindowSummaryForContext(
          context,
          otherColor(color),
          remainingMonMovesForActive,
          false,
          includeRegularManaMoveWindows,
          includeRegularManaMoveWindows && game.playerCanMoveMana(),
        )
      : exactImmediateScoreWindowForContext(
          context,
          otherColor(color),
          requireExactSummary(opponentExactSummary),
          includeRegularManaMoveWindows && game.playerCanMoveMana(),
        );
    score =
      score +
      -scaleByBp(
        weights.race.opponentImmediateScoreWindow *
          opponentImmediateWindow.bestScore,
        10_000,
      );
    if (!useHeuristicFormula) {
      score =
        score +
        -scaleByBp(
          Math.trunc(
            (weights.race.opponentImmediateScoreMultiWindow *
              opponentImmediateWindow.multiPressure) /
              100,
          ),
          10_000,
        );
      const myNextTurnWindow = exactImmediateScoreWindowForContext(
        context,
        color,
        requireExactSummary(myExactSummary),
        includeRegularManaMoveWindows,
      );
      score =
        score +
        scaleByBp(
          Math.trunc(
            (weights.race.immediateScoreWindow *
              myNextTurnWindow.bestScore *
              nextTurnWindowScaleBp) /
              10_000,
          ),
          10_000,
        );
      score =
        score +
        scaleByBp(
          Math.trunc(
            (weights.race.immediateScoreMultiWindow *
              myNextTurnWindow.multiPressure *
              nextTurnWindowScaleBp) /
              1_000_000,
          ),
          10_000,
        );
      if (includeMatchPointWindow) {
        if (
          opponentScoreNow + opponentImmediateWindow.bestScore >=
          TARGET_SCORE
        ) {
          score = score + -weights.mana.immediateWinningCarrier;
        }
        if (myScoreNow + myNextTurnWindow.bestScore >= TARGET_SCORE) {
          score = score + weights.mana.immediateWinningCarrier;
        }
      }
    }
  }

  return score;
}

function requireExactSummary(
  summary: ExactColorSummary | undefined,
): ExactColorSummary {
  if (summary === undefined) {
    throw new Error("exact strategic analysis should be available");
  }
  return summary;
}

function scaleByBp(value: number, basisPoints: number): number {
  return Math.trunc((Math.trunc(value) * Math.trunc(basisPoints)) / 10_000);
}

function exactSummaryForScoring(
  mySummary: ExactColorSummary,
  opponentSummary: ExactColorSummary,
  actorColor: Color,
  perspective: Color,
): ExactColorSummary {
  return actorColor === perspective ? mySummary : opponentSummary;
}

function spiritOnOwnBasePenalty(
  board: Board,
  mon: Mon,
  location: Location,
  penalty: number,
): number {
  return mon.kind === MonKind.Spirit &&
    !isMonFainted(mon) &&
    locationEquals(location, board.base(mon))
    ? penalty
    : 0;
}

type ScorePathWindow = {
  readonly bestSteps: number | undefined;
  readonly multiPressure: number;
};

type ImmediateScoreWindow = {
  readonly bestScore: number;
  readonly multiPressure: number;
};

type ManaPathCandidate = ScoringManaEntry;

type ManaPathSnapshot = {
  readonly candidates: ManaPathCandidate[];
  readonly regularManaMoveScores: readonly [number, number];
};

function manaPathSnapshot(summary: ScoringBoardSummary): ManaPathSnapshot {
  return {
    candidates: summary.manaEntries.slice(),
    regularManaMoveScores: [
      summary.regularManaMoveScores[0],
      summary.regularManaMoveScores[1],
    ],
  };
}

function exactScorePathWindowForContext(
  context: ScoringEvalContext,
  color: Color,
  exactSummary: ExactColorSummary,
  includeRegularManaMoveWindows: boolean,
): ScorePathWindow {
  if (!includeRegularManaMoveWindows) {
    return exactSummary.scorePathWindow;
  }
  const summary = context.boardSummary();
  const candidateSteps = summary.regularManaScorePathSteps[colorSlot(color)];
  const bestSteps =
    candidateSteps === undefined
      ? exactSummary.scorePathWindow.bestSteps
      : exactSummary.scorePathWindow.bestSteps === undefined
        ? candidateSteps
        : Math.min(candidateSteps, exactSummary.scorePathWindow.bestSteps);
  return {
    bestSteps,
    multiPressure: exactSummary.scorePathWindow.multiPressure,
  };
}

function scorePathWindowToAnyPoolForContext(
  context: ScoringEvalContext,
  color: Color,
  includeDrainerPickups: boolean,
  includeRegularManaMoveWindows: boolean,
): ScorePathWindow {
  const summary = context.boardSummary();
  const topSteps = [0x7fff_ffff, 0x7fff_ffff, 0x7fff_ffff];
  for (const carrier of summary.liveManaCarriers[colorSlot(color)]) {
    insertLowestStep(topSteps, carrier.scoreSteps + 1);
  }
  if (includeDrainerPickups) {
    const snapshot = context.manaPathSnapshot();
    for (const location of summary.liveDrainerLocations[colorSlot(color)]) {
      const pickup = bestDrainerPickupPathWithSnapshot(
        snapshot,
        color,
        location,
      );
      if (pickup !== undefined) {
        insertLowestStep(topSteps, pickup[0] + 1);
      }
    }
  }
  if (includeRegularManaMoveWindows) {
    const candidate = summary.regularManaScorePathSteps[colorSlot(color)];
    if (candidate !== undefined) {
      insertLowestStep(topSteps, candidate);
    }
  }
  const bestSteps = topSteps[0] === 0x7fff_ffff ? undefined : topSteps[0];
  let multiPressure = 0;
  if (topSteps[1] !== 0x7fff_ffff) {
    multiPressure =
      multiPressure + Math.trunc(70 / Math.max(1, topSteps[1] ?? 1));
  }
  if (topSteps[2] !== 0x7fff_ffff) {
    multiPressure =
      multiPressure + Math.trunc(40 / Math.max(1, topSteps[2] ?? 1));
  }
  return { bestSteps, multiPressure };
}

function exactImmediateScoreWindowForContext(
  context: ScoringEvalContext,
  color: Color,
  exactSummary: ExactColorSummary,
  allowManaMove: boolean,
): ImmediateScoreWindow {
  if (!allowManaMove) {
    return exactSummary.immediateWindow;
  }
  const regularScore =
    context.boardSummary().regularManaMoveScores[colorSlot(color)];
  return {
    bestScore: Math.max(exactSummary.immediateWindow.bestScore, regularScore),
    multiPressure: exactSummary.immediateWindow.multiPressure,
  };
}

function immediateScoreWindowSummaryForContext(
  context: ScoringEvalContext,
  color: Color,
  remainingMonMoves: number,
  includeDrainerPickups: boolean,
  includeRegularManaMoveWindows: boolean,
  allowManaMove: boolean,
): ImmediateScoreWindow {
  if (remainingMonMoves <= 0) {
    return { bestScore: 0, multiPressure: 0 };
  }
  const summary = context.boardSummary();
  const topScores = [0, 0, 0];
  for (const carrier of summary.liveManaCarriers[colorSlot(color)]) {
    if (carrier.scoreSteps <= remainingMonMoves) {
      insertTopScore(topScores, manaScore(carrier.mana, color));
    }
  }
  if (includeDrainerPickups) {
    const snapshot = context.manaPathSnapshot();
    for (const location of summary.liveDrainerLocations[colorSlot(color)]) {
      let bestPickupScore = 0;
      for (const candidate of snapshot.candidates) {
        const pickupSteps = locationDistance(location, candidate.location);
        if (pickupSteps + candidate.scoreSteps <= remainingMonMoves) {
          bestPickupScore = Math.max(
            bestPickupScore,
            manaScore(candidate.mana, color),
          );
        }
      }
      if (bestPickupScore > 0) {
        insertTopScore(topScores, bestPickupScore);
      }
    }
  }
  if (includeRegularManaMoveWindows && allowManaMove) {
    const regularScore = summary.regularManaMoveScores[colorSlot(color)];
    if (regularScore > 0) {
      insertTopScore(topScores, regularScore);
    }
  }
  return {
    bestScore: topScores[0] ?? 0,
    multiPressure: (topScores[1] ?? 0) * 70 + (topScores[2] ?? 0) * 35,
  };
}

function exactSpiritPressureBonus(
  spirit: ExactSpiritSummary,
  weights: ScoringWeights,
): number {
  const setupGain = Math.min(4, Math.max(0, spirit.nextTurnSetupGain));
  let bonus = 0;
  if (setupGain > 0) {
    bonus = saturatingScoreAdd(
      bonus,
      Math.trunc(
        saturatingScoreMultiply(
          Math.max(0, weights.race.scoreRacePathProgress),
          setupGain,
        ) / 4,
      ),
    );
    bonus = saturatingScoreAdd(
      bonus,
      Math.trunc(
        saturatingScoreMultiply(
          Math.max(0, weights.race.opponentScoreRacePathProgress),
          setupGain,
        ) / 6,
      ),
    );
    bonus = saturatingScoreAdd(
      bonus,
      Math.trunc(
        saturatingScoreMultiply(
          Math.max(0, weights.race.scoreRaceMultiPath),
          setupGain,
        ) / 8,
      ),
    );
    bonus = saturatingScoreAdd(
      bonus,
      Math.trunc(
        saturatingScoreMultiply(
          Math.max(0, weights.race.opponentScoreRaceMultiPath),
          setupGain,
        ) / 10,
      ),
    );
  }
  if (spirit.supermanaProgress && !spirit.sameTurnScore) {
    bonus = saturatingScoreAdd(
      saturatingScoreAdd(
        bonus,
        saturatingScoreMultiply(
          Math.max(0, weights.mana.supermanaRaceControl),
          3,
        ),
      ),
      Math.trunc(Math.max(0, weights.threat.drainerBestManaPath) / 4),
    );
  }
  if (spirit.opponentManaProgress && !spirit.sameTurnOpponentManaScore) {
    bonus = saturatingScoreAdd(
      saturatingScoreAdd(
        saturatingScoreAdd(
          bonus,
          saturatingScoreMultiply(
            Math.max(0, weights.mana.opponentManaDenial),
            3,
          ),
        ),
        Math.trunc(Math.max(0, weights.threat.drainerBestManaPath) / 4),
      ),
      Math.trunc(Math.max(0, weights.race.scoreRacePathProgress) / 5),
    );
  }
  return bonus;
}

function heuristicSpiritActionUtility(
  board: Board,
  location: Location,
): number {
  let utility = 0;
  for (const target of spiritReachableLocations(location)) {
    const item = board.get(target);
    if (item === undefined) continue;
    if (isSpiritTargetAllowed(item)) {
      utility += 1;
    }
  }
  return utility;
}

function colorSlot(color: Color): 0 | 1 {
  return color === Color.White ? 0 : 1;
}

function insertLowestStep(topSteps: number[], step: number): void {
  if (step >= (topSteps[2] ?? 0x7fff_ffff)) {
    return;
  }
  if (step < (topSteps[0] ?? 0x7fff_ffff)) {
    topSteps[2] = topSteps[1] ?? 0x7fff_ffff;
    topSteps[1] = topSteps[0] ?? 0x7fff_ffff;
    topSteps[0] = step;
  } else if (step < (topSteps[1] ?? 0x7fff_ffff)) {
    topSteps[2] = topSteps[1] ?? 0x7fff_ffff;
    topSteps[1] = step;
  } else {
    topSteps[2] = step;
  }
}

function insertTopScore(topScores: number[], value: number): void {
  if (value <= (topScores[2] ?? 0)) {
    return;
  }
  if (value > (topScores[0] ?? 0)) {
    topScores[2] = topScores[1] ?? 0;
    topScores[1] = topScores[0] ?? 0;
    topScores[0] = value;
  } else if (value > (topScores[1] ?? 0)) {
    topScores[2] = topScores[1] ?? 0;
    topScores[1] = value;
  } else {
    topScores[2] = value;
  }
}

function bestDrainerPickupPathWithSnapshot(
  snapshot: ManaPathSnapshot,
  color: Color,
  from: Location,
): readonly [number, number] | undefined {
  let best: readonly [number, number] | undefined;
  for (const candidate of snapshot.candidates) {
    const pickupSteps = locationDistance(from, candidate.location);
    const totalSteps = pickupSteps + candidate.scoreSteps;
    const manaValue = manaScore(candidate.mana, color);
    if (best === undefined) {
      best = [totalSteps, manaValue];
      continue;
    }
    const metric = totalSteps * 3 - manaValue;
    const bestMetric = best[0] * 3 - best[1];
    if (metric < bestMetric || (metric === bestMetric && manaValue > best[1])) {
      best = [totalSteps, manaValue];
    }
  }
  return best;
}

function drainerDistancesWithContext(
  color: Color,
  location: Location,
  useHeuristicFormula: boolean,
  context: ScoringEvalContext,
): readonly [number, number, boolean] {
  const summary = context.boardSummary();
  let minMana = BOARD_SIZE;
  let minDanger = BOARD_SIZE;
  for (const entry of summary.manaEntries) {
    minMana = Math.min(minMana, locationDistance(entry.location, location));
  }
  for (const danger of summary.dangerSources[colorSlot(otherColor(color))]) {
    if (useHeuristicFormula) {
      if (danger.heuristicThreat) {
        minDanger = Math.min(
          minDanger,
          locationDistance(danger.location, location),
        );
      }
      continue;
    }
    let delta = 0x7fff_ffff;
    if (danger.exactActionThreat) {
      delta = locationDistance(danger.location, location);
    }
    if (danger.exactBombThreat) {
      const bombDelta = Math.max(
        1,
        locationDistance(danger.location, location) - 2,
      );
      delta = Math.min(delta, bombDelta);
    }
    minDanger = Math.min(minDanger, delta);
  }
  if (useHeuristicFormula) {
    for (const consumable of summary.looseConsumableLocations) {
      minDanger = Math.min(minDanger, locationDistance(consumable, location));
    }
  }
  const angelNearby = summary.liveAngelLocations[colorSlot(color)].some(
    (angel) => locationDistance(angel, location) === 1,
  );
  return useHeuristicFormula
    ? [minDanger, minMana, angelNearby]
    : [Math.max(1, minDanger), Math.max(1, minMana), angelNearby];
}

function drainerSafetySnapshotWithContext(
  color: Color,
  location: Location,
  useHeuristicFormula: boolean,
  includeWalkThreat: boolean,
  context: ScoringEvalContext,
): DrainerSafetySnapshot {
  const board = context.board;
  const [rawDanger, minMana, angelNearby] = drainerDistancesWithContext(
    color,
    location,
    useHeuristicFormula,
    context,
  );
  const exactDangerThreat = useHeuristicFormula
    ? isDrainerUnderImmediateThreat(
        context.execution,
        board,
        color,
        location,
        angelNearby,
      )
    : context.canAttackTargetOnBoard(
        otherColor(color),
        color,
        location,
        MONS_MOVES_PER_TURN,
        true,
      );
  const walkThreat =
    includeWalkThreat &&
    !exactDangerThreat &&
    isDrainerUnderWalkThreatWithHash(
      context.execution,
      board,
      context.boardHash,
      color,
      location,
      angelNearby,
    );
  return {
    riskDanger: useHeuristicFormula
      ? Math.max(1, rawDanger)
      : exactDangerThreat
        ? 1
        : Math.max(1, rawDanger),
    minMana,
    angelNearby,
    exactDangerThreat,
    walkThreat,
  };
}

function drainerImmediateThreatsWithContext(
  color: Color,
  location: Location,
  context: ScoringEvalContext,
): readonly [number, number] {
  return context.drainerImmediateThreats(color, location);
}

function nearestEnemyMonDistanceWithContext(
  color: Color,
  location: Location,
  context: ScoringEvalContext,
): number {
  let best = BOARD_SIZE;
  for (const occupied of context.boardSummary().liveMonLocations[
    colorSlot(otherColor(color))
  ]) {
    best = Math.min(best, locationDistance(occupied, location));
  }
  return Math.max(1, best);
}

function nearestFriendlyDrainerDistanceWithContext(
  color: Color,
  location: Location,
  context: ScoringEvalContext,
): number {
  let best = BOARD_SIZE;
  for (const occupied of context.boardSummary().liveDrainerLocations[
    colorSlot(color)
  ]) {
    best = Math.min(best, locationDistance(occupied, location));
  }
  return Math.max(1, best);
}

function distanceToLocation(location: Location, destination: Location): number {
  return locationDistance(location, destination) + 1;
}

function distanceToCenter(location: Location): number {
  return Math.max(1, Math.abs(BOARD_CENTER_INDEX - location.i)) + 1;
}

function distanceToAnyClosestPool(location: Location): number {
  return (
    Math.max(
      Math.min(location.i, Math.abs(MAX_LOCATION_INDEX - location.i)),
      Math.min(location.j, Math.abs(MAX_LOCATION_INDEX - location.j)),
    ) + 1
  );
}

function distanceToClosestPool(location: Location, color: Color): number {
  const poolRow = color === Color.White ? MAX_LOCATION_INDEX : 0;
  return (
    Math.max(
      Math.abs(poolRow - location.i),
      Math.min(location.j, Math.abs(MAX_LOCATION_INDEX - location.j)),
    ) + 1
  );
}
