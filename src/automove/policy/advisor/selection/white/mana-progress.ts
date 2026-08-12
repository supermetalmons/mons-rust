import { Color } from "../../../../../api/types.js";
import { MonsGame } from "../../../../../engine/game/mons-game.js";
import { exactOpportunityContext } from "../../../../exact/turn-opportunity.js";
import type { AutomoveExecutionContext } from "../../../../core/execution-context.js";
import { isProductionModeWhiteManaSiblingPair } from "../../../reply-risk/reentry.js";
import { rootProgressOrSetupBetter } from "../../../reply-risk/shortlist.js";
import { rootReplyRiskSnapshot } from "../../../reply-risk/snapshot.js";
import {
  spiritFollowupFloorScore,
  turnEngineRootPlanUtility,
} from "../../../reply-risk/projection.js";
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
  compareUtilityPrimaryAxes,
  utilityStrictlyDominatesOverrideAxes,
} from "../../../../turn/ordering.js";
import { rootIsNonTactical } from "../black/baseline.js";
import { replyLimitForRoots } from "../shared/cross-color.js";
import {
  bestOverrideIndex,
  compareRootRankThenRanked,
  exactContextIsQuiet,
  isTurnPlanFamilyOneOf,
  memoizedByIndex,
  rootUtility,
  utilityCompetes,
} from "../../support.js";
import { inputChainsShareFirstInput as sameFirstInput } from "../../../../../engine/model/domain.js";

function whiteManaCompetitionOverride(
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
    game.turnNumber !== 3 ||
    game.monsMovesCount < 3 ||
    roots.length === 0
  ) {
    return undefined;
  }
  const approved = roots[approvedIndex];
  if (
    approved === undefined ||
    advisorRootFamily(approved) !== TurnPlanFamily.ManaTempo ||
    approved.ownDrainerVulnerable ||
    approved.ownDrainerWalkVulnerable ||
    approved.manaHandoffToOpponent ||
    approved.hasRoundtrip ||
    approved.winsImmediately ||
    approved.attacksOpponentDrainer ||
    approved.sameTurnScoreWindowValue > 0 ||
    approved.scoresSupermanaThisTurn ||
    approved.scoresOpponentManaThisTurn ||
    approved.safeSupermanaPickupNow ||
    approved.safeOpponentManaPickupNow ||
    advisorRootIsUnsafe(approved)
  ) {
    return undefined;
  }
  const approvedUtility = turnEngineRootPlanUtility(
    execution,
    game,
    approved,
    perspective,
    config,
    TurnPlanFamily.ManaTempo,
  );
  const utility = memoizedByIndex((index) => {
    const root = roots[index];
    return root === undefined
      ? undefined
      : turnEngineRootPlanUtility(
          execution,
          game,
          root,
          perspective,
          config,
          TurnPlanFamily.ManaTempo,
        );
  });
  let bestIndex: number | undefined;
  let bestUtility = approvedUtility;
  let bestIsDominance = false;
  for (const index of selectionIndices) {
    if (index === approvedIndex) continue;
    const challenger = roots[index];
    if (challenger === undefined) continue;
    const sameLaneNearBest =
      isProductionModeWhiteManaSiblingPair(challenger, approved, config) &&
      saturatingScoreSubtract(approved.score, challenger.score) <= 24 &&
      Math.abs(approved.rootRank - challenger.rootRank) <= 4 &&
      challenger.rootRank < approved.rootRank;
    if (
      advisorRootFamily(challenger) !== TurnPlanFamily.ManaTempo ||
      challenger.ownDrainerVulnerable ||
      challenger.ownDrainerWalkVulnerable ||
      challenger.manaHandoffToOpponent ||
      challenger.hasRoundtrip ||
      challenger.winsImmediately ||
      challenger.attacksOpponentDrainer ||
      challenger.sameTurnScoreWindowValue > 0 ||
      challenger.scoresSupermanaThisTurn ||
      challenger.scoresOpponentManaThisTurn ||
      challenger.safeSupermanaPickupNow ||
      challenger.safeOpponentManaPickupNow ||
      (!sameLaneNearBest &&
        challenger.score < saturatingScoreSubtract(approved.score, 96)) ||
      advisorRootIsUnsafe(challenger) ||
      (challenger.rootRank >= approved.rootRank && !sameLaneNearBest)
    ) {
      continue;
    }
    const challengerUtility = utility(index);
    if (challengerUtility === undefined) continue;
    const utilityOrder = compareUtilityPrimaryAxes(challengerUtility, approvedUtility);
    const dominance =
      utilityOrder > 0 ||
      utilityStrictlyDominatesOverrideAxes(challengerUtility, approvedUtility);
    if (!dominance && !sameLaneNearBest) continue;
    let replace = bestIndex === undefined;
    if (bestIndex !== undefined) {
      const current = roots[bestIndex];
      if (current === undefined) replace = true;
      else {
        const currentSameLane =
          isProductionModeWhiteManaSiblingPair(current, approved, config) &&
          saturatingScoreSubtract(approved.score, current.score) <= 24 &&
          Math.abs(approved.rootRank - current.rootRank) <= 4 &&
          current.rootRank < approved.rootRank;
        if (dominance !== bestIsDominance) replace = dominance;
        else if (dominance) {
          const order = compareUtilityPrimaryAxes(challengerUtility, bestUtility);
          replace =
            order > 0 ||
            (order === 0 &&
              compareRankedEvaluatedRootIndices(roots, index, bestIndex) < 0);
        } else if (sameLaneNearBest && currentSameLane) {
          replace =
            challenger.rootRank < current.rootRank ||
            (challenger.rootRank === current.rootRank &&
              compareRankedEvaluatedRootIndices(roots, index, bestIndex) < 0);
        } else replace = sameLaneNearBest;
      }
    }
    if (replace) {
      bestIndex = index;
      bestUtility = challengerUtility;
      bestIsDominance = dominance;
    }
  }
  return bestIndex;
}

