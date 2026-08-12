import {
  defineScoringProfile,
  freezeScoringSection,
  normalizeScoringProfile,
} from "./profile-validation.js";
import type {
  EvaluationFormulaWeights,
  ManaWeights,
  MaterialWeights,
  PositionWeights,
  RaceWeights,
  ScoringWeights,
  ThreatWeights,
} from "./profile-schema.js";

const DEFAULT_FORMULA = freezeScoringSection<EvaluationFormulaWeights>({
  useHeuristicFormula: true,
  includeRegularManaMoveWindows: false,
  includeMatchPointWindow: false,
  nextTurnWindowScaleBp: 5_000,
  doubleConfirmedScore: true,
});

const DEFAULT_MATERIAL = freezeScoringSection<MaterialWeights>({
  confirmedScore: 1_000,
  faintedMon: -500,
  faintedDrainer: -800,
  faintedCooldownStep: 0,
  hasConsumable: 110,
  activeMon: 50,
});

const DEFAULT_POSITION = freezeScoringSection<PositionWeights>({
  drainerAtRisk: -350,
  manaCloseToSamePool: 500,
  monWithManaCloseToAnyPool: 800,
  extraForSupermana: 120,
  extraForOpponentsMana: 100,
  drainerCloseToMana: 300,
  drainerHoldingMana: 350,
  drainerCloseToOwnPool: 180,
  drainerCloseToSupermana: 120,
  monCloseToCenter: 210,
  spiritCloseToEnemy: 160,
  spiritOnOwnBasePenalty: 180,
  angelGuardingDrainer: 180,
  angelCloseToFriendlyDrainer: 120,
});

const DEFAULT_MANA = freezeScoringSection<ManaWeights>({
  regularManaToOwnerPool: 0,
  regularManaDrainerControl: 0,
  supermanaDrainerControl: 0,
  supermanaRaceControl: 0,
  opponentManaDenial: 0,
  manaCarrierAtRisk: 0,
  manaCarrierGuarded: 0,
  manaCarrierOneStepFromPool: 0,
  supermanaCarrierOneStepFromPoolExtra: 0,
  immediateWinningCarrier: 0,
});

const DEFAULT_RACE = freezeScoringSection<RaceWeights>({
  scoreRacePathProgress: 0,
  opponentScoreRacePathProgress: 0,
  scoreRaceMultiPath: 0,
  opponentScoreRaceMultiPath: 0,
  immediateScoreWindow: 0,
  opponentImmediateScoreWindow: 0,
  immediateScoreMultiWindow: 0,
  opponentImmediateScoreMultiWindow: 0,
});

const DEFAULT_THREAT = freezeScoringSection<ThreatWeights>({
  drainerBestManaPath: 0,
  drainerPickupScoreThisTurn: 0,
  manaCarrierScoreThisTurn: 0,
  drainerImmediateThreat: 0,
  spiritActionUtility: 0,
  drainerDangerBoolean: 0,
  manaCarrierDangerBoolean: 0,
  drainerWalkThreatBoolean: 0,
  manaCarrierWalkThreatBoolean: 0,
  opponentDrainerAttackBonus: 0,
  attackerCloseToOpponentDrainer: 0,
});

export const DEFAULT_SCORING_WEIGHTS: ScoringWeights = normalizeScoringProfile({
  id: "default",
  formula: DEFAULT_FORMULA,
  material: DEFAULT_MATERIAL,
  position: DEFAULT_POSITION,
  mana: DEFAULT_MANA,
  race: DEFAULT_RACE,
  threat: DEFAULT_THREAT,
});

export const BALANCED_DISTANCE_SCORING_WEIGHTS = defineScoringProfile({
  id: "balanced-distance",
  base: DEFAULT_SCORING_WEIGHTS,
  material: {
    faintedMon: -520,
    faintedDrainer: -900,
    faintedCooldownStep: -80,
    hasConsumable: 105,
    activeMon: 45,
  },
  position: {
    drainerAtRisk: -420,
    manaCloseToSamePool: 520,
    monWithManaCloseToAnyPool: 820,
    extraForSupermana: 130,
    extraForOpponentsMana: 120,
    drainerCloseToMana: 330,
    drainerHoldingMana: 370,
    drainerCloseToOwnPool: 280,
    drainerCloseToSupermana: 180,
    monCloseToCenter: 180,
    spiritCloseToEnemy: 220,
    angelGuardingDrainer: 280,
    angelCloseToFriendlyDrainer: 180,
  },
  mana: {
    manaCarrierOneStepFromPool: 160,
    supermanaCarrierOneStepFromPoolExtra: 80,
  },
});

const MANA_RACE_LITE_SCORING_WEIGHTS = defineScoringProfile({
  id: "mana-race-lite",
  base: BALANCED_DISTANCE_SCORING_WEIGHTS,
  material: { faintedCooldownStep: -70 },
  position: {
    manaCloseToSamePool: 420,
    drainerCloseToOwnPool: 290,
    drainerCloseToSupermana: 200,
    angelGuardingDrainer: 290,
  },
  mana: {
    regularManaToOwnerPool: 150,
    regularManaDrainerControl: 15,
    supermanaDrainerControl: 26,
    manaCarrierAtRisk: -150,
    manaCarrierGuarded: 70,
    manaCarrierOneStepFromPool: 220,
    supermanaCarrierOneStepFromPoolExtra: 120,
    immediateWinningCarrier: 0,
  },
});

