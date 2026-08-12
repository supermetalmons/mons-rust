import { Color } from "../../../../../api/types.js";
import { MonsGame } from "../../../../../engine/game/mons-game.js";
import { exactOpportunityContext } from "../../../../exact/turn-opportunity.js";
import type { AutomoveExecutionContext } from "../../../../core/execution-context.js";
import {
  blackPlainSpiritFollowupReplyOrder,
  isProductionModeBlackPlainSpiritFollowupSetupPair,
} from "../../../reply-risk/followup-ordering.js";
import { rootProgressOrSetupBetter } from "../../../reply-risk/shortlist.js";
import { rootReplyRiskSnapshot } from "../../../reply-risk/snapshot.js";
import {
  spiritFollowupFloorScore,
  turnEngineRootPlanUtility,
} from "../../../reply-risk/projection.js";
import { rootFamily as advisorRootFamily } from "../../../../root/family.js";
import { compareRankedEvaluatedRootIndices } from "../../../../root/evaluated-ordering.js";
import {
  MIN_SCORE,
  saturatingScoreAdd,
  saturatingScoreSubtract,
} from "../../../../core/score-math.js";
import type { EvaluatedRoot } from "../../../../root/types.js";
import {
  hasProgressSurface,
  productionEnabled,
  rootIsUnsafe as advisorRootIsUnsafe,
} from "../../../../config/types.js";
import type { AutomoveConfig } from "../../../../config/types.js";
import { TurnPlanFamily } from "../../../../turn/model.js";
import { spiritSetupCompetes } from "../../competition.js";
import { rootIsNonTactical } from "./baseline.js";
import { replyLimitForRoots } from "../shared/cross-color.js";
import { isNonConcreteManaWindowRoot } from "../white/mana-progress.js";
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

function isLateBlackActionManaTurnStart(game: MonsGame): boolean {
  return (
    game.activeColor === Color.Black &&
    game.turnNumber >= 8 &&
    game.monsMovesCount === 0 &&
    game.playerCanUseAction() &&
    game.playerCanMoveMana()
  );
}

function isLateBlackSpiritFollowupManaPair(
  game: MonsGame,
  candidate: EvaluatedRoot,
  incumbent: EvaluatedRoot,
  config: AutomoveConfig,
): boolean {
  return (
    productionEnabled(config) &&
    isLateBlackActionManaTurnStart(game) &&
    advisorRootFamily(candidate) === TurnPlanFamily.SpiritImpact &&
    candidate.spiritDevelopment &&
    !candidate.spiritSameTurnScoreSetupNow &&
    !candidate.spiritOwnManaSetupNow &&
    candidate.sameTurnScoreWindowValue === 0 &&
    rootIsNonTactical(candidate) &&
    !advisorRootIsUnsafe(candidate) &&
    advisorRootFamily(incumbent) === TurnPlanFamily.ManaTempo &&
    incumbent.sameTurnScoreWindowValue === 0 &&
    rootIsNonTactical(incumbent) &&
    !advisorRootIsUnsafe(incumbent) &&
    sameFirstInput(candidate.inputs, incumbent.inputs)
  );
}

function blackLateFollowupCompetitionOverride(
  game: MonsGame,
  roots: readonly EvaluatedRoot[],
  selectionIndices: readonly number[],
  approvedIndex: number,
  config: AutomoveConfig,
): number | undefined {
  if (
    !productionEnabled(config) ||
    !isLateBlackActionManaTurnStart(game) ||
    roots.length === 0
  ) {
    return undefined;
  }
  const approved = roots[approvedIndex];
  if (
    approved === undefined ||
    advisorRootFamily(approved) !== TurnPlanFamily.ManaTempo
  ) {
    return undefined;
  }
  return bestOverrideIndex(
    roots,
    selectionIndices,
    (challenger, index) =>
      index !== approvedIndex &&
      isLateBlackSpiritFollowupManaPair(game, challenger, approved, config) &&
      saturatingScoreSubtract(approved.score, challenger.score) <= 512 &&
      challenger.rootRank < approved.rootRank &&
      Math.abs(approved.rootRank - challenger.rootRank) <= 16,
    (left, right) => compareRootRankThenRanked(roots, left, right),
  );
}

