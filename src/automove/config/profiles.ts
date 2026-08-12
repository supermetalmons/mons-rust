import { TARGET_SCORE } from "../../engine/board/config.js";
import { Color } from "../../api/types.js";
import type { MonsGame } from "../../engine/game/mons-game.js";
import {
  BALANCED_DISTANCE_SCORING_WEIGHTS,
  FINISHER_BALANCED_SOFT_AGGRESSIVE_SCORING_WEIGHTS,
  FINISHER_BALANCED_SOFT_SCORING_WEIGHTS,
  RUNTIME_FAST_BOOLEAN_DRAINER_SCORING_WEIGHTS,
  TACTICAL_BALANCED_AGGRESSIVE_SCORING_WEIGHTS,
  TACTICAL_BALANCED_SCORING_WEIGHTS,
} from "../scoring/presets.js";
import { defineScoringProfile } from "../scoring/profile-validation.js";
import type { ScoringWeights } from "../scoring/profile-schema.js";

const NORMAL_BASE_PROFILES = Object.freeze({
  balanced: defineScoringProfile({
    id: "runtime-normal.base.balanced",
    base: BALANCED_DISTANCE_SCORING_WEIGHTS,
    formula: { useHeuristicFormula: false },
    material: { confirmedScore: 900 },
    position: {
      spiritOnOwnBasePenalty: 260,
      angelGuardingDrainer: 120,
      drainerHoldingMana: 470,
      drainerCloseToMana: 360,
    },
    mana: {
      supermanaRaceControl: 30,
      opponentManaDenial: 24,
    },
    race: {
      scoreRacePathProgress: 86,
      opponentScoreRacePathProgress: 184,
      immediateScoreWindow: 96,
      opponentImmediateScoreWindow: 245,
    },
    threat: {
      drainerImmediateThreat: -55,
      drainerBestManaPath: 58,
      drainerPickupScoreThisTurn: 90,
      manaCarrierScoreThisTurn: 150,
      spiritActionUtility: 86,
    },
  }),
  tactical: defineScoringProfile({
    id: "runtime-normal.base.tactical",
    base: TACTICAL_BALANCED_SCORING_WEIGHTS,
    formula: { useHeuristicFormula: false },
    material: { confirmedScore: 900 },
    position: {
      spiritOnOwnBasePenalty: 260,
      angelGuardingDrainer: 180,
      drainerHoldingMana: 500,
      drainerCloseToMana: 390,
    },
    mana: {
      supermanaRaceControl: 34,
      opponentManaDenial: 30,
    },
    race: {
      scoreRacePathProgress: 94,
      opponentScoreRacePathProgress: 220,
      immediateScoreWindow: 102,
      opponentImmediateScoreWindow: 310,
    },
    threat: {
      drainerImmediateThreat: -90,
      drainerBestManaPath: 84,
      drainerPickupScoreThisTurn: 110,
      manaCarrierScoreThisTurn: 180,
      spiritActionUtility: 90,
    },
  }),
  tacticalAggressive: defineScoringProfile({
    id: "runtime-normal.base.tactical-aggressive",
    base: TACTICAL_BALANCED_AGGRESSIVE_SCORING_WEIGHTS,
    formula: { useHeuristicFormula: false },
    material: { confirmedScore: 890 },
    position: {
      spiritOnOwnBasePenalty: 260,
      angelGuardingDrainer: 190,
      drainerHoldingMana: 520,
      drainerCloseToMana: 410,
    },
    mana: {
      supermanaRaceControl: 40,
      opponentManaDenial: 34,
    },
    race: {
      scoreRacePathProgress: 104,
      opponentScoreRacePathProgress: 255,
      immediateScoreWindow: 114,
      opponentImmediateScoreWindow: 360,
    },
    threat: {
      drainerImmediateThreat: -120,
      drainerBestManaPath: 96,
      drainerPickupScoreThisTurn: 130,
      manaCarrierScoreThisTurn: 220,
      spiritActionUtility: 94,
    },
  }),
  finisher: defineScoringProfile({
    id: "runtime-normal.base.finisher",
    base: FINISHER_BALANCED_SOFT_SCORING_WEIGHTS,
    formula: { useHeuristicFormula: false },
    material: { confirmedScore: 930 },
    position: {
      spiritOnOwnBasePenalty: 260,
      drainerHoldingMana: 500,
      drainerCloseToMana: 375,
      angelGuardingDrainer: 170,
    },
    mana: {
      supermanaRaceControl: 32,
      opponentManaDenial: 28,
    },
    race: {
      scoreRacePathProgress: 170,
      opponentScoreRacePathProgress: 170,
      immediateScoreWindow: 275,
      opponentImmediateScoreWindow: 235,
    },
    threat: {
      drainerBestManaPath: 72,
      drainerPickupScoreThisTurn: 120,
      manaCarrierScoreThisTurn: 240,
      spiritActionUtility: 88,
    },
  }),
  finisherAggressive: defineScoringProfile({
    id: "runtime-normal.base.finisher-aggressive",
    base: FINISHER_BALANCED_SOFT_AGGRESSIVE_SCORING_WEIGHTS,
    formula: { useHeuristicFormula: false },
    material: { confirmedScore: 940 },
    position: {
      spiritOnOwnBasePenalty: 260,
      drainerHoldingMana: 520,
      drainerCloseToMana: 395,
      angelGuardingDrainer: 180,
    },
    mana: {
      supermanaRaceControl: 36,
      opponentManaDenial: 30,
    },
    race: {
      scoreRacePathProgress: 195,
      opponentScoreRacePathProgress: 185,
      immediateScoreWindow: 330,
      opponentImmediateScoreWindow: 265,
    },
    threat: {
      drainerBestManaPath: 84,
      drainerPickupScoreThisTurn: 140,
      manaCarrierScoreThisTurn: 280,
      spiritActionUtility: 90,
    },
  }),
});

