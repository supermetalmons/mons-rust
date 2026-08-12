export const MIN_SEARCH_DEPTH = 1;
export const MAX_SEARCH_DEPTH = 5;
export const MIN_MAX_VISITED_NODES = 32;
export const MAX_MAX_VISITED_NODES = 180_000;
const FAST_DEPTH = 2;
const FAST_MAX_VISITED_NODES = 480;
const NORMAL_DEPTH = 3;
const NORMAL_MAX_VISITED_NODES = 3_800;
const PRO_DEPTH = 4;
export const PRO_MAX_VISITED_NODES = Math.trunc((NORMAL_MAX_VISITED_NODES * 369) / 100);

export const ROOT_REPLY_RISK_SCORE_MARGIN = 140;
export const ROOT_REPLY_RISK_SHORTLIST_FAST = 3;
export const ROOT_REPLY_RISK_REPLY_LIMIT_FAST = 8;
export const ROOT_REPLY_RISK_NODE_SHARE_BP_FAST = 600;
export const ROOT_ANTI_HELP_SCORE_MARGIN = 180;
export const ROOT_ANTI_HELP_REPLY_LIMIT_FAST = 6;
export const ROOT_DRAINER_SAFETY_SCORE_MARGIN = 2_200;
export const ROOT_MANA_HANDOFF_PENALTY = 220;
export const ROOT_BACKTRACK_PENALTY = 140;
export const ROOT_EFFICIENCY_SCORE_MARGIN = 2_500;
export const POTION_SPEND_PENALTY_FAST = 340;
export const POTION_SPEND_PENALTY_NORMAL = 260;
export const SOFT_SUPERMANA_PROGRESS_BONUS = 240;
export const SOFT_SUPERMANA_SCORE_BONUS = 420;
export const SOFT_OPPONENT_MANA_PROGRESS_BONUS = 210;
export const SOFT_OPPONENT_MANA_SCORE_BONUS = 360;
export const SOFT_MANA_HANDOFF_PENALTY = 220;
export const SOFT_ROUNDTRIP_PENALTY = 140;

export const AUTOMOVE_SEARCH_CONSTANTS = Object.freeze({
  maxExtensionsPerPath: 1,
  extensionNodeShareBp: 1_500,
  quiescenceNodes: 120,
  quiescenceEnumerationLimit: 12,
  futilityMargin: 2_300,
  transpositionCapacity: 12_000,
  evaluationCacheCapacity: 32_768,
});

export const AUTOMOVE_SEARCH_BUDGETS = Object.freeze({
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
