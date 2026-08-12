import { validateScoringProfile } from "../scoring/profile-validation.js";
import {
  AUTOMOVE_SEARCH_BUDGETS,
  MAX_MAX_VISITED_NODES,
  MAX_SEARCH_DEPTH,
  MIN_SEARCH_DEPTH,
} from "./constants.js";
import {
  AUTOMOVE_TURN_ENGINE_MODE,
  type AutomoveBudgetConfig,
  type AutomoveConfig,
  type AutomoveEvaluationConfig,
  type AutomovePlannerConfig,
  type AutomovePolicyConfig,
  type AutomoveReplyRiskConfig,
  type AutomoveTreeSearchConfig,
} from "./types.js";

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
    throw new TypeError(`${label} must contain exactly: ${expected.join(", ")}`);
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
    throw new RangeError(`unknown automove preference: ${config.budget.preference}`);
  }
  const plannerMode = (value["planner"] as Readonly<Record<string, unknown>>)["mode"];
  if (
    plannerMode !== AUTOMOVE_TURN_ENGINE_MODE.Baseline &&
    plannerMode !== AUTOMOVE_TURN_ENGINE_MODE.Production
  ) {
    throw new RangeError(`unknown planner mode: ${String(plannerMode)}`);
  }
  validateConfigNumbers("budget", config.budget, BUDGET_NUMBER_KEYS);
  validateConfigNumbers("search", config.search, SEARCH_NUMBER_KEYS);
  validateConfigNumbers("planner", config.planner, PLANNER_NUMBER_KEYS);
  validateConfigNumbers("evaluation", config.evaluation, EVALUATION_NUMBER_KEYS);
  validateConfigNumbers("replyRisk", config.replyRisk, REPLY_RISK_NUMBER_KEYS);
  validateConfigNumbers("policy", config.policy, POLICY_NUMBER_KEYS);
  validateConfigBooleans("search", config.search, SEARCH_BOOLEAN_KEYS);
  validateConfigBooleans("planner", config.planner, PLANNER_BOOLEAN_KEYS);
  validateConfigBooleans("replyRisk", config.replyRisk, REPLY_RISK_BOOLEAN_KEYS);
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
    throw new RangeError(`basis-point shares must not exceed ${MAX_BASIS_POINTS}`);
  }
}
