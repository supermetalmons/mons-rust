// Keeps every valid-game evaluation comfortably inside the search score
// sentinels, including the historical double-confirmed-score formula.
const MAX_ABSOLUTE_WEIGHT = 10_000;
const MAX_NEXT_TURN_WINDOW_SCALE_BP = 20_000;
const PROFILE_ID_PATTERN = /^[a-z0-9]+(?:[.-][a-z0-9]+)*$/;

type EvaluationFormulaWeights = {
  readonly useHeuristicFormula: boolean;
  readonly includeRegularManaMoveWindows: boolean;
  readonly includeMatchPointWindow: boolean;
  readonly nextTurnWindowScaleBp: number;
  readonly doubleConfirmedScore: boolean;
};

type MaterialWeights = {
  readonly confirmedScore: number;
  readonly faintedMon: number;
  readonly faintedDrainer: number;
  readonly faintedCooldownStep: number;
  readonly hasConsumable: number;
  readonly activeMon: number;
};

type PositionWeights = {
  readonly drainerAtRisk: number;
  readonly manaCloseToSamePool: number;
  readonly monWithManaCloseToAnyPool: number;
  readonly extraForSupermana: number;
  readonly extraForOpponentsMana: number;
  readonly drainerCloseToMana: number;
  readonly drainerHoldingMana: number;
  readonly drainerCloseToOwnPool: number;
  readonly drainerCloseToSupermana: number;
  readonly monCloseToCenter: number;
  readonly spiritCloseToEnemy: number;
  readonly spiritOnOwnBasePenalty: number;
  readonly angelGuardingDrainer: number;
  readonly angelCloseToFriendlyDrainer: number;
};

type ManaWeights = {
  readonly regularManaToOwnerPool: number;
  readonly regularManaDrainerControl: number;
  readonly supermanaDrainerControl: number;
  readonly supermanaRaceControl: number;
  readonly opponentManaDenial: number;
  readonly manaCarrierAtRisk: number;
  readonly manaCarrierGuarded: number;
  readonly manaCarrierOneStepFromPool: number;
  readonly supermanaCarrierOneStepFromPoolExtra: number;
  readonly immediateWinningCarrier: number;
};

type RaceWeights = {
  readonly scoreRacePathProgress: number;
  readonly opponentScoreRacePathProgress: number;
  readonly scoreRaceMultiPath: number;
  readonly opponentScoreRaceMultiPath: number;
  readonly immediateScoreWindow: number;
  readonly opponentImmediateScoreWindow: number;
  readonly immediateScoreMultiWindow: number;
  readonly opponentImmediateScoreMultiWindow: number;
};

type ThreatWeights = {
  readonly drainerBestManaPath: number;
  readonly drainerPickupScoreThisTurn: number;
  readonly manaCarrierScoreThisTurn: number;
  readonly drainerImmediateThreat: number;
  readonly spiritActionUtility: number;
  readonly drainerDangerBoolean: number;
  readonly manaCarrierDangerBoolean: number;
  readonly drainerWalkThreatBoolean: number;
  readonly manaCarrierWalkThreatBoolean: number;
  readonly opponentDrainerAttackBonus: number;
  readonly attackerCloseToOpponentDrainer: number;
};

export type ScoringWeights = {
  /** Stable cache and diagnostics identity for this immutable profile. */
  readonly id: string;
  readonly formula: EvaluationFormulaWeights;
  readonly material: MaterialWeights;
  readonly position: PositionWeights;
  readonly mana: ManaWeights;
  readonly race: RaceWeights;
  readonly threat: ThreatWeights;
};

type ScoringProfileDefinition = {
  readonly id: string;
  readonly base?: ScoringWeights;
  readonly formula?: Partial<EvaluationFormulaWeights>;
  readonly material?: Partial<MaterialWeights>;
  readonly position?: Partial<PositionWeights>;
  readonly mana?: Partial<ManaWeights>;
  readonly race?: Partial<RaceWeights>;
  readonly threat?: Partial<ThreatWeights>;
};

