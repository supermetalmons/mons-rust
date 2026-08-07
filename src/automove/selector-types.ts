import {
  MonKind,
  isMonFainted,
  itemMon,
  type Color,
  type Input,
} from "../engine/domain.js";
import type { MonsGame } from "../engine/game.js";
import type { ScoringWeights } from "./scoring.js";

export type SmartAutomovePreference = "fast" | "normal" | "pro";

/** Value-shaped modes keep selector configuration independent of the engine. */
export const AUTOMOVE_TURN_ENGINE_MODE = Object.freeze({
  Baseline: "baseline",
  Production: "production",
} as const);

type AutomoveTurnEngineMode =
  (typeof AUTOMOVE_TURN_ENGINE_MODE)[keyof typeof AUTOMOVE_TURN_ENGINE_MODE];

export type AutomoveBudgetConfig = {
  readonly preference: SmartAutomovePreference;
  readonly depth: number;
  readonly maxVisitedNodes: number;
  readonly exactLiteRootCalls: number;
  readonly exactLiteStaticCalls: number;
  readonly extensionNodeShareBp: number;
  readonly quiescenceNodes: number;
};

export type AutomoveTreeSearchConfig = {
  readonly rootEnumerationLimit: number;
  readonly rootBranchLimit: number;
  readonly nodeEnumerationLimit: number;
  readonly nodeBranchLimit: number;
  readonly twoPassRootAllocation: boolean;
  readonly volatilityFocus: boolean;
  readonly selectiveExtensions: boolean;
  readonly maxExtensionsPerPath: number;
  readonly quietReductions: boolean;
  readonly quietReductionDepthThreshold: number;
  readonly quiescence: boolean;
  readonly quiescenceEnumerationLimit: number;
  readonly futilityPruning: boolean;
  readonly futilityMargin: number;
  readonly transpositionTable: boolean;
  readonly transpositionCapacity: number;
  readonly exactLiteChecks: boolean;
  readonly exactRootAnalysis: boolean;
};

export type AutomovePlannerConfig = {
  readonly enabled: boolean;
  readonly rerankHeads: boolean;
  readonly lowBudgetGuard: boolean;
  readonly midTurnTacticalGuard: boolean;
  readonly secondaryAnalysis: boolean;
  readonly selectedFollowupProjection: boolean;
  readonly mode: AutomoveTurnEngineMode;
  readonly ownSeedCap: number;
  readonly ownBeam: number;
  readonly perNodeFamilyCap: number;
  readonly stepCap: number;
  readonly opponentSeedCap: number;
  readonly opponentBeam: number;
  readonly replySeedCap: number;
  readonly replyBeam: number;
  readonly expansionCap: number;
  readonly spiritFamily: boolean;
};

export type AutomoveEvaluationConfig = {
  readonly weights: ScoringWeights;
  readonly cacheCapacity: number;
  readonly rootManaHandoffPenalty: number;
  readonly rootBacktrackPenalty: number;
  readonly potionSpendPenalty: number;
  readonly supermanaProgressBonus: number;
  readonly supermanaScoreBonus: number;
  readonly opponentManaProgressBonus: number;
  readonly opponentManaScoreBonus: number;
  readonly softManaHandoffPenalty: number;
  readonly softRoundtripPenalty: number;
};

export type AutomoveReplyRiskConfig = {
  readonly enabled: boolean;
  readonly scoreMargin: number;
  readonly shortlistLimit: number;
  readonly replyLimit: number;
  readonly nodeShareBp: number;
  readonly antiHelpScoreMargin: number;
  readonly antiHelpReplyLimit: number;
  readonly safetyRerank: boolean;
  readonly deepSafetyFloor: boolean;
  readonly deterministicTiebreak: boolean;
  readonly preferCleanRoots: boolean;
  readonly lateSafeManaRootPreference: boolean;
};

export type AutomovePolicyConfig = {
  readonly targetedDrainerAttackFallback: boolean;
  readonly forcedTacticalPrepass: boolean;
  readonly preferSpiritDevelopment: boolean;
  readonly hardSpiritDeployment: boolean;
  readonly supermanaPrepassException: boolean;
  readonly drainerSafetyScoreMargin: number;
  readonly efficiencyScoreMargin: number;
};

