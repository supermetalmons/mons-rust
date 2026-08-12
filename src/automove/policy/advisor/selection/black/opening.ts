import { Color } from "../../../../../api/types.js";
import { MonsGame } from "../../../../../engine/game/mons-game.js";
import { exactOpportunityContext } from "../../../../exact/turn-opportunity.js";
import type { AutomoveExecutionContext } from "../../../../core/execution-context.js";
import { rootFamily as advisorRootFamily } from "../../../../root/family.js";
import { compareRankedEvaluatedRootIndices } from "../../../../root/evaluated-ordering.js";
import {
  saturatingScoreAdd,
  saturatingScoreSubtract,
} from "../../../../core/score-math.js";
import type { EvaluatedRoot } from "../../../../root/types.js";
import {
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
  compareRootRankThenScoreThenRanked,
  exactContextIsQuiet,
  isTurnPlanFamilyOneOf,
  sameInputAt,
} from "../../support.js";
import { inputChainsShareFirstInput as sameFirstInput } from "../../../../../engine/model/domain.js";

function blackOpeningSetupSiblingOverride(
  game: MonsGame,
  roots: readonly EvaluatedRoot[],
  selectionIndices: readonly number[],
  approvedIndex: number,
  config: AutomoveConfig,
): number | undefined {
  if (
    !productionEnabled(config) ||
    game.activeColor !== Color.Black ||
    game.turnNumber > 2 ||
    !game.playerCanUseAction() ||
    !game.playerCanMoveMana()
  ) {
    return undefined;
  }
  const approved = roots[approvedIndex];
  if (
    approved === undefined ||
    advisorRootFamily(approved) !== TurnPlanFamily.SpiritImpact ||
    !approved.spiritOwnManaSetupNow ||
    approved.spiritSameTurnScoreSetupNow ||
    approved.winsImmediately ||
    approved.attacksOpponentDrainer ||
    approved.scoresSupermanaThisTurn ||
    approved.scoresOpponentManaThisTurn ||
    approved.safeSupermanaPickupNow ||
    approved.safeOpponentManaPickupNow ||
    approved.sameTurnScoreWindowValue > 0 ||
    approved.manaHandoffToOpponent ||
    approved.hasRoundtrip ||
    advisorRootIsUnsafe(approved) ||
    approved.inputs.length < 2
  ) {
    return undefined;
  }
  return bestOverrideIndex(
    roots,
    selectionIndices,
    (challenger, index) =>
      index !== approvedIndex &&
      advisorRootFamily(challenger) === TurnPlanFamily.SpiritImpact &&
      challenger.spiritOwnManaSetupNow &&
      !challenger.spiritSameTurnScoreSetupNow &&
      !challenger.winsImmediately &&
      !challenger.attacksOpponentDrainer &&
      !challenger.scoresSupermanaThisTurn &&
      !challenger.scoresOpponentManaThisTurn &&
      !challenger.safeSupermanaPickupNow &&
      !challenger.safeOpponentManaPickupNow &&
      challenger.sameTurnScoreWindowValue === 0 &&
      !challenger.manaHandoffToOpponent &&
      !challenger.hasRoundtrip &&
      !advisorRootIsUnsafe(challenger) &&
      challenger.inputs.length >= 2 &&
      sameInputAt(challenger.inputs, approved.inputs, 0) &&
      sameInputAt(challenger.inputs, approved.inputs, 1) &&
      challenger.spiritSetupGain === approved.spiritSetupGain &&
      challenger.ownDrainerVulnerable === approved.ownDrainerVulnerable &&
      challenger.ownDrainerWalkVulnerable === approved.ownDrainerWalkVulnerable &&
      challenger.rootRank < approved.rootRank,
    (left, right) => compareRootRankThenRanked(roots, left, right),
  );
}

