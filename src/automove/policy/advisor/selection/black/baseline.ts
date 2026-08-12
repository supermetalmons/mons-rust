import { Color } from "../../../../../api/types.js";
import { MonsGame } from "../../../../../engine/game/mons-game.js";
import { exactOpportunityContext } from "../../../../exact/turn-opportunity.js";
import type { AutomoveExecutionContext } from "../../../../core/execution-context.js";
import { pickRootWithReplyRiskGuard } from "../../../reply-risk/selection.js";
import { rootReplyRiskSnapshot } from "../../../reply-risk/snapshot.js";
import { rootFamily as advisorRootFamily } from "../../../../root/family.js";
import { pickBaselineRootIndexFromCandidateIndices } from "../../../../root/selection.js";
import {
  saturatingScoreAdd,
  saturatingScoreSubtract,
} from "../../../../core/score-math.js";
import type { EvaluatedRoot } from "../../../../root/types.js";
import {
  AUTOMOVE_TURN_ENGINE_MODE,
  hasProgressSurface,
  isPlainSpiritDevelopmentRoot,
  productionEnabled,
  rootIsUnsafe as advisorRootIsUnsafe,
} from "../../../../config/types.js";
import type { AutomoveConfig } from "../../../../config/types.js";
import { TurnPlanFamily } from "../../../../turn/model.js";
import {
  bestOverrideIndex,
  compareRootRankThenRanked,
  withPlannerMode,
} from "../../support.js";
import { inputChainsShareFirstInput as sameFirstInput } from "../../../../../engine/model/domain.js";

function rootIsNonTactical(root: EvaluatedRoot): boolean {
  return (
    !root.winsImmediately &&
    !root.attacksOpponentDrainer &&
    !root.scoresSupermanaThisTurn &&
    !root.scoresOpponentManaThisTurn &&
    !root.safeSupermanaPickupNow &&
    !root.safeOpponentManaPickupNow &&
    !root.manaHandoffToOpponent &&
    !root.hasRoundtrip
  );
}

function blackTurnFourWindowManaSiblingOverride(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  roots: readonly EvaluatedRoot[],
  selectionIndices: readonly number[],
  approvedIndex: number,
  config: AutomoveConfig,
): number | undefined {
  if (
    !productionEnabled(config) ||
    game.activeColor !== Color.Black ||
    game.turnNumber !== 4 ||
    game.monsMovesCount !== 0 ||
    game.playerCanUseAction() ||
    !game.playerCanMoveMana()
  ) {
    return undefined;
  }
  const exact = exactOpportunityContext(execution, game, game.activeColor);
  if (
    exact.delta.sameTurnScoreWindowValue > 1 ||
    exact.delta.opponentWindowDenyGain > 1 ||
    (exact.delta.sameTurnScoreWindowValue === 0 &&
      exact.delta.opponentWindowDenyGain === 0)
  ) {
    return undefined;
  }
  const approved = roots[approvedIndex];
  if (
    approved === undefined ||
    advisorRootFamily(approved) !== TurnPlanFamily.ManaTempo ||
    approved.rootRank !== 0 ||
    !approved.ownDrainerVulnerable ||
    !rootIsNonTactical(approved)
  ) {
    return undefined;
  }
  return bestOverrideIndex(
    roots,
    selectionIndices,
    (challenger, index) =>
      index !== approvedIndex &&
      advisorRootFamily(challenger) === TurnPlanFamily.ManaTempo &&
      challenger.rootRank > approved.rootRank &&
      challenger.rootRank <= approved.rootRank + 2 &&
      challenger.sameTurnScoreWindowValue === approved.sameTurnScoreWindowValue &&
      challenger.ownDrainerVulnerable === approved.ownDrainerVulnerable &&
      challenger.ownDrainerWalkVulnerable === approved.ownDrainerWalkVulnerable &&
      rootIsNonTactical(challenger) &&
      challenger.score >= saturatingScoreSubtract(approved.score, 96) &&
      challenger.scorePathBestSteps > approved.scorePathBestSteps,
    (left, right) => compareRootRankThenRanked(roots, left, right),
  );
}