/** Complete immutable configuration shared by every automove stage. */
export type AutomoveConfig = {
  readonly budget: AutomoveBudgetConfig;
  readonly search: AutomoveTreeSearchConfig;
  readonly planner: AutomovePlannerConfig;
  readonly evaluation: AutomoveEvaluationConfig;
  readonly replyRisk: AutomoveReplyRiskConfig;
  readonly policy: AutomovePolicyConfig;
};

export type MoveClassFlags = {
  readonly immediateScore: boolean;
  readonly drainerAttack: boolean;
  readonly drainerSafetyRecover: boolean;
  readonly carrierProgress: boolean;
  readonly material: boolean;
  readonly quiet: boolean;
};

/** Observations shared by ranked candidates and completed root evaluations. */
export type RootObservation = {
  readonly rootRank: number;
  readonly inputs: readonly Input[];
  readonly game: MonsGame;
  readonly efficiency: number;
  readonly winsImmediately: boolean;
  readonly attacksOpponentDrainer: boolean;
  readonly ownDrainerVulnerable: boolean;
  readonly ownDrainerWalkVulnerable: boolean;
  readonly spiritDevelopment: boolean;
  readonly keepsAwakeSpiritOnBase: boolean;
  readonly manaHandoffToOpponent: boolean;
  readonly hasRoundtrip: boolean;
  readonly scoresSupermanaThisTurn: boolean;
  readonly scoresOpponentManaThisTurn: boolean;
  readonly safeSupermanaPickupNow: boolean;
  readonly safeOpponentManaPickupNow: boolean;
  readonly safeSupermanaProgressSteps: number;
  readonly safeOpponentManaProgressSteps: number;
  readonly scorePathBestSteps: number;
  readonly sameTurnScoreWindowValue: number;
  readonly spiritSetupGain: number;
  readonly spiritSameTurnScoreSetupNow: boolean;
  readonly spiritOwnManaSetupNow: boolean;
  readonly supermanaProgress: boolean;
  readonly opponentManaProgress: boolean;
  readonly policyPriority: number;
  readonly classes: MoveClassFlags;
};

export function productionEnabled(config: AutomoveConfig): boolean {
  return config.planner.mode === AUTOMOVE_TURN_ENGINE_MODE.Production;
}

export function hasAwakeSpiritOnBase(
  game: MonsGame,
  perspective: Color,
): boolean {
  const item = game.board.get(
    game.board.base({
      kind: MonKind.Spirit,
      color: perspective,
      cooldown: 0,
    }),
  );
  const mon = item === undefined ? undefined : itemMon(item);
  return (
    mon?.kind === MonKind.Spirit &&
    mon.color === perspective &&
    !isMonFainted(mon)
  );
}

export function shouldPreferSpiritDevelopment(
  game: MonsGame,
  perspective: Color,
): boolean {
  return (
    !game.isFirstTurn() &&
    game.playerCanMoveMon() &&
    hasAwakeSpiritOnBase(game, perspective)
  );
}

export function hasConcreteScoreSurface(root: RootObservation): boolean {
  return (
    root.winsImmediately ||
    root.scoresSupermanaThisTurn ||
    root.scoresOpponentManaThisTurn ||
    root.safeSupermanaPickupNow ||
    root.safeOpponentManaPickupNow
  );
}

export function hasProgressSurface(root: RootObservation): boolean {
  return (
    root.safeSupermanaPickupNow ||
    root.safeOpponentManaPickupNow ||
    root.supermanaProgress ||
    root.opponentManaProgress
  );
}

export function rootIsUnsafe(root: RootObservation): boolean {
  return (
    root.manaHandoffToOpponent ||
    (root.ownDrainerVulnerable &&
      !root.classes.drainerSafetyRecover &&
      !root.attacksOpponentDrainer &&
      !hasConcreteScoreSurface(root))
  );
}

export function isPlainSpiritDevelopmentRoot(root: RootObservation): boolean {
  return (
    root.spiritDevelopment &&
    !root.spiritSameTurnScoreSetupNow &&
    !root.spiritOwnManaSetupNow
  );
}
