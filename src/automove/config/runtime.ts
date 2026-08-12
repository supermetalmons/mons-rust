import type { MonsGame } from "../../engine/game/mons-game.js";
import {
  RUNTIME_FAST_BOOLEAN_DRAINER_SCORING_WEIGHTS_POTION_PREF,
  RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS_POTION_PREF,
} from "../scoring/presets.js";
import {
  POTION_SPEND_PENALTY_NORMAL,
  PRO_MAX_VISITED_NODES,
  ROOT_ANTI_HELP_REPLY_LIMIT_FAST,
  ROOT_DRAINER_SAFETY_SCORE_MARGIN,
} from "./constants.js";
import { clamp, scaleFloor, subtractSaturating } from "./math.js";
import { patchAutomoveConfig } from "./patch.js";
import { automoveConfigFromPreference } from "./presets.js";
import {
  runtimePhaseAdaptiveAttackerProximityScoringProfile,
  runtimePhaseAdaptiveWalkThreatMediumScoringProfile,
} from "./profiles.js";
import {
  AUTOMOVE_TURN_ENGINE_MODE,
  type AutomoveConfig,
  type SmartAutomovePreference,
} from "./types.js";

function withRuntimeScoringWeights(
  game: MonsGame,
  config: AutomoveConfig,
): AutomoveConfig {
  const scoringWeights =
    config.budget.depth < 3
      ? RUNTIME_FAST_BOOLEAN_DRAINER_SCORING_WEIGHTS_POTION_PREF
      : runtimePhaseAdaptiveWalkThreatMediumScoringProfile(game, config.budget.depth)
          .weights;
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

function applyRuntimeNormalFastPolicyBlock(config: AutomoveConfig): AutomoveConfig {
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
  const rootBranchLimitBeforeFinalCap = clamp(config.search.rootBranchLimit, 14, 34);
  const nodeBranchLimitBeforeFinalCap = clamp(config.search.nodeBranchLimit, 9, 15);
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