function withBooleanDrainer(id: string, base: ScoringWeights): ScoringWeights {
  return defineScoringProfile({
    id,
    base,
    threat: {
      drainerDangerBoolean: -1_200,
      manaCarrierDangerBoolean: -800,
    },
  });
}

const NORMAL_BOOLEAN_PROFILES = Object.freeze({
  balanced: withBooleanDrainer(
    "runtime-normal.boolean.balanced",
    NORMAL_BASE_PROFILES.balanced,
  ),
  tactical: withBooleanDrainer(
    "runtime-normal.boolean.tactical",
    NORMAL_BASE_PROFILES.tactical,
  ),
  tacticalAggressive: withBooleanDrainer(
    "runtime-normal.boolean.tactical-aggressive",
    NORMAL_BASE_PROFILES.tacticalAggressive,
  ),
  finisher: withBooleanDrainer(
    "runtime-normal.boolean.finisher",
    NORMAL_BASE_PROFILES.finisher,
  ),
  finisherAggressive: withBooleanDrainer(
    "runtime-normal.boolean.finisher-aggressive",
    NORMAL_BASE_PROFILES.finisherAggressive,
  ),
});

function withMediumWalkThreat(id: string, base: ScoringWeights): ScoringWeights {
  return defineScoringProfile({
    id,
    base,
    threat: {
      drainerWalkThreatBoolean: -300,
      manaCarrierWalkThreatBoolean: -150,
    },
  });
}

function withAttackerProximity(id: string, base: ScoringWeights): ScoringWeights {
  return defineScoringProfile({
    id,
    base,
    threat: { attackerCloseToOpponentDrainer: 200 },
  });
}

type ScoringProfile = {
  readonly weights: ScoringWeights;
};

