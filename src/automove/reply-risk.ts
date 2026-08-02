export {
  clearReplyRiskCache,
  selectedOverrideConfigKey,
} from "./reply-risk/cache.js";
export { whiteSpiritFollowupSetupCompetition } from "./reply-risk/competition.js";
export {
  blackPlainSpiritFollowupReplyOrder,
  isProductionModeBlackPlainSpiritFollowupSetupPair,
  spiritFollowupFloorOrder,
} from "./reply-risk/followup-ordering.js";
export {
  replyRiskGuardShortlistIndices,
  rootProgressOrSetupBetter,
  sameNonTacticalProgressLane,
} from "./reply-risk/ranking.js";
export { pickRootWithReplyRiskGuard } from "./reply-risk/guarded-pick.js";
export {
  isProductionModeWhiteManaSiblingPair,
  productionWhiteTurnFourManaSiblingReentry,
} from "./reply-risk/white-reentry.js";
export {
  buildSpiritRootProjections,
  canChallengeSpiritPreferenceRoot,
  canChallengeSpiritPreferenceRootWithRecovery,
  spiritFollowupFloorScore,
  turnEngineRootPlanUtility,
  turnEngineSelectedOverrideUtility,
} from "./reply-risk/projection.js";
export {
  compareSpiritProjectionPlans,
  spiritProjectionChallengeOrder,
} from "./reply-risk/spirit-ordering.js";
export { rootReplyRiskSnapshot } from "./reply-risk/snapshot.js";
export type { ReplyRiskHooks } from "./reply-risk/types.js";
