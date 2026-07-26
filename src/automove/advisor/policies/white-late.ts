import { Color } from "../../../engine/domain.js";
import { MonsGame } from "../../../engine/game.js";
import { exactOpportunityContext } from "../../exact.js";
import type { AutomoveExecutionContext } from "../../execution-context.js";
import { rootFamily as advisorRootFamily } from "../../root-family.js";
import { compareRankedEvaluatedRootIndices } from "../../root-selector.js";
import {
  saturatingScoreAdd,
  saturatingScoreSubtract,
} from "../../score-math.js";
import type { EvaluatedRoot } from "../../search.js";
import {
  hasProgressSurface,
  productionEnabled,
  rootIsUnsafe as advisorRootIsUnsafe,
} from "../../selector-types.js";
import type { AutomoveConfig } from "../../selector-types.js";
import { TurnPlanFamily } from "../../turn-engine.js";
import { rootIsNonTactical } from "./black-baseline.js";
import {
  bestOverrideIndex,
  compareRootRankThenRanked,
  isTurnPlanFamilyOneOf,
  sameFirstInput,
} from "../support.js";

function whiteEarlySafeProgressSetupCompetitionOverride(
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
    game.monsMovesCount !== 1 ||
    !game.playerCanUseAction() ||
    !game.playerCanMoveMana() ||
    roots.length === 0
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
    advisorRootIsUnsafe(approved) ||
    !rootIsNonTactical(approved) ||
    approved.spiritDevelopment ||
    approved.spiritSameTurnScoreSetupNow ||
    approved.spiritOwnManaSetupNow ||
    approved.sameTurnScoreWindowValue > 0
  ) {
    return undefined;
  }
  return bestOverrideIndex(
    roots,
    selectionIndices,
    (challenger, index) => {
      const rankRescue =
        saturatingScoreSubtract(approved.score, challenger.score) <= 160 &&
        challenger.spiritSetupGain >=
          saturatingScoreAdd(approved.spiritSetupGain, 64) &&
        challenger.safeSupermanaProgressSteps ===
          approved.safeSupermanaProgressSteps &&
        challenger.safeOpponentManaProgressSteps ===
          approved.safeOpponentManaProgressSteps &&
        challenger.rootRank + 6 <= approved.rootRank;
      return (
        index !== approvedIndex &&
        advisorRootFamily(challenger) === TurnPlanFamily.SpiritImpact &&
        challenger.spiritOwnManaSetupNow &&
        !challenger.spiritSameTurnScoreSetupNow &&
        hasProgressSurface(challenger) &&
        !advisorRootIsUnsafe(challenger) &&
        rootIsNonTactical(challenger) &&
        challenger.sameTurnScoreWindowValue === 0 &&
        challenger.manaHandoffToOpponent === approved.manaHandoffToOpponent &&
        challenger.hasRoundtrip === approved.hasRoundtrip &&
        challenger.ownDrainerVulnerable === approved.ownDrainerVulnerable &&
        challenger.ownDrainerWalkVulnerable ===
          approved.ownDrainerWalkVulnerable &&
        (saturatingScoreSubtract(approved.score, challenger.score) <= 32 ||
          rankRescue) &&
        challenger.spiritSetupGain >=
          saturatingScoreAdd(approved.spiritSetupGain, 64) &&
        challenger.safeSupermanaProgressSteps <=
          approved.safeSupermanaProgressSteps &&
        challenger.safeOpponentManaProgressSteps <=
          approved.safeOpponentManaProgressSteps &&
        challenger.rootRank <= approved.rootRank + 4
      );
    },
    (left, right) => {
      const leftRoot = roots[left];
      const rightRoot = roots[right];
      if (leftRoot === undefined || rightRoot === undefined)
        return left - right;
      if (leftRoot.score !== rightRoot.score)
        return leftRoot.score > rightRoot.score ? -1 : 1;
      if (leftRoot.spiritSetupGain !== rightRoot.spiritSetupGain)
        return leftRoot.spiritSetupGain < rightRoot.spiritSetupGain ? -1 : 1;
      if (leftRoot.rootRank !== rightRoot.rootRank)
        return leftRoot.rootRank < rightRoot.rootRank ? -1 : 1;
      return compareRankedEvaluatedRootIndices(roots, left, right);
    },
  );
}