const PROFILE_KEYS = [
  "id",
  "formula",
  "material",
  "position",
  "mana",
  "race",
  "threat",
] as const satisfies readonly (keyof ScoringWeights)[];
const FORMULA_KEYS = [
  "useHeuristicFormula",
  "includeRegularManaMoveWindows",
  "includeMatchPointWindow",
  "nextTurnWindowScaleBp",
  "doubleConfirmedScore",
] as const satisfies readonly (keyof EvaluationFormulaWeights)[];
const MATERIAL_KEYS = [
  "confirmedScore",
  "faintedMon",
  "faintedDrainer",
  "faintedCooldownStep",
  "hasConsumable",
  "activeMon",
] as const satisfies readonly (keyof MaterialWeights)[];
const POSITION_KEYS = [
  "drainerAtRisk",
  "manaCloseToSamePool",
  "monWithManaCloseToAnyPool",
  "extraForSupermana",
  "extraForOpponentsMana",
  "drainerCloseToMana",
  "drainerHoldingMana",
  "drainerCloseToOwnPool",
  "drainerCloseToSupermana",
  "monCloseToCenter",
  "spiritCloseToEnemy",
  "spiritOnOwnBasePenalty",
  "angelGuardingDrainer",
  "angelCloseToFriendlyDrainer",
] as const satisfies readonly (keyof PositionWeights)[];
const MANA_KEYS = [
  "regularManaToOwnerPool",
  "regularManaDrainerControl",
  "supermanaDrainerControl",
  "supermanaRaceControl",
  "opponentManaDenial",
  "manaCarrierAtRisk",
  "manaCarrierGuarded",
  "manaCarrierOneStepFromPool",
  "supermanaCarrierOneStepFromPoolExtra",
  "immediateWinningCarrier",
] as const satisfies readonly (keyof ManaWeights)[];
const RACE_KEYS = [
  "scoreRacePathProgress",
  "opponentScoreRacePathProgress",
  "scoreRaceMultiPath",
  "opponentScoreRaceMultiPath",
  "immediateScoreWindow",
  "opponentImmediateScoreWindow",
  "immediateScoreMultiWindow",
  "opponentImmediateScoreMultiWindow",
] as const satisfies readonly (keyof RaceWeights)[];
const THREAT_KEYS = [
  "drainerBestManaPath",
  "drainerPickupScoreThisTurn",
  "manaCarrierScoreThisTurn",
  "drainerImmediateThreat",
  "spiritActionUtility",
  "drainerDangerBoolean",
  "manaCarrierDangerBoolean",
  "drainerWalkThreatBoolean",
  "manaCarrierWalkThreatBoolean",
  "opponentDrainerAttackBonus",
  "attackerCloseToOpponentDrainer",
] as const satisfies readonly (keyof ThreatWeights)[];

type NumericSection =
  MaterialWeights | PositionWeights | ManaWeights | RaceWeights | ThreatWeights;

const PROFILE_SIGNATURES = new Map<string, string>();
const IMMUTABLE_PROFILES = new WeakSet<object>();

function validateExactKeys(
  sectionName: string,
  section: object,
  expectedKeys: readonly string[],
): void {
  const actualKeys = Object.keys(section).sort();
  const sortedExpectedKeys = [...expectedKeys].sort();
  if (
    actualKeys.length !== sortedExpectedKeys.length ||
    actualKeys.some((key, index) => key !== sortedExpectedKeys[index])
  ) {
    throw new TypeError(
      `${sectionName} must contain exactly: ${sortedExpectedKeys.join(", ")}`,
    );
  }
}

function validateProfileId(id: string): void {
  if (
    typeof id !== "string" ||
    id.length > 96 ||
    !PROFILE_ID_PATTERN.test(id)
  ) {
    throw new RangeError(
      `scoring profile id must be a lower-case dotted or dashed identifier: ${id}`,
    );
  }
}

function validateNumericSection(
  sectionName: string,
  section: NumericSection,
  expectedKeys: readonly string[],
): void {
  validateExactKeys(sectionName, section, expectedKeys);
  for (const [name, value] of Object.entries(section)) {
    if (!Number.isSafeInteger(value) || Math.abs(value) > MAX_ABSOLUTE_WEIGHT) {
      throw new RangeError(
        `${sectionName}.${name} must be a safe integer between -${MAX_ABSOLUTE_WEIGHT} and ${MAX_ABSOLUTE_WEIGHT}`,
      );
    }
  }
}

function validateFormula(formula: EvaluationFormulaWeights): void {
  validateExactKeys("formula", formula, FORMULA_KEYS);
  for (const [name, value] of Object.entries(formula)) {
    if (name === "nextTurnWindowScaleBp") {
      if (
        typeof value !== "number" ||
        !Number.isSafeInteger(value) ||
        value < 0 ||
        value > MAX_NEXT_TURN_WINDOW_SCALE_BP
      ) {
        throw new RangeError(
          `formula.nextTurnWindowScaleBp must be an integer between 0 and ${MAX_NEXT_TURN_WINDOW_SCALE_BP}`,
        );
      }
    } else if (typeof value !== "boolean") {
      throw new TypeError(`formula.${name} must be a boolean`);
    }
  }
}

function profileSignature(profile: ScoringWeights): string {
  return JSON.stringify([
    FORMULA_KEYS.map((key) => profile.formula[key]),
    MATERIAL_KEYS.map((key) => profile.material[key]),
    POSITION_KEYS.map((key) => profile.position[key]),
    MANA_KEYS.map((key) => profile.mana[key]),
    RACE_KEYS.map((key) => profile.race[key]),
    THREAT_KEYS.map((key) => profile.threat[key]),
  ]);
}

function registerProfileIdentity(profile: ScoringWeights): void {
  const signature = profileSignature(profile);
  const registered = PROFILE_SIGNATURES.get(profile.id);
  if (registered !== undefined && registered !== signature) {
    throw new RangeError(
      `scoring profile id ${profile.id} is already registered with different weights`,
    );
  }
  PROFILE_SIGNATURES.set(profile.id, signature);
}