function blackLateReplyRiskSetupOverride(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  roots: readonly EvaluatedRoot[],
  replyRiskShortlist: readonly number[],
  approvedIndex: number,
  config: AutomoveConfig,
): number | undefined {
  if (
    !productionEnabled(config) ||
    !isLateBlackActionManaTurnStart(game) ||
    replyRiskShortlist.length === 0 ||
    roots.length === 0 ||
    !exactContextIsQuiet(execution, game)
  ) {
    return undefined;
  }
  const approved = roots[approvedIndex];
  if (
    approved === undefined ||
    advisorRootFamily(approved) !== TurnPlanFamily.ManaTempo ||
    approved.spiritDevelopment ||
    approved.spiritSameTurnScoreSetupNow ||
    approved.spiritOwnManaSetupNow ||
    hasProgressSurface(approved) ||
    !rootIsNonTactical(approved) ||
    approved.sameTurnScoreWindowValue > 0 ||
    advisorRootIsUnsafe(approved) ||
    approved.ownDrainerVulnerable ||
    approved.ownDrainerWalkVulnerable
  ) {
    return undefined;
  }
  return bestOverrideIndex(
    roots,
    replyRiskShortlist,
    (challenger, index) =>
      index !== approvedIndex &&
      advisorRootFamily(challenger) === TurnPlanFamily.SpiritImpact &&
      challenger.spiritOwnManaSetupNow &&
      !challenger.spiritSameTurnScoreSetupNow &&
      !hasProgressSurface(challenger) &&
      rootIsNonTactical(challenger) &&
      challenger.sameTurnScoreWindowValue === 0 &&
      !advisorRootIsUnsafe(challenger) &&
      !challenger.ownDrainerVulnerable &&
      !challenger.ownDrainerWalkVulnerable &&
      challenger.spiritSetupGain >= saturatingScoreAdd(approved.spiritSetupGain, 64) &&
      saturatingScoreSubtract(approved.score, challenger.score) <= 2_048 &&
      challenger.safeSupermanaProgressSteps === approved.safeSupermanaProgressSteps &&
      challenger.safeOpponentManaProgressSteps ===
        approved.safeOpponentManaProgressSteps &&
      challenger.scorePathBestSteps === approved.scorePathBestSteps &&
      challenger.rootRank < approved.rootRank &&
      Math.abs(approved.rootRank - challenger.rootRank) <= 4,
    (left, right) => {
      const leftRoot = roots[left];
      const rightRoot = roots[right];
      if (leftRoot === undefined || rightRoot === undefined) return left - right;
      if (leftRoot.rootRank !== rightRoot.rootRank)
        return leftRoot.rootRank < rightRoot.rootRank ? -1 : 1;
      if (leftRoot.spiritSetupGain !== rightRoot.spiritSetupGain)
        return leftRoot.spiritSetupGain > rightRoot.spiritSetupGain ? -1 : 1;
      return compareRankedEvaluatedRootIndices(roots, left, right);
    },
  );
}

