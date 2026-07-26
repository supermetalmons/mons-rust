export {
  clearReplyRiskCache,
  selectedOverrideConfigKey,
} from "./reply-risk/cache.js";
export {
  blackManaWindowProgressCompetition,
  closePositiveScoreCompetition,
  safeProgressCompetition,
  whiteSpiritFollowupSetupCompetition,
} from "./reply-risk/competition.js";
export {
  blackPlainSpiritFollowupReplyOrder,
  earlyBlackManaProgressReplyOrder,
  earlyBlackPlainSpiritManaReplyOrder,
  earlyBlackPlainSpiritSiblingOrder,
  isProductionModeBlackPlainSpiritFollowupSetupPair,
  safeNonSpiritFollowupOrder,
  spiritFollowupFloorOrder,
  whiteSpiritFollowupSetupReplyOrder,
} from "./reply-risk/followup-ordering.js";
export {
  compareRankedReplyRiskEvaluations,
  replyRiskGuardShortlistIndices,
  rootProgressOrSetupBetter,
  safePlainSpiritCompetition,
  sameNonTacticalProgressLane,
} from "./reply-risk/ranking.js";
export { isBetterReplyRiskCandidate } from "./reply-risk/arbitration.js";
export { pickRootWithReplyRiskGuard } from "./reply-risk/guarded-pick.js";
export {
  isProductionModeWhiteManaSiblingPair,
  productionWhiteTurnFourManaSiblingReentry,
} from "./reply-risk/white-reentry.js";
export {
  buildSpiritRootProjections,
  canChallengeSpiritPreferenceRoot,
  canChallengeSpiritPreferenceRootWithRecovery,
  canTurnEngineProjectReplyRiskRoot,
  rootReplyRiskSnapshotWithProjection,
  shouldUseReplyRiskProjectionForRoot,
  spiritFollowupFloorScore,
  turnEngineRootPlanUtility,
  turnEngineReplyRiskProjections,
  turnEngineSelectedOverrideUtility,
} from "./reply-risk/projection.js";
export {
  compareSpiritProjectionPlans,
  spiritProjectionChallengeOrder,
} from "./reply-risk/spirit-ordering.js";
export { rootReplyRiskSnapshot } from "./reply-risk/snapshot.js";
export type {
  ReplyRiskComparisonContext,
  ReplyRiskHooks,
  RootReplyRiskSnapshot,
  TurnEngineRootProjection,
} from "./reply-risk/types.js";
