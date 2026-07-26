export {
  EMPTY_TURN_UTILITY,
  OpportunityKind,
  TURN_PLAN_FAMILY_CACHE_TAG,
  TURN_PLAN_FAMILY_PRIORITY_ORDER,
  TurnEngineMode,
  TurnPlanFamily,
  createTurnUtility,
  compareUtilityPrimaryAxes,
  compareTurnUtilities,
  familyRank,
  turnEngineComparePlans,
  turnSnapshotFromGame,
  utilityHasNonnegativeDenyGain,
  utilityHasScoreDeltaForce,
  utilityImprovesNonScoreOverrideAxes,
  utilityPassesOverrideGuard,
  utilityStrictlyDominatesOverrideAxes,
  utilitySupportsFamilyFallback,
  utilitySupportsPrimaryAxesEvalTolerance,
  utilitySupportsTemporaryRiskRecovery,
} from "./turn-types.js";
export type {
  OpportunityBudget,
  OpportunityDelta,
  TurnAction,
  TurnEngineConfig,
  TurnOpportunity,
  TurnPackageMeta,
  TurnPlan,
  TurnSnapshot,
  TurnUtility,
} from "./turn-types.js";
export { clearTurnEnginePlanCache } from "./turn-cache.js";
export { turnEngineEvaluateStateUtility } from "./turn-evaluation.js";
export {
  turnEngineCachedStep,
  turnEngineCandidatePlan,
  turnEngineCandidatePlanFromAllowedHeads,
  turnEngineCandidatePlanLive,
  turnEngineCommitPlan,
  turnEngineEvaluatePlanWithReplies,
  turnEngineNextInputsFromAllowedHeads,
  turnEngineStoreCachedStep,
} from "./turn-planner.js";
export { discoverTurnOpportunities } from "./turn-opportunities.js";
