import {
  BALANCED_DISTANCE_SCORING_WEIGHTS,
  DEFAULT_SCORING_WEIGHTS,
  MANA_RACE_LITE_D2_TUNED_SCORING_WEIGHTS,
  RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS_POTION_PREF,
} from "../scoring/presets.js";
import {
  AUTOMOVE_SEARCH_BUDGETS,
  AUTOMOVE_SEARCH_CONSTANTS,
  MAX_MAX_VISITED_NODES,
  MAX_SEARCH_DEPTH,
  MIN_MAX_VISITED_NODES,
  MIN_SEARCH_DEPTH,
  POTION_SPEND_PENALTY_FAST,
  POTION_SPEND_PENALTY_NORMAL,
  PRO_MAX_VISITED_NODES,
  ROOT_ANTI_HELP_REPLY_LIMIT_FAST,
  ROOT_ANTI_HELP_SCORE_MARGIN,
  ROOT_BACKTRACK_PENALTY,
  ROOT_DRAINER_SAFETY_SCORE_MARGIN,
  ROOT_EFFICIENCY_SCORE_MARGIN,
  ROOT_MANA_HANDOFF_PENALTY,
  ROOT_REPLY_RISK_NODE_SHARE_BP_FAST,
  ROOT_REPLY_RISK_REPLY_LIMIT_FAST,
  ROOT_REPLY_RISK_SCORE_MARGIN,
  ROOT_REPLY_RISK_SHORTLIST_FAST,
  SOFT_MANA_HANDOFF_PENALTY,
  SOFT_OPPONENT_MANA_PROGRESS_BONUS,
  SOFT_OPPONENT_MANA_SCORE_BONUS,
  SOFT_ROUNDTRIP_PENALTY,
  SOFT_SUPERMANA_PROGRESS_BONUS,
  SOFT_SUPERMANA_SCORE_BONUS,
} from "./constants.js";
import { clamp, scaleFloor, subtractSaturating } from "./math.js";
import { defineAutomoveConfig, patchAutomoveConfig } from "./patch.js";
import {
  AUTOMOVE_TURN_ENGINE_MODE,
  type AutomoveConfig,
  type SmartAutomovePreference,
} from "./types.js";

function automoveConfigFromBudget(
  preference: SmartAutomovePreference,
  requestedDepth: number,
  requestedMaxVisitedNodes: number,
): AutomoveConfig {
  const depth = clamp(Math.trunc(requestedDepth), MIN_SEARCH_DEPTH, MAX_SEARCH_DEPTH);
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
      quiescenceEnumerationLimit: AUTOMOVE_SEARCH_CONSTANTS.quiescenceEnumerationLimit,
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

export function automoveConfigFromPreference(
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
        nodeEnumerationLimit: clamp((nodeBranchLimit + 2) * 6, nodeBranchLimit, 156),
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
      nodeEnumerationLimit: clamp((nodeBranchLimit + 2) * 6, nodeBranchLimit, 132),
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