function whiteEarlySetupCompetitionOverride(
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
    game.monsMovesCount !== 1 ||
    !game.playerCanUseAction() ||
    !game.playerCanMoveMana() ||
    selectionIndices.length !== 1 ||
    roots.length === 0
  ) {
    return undefined;
  }
  const approved = roots[approvedIndex];
  if (
    approved === undefined ||
    advisorRootFamily(approved) !== TurnPlanFamily.SpiritImpact ||
    !approved.spiritOwnManaSetupNow ||
    approved.spiritSameTurnScoreSetupNow ||
    !rootIsNonTactical(approved) ||
    approved.sameTurnScoreWindowValue > 0 ||
    advisorRootIsUnsafe(approved)
  ) {
    return undefined;
  }
  const exact = exactOpportunityContext(execution, game, game.activeColor);
  if (exact.delta.sameTurnScoreWindowValue === 0) return undefined;
  return bestOverrideIndex(
    roots,
    roots.map((_root, index) => index),
    (root, index) =>
      index !== approvedIndex &&
      advisorRootFamily(root) === TurnPlanFamily.ManaTempo &&
      !root.ownDrainerVulnerable &&
      !root.ownDrainerWalkVulnerable &&
      !root.spiritDevelopment &&
      !root.spiritSameTurnScoreSetupNow &&
      !root.spiritOwnManaSetupNow &&
      root.sameTurnScoreWindowValue === 0 &&
      rootIsNonTactical(root) &&
      !advisorRootIsUnsafe(root),
    (left, right) => compareRankedEvaluatedRootIndices(roots, left, right),
  );
}

function isLateWhiteSpiritFollowupSafeProgressPair(
  game: MonsGame,
  candidate: EvaluatedRoot,
  incumbent: EvaluatedRoot,
  config: AutomoveConfig,
): boolean {
  return (
    productionEnabled(config) &&
    game.activeColor === Color.White &&
    game.turnNumber >= 8 &&
    game.monsMovesCount === 0 &&
    game.playerCanUseAction() &&
    game.playerCanMoveMana() &&
    advisorRootFamily(candidate) === TurnPlanFamily.SpiritImpact &&
    candidate.spiritDevelopment &&
    !candidate.spiritSameTurnScoreSetupNow &&
    candidate.sameTurnScoreWindowValue === 0 &&
    rootIsNonTactical(candidate) &&
    !advisorRootIsUnsafe(candidate) &&
    isTurnPlanFamilyOneOf(
      advisorRootFamily(incumbent),
      TurnPlanFamily.SafeSupermanaProgress,
      TurnPlanFamily.ManaTempo,
    ) &&
    !incumbent.spiritDevelopment &&
    incumbent.sameTurnScoreWindowValue === 0 &&
    rootIsNonTactical(incumbent) &&
    !advisorRootIsUnsafe(incumbent) &&
    sameFirstInput(candidate.inputs, incumbent.inputs)
  );
}

function whiteLateFollowupCompetitionOverride(
  game: MonsGame,
  roots: readonly EvaluatedRoot[],
  approvedIndex: number,
  config: AutomoveConfig,
): number | undefined {
  if (
    !productionEnabled(config) ||
    game.activeColor !== Color.White ||
    game.turnNumber < 8 ||
    game.monsMovesCount !== 0 ||
    !game.playerCanUseAction() ||
    !game.playerCanMoveMana() ||
    roots.length === 0
  ) {
    return undefined;
  }
  const approved = roots[approvedIndex];
  if (
    approved === undefined ||
    !isTurnPlanFamilyOneOf(
      advisorRootFamily(approved),
      TurnPlanFamily.SafeSupermanaProgress,
      TurnPlanFamily.ManaTempo,
    )
  ) {
    return undefined;
  }
  const candidates = roots
    .map((_root, index) => index)
    .filter((index) => {
      const challenger = roots[index];
      return (
        index !== approvedIndex &&
        challenger !== undefined &&
        isLateWhiteSpiritFollowupSafeProgressPair(
          game,
          challenger,
          approved,
          config,
        ) &&
        saturatingScoreSubtract(approved.score, challenger.score) <= 512 &&
        Math.abs(approved.rootRank - challenger.rootRank) <= 10
      );
    });
  const hasOwnSetup = candidates.some(
    (index) => roots[index]?.spiritOwnManaSetupNow === true,
  );
  return candidates.sort((left, right) => {
    const leftRoot = roots[left];
    const rightRoot = roots[right];
    if (leftRoot === undefined || rightRoot === undefined) return left - right;
    if (
      hasOwnSetup &&
      leftRoot.spiritOwnManaSetupNow !== rightRoot.spiritOwnManaSetupNow
    ) {
      return leftRoot.spiritOwnManaSetupNow ? -1 : 1;
    }
    if (hasOwnSetup && leftRoot.spiritSetupGain !== rightRoot.spiritSetupGain) {
      return leftRoot.spiritSetupGain > rightRoot.spiritSetupGain ? -1 : 1;
    }
    return compareRootRankThenRanked(roots, left, right);
  })[0];
}

export {
  whiteEarlySafeProgressSetupCompetitionOverride,
  whiteEarlySetupCompetitionOverride,
  whiteLateFollowupCompetitionOverride,
};
