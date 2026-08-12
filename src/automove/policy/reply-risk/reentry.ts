import { Color } from "../../../api/types.js";
import type { MonsGame } from "../../../engine/game/mons-game.js";
import type { AutomoveExecutionContext } from "../../core/execution-context.js";
import { saturatingScoreSubtract } from "../../core/score-math.js";
import { rootFamily } from "../../root/family.js";
import type { EvaluatedRoot } from "../../root/types.js";
import { productionEnabled, rootIsUnsafe as isUnsafe } from "../../config/types.js";
import type { AutomoveConfig } from "../../config/types.js";
import { TurnPlanFamily } from "../../turn/model.js";
import {
  compareUtilityPrimaryAxes,
  utilityStrictlyDominatesOverrideAxes,
} from "../../turn/ordering.js";
import type { TurnUtility } from "../../turn/model.js";
import { turnEngineRootPlanUtility } from "./projection.js";
import { rankedRootOrder, sameNonTacticalProgressLane } from "./shortlist.js";

export function isProductionModeWhiteManaSiblingPair(
  candidate: EvaluatedRoot,
  incumbent: EvaluatedRoot,
  config: AutomoveConfig,
): boolean {
  return (
    productionEnabled(config) &&
    rootFamily(candidate) === TurnPlanFamily.ManaTempo &&
    rootFamily(incumbent) === TurnPlanFamily.ManaTempo &&
    candidate.efficiency === incumbent.efficiency &&
    !candidate.ownDrainerVulnerable &&
    !incumbent.ownDrainerVulnerable &&
    !candidate.ownDrainerWalkVulnerable &&
    !incumbent.ownDrainerWalkVulnerable &&
    !isUnsafe(candidate) &&
    !isUnsafe(incumbent) &&
    sameNonTacticalProgressLane(candidate, incumbent)
  );
}

export function productionWhiteTurnFourManaSiblingReentry(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  evaluations: readonly EvaluatedRoot[],
  shortlist: readonly number[],
  perspective: Color,
  config: AutomoveConfig,
): number | undefined {
  if (
    !productionEnabled(config) ||
    game.activeColor !== Color.White ||
    game.turnNumber !== 3 ||
    game.monsMovesCount < 4
  ) {
    return undefined;
  }
  const anchorIndex = [...shortlist]
    .filter((index) => {
      const root = evaluations[index];
      return (
        root !== undefined &&
        rootFamily(root) === TurnPlanFamily.ManaTempo &&
        !root.ownDrainerVulnerable &&
        !root.ownDrainerWalkVulnerable &&
        !isUnsafe(root) &&
        !root.manaHandoffToOpponent &&
        !root.hasRoundtrip &&
        !root.winsImmediately &&
        !root.attacksOpponentDrainer &&
        root.sameTurnScoreWindowValue === 0 &&
        !root.scoresSupermanaThisTurn &&
        !root.scoresOpponentManaThisTurn &&
        !root.safeSupermanaPickupNow &&
        !root.safeOpponentManaPickupNow
      );
    })
    .sort((left, right) => -rankedRootOrder(evaluations, left, right))[0];
  if (anchorIndex === undefined) return undefined;
  const anchor = evaluations[anchorIndex];
  if (anchor === undefined) return undefined;
  const anchorUtility = turnEngineRootPlanUtility(
    execution,
    game,
    anchor,
    perspective,
    config,
    TurnPlanFamily.ManaTempo,
  );
  let bestIndex: number | undefined;
  let bestUtility: TurnUtility | undefined;
  let bestIsDominance = false;
  evaluations.forEach((candidate, index) => {
    const sameLaneNearBest =
      isProductionModeWhiteManaSiblingPair(candidate, anchor, config) &&
      saturatingScoreSubtract(anchor.score, candidate.score) <= 24 &&
      Math.abs(anchor.rootRank - candidate.rootRank) <= 4 &&
      candidate.rootRank < anchor.rootRank;
    if (
      shortlist.includes(index) ||
      rootFamily(candidate) !== TurnPlanFamily.ManaTempo ||
      candidate.ownDrainerVulnerable ||
      candidate.ownDrainerWalkVulnerable ||
      isUnsafe(candidate) ||
      candidate.manaHandoffToOpponent ||
      candidate.hasRoundtrip ||
      candidate.winsImmediately ||
      candidate.attacksOpponentDrainer ||
      candidate.sameTurnScoreWindowValue > 0 ||
      candidate.scoresSupermanaThisTurn ||
      candidate.scoresOpponentManaThisTurn ||
      candidate.safeSupermanaPickupNow ||
      candidate.safeOpponentManaPickupNow ||
      (!sameLaneNearBest &&
        candidate.score < saturatingScoreSubtract(anchor.score, 96)) ||
      (candidate.rootRank >= anchor.rootRank && !sameLaneNearBest)
    ) {
      return;
    }
    const utility = turnEngineRootPlanUtility(
      execution,
      game,
      candidate,
      perspective,
      config,
      TurnPlanFamily.ManaTempo,
    );
    const utilityOrder = compareUtilityPrimaryAxes(utility, anchorUtility);
    const dominance =
      utilityOrder > 0 || utilityStrictlyDominatesOverrideAxes(utility, anchorUtility);
    if (!dominance && !sameLaneNearBest) return;
    const currentIndex = bestIndex;
    const currentBestUtility = bestUtility;
    let replace = currentIndex === undefined || currentBestUtility === undefined;
    if (currentIndex !== undefined && currentBestUtility !== undefined) {
      const current = evaluations[currentIndex];
      if (current === undefined) {
        replace = true;
      } else {
        const currentSameLane =
          isProductionModeWhiteManaSiblingPair(current, anchor, config) &&
          saturatingScoreSubtract(anchor.score, current.score) <= 24 &&
          Math.abs(anchor.rootRank - current.rootRank) <= 4 &&
          current.rootRank < anchor.rootRank;
        if (dominance !== bestIsDominance) {
          replace = dominance;
        } else if (dominance) {
          const currentOrder = compareUtilityPrimaryAxes(utility, currentBestUtility);
          replace =
            currentOrder > 0 ||
            (currentOrder === 0 &&
              rankedRootOrder(evaluations, index, currentIndex) > 0);
        } else if (sameLaneNearBest && currentSameLane) {
          replace =
            candidate.rootRank < current.rootRank ||
            (candidate.rootRank === current.rootRank &&
              rankedRootOrder(evaluations, index, currentIndex) > 0);
        } else {
          replace = sameLaneNearBest;
        }
      }
    }
    if (replace) {
      bestIndex = index;
      bestUtility = utility;
      bestIsDominance = dominance;
    }
  });
  return bestIndex;
}
