import { Color } from "../../engine/domain.js";
import { MonsGame } from "../../engine/game.js";
import { exactOpportunityContext } from "../exact.js";
import type { AutomoveExecutionContext } from "../execution-context.js";
import {
  productionWhiteTurnFourManaSiblingReentry,
  sameNonTacticalProgressLane,
  spiritFollowupFloorScore,
  whiteSpiritFollowupSetupCompetition,
} from "../reply-risk.js";
import { rootFamily as advisorRootFamily } from "../root-family.js";
import { compareRankedEvaluatedRootIndices } from "../root-selector.js";
import {
  MIN_SCORE,
  saturatingScoreAdd,
  saturatingScoreSubtract,
} from "../score-math.js";
import type { EvaluatedRoot } from "../search.js";
import {
  hasConcreteScoreSurface,
  hasProgressSurface,
  isPlainSpiritDevelopmentRoot,
  productionEnabled,
  rootIsUnsafe as advisorRootIsUnsafe,
} from "../selector-types.js";
import type { AutomoveConfig } from "../selector-types.js";
import {
  TurnPlanFamily,
  compareUtilityPrimaryAxes,
  utilitySupportsTemporaryRiskRecovery,
} from "../turn-engine.js";
import {
  advisorRootIsSafe,
  memoizedByIndex,
  rootUtility,
  utilitiesEqual,
  utilityCompetes,
} from "./support.js";

function findScoredRepresentative(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  roots: readonly EvaluatedRoot[],
  orderedShortlist: readonly number[],
  perspective: Color,
  config: AutomoveConfig,
  predicate: (root: EvaluatedRoot) => boolean,
): number | undefined {
  const anchorIndex = orderedShortlist[0];
  if (anchorIndex === undefined) return undefined;
  if (
    orderedShortlist.some((index) => {
      const root = roots[index];
      return root !== undefined && predicate(root) && advisorRootIsSafe(root);
    })
  ) {
    return undefined;
  }
  const anchor = roots[anchorIndex];
  if (anchor === undefined) return undefined;
  const anchorUtility = rootUtility(
    execution,
    game,
    anchor,
    perspective,
    config,
  );
  return roots
    .map((_root, index) => index)
    .filter((index) => {
      const candidate = roots[index];
      return (
        candidate !== undefined &&
        predicate(candidate) &&
        advisorRootIsSafe(candidate) &&
        utilityCompetes(
          rootUtility(execution, game, candidate, perspective, config),
          anchorUtility,
        )
      );
    })
    .sort((left, right) =>
      compareRankedEvaluatedRootIndices(roots, left, right),
    )[0];
}

function whiteFollowupRepresentative(
  game: MonsGame,
  roots: readonly EvaluatedRoot[],
  shortlist: readonly number[],
  config: AutomoveConfig,
): number | undefined {
  if (shortlist.length === 0) return undefined;
  return roots
    .map((_root, index) => index)
    .filter((index) => {
      const root = roots[index];
      return (
        root !== undefined &&
        !shortlist.includes(index) &&
        root.spiritOwnManaSetupNow &&
        root.opponentManaProgress &&
        !isPlainSpiritDevelopmentRoot(root) &&
        advisorRootIsSafe(root) &&
        shortlist.some((shortlistIndex) =>
          whiteSpiritFollowupSetupCompetition(
            game,
            roots,
            [index, shortlistIndex],
            config,
          ),
        )
      );
    })
    .sort((left, right) =>
      compareRankedEvaluatedRootIndices(roots, left, right),
    )[0];
}

