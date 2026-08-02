import { TARGET_SCORE } from "../engine/config.js";
import { Color } from "../engine/domain.js";
import type { MonsGame } from "../engine/game.js";
import {
  BALANCED_DISTANCE_SCORING_WEIGHTS,
  DEFAULT_SCORING_WEIGHTS,
  FINISHER_BALANCED_SOFT_AGGRESSIVE_SCORING_WEIGHTS,
  FINISHER_BALANCED_SOFT_SCORING_WEIGHTS,
  MANA_RACE_LITE_D2_TUNED_SCORING_WEIGHTS,
  RUNTIME_FAST_BOOLEAN_DRAINER_SCORING_WEIGHTS,
  RUNTIME_FAST_BOOLEAN_DRAINER_SCORING_WEIGHTS_POTION_PREF,
  RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS_POTION_PREF,
  TACTICAL_BALANCED_AGGRESSIVE_SCORING_WEIGHTS,
  TACTICAL_BALANCED_SCORING_WEIGHTS,
  defineScoringProfile,
  normalizeScoringProfile,
  validateScoringProfile,
  type ScoringWeights,
} from "./scoring.js";
import {
  AUTOMOVE_TURN_ENGINE_MODE,
  type AutomoveBudgetConfig,
  type AutomoveConfig,
  type AutomoveEvaluationConfig,
  type AutomovePlannerConfig,
  type AutomovePolicyConfig,
  type AutomoveReplyRiskConfig,
  type AutomoveTreeSearchConfig,
  type SmartAutomovePreference,
} from "./selector-types.js";

const MIN_SEARCH_DEPTH = 1;
const MAX_SEARCH_DEPTH = 5;
const MIN_MAX_VISITED_NODES = 32;
const MAX_MAX_VISITED_NODES = 180_000;
const FAST_DEPTH = 2;
const FAST_MAX_VISITED_NODES = 480;
const NORMAL_DEPTH = 3;
const NORMAL_MAX_VISITED_NODES = 3_800;
const PRO_DEPTH = 4;
const PRO_MAX_VISITED_NODES = Math.trunc(
  (NORMAL_MAX_VISITED_NODES * 369) / 100,
);

const ROOT_REPLY_RISK_SCORE_MARGIN = 140;
const ROOT_REPLY_RISK_SHORTLIST_FAST = 3;
const ROOT_REPLY_RISK_REPLY_LIMIT_FAST = 8;
const ROOT_REPLY_RISK_NODE_SHARE_BP_FAST = 600;
const ROOT_ANTI_HELP_SCORE_MARGIN = 180;
const ROOT_ANTI_HELP_REPLY_LIMIT_FAST = 6;
const ROOT_DRAINER_SAFETY_SCORE_MARGIN = 2_200;
const ROOT_MANA_HANDOFF_PENALTY = 220;
const ROOT_BACKTRACK_PENALTY = 140;
const ROOT_EFFICIENCY_SCORE_MARGIN = 2_500;
const POTION_SPEND_PENALTY_FAST = 340;
const POTION_SPEND_PENALTY_NORMAL = 260;
const SOFT_SUPERMANA_PROGRESS_BONUS = 240;
const SOFT_SUPERMANA_SCORE_BONUS = 420;
const SOFT_OPPONENT_MANA_PROGRESS_BONUS = 210;
const SOFT_OPPONENT_MANA_SCORE_BONUS = 360;
const SOFT_MANA_HANDOFF_PENALTY = 220;
const SOFT_ROUNDTRIP_PENALTY = 140;

const AUTOMOVE_SEARCH_CONSTANTS = Object.freeze({
  maxExtensionsPerPath: 1,
  extensionNodeShareBp: 1_500,
  quiescenceNodes: 120,
  quiescenceEnumerationLimit: 12,
  futilityMargin: 2_300,
  transpositionCapacity: 12_000,
  evaluationCacheCapacity: 32_768,
});

const AUTOMOVE_SEARCH_BUDGETS = Object.freeze({
  fast: Object.freeze({
    depth: FAST_DEPTH,
    maxVisitedNodes: FAST_MAX_VISITED_NODES,
  }),
  normal: Object.freeze({
    depth: NORMAL_DEPTH,
    maxVisitedNodes: NORMAL_MAX_VISITED_NODES,
  }),
  pro: Object.freeze({
    depth: PRO_DEPTH,
    maxVisitedNodes: PRO_MAX_VISITED_NODES,
  }),
});

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

function withMediumWalkThreat(
  id: string,
  base: ScoringWeights,
): ScoringWeights {
  return defineScoringProfile({
    id,
    base,
    threat: {
      drainerWalkThreatBoolean: -300,
      manaCarrierWalkThreatBoolean: -150,
    },
  });
}

