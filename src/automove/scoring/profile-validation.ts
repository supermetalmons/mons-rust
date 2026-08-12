import {
  FORMULA_KEYS,
  MANA_KEYS,
  MATERIAL_KEYS,
  POSITION_KEYS,
  PROFILE_KEYS,
  RACE_KEYS,
  THREAT_KEYS,
  type EvaluationFormulaWeights,
  type NumericSection,
  type ScoringProfileDefinition,
  type ScoringWeights,
} from "./profile-schema.js";
import { BUILT_IN_PROFILE_SIGNATURES } from "./built-in-profile-signatures.js";

const MAX_ABSOLUTE_WEIGHT = 10_000;
const MAX_NEXT_TURN_WINDOW_SCALE_BP = 20_000;
const PROFILE_ID_PATTERN = /^[a-z0-9]+(?:[.-][a-z0-9]+)*$/;
const PROFILE_SIGNATURES = new Map<string, string>(
  Object.entries(BUILT_IN_PROFILE_SIGNATURES),
);
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
  if (typeof id !== "string" || id.length > 96 || !PROFILE_ID_PATTERN.test(id)) {
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

export function freezeScoringSection<T extends object>(section: T): Readonly<T> {
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
    formula: freezeScoringSection({ ...base.formula, ...definition.formula }),
    material: freezeScoringSection({
      ...base.material,
      ...definition.material,
    }),
    position: freezeScoringSection({
      ...base.position,
      ...definition.position,
    }),
    mana: freezeScoringSection({ ...base.mana, ...definition.mana }),
    race: freezeScoringSection({ ...base.race, ...definition.race }),
    threat: freezeScoringSection({ ...base.threat, ...definition.threat }),
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

export function normalizeScoringProfile(profile: ScoringWeights): ScoringWeights {
  validateScoringProfile(profile);
  if (IMMUTABLE_PROFILES.has(profile)) return profile;

  const normalized: ScoringWeights = Object.freeze({
    id: profile.id,
    formula: freezeScoringSection({ ...profile.formula }),
    material: freezeScoringSection({ ...profile.material }),
    position: freezeScoringSection({ ...profile.position }),
    mana: freezeScoringSection({ ...profile.mana }),
    race: freezeScoringSection({ ...profile.race }),
    threat: freezeScoringSection({ ...profile.threat }),
  });
  validateScoringProfile(normalized);
  IMMUTABLE_PROFILES.add(normalized);
  return normalized;
}