function blackEarlySafeManaFollowupOverride(
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
    game.turnNumber !== 2 ||
    game.monsMovesCount < 2 ||
    !game.playerCanUseAction() ||
    !game.playerCanMoveMana() ||
    roots.length === 0 ||
    !exactContextIsQuiet(execution, game)
  ) {
    return undefined;
  }
  const approved = roots[approvedIndex];
  if (
    approved === undefined ||
    advisorRootFamily(approved) !== TurnPlanFamily.SpiritImpact ||
    !isPlainSpiritDevelopmentRoot(approved) ||
    hasProgressSurface(approved) ||
    approved.spiritSameTurnScoreSetupNow ||
    approved.spiritOwnManaSetupNow ||
    approved.winsImmediately ||
    approved.attacksOpponentDrainer ||
    approved.sameTurnScoreWindowValue > 0 ||
    approved.scoresSupermanaThisTurn ||
    approved.scoresOpponentManaThisTurn ||
    approved.safeSupermanaPickupNow ||
    approved.safeOpponentManaPickupNow ||
    approved.manaHandoffToOpponent ||
    approved.hasRoundtrip ||
    advisorRootIsUnsafe(approved)
  ) {
    return undefined;
  }
  return bestOverrideIndex(
    roots,
    selectionIndices,
    (challenger, index) =>
      index !== approvedIndex &&
      advisorRootFamily(challenger) === TurnPlanFamily.ManaTempo &&
      !challenger.spiritDevelopment &&
      !challenger.spiritSameTurnScoreSetupNow &&
      !challenger.spiritOwnManaSetupNow &&
      !hasProgressSurface(challenger) &&
      !challenger.winsImmediately &&
      !challenger.attacksOpponentDrainer &&
      challenger.sameTurnScoreWindowValue === 0 &&
      !challenger.scoresSupermanaThisTurn &&
      !challenger.scoresOpponentManaThisTurn &&
      !challenger.safeSupermanaPickupNow &&
      !challenger.safeOpponentManaPickupNow &&
      !challenger.manaHandoffToOpponent &&
      !challenger.hasRoundtrip &&
      !advisorRootIsUnsafe(challenger) &&
      challenger.rootRank < approved.rootRank &&
      challenger.score >= saturatingScoreAdd(approved.score, 48) &&
      challenger.safeSupermanaProgressSteps <= approved.safeSupermanaProgressSteps &&
      challenger.safeOpponentManaProgressSteps <=
        approved.safeOpponentManaProgressSteps &&
      challenger.scorePathBestSteps <= approved.scorePathBestSteps &&
      challenger.ownDrainerVulnerable === approved.ownDrainerVulnerable &&
      challenger.ownDrainerWalkVulnerable === approved.ownDrainerWalkVulnerable,
    (left, right) => compareRootRankThenScoreThenRanked(roots, left, right),
  );
}

function blackEarlyPlainSpiritFollowupOverride(
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
    game.turnNumber !== 2 ||
    game.monsMovesCount > 1 ||
    !game.playerCanUseAction() ||
    !game.playerCanMoveMana() ||
    roots.length === 0 ||
    !exactContextIsQuiet(execution, game)
  ) {
    return undefined;
  }
  const approved = roots[approvedIndex];
  if (
    approved === undefined ||
    advisorRootFamily(approved) !== TurnPlanFamily.ManaTempo ||
    hasProgressSurface(approved) ||
    approved.spiritDevelopment ||
    approved.spiritSameTurnScoreSetupNow ||
    approved.spiritOwnManaSetupNow ||
    approved.winsImmediately ||
    approved.attacksOpponentDrainer ||
    approved.sameTurnScoreWindowValue > 0 ||
    approved.scoresSupermanaThisTurn ||
    approved.scoresOpponentManaThisTurn ||
    approved.safeSupermanaPickupNow ||
    approved.safeOpponentManaPickupNow ||
    approved.manaHandoffToOpponent ||
    approved.hasRoundtrip ||
    advisorRootIsUnsafe(approved)
  ) {
    return undefined;
  }
  return bestOverrideIndex(
    roots,
    selectionIndices,
    (challenger, index) =>
      index !== approvedIndex &&
      advisorRootFamily(challenger) === TurnPlanFamily.SpiritImpact &&
      isPlainSpiritDevelopmentRoot(challenger) &&
      !hasProgressSurface(challenger) &&
      !challenger.spiritSameTurnScoreSetupNow &&
      !challenger.spiritOwnManaSetupNow &&
      !challenger.winsImmediately &&
      !challenger.attacksOpponentDrainer &&
      challenger.sameTurnScoreWindowValue === 0 &&
      !challenger.scoresSupermanaThisTurn &&
      !challenger.scoresOpponentManaThisTurn &&
      !challenger.safeSupermanaPickupNow &&
      !challenger.safeOpponentManaPickupNow &&
      !challenger.manaHandoffToOpponent &&
      !challenger.hasRoundtrip &&
      !advisorRootIsUnsafe(challenger) &&
      challenger.rootRank > approved.rootRank &&
      challenger.rootRank <= approved.rootRank + 4 &&
      challenger.score >= saturatingScoreSubtract(approved.score, 32) &&
      challenger.spiritSetupGain >= saturatingScoreAdd(approved.spiritSetupGain, 16) &&
      challenger.scorePathBestSteps >= approved.scorePathBestSteps &&
      challenger.ownDrainerVulnerable === approved.ownDrainerVulnerable &&
      challenger.ownDrainerWalkVulnerable === approved.ownDrainerWalkVulnerable,
    (left, right) => {
      const leftRoot = roots[left];
      const rightRoot = roots[right];
      if (leftRoot === undefined || rightRoot === undefined) return left - right;
      if (leftRoot.score !== rightRoot.score)
        return leftRoot.score > rightRoot.score ? -1 : 1;
      if (leftRoot.spiritSetupGain !== rightRoot.spiritSetupGain) {
        return leftRoot.spiritSetupGain > rightRoot.spiritSetupGain ? -1 : 1;
      }
      if (leftRoot.rootRank !== rightRoot.rootRank) {
        return leftRoot.rootRank < rightRoot.rootRank ? -1 : 1;
      }
      return compareRankedEvaluatedRootIndices(roots, left, right);
    },
  );
}