export const FINISHER_BALANCED_SOFT_SCORING_WEIGHTS = defineScoringProfile({
  id: "finisher-balanced",
  base: BALANCED_DISTANCE_SCORING_WEIGHTS,
  mana: {
    manaCarrierOneStepFromPool: 220,
    supermanaCarrierOneStepFromPoolExtra: 110,
    immediateWinningCarrier: 360,
  },
});

export const FINISHER_BALANCED_SOFT_AGGRESSIVE_SCORING_WEIGHTS = defineScoringProfile({
  id: "finisher-aggressive",
  base: BALANCED_DISTANCE_SCORING_WEIGHTS,
  mana: {
    manaCarrierOneStepFromPool: 250,
    supermanaCarrierOneStepFromPoolExtra: 130,
    immediateWinningCarrier: 540,
  },
});

export const MANA_RACE_LITE_D2_TUNED_SCORING_WEIGHTS = defineScoringProfile({
  id: "mana-race-lite-d2",
  base: MANA_RACE_LITE_SCORING_WEIGHTS,
  position: {
    manaCloseToSamePool: 380,
    drainerCloseToOwnPool: 320,
  },
  mana: {
    regularManaToOwnerPool: 170,
    regularManaDrainerControl: 18,
    manaCarrierAtRisk: -210,
    manaCarrierGuarded: 95,
    manaCarrierOneStepFromPool: 260,
    supermanaCarrierOneStepFromPoolExtra: 150,
    immediateWinningCarrier: 300,
  },
});

export const TACTICAL_BALANCED_SCORING_WEIGHTS = defineScoringProfile({
  id: "tactical-balanced",
  base: BALANCED_DISTANCE_SCORING_WEIGHTS,
  material: { faintedCooldownStep: -120 },
  position: {
    spiritCloseToEnemy: 230,
    angelGuardingDrainer: 300,
  },
  mana: {
    manaCarrierAtRisk: -200,
    manaCarrierGuarded: 110,
    manaCarrierOneStepFromPool: 240,
    supermanaCarrierOneStepFromPoolExtra: 150,
  },
});

export const TACTICAL_BALANCED_AGGRESSIVE_SCORING_WEIGHTS = defineScoringProfile({
  id: "tactical-aggressive",
  base: TACTICAL_BALANCED_SCORING_WEIGHTS,
  material: { faintedCooldownStep: -160 },
  position: {
    spiritCloseToEnemy: 250,
    angelGuardingDrainer: 320,
  },
  mana: {
    manaCarrierAtRisk: -260,
    manaCarrierGuarded: 140,
    manaCarrierOneStepFromPool: 320,
    supermanaCarrierOneStepFromPoolExtra: 220,
  },
});

const RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS = defineScoringProfile({
  id: "runtime-fast-context",
  base: MANA_RACE_LITE_D2_TUNED_SCORING_WEIGHTS,
  formula: { useHeuristicFormula: false },
  material: { confirmedScore: 920 },
  position: {
    drainerCloseToMana: 360,
    drainerHoldingMana: 430,
  },
  mana: {
    manaCarrierAtRisk: -285,
    manaCarrierGuarded: 145,
    manaCarrierOneStepFromPool: 320,
    supermanaCarrierOneStepFromPoolExtra: 210,
    immediateWinningCarrier: 520,
  },
  race: {
    scoreRacePathProgress: 165,
    opponentScoreRacePathProgress: 150,
    scoreRaceMultiPath: 60,
    opponentScoreRaceMultiPath: 90,
    immediateScoreWindow: 240,
    opponentImmediateScoreWindow: 220,
    immediateScoreMultiWindow: 80,
    opponentImmediateScoreMultiWindow: 120,
  },
  threat: {
    drainerBestManaPath: 250,
    drainerPickupScoreThisTurn: 210,
    manaCarrierScoreThisTurn: 290,
    drainerImmediateThreat: -220,
    spiritActionUtility: 56,
  },
});

export const RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS_POTION_PREF =
  defineScoringProfile({
    id: "runtime-fast-context-potion",
    base: RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS,
    material: { hasConsumable: 320 },
    threat: { spiritActionUtility: 72 },
  });

export const RUNTIME_FAST_BOOLEAN_DRAINER_SCORING_WEIGHTS = defineScoringProfile({
  id: "runtime-fast-boolean",
  base: RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS,
  mana: { supermanaRaceControl: 30 },
  threat: {
    drainerDangerBoolean: -400,
    manaCarrierDangerBoolean: -300,
  },
});

export const RUNTIME_FAST_BOOLEAN_DRAINER_SCORING_WEIGHTS_POTION_PREF =
  defineScoringProfile({
    id: "runtime-fast-boolean-potion",
    base: RUNTIME_FAST_BOOLEAN_DRAINER_SCORING_WEIGHTS,
    material: { hasConsumable: 320 },
    threat: { spiritActionUtility: 72 },
  });
