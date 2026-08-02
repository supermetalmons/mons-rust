import { Color } from "../../../engine/domain.js";
import { MonsGame } from "../../../engine/game.js";
import type { AutomoveExecutionContext } from "../../execution-context.js";
import {
  rootProgressOrSetupBetter,
  rootReplyRiskSnapshot,
  spiritFollowupFloorScore,
} from "../../reply-risk.js";
import { rootFamily as advisorRootFamily } from "../../root-family.js";
import { compareRankedEvaluatedRootIndices } from "../../root-selector.js";
import {
  saturatingScoreAdd,
  saturatingScoreSubtract,
} from "../../score-math.js";
import type { EvaluatedRoot } from "../../search.js";
import {
  hasProgressSurface,
  isPlainSpiritDevelopmentRoot,
  productionEnabled,
  rootIsUnsafe as advisorRootIsUnsafe,
} from "../../selector-types.js";
import type { AutomoveConfig } from "../../selector-types.js";
import {
  TurnPlanFamily,
  compareUtilityPrimaryAxes,
} from "../../turn-engine.js";
import { rootIsNonTactical } from "./black-baseline.js";
import { replyLimitForRoots } from "./cross-color.js";
import {
  advisorRootIsSafe,
  bestOverrideIndex,
  compareRootRankThenRanked,
  exactContextIsQuiet,
  isTurnPlanFamilyOneOf,
  memoizedByIndex,
  rootUtility,
  sameFirstInput,
  utilityCompetes,
} from "../support.js";

