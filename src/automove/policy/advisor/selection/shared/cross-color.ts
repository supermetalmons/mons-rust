import { Color } from "../../../../../api/types.js";
import { MonsGame } from "../../../../../engine/game/mons-game.js";
import { exactOpportunityContext } from "../../../../exact/turn-opportunity.js";
import type { AutomoveExecutionContext } from "../../../../core/execution-context.js";
import { rootProgressOrSetupBetter } from "../../../reply-risk/shortlist.js";
import { rootReplyRiskSnapshot } from "../../../reply-risk/snapshot.js";
import { spiritFollowupFloorScore } from "../../../reply-risk/projection.js";
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
import { compareUtilityPrimaryAxes } from "../../../../turn/ordering.js";
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

function earlySameLaneHigherScoreOverride(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  roots: readonly EvaluatedRoot[],
  selectionIndices: readonly number[],
  approvedIndex: number,
  config: AutomoveConfig,
): number | undefined {
  const supportedState =
    (game.activeColor === Color.Black && game.turnNumber === 2) ||
    (game.activeColor === Color.White && game.turnNumber === 3);
  if (
    !productionEnabled(config) ||
    !game.playerCanMoveMana() ||
    roots.length === 0 ||
    !supportedState ||
    !exactContextIsQuiet(execution, game)
  ) {
    return undefined;
  }
  const approved = roots[approvedIndex];
  if (approved === undefined) return undefined;
  const approvedFamily = advisorRootFamily(approved);
  if (
    !isTurnPlanFamilyOneOf(
      approvedFamily,
      TurnPlanFamily.ManaTempo,
      TurnPlanFamily.SpiritImpact,
    ) ||
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
      advisorRootFamily(challenger) === approvedFamily &&
      sameFirstInput(challenger.inputs, approved.inputs) &&
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
      challenger.spiritDevelopment === approved.spiritDevelopment &&
      challenger.spiritSameTurnScoreSetupNow === approved.spiritSameTurnScoreSetupNow &&
      challenger.spiritOwnManaSetupNow === approved.spiritOwnManaSetupNow &&
      challenger.supermanaProgress === approved.supermanaProgress &&
      challenger.opponentManaProgress === approved.opponentManaProgress &&
      challenger.safeSupermanaProgressSteps === approved.safeSupermanaProgressSteps &&
      challenger.safeOpponentManaProgressSteps ===
        approved.safeOpponentManaProgressSteps &&
      challenger.scorePathBestSteps === approved.scorePathBestSteps &&
      challenger.spiritSetupGain === approved.spiritSetupGain &&
      challenger.ownDrainerVulnerable === approved.ownDrainerVulnerable &&
      challenger.ownDrainerWalkVulnerable === approved.ownDrainerWalkVulnerable &&
      challenger.score >= saturatingScoreAdd(approved.score, 8) &&
      challenger.rootRank <= approved.rootRank + 8,
    (left, right) => {
      const leftRoot = roots[left];
      const rightRoot = roots[right];
      if (leftRoot === undefined || rightRoot === undefined) return left - right;
      if (leftRoot.score !== rightRoot.score)
        return leftRoot.score > rightRoot.score ? -1 : 1;
      if (leftRoot.rootRank !== rightRoot.rootRank)
        return leftRoot.rootRank > rightRoot.rootRank ? -1 : 1;
      return -compareRankedEvaluatedRootIndices(roots, left, right);
    },
  );
}

function whiteTurnFiveWeakWindowSetupOverride(
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
    game.turnNumber !== 5 ||
    game.monsMovesCount !== 0 ||
    !game.playerCanUseAction() ||
    !game.playerCanMoveMana() ||
    roots.length === 0
  ) {
    return undefined;
  }
  const exact = exactOpportunityContext(execution, game, game.activeColor);
  const weakWindow =
    exact.delta.sameTurnScoreWindowValue <= 1 &&
    exact.delta.opponentWindowDenyGain <= 1 &&
    !exact.delta.drainerAttackAvailable &&
    (exact.delta.sameTurnScoreWindowValue > 0 ||
      exact.delta.opponentWindowDenyGain > 0);
  if (!weakWindow) return undefined;
  const approved = roots[approvedIndex];
  if (
    approved === undefined ||
    !isTurnPlanFamilyOneOf(
      advisorRootFamily(approved),
      TurnPlanFamily.SafeSupermanaProgress,
      TurnPlanFamily.SafeOpponentManaProgress,
    ) ||
    advisorRootIsUnsafe(approved) ||
    approved.winsImmediately ||
    approved.attacksOpponentDrainer ||
    approved.spiritDevelopment ||
    approved.spiritSameTurnScoreSetupNow ||
    approved.spiritOwnManaSetupNow ||
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
      hasProgressSurface(challenger) &&
      !advisorRootIsUnsafe(challenger) &&
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
      saturatingScoreSubtract(approved.score, challenger.score) <= 96 &&
      challenger.spiritSetupGain >= saturatingScoreAdd(approved.spiritSetupGain, 48) &&
      challenger.safeSupermanaProgressSteps <=
        approved.safeSupermanaProgressSteps + 1 &&
      challenger.safeOpponentManaProgressSteps <=
        approved.safeOpponentManaProgressSteps + 1 &&
      challenger.rootRank <= approved.rootRank + 4,
    (left, right) => compareRootRankThenRanked(roots, left, right),
  );
}