function blackLateWeakWindowSafeProgressSetupOverride(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  roots: readonly EvaluatedRoot[],
  replyRiskShortlist: readonly number[],
  approvedIndex: number,
  perspective: Color,
  config: AutomoveConfig,
): number | undefined {
  if (
    !productionEnabled(config) ||
    !isLateBlackActionManaTurnStart(game) ||
    replyRiskShortlist.length === 0 ||
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
  if (approved === undefined) return undefined;
  const approvedFamily = advisorRootFamily(approved);
  if (
    !isTurnPlanFamilyOneOf(
      approvedFamily,
      TurnPlanFamily.SafeSupermanaProgress,
      TurnPlanFamily.SafeOpponentManaProgress,
    ) ||
    !hasProgressSurface(approved) ||
    approved.spiritDevelopment ||
    approved.spiritSameTurnScoreSetupNow ||
    approved.spiritOwnManaSetupNow ||
    !rootIsNonTactical(approved) ||
    approved.sameTurnScoreWindowValue > 0 ||
    advisorRootIsUnsafe(approved) ||
    approved.ownDrainerVulnerable ||
    approved.ownDrainerWalkVulnerable
  ) {
    return undefined;
  }
  const approvedUtility = rootUtility(execution, game, approved, perspective, config);
  return bestOverrideIndex(
    roots,
    replyRiskShortlist,
    (challenger, index) =>
      index !== approvedIndex &&
      advisorRootFamily(challenger) === TurnPlanFamily.SpiritImpact &&
      challenger.spiritOwnManaSetupNow &&
      !challenger.spiritSameTurnScoreSetupNow &&
      hasProgressSurface(challenger) &&
      rootIsNonTactical(challenger) &&
      challenger.sameTurnScoreWindowValue === 0 &&
      challenger.manaHandoffToOpponent === approved.manaHandoffToOpponent &&
      challenger.hasRoundtrip === approved.hasRoundtrip &&
      !advisorRootIsUnsafe(challenger) &&
      challenger.ownDrainerVulnerable === approved.ownDrainerVulnerable &&
      challenger.ownDrainerWalkVulnerable === approved.ownDrainerWalkVulnerable &&
      challenger.supermanaProgress === approved.supermanaProgress &&
      challenger.opponentManaProgress === approved.opponentManaProgress &&
      utilityCompetes(
        rootUtility(execution, game, challenger, perspective, config),
        approvedUtility,
      ) &&
      saturatingScoreSubtract(approved.score, challenger.score) <= 32 &&
      challenger.spiritSetupGain >= saturatingScoreAdd(approved.spiritSetupGain, 64) &&
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

function isBlackSpiritProgressWindowPair(
  game: MonsGame,
  spirit: EvaluatedRoot,
  mana: EvaluatedRoot,
  config: AutomoveConfig,
): boolean {
  return (
    productionEnabled(config) &&
    game.activeColor === Color.Black &&
    game.turnNumber <= 4 &&
    advisorRootFamily(spirit) === TurnPlanFamily.SpiritImpact &&
    !spirit.spiritSameTurnScoreSetupNow &&
    !spirit.spiritOwnManaSetupNow &&
    hasProgressSurface(spirit) &&
    !spirit.winsImmediately &&
    !spirit.attacksOpponentDrainer &&
    !spirit.scoresSupermanaThisTurn &&
    !spirit.scoresOpponentManaThisTurn &&
    !spirit.safeSupermanaPickupNow &&
    !spirit.safeOpponentManaPickupNow &&
    !spirit.manaHandoffToOpponent &&
    !spirit.hasRoundtrip &&
    isNonConcreteManaWindowRoot(mana) &&
    spirit.ownDrainerVulnerable === mana.ownDrainerVulnerable &&
    spirit.ownDrainerWalkVulnerable === mana.ownDrainerWalkVulnerable
  );
}

function blackFamilyCompetitionOverride(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  roots: readonly EvaluatedRoot[],
  replyRiskShortlist: readonly number[],
  selectionIndices: readonly number[],
  approvedIndex: number,
  perspective: Color,
  config: AutomoveConfig,
): number | undefined {
  if (
    !productionEnabled(config) ||
    game.activeColor !== Color.Black ||
    roots.length === 0
  ) {
    return undefined;
  }
  const approved = roots[approvedIndex];
  if (
    approved === undefined ||
    advisorRootFamily(approved) !== TurnPlanFamily.ManaTempo ||
    (approvedIndex > 1 &&
      !(game.turnNumber <= 2 && game.playerCanUseAction() && game.playerCanMoveMana()))
  ) {
    return undefined;
  }
  const approvedNonConcreteWindow = isNonConcreteManaWindowRoot(approved);
  if (
    approved.winsImmediately ||
    approved.attacksOpponentDrainer ||
    (approved.sameTurnScoreWindowValue > 0 && !approvedNonConcreteWindow) ||
    approved.scoresSupermanaThisTurn ||
    approved.scoresOpponentManaThisTurn ||
    approved.safeSupermanaPickupNow ||
    approved.safeOpponentManaPickupNow
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
  const approvedUnsafe = advisorRootIsUnsafe(approved);
  const approvedProgress = hasProgressSurface(approved);
  const replyLimit = replyLimitForRoots(replyRiskShortlist.length, config);
  const snapshot = memoizedByIndex((index) => {
    const root = roots[index];
    return root === undefined
      ? undefined
      : rootReplyRiskSnapshot(execution, root.game, perspective, config, replyLimit);
  });
  const followups = new Map<number, number>();
  const followup = memoizedByIndex((index) => {
    const root = roots[index];
    return root === undefined
      ? undefined
      : spiritFollowupFloorScore(execution, root.game, perspective, config);
  }, followups);
  const qualifyingIndices: number[] = [];
  for (const index of selectionIndices) {
    if (index === approvedIndex) continue;
    const candidate = roots[index];
    if (candidate === undefined) continue;
    const family = advisorRootFamily(candidate);
    const concreteSpiritSetup =
      candidate.spiritOwnManaSetupNow || candidate.spiritSameTurnScoreSetupNow;
    const spiritProgressFamily =
      family === TurnPlanFamily.SpiritImpact &&
      !concreteSpiritSetup &&
      hasProgressSurface(candidate);
    const progressFamily = isTurnPlanFamilyOneOf(
      family,
      TurnPlanFamily.SafeSupermanaProgress,
      TurnPlanFamily.SafeOpponentManaProgress,
    );
    const progressFamilyAllowed = progressFamily && !approvedNonConcreteWindow;
    const concreteSpiritSetupAllowed =
      concreteSpiritSetup && !approvedNonConcreteWindow && game.turnNumber <= 4;
    if (
      !concreteSpiritSetupAllowed &&
      !progressFamilyAllowed &&
      !spiritProgressFamily
    ) {
      continue;
    }
    const candidateUtility = rootUtility(
      execution,
      game,
      candidate,
      perspective,
      config,
    );
    const progress = hasProgressSurface(candidate);
    const progressBetter =
      rootProgressOrSetupBetter(candidate, approved) || (progress && !approvedProgress);
    const approvedSafeManaBlocksPlainSpiritProgress =
      family === TurnPlanFamily.SpiritImpact &&
      !approvedUnsafe &&
      !advisorRootIsUnsafe(candidate) &&
      advisorRootFamily(approved) === TurnPlanFamily.ManaTempo &&
      !approvedProgress &&
      !approved.spiritDevelopment &&
      !approved.spiritSameTurnScoreSetupNow &&
      !approved.spiritOwnManaSetupNow &&
      candidate.spiritDevelopment &&
      !candidate.spiritSameTurnScoreSetupNow &&
      !candidate.spiritOwnManaSetupNow &&
      progress &&
      !candidate.winsImmediately &&
      !candidate.attacksOpponentDrainer &&
      !candidate.scoresSupermanaThisTurn &&
      !candidate.scoresOpponentManaThisTurn &&
      !candidate.safeSupermanaPickupNow &&
      !candidate.safeOpponentManaPickupNow &&
      !candidate.manaHandoffToOpponent &&
      !candidate.hasRoundtrip &&
      candidate.sameTurnScoreWindowValue === 0 &&
      candidate.score <= approved.score;
    if (approvedSafeManaBlocksPlainSpiritProgress) continue;
    const earlyBlackProgressScoreOverride =
      progressFamilyAllowed &&
      game.turnNumber <= 6 &&
      approved.score < 0 &&
      candidate.score >= 0 &&
      progressBetter;
    let blackSpiritProgressWindowReplyOverride = false;
    if (
      spiritProgressFamily &&
      approvedNonConcreteWindow &&
      isBlackSpiritProgressWindowPair(game, candidate, approved, config) &&
      candidate.rootRank <= approved.rootRank + 8
    ) {
      const candidateSnapshot = snapshot(index);
      const approvedSnapshot = snapshot(approvedIndex);
      const candidateFollowup = followup(index);
      const approvedFollowup = followup(approvedIndex);
      blackSpiritProgressWindowReplyOverride =
        candidateSnapshot !== undefined &&
        approvedSnapshot !== undefined &&
        candidateFollowup !== undefined &&
        approvedFollowup !== undefined &&
        !candidateSnapshot.allowsImmediateOpponentWin &&
        !approvedSnapshot.allowsImmediateOpponentWin &&
        !candidateSnapshot.opponentReachesMatchPoint &&
        !approvedSnapshot.opponentReachesMatchPoint &&
        rootProgressOrSetupBetter(candidate, approved) &&
        saturatingScoreAdd(candidateSnapshot.worstReplyScore, 192) >=
          approvedSnapshot.worstReplyScore &&
        saturatingScoreAdd(candidateFollowup, 32) >= approvedFollowup;
    }
    if (
      !utilityCompetes(candidateUtility, approvedUtility) &&
      !earlyBlackProgressScoreOverride &&
      !blackSpiritProgressWindowReplyOverride
    ) {
      continue;
    }
    const candidateUnsafe = advisorRootIsUnsafe(candidate);
    if (
      candidateUnsafe &&
      !approvedUnsafe &&
      !concreteSpiritSetupAllowed &&
      !earlyBlackProgressScoreOverride &&
      !blackSpiritProgressWindowReplyOverride
    ) {
      continue;
    }
    const setupCompetes =
      concreteSpiritSetupAllowed &&
      spiritSetupCompetes(execution, game, candidate, approved, perspective, config);
    if (progressBetter || setupCompetes || blackSpiritProgressWindowReplyOverride) {
      qualifyingIndices.push(index);
    }
  }
  const mapped = qualifyingIndices.map((index) => {
    const candidate = roots[index];
    if (
      candidate === undefined ||
      !candidate.spiritOwnManaSetupNow ||
      candidate.spiritSameTurnScoreSetupNow
    ) {
      return index;
    }
    const candidateSnapshot = snapshot(index);
    if (candidateSnapshot === undefined) return index;
    const plainIndices = replyRiskShortlist.filter((plainIndex) => {
      const plain = roots[plainIndex];
      if (
        plainIndex === index ||
        plain === undefined ||
        !isProductionModeBlackPlainSpiritFollowupSetupPair(
          game,
          plain,
          candidate,
          config,
        )
      ) {
        return false;
      }
      const plainSnapshot = snapshot(plainIndex);
      return (
        plainSnapshot !== undefined &&
        (blackPlainSpiritFollowupReplyOrder(
          execution,
          game,
          roots,
          plainIndex,
          plainSnapshot,
          index,
          candidateSnapshot,
          perspective,
          config,
          followups,
        ) ?? 0) > 0
      );
    });
    return (
      plainIndices.sort((left, right) => {
        const leftRoot = roots[left];
        const rightRoot = roots[right];
        const leftSnapshot = snapshot(left);
        const rightSnapshot = snapshot(right);
        if (
          leftRoot === undefined ||
          rightRoot === undefined ||
          leftSnapshot === undefined ||
          rightSnapshot === undefined
        ) {
          return left - right;
        }
        if (leftSnapshot.worstReplyScore !== rightSnapshot.worstReplyScore) {
          return leftSnapshot.worstReplyScore > rightSnapshot.worstReplyScore ? -1 : 1;
        }
        const leftFollowup = followup(left) ?? MIN_SCORE;
        const rightFollowup = followup(right) ?? MIN_SCORE;
        if (leftFollowup !== rightFollowup)
          return leftFollowup > rightFollowup ? -1 : 1;
        if (leftRoot.score !== rightRoot.score)
          return leftRoot.score > rightRoot.score ? -1 : 1;
        return compareRankedEvaluatedRootIndices(roots, left, right);
      })[0] ?? index
    );
  });
  return mapped.sort((left, right) =>
    compareRankedEvaluatedRootIndices(roots, left, right),
  )[0];
}

export {
  blackFamilyCompetitionOverride,
  blackLateFollowupCompetitionOverride,
  blackLateReplyRiskSetupOverride,
  blackLateWeakWindowSafeProgressSetupOverride,
};