function blackTurnFourVulnerableProgressManaOverride(
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
    game.monsMovesCount === 0 ||
    game.playerCanUseAction() ||
    !game.playerCanMoveMana() ||
    roots.length === 0 ||
    !exactContextIsQuiet(execution, game)
  ) {
    return undefined;
  }
  const approved = roots[approvedIndex];
  if (
    approved === undefined ||
    !isTurnPlanFamilyOneOf(
      advisorRootFamily(approved),
      TurnPlanFamily.ManaTempo,
      TurnPlanFamily.SafeSupermanaProgress,
      TurnPlanFamily.SafeOpponentManaProgress,
    ) ||
    !approved.ownDrainerVulnerable ||
    approved.winsImmediately ||
    approved.attacksOpponentDrainer ||
    approved.sameTurnScoreWindowValue > 0 ||
    approved.scoresSupermanaThisTurn ||
    approved.scoresOpponentManaThisTurn ||
    approved.safeSupermanaPickupNow ||
    approved.safeOpponentManaPickupNow ||
    approved.manaHandoffToOpponent ||
    approved.hasRoundtrip
  ) {
    return undefined;
  }
  return bestOverrideIndex(
    roots,
    selectionIndices,
    (challenger, index) =>
      index !== approvedIndex &&
      advisorRootFamily(challenger) === TurnPlanFamily.ManaTempo &&
      !challenger.ownDrainerVulnerable &&
      !challenger.ownDrainerWalkVulnerable &&
      !challenger.winsImmediately &&
      !challenger.attacksOpponentDrainer &&
      !challenger.spiritDevelopment &&
      !challenger.spiritSameTurnScoreSetupNow &&
      !challenger.spiritOwnManaSetupNow &&
      challenger.sameTurnScoreWindowValue === 0 &&
      !challenger.scoresSupermanaThisTurn &&
      !challenger.scoresOpponentManaThisTurn &&
      !challenger.safeSupermanaPickupNow &&
      !challenger.safeOpponentManaPickupNow &&
      !challenger.manaHandoffToOpponent &&
      !challenger.hasRoundtrip,
    (left, right) => compareRootRankThenScoreThenRanked(roots, left, right),
  );
}