function whiteSetupProgressCompetitionOverride(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  roots: readonly EvaluatedRoot[],
  selectionIndices: readonly number[],
  approvedIndex: number,
  perspective: Color,
  config: AutomoveConfig,
): number | undefined {
  if (
    !productionEnabled(config) ||
    game.activeColor !== Color.White ||
    game.turnNumber < 5 ||
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
    advisorRootFamily(approved) !== TurnPlanFamily.SpiritImpact ||
    !isPlainSpiritDevelopmentRoot(approved) ||
    !hasProgressSurface(approved) ||
    !rootIsNonTactical(approved) ||
    approved.sameTurnScoreWindowValue > 0 ||
    advisorRootIsUnsafe(approved)
  ) {
    return undefined;
  }
  const replyLimit = replyLimitForRoots(selectionIndices.length, config);
  const approvedSnapshot = rootReplyRiskSnapshot(
    execution,
    approved.game,
    perspective,
    config,
    replyLimit,
  );
  if (
    approvedSnapshot.allowsImmediateOpponentWin ||
    approvedSnapshot.opponentReachesMatchPoint
  ) {
    return undefined;
  }
  const utility = memoizedByIndex((index) => {
    const root = roots[index];
    return root === undefined
      ? undefined
      : rootUtility(execution, game, root, perspective, config);
  });
  const snapshot = memoizedByIndex(
    (index) => {
      const root = roots[index];
      return root === undefined
        ? undefined
        : rootReplyRiskSnapshot(
            execution,
            root.game,
            perspective,
            config,
            replyLimit,
          );
    },
    new Map([[approvedIndex, approvedSnapshot]]),
  );
  const followup = memoizedByIndex((index) => {
    const root = roots[index];
    return root === undefined
      ? undefined
      : spiritFollowupFloorScore(execution, root.game, perspective, config);
  });
  const approvedUtility = utility(approvedIndex);
  const approvedFollowup = followup(approvedIndex);
  if (approvedUtility === undefined || approvedFollowup === undefined)
    return undefined;
  let bestIndex: number | undefined;
  for (const index of selectionIndices) {
    if (index === approvedIndex) continue;
    const challenger = roots[index];
    if (
      challenger === undefined ||
      advisorRootFamily(challenger) !== TurnPlanFamily.SpiritImpact ||
      !challenger.spiritOwnManaSetupNow ||
      challenger.spiritSameTurnScoreSetupNow ||
      !hasProgressSurface(challenger) ||
      !advisorRootIsSafe(challenger) ||
      !rootIsNonTactical(challenger) ||
      challenger.sameTurnScoreWindowValue > 0 ||
      !sameFirstInput(challenger.inputs, approved.inputs) ||
      challenger.ownDrainerVulnerable !== approved.ownDrainerVulnerable ||
      challenger.ownDrainerWalkVulnerable !== approved.ownDrainerWalkVulnerable
    ) {
      continue;
    }
    const strictCompetition =
      challenger.safeSupermanaProgressSteps ===
        approved.safeSupermanaProgressSteps &&
      challenger.safeOpponentManaProgressSteps ===
        approved.safeOpponentManaProgressSteps &&
      challenger.manaHandoffToOpponent === approved.manaHandoffToOpponent &&
      challenger.hasRoundtrip === approved.hasRoundtrip &&
      saturatingScoreSubtract(approved.score, challenger.score) <= 64 &&
      challenger.spiritSetupGain >=
        saturatingScoreAdd(approved.spiritSetupGain, 32) &&
      challenger.rootRank <= approved.rootRank + 2;
    const challengerUtility = utility(index);
    const challengerSnapshot = snapshot(index);
    const challengerFollowup = followup(index);
    if (
      challengerUtility === undefined ||
      challengerSnapshot === undefined ||
      challengerFollowup === undefined
    ) {
      continue;
    }
    const followupCompetition =
      !challengerSnapshot.allowsImmediateOpponentWin &&
      !challengerSnapshot.opponentReachesMatchPoint &&
      saturatingScoreSubtract(approved.score, challenger.score) <= 128 &&
      saturatingScoreAdd(challengerSnapshot.worstReplyScore, 320) >=
        approvedSnapshot.worstReplyScore &&
      saturatingScoreAdd(challengerFollowup, 32) >= approvedFollowup &&
      challenger.rootRank <= approved.rootRank + 4 &&
      (utilityCompetes(challengerUtility, approvedUtility) ||
        rootProgressOrSetupBetter(challenger, approved));
    if (!strictCompetition && !followupCompetition) continue;
    if (bestIndex === undefined) {
      bestIndex = index;
      continue;
    }
    const current = roots[bestIndex];
    const currentUtility = utility(bestIndex);
    const currentSnapshot = snapshot(bestIndex);
    const currentFollowup = followup(bestIndex);
    if (
      current === undefined ||
      currentUtility === undefined ||
      currentSnapshot === undefined ||
      currentFollowup === undefined
    ) {
      bestIndex = index;
      continue;
    }
    const utilityOrder = compareUtilityPrimaryAxes(
      challengerUtility,
      currentUtility,
    );
    const replace =
      utilityOrder > 0 ||
      (utilityOrder >= 0 && challengerFollowup > currentFollowup) ||
      (utilityOrder >= 0 &&
        challengerFollowup === currentFollowup &&
        challengerSnapshot.worstReplyScore > currentSnapshot.worstReplyScore) ||
      (challengerFollowup === currentFollowup &&
        challengerSnapshot.worstReplyScore ===
          currentSnapshot.worstReplyScore &&
        challenger.spiritSetupGain > current.spiritSetupGain) ||
      (challengerFollowup === currentFollowup &&
        challengerSnapshot.worstReplyScore ===
          currentSnapshot.worstReplyScore &&
        challenger.spiritSetupGain === current.spiritSetupGain &&
        compareRankedEvaluatedRootIndices(roots, index, bestIndex) < 0);
    if (replace) bestIndex = index;
  }
  return bestIndex;
}

function whiteEarlyFollowupSetupCompetitionOverride(
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
    advisorRootFamily(approved) !== TurnPlanFamily.SpiritImpact ||
    !isPlainSpiritDevelopmentRoot(approved) ||
    !hasProgressSurface(approved) ||
    !rootIsNonTactical(approved) ||
    approved.sameTurnScoreWindowValue > 0 ||
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
      challenger.spiritOwnManaSetupNow &&
      !challenger.spiritSameTurnScoreSetupNow &&
      hasProgressSurface(challenger) &&
      rootIsNonTactical(challenger) &&
      challenger.sameTurnScoreWindowValue === 0 &&
      !advisorRootIsUnsafe(challenger) &&
      sameFirstInput(challenger.inputs, approved.inputs) &&
      challenger.supermanaProgress === approved.supermanaProgress &&
      challenger.opponentManaProgress === approved.opponentManaProgress &&
      challenger.safeSupermanaProgressSteps ===
        approved.safeSupermanaProgressSteps &&
      challenger.safeOpponentManaProgressSteps ===
        approved.safeOpponentManaProgressSteps &&
      challenger.ownDrainerVulnerable === approved.ownDrainerVulnerable &&
      challenger.ownDrainerWalkVulnerable ===
        approved.ownDrainerWalkVulnerable &&
      saturatingScoreSubtract(approved.score, challenger.score) <= 64 &&
      challenger.spiritSetupGain >=
        saturatingScoreAdd(approved.spiritSetupGain, 32) &&
      challenger.rootRank <= approved.rootRank + 4,
    (left, right) => {
      const leftRoot = roots[left];
      const rightRoot = roots[right];
      if (leftRoot === undefined || rightRoot === undefined)
        return left - right;
      if (leftRoot.score !== rightRoot.score)
        return leftRoot.score > rightRoot.score ? -1 : 1;
      if (leftRoot.rootRank !== rightRoot.rootRank)
        return leftRoot.rootRank < rightRoot.rootRank ? -1 : 1;
      return compareRankedEvaluatedRootIndices(roots, left, right);
    },
  );
}