function blackBaselineAlignmentOverride(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  roots: readonly EvaluatedRoot[],
  selectionIndices: readonly number[],
  approvedIndex: number,
  baselineIndex: number,
  config: AutomoveConfig,
): number | undefined {
  if (
    !productionEnabled(config) ||
    game.activeColor !== Color.Black ||
    approvedIndex === baselineIndex ||
    !selectionIndices.includes(baselineIndex)
  ) {
    return undefined;
  }
  const approved = roots[approvedIndex];
  const baseline = roots[baselineIndex];
  if (approved === undefined || baseline === undefined) return undefined;
  const approvedFamily = advisorRootFamily(approved);
  const baselineFamily = advisorRootFamily(baseline);
  const exact = exactOpportunityContext(execution, game, game.activeColor);
  const approvedNonTactical = rootIsNonTactical(approved);
  const baselineNonTactical = rootIsNonTactical(baseline);
  const weakBlackPlainSpiritBaselineMana =
    game.turnNumber >= 6 &&
    game.monsMovesCount >= 2 &&
    game.playerCanUseAction() &&
    game.playerCanMoveMana() &&
    exact.delta.sameTurnScoreWindowValue === 0 &&
    exact.delta.opponentWindowDenyGain === 0 &&
    !exact.delta.drainerAttackAvailable &&
    approvedFamily === TurnPlanFamily.SpiritImpact &&
    baselineFamily === TurnPlanFamily.ManaTempo &&
    isPlainSpiritDevelopmentRoot(approved) &&
    !hasProgressSurface(approved) &&
    approvedNonTactical &&
    baselineNonTactical &&
    approved.ownDrainerVulnerable &&
    baseline.ownDrainerVulnerable &&
    baseline.score >= approved.score;
  if (weakBlackPlainSpiritBaselineMana) return baselineIndex;
  const earlyBlackSetupBranchBaselineSpirit =
    game.turnNumber === 2 &&
    game.monsMovesCount >= 2 &&
    game.playerCanUseAction() &&
    game.playerCanMoveMana() &&
    exact.delta.sameTurnScoreWindowValue === 0 &&
    exact.delta.opponentWindowDenyGain === 0 &&
    !exact.delta.drainerAttackAvailable &&
    approvedFamily === TurnPlanFamily.SpiritImpact &&
    baselineFamily === TurnPlanFamily.SpiritImpact &&
    approvedNonTactical &&
    baselineNonTactical &&
    approved.spiritOwnManaSetupNow &&
    baseline.spiritOwnManaSetupNow &&
    !approved.spiritSameTurnScoreSetupNow &&
    !baseline.spiritSameTurnScoreSetupNow &&
    !advisorRootIsUnsafe(approved) &&
    !advisorRootIsUnsafe(baseline) &&
    sameFirstInput(approved.inputs, baseline.inputs) &&
    approved.spiritSetupGain === baseline.spiritSetupGain &&
    approved.safeSupermanaProgressSteps === baseline.safeSupermanaProgressSteps &&
    approved.safeOpponentManaProgressSteps === baseline.safeOpponentManaProgressSteps &&
    approved.score === baseline.score &&
    baseline.rootRank < approved.rootRank &&
    Math.abs(approved.rootRank - baseline.rootRank) <= 2;
  if (earlyBlackSetupBranchBaselineSpirit) return baselineIndex;
  const weakBlackNoActionWindowBaselineMana =
    game.turnNumber >= 4 &&
    game.monsMovesCount === 0 &&
    !game.playerCanUseAction() &&
    game.playerCanMoveMana() &&
    exact.delta.sameTurnScoreWindowValue <= 1 &&
    exact.delta.opponentWindowDenyGain <= 1 &&
    (exact.delta.sameTurnScoreWindowValue > 0 ||
      exact.delta.opponentWindowDenyGain > 0) &&
    approvedFamily === TurnPlanFamily.ManaTempo &&
    baselineFamily === TurnPlanFamily.ManaTempo &&
    approvedNonTactical &&
    baselineNonTactical &&
    approved.sameTurnScoreWindowValue === baseline.sameTurnScoreWindowValue &&
    approved.ownDrainerVulnerable === baseline.ownDrainerVulnerable &&
    approved.rootRank < baseline.rootRank &&
    approved.score >= baseline.score &&
    baseline.scorePathBestSteps > approved.scorePathBestSteps;
  return weakBlackNoActionWindowBaselineMana ? baselineIndex : undefined;
}