function plainSpiritClusterProgressReentry(
  execution: AutomoveExecutionContext,
  roots: readonly EvaluatedRoot[],
  candidateIndices: readonly number[],
  perspective: Color,
  config: AutomoveConfig,
): number | undefined {
  if (
    !productionEnabled(config) ||
    !config.planner.enabled ||
    !config.planner.secondaryAnalysis ||
    candidateIndices.length < 2 ||
    !candidateIndices.every((index) => {
      const root = roots[index];
      return (
        root !== undefined &&
        isPlainSpiritDevelopmentRoot(root) &&
        advisorRootIsSafe(root)
      );
    })
  ) {
    return undefined;
  }
  const candidateRoots = candidateIndices
    .map((index) => roots[index])
    .filter((root): root is EvaluatedRoot => root !== undefined);
  const bestScore = Math.max(...candidateRoots.map((root) => root.score));
  const bestRank = Math.min(...candidateRoots.map((root) => root.rootRank));
  const followup = memoizedByIndex((index): number => {
    const root = roots[index];
    return root === undefined
      ? MIN_SCORE
      : spiritFollowupFloorScore(execution, root.game, perspective, config);
  });
  const bestFollowup = Math.max(...candidateIndices.map(followup));
  const omitted = roots
    .map((_root, index) => index)
    .filter((index) => !candidateIndices.includes(index));
  const followupReentry = omitted
    .filter((index) => {
      const root = roots[index];
      return (
        root !== undefined &&
        advisorRootIsSafe(root) &&
        !root.ownDrainerVulnerable &&
        !root.ownDrainerWalkVulnerable &&
        !root.spiritDevelopment &&
        !root.spiritSameTurnScoreSetupNow &&
        !root.spiritOwnManaSetupNow &&
        saturatingScoreAdd(root.score, 32) >= bestScore &&
        followup(index) >= saturatingScoreAdd(bestFollowup, 32)
      );
    })
    .sort((left, right) => {
      const followupOrder = followup(right) - followup(left);
      return (
        followupOrder || compareRankedEvaluatedRootIndices(roots, left, right)
      );
    })[0];
  if (followupReentry !== undefined) return followupReentry;
  return omitted
    .filter((index) => {
      const root = roots[index];
      return (
        root !== undefined &&
        advisorRootIsSafe(root) &&
        !root.ownDrainerVulnerable &&
        !root.ownDrainerWalkVulnerable &&
        !root.spiritDevelopment &&
        !root.spiritSameTurnScoreSetupNow &&
        !root.spiritOwnManaSetupNow &&
        hasProgressSurface(root) &&
        root.policyPriority > 0 &&
        root.score >= bestScore &&
        root.rootRank + 2 <= bestRank
      );
    })
    .sort((left, right) =>
      compareRankedEvaluatedRootIndices(roots, left, right),
    )[0];
}

function riskyRecoveryReentry(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  roots: readonly EvaluatedRoot[],
  candidateIndices: readonly number[],
  perspective: Color,
  config: AutomoveConfig,
): number | undefined {
  if (!productionEnabled(config) || !config.planner.enabled) {
    return undefined;
  }
  const anchorIndex = candidateIndices.find((index) => {
    const root = roots[index];
    return (
      root !== undefined &&
      advisorRootIsSafe(root) &&
      hasProgressSurface(root) &&
      root.policyPriority > 0
    );
  });
  const anchor = anchorIndex === undefined ? undefined : roots[anchorIndex];
  if (anchor === undefined) return undefined;
  let bestIndex: number | undefined;
  let bestUtility: ReturnType<typeof rootUtility> | undefined;
  roots.forEach((root, index) => {
    if (
      candidateIndices.includes(index) ||
      !advisorRootIsUnsafe(root) ||
      root.ownDrainerWalkVulnerable ||
      root.manaHandoffToOpponent ||
      root.hasRoundtrip ||
      !sameNonTacticalProgressLane(root, anchor) ||
      root.score + 32 < anchor.score ||
      root.game.activeColor !== perspective ||
      root.game.winnerColor() !== undefined
    ) {
      return;
    }
    const utility = rootUtility(execution, game, root, perspective, config);
    if (!utilitySupportsTemporaryRiskRecovery(utility)) return;
    if (
      bestUtility === undefined ||
      compareUtilityPrimaryAxes(utility, bestUtility) > 0 ||
      (utilitiesEqual(utility, bestUtility) &&
        bestIndex !== undefined &&
        compareRankedEvaluatedRootIndices(roots, index, bestIndex) < 0)
    ) {
      bestIndex = index;
      bestUtility = utility;
    }
  });
  return bestIndex;
}