function whiteEarlySetupSiblingProgressOverride(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  roots: readonly EvaluatedRoot[],
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
    roots.length === 0 ||
    !exactContextIsQuiet(execution, game)
  ) {
    return undefined;
  }
  const approved = roots[approvedIndex];
  if (
    approved === undefined ||
    advisorRootFamily(approved) !== TurnPlanFamily.SpiritImpact ||
    !approved.spiritOwnManaSetupNow ||
    approved.spiritSameTurnScoreSetupNow ||
    !hasProgressSurface(approved) ||
    !rootIsNonTactical(approved) ||
    approved.sameTurnScoreWindowValue > 0 ||
    advisorRootIsUnsafe(approved)
  ) {
    return undefined;
  }
  return bestOverrideIndex(
    roots,
    roots.map((_root, index) => index),
    (challenger, index) => {
      const strictImprovement =
        challenger.safeSupermanaProgressSteps <
          approved.safeSupermanaProgressSteps ||
        challenger.safeOpponentManaProgressSteps <
          approved.safeOpponentManaProgressSteps ||
        challenger.spiritSetupGain > approved.spiritSetupGain;
      return (
        index !== approvedIndex &&
        advisorRootFamily(challenger) === TurnPlanFamily.SpiritImpact &&
        challenger.spiritOwnManaSetupNow &&
        !challenger.spiritSameTurnScoreSetupNow &&
        hasProgressSurface(challenger) &&
        rootIsNonTactical(challenger) &&
        challenger.sameTurnScoreWindowValue === 0 &&
        !advisorRootIsUnsafe(challenger) &&
        sameFirstInput(challenger.inputs, approved.inputs) &&
        challenger.supermanaProgress === approved.supermanaProgress &&
        challenger.opponentManaProgress === approved.opponentManaProgress &&
        challenger.safeSupermanaProgressSteps <=
          approved.safeSupermanaProgressSteps &&
        challenger.safeOpponentManaProgressSteps <=
          approved.safeOpponentManaProgressSteps &&
        challenger.scorePathBestSteps <= approved.scorePathBestSteps &&
        challenger.ownDrainerVulnerable === approved.ownDrainerVulnerable &&
        challenger.ownDrainerWalkVulnerable ===
          approved.ownDrainerWalkVulnerable &&
        challenger.score >= saturatingScoreSubtract(approved.score, 64) &&
        challenger.rootRank <= approved.rootRank + 8 &&
        strictImprovement
      );
    },
    (left, right) => {
      const leftRoot = roots[left];
      const rightRoot = roots[right];
      if (leftRoot === undefined || rightRoot === undefined)
        return left - right;
      const leftStepDelta =
        approved.safeSupermanaProgressSteps -
        leftRoot.safeSupermanaProgressSteps;
      const rightStepDelta =
        approved.safeSupermanaProgressSteps -
        rightRoot.safeSupermanaProgressSteps;
      if (leftStepDelta !== rightStepDelta)
        return leftStepDelta > rightStepDelta ? -1 : 1;
      const leftSetupDelta = saturatingScoreSubtract(
        approved.spiritSetupGain,
        leftRoot.spiritSetupGain,
      );
      const rightSetupDelta = saturatingScoreSubtract(
        approved.spiritSetupGain,
        rightRoot.spiritSetupGain,
      );
      if (leftSetupDelta !== rightSetupDelta)
        return leftSetupDelta > rightSetupDelta ? -1 : 1;
      if (leftRoot.score !== rightRoot.score)
        return leftRoot.score > rightRoot.score ? -1 : 1;
      if (leftRoot.rootRank !== rightRoot.rootRank)
        return leftRoot.rootRank < rightRoot.rootRank ? -1 : 1;
      return compareRankedEvaluatedRootIndices(roots, left, right);
    },
  );
}