function blackTurnStartGuardedBaselineManaOverride(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  roots: readonly EvaluatedRoot[],
  candidateIndices: readonly number[],
  approvedIndex: number,
  config: AutomoveConfig,
): number | undefined {
  if (
    !productionEnabled(config) ||
    game.activeColor !== Color.Black ||
    game.turnNumber !== 6 ||
    game.monsMovesCount !== 0 ||
    !game.playerCanUseAction() ||
    !game.playerCanMoveMana() ||
    candidateIndices.length === 0
  ) {
    return undefined;
  }
  const approved = roots[approvedIndex];
  if (approved === undefined) return undefined;
  const exact = exactOpportunityContext(execution, game, game.activeColor);
  if (
    exact.delta.sameTurnScoreWindowValue !== 0 ||
    exact.delta.opponentWindowDenyGain !== 0 ||
    exact.delta.drainerAttackAvailable ||
    advisorRootFamily(approved) !== TurnPlanFamily.SpiritImpact ||
    !isPlainSpiritDevelopmentRoot(approved) ||
    hasProgressSurface(approved) ||
    !rootIsNonTactical(approved) ||
    !approved.ownDrainerVulnerable
  ) {
    return undefined;
  }
  const baselineConfig = withPlannerMode(config, AUTOMOVE_TURN_ENGINE_MODE.Baseline);
  let baselineIndex: number | undefined;
  if (baselineConfig.replyRisk.enabled) {
    baselineIndex = pickRootWithReplyRiskGuard(
      execution,
      game,
      roots,
      candidateIndices,
      game.activeColor,
      baselineConfig,
    );
  }
  baselineIndex ??= pickBaselineRootIndexFromCandidateIndices(
    game,
    roots,
    candidateIndices,
    game.activeColor,
    baselineConfig,
  );
  if (
    baselineIndex === undefined ||
    baselineIndex === approvedIndex ||
    !candidateIndices.includes(baselineIndex)
  ) {
    return undefined;
  }
  const baseline = roots[baselineIndex];
  if (
    baseline === undefined ||
    advisorRootFamily(baseline) !== TurnPlanFamily.ManaTempo ||
    !rootIsNonTactical(baseline) ||
    !baseline.ownDrainerVulnerable ||
    approved.ownDrainerWalkVulnerable !== baseline.ownDrainerWalkVulnerable ||
    approved.safeSupermanaProgressSteps !== baseline.safeSupermanaProgressSteps ||
    approved.safeOpponentManaProgressSteps !== baseline.safeOpponentManaProgressSteps ||
    approved.scorePathBestSteps !== baseline.scorePathBestSteps ||
    baseline.score < saturatingScoreAdd(approved.score, 256)
  ) {
    return undefined;
  }
  const replyLimit = Math.min(Math.max(config.replyRisk.replyLimit, 1), 24);
  const approvedSnapshot = rootReplyRiskSnapshot(
    execution,
    approved.game,
    game.activeColor,
    config,
    replyLimit,
  );
  const baselineSnapshot = rootReplyRiskSnapshot(
    execution,
    baseline.game,
    game.activeColor,
    config,
    replyLimit,
  );
  if (
    approvedSnapshot.allowsImmediateOpponentWin ||
    baselineSnapshot.allowsImmediateOpponentWin ||
    approvedSnapshot.opponentReachesMatchPoint ||
    baselineSnapshot.opponentReachesMatchPoint ||
    baselineSnapshot.worstReplyScore <
      saturatingScoreAdd(approvedSnapshot.worstReplyScore, 32)
  ) {
    return undefined;
  }
  return baselineIndex;
}