function blackTurnSixSpiritReentry(
  game: MonsGame,
  roots: readonly EvaluatedRoot[],
  shortlist: readonly number[],
  config: AutomoveConfig,
): number | undefined {
  if (
    !productionEnabled(config) ||
    game.activeColor !== Color.Black ||
    game.turnNumber !== 6 ||
    !game.playerCanMoveMana() ||
    shortlist.length === 0 ||
    !shortlist.every((index) => {
      const root = roots[index];
      return (
        root !== undefined &&
        advisorRootFamily(root) === TurnPlanFamily.ManaTempo &&
        advisorRootIsUnsafe(root) &&
        !root.winsImmediately &&
        !root.attacksOpponentDrainer &&
        !hasConcreteScoreSurface(root) &&
        root.sameTurnScoreWindowValue === 0 &&
        !root.manaHandoffToOpponent &&
        !root.hasRoundtrip
      );
    })
  ) {
    return undefined;
  }
  const bestRank = Math.min(
    ...shortlist.map(
      (index) => roots[index]?.rootRank ?? Number.MAX_SAFE_INTEGER,
    ),
  );
  const bestScore = Math.max(
    ...shortlist.map((index) => roots[index]?.score ?? MIN_SCORE),
  );
  return roots
    .map((_root, index) => index)
    .filter((index) => {
      const root = roots[index];
      return (
        root !== undefined &&
        !shortlist.includes(index) &&
        advisorRootFamily(root) === TurnPlanFamily.SpiritImpact &&
        isPlainSpiritDevelopmentRoot(root) &&
        !hasConcreteScoreSurface(root) &&
        !root.attacksOpponentDrainer &&
        root.sameTurnScoreWindowValue === 0 &&
        !root.manaHandoffToOpponent &&
        !root.hasRoundtrip &&
        root.rootRank + 4 <= bestRank &&
        saturatingScoreSubtract(bestScore, root.score) <= 1_024
      );
    })
    .sort((left, right) =>
      compareRankedEvaluatedRootIndices(roots, left, right),
    )[0];
}

function safeProgressSiblingReentry(
  roots: readonly EvaluatedRoot[],
  candidateIndices: readonly number[],
  shortlist: readonly number[],
  config: AutomoveConfig,
): number | undefined {
  if (!productionEnabled(config)) return undefined;
  const anchorIndex = shortlist[0];
  const anchor = anchorIndex === undefined ? undefined : roots[anchorIndex];
  if (
    anchor === undefined ||
    !advisorRootIsUnsafe(anchor) ||
    !hasProgressSurface(anchor)
  ) {
    return undefined;
  }
  return candidateIndices
    .filter((index) => {
      const root = roots[index];
      return (
        root !== undefined &&
        !shortlist.includes(index) &&
        advisorRootIsSafe(root) &&
        sameNonTacticalProgressLane(root, anchor) &&
        saturatingScoreSubtract(anchor.score, root.score) <= 320
      );
    })
    .sort((left, right) =>
      compareRankedEvaluatedRootIndices(roots, left, right),
    )[0];
}

function blackNoActionSafeProgressReentry(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  roots: readonly EvaluatedRoot[],
  shortlist: readonly number[],
  config: AutomoveConfig,
): number | undefined {
  if (
    !productionEnabled(config) ||
    game.activeColor !== Color.Black ||
    game.turnNumber < 6 ||
    game.monsMovesCount !== 0 ||
    game.playerCanUseAction() ||
    !game.playerCanMoveMana() ||
    shortlist.length === 0
  ) {
    return undefined;
  }
  const delta = exactOpportunityContext(
    execution,
    game,
    game.activeColor,
  ).delta;
  if (
    delta.sameTurnScoreWindowValue > 1 ||
    delta.opponentWindowDenyGain > 1 ||
    (delta.sameTurnScoreWindowValue === 0 &&
      delta.opponentWindowDenyGain === 0) ||
    shortlist.some((index) => {
      const root = roots[index];
      return (
        root !== undefined &&
        (advisorRootFamily(root) === TurnPlanFamily.SafeSupermanaProgress ||
          advisorRootFamily(root) ===
            TurnPlanFamily.SafeOpponentManaProgress) &&
        advisorRootIsSafe(root)
      );
    })
  ) {
    return undefined;
  }
  return roots
    .map((_root, index) => index)
    .filter((index) => {
      const root = roots[index];
      const family = root === undefined ? undefined : advisorRootFamily(root);
      return (
        root !== undefined &&
        !shortlist.includes(index) &&
        (family === TurnPlanFamily.SafeSupermanaProgress ||
          family === TurnPlanFamily.SafeOpponentManaProgress) &&
        !root.manaHandoffToOpponent &&
        !root.hasRoundtrip &&
        !root.winsImmediately &&
        !root.attacksOpponentDrainer &&
        !hasConcreteScoreSurface(root) &&
        root.sameTurnScoreWindowValue === 0 &&
        root.score >= 0
      );
    })
    .sort((left, right) => {
      const leftRoot = roots[left];
      const rightRoot = roots[right];
      if (leftRoot?.score !== rightRoot?.score) {
        return (rightRoot?.score ?? MIN_SCORE) - (leftRoot?.score ?? MIN_SCORE);
      }
      return compareRankedEvaluatedRootIndices(roots, left, right);
    })[0];
}