function blackTurnSixAttackVulnerableProgressManaOverride(
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
    game.turnNumber !== 6 ||
    game.monsMovesCount !== 0 ||
    !game.playerCanUseAction() ||
    !game.playerCanMoveMana() ||
    roots.length === 0
  ) {
    return undefined;
  }
  const exact = exactOpportunityContext(execution, game, game.activeColor);
  if (
    exact.delta.sameTurnScoreWindowValue > 1 ||
    exact.delta.opponentWindowDenyGain > 1 ||
    !exact.delta.drainerAttackAvailable
  ) {
    return undefined;
  }
  const approved = roots[approvedIndex];
  if (
    approved === undefined ||
    !isTurnPlanFamilyOneOf(
      advisorRootFamily(approved),
      TurnPlanFamily.SafeSupermanaProgress,
      TurnPlanFamily.SafeOpponentManaProgress,
    ) ||
    !approved.ownDrainerVulnerable ||
    approved.ownDrainerWalkVulnerable ||
    approved.spiritDevelopment ||
    approved.spiritSameTurnScoreSetupNow ||
    approved.spiritOwnManaSetupNow ||
    approved.sameTurnScoreWindowValue > 0 ||
    approved.winsImmediately ||
    approved.attacksOpponentDrainer ||
    approved.scoresSupermanaThisTurn ||
    approved.scoresOpponentManaThisTurn ||
    approved.safeSupermanaPickupNow ||
    approved.safeOpponentManaPickupNow ||
    approved.manaHandoffToOpponent ||
    approved.hasRoundtrip
  ) {
    return undefined;
  }
  return bestOverrideIndex(
    roots,
    selectionIndices,
    (challenger, index) =>
      index !== approvedIndex &&
      advisorRootFamily(challenger) === TurnPlanFamily.ManaTempo &&
      !challenger.ownDrainerVulnerable &&
      !challenger.ownDrainerWalkVulnerable &&
      !challenger.winsImmediately &&
      !challenger.attacksOpponentDrainer &&
      !challenger.spiritDevelopment &&
      !challenger.spiritSameTurnScoreSetupNow &&
      !challenger.spiritOwnManaSetupNow &&
      challenger.sameTurnScoreWindowValue === 0 &&
      !challenger.scoresSupermanaThisTurn &&
      !challenger.scoresOpponentManaThisTurn &&
      !challenger.safeSupermanaPickupNow &&
      !challenger.safeOpponentManaPickupNow &&
      !challenger.manaHandoffToOpponent &&
      !challenger.hasRoundtrip &&
      challenger.safeSupermanaProgressSteps === approved.safeSupermanaProgressSteps &&
      challenger.safeOpponentManaProgressSteps ===
        approved.safeOpponentManaProgressSteps + 1 &&
      challenger.scorePathBestSteps === approved.scorePathBestSteps &&
      saturatingScoreSubtract(approved.score, challenger.score) <= 160 &&
      challenger.rootRank <= approved.rootRank + 4,
    (left, right) => compareRootRankThenRanked(roots, left, right),
  );
}

function blackTurnFourSetupClusterOverride(
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
    game.monsMovesCount === 0 ||
    !game.playerCanUseAction() ||
    !game.playerCanMoveMana() ||
    roots.length === 0 ||
    !exactContextIsQuiet(execution, game)
  ) {
    return undefined;
  }
  const approved = roots[approvedIndex];
  if (
    approved === undefined ||
    advisorRootFamily(approved) !== TurnPlanFamily.SpiritImpact ||
    !isPlainSpiritDevelopmentRoot(approved) ||
    approved.spiritOwnManaSetupNow ||
    approved.spiritSameTurnScoreSetupNow ||
    approved.winsImmediately ||
    approved.attacksOpponentDrainer ||
    approved.sameTurnScoreWindowValue > 0 ||
    approved.scoresSupermanaThisTurn ||
    approved.scoresOpponentManaThisTurn ||
    approved.safeSupermanaPickupNow ||
    approved.safeOpponentManaPickupNow ||
    approved.manaHandoffToOpponent ||
    approved.hasRoundtrip
  ) {
    return undefined;
  }
  return bestOverrideIndex(
    roots,
    selectionIndices,
    (challenger, index) =>
      index !== approvedIndex &&
      advisorRootFamily(challenger) === TurnPlanFamily.SpiritImpact &&
      challenger.spiritOwnManaSetupNow &&
      !challenger.spiritSameTurnScoreSetupNow &&
      !challenger.winsImmediately &&
      !challenger.attacksOpponentDrainer &&
      challenger.sameTurnScoreWindowValue === 0 &&
      !challenger.scoresSupermanaThisTurn &&
      !challenger.scoresOpponentManaThisTurn &&
      !challenger.safeSupermanaPickupNow &&
      !challenger.safeOpponentManaPickupNow &&
      !challenger.manaHandoffToOpponent &&
      !challenger.hasRoundtrip &&
      sameFirstInput(challenger.inputs, approved.inputs) &&
      challenger.ownDrainerVulnerable === approved.ownDrainerVulnerable &&
      challenger.ownDrainerWalkVulnerable === approved.ownDrainerWalkVulnerable &&
      challenger.spiritSetupGain >= saturatingScoreAdd(approved.spiritSetupGain, 32) &&
      challenger.rootRank < approved.rootRank,
    (left, right) => compareRootRankThenRanked(roots, left, right),
  );
}

export {
  blackEarlyPlainSpiritFollowupOverride,
  blackEarlySafeManaFollowupOverride,
  blackOpeningSetupSiblingOverride,
  blackTurnFourSetupClusterOverride,
  blackTurnFourVulnerableProgressManaOverride,
  blackTurnSixAttackVulnerableProgressManaOverride,
};
