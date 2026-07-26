import { Color } from "../../engine/domain.js";
import { rootProgressOrSetupBetter } from "../reply-risk.js";
import { saturatingScoreSubtract } from "../score-math.js";
import {
  hasConcreteScoreSurface as rootHasConcreteScoreSurface,
  isPlainSpiritDevelopmentRoot,
} from "../selector-types.js";
import {
  TurnPlanFamily,
  utilityImprovesNonScoreOverrideAxes,
} from "../turn-engine.js";
import type {
  TurnEngineHeadAcceptanceContext,
  TurnEngineHeadInitialGuardResult,
} from "./head-types.js";
import { firstInputsEqual, hasPickupUpgrade } from "./shared.js";

export function evaluateInitialHeadGuards(
  context: TurnEngineHeadAcceptanceContext,
): TurnEngineHeadInitialGuardResult {
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
    scoreGap,
    sameTurnWindowBetter,
    drainerAttackBetter,
    scoresNowBetter,
    candidateSpiritTactical,
    selectedSpiritPhase,
    candidateFamily,
    selectedFamily,
    selectedUtilityValue,
  } = context;
  const narrowUnsafeBlackManaScore =
    macroMode &&
    plan.headFamily === TurnPlanFamily.ImmediateScore &&
    game.activeColor === Color.Black &&
    game.turnNumber <= 4 &&
    game.monsMovesCount === 0 &&
    !game.playerCanUseAction() &&
    game.playerCanMoveMana() &&
    candidateUnsafe &&
    !selectedUnsafe &&
    !candidate.winsImmediately &&
    !scoresNowBetter &&
    !drainerAttackBetter &&
    candidate.sameTurnScoreWindowValue <= selected.sameTurnScoreWindowValue;
  const earlySafeManaBlocksSpirit =
    macroMode &&
    game.activeColor === Color.White &&
    game.turnNumber === 3 &&
    game.monsMovesCount <= 2 &&
    game.playerCanUseAction() &&
    game.playerCanMoveMana() &&
    plan.headFamily === TurnPlanFamily.SpiritImpact &&
    selectedFamily === TurnPlanFamily.ManaTempo &&
    !selectedUnsafe &&
    !candidateUnsafe &&
    !selected.spiritDevelopment &&
    !selected.spiritSameTurnScoreSetupNow &&
    !selected.spiritOwnManaSetupNow &&
    candidate.spiritDevelopment &&
    candidate.spiritOwnManaSetupNow &&
    !candidate.spiritSameTurnScoreSetupNow &&
    !candidate.winsImmediately &&
    !candidate.attacksOpponentDrainer &&
    !rootHasConcreteScoreSurface(candidate) &&
    candidate.sameTurnScoreWindowValue === 0;
  const blackTurnStartSafeManaBlocksPlainSpirit =
    macroMode &&
    game.activeColor === Color.Black &&
    game.turnNumber >= 5 &&
    game.monsMovesCount === 0 &&
    game.playerCanUseAction() &&
    game.playerCanMoveMana() &&
    plan.headFamily === TurnPlanFamily.SpiritImpact &&
    selectedFamily === TurnPlanFamily.ManaTempo &&
    !selectedSpiritPhase &&
    isPlainSpiritDevelopmentRoot(candidate) &&
    !candidateProgress &&
    !candidateSpiritTactical &&
    firstInputsEqual(candidate.inputs, selected.inputs) &&
    candidateUnsafe === selectedUnsafe &&
    candidate.safeSupermanaProgressSteps >=
      selected.safeSupermanaProgressSteps &&
    candidate.safeOpponentManaProgressSteps >=
      selected.safeOpponentManaProgressSteps &&
    candidate.ownDrainerVulnerable === selected.ownDrainerVulnerable &&
    candidate.ownDrainerWalkVulnerable === selected.ownDrainerWalkVulnerable &&
    candidate.manaHandoffToOpponent === selected.manaHandoffToOpponent &&
    candidate.hasRoundtrip === selected.hasRoundtrip &&
    !scoresNowBetter &&
    !drainerAttackBetter &&
    !sameTurnWindowBetter &&
    scoreGap > 96;
  const whiteTurnStartSafeManaBlocksPlainSpirit =
    macroMode &&
    game.activeColor === Color.White &&
    game.turnNumber >= 5 &&
    game.monsMovesCount === 0 &&
    game.playerCanUseAction() &&
    game.playerCanMoveMana() &&
    plan.headFamily === TurnPlanFamily.SpiritImpact &&
    selectedFamily === TurnPlanFamily.ManaTempo &&
    !selectedUnsafe &&
    !candidateUnsafe &&
    !selectedSpiritPhase &&
    isPlainSpiritDevelopmentRoot(candidate) &&
    candidateProgress &&
    !candidateSpiritTactical &&
    firstInputsEqual(candidate.inputs, selected.inputs) &&
    candidate.safeSupermanaProgressSteps >=
      selected.safeSupermanaProgressSteps &&
    candidate.safeOpponentManaProgressSteps >=
      selected.safeOpponentManaProgressSteps &&
    candidate.ownDrainerVulnerable === selected.ownDrainerVulnerable &&
    candidate.ownDrainerWalkVulnerable === selected.ownDrainerWalkVulnerable &&
    candidate.manaHandoffToOpponent === selected.manaHandoffToOpponent &&
    candidate.hasRoundtrip === selected.hasRoundtrip &&
    candidate.score <= selected.score &&
    !scoresNowBetter &&
    !drainerAttackBetter &&
    !sameTurnWindowBetter;
  const whiteLateSafeManaBlocksPlainSpirit =
    macroMode &&
    game.activeColor === Color.White &&
    game.turnNumber >= 5 &&
    game.monsMovesCount >= 1 &&
    plan.headFamily === TurnPlanFamily.SpiritImpact &&
    selectedFamily === TurnPlanFamily.ManaTempo &&
    !selectedSpiritPhase &&
    isPlainSpiritDevelopmentRoot(candidate) &&
    !candidateProgress &&
    !candidateSpiritTactical &&
    candidate.sameTurnScoreWindowValue === 0 &&
    candidate.ownDrainerVulnerable === selected.ownDrainerVulnerable &&
    candidate.ownDrainerWalkVulnerable === selected.ownDrainerWalkVulnerable &&
    candidate.manaHandoffToOpponent === selected.manaHandoffToOpponent &&
    candidate.hasRoundtrip === selected.hasRoundtrip &&
    candidate.score <= selected.score &&
    !scoresNowBetter &&
    !drainerAttackBetter &&
    !sameTurnWindowBetter;
  const blackNoActionVulnerableProgressHead =
    macroMode &&
    game.activeColor === Color.Black &&
    game.turnNumber >= 6 &&
    game.monsMovesCount === 0 &&
    !game.playerCanUseAction() &&
    game.playerCanMoveMana() &&
    (candidateFamily === TurnPlanFamily.SafeSupermanaProgress ||
      candidateFamily === TurnPlanFamily.SafeOpponentManaProgress) &&
    selectedFamily === TurnPlanFamily.ManaTempo &&
    !selectedSpiritPhase &&
    !candidate.spiritDevelopment &&
    !candidate.spiritSameTurnScoreSetupNow &&
    !candidate.spiritOwnManaSetupNow &&
    candidate.sameTurnScoreWindowValue === 0 &&
    selected.sameTurnScoreWindowValue === 0 &&
    !rootHasConcreteScoreSurface(candidate) &&
    !rootHasConcreteScoreSurface(selected) &&
    candidate.ownDrainerVulnerable &&
    !selected.ownDrainerVulnerable &&
    candidate.ownDrainerWalkVulnerable === selected.ownDrainerWalkVulnerable &&
    candidate.manaHandoffToOpponent === selected.manaHandoffToOpponent &&
    candidate.hasRoundtrip === selected.hasRoundtrip &&
    !scoresNowBetter &&
    !drainerAttackBetter &&
    !sameTurnWindowBetter;
  const blackEarlySameWindowManaHead =
    macroMode &&
    game.activeColor === Color.Black &&
    game.turnNumber <= 4 &&
    !game.playerCanUseAction() &&
    game.playerCanMoveMana() &&
    candidateFamily === TurnPlanFamily.ManaTempo &&
    selectedFamily === TurnPlanFamily.ManaTempo &&
    candidate.sameTurnScoreWindowValue > 0 &&
    candidate.sameTurnScoreWindowValue === selected.sameTurnScoreWindowValue &&
    candidate.safeSupermanaProgressSteps ===
      selected.safeSupermanaProgressSteps &&
    candidate.safeOpponentManaProgressSteps ===
      selected.safeOpponentManaProgressSteps &&
    candidate.ownDrainerVulnerable === selected.ownDrainerVulnerable &&
    candidate.ownDrainerWalkVulnerable === selected.ownDrainerWalkVulnerable &&
    candidate.manaHandoffToOpponent === selected.manaHandoffToOpponent &&
    candidate.hasRoundtrip === selected.hasRoundtrip &&
    !rootHasConcreteScoreSurface(candidate) &&
    !rootHasConcreteScoreSurface(selected) &&
    !scoresNowBetter &&
    !drainerAttackBetter;
  const blackNoActionWindowedVulnerableManaHead =
    macroMode &&
    game.activeColor === Color.Black &&
    game.turnNumber >= 6 &&
    game.monsMovesCount === 0 &&
    !game.playerCanUseAction() &&
    game.playerCanMoveMana() &&
    candidateFamily === TurnPlanFamily.ManaTempo &&
    selectedFamily === TurnPlanFamily.ManaTempo &&
    candidate.sameTurnScoreWindowValue > 0 &&
    candidate.sameTurnScoreWindowValue === selected.sameTurnScoreWindowValue &&
    candidate.safeSupermanaProgressSteps ===
      selected.safeSupermanaProgressSteps &&
    candidate.safeOpponentManaProgressSteps ===
      selected.safeOpponentManaProgressSteps &&
    candidate.ownDrainerVulnerable &&
    !selected.ownDrainerVulnerable &&
    candidate.manaHandoffToOpponent === selected.manaHandoffToOpponent &&
    candidate.hasRoundtrip === selected.hasRoundtrip &&
    !rootHasConcreteScoreSurface(candidate) &&
    !rootHasConcreteScoreSurface(selected) &&
    !scoresNowBetter &&
    !drainerAttackBetter;
  const blackLateWindowedVulnerableManaHead =
    macroMode &&
    game.activeColor === Color.Black &&
    game.turnNumber >= 8 &&
    game.monsMovesCount >= 1 &&
    !game.playerCanUseAction() &&
    game.playerCanMoveMana() &&
    candidateFamily === TurnPlanFamily.ManaTempo &&
    selectedFamily === TurnPlanFamily.ManaTempo &&
    firstInputsEqual(candidate.inputs, selected.inputs) &&
    candidate.sameTurnScoreWindowValue > 0 &&
    selected.sameTurnScoreWindowValue === 0 &&
    candidate.ownDrainerVulnerable &&
    !selected.ownDrainerVulnerable &&
    !rootHasConcreteScoreSurface(candidate) &&
    !rootHasConcreteScoreSurface(selected) &&
    saturatingScoreSubtract(candidate.score, selected.score) <= 32 &&
    !scoresNowBetter &&
    !drainerAttackBetter;
  if (earlySafeManaBlocksSpirit) return { kind: "reject" };
  if (blackTurnStartSafeManaBlocksPlainSpirit) return { kind: "reject" };
  if (whiteTurnStartSafeManaBlocksPlainSpirit) return { kind: "reject" };
  if (whiteLateSafeManaBlocksPlainSpirit) return { kind: "reject" };
  if (blackNoActionVulnerableProgressHead) return { kind: "reject" };
  if (blackEarlySameWindowManaHead) return { kind: "reject" };
  if (blackNoActionWindowedVulnerableManaHead) return { kind: "reject" };
  if (blackLateWindowedVulnerableManaHead) return { kind: "reject" };

  const pickupUpgrade = hasPickupUpgrade(candidate, selected);
  const blackEarlyProgressBlocksMana =
    macroMode &&
    game.activeColor === Color.Black &&
    game.turnNumber <= 4 &&
    candidateFamily === TurnPlanFamily.ManaTempo &&
    (selectedFamily === TurnPlanFamily.ManaTempo ||
      selectedFamily === TurnPlanFamily.SafeSupermanaProgress ||
      selectedFamily === TurnPlanFamily.SafeOpponentManaProgress) &&
    !candidate.winsImmediately &&
    !candidate.attacksOpponentDrainer &&
    !candidate.spiritDevelopment &&
    !candidate.spiritSameTurnScoreSetupNow &&
    !candidate.spiritOwnManaSetupNow &&
    !pickupUpgrade &&
    scoreGap > 0 &&
    (rootProgressOrSetupBetter(selected, candidate) ||
      (selectedProgress && !candidateProgress));
  const blackEarlyProgressBlocksNonConcreteWindow =
    macroMode &&
    game.activeColor === Color.Black &&
    game.turnNumber <= 4 &&
    (plan.headFamily === TurnPlanFamily.SafeSupermanaProgress ||
      plan.headFamily === TurnPlanFamily.SafeOpponentManaProgress) &&
    candidateFamily === TurnPlanFamily.ManaTempo &&
    selectedFamily === TurnPlanFamily.ManaTempo &&
    candidate.sameTurnScoreWindowValue > 0 &&
    selected.sameTurnScoreWindowValue === 0 &&
    !candidate.winsImmediately &&
    !scoresNowBetter &&
    !drainerAttackBetter &&
    !pickupUpgrade &&
    selected.ownDrainerVulnerable === candidate.ownDrainerVulnerable &&
    selected.ownDrainerWalkVulnerable === candidate.ownDrainerWalkVulnerable &&
    selected.manaHandoffToOpponent === candidate.manaHandoffToOpponent &&
    selected.hasRoundtrip === candidate.hasRoundtrip &&
    rootProgressOrSetupBetter(selected, candidate) &&
    scoreGap >= -192 &&
    !utilityImprovesNonScoreOverrideAxes(plan.utility, selectedUtilityValue());
  if (blackEarlyProgressBlocksMana) return { kind: "reject" };
  if (blackEarlyProgressBlocksNonConcreteWindow) return { kind: "reject" };
  return {
    kind: "continue",
    facts: { narrowUnsafeBlackManaScore, pickupUpgrade },
  };
}