function whiteNoActionSafeProgressManaOverride(
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
    game.turnNumber < 5 ||
    game.monsMovesCount !== 0 ||
    game.playerCanUseAction() ||
    !game.playerCanMoveMana() ||
    !exactContextIsQuiet(execution, game)
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
    !rootIsNonTactical(approved) ||
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
      !advisorRootIsUnsafe(challenger) &&
      !challenger.ownDrainerVulnerable &&
      !challenger.ownDrainerWalkVulnerable &&
      rootIsNonTactical(challenger) &&
      challenger.sameTurnScoreWindowValue === 0 &&
      challenger.safeSupermanaProgressSteps <=
        approved.safeSupermanaProgressSteps + 1 &&
      challenger.safeOpponentManaProgressSteps <=
        approved.safeOpponentManaProgressSteps + 1 &&
      saturatingScoreAdd(challenger.score, 448) >= approved.score,
    (left, right) => compareRootRankThenRanked(roots, left, right),
  );
}

function isNonConcreteManaWindowRoot(root: EvaluatedRoot): boolean {
  return (
    advisorRootFamily(root) === TurnPlanFamily.ManaTempo &&
    root.sameTurnScoreWindowValue > 0 &&
    root.sameTurnScoreWindowValue <= 1 &&
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

function isWhiteSpiritProgressWindowPair(
  game: MonsGame,
  spirit: EvaluatedRoot,
  mana: EvaluatedRoot,
  config: AutomoveConfig,
): boolean {
  return (
    productionEnabled(config) &&
    game.activeColor === Color.White &&
    game.turnNumber >= 5 &&
    game.monsMovesCount === 0 &&
    game.playerCanUseAction() &&
    game.playerCanMoveMana() &&
    advisorRootFamily(spirit) === TurnPlanFamily.SpiritImpact &&
    !spirit.spiritSameTurnScoreSetupNow &&
    !spirit.spiritOwnManaSetupNow &&
    hasProgressSurface(spirit) &&
    rootIsNonTactical(spirit) &&
    spirit.sameTurnScoreWindowValue === 0 &&
    isNonConcreteManaWindowRoot(mana) &&
    spirit.ownDrainerVulnerable === mana.ownDrainerVulnerable &&
    spirit.ownDrainerWalkVulnerable === mana.ownDrainerWalkVulnerable
  );
}

function whiteWindowProgressCompetitionOverride(
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
  if (approved === undefined || !isNonConcreteManaWindowRoot(approved)) {
    return undefined;
  }
  const exact = exactOpportunityContext(execution, game, game.activeColor);
  const weakWindow =
    exact.delta.sameTurnScoreWindowValue <= 1 &&
    exact.delta.opponentWindowDenyGain <= 1;
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
  const approvedUtility = rootUtility(execution, game, approved, perspective, config);
  const approvedFollowup = spiritFollowupFloorScore(
    execution,
    approved.game,
    perspective,
    config,
  );
  const snapshot = memoizedByIndex(
    (index) => {
      const root = roots[index];
      return root === undefined
        ? undefined
        : rootReplyRiskSnapshot(execution, root.game, perspective, config, replyLimit);
    },
    new Map([[approvedIndex, approvedSnapshot]]),
  );
  const followup = memoizedByIndex(
    (index) => {
      const root = roots[index];
      return root === undefined
        ? undefined
        : spiritFollowupFloorScore(execution, root.game, perspective, config);
    },
    new Map([[approvedIndex, approvedFollowup]]),
  );
  let bestIndex: number | undefined;
  for (const index of selectionIndices) {
    if (index === approvedIndex) continue;
    const challenger = roots[index];
    if (
      challenger === undefined ||
      !isWhiteSpiritProgressWindowPair(game, challenger, approved, config) ||
      challenger.rootRank > approved.rootRank + 8
    ) {
      continue;
    }
    const challengerSnapshot = snapshot(index);
    const challengerFollowup = followup(index);
    if (challengerSnapshot === undefined || challengerFollowup === undefined) continue;
    const progressBetter = rootProgressOrSetupBetter(challenger, approved);
    const weakCompetition =
      weakWindow &&
      progressBetter &&
      challenger.score >= saturatingScoreSubtract(approved.score, 32);
    const utilityCompetition =
      utilityCompetes(
        rootUtility(execution, game, challenger, perspective, config),
        approvedUtility,
      ) || progressBetter;
    if (
      challengerSnapshot.allowsImmediateOpponentWin ||
      challengerSnapshot.opponentReachesMatchPoint ||
      (!weakCompetition && !utilityCompetition) ||
      (!weakCompetition &&
        saturatingScoreAdd(challengerSnapshot.worstReplyScore, 192) <
          approvedSnapshot.worstReplyScore) ||
      (!weakCompetition &&
        saturatingScoreAdd(challengerFollowup, 32) < approvedFollowup)
    ) {
      continue;
    }
    if (bestIndex === undefined) bestIndex = index;
    else {
      const currentSnapshot = snapshot(bestIndex);
      const currentFollowup = followup(bestIndex);
      if (
        currentSnapshot === undefined ||
        currentFollowup === undefined ||
        challengerFollowup > currentFollowup ||
        (challengerFollowup === currentFollowup &&
          challengerSnapshot.worstReplyScore > currentSnapshot.worstReplyScore) ||
        (challengerFollowup === currentFollowup &&
          challengerSnapshot.worstReplyScore === currentSnapshot.worstReplyScore &&
          compareRankedEvaluatedRootIndices(roots, index, bestIndex) < 0)
      ) {
        bestIndex = index;
      }
    }
  }
  return bestIndex;
}

function whiteActionManaClusterOverride(
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
      advisorRootFamily(challenger) === TurnPlanFamily.ManaTempo &&
      !challenger.spiritDevelopment &&
      !challenger.spiritSameTurnScoreSetupNow &&
      !challenger.spiritOwnManaSetupNow &&
      rootIsNonTactical(challenger) &&
      challenger.sameTurnScoreWindowValue === 0 &&
      !advisorRootIsUnsafe(challenger) &&
      sameFirstInput(challenger.inputs, approved.inputs) &&
      challenger.ownDrainerVulnerable === approved.ownDrainerVulnerable &&
      challenger.ownDrainerWalkVulnerable === approved.ownDrainerWalkVulnerable &&
      challenger.safeSupermanaProgressSteps <= approved.safeSupermanaProgressSteps &&
      challenger.safeOpponentManaProgressSteps <=
        approved.safeOpponentManaProgressSteps &&
      challenger.score >= approved.score,
    (left, right) => {
      const leftRoot = roots[left];
      const rightRoot = roots[right];
      if (leftRoot === undefined || rightRoot === undefined) return left - right;
      if (leftRoot.score !== rightRoot.score)
        return leftRoot.score > rightRoot.score ? -1 : 1;
      return compareRankedEvaluatedRootIndices(roots, left, right);
    },
  );
}

function whiteFollowupManaOverride(
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
    game.turnNumber !== 3 ||
    game.monsMovesCount < 2 ||
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
  const approvedUtility = rootUtility(execution, game, approved, perspective, config);
  return bestOverrideIndex(
    roots,
    roots.map((_root, index) => index),
    (root, index) => {
      if (
        index === approvedIndex ||
        advisorRootFamily(root) !== TurnPlanFamily.ManaTempo ||
        root.spiritDevelopment ||
        root.spiritSameTurnScoreSetupNow ||
        root.spiritOwnManaSetupNow ||
        root.ownDrainerVulnerable ||
        root.ownDrainerWalkVulnerable ||
        !rootIsNonTactical(root) ||
        root.sameTurnScoreWindowValue !== 0 ||
        advisorRootIsUnsafe(root) ||
        root.rootRank > approved.rootRank + 2 ||
        root.safeSupermanaProgressSteps > approved.safeSupermanaProgressSteps ||
        root.safeOpponentManaProgressSteps > approved.safeOpponentManaProgressSteps ||
        root.scorePathBestSteps > approved.scorePathBestSteps
      ) {
        return false;
      }
      return (
        compareUtilityPrimaryAxes(
          rootUtility(execution, game, root, perspective, config),
          approvedUtility,
        ) > 0 || root.score >= saturatingScoreAdd(approved.score, 512)
      );
    },
    (left, right) => compareRankedEvaluatedRootIndices(roots, left, right),
  );
}

export {
  isNonConcreteManaWindowRoot,
  whiteActionManaClusterOverride,
  whiteFollowupManaOverride,
  whiteManaCompetitionOverride,
  whiteNoActionSafeProgressManaOverride,
  whiteWindowProgressCompetitionOverride,
};