function whiteTurnThreeBaselineAlignmentOverride(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  roots: readonly EvaluatedRoot[],
  selectionIndices: readonly number[],
  approvedIndex: number,
  baselineIndex: number,
  config: AutomoveConfig,
): number | undefined {
  if (
    !productionEnabled(config) ||
    game.activeColor !== Color.White ||
    game.turnNumber !== 3 ||
    approvedIndex === baselineIndex
  ) {
    return undefined;
  }
  const approved = roots[approvedIndex];
  const baseline = roots[baselineIndex];
  if (approved === undefined || baseline === undefined) return undefined;
  const exact = exactOpportunityContext(execution, game, game.activeColor);
  const aligned =
    game.monsMovesCount >= 3 &&
    !game.playerCanUseAction() &&
    game.playerCanMoveMana() &&
    selectionIndices.includes(baselineIndex) &&
    exact.delta.sameTurnScoreWindowValue === 0 &&
    exact.delta.opponentWindowDenyGain === 0 &&
    !exact.delta.drainerAttackAvailable &&
    advisorRootFamily(approved) === TurnPlanFamily.ManaTempo &&
    advisorRootFamily(baseline) === TurnPlanFamily.ManaTempo &&
    rootIsNonTactical(approved) &&
    rootIsNonTactical(baseline) &&
    !approved.spiritDevelopment &&
    !baseline.spiritDevelopment &&
    !approved.spiritSameTurnScoreSetupNow &&
    !baseline.spiritSameTurnScoreSetupNow &&
    !approved.spiritOwnManaSetupNow &&
    !baseline.spiritOwnManaSetupNow &&
    !approved.ownDrainerVulnerable &&
    !baseline.ownDrainerVulnerable &&
    !approved.ownDrainerWalkVulnerable &&
    !baseline.ownDrainerWalkVulnerable &&
    approved.safeSupermanaProgressSteps === baseline.safeSupermanaProgressSteps &&
    approved.safeOpponentManaProgressSteps === baseline.safeOpponentManaProgressSteps &&
    approved.scorePathBestSteps === baseline.scorePathBestSteps &&
    approved.spiritSetupGain === baseline.spiritSetupGain &&
    baseline.score >= saturatingScoreAdd(approved.score, 16) &&
    baseline.rootRank >= approved.rootRank + 2 &&
    baseline.rootRank <= approved.rootRank + 4;
  return aligned ? baselineIndex : undefined;
}

function whiteTurnThreeAttackBridgeEscape(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  roots: readonly EvaluatedRoot[],
  selectionIndices: readonly number[],
  approvedIndex: number,
  config: AutomoveConfig,
): number | undefined {
  if (
    !productionEnabled(config) ||
    game.activeColor !== Color.White ||
    game.turnNumber !== 3 ||
    selectionIndices.length !== 1 ||
    selectionIndices[0] !== approvedIndex ||
    roots.length === 0
  ) {
    return undefined;
  }
  const approved = roots[approvedIndex];
  if (approved === undefined) return undefined;
  const exact = exactOpportunityContext(execution, game, game.activeColor);
  if (
    !exact.delta.drainerAttackAvailable ||
    exact.delta.sameTurnScoreWindowValue !== 0 ||
    exact.delta.opponentWindowDenyGain !== 0 ||
    game.monsMovesCount < 2 ||
    !game.playerCanUseAction() ||
    !game.playerCanMoveMana() ||
    advisorRootFamily(approved) !== TurnPlanFamily.SpiritImpact ||
    !approved.spiritDevelopment ||
    approved.spiritSameTurnScoreSetupNow ||
    !approved.spiritOwnManaSetupNow ||
    !rootIsNonTactical(approved) ||
    advisorRootIsUnsafe(approved) ||
    approved.ownDrainerVulnerable ||
    approved.ownDrainerWalkVulnerable
  ) {
    return undefined;
  }
  const allIndices = roots.map((_root, index) => index);
  return bestOverrideIndex(
    roots,
    allIndices,
    (root, index) =>
      index !== approvedIndex &&
      !selectionIndices.includes(index) &&
      advisorRootFamily(root) === TurnPlanFamily.ManaTempo &&
      !root.spiritDevelopment &&
      !root.spiritSameTurnScoreSetupNow &&
      !root.spiritOwnManaSetupNow &&
      rootIsNonTactical(root) &&
      !advisorRootIsUnsafe(root) &&
      !root.ownDrainerVulnerable &&
      !root.ownDrainerWalkVulnerable &&
      root.safeSupermanaProgressSteps === approved.safeSupermanaProgressSteps &&
      root.safeOpponentManaProgressSteps === approved.safeOpponentManaProgressSteps &&
      root.scorePathBestSteps === approved.scorePathBestSteps &&
      root.inputs.length >= 3 &&
      root.rootRank <= approved.rootRank + 2,
    (left, right) => compareRootRankThenRanked(roots, left, right),
  );
}

export {
  blackBaselineAlignmentOverride,
  blackTurnFourWindowManaSiblingOverride,
  blackTurnStartGuardedBaselineManaOverride,
  rootIsNonTactical,
  whiteTurnThreeAttackBridgeEscape,
  whiteTurnThreeBaselineAlignmentOverride,
};
