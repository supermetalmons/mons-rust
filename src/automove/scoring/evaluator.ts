import type { AutomoveExecutionContext } from "../core/execution-context.js";
import { Color, Consumable, MonKind, type Mon } from "../../api/types.js";
import { isMonFainted, manaScore, otherColor } from "../../engine/model/domain.js";
import { locationIndex, type Location } from "../../engine/board/geometry.js";
import {
  MON_BASE_LOCATIONS,
  MONS_MOVES_PER_TURN,
  TARGET_SCORE,
} from "../../engine/board/config.js";
import { MonsGame } from "../../engine/game/mons-game.js";
import { clampHeuristicScore, saturatingScoreMultiply } from "../core/score-math.js";
import type { ScoringWeights } from "./profile-schema.js";
import type { ExactColorSummary, ExactStrategicAnalysis } from "../exact/types.js";
import { ScoringEvalContext } from "./context.js";
import {
  bestDrainerPickupPathWithSnapshot,
  distanceToAnyClosestPool,
  distanceToCenter,
  distanceToClosestPool,
  distanceToLocation,
  drainerImmediateThreatsWithContext,
  drainerSafetySnapshotWithContext,
  exactSafe,
  exactSpiritPressureBonus,
  exactSummaryForScoring,
  guardedAgainstExactAttack,
  heuristicSpiritActionUtility,
  nearestEnemyMonDistanceWithContext,
  nearestFriendlyDrainerDistanceWithContext,
  requireExactSummary,
  spiritOnOwnBasePenalty,
} from "./threat-support.js";
import {
  scaleByBp,
  scorePreferabilityImmediateWindows,
  scorePreferabilityRaceWindows,
} from "./windows.js";

const PROTECTED_HIGH_VALUE_CARRIER_SUPERMANA_SCALE_BP = 2_500;
const PROTECTED_HIGH_VALUE_CARRIER_OPPONENT_MANA_SCALE_BP = 2_500;
const PROTECTED_HIGH_VALUE_CARRIER_VIRTUAL_SCORE_BP_MAX = 9_200;
const PROTECTED_HIGH_VALUE_CARRIER_OPPONENT_SCORE_MARGIN = 2;
const MON_BASE_INDICES = new Set(MON_BASE_LOCATIONS.map(locationIndex));

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
  const exactAnalysis = useHeuristicFormula ? undefined : context.exactAnalysis();
  const myExactSummary = exactAnalysis?.colorSummary(color);
  const opponentExactSummary = exactAnalysis?.colorSummary(otherColor(color));
  const myScoreNow = color === Color.White ? game.whiteScore : game.blackScore;
  const opponentScoreNow = color === Color.White ? game.blackScore : game.whiteScore;

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
  const addSignedRatio = (multiplier: number, value: number, divisor: number): void => {
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
    addSignedRatio(multiplier, weights.position.drainerCloseToMana, safety.minMana);
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
      addSignedRatio(multiplier, weights.position.drainerAtRisk, safety.riskDanger);
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
        const totalMoves = "totalMoves" in path ? path.totalMoves : pathSteps + 1;
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
          addSigned(multiplier, weights.threat.drainerPickupScoreThisTurn * manaValue);
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
        addSigned(multiplier, weights.threat.drainerImmediateThreat * immediateThreats);
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

  const evaluateSpirit = (mon: Mon, location: Location, multiplier: number): void => {
    const enemyDistance = nearestEnemyMonDistanceWithContext(
      mon.color,
      location,
      context,
    );
    addSignedRatio(multiplier, weights.position.spiritCloseToEnemy, enemyDistance);
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
          addSigned(multiplier, weights.material.faintedCooldownStep * mon.cooldown);
        } else if (mon.kind === MonKind.Drainer) {
          evaluateDrainer(mon, location, multiplier, true);
        } else if (mon.kind === MonKind.Spirit) {
          evaluateSpirit(mon, location, multiplier);
        } else if (mon.kind === MonKind.Angel) {
          addSignedRatio(
            multiplier,
            weights.position.angelCloseToFriendlyDrainer,
            nearestFriendlyDrainerDistanceWithContext(mon.color, location, context),
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
            nearestFriendlyDrainerDistanceWithContext(mon.color, location, context),
          );
        } else {
          addSignedRatio(
            multiplier,
            weights.position.monCloseToCenter,
            distanceToCenter(location),
          );
        }
        if (weights.threat.attackerCloseToOpponentDrainer !== 0 && !isMonFainted(mon)) {
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
        if (!useHeuristicFormula && !MON_BASE_INDICES.has(locationIndex(location))) {
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
          const ownerDrainerDistance = nearestFriendlyDrainerDistanceWithContext(
            manaColor,
            location,
            context,
          );
          const enemyDrainerDistance = nearestFriendlyDrainerDistanceWithContext(
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
            (Math.trunc(weights.mana.regularManaToOwnerPool / ownerPoolDistance) +
              weights.mana.regularManaDrainerControl * drainerControl);
          if (!useHeuristicFormula && manaColor === otherColor(color)) {
            manaBonus = manaBonus + weights.mana.opponentManaDenial * -drainerControl;
          }
        } else {
          const myDrainerDistance = nearestFriendlyDrainerDistanceWithContext(
            color,
            location,
            context,
          );
          const enemyDrainerDistance = nearestFriendlyDrainerDistanceWithContext(
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
          weights.threat.manaCarrierWalkThreatBoolean !== 0 || carriesHighValueMana,
          context,
        );
        addSignedRatio(multiplier, weights.mana.manaCarrierAtRisk, safety.riskDanger);
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
            addSigned(multiplier, scaleByBp(virtualTwoPointScore, virtualScoreBp));
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
          const evaluateDanger =
            weights.threat.manaCarrierDangerBoolean !== 0 ||
            weights.threat.manaCarrierWalkThreatBoolean !== 0;
          const underDangerThreat = evaluateDanger && safety.exactDangerThreat;
          if (weights.threat.manaCarrierDangerBoolean !== 0 && underDangerThreat) {
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
        if (!useHeuristicFormula && !MON_BASE_INDICES.has(locationIndex(location))) {
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