function replyLimitForRoots(shortlistLength: number, config: AutomoveConfig): number {
  const length = Math.max(shortlistLength, 1);
  const rootNodeBudget = Math.max(
    Math.trunc(
      (config.budget.maxVisitedNodes * Math.max(config.replyRisk.nodeShareBp, 0)) /
        10_000,
    ),
    length,
    1,
  );
  return Math.min(
    Math.max(Math.trunc(rootNodeBudget / length), 1),
    Math.max(config.replyRisk.replyLimit, 1),
  );
}

function blackSetupProgressCompetitionOverride(
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
    game.activeColor !== Color.Black ||
    game.turnNumber < 6 ||
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
    return rootUtility(execution, game, root ?? approved, perspective, config);
  });
  const snapshot = memoizedByIndex(
    (index) => {
      const root = roots[index];
      return rootReplyRiskSnapshot(
        execution,
        root?.game ?? approved.game,
        perspective,
        config,
        replyLimit,
      );
    },
    new Map([[approvedIndex, approvedSnapshot]]),
  );
  const followup = memoizedByIndex((index) => {
    const root = roots[index];
    return spiritFollowupFloorScore(
      execution,
      root?.game ?? approved.game,
      perspective,
      config,
    );
  });
  const approvedUtility = utility(approvedIndex);
  const approvedFollowup = followup(approvedIndex);
  const exact = exactOpportunityContext(execution, game, game.activeColor);
  const weakWindowContext =
    exact.delta.sameTurnScoreWindowValue <= 1 &&
    exact.delta.opponentWindowDenyGain <= 1;
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
      challenger.winsImmediately ||
      challenger.attacksOpponentDrainer ||
      challenger.sameTurnScoreWindowValue > 0 ||
      challenger.scoresSupermanaThisTurn ||
      challenger.scoresOpponentManaThisTurn ||
      challenger.safeSupermanaPickupNow ||
      challenger.safeOpponentManaPickupNow ||
      challenger.manaHandoffToOpponent ||
      challenger.hasRoundtrip ||
      advisorRootIsUnsafe(challenger) ||
      !sameFirstInput(challenger.inputs, approved.inputs) ||
      challenger.supermanaProgress !== approved.supermanaProgress ||
      challenger.opponentManaProgress !== approved.opponentManaProgress ||
      challenger.ownDrainerVulnerable !== approved.ownDrainerVulnerable ||
      challenger.ownDrainerWalkVulnerable !== approved.ownDrainerWalkVulnerable
    ) {
      continue;
    }
    const challengerUtility = utility(index);
    const challengerSnapshot = snapshot(index);
    const challengerFollowup = followup(index);
    const utilityCompetition =
      utilityCompetes(challengerUtility, approvedUtility) ||
      rootProgressOrSetupBetter(challenger, approved);
    const weakContextCompetition =
      weakWindowContext &&
      challenger.spiritSetupGain > approved.spiritSetupGain &&
      challenger.score >= saturatingScoreSubtract(approved.score, 512);
    if (
      (!weakContextCompetition && !utilityCompetition) ||
      challengerSnapshot.allowsImmediateOpponentWin ||
      challengerSnapshot.opponentReachesMatchPoint ||
      (!weakContextCompetition &&
        saturatingScoreAdd(challengerSnapshot.worstReplyScore, 320) <
          approvedSnapshot.worstReplyScore) ||
      (!weakContextCompetition &&
        saturatingScoreAdd(challengerFollowup, 32) < approvedFollowup)
    ) {
      continue;
    }
    if (bestIndex === undefined) {
      bestIndex = index;
      continue;
    }
    const current = roots[bestIndex];
    if (current === undefined) {
      bestIndex = index;
      continue;
    }
    let replace: boolean;
    if (weakWindowContext) {
      replace =
        challenger.score > current.score ||
        (challenger.score === current.score &&
          (challenger.spiritSetupGain > current.spiritSetupGain ||
            (challenger.spiritSetupGain === current.spiritSetupGain &&
              compareRankedEvaluatedRootIndices(roots, index, bestIndex) < 0)));
    } else {
      const utilityOrder = compareUtilityPrimaryAxes(
        challengerUtility,
        utility(bestIndex),
      );
      const currentSnapshot = snapshot(bestIndex);
      const currentFollowup = followup(bestIndex);
      if (utilityOrder > 0) replace = true;
      else if (utilityOrder !== 0) replace = false;
      else if (challengerSnapshot.worstReplyScore !== currentSnapshot.worstReplyScore) {
        replace = challengerSnapshot.worstReplyScore > currentSnapshot.worstReplyScore;
      } else if (challengerFollowup !== currentFollowup) {
        replace = challengerFollowup > currentFollowup;
      } else if (challenger.spiritSetupGain !== current.spiritSetupGain) {
        replace = challenger.spiritSetupGain > current.spiritSetupGain;
      } else if (challenger.score !== current.score) {
        replace = challenger.score > current.score;
      } else {
        replace = compareRankedEvaluatedRootIndices(roots, index, bestIndex) < 0;
      }
    }
    if (replace) bestIndex = index;
  }
  return bestIndex;
}

export {
  blackSetupProgressCompetitionOverride,
  earlySameLaneHigherScoreOverride,
  replyLimitForRoots,
  whiteTurnFiveWeakWindowSetupOverride,
};
