import { Color } from "../../engine/domain.js";
import type { AutomoveExecutionContext } from "../execution-context.js";
import { saturatingScoreAdd } from "../score-math.js";
import { isPlainSpiritDevelopmentRoot } from "../selector-types.js";
import { productionIsEarlyWhiteTurnStart } from "../turn-engine-config.js";
import {
  TurnPlanFamily,
  compareUtilityPrimaryAxes,
  utilityImprovesNonScoreOverrideAxes,
  utilityPassesOverrideGuard,
  utilitySupportsPrimaryAxesEvalTolerance,
} from "../turn-engine.js";
import { productionSecondaryAnalysisLive } from "./config.js";
import type {
  TurnEngineHeadAcceptanceContext,
  TurnEngineHeadOrderedDecision,
  TurnEngineHeadOrderedFacts,
} from "./head-types.js";
import { projectedPlanIsSafelyCompleted } from "./plan-support.js";

export function evaluateProjectedHeadGuards(
  execution: AutomoveExecutionContext,
  context: TurnEngineHeadAcceptanceContext,
  facts: TurnEngineHeadOrderedFacts,
): TurnEngineHeadOrderedDecision {
  const {
    game,
    perspective,
    config,
    plan,
    candidateIndex,
    candidate,
    selected,
    macroMode,
    candidateUnsafe,
    selectedUnsafe,
    candidateProgress,
    selectedProgress,
    scoreGap,
    sameTurnWindowBetter,
    drainerAttackBetter,
    scoresNowBetter,
    candidateSpiritTactical,
    progressBetter,
    selectedSpiritPhase,
    selectedFamily,
    whiteSpiritSetupGain,
  } = context;
  const {
    narrowUnsafeBlackManaScore,
    pickupUpgrade,
    nearTieProgress,
    primaryAxesOrder,
    strategicAxesBetter,
    selectedUtility,
  } = facts;
  const projectedSafe =
    macroMode &&
    projectedPlanIsSafelyCompleted(execution, game, perspective, config, plan);
  const projectedReplyNotWorse =
    compareUtilityPrimaryAxes(plan.utility, selectedUtility) >= 0;
  const projectedHeadNotWorse =
    compareUtilityPrimaryAxes(plan.headUtility, selectedUtility) >= 0;
  const narrowWhiteManaOnlyProgressTie =
    macroMode &&
    (plan.headFamily === TurnPlanFamily.SafeSupermanaProgress ||
      plan.headFamily === TurnPlanFamily.SafeOpponentManaProgress) &&
    game.activeColor === Color.White &&
    game.turnNumber >= 5 &&
    game.monsMovesCount <= 1 &&
    !game.playerCanUseAction() &&
    game.playerCanMoveMana() &&
    !candidateUnsafe &&
    !selectedUnsafe &&
    candidateProgress &&
    selectedProgress &&
    !selectedSpiritPhase &&
    nearTieProgress &&
    scoreGap > 48 &&
    primaryAxesOrder === 0 &&
    !pickupUpgrade &&
    !strategicAxesBetter;
  const projectedProgressRegressesSafePickup =
    !selectedUnsafe &&
    (selected.safeSupermanaPickupNow || selected.safeOpponentManaPickupNow) &&
    (plan.headFamily === TurnPlanFamily.SafeSupermanaProgress ||
      plan.headFamily === TurnPlanFamily.SafeOpponentManaProgress) &&
    !pickupUpgrade &&
    !projectedHeadNotWorse;
  const projectedDeferredRecoveryWithoutConcreteGain =
    selectedUnsafe &&
    candidateUnsafe &&
    !selectedSpiritPhase &&
    !selectedProgress &&
    candidateProgress &&
    (plan.headFamily === TurnPlanFamily.SafeSupermanaProgress ||
      plan.headFamily === TurnPlanFamily.SafeOpponentManaProgress) &&
    plan.goalFamily === TurnPlanFamily.DrainerSafetyRecovery &&
    !candidate.classes.drainerSafetyRecover &&
    candidate.ownDrainerVulnerable &&
    !pickupUpgrade &&
    !scoresNowBetter &&
    !drainerAttackBetter &&
    !sameTurnWindowBetter &&
    scoreGap > 48;
  const safeRootBlocksPlainSpirit =
    game.activeColor === Color.Black &&
    plan.headFamily === TurnPlanFamily.SpiritImpact &&
    !selectedUnsafe &&
    !candidateUnsafe &&
    !selectedProgress &&
    !selectedSpiritPhase &&
    !candidateProgress &&
    !candidateSpiritTactical &&
    !candidate.spiritOwnManaSetupNow &&
    scoreGap > 96 &&
    !strategicAxesBetter;
  const safeRootBlocksPlainSpiritProgress =
    game.activeColor === Color.Black &&
    plan.headFamily === TurnPlanFamily.SpiritImpact &&
    !selectedUnsafe &&
    !candidateUnsafe &&
    !selectedProgress &&
    !selectedSpiritPhase &&
    isPlainSpiritDevelopmentRoot(candidate) &&
    candidateProgress &&
    !candidateSpiritTactical &&
    !progressBetter &&
    candidate.safeSupermanaProgressSteps >=
      selected.safeSupermanaProgressSteps &&
    candidate.safeOpponentManaProgressSteps >=
      selected.safeOpponentManaProgressSteps &&
    candidate.ownDrainerVulnerable === selected.ownDrainerVulnerable &&
    candidate.manaHandoffToOpponent === selected.manaHandoffToOpponent &&
    candidate.hasRoundtrip === selected.hasRoundtrip &&
    !scoresNowBetter &&
    !drainerAttackBetter &&
    scoreGap > 64 &&
    !strategicAxesBetter;
  const plainSpiritSiblingRegresses =
    plan.headFamily === TurnPlanFamily.SpiritImpact &&
    !selectedUnsafe &&
    !candidateUnsafe &&
    isPlainSpiritDevelopmentRoot(selected) &&
    isPlainSpiritDevelopmentRoot(candidate) &&
    scoreGap >= 0 &&
    !progressBetter &&
    candidate.spiritSetupGain <= selected.spiritSetupGain &&
    candidate.safeSupermanaProgressSteps >=
      selected.safeSupermanaProgressSteps &&
    candidate.safeOpponentManaProgressSteps >=
      selected.safeOpponentManaProgressSteps &&
    candidate.ownDrainerVulnerable === selected.ownDrainerVulnerable &&
    candidate.manaHandoffToOpponent === selected.manaHandoffToOpponent &&
    candidate.hasRoundtrip === selected.hasRoundtrip &&
    !scoresNowBetter &&
    !drainerAttackBetter &&
    !candidateSpiritTactical;
  const projectedOverride =
    projectedSafe &&
    !selected.winsImmediately &&
    plan.goalFamily !== TurnPlanFamily.ManaTempo &&
    candidateIndex <= (plan.compiledChunks.length > 1 ? 16 : 10) &&
    projectedReplyNotWorse &&
    (utilityPassesOverrideGuard(plan.utility, selectedUtility) ||
      utilitySupportsPrimaryAxesEvalTolerance(
        plan.utility,
        selectedUtility,
        96,
      )) &&
    (candidateUnsafe ||
      plan.compiledChunks.length > 1 ||
      utilityImprovesNonScoreOverrideAxes(plan.utility, selectedUtility)) &&
    !safeRootBlocksPlainSpirit &&
    !safeRootBlocksPlainSpiritProgress &&
    !plainSpiritSiblingRegresses &&
    !projectedProgressRegressesSafePickup &&
    !projectedDeferredRecoveryWithoutConcreteGain &&
    !narrowUnsafeBlackManaScore &&
    !narrowWhiteManaOnlyProgressTie;
  if (candidateUnsafe && !selectedUnsafe && !projectedOverride) {
    return { kind: "reject" };
  }
  if (
    macroMode &&
    compareUtilityPrimaryAxes(selectedUtility, plan.utility) > 0 &&
    !whiteSpiritSetupGain
  ) {
    return { kind: "reject" };
  }
  const allowNonConcreteWhiteProgress =
    macroMode &&
    (plan.headFamily === TurnPlanFamily.SafeSupermanaProgress ||
      plan.headFamily === TurnPlanFamily.SafeOpponentManaProgress) &&
    !candidateProgress &&
    productionIsEarlyWhiteTurnStart(game) &&
    isPlainSpiritDevelopmentRoot(selected) &&
    !selectedUnsafe &&
    (!candidateUnsafe || projectedOverride) &&
    candidateIndex <= 3 &&
    candidate.score >= selected.score &&
    utilitySupportsPrimaryAxesEvalTolerance(plan.utility, selectedUtility, 64);
  if (whiteSpiritSetupGain) return { kind: "accept" };
  if (
    macroMode &&
    (plan.headFamily === TurnPlanFamily.SafeSupermanaProgress ||
      plan.headFamily === TurnPlanFamily.SafeOpponentManaProgress) &&
    !candidateProgress &&
    !projectedOverride &&
    !allowNonConcreteWhiteProgress
  ) {
    return { kind: "reject" };
  }
  if (
    macroMode &&
    plan.headFamily === TurnPlanFamily.DrainerKill &&
    !candidate.attacksOpponentDrainer
  ) {
    return { kind: "reject" };
  }
  const whiteSetupRecoveryBlocksUtilityOverride =
    macroMode &&
    game.activeColor === Color.White &&
    plan.headFamily === TurnPlanFamily.DrainerSafetyRecovery &&
    selected.spiritOwnManaSetupNow &&
    !candidate.spiritOwnManaSetupNow &&
    !selectedUnsafe &&
    !candidateUnsafe &&
    selected.sameTurnScoreWindowValue === 0 &&
    candidate.sameTurnScoreWindowValue === 0 &&
    selected.spiritSetupGain >=
      saturatingScoreAdd(candidate.spiritSetupGain, 48) &&
    !scoresNowBetter &&
    !drainerAttackBetter &&
    !sameTurnWindowBetter;
  const whiteVulnerableProgressBlocksImmediateScore =
    macroMode &&
    game.activeColor === Color.White &&
    (plan.headFamily === TurnPlanFamily.SafeSupermanaProgress ||
      plan.headFamily === TurnPlanFamily.SafeOpponentManaProgress) &&
    plan.goalFamily === TurnPlanFamily.ImmediateScore &&
    selectedFamily === TurnPlanFamily.ManaTempo &&
    !selectedProgress &&
    candidateProgress &&
    selected.ownDrainerVulnerable &&
    candidate.ownDrainerVulnerable &&
    selected.ownDrainerWalkVulnerable === candidate.ownDrainerWalkVulnerable &&
    selected.manaHandoffToOpponent === candidate.manaHandoffToOpponent &&
    selected.hasRoundtrip === candidate.hasRoundtrip &&
    !selected.spiritDevelopment &&
    !candidate.spiritDevelopment &&
    !selected.spiritSameTurnScoreSetupNow &&
    !candidate.spiritSameTurnScoreSetupNow &&
    !selected.spiritOwnManaSetupNow &&
    !candidate.spiritOwnManaSetupNow &&
    !scoresNowBetter &&
    !drainerAttackBetter &&
    !sameTurnWindowBetter &&
    !pickupUpgrade &&
    !candidate.classes.drainerSafetyRecover &&
    candidate.safeSupermanaProgressSteps + 1 >=
      selected.safeSupermanaProgressSteps &&
    candidate.safeOpponentManaProgressSteps + 1 >=
      selected.safeOpponentManaProgressSteps &&
    candidate.score <= saturatingScoreAdd(selected.score, 16) &&
    !strategicAxesBetter;
  if (whiteVulnerableProgressBlocksImmediateScore) {
    return { kind: "reject" };
  }
  const allowGenericProductionOverride =
    plan.headFamily === TurnPlanFamily.ImmediateScore ||
    plan.headFamily === TurnPlanFamily.DenyOpponentWindow ||
    plan.headFamily === TurnPlanFamily.DrainerKill ||
    (plan.headFamily === TurnPlanFamily.DrainerSafetyRecovery &&
      productionSecondaryAnalysisLive(config));
  if (
    macroMode &&
    !selected.winsImmediately &&
    allowGenericProductionOverride &&
    utilityPassesOverrideGuard(plan.utility, selectedUtility) &&
    !whiteSetupRecoveryBlocksUtilityOverride &&
    (!candidateUnsafe || selectedUnsafe)
  ) {
    return { kind: "accept" };
  }
  if (
    plan.headFamily === TurnPlanFamily.SpiritImpact &&
    !candidateSpiritTactical &&
    !candidate.spiritDevelopment &&
    !candidate.spiritOwnManaSetupNow &&
    !projectedOverride
  ) {
    return { kind: "reject" };
  }
  if (projectedOverride) return { kind: "accept" };

  return {
    kind: "delegate",
    policy: {
      selectedUtility,
      pickupUpgrade,
      strategicAxesBetter,
      projectedDeferredRecoveryWithoutConcreteGain,
      safeRootBlocksPlainSpirit,
      safeRootBlocksPlainSpiritProgress,
      plainSpiritSiblingRegresses,
      allowNonConcreteWhiteProgress,
      whiteSetupRecoveryBlocksUtilityOverride,
    },
  };
}
