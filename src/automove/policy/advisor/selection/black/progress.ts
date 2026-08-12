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
} from "../../../../config/types.js";
import type { AutomoveConfig } from "../../../../config/types.js";
import { TurnPlanFamily } from "../../../../turn/model.js";
import {
  bestOverrideIndex,
  compareRootRankThenRanked,
  exactContextIsQuiet,
  isTurnPlanFamilyOneOf,
} from "../../support.js";
import { inputChainsShareFirstInput as sameFirstInput } from "../../../../../engine/model/domain.js";

function blackPlainSpiritSetupCompetitionOverride(
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
    game.turnNumber < 6 ||
    game.monsMovesCount !== 0 ||
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
      !hasProgressSurface(challenger) &&
      !challenger.winsImmediately &&
      !challenger.attacksOpponentDrainer &&
      challenger.sameTurnScoreWindowValue === 0 &&
      !challenger.scoresSupermanaThisTurn &&
      !challenger.scoresOpponentManaThisTurn &&
      !challenger.safeSupermanaPickupNow &&
      !challenger.safeOpponentManaPickupNow &&
      challenger.manaHandoffToOpponent === approved.manaHandoffToOpponent &&
      challenger.hasRoundtrip === approved.hasRoundtrip &&
      challenger.ownDrainerVulnerable === approved.ownDrainerVulnerable &&
      challenger.ownDrainerWalkVulnerable === approved.ownDrainerWalkVulnerable &&
      sameFirstInput(challenger.inputs, approved.inputs) &&
      challenger.spiritSetupGain >= saturatingScoreAdd(approved.spiritSetupGain, 32) &&
      challenger.score >= saturatingScoreSubtract(approved.score, 96) &&
      challenger.rootRank <= approved.rootRank + 4,
    (left, right) => {
      const leftRoot = roots[left];
      const rightRoot = roots[right];
      if (leftRoot === undefined || rightRoot === undefined) return left - right;
      if (leftRoot.score !== rightRoot.score)
        return leftRoot.score > rightRoot.score ? -1 : 1;
      if (leftRoot.spiritSetupGain !== rightRoot.spiritSetupGain)
        return leftRoot.spiritSetupGain > rightRoot.spiritSetupGain ? -1 : 1;
      if (leftRoot.rootRank !== rightRoot.rootRank)
        return leftRoot.rootRank < rightRoot.rootRank ? -1 : 1;
      return compareRankedEvaluatedRootIndices(roots, left, right);
    },
  );
}

function blackNoActionProgressOverride(
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
    game.turnNumber < 6 ||
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
    approved.winsImmediately ||
    approved.attacksOpponentDrainer ||
    approved.scoresSupermanaThisTurn ||
    approved.scoresOpponentManaThisTurn ||
    approved.safeSupermanaPickupNow ||
    approved.safeOpponentManaPickupNow ||
    approved.sameTurnScoreWindowValue > 0 ||
    approved.manaHandoffToOpponent ||
    approved.hasRoundtrip ||
    approved.score < 0
  ) {
    return undefined;
  }
  return bestOverrideIndex(
    roots,
    selectionIndices,
    (challenger, index) =>
      index !== approvedIndex &&
      isTurnPlanFamilyOneOf(
        advisorRootFamily(challenger),
        TurnPlanFamily.SafeSupermanaProgress,
        TurnPlanFamily.SafeOpponentManaProgress,
      ) &&
      challenger.score >= 0 &&
      challenger.rootRank < approved.rootRank &&
      !challenger.winsImmediately &&
      !challenger.attacksOpponentDrainer &&
      !challenger.scoresSupermanaThisTurn &&
      !challenger.scoresOpponentManaThisTurn &&
      !challenger.safeSupermanaPickupNow &&
      !challenger.safeOpponentManaPickupNow &&
      challenger.sameTurnScoreWindowValue === 0 &&
      !challenger.manaHandoffToOpponent &&
      !challenger.hasRoundtrip,
    (left, right) => {
      const leftRoot = roots[left];
      const rightRoot = roots[right];
      if (leftRoot === undefined || rightRoot === undefined) return left - right;
      if (leftRoot.score !== rightRoot.score)
        return leftRoot.score > rightRoot.score ? -1 : 1;
      if (leftRoot.rootRank !== rightRoot.rootRank)
        return leftRoot.rootRank < rightRoot.rootRank ? -1 : 1;
      return compareRankedEvaluatedRootIndices(roots, left, right);
    },
  );
}