function whiteEarlyNoActionProgressCompetitionOverride(
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
    game.playerCanUseAction() ||
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
    !approved.ownDrainerVulnerable ||
    approved.ownDrainerWalkVulnerable ||
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
    (challenger, index) =>
      index !== approvedIndex &&
      isTurnPlanFamilyOneOf(
        advisorRootFamily(challenger),
        TurnPlanFamily.SafeSupermanaProgress,
        TurnPlanFamily.SafeOpponentManaProgress,
      ) &&
      !challenger.ownDrainerVulnerable &&
      !challenger.ownDrainerWalkVulnerable &&
      rootIsNonTactical(challenger) &&
      !challenger.spiritDevelopment &&
      !challenger.spiritSameTurnScoreSetupNow &&
      !challenger.spiritOwnManaSetupNow &&
      challenger.sameTurnScoreWindowValue === 0 &&
      challenger.supermanaProgress === approved.supermanaProgress &&
      challenger.opponentManaProgress === approved.opponentManaProgress &&
      challenger.safeSupermanaProgressSteps ===
        approved.safeSupermanaProgressSteps &&
      challenger.safeOpponentManaProgressSteps ===
        approved.safeOpponentManaProgressSteps &&
      challenger.score >= saturatingScoreSubtract(approved.score, 32),
    (left, right) => compareRootRankThenRanked(roots, left, right),
  );
}

function whiteManaOnlyCompetitionOverride(
  game: MonsGame,
  roots: readonly EvaluatedRoot[],
  selectionIndices: readonly number[],
  approvedIndex: number,
  config: AutomoveConfig,
): number | undefined {
  if (
    !productionEnabled(config) ||
    game.activeColor !== Color.White ||
    game.turnNumber < 5 ||
    game.monsMovesCount !== 0 ||
    game.playerCanUseAction() ||
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
    !approved.ownDrainerVulnerable ||
    approved.ownDrainerWalkVulnerable ||
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
    (challenger, index) =>
      index !== approvedIndex &&
      advisorRootFamily(challenger) === TurnPlanFamily.ManaTempo &&
      !challenger.ownDrainerVulnerable &&
      !challenger.ownDrainerWalkVulnerable &&
      !challenger.spiritDevelopment &&
      !challenger.spiritSameTurnScoreSetupNow &&
      !challenger.spiritOwnManaSetupNow &&
      challenger.sameTurnScoreWindowValue === 0 &&
      rootIsNonTactical(challenger) &&
      !advisorRootIsUnsafe(challenger) &&
      saturatingScoreSubtract(approved.score, challenger.score) <= 64 &&
      Math.abs(approved.rootRank - challenger.rootRank) <= 6,
    (left, right) => compareRootRankThenRanked(roots, left, right),
  );
}

function whiteTurnThreeSafeProgressSurfaceOverride(
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
    game.monsMovesCount !== 0 ||
    game.playerCanUseAction() ||
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
    (challenger, index) =>
      index !== approvedIndex &&
      isTurnPlanFamilyOneOf(
        advisorRootFamily(challenger),
        TurnPlanFamily.SafeSupermanaProgress,
        TurnPlanFamily.SafeOpponentManaProgress,
      ) &&
      !advisorRootIsUnsafe(challenger) &&
      rootIsNonTactical(challenger) &&
      !challenger.spiritDevelopment &&
      !challenger.spiritSameTurnScoreSetupNow &&
      !challenger.spiritOwnManaSetupNow &&
      challenger.sameTurnScoreWindowValue === 0 &&
      challenger.ownDrainerVulnerable === approved.ownDrainerVulnerable &&
      challenger.ownDrainerWalkVulnerable ===
        approved.ownDrainerWalkVulnerable &&
      challenger.manaHandoffToOpponent === approved.manaHandoffToOpponent &&
      challenger.hasRoundtrip === approved.hasRoundtrip &&
      challenger.rootRank < approved.rootRank &&
      challenger.score >= saturatingScoreSubtract(approved.score, 32) &&
      rootProgressOrSetupBetter(challenger, approved),
    (left, right) => compareRootRankThenRanked(roots, left, right),
  );
}

export {
  whiteEarlyFollowupSetupCompetitionOverride,
  whiteEarlyNoActionProgressCompetitionOverride,
  whiteEarlySetupSiblingProgressOverride,
  whiteManaOnlyCompetitionOverride,
  whiteSetupProgressCompetitionOverride,
  whiteTurnThreeSafeProgressSurfaceOverride,
};