function phaseProfile(
  game: MonsGame,
  balanced: ScoringProfile,
  tactical: ScoringProfile,
  tacticalAggressive: ScoringProfile,
  finisher: ScoringProfile,
  finisherAggressive: ScoringProfile,
): ScoringProfile {
  const [myScore, opponentScore] =
    game.activeColor === Color.White
      ? [game.whiteScore, game.blackScore]
      : [game.blackScore, game.whiteScore];
  const myDistanceToWin = TARGET_SCORE - myScore;
  const opponentDistanceToWin = TARGET_SCORE - opponentScore;
  const scoreGap = myScore - opponentScore;

  if (myDistanceToWin <= 1) return finisherAggressive;
  if (opponentDistanceToWin <= 1) return tacticalAggressive;
  if (myDistanceToWin <= 2) return finisher;
  if (opponentDistanceToWin <= 2 || scoreGap <= -1) return tactical;
  return balanced;
}

const WALK_PROFILES = Object.freeze({
  balanced: Object.freeze({
    weights: withMediumWalkThreat(
      "runtime-normal-walk-balanced",
      NORMAL_BOOLEAN_PROFILES.balanced,
    ),
  }),
  tactical: Object.freeze({
    weights: withMediumWalkThreat(
      "runtime-normal-walk-tactical",
      NORMAL_BOOLEAN_PROFILES.tactical,
    ),
  }),
  tacticalAggressive: Object.freeze({
    weights: withMediumWalkThreat(
      "runtime-normal-walk-tactical-aggressive",
      NORMAL_BOOLEAN_PROFILES.tacticalAggressive,
    ),
  }),
  finisher: Object.freeze({
    weights: withMediumWalkThreat(
      "runtime-normal-walk-finisher",
      NORMAL_BOOLEAN_PROFILES.finisher,
    ),
  }),
  finisherAggressive: Object.freeze({
    weights: withMediumWalkThreat(
      "runtime-normal-walk-finisher-aggressive",
      NORMAL_BOOLEAN_PROFILES.finisherAggressive,
    ),
  }),
});

const ATTACKER_PROFILES = Object.freeze({
  balanced: Object.freeze({
    weights: withAttackerProximity(
      "runtime-normal-attacker-balanced",
      NORMAL_BOOLEAN_PROFILES.balanced,
    ),
  }),
  tactical: Object.freeze({
    weights: withAttackerProximity(
      "runtime-normal-attacker-tactical",
      NORMAL_BOOLEAN_PROFILES.tactical,
    ),
  }),
  tacticalAggressive: Object.freeze({
    weights: withAttackerProximity(
      "runtime-normal-attacker-tactical-aggressive",
      NORMAL_BOOLEAN_PROFILES.tacticalAggressive,
    ),
  }),
  finisher: Object.freeze({
    weights: withAttackerProximity(
      "runtime-normal-attacker-finisher",
      NORMAL_BOOLEAN_PROFILES.finisher,
    ),
  }),
  finisherAggressive: Object.freeze({
    weights: withAttackerProximity(
      "runtime-normal-attacker-finisher-aggressive",
      NORMAL_BOOLEAN_PROFILES.finisherAggressive,
    ),
  }),
});

export function runtimePhaseAdaptiveWalkThreatMediumScoringProfile(
  game: MonsGame,
  depth: number,
): ScoringProfile {
  if (depth < 3) {
    return Object.freeze({
      weights: RUNTIME_FAST_BOOLEAN_DRAINER_SCORING_WEIGHTS,
    });
  }
  return phaseProfile(
    game,
    WALK_PROFILES.balanced,
    WALK_PROFILES.tactical,
    WALK_PROFILES.tacticalAggressive,
    WALK_PROFILES.finisher,
    WALK_PROFILES.finisherAggressive,
  );
}

export function runtimePhaseAdaptiveAttackerProximityScoringProfile(
  game: MonsGame,
  depth: number,
): ScoringProfile {
  if (depth < 3) {
    return Object.freeze({
      weights: RUNTIME_FAST_BOOLEAN_DRAINER_SCORING_WEIGHTS,
    });
  }
  return phaseProfile(
    game,
    ATTACKER_PROFILES.balanced,
    ATTACKER_PROFILES.tactical,
    ATTACKER_PROFILES.tacticalAggressive,
    ATTACKER_PROFILES.finisher,
    ATTACKER_PROFILES.finisherAggressive,
  );
}