function blackNoActionManaSiblingReentry(
  game: MonsGame,
  roots: readonly EvaluatedRoot[],
  shortlist: readonly number[],
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
  const anchorIndex = [...shortlist]
    .filter((index) => {
      const root = roots[index];
      return (
        root !== undefined &&
        advisorRootFamily(root) === TurnPlanFamily.ManaTempo &&
        advisorRootIsSafe(root) &&
        !root.winsImmediately &&
        !root.attacksOpponentDrainer &&
        !hasConcreteScoreSurface(root)
      );
    })
    .sort((left, right) =>
      compareRankedEvaluatedRootIndices(roots, left, right),
    )[0];
  const anchor = anchorIndex === undefined ? undefined : roots[anchorIndex];
  if (anchor === undefined) return undefined;
  const sameWindowLane =
    anchor.sameTurnScoreWindowValue > 0 &&
    !anchor.ownDrainerVulnerable &&
    !anchor.ownDrainerWalkVulnerable;
  if (anchor.rootRank < 6 && !sameWindowLane) return undefined;
  return roots
    .map((_root, index) => index)
    .filter((index) => {
      const root = roots[index];
      if (
        root === undefined ||
        shortlist.includes(index) ||
        advisorRootFamily(root) !== TurnPlanFamily.ManaTempo ||
        root.manaHandoffToOpponent ||
        root.hasRoundtrip ||
        root.winsImmediately ||
        root.attacksOpponentDrainer ||
        hasConcreteScoreSurface(root) ||
        !(
          root.rootRank + 4 <= anchor.rootRank ||
          (sameWindowLane && root.rootRank < anchor.rootRank)
        ) ||
        root.sameTurnScoreWindowValue > anchor.sameTurnScoreWindowValue
      ) {
        return false;
      }
      if (anchor.score >= 0) return root.score >= 0;
      if (
        sameWindowLane &&
        root.sameTurnScoreWindowValue === anchor.sameTurnScoreWindowValue &&
        root.safeSupermanaProgressSteps === anchor.safeSupermanaProgressSteps &&
        root.safeOpponentManaProgressSteps ===
          anchor.safeOpponentManaProgressSteps &&
        root.ownDrainerVulnerable === anchor.ownDrainerVulnerable &&
        root.ownDrainerWalkVulnerable === anchor.ownDrainerWalkVulnerable
      ) {
        return true;
      }
      return saturatingScoreSubtract(anchor.score, root.score) <= 192;
    })
    .sort((left, right) =>
      compareRankedEvaluatedRootIndices(roots, left, right),
    )[0];
}

function collectAdvisorReentries(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  roots: readonly EvaluatedRoot[],
  candidateIndices: readonly number[],
  shortlist: readonly number[],
  perspective: Color,
  config: AutomoveConfig,
): number[] {
  const result: number[] = [];
  const add = (index: number | undefined): void => {
    if (index !== undefined && !result.includes(index)) result.push(index);
  };
  add(
    plainSpiritClusterProgressReentry(
      execution,
      roots,
      candidateIndices,
      perspective,
      config,
    ),
  );
  add(
    riskyRecoveryReentry(
      execution,
      game,
      roots,
      candidateIndices,
      perspective,
      config,
    ),
  );
  add(blackTurnSixSpiritReentry(game, roots, shortlist, config));
  add(safeProgressSiblingReentry(roots, candidateIndices, shortlist, config));
  add(
    blackNoActionSafeProgressReentry(execution, game, roots, shortlist, config),
  );
  add(blackNoActionManaSiblingReentry(game, roots, shortlist, config));
  add(
    productionWhiteTurnFourManaSiblingReentry(
      execution,
      game,
      roots,
      shortlist,
      perspective,
      config,
    ),
  );
  return result;
}

export {
  collectAdvisorReentries,
  findScoredRepresentative,
  plainSpiritClusterProgressReentry,
  riskyRecoveryReentry,
  whiteFollowupRepresentative,
};