function withAttackerProximity(
  id: string,
  base: ScoringWeights,
): ScoringWeights {
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

function runtimePhaseAdaptiveWalkThreatMediumScoringProfile(
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

function runtimePhaseAdaptiveAttackerProximityScoringProfile(
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

function clamp(value: number, minimum: number, maximum: number): number {
  return Math.min(maximum, Math.max(minimum, Math.trunc(value)));
}

function subtractSaturating(value: number, amount: number): number {
  return Math.max(0, value - amount);
}

function scaleFloor(
  value: number,
  numerator: number,
  denominator: number,
): number {
  return Math.trunc((value * numerator) / denominator);
}

type AutomoveConfigDefinition = AutomoveConfig;

type SectionPatch<Section> = {
  readonly [Key in keyof Section]?: Section[Key];
};

export type AutomoveConfigPatch = {
  readonly budget?: SectionPatch<AutomoveBudgetConfig>;
  readonly search?: SectionPatch<AutomoveTreeSearchConfig>;
  readonly planner?: SectionPatch<AutomovePlannerConfig>;
  readonly evaluation?: SectionPatch<AutomoveEvaluationConfig>;
  readonly replyRisk?: SectionPatch<AutomoveReplyRiskConfig>;
  readonly policy?: SectionPatch<AutomovePolicyConfig>;
};

const MAX_CONFIG_INTEGER = 1_000_000;
const MAX_BASIS_POINTS = 10_000;
const CONFIG_KEYS = [
  "budget",
  "search",
  "planner",
  "evaluation",
  "replyRisk",
  "policy",
] as const satisfies readonly (keyof AutomoveConfig)[];
const BUDGET_KEYS = [
  "preference",
  "depth",
  "maxVisitedNodes",
  "exactLiteRootCalls",
  "exactLiteStaticCalls",
  "extensionNodeShareBp",
  "quiescenceNodes",
] as const satisfies readonly (keyof AutomoveBudgetConfig)[];
const BUDGET_NUMBER_KEYS = [
  "depth",
  "maxVisitedNodes",
  "exactLiteRootCalls",
  "exactLiteStaticCalls",
  "extensionNodeShareBp",
  "quiescenceNodes",
] as const satisfies readonly (keyof AutomoveBudgetConfig)[];
const SEARCH_KEYS = [
  "rootEnumerationLimit",
  "rootBranchLimit",
  "nodeEnumerationLimit",
  "nodeBranchLimit",
  "twoPassRootAllocation",
  "volatilityFocus",
  "selectiveExtensions",
  "maxExtensionsPerPath",
  "quietReductions",
  "quietReductionDepthThreshold",
  "quiescence",
  "quiescenceEnumerationLimit",
  "futilityPruning",
  "futilityMargin",
  "transpositionTable",
  "transpositionCapacity",
  "exactLiteChecks",
  "exactRootAnalysis",
] as const satisfies readonly (keyof AutomoveTreeSearchConfig)[];
const SEARCH_BOOLEAN_KEYS = [
  "twoPassRootAllocation",
  "volatilityFocus",
  "selectiveExtensions",
  "quietReductions",
  "quiescence",
  "futilityPruning",
  "transpositionTable",
  "exactLiteChecks",
  "exactRootAnalysis",
] as const satisfies readonly (keyof AutomoveTreeSearchConfig)[];
const SEARCH_NUMBER_KEYS = [
  "rootEnumerationLimit",
  "rootBranchLimit",
  "nodeEnumerationLimit",
  "nodeBranchLimit",
  "maxExtensionsPerPath",
  "quietReductionDepthThreshold",
  "quiescenceEnumerationLimit",
  "futilityMargin",
  "transpositionCapacity",
] as const satisfies readonly (keyof AutomoveTreeSearchConfig)[];
const PLANNER_KEYS = [
  "enabled",
  "rerankHeads",
  "lowBudgetGuard",
  "midTurnTacticalGuard",
  "secondaryAnalysis",
  "selectedFollowupProjection",
  "mode",
  "ownSeedCap",
  "ownBeam",
  "perNodeFamilyCap",
  "stepCap",
  "opponentSeedCap",
  "opponentBeam",
  "replySeedCap",
  "replyBeam",
  "expansionCap",
  "spiritFamily",
] as const satisfies readonly (keyof AutomovePlannerConfig)[];
const PLANNER_BOOLEAN_KEYS = [
  "enabled",
  "rerankHeads",
  "lowBudgetGuard",
  "midTurnTacticalGuard",
  "secondaryAnalysis",
  "selectedFollowupProjection",
  "spiritFamily",
] as const satisfies readonly (keyof AutomovePlannerConfig)[];
const PLANNER_NUMBER_KEYS = [
  "ownSeedCap",
  "ownBeam",
  "perNodeFamilyCap",
  "stepCap",
  "opponentSeedCap",
  "opponentBeam",
  "replySeedCap",
  "replyBeam",
  "expansionCap",
] as const satisfies readonly (keyof AutomovePlannerConfig)[];
const EVALUATION_KEYS = [
  "weights",
  "cacheCapacity",
  "rootManaHandoffPenalty",
  "rootBacktrackPenalty",
  "potionSpendPenalty",
  "supermanaProgressBonus",
  "supermanaScoreBonus",
  "opponentManaProgressBonus",
  "opponentManaScoreBonus",
  "softManaHandoffPenalty",
  "softRoundtripPenalty",
] as const satisfies readonly (keyof AutomoveEvaluationConfig)[];
const EVALUATION_NUMBER_KEYS = [
  "cacheCapacity",
  "rootManaHandoffPenalty",
  "rootBacktrackPenalty",
  "potionSpendPenalty",
  "supermanaProgressBonus",
  "supermanaScoreBonus",
  "opponentManaProgressBonus",
  "opponentManaScoreBonus",
  "softManaHandoffPenalty",
  "softRoundtripPenalty",
] as const satisfies readonly (keyof AutomoveEvaluationConfig)[];
const REPLY_RISK_KEYS = [
  "enabled",
  "scoreMargin",
  "shortlistLimit",
  "replyLimit",
  "nodeShareBp",
  "antiHelpScoreMargin",
  "antiHelpReplyLimit",
  "safetyRerank",
  "deepSafetyFloor",
  "deterministicTiebreak",
  "preferCleanRoots",
  "lateSafeManaRootPreference",
] as const satisfies readonly (keyof AutomoveReplyRiskConfig)[];
const REPLY_RISK_BOOLEAN_KEYS = [
  "enabled",
  "safetyRerank",
  "deepSafetyFloor",
  "deterministicTiebreak",
  "preferCleanRoots",
  "lateSafeManaRootPreference",
] as const satisfies readonly (keyof AutomoveReplyRiskConfig)[];
const REPLY_RISK_NUMBER_KEYS = [
  "scoreMargin",
  "shortlistLimit",
  "replyLimit",
  "nodeShareBp",
  "antiHelpScoreMargin",
  "antiHelpReplyLimit",
] as const satisfies readonly (keyof AutomoveReplyRiskConfig)[];
const POLICY_KEYS = [
  "targetedDrainerAttackFallback",
  "forcedTacticalPrepass",
  "preferSpiritDevelopment",
  "hardSpiritDeployment",
  "supermanaPrepassException",
  "drainerSafetyScoreMargin",
  "efficiencyScoreMargin",
] as const satisfies readonly (keyof AutomovePolicyConfig)[];
const POLICY_BOOLEAN_KEYS = [
  "targetedDrainerAttackFallback",
  "forcedTacticalPrepass",
  "preferSpiritDevelopment",
  "hardSpiritDeployment",
  "supermanaPrepassException",
] as const satisfies readonly (keyof AutomovePolicyConfig)[];
const POLICY_NUMBER_KEYS = [
  "drainerSafetyScoreMargin",
  "efficiencyScoreMargin",
] as const satisfies readonly (keyof AutomovePolicyConfig)[];

function freezeSection<Section extends object>(
  section: Section,
): Readonly<Section> {
  return Object.freeze(section);
}

function requireRecord(
  label: string,
  value: unknown,
): asserts value is Readonly<Record<string, unknown>> {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new TypeError(`${label} must be a plain object`);
  }
}

function validateExactKeys(
  label: string,
  value: Readonly<Record<string, unknown>>,
  expectedKeys: readonly string[],
): void {
  const actual = Object.keys(value).sort();
  const expected = [...expectedKeys].sort();
  if (
    actual.length !== expected.length ||
    actual.some((key, index) => key !== expected[index])
  ) {
    throw new TypeError(
      `${label} must contain exactly: ${expected.join(", ")}`,
    );
  }
}

function validateConfigNumbers(
  sectionName: string,
  section: Readonly<Record<string, unknown>>,
  keys: readonly string[],
): void {
  for (const name of keys) {
    const value = section[name];
    if (
      typeof value !== "number" ||
      !Number.isSafeInteger(value) ||
      value < 0 ||
      value > MAX_CONFIG_INTEGER
    ) {
      throw new RangeError(
        `${sectionName}.${name} must be an integer from 0 through ${MAX_CONFIG_INTEGER}`,
      );
    }
  }
}

function validateConfigBooleans(
  sectionName: string,
  section: Readonly<Record<string, unknown>>,
  keys: readonly string[],
): void {
  for (const name of keys) {
    if (typeof section[name] !== "boolean") {
      throw new TypeError(`${sectionName}.${name} must be a boolean`);
    }
  }
}

export function validateAutomoveConfig(
  value: unknown,
): asserts value is AutomoveConfig {
  requireRecord("automove config", value);
  validateExactKeys("automove config", value, CONFIG_KEYS);
  for (const [name, keys] of [
    ["budget", BUDGET_KEYS],
    ["search", SEARCH_KEYS],
    ["planner", PLANNER_KEYS],
    ["evaluation", EVALUATION_KEYS],
    ["replyRisk", REPLY_RISK_KEYS],
    ["policy", POLICY_KEYS],
  ] as const) {
    const section = value[name];
    requireRecord(`${name} config`, section);
    validateExactKeys(`${name} config`, section, keys);
  }

  const config = value as AutomoveConfig;
  validateScoringProfile(config.evaluation.weights);
  if (
    !Object.prototype.hasOwnProperty.call(
      AUTOMOVE_SEARCH_BUDGETS,
      config.budget.preference,
    )
  ) {
    throw new RangeError(
      `unknown automove preference: ${config.budget.preference}`,
    );
  }
  const plannerMode = (value["planner"] as Readonly<Record<string, unknown>>)[
    "mode"
  ];
  if (
    plannerMode !== AUTOMOVE_TURN_ENGINE_MODE.Baseline &&
    plannerMode !== AUTOMOVE_TURN_ENGINE_MODE.Production
  ) {
    throw new RangeError(`unknown planner mode: ${String(plannerMode)}`);
  }
  validateConfigNumbers("budget", config.budget, BUDGET_NUMBER_KEYS);
  validateConfigNumbers("search", config.search, SEARCH_NUMBER_KEYS);
  validateConfigNumbers("planner", config.planner, PLANNER_NUMBER_KEYS);
  validateConfigNumbers(
    "evaluation",
    config.evaluation,
    EVALUATION_NUMBER_KEYS,
  );
  validateConfigNumbers("replyRisk", config.replyRisk, REPLY_RISK_NUMBER_KEYS);
  validateConfigNumbers("policy", config.policy, POLICY_NUMBER_KEYS);
  validateConfigBooleans("search", config.search, SEARCH_BOOLEAN_KEYS);
  validateConfigBooleans("planner", config.planner, PLANNER_BOOLEAN_KEYS);
  validateConfigBooleans(
    "replyRisk",
    config.replyRisk,
    REPLY_RISK_BOOLEAN_KEYS,
  );
  validateConfigBooleans("policy", config.policy, POLICY_BOOLEAN_KEYS);
  if (
    config.budget.depth < MIN_SEARCH_DEPTH ||
    config.budget.depth > MAX_SEARCH_DEPTH
  ) {
    throw new RangeError(
      `budget.depth must be from ${MIN_SEARCH_DEPTH} through ${MAX_SEARCH_DEPTH}`,
    );
  }
  if (
    config.budget.maxVisitedNodes < 1 ||
    config.budget.maxVisitedNodes > MAX_MAX_VISITED_NODES
  ) {
    throw new RangeError(
      `budget.maxVisitedNodes must be from 1 through ${MAX_MAX_VISITED_NODES}`,
    );
  }
  if (config.search.transpositionCapacity < 1) {
    throw new RangeError("search.transpositionCapacity must be positive");
  }
  if (
    config.budget.extensionNodeShareBp > MAX_BASIS_POINTS ||
    config.replyRisk.nodeShareBp > MAX_BASIS_POINTS
  ) {
    throw new RangeError(
      `basis-point shares must not exceed ${MAX_BASIS_POINTS}`,
    );
  }
}

function defineAutomoveConfig(
  definition: AutomoveConfigDefinition,
): AutomoveConfig {
  const weights = normalizeScoringProfile(definition.evaluation.weights);
  const config: AutomoveConfig = Object.freeze({
    budget: freezeSection({ ...definition.budget }),
    search: freezeSection({ ...definition.search }),
    planner: freezeSection({ ...definition.planner }),
    evaluation: freezeSection({
      ...definition.evaluation,
      weights,
    }),
    replyRisk: freezeSection({ ...definition.replyRisk }),
    policy: freezeSection({ ...definition.policy }),
  });
  validateAutomoveConfig(config);
  return config;
}

export function patchAutomoveConfig(
  config: AutomoveConfig,
  patch: AutomoveConfigPatch,
): AutomoveConfig {
  const evaluation = (() => {
    if (patch.evaluation === undefined) return config.evaluation;
    const weights = normalizeScoringProfile(
      patch.evaluation.weights ?? config.evaluation.weights,
    );
    return freezeSection({
      ...config.evaluation,
      ...patch.evaluation,
      weights,
    });
  })();
  const next: AutomoveConfig = Object.freeze({
    budget:
      patch.budget === undefined
        ? config.budget
        : freezeSection({ ...config.budget, ...patch.budget }),
    search:
      patch.search === undefined
        ? config.search
        : freezeSection({ ...config.search, ...patch.search }),
    planner:
      patch.planner === undefined
        ? config.planner
        : freezeSection({ ...config.planner, ...patch.planner }),
    evaluation,
    replyRisk:
      patch.replyRisk === undefined
        ? config.replyRisk
        : freezeSection({ ...config.replyRisk, ...patch.replyRisk }),
    policy:
      patch.policy === undefined
        ? config.policy
        : freezeSection({ ...config.policy, ...patch.policy }),
  });
  validateAutomoveConfig(next);
  return next;
}

function automoveConfigFromBudget(
  preference: SmartAutomovePreference,
  requestedDepth: number,
  requestedMaxVisitedNodes: number,
): AutomoveConfig {
  const depth = clamp(
    Math.trunc(requestedDepth),
    MIN_SEARCH_DEPTH,
    MAX_SEARCH_DEPTH,
  );
  const maxVisitedNodes = clamp(
    Math.trunc(requestedMaxVisitedNodes),
    MIN_MAX_VISITED_NODES,
    MAX_MAX_VISITED_NODES,
  );
  const rootBranchLimit = clamp(Math.trunc(maxVisitedNodes / 24), 4, 28);
  const nodeBranchLimit = clamp(Math.trunc(maxVisitedNodes / 40), 4, 18);
  const rootEnumerationLimit = clamp(rootBranchLimit * 5, rootBranchLimit, 180);
  const nodeEnumerationLimit = clamp(nodeBranchLimit * 3, nodeBranchLimit, 96);

  return defineAutomoveConfig({
    budget: {
      preference,
      depth,
      maxVisitedNodes,
      exactLiteRootCalls: 0,
      exactLiteStaticCalls: 0,
      extensionNodeShareBp: AUTOMOVE_SEARCH_CONSTANTS.extensionNodeShareBp,
      quiescenceNodes: AUTOMOVE_SEARCH_CONSTANTS.quiescenceNodes,
    },
    search: {
      rootEnumerationLimit,
      rootBranchLimit,
      nodeEnumerationLimit,
      nodeBranchLimit,
      twoPassRootAllocation: false,
      volatilityFocus: false,
      selectiveExtensions: false,
      maxExtensionsPerPath: AUTOMOVE_SEARCH_CONSTANTS.maxExtensionsPerPath,
      quietReductions: false,
      quietReductionDepthThreshold: 3,
      quiescence: false,
      quiescenceEnumerationLimit:
        AUTOMOVE_SEARCH_CONSTANTS.quiescenceEnumerationLimit,
      futilityPruning: false,
      futilityMargin: AUTOMOVE_SEARCH_CONSTANTS.futilityMargin,
      transpositionTable: true,
      transpositionCapacity: AUTOMOVE_SEARCH_CONSTANTS.transpositionCapacity,
      exactLiteChecks: false,
      exactRootAnalysis: false,
    },
    planner: {
      enabled: false,
      rerankHeads: false,
      lowBudgetGuard: false,
      midTurnTacticalGuard: false,
      secondaryAnalysis: true,
      selectedFollowupProjection: true,
      mode: AUTOMOVE_TURN_ENGINE_MODE.Baseline,
      ownSeedCap: 0,
      ownBeam: 0,
      perNodeFamilyCap: 0,
      stepCap: 0,
      opponentSeedCap: 0,
      opponentBeam: 0,
      replySeedCap: 0,
      replyBeam: 0,
      expansionCap: 0,
      spiritFamily: false,
    },
    evaluation: {
      weights: DEFAULT_SCORING_WEIGHTS,
      cacheCapacity: AUTOMOVE_SEARCH_CONSTANTS.evaluationCacheCapacity,
      rootManaHandoffPenalty: ROOT_MANA_HANDOFF_PENALTY,
      rootBacktrackPenalty: ROOT_BACKTRACK_PENALTY,
      potionSpendPenalty:
        depth >= 3 ? POTION_SPEND_PENALTY_NORMAL : POTION_SPEND_PENALTY_FAST,
      supermanaProgressBonus: SOFT_SUPERMANA_PROGRESS_BONUS,
      supermanaScoreBonus: SOFT_SUPERMANA_SCORE_BONUS,
      opponentManaProgressBonus: SOFT_OPPONENT_MANA_PROGRESS_BONUS,
      opponentManaScoreBonus: SOFT_OPPONENT_MANA_SCORE_BONUS,
      softManaHandoffPenalty: SOFT_MANA_HANDOFF_PENALTY,
      softRoundtripPenalty: SOFT_ROUNDTRIP_PENALTY,
    },
    replyRisk: {
      enabled: false,
      scoreMargin: ROOT_REPLY_RISK_SCORE_MARGIN,
      shortlistLimit: ROOT_REPLY_RISK_SHORTLIST_FAST,
      replyLimit: ROOT_REPLY_RISK_REPLY_LIMIT_FAST,
      nodeShareBp: ROOT_REPLY_RISK_NODE_SHARE_BP_FAST,
      antiHelpScoreMargin: ROOT_ANTI_HELP_SCORE_MARGIN,
      antiHelpReplyLimit: ROOT_ANTI_HELP_REPLY_LIMIT_FAST,
      safetyRerank: false,
      deepSafetyFloor: false,
      deterministicTiebreak: false,
      preferCleanRoots: false,
      lateSafeManaRootPreference: false,
    },
    policy: {
      targetedDrainerAttackFallback: false,
      forcedTacticalPrepass: true,
      preferSpiritDevelopment: true,
      hardSpiritDeployment: false,
      supermanaPrepassException: false,
      drainerSafetyScoreMargin: ROOT_DRAINER_SAFETY_SCORE_MARGIN,
      efficiencyScoreMargin: ROOT_EFFICIENCY_SCORE_MARGIN,
    },
  });
}

function withRuntimeSearchShape(config: AutomoveConfig): AutomoveConfig {
  if (config.budget.depth >= 3) {
    const rootBranchLimit = clamp(config.search.rootBranchLimit + 10, 6, 36);
    const nodeBranchLimit = clamp(
      subtractSaturating(config.search.nodeBranchLimit, 11),
      6,
      18,
    );
    return patchAutomoveConfig(config, {
      search: {
        rootBranchLimit,
        nodeBranchLimit,
        rootEnumerationLimit: clamp(rootBranchLimit * 6, rootBranchLimit, 220),
        nodeEnumerationLimit: clamp(nodeBranchLimit * 4, nodeBranchLimit, 108),
      },
      evaluation: { weights: BALANCED_DISTANCE_SCORING_WEIGHTS },
    });
  }
  return patchAutomoveConfig(config, {
    evaluation: { weights: MANA_RACE_LITE_D2_TUNED_SCORING_WEIGHTS },
  });
}

function withFastWideRootShape(config: AutomoveConfig): AutomoveConfig {
  const rootBranchLimit = clamp(config.search.rootBranchLimit + 8, 8, 40);
  const nodeBranchLimit = clamp(
    subtractSaturating(config.search.nodeBranchLimit, 2),
    6,
    18,
  );
  return patchAutomoveConfig(config, {
    search: {
      rootBranchLimit,
      nodeBranchLimit,
      rootEnumerationLimit: clamp(rootBranchLimit * 6, rootBranchLimit, 240),
      nodeEnumerationLimit: clamp(nodeBranchLimit * 4, nodeBranchLimit, 108),
    },
  });
}

function withNormalDeeperShape(config: AutomoveConfig): AutomoveConfig {
  const rootBranchLimit = clamp(config.search.rootBranchLimit, 8, 36);
  const nodeBranchLimit = clamp(config.search.nodeBranchLimit + 3, 9, 18);
  return patchAutomoveConfig(config, {
    search: {
      rootBranchLimit,
      nodeBranchLimit,
      rootEnumerationLimit: clamp(rootBranchLimit * 6, rootBranchLimit, 220),
      nodeEnumerationLimit: clamp(nodeBranchLimit * 6, nodeBranchLimit, 132),
    },
  });
}

function automoveConfigFromPreference(
  preference: SmartAutomovePreference,
): AutomoveConfig {
  const budget = AUTOMOVE_SEARCH_BUDGETS[preference];
  const runtime = withRuntimeSearchShape(
    automoveConfigFromBudget(preference, budget.depth, budget.maxVisitedNodes),
  );

  if (preference === "fast") {
    return patchAutomoveConfig(withFastWideRootShape(runtime), {
      search: {
        twoPassRootAllocation: false,
        selectiveExtensions: false,
        quietReductions: true,
        volatilityFocus: false,
      },
      evaluation: {
        weights: RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS_POTION_PREF,
        rootManaHandoffPenalty: 300,
        rootBacktrackPenalty: 220,
        potionSpendPenalty: 220,
        supermanaProgressBonus: 320,
        supermanaScoreBonus: 600,
        opponentManaProgressBonus: 200,
        opponentManaScoreBonus: 310,
        softManaHandoffPenalty: 280,
        softRoundtripPenalty: 220,
      },
      replyRisk: {
        enabled: true,
        scoreMargin: 125,
        shortlistLimit: 4,
        replyLimit: 10,
        nodeShareBp: 650,
        antiHelpScoreMargin: 220,
        antiHelpReplyLimit: ROOT_ANTI_HELP_REPLY_LIMIT_FAST,
        safetyRerank: false,
        deepSafetyFloor: false,
        deterministicTiebreak: false,
        preferCleanRoots: false,
      },
      policy: {
        forcedTacticalPrepass: true,
        preferSpiritDevelopment: true,
        hardSpiritDeployment: false,
        supermanaPrepassException: true,
        drainerSafetyScoreMargin: ROOT_DRAINER_SAFETY_SCORE_MARGIN,
        efficiencyScoreMargin: 1_700,
      },
    });
  }

  const deeper = withNormalDeeperShape(runtime);
  if (preference === "normal") {
    const firstNodeBudget = clamp(
      scaleFloor(deeper.budget.maxVisitedNodes, 3, 2),
      deeper.budget.maxVisitedNodes,
      MAX_MAX_VISITED_NODES,
    );
    const maxVisitedNodes = clamp(
      scaleFloor(firstNodeBudget, 112, 100),
      firstNodeBudget,
      MAX_MAX_VISITED_NODES,
    );
    const rootBranchLimit = clamp(
      subtractSaturating(deeper.search.rootBranchLimit, 1),
      12,
      38,
    );
    const nodeBranchLimit = clamp(deeper.search.nodeBranchLimit + 2, 8, 18);
    return patchAutomoveConfig(deeper, {
      budget: { maxVisitedNodes },
      search: {
        rootBranchLimit,
        nodeBranchLimit,
        rootEnumerationLimit: clamp(rootBranchLimit * 6, rootBranchLimit, 240),
        nodeEnumerationLimit: clamp(
          (nodeBranchLimit + 2) * 6,
          nodeBranchLimit,
          156,
        ),
        twoPassRootAllocation: true,
        selectiveExtensions: false,
        quietReductions: false,
        volatilityFocus: true,
      },
      evaluation: {
        rootManaHandoffPenalty: 340,
        rootBacktrackPenalty: 240,
        potionSpendPenalty: 130,
        supermanaProgressBonus: 240,
        supermanaScoreBonus: 300,
        opponentManaProgressBonus: 220,
        opponentManaScoreBonus: 280,
        softManaHandoffPenalty: 340,
        softRoundtripPenalty: 260,
      },
      replyRisk: {
        enabled: true,
        scoreMargin: 145,
        shortlistLimit: 7,
        replyLimit: 16,
        nodeShareBp: 1_350,
        antiHelpScoreMargin: 300,
        antiHelpReplyLimit: 10,
        safetyRerank: true,
        deepSafetyFloor: true,
        deterministicTiebreak: false,
        preferCleanRoots: true,
      },
      policy: {
        forcedTacticalPrepass: true,
        preferSpiritDevelopment: true,
        hardSpiritDeployment: true,
        drainerSafetyScoreMargin: 4_200,
        efficiencyScoreMargin: 1_400,
      },
    });
  }

  const rootBranchLimit = clamp(deeper.search.rootBranchLimit, 14, 34);
  const nodeBranchLimit = clamp(deeper.search.nodeBranchLimit, 9, 15);
  return patchAutomoveConfig(deeper, {
    budget: { maxVisitedNodes: PRO_MAX_VISITED_NODES },
    search: {
      rootBranchLimit,
      nodeBranchLimit,
      rootEnumerationLimit: clamp(rootBranchLimit * 6, rootBranchLimit, 204),
      nodeEnumerationLimit: clamp(
        (nodeBranchLimit + 2) * 6,
        nodeBranchLimit,
        132,
      ),
      twoPassRootAllocation: true,
      selectiveExtensions: true,
      quietReductions: true,
      volatilityFocus: true,
      futilityPruning: true,
      quietReductionDepthThreshold: 2,
    },
    evaluation: {
      rootManaHandoffPenalty: 340,
      rootBacktrackPenalty: 240,
      potionSpendPenalty: 130,
      supermanaProgressBonus: 240,
      supermanaScoreBonus: 300,
      opponentManaProgressBonus: 280,
      opponentManaScoreBonus: 340,
      softManaHandoffPenalty: 340,
      softRoundtripPenalty: 260,
    },
    replyRisk: {
      enabled: true,
      scoreMargin: 165,
      shortlistLimit: 9,
      replyLimit: 24,
      nodeShareBp: 2_000,
      antiHelpScoreMargin: 300,
      antiHelpReplyLimit: 10,
      safetyRerank: true,
      deepSafetyFloor: true,
      deterministicTiebreak: false,
      preferCleanRoots: true,
    },
    policy: {
      forcedTacticalPrepass: false,
      preferSpiritDevelopment: true,
      hardSpiritDeployment: true,
      drainerSafetyScoreMargin: 4_800,
      efficiencyScoreMargin: 1_400,
    },
  });
}

function withRuntimeScoringWeights(
  game: MonsGame,
  config: AutomoveConfig,
): AutomoveConfig {
  const scoringWeights =
    config.budget.depth < 3
      ? RUNTIME_FAST_BOOLEAN_DRAINER_SCORING_WEIGHTS_POTION_PREF
      : runtimePhaseAdaptiveWalkThreatMediumScoringProfile(
          game,
          config.budget.depth,
        ).weights;
  return patchAutomoveConfig(config, {
    evaluation: { weights: scoringWeights },
    budget: {
      maxVisitedNodes:
        config.budget.depth >= 3
          ? scaleFloor(config.budget.maxVisitedNodes, 120, 100)
          : config.budget.maxVisitedNodes,
    },
  });
}

function applyRuntimeNormalFastPolicyBlock(
  config: AutomoveConfig,
): AutomoveConfig {
  const rootBranchLimit = clamp(config.search.rootBranchLimit + 5, 12, 40);
  const nodeBranchLimit = clamp(
    subtractSaturating(config.search.nodeBranchLimit, 2),
    8,
    18,
  );
  return patchAutomoveConfig(config, {
    search: {
      rootBranchLimit,
      nodeBranchLimit,
      rootEnumerationLimit: clamp(rootBranchLimit * 6, rootBranchLimit, 240),
      nodeEnumerationLimit: clamp(nodeBranchLimit * 4, nodeBranchLimit, 108),
      twoPassRootAllocation: false,
      selectiveExtensions: false,
      quietReductions: true,
      volatilityFocus: false,
    },
    evaluation: {
      weights: RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS_POTION_PREF,
      rootManaHandoffPenalty: 300,
      rootBacktrackPenalty: 220,
      potionSpendPenalty: POTION_SPEND_PENALTY_NORMAL,
      supermanaProgressBonus: 320,
      supermanaScoreBonus: 600,
      opponentManaProgressBonus: 200,
      opponentManaScoreBonus: 310,
      softManaHandoffPenalty: 280,
      softRoundtripPenalty: 220,
    },
    replyRisk: {
      scoreMargin: 125,
      shortlistLimit: 4,
      replyLimit: 10,
      nodeShareBp: 650,
      antiHelpScoreMargin: 220,
      antiHelpReplyLimit: ROOT_ANTI_HELP_REPLY_LIMIT_FAST,
      safetyRerank: false,
      deepSafetyFloor: false,
      deterministicTiebreak: false,
      preferCleanRoots: false,
    },
    policy: {
      hardSpiritDeployment: false,
      supermanaPrepassException: true,
      drainerSafetyScoreMargin: ROOT_DRAINER_SAFETY_SCORE_MARGIN,
      efficiencyScoreMargin: 1_700,
    },
  });
}

function applyRuntimeNormalFastCoreBudgetSpendProfile(
  config: AutomoveConfig,
): AutomoveConfig {
  const policy = applyRuntimeNormalFastPolicyBlock(config);
  const rootBranchLimit = clamp(policy.search.rootBranchLimit + 2, 12, 40);
  return patchAutomoveConfig(policy, {
    budget: {
      exactLiteRootCalls: 1,
      exactLiteStaticCalls: 1,
      maxVisitedNodes: scaleFloor(policy.budget.maxVisitedNodes, 130, 100),
    },
    search: {
      exactLiteChecks: true,
      rootBranchLimit,
      rootEnumerationLimit: clamp(rootBranchLimit * 6, rootBranchLimit, 240),
    },
    replyRisk: {
      shortlistLimit: Math.max(policy.replyRisk.shortlistLimit, 5),
      replyLimit: Math.max(policy.replyRisk.replyLimit, 12),
      nodeShareBp: Math.max(policy.replyRisk.nodeShareBp, 900),
    },
  });
}

function applyProPrimaryProfile(
  game: MonsGame,
  config: AutomoveConfig,
): AutomoveConfig {
  const rootBranchLimitBeforeFinalCap = clamp(
    config.search.rootBranchLimit,
    14,
    34,
  );
  const nodeBranchLimitBeforeFinalCap = clamp(
    config.search.nodeBranchLimit,
    9,
    15,
  );
  return patchAutomoveConfig(config, {
    budget: {
      maxVisitedNodes: scaleFloor(PRO_MAX_VISITED_NODES, 9, 8),
    },
    search: {
      rootBranchLimit: Math.min(rootBranchLimitBeforeFinalCap + 1, 16),
      nodeBranchLimit: Math.min(nodeBranchLimitBeforeFinalCap + 1, 12),
      rootEnumerationLimit: clamp(
        rootBranchLimitBeforeFinalCap * 6,
        rootBranchLimitBeforeFinalCap,
        204,
      ),
      nodeEnumerationLimit: clamp(
        (nodeBranchLimitBeforeFinalCap + 2) * 6,
        nodeBranchLimitBeforeFinalCap,
        132,
      ),
      futilityPruning: true,
      quietReductions: true,
      quietReductionDepthThreshold: 2,
      selectiveExtensions: true,
      quiescence: true,
    },
    planner: { rerankHeads: true },
    evaluation: {
      weights: runtimePhaseAdaptiveAttackerProximityScoringProfile(
        game,
        config.budget.depth,
      ).weights,
      opponentManaProgressBonus: 320,
      opponentManaScoreBonus: 400,
    },
    replyRisk: {
      enabled: true,
      scoreMargin: 165,
      shortlistLimit: 9,
      replyLimit: 24,
      nodeShareBp: 2_000,
      safetyRerank: true,
      deepSafetyFloor: true,
      deterministicTiebreak: true,
    },
    policy: {
      forcedTacticalPrepass: false,
      drainerSafetyScoreMargin: 4_800,
    },
  });
}

function withPreExactRuntimePolicy(config: AutomoveConfig): AutomoveConfig {
  return patchAutomoveConfig(config, {
    budget: { exactLiteRootCalls: 0, exactLiteStaticCalls: 0 },
    search: { exactLiteChecks: false },
  });
}

export function withProductionPlanner(config: AutomoveConfig): AutomoveConfig {
  return patchAutomoveConfig(config, {
    planner: {
      rerankHeads: false,
      enabled: true,
      mode: AUTOMOVE_TURN_ENGINE_MODE.Production,
      ownSeedCap: 14,
      ownBeam: 5,
      perNodeFamilyCap: 4,
      stepCap: 6,
      opponentSeedCap: 6,
      opponentBeam: 2,
      replySeedCap: 3,
      replyBeam: 1,
      expansionCap: 176,
      spiritFamily: true,
      lowBudgetGuard: true,
      midTurnTacticalGuard: true,
    },
    replyRisk: {
      enabled: false,
      lateSafeManaRootPreference: true,
    },
    policy: { targetedDrainerAttackFallback: true },
  });
}

export function automoveConfigForGame(
  game: MonsGame,
  preference: SmartAutomovePreference,
): AutomoveConfig {
  let config = withRuntimeScoringWeights(
    game,
    automoveConfigFromPreference(preference),
  );
  if (preference === "pro") {
    config = applyProPrimaryProfile(game, config);
  }
  config = withPreExactRuntimePolicy(config);
  if (preference === "normal") {
    config = applyRuntimeNormalFastCoreBudgetSpendProfile(config);
  }
  return config;
}
