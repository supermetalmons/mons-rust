import { Color } from "../../../api/types.js";
import { saturatingScoreAdd, saturatingScoreSubtract } from "../../core/score-math.js";
import { hasConcreteScoreSurface as rootHasConcreteScoreSurface } from "../../config/types.js";
import { TurnPlanFamily } from "../../turn/model.js";
import { utilitySupportsPrimaryAxesEvalTolerance } from "../../turn/ordering.js";
import {
  ProductionHeadGuardId,
  type ProductionHeadGuardId as ProductionHeadGuardReason,
} from "./head-guard-order.js";
import type {
  TurnEngineHeadAcceptanceContext,
  TurnEngineHeadOrderedFacts,
} from "./head-types.js";
import { inputChainsShareFirstInput as firstInputsEqual } from "../../../engine/model/domain.js";
import { isProductionModeNonConcreteManaWindowRoot } from "./shared.js";

export function firstRejectedManaAndRecoveryHeadGuard(
  context: TurnEngineHeadAcceptanceContext,
  facts: TurnEngineHeadOrderedFacts,
): ProductionHeadGuardReason | undefined {
  const {
    game,
    plan,
    candidate,
    selected,
    macroMode,
    candidateUnsafe,
    selectedUnsafe,
    candidateProgress,
    selectedProgress,
    exactContext,
    sameTurnWindowBetter,
    drainerAttackBetter,
    scoresNowBetter,
    selectedSpiritPhase,
    candidateFamily,
    selectedFamily,
  } = context;
  const { pickupUpgrade, selectedUtility } = facts;
  const earlyBlackSafeManaBlocksWeakerMana = (): boolean =>
    macroMode &&
    game.activeColor === Color.Black &&
    game.turnNumber <= 4 &&
    game.monsMovesCount >= 1 &&
    game.playerCanUseAction() &&
    game.playerCanMoveMana() &&
    plan.headFamily === TurnPlanFamily.ManaTempo &&
    plan.goalFamily === TurnPlanFamily.SpiritImpact &&
    candidateFamily === TurnPlanFamily.ManaTempo &&
    selectedFamily === TurnPlanFamily.ManaTempo &&
    !candidateUnsafe &&
    !selectedUnsafe &&
    !candidate.spiritDevelopment &&
    !selected.spiritDevelopment &&
    !candidate.spiritSameTurnScoreSetupNow &&
    !selected.spiritSameTurnScoreSetupNow &&
    !candidate.spiritOwnManaSetupNow &&
    !selected.spiritOwnManaSetupNow &&
    !rootHasConcreteScoreSurface(candidate) &&
    !rootHasConcreteScoreSurface(selected) &&
    candidate.sameTurnScoreWindowValue === selected.sameTurnScoreWindowValue &&
    !candidateProgress &&
    !selectedProgress &&
    candidate.safeSupermanaProgressSteps === selected.safeSupermanaProgressSteps &&
    candidate.safeOpponentManaProgressSteps ===
      selected.safeOpponentManaProgressSteps &&
    candidate.ownDrainerVulnerable === selected.ownDrainerVulnerable &&
    candidate.ownDrainerWalkVulnerable === selected.ownDrainerWalkVulnerable &&
    candidate.manaHandoffToOpponent === selected.manaHandoffToOpponent &&
    candidate.hasRoundtrip === selected.hasRoundtrip &&
    selected.score > saturatingScoreAdd(candidate.score, 128) &&
    !scoresNowBetter &&
    !drainerAttackBetter &&
    !sameTurnWindowBetter &&
    !pickupUpgrade;
  const blackQuietManaBlocksLowerScoredMana = (): boolean =>
    macroMode &&
    game.activeColor === Color.Black &&
    game.turnNumber <= 6 &&
    game.monsMovesCount >= 1 &&
    game.playerCanMoveMana() &&
    (plan.headFamily === TurnPlanFamily.ManaTempo ||
      plan.headFamily === TurnPlanFamily.SafeSupermanaProgress ||
      plan.headFamily === TurnPlanFamily.SafeOpponentManaProgress) &&
    candidateFamily === TurnPlanFamily.ManaTempo &&
    selectedFamily === TurnPlanFamily.ManaTempo &&
    candidateUnsafe === selectedUnsafe &&
    !candidate.spiritDevelopment &&
    !selected.spiritDevelopment &&
    !candidate.spiritSameTurnScoreSetupNow &&
    !selected.spiritSameTurnScoreSetupNow &&
    !candidate.spiritOwnManaSetupNow &&
    !selected.spiritOwnManaSetupNow &&
    !rootHasConcreteScoreSurface(candidate) &&
    !rootHasConcreteScoreSurface(selected) &&
    candidate.sameTurnScoreWindowValue === 0 &&
    selected.sameTurnScoreWindowValue === 0 &&
    candidate.ownDrainerVulnerable === selected.ownDrainerVulnerable &&
    candidate.ownDrainerWalkVulnerable === selected.ownDrainerWalkVulnerable &&
    candidate.manaHandoffToOpponent === selected.manaHandoffToOpponent &&
    candidate.hasRoundtrip === selected.hasRoundtrip &&
    selected.score > saturatingScoreAdd(candidate.score, 48) &&
    selected.safeSupermanaProgressSteps <= candidate.safeSupermanaProgressSteps + 1 &&
    selected.safeOpponentManaProgressSteps <=
      candidate.safeOpponentManaProgressSteps + 1 &&
    !scoresNowBetter &&
    !drainerAttackBetter &&
    !sameTurnWindowBetter &&
    !pickupUpgrade;
  const whiteSameWindowManaBlocksLowerScoredMana = (): boolean =>
    macroMode &&
    game.activeColor === Color.White &&
    game.turnNumber === 5 &&
    game.monsMovesCount === 0 &&
    game.playerCanMoveMana() &&
    (plan.headFamily === TurnPlanFamily.SafeSupermanaProgress ||
      plan.headFamily === TurnPlanFamily.SafeOpponentManaProgress) &&
    plan.goalFamily === TurnPlanFamily.ImmediateScore &&
    candidateFamily === TurnPlanFamily.ManaTempo &&
    selectedFamily === TurnPlanFamily.ManaTempo &&
    candidateUnsafe === selectedUnsafe &&
    !candidate.spiritDevelopment &&
    !selected.spiritDevelopment &&
    !candidate.spiritSameTurnScoreSetupNow &&
    !selected.spiritSameTurnScoreSetupNow &&
    !candidate.spiritOwnManaSetupNow &&
    !selected.spiritOwnManaSetupNow &&
    !rootHasConcreteScoreSurface(candidate) &&
    !rootHasConcreteScoreSurface(selected) &&
    candidate.sameTurnScoreWindowValue > 0 &&
    candidate.sameTurnScoreWindowValue === selected.sameTurnScoreWindowValue &&
    !candidateProgress &&
    !selectedProgress &&
    candidate.safeSupermanaProgressSteps === selected.safeSupermanaProgressSteps &&
    candidate.safeOpponentManaProgressSteps ===
      selected.safeOpponentManaProgressSteps &&
    firstInputsEqual(candidate.inputs, selected.inputs) &&
    candidate.ownDrainerVulnerable === selected.ownDrainerVulnerable &&
    candidate.ownDrainerWalkVulnerable === selected.ownDrainerWalkVulnerable &&
    candidate.manaHandoffToOpponent === selected.manaHandoffToOpponent &&
    candidate.hasRoundtrip === selected.hasRoundtrip &&
    selected.score > candidate.score &&
    !scoresNowBetter &&
    !drainerAttackBetter &&
    !sameTurnWindowBetter &&
    !pickupUpgrade;
  const whiteMidTurnManaBlocksLowerScoredWindowMana = (): boolean =>
    macroMode &&
    game.activeColor === Color.White &&
    game.turnNumber === 5 &&
    game.monsMovesCount >= 1 &&
    game.playerCanMoveMana() &&
    (plan.headFamily === TurnPlanFamily.ImmediateScore ||
      plan.headFamily === TurnPlanFamily.SafeSupermanaProgress ||
      plan.headFamily === TurnPlanFamily.SafeOpponentManaProgress) &&
    plan.goalFamily === TurnPlanFamily.ImmediateScore &&
    candidateFamily === TurnPlanFamily.ManaTempo &&
    selectedFamily === TurnPlanFamily.ManaTempo &&
    candidateUnsafe === selectedUnsafe &&
    !candidate.spiritDevelopment &&
    !selected.spiritDevelopment &&
    !candidate.spiritSameTurnScoreSetupNow &&
    !selected.spiritSameTurnScoreSetupNow &&
    !candidate.spiritOwnManaSetupNow &&
    !selected.spiritOwnManaSetupNow &&
    !rootHasConcreteScoreSurface(candidate) &&
    !rootHasConcreteScoreSurface(selected) &&
    candidate.sameTurnScoreWindowValue > 0 &&
    candidate.sameTurnScoreWindowValue === selected.sameTurnScoreWindowValue &&
    !candidateProgress &&
    !selectedProgress &&
    candidate.safeSupermanaProgressSteps === selected.safeSupermanaProgressSteps &&
    candidate.safeOpponentManaProgressSteps ===
      selected.safeOpponentManaProgressSteps &&
    selected.scorePathBestSteps >= candidate.scorePathBestSteps &&
    candidate.ownDrainerVulnerable === selected.ownDrainerVulnerable &&
    candidate.ownDrainerWalkVulnerable === selected.ownDrainerWalkVulnerable &&
    candidate.manaHandoffToOpponent === selected.manaHandoffToOpponent &&
    candidate.hasRoundtrip === selected.hasRoundtrip &&
    selected.score > candidate.score &&
    !scoresNowBetter &&
    !drainerAttackBetter &&
    !sameTurnWindowBetter &&
    !pickupUpgrade;
  const whiteMidTurnSpiritSetupBlocksWindowMana = (): boolean =>
    macroMode &&
    game.activeColor === Color.White &&
    game.turnNumber === 5 &&
    game.monsMovesCount >= 1 &&
    game.playerCanUseAction() &&
    game.playerCanMoveMana() &&
    (plan.headFamily === TurnPlanFamily.SafeSupermanaProgress ||
      plan.headFamily === TurnPlanFamily.SafeOpponentManaProgress) &&
    plan.goalFamily === TurnPlanFamily.ImmediateScore &&
    candidateFamily === TurnPlanFamily.ManaTempo &&
    selectedFamily === TurnPlanFamily.SpiritImpact &&
    candidateUnsafe === selectedUnsafe &&
    !candidate.spiritDevelopment &&
    !candidate.spiritSameTurnScoreSetupNow &&
    !candidate.spiritOwnManaSetupNow &&
    selected.spiritDevelopment &&
    selected.spiritSameTurnScoreSetupNow &&
    selected.spiritOwnManaSetupNow &&
    selected.spiritSetupGain >= saturatingScoreAdd(candidate.spiritSetupGain, 64) &&
    !rootHasConcreteScoreSurface(candidate) &&
    !rootHasConcreteScoreSurface(selected) &&
    candidate.sameTurnScoreWindowValue > 0 &&
    candidate.sameTurnScoreWindowValue === selected.sameTurnScoreWindowValue &&
    !candidateProgress &&
    !selectedProgress &&
    candidate.safeSupermanaProgressSteps === selected.safeSupermanaProgressSteps &&
    candidate.safeOpponentManaProgressSteps ===
      selected.safeOpponentManaProgressSteps &&
    selected.scorePathBestSteps > candidate.scorePathBestSteps &&
    candidate.ownDrainerVulnerable === selected.ownDrainerVulnerable &&
    candidate.ownDrainerWalkVulnerable === selected.ownDrainerWalkVulnerable &&
    candidate.manaHandoffToOpponent === selected.manaHandoffToOpponent &&
    candidate.hasRoundtrip === selected.hasRoundtrip &&
    candidate.score <= saturatingScoreAdd(selected.score, 64) &&
    !scoresNowBetter &&
    !drainerAttackBetter &&
    !sameTurnWindowBetter &&
    !pickupUpgrade;
  const whiteTurnStartSpiritSetupBlocksWindowMana = (): boolean =>
    macroMode &&
    game.activeColor === Color.White &&
    game.turnNumber === 5 &&
    game.monsMovesCount === 0 &&
    game.playerCanUseAction() &&
    game.playerCanMoveMana() &&
    exactContext.delta.sameTurnScoreWindowValue <= 1 &&
    exactContext.delta.opponentWindowDenyGain <= 1 &&
    !exactContext.delta.drainerAttackAvailable &&
    (exactContext.delta.sameTurnScoreWindowValue > 0 ||
      exactContext.delta.opponentWindowDenyGain > 0) &&
    (plan.headFamily === TurnPlanFamily.SafeSupermanaProgress ||
      plan.headFamily === TurnPlanFamily.SafeOpponentManaProgress) &&
    plan.goalFamily === TurnPlanFamily.ImmediateScore &&
    isProductionModeNonConcreteManaWindowRoot(candidate) &&
    selectedFamily === TurnPlanFamily.SpiritImpact &&
    candidateUnsafe &&
    !selectedUnsafe &&
    !candidate.spiritDevelopment &&
    !candidate.spiritSameTurnScoreSetupNow &&
    !candidate.spiritOwnManaSetupNow &&
    selected.spiritDevelopment &&
    selected.spiritSameTurnScoreSetupNow &&
    !selected.spiritOwnManaSetupNow &&
    selected.spiritSetupGain >= saturatingScoreAdd(candidate.spiritSetupGain, 96) &&
    !rootHasConcreteScoreSurface(candidate) &&
    !rootHasConcreteScoreSurface(selected) &&
    candidate.sameTurnScoreWindowValue === selected.sameTurnScoreWindowValue &&
    !candidateProgress &&
    !selectedProgress &&
    candidate.safeSupermanaProgressSteps === selected.safeSupermanaProgressSteps &&
    candidate.safeOpponentManaProgressSteps ===
      selected.safeOpponentManaProgressSteps &&
    selected.scorePathBestSteps > candidate.scorePathBestSteps &&
    candidate.ownDrainerVulnerable &&
    !selected.ownDrainerVulnerable &&
    candidate.ownDrainerWalkVulnerable === selected.ownDrainerWalkVulnerable &&
    candidate.manaHandoffToOpponent === selected.manaHandoffToOpponent &&
    candidate.hasRoundtrip === selected.hasRoundtrip &&
    !scoresNowBetter &&
    !drainerAttackBetter &&
    !sameTurnWindowBetter &&
    !pickupUpgrade;
  const whiteSafeManaBlocksDeferredRecoveryProgress = (): boolean =>
    macroMode &&
    game.activeColor === Color.White &&
    game.turnNumber === 5 &&
    game.monsMovesCount >= 1 &&
    !game.playerCanUseAction() &&
    game.playerCanMoveMana() &&
    (plan.headFamily === TurnPlanFamily.SafeSupermanaProgress ||
      plan.headFamily === TurnPlanFamily.SafeOpponentManaProgress) &&
    plan.goalFamily === TurnPlanFamily.DrainerSafetyRecovery &&
    candidateFamily === TurnPlanFamily.ManaTempo &&
    selectedFamily === TurnPlanFamily.ManaTempo &&
    !candidateUnsafe &&
    !selectedUnsafe &&
    !candidateProgress &&
    !selectedProgress &&
    !candidate.spiritDevelopment &&
    !selected.spiritDevelopment &&
    !candidate.spiritSameTurnScoreSetupNow &&
    !selected.spiritSameTurnScoreSetupNow &&
    !candidate.spiritOwnManaSetupNow &&
    !selected.spiritOwnManaSetupNow &&
    !rootHasConcreteScoreSurface(candidate) &&
    !rootHasConcreteScoreSurface(selected) &&
    candidate.sameTurnScoreWindowValue === 0 &&
    selected.sameTurnScoreWindowValue === 0 &&
    candidate.safeSupermanaProgressSteps <= selected.safeSupermanaProgressSteps &&
    candidate.safeOpponentManaProgressSteps <= selected.safeOpponentManaProgressSteps &&
    candidate.scorePathBestSteps === selected.scorePathBestSteps &&
    candidate.spiritSetupGain <= saturatingScoreAdd(selected.spiritSetupGain, 16) &&
    !candidate.ownDrainerVulnerable &&
    !selected.ownDrainerVulnerable &&
    !candidate.ownDrainerWalkVulnerable &&
    !selected.ownDrainerWalkVulnerable &&
    !candidate.manaHandoffToOpponent &&
    !selected.manaHandoffToOpponent &&
    !candidate.hasRoundtrip &&
    !selected.hasRoundtrip &&
    candidate.rootRank >= selected.rootRank + 8 &&
    saturatingScoreSubtract(candidate.score, selected.score) <= 128 &&
    !scoresNowBetter &&
    !drainerAttackBetter &&
    !sameTurnWindowBetter &&
    !pickupUpgrade &&
    !utilitySupportsPrimaryAxesEvalTolerance(plan.headUtility, selectedUtility, 192);
  const blackLateSafeProgressBlocksQuietMana = (): boolean =>
    macroMode &&
    game.activeColor === Color.Black &&
    game.turnNumber === 6 &&
    game.monsMovesCount === 0 &&
    !game.playerCanUseAction() &&
    game.playerCanMoveMana() &&
    exactContext.delta.sameTurnScoreWindowValue <= 1 &&
    exactContext.delta.opponentWindowDenyGain <= 1 &&
    !exactContext.delta.drainerAttackAvailable &&
    (exactContext.delta.sameTurnScoreWindowValue > 0 ||
      exactContext.delta.opponentWindowDenyGain > 0) &&
    plan.headFamily === TurnPlanFamily.ManaTempo &&
    plan.goalFamily === TurnPlanFamily.ImmediateScore &&
    candidateFamily === TurnPlanFamily.ManaTempo &&
    (selectedFamily === TurnPlanFamily.SafeSupermanaProgress ||
      selectedFamily === TurnPlanFamily.SafeOpponentManaProgress) &&
    !candidateUnsafe &&
    !candidate.spiritDevelopment &&
    !selected.spiritDevelopment &&
    !candidate.spiritSameTurnScoreSetupNow &&
    !selected.spiritSameTurnScoreSetupNow &&
    !candidate.spiritOwnManaSetupNow &&
    !selected.spiritOwnManaSetupNow &&
    !rootHasConcreteScoreSurface(candidate) &&
    !rootHasConcreteScoreSurface(selected) &&
    candidate.sameTurnScoreWindowValue === 0 &&
    selected.sameTurnScoreWindowValue === 0 &&
    !candidateProgress &&
    selectedProgress &&
    selected.safeSupermanaProgressSteps < candidate.safeSupermanaProgressSteps &&
    selected.safeOpponentManaProgressSteps < candidate.safeOpponentManaProgressSteps &&
    selected.scorePathBestSteps === candidate.scorePathBestSteps &&
    selected.ownDrainerVulnerable &&
    !candidate.ownDrainerVulnerable &&
    selected.ownDrainerWalkVulnerable === candidate.ownDrainerWalkVulnerable &&
    selected.manaHandoffToOpponent === candidate.manaHandoffToOpponent &&
    selected.hasRoundtrip === candidate.hasRoundtrip &&
    !scoresNowBetter &&
    !drainerAttackBetter &&
    !sameTurnWindowBetter &&
    !pickupUpgrade;
  const blackRecoveryRootBlocksNonConcreteWindow = (): boolean =>
    macroMode &&
    game.activeColor === Color.Black &&
    game.turnNumber === 4 &&
    game.monsMovesCount >= 1 &&
    !game.playerCanUseAction() &&
    game.playerCanMoveMana() &&
    exactContext.delta.sameTurnScoreWindowValue <= 1 &&
    exactContext.delta.opponentWindowDenyGain <= 1 &&
    !exactContext.delta.drainerAttackAvailable &&
    exactContext.delta.drainerSafety < 0 &&
    (exactContext.delta.sameTurnScoreWindowValue > 0 ||
      exactContext.delta.opponentWindowDenyGain > 0) &&
    plan.headFamily === TurnPlanFamily.ImmediateScore &&
    plan.goalFamily === TurnPlanFamily.ImmediateScore &&
    isProductionModeNonConcreteManaWindowRoot(candidate) &&
    selectedFamily === TurnPlanFamily.DrainerSafetyRecovery &&
    selected.classes.drainerSafetyRecover &&
    !selectedUnsafe &&
    !selected.ownDrainerVulnerable &&
    !selected.ownDrainerWalkVulnerable &&
    !rootHasConcreteScoreSurface(selected) &&
    !selected.attacksOpponentDrainer &&
    !selectedSpiritPhase &&
    selected.sameTurnScoreWindowValue === 0 &&
    !selected.manaHandoffToOpponent &&
    !selected.hasRoundtrip &&
    firstInputsEqual(candidate.inputs, selected.inputs) &&
    candidate.ownDrainerVulnerable &&
    !candidate.classes.drainerSafetyRecover &&
    selected.safeSupermanaProgressSteps <= candidate.safeSupermanaProgressSteps &&
    selected.safeOpponentManaProgressSteps <= candidate.safeOpponentManaProgressSteps &&
    selected.scorePathBestSteps > candidate.scorePathBestSteps &&
    !scoresNowBetter &&
    !drainerAttackBetter &&
    !pickupUpgrade;
  const whiteRecoveryRootBlocksNonConcreteWindow = (): boolean =>
    macroMode &&
    game.activeColor === Color.White &&
    game.turnNumber === 3 &&
    game.monsMovesCount === 0 &&
    !game.playerCanUseAction() &&
    game.playerCanMoveMana() &&
    (plan.headFamily === TurnPlanFamily.SafeSupermanaProgress ||
      plan.headFamily === TurnPlanFamily.SafeOpponentManaProgress) &&
    plan.goalFamily === TurnPlanFamily.ImmediateScore &&
    isProductionModeNonConcreteManaWindowRoot(candidate) &&
    selectedFamily === TurnPlanFamily.DrainerSafetyRecovery &&
    selected.classes.drainerSafetyRecover &&
    !selectedUnsafe &&
    !selected.ownDrainerVulnerable &&
    !selected.ownDrainerWalkVulnerable &&
    !rootHasConcreteScoreSurface(selected) &&
    !selected.attacksOpponentDrainer &&
    !selectedSpiritPhase &&
    selected.sameTurnScoreWindowValue === 0 &&
    !selected.manaHandoffToOpponent &&
    !selected.hasRoundtrip &&
    firstInputsEqual(candidate.inputs, selected.inputs) &&
    candidate.ownDrainerVulnerable &&
    !candidate.classes.drainerSafetyRecover &&
    selected.safeSupermanaProgressSteps <= candidate.safeSupermanaProgressSteps &&
    selected.safeOpponentManaProgressSteps <= candidate.safeOpponentManaProgressSteps &&
    selected.scorePathBestSteps > candidate.scorePathBestSteps &&
    !scoresNowBetter &&
    !drainerAttackBetter &&
    !pickupUpgrade;
  const vulnerableWhiteManaHead = (): boolean =>
    macroMode &&
    game.activeColor === Color.White &&
    game.turnNumber === 3 &&
    game.monsMovesCount >= 1 &&
    game.playerCanMoveMana() &&
    (plan.headFamily === TurnPlanFamily.ManaTempo ||
      plan.headFamily === TurnPlanFamily.SafeSupermanaProgress ||
      plan.headFamily === TurnPlanFamily.SafeOpponentManaProgress) &&
    plan.goalFamily === TurnPlanFamily.DrainerSafetyRecovery &&
    candidateFamily === TurnPlanFamily.ManaTempo &&
    selectedFamily === TurnPlanFamily.ManaTempo &&
    candidateUnsafe &&
    !selectedUnsafe &&
    !candidate.spiritDevelopment &&
    !selected.spiritDevelopment &&
    !candidate.spiritSameTurnScoreSetupNow &&
    !selected.spiritSameTurnScoreSetupNow &&
    !candidate.spiritOwnManaSetupNow &&
    !selected.spiritOwnManaSetupNow &&
    !rootHasConcreteScoreSurface(candidate) &&
    !rootHasConcreteScoreSurface(selected) &&
    candidate.sameTurnScoreWindowValue === 0 &&
    selected.sameTurnScoreWindowValue === 0 &&
    candidate.manaHandoffToOpponent === selected.manaHandoffToOpponent &&
    candidate.hasRoundtrip === selected.hasRoundtrip &&
    !scoresNowBetter &&
    !drainerAttackBetter &&
    !sameTurnWindowBetter &&
    !pickupUpgrade;
  if (earlyBlackSafeManaBlocksWeakerMana()) {
    return ProductionHeadGuardId.EarlyBlackSafeManaBlocksWeakerMana;
  }
  if (blackQuietManaBlocksLowerScoredMana()) {
    return ProductionHeadGuardId.BlackQuietManaBlocksLowerScoredMana;
  }
  if (whiteSameWindowManaBlocksLowerScoredMana()) {
    return ProductionHeadGuardId.WhiteSameWindowManaBlocksLowerScoredMana;
  }
  if (whiteMidTurnManaBlocksLowerScoredWindowMana()) {
    return ProductionHeadGuardId.WhiteMidTurnManaBlocksLowerScoredWindowMana;
  }
  if (whiteMidTurnSpiritSetupBlocksWindowMana()) {
    return ProductionHeadGuardId.WhiteMidTurnSpiritSetupBlocksWindowMana;
  }
  if (whiteTurnStartSpiritSetupBlocksWindowMana()) {
    return ProductionHeadGuardId.WhiteTurnStartSpiritSetupBlocksWindowMana;
  }
  if (whiteSafeManaBlocksDeferredRecoveryProgress()) {
    return ProductionHeadGuardId.WhiteSafeManaBlocksDeferredRecoveryProgress;
  }
  if (vulnerableWhiteManaHead()) {
    return ProductionHeadGuardId.VulnerableWhiteManaHead;
  }
  if (blackLateSafeProgressBlocksQuietMana()) {
    return ProductionHeadGuardId.BlackLateSafeProgressBlocksQuietMana;
  }
  if (blackRecoveryRootBlocksNonConcreteWindow()) {
    return ProductionHeadGuardId.BlackRecoveryRootBlocksNonConcreteWindow;
  }
  if (whiteRecoveryRootBlocksNonConcreteWindow()) {
    return ProductionHeadGuardId.WhiteRecoveryRootBlocksNonConcreteWindow;
  }
  return undefined;
}