function freezeSection<T extends object>(section: T): Readonly<T> {
  return Object.freeze(section);
}

export function defineScoringProfile(
  definition: ScoringProfileDefinition,
): ScoringWeights {
  validateProfileId(definition.id);
  const base = definition.base;
  if (base === undefined) {
    throw new TypeError(
      "a scoring profile must extend a complete base profile; use the built-in default profile as the root",
    );
  }

  const profile: ScoringWeights = {
    id: definition.id,
    formula: freezeSection({ ...base.formula, ...definition.formula }),
    material: freezeSection({ ...base.material, ...definition.material }),
    position: freezeSection({ ...base.position, ...definition.position }),
    mana: freezeSection({ ...base.mana, ...definition.mana }),
    race: freezeSection({ ...base.race, ...definition.race }),
    threat: freezeSection({ ...base.threat, ...definition.threat }),
  };
  validateScoringProfile(profile);
  const immutableProfile = Object.freeze(profile);
  IMMUTABLE_PROFILES.add(immutableProfile);
  return immutableProfile;
}

export function validateScoringProfile(profile: unknown): void {
  if (typeof profile !== "object" || profile === null) {
    throw new TypeError("scoring profile must be an object");
  }
  const candidate = profile as ScoringWeights;
  validateExactKeys("scoring profile", candidate, PROFILE_KEYS);
  validateProfileId(candidate.id);
  validateFormula(candidate.formula);
  validateNumericSection("material", candidate.material, MATERIAL_KEYS);
  validateNumericSection("position", candidate.position, POSITION_KEYS);
  validateNumericSection("mana", candidate.mana, MANA_KEYS);
  validateNumericSection("race", candidate.race, RACE_KEYS);
  validateNumericSection("threat", candidate.threat, THREAT_KEYS);
  registerProfileIdentity(candidate);
}

export function scoringProfileId(profile: ScoringWeights): string {
  return profile.id;
}

/**
 * Snapshot caller-provided weights before a config or long-lived cache keeps
 * them. Profiles created by this module are already immutable and are reused.
 */
export function normalizeScoringProfile(
  profile: ScoringWeights,
): ScoringWeights {
  validateScoringProfile(profile);
  if (IMMUTABLE_PROFILES.has(profile)) return profile;

  const normalized: ScoringWeights = Object.freeze({
    id: profile.id,
    formula: freezeSection({ ...profile.formula }),
    material: freezeSection({ ...profile.material }),
    position: freezeSection({ ...profile.position }),
    mana: freezeSection({ ...profile.mana }),
    race: freezeSection({ ...profile.race }),
    threat: freezeSection({ ...profile.threat }),
  });
  validateScoringProfile(normalized);
  IMMUTABLE_PROFILES.add(normalized);
  return normalized;
}

const DEFAULT_FORMULA = freezeSection<EvaluationFormulaWeights>({
  useHeuristicFormula: true,
  includeRegularManaMoveWindows: false,
  includeMatchPointWindow: false,
  nextTurnWindowScaleBp: 5_000,
  doubleConfirmedScore: true,
});

const DEFAULT_MATERIAL = freezeSection<MaterialWeights>({
  confirmedScore: 1_000,
  faintedMon: -500,
  faintedDrainer: -800,
  faintedCooldownStep: 0,
  hasConsumable: 110,
  activeMon: 50,
});

const DEFAULT_POSITION = freezeSection<PositionWeights>({
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

const DEFAULT_MANA = freezeSection<ManaWeights>({
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

const DEFAULT_RACE = freezeSection<RaceWeights>({
  scoreRacePathProgress: 0,
  opponentScoreRacePathProgress: 0,
  scoreRaceMultiPath: 0,
  opponentScoreRaceMultiPath: 0,
  immediateScoreWindow: 0,
  opponentImmediateScoreWindow: 0,
  immediateScoreMultiWindow: 0,
  opponentImmediateScoreMultiWindow: 0,
});

const DEFAULT_THREAT = freezeSection<ThreatWeights>({
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

export const DEFAULT_SCORING_WEIGHTS: ScoringWeights = Object.freeze({
  id: "default",
  formula: DEFAULT_FORMULA,
  material: DEFAULT_MATERIAL,
  position: DEFAULT_POSITION,
  mana: DEFAULT_MANA,
  race: DEFAULT_RACE,
  threat: DEFAULT_THREAT,
});
IMMUTABLE_PROFILES.add(DEFAULT_SCORING_WEIGHTS);
validateScoringProfile(DEFAULT_SCORING_WEIGHTS);

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

export const FINISHER_BALANCED_SOFT_AGGRESSIVE_SCORING_WEIGHTS =
  defineScoringProfile({
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

export const TACTICAL_BALANCED_AGGRESSIVE_SCORING_WEIGHTS =
  defineScoringProfile({
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

export const RUNTIME_FAST_BOOLEAN_DRAINER_SCORING_WEIGHTS =
  defineScoringProfile({
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
