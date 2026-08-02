export {
  EXACT_TURN_TACTICAL_ALL_FLAGS,
  EXACT_TURN_TACTICAL_NEED_OPPONENT_MANA_PROGRESS,
  EXACT_TURN_TACTICAL_NEED_SCORE_WINDOW,
  EXACT_TURN_TACTICAL_NEED_SPIRIT_DENIAL,
  EXACT_TURN_TACTICAL_NEED_SPIRIT_SCORE,
  EXACT_TURN_TACTICAL_NEED_SUPERMANA_PROGRESS,
  ExactStrategicAnalysis,
  defaultColorSummary,
  defaultOpportunityContext,
  type ExactColorSummary,
  type ExactOpportunityBudget,
  type ExactOpportunityContext,
  type ExactSpiritSummary,
  type ExactTurnSummary,
  type ExactTurnTacticalProjection,
} from "./exact-types.js";

export { exactBoardHash, exactSearchStateHash } from "./exact-hash.js";

export {
  AttackReachSummary,
  attackReachSummary,
  attackReachSummaryForTargets,
  attackReachSummaryTargetLocations,
  canAttackTargetOnBoardWithHash,
  drainerImmediateThreats,
  exactOwnDrainerSafetyScoreWithHash,
  isDrainerExactlySafeNextTurnOnBoard,
  isDrainerExactlySafeNextTurnOnBoardWithHash,
  isDrainerUnderImmediateThreat,
  isDrainerUnderWalkThreatWithHash,
} from "./exact-attack.js";

export {
  exactBestScoreStepsOnBoard,
  exactSecureSpecificManaPathFrom,
} from "./exact-mana.js";

export {
  canAttackOpponentDrainerThisTurn,
  exactOpportunityContext,
  exactOpportunityContextWithSearchHash,
  exactSameTurnScoreWindowWithSearchHash,
  exactStrategicAnalysis,
  exactStrategicAnalysisWithSearchHash,
  exactTurnSummary,
  exactTurnTacticalProjectionWithSearchHash,
} from "./exact-analysis.js";

export { clearExactStateAnalysisCache } from "./exact-cache.js";