function blackNoActionManaSiblingOverride(
  game: MonsGame,
  roots: readonly EvaluatedRoot[],
  selectionIndices: readonly number[],
  approvedIndex: number,
  config: AutomoveConfig,
): number | undefined {
  if (
    !productionEnabled(config) ||
    game.activeColor !== Color.Black ||
    game.turnNumber < 6 ||
    game.monsMovesCount !== 0 ||
    game.playerCanUseAction() ||
    !game.playerCanMoveMana()
  ) {
    return undefined;
  }
  const approved = roots[approvedIndex];
  if (approved === undefined) return undefined;
  const allowWindowedSameLane =
    approved.sameTurnScoreWindowValue > 0 &&
    !approved.ownDrainerVulnerable &&
    !approved.ownDrainerWalkVulnerable;
  if (
    advisorRootFamily(approved) !== TurnPlanFamily.ManaTempo ||
    approved.winsImmediately ||
    approved.attacksOpponentDrainer ||
    approved.scoresSupermanaThisTurn ||
    approved.scoresOpponentManaThisTurn ||
    approved.safeSupermanaPickupNow ||
    approved.safeOpponentManaPickupNow ||
    approved.manaHandoffToOpponent ||
    approved.hasRoundtrip ||
    (approved.sameTurnScoreWindowValue > 0 && !allowWindowedSameLane) ||
    (approved.rootRank < 3 && !allowWindowedSameLane)
  ) {
    return undefined;
  }
  return bestOverrideIndex(
    roots,
    selectionIndices,
    (challenger, index) => {
      if (
        index === approvedIndex ||
        advisorRootFamily(challenger) !== TurnPlanFamily.ManaTempo ||
        challenger.rootRank >= approved.rootRank ||
        challenger.sameTurnScoreWindowValue > approved.sameTurnScoreWindowValue ||
        challenger.winsImmediately ||
        challenger.attacksOpponentDrainer ||
        challenger.scoresSupermanaThisTurn ||
        challenger.scoresOpponentManaThisTurn ||
        challenger.safeSupermanaPickupNow ||
        challenger.safeOpponentManaPickupNow ||
        challenger.manaHandoffToOpponent ||
        challenger.hasRoundtrip
      ) {
        return false;
      }
      if (approved.score >= 0) return challenger.score >= 0;
      if (
        allowWindowedSameLane &&
        challenger.sameTurnScoreWindowValue === approved.sameTurnScoreWindowValue &&
        challenger.safeSupermanaProgressSteps === approved.safeSupermanaProgressSteps &&
        challenger.safeOpponentManaProgressSteps ===
          approved.safeOpponentManaProgressSteps &&
        challenger.ownDrainerVulnerable === approved.ownDrainerVulnerable &&
        challenger.ownDrainerWalkVulnerable === approved.ownDrainerWalkVulnerable &&
        challenger.manaHandoffToOpponent === approved.manaHandoffToOpponent &&
        challenger.hasRoundtrip === approved.hasRoundtrip
      ) {
        return true;
      }
      return saturatingScoreSubtract(approved.score, challenger.score) <= 192;
    },
    (left, right) => compareRootRankThenRanked(roots, left, right),
  );
}

export {
  blackNoActionManaSiblingOverride,
  blackNoActionProgressOverride,
  blackPlainSpiritSetupCompetitionOverride,
};
