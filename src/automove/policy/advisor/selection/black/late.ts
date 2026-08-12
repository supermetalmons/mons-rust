import { Color } from "../../../../../api/types.js";
import { MonsGame } from "../../../../../engine/game/mons-game.js";
import { exactOpportunityContext } from "../../../../exact/turn-opportunity.js";
import type { AutomoveExecutionContext } from "../../../../core/execution-context.js";
import { rootProgressOrSetupBetter } from "../../../reply-risk/shortlist.js";
import { rootFamily as advisorRootFamily } from "../../../../root/family.js";
import { compareRankedEvaluatedRootIndices } from "../../../../root/evaluated-ordering.js";
import { saturatingScoreSubtract } from "../../../../core/score-math.js";
import type { EvaluatedRoot } from "../../../../root/types.js";
import {
  hasProgressSurface,
  productionEnabled,
  rootIsUnsafe as advisorRootIsUnsafe,
} from "../../../../config/types.js";
import type { AutomoveConfig } from "../../../../config/types.js";
import { TurnPlanFamily } from "../../../../turn/model.js";
import { rootIsNonTactical } from "./baseline.js";
import { isNonConcreteManaWindowRoot } from "../white/mana-progress.js";
import {
  bestOverrideIndex,
  compareRootRankThenRanked,
  sameInputAt,
} from "../../support.js";
import { inputChainsShareFirstInput as sameFirstInput } from "../../../../../engine/model/domain.js";

function blackLateWindowManaSafetyOverride(
  game: MonsGame,
  roots: readonly EvaluatedRoot[],
  selectionIndices: readonly number[],
  approvedIndex: number,
  config: AutomoveConfig,
): number | undefined {
  if (
    !productionEnabled(config) ||
    game.activeColor !== Color.Black ||
    game.turnNumber < 8 ||
    game.monsMovesCount === 0 ||
    game.playerCanUseAction() ||
    !game.playerCanMoveMana() ||
    roots.length === 0
  ) {
    return undefined;
  }
  const approved = roots[approvedIndex];
  if (
    approved === undefined ||
    advisorRootFamily(approved) !== TurnPlanFamily.ManaTempo ||
    !isNonConcreteManaWindowRoot(approved) ||
    !approved.ownDrainerVulnerable ||
    approved.ownDrainerWalkVulnerable
  ) {
    return undefined;
  }
  const approvedProgress = hasProgressSurface(approved);
  return bestOverrideIndex(
    roots,
    selectionIndices,
    (challenger, index) => {
      const progress = hasProgressSurface(challenger);
      const progressBetter =
        rootProgressOrSetupBetter(challenger, approved) ||
        (progress && !approvedProgress);
      return (
        index !== approvedIndex &&
        advisorRootFamily(challenger) === TurnPlanFamily.ManaTempo &&
        sameFirstInput(challenger.inputs, approved.inputs) &&
        challenger.sameTurnScoreWindowValue === 0 &&
        !challenger.ownDrainerVulnerable &&
        !challenger.ownDrainerWalkVulnerable &&
        rootIsNonTactical(challenger) &&
        !advisorRootIsUnsafe(challenger) &&
        (approved.rootRank > 0 || progressBetter) &&
        saturatingScoreSubtract(approved.score, challenger.score) <= 32 &&
        challenger.rootRank <= approved.rootRank + 2
      );
    },
    (left, right) => compareRootRankThenRanked(roots, left, right),
  );
}

function blackLateWindowCompetitionOverride(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  roots: readonly EvaluatedRoot[],
  approvedIndex: number,
  config: AutomoveConfig,
): number | undefined {
  if (
    !productionEnabled(config) ||
    game.activeColor !== Color.Black ||
    game.turnNumber < 8 ||
    game.monsMovesCount < 3 ||
    game.playerCanUseAction() ||
    !game.playerCanMoveMana() ||
    roots.length === 0
  ) {
    return undefined;
  }
  const exact = exactOpportunityContext(execution, game, game.activeColor);
  if (
    exact.delta.sameTurnScoreWindowValue === 0 &&
    exact.delta.opponentWindowDenyGain === 0
  ) {
    return undefined;
  }
  const approved = roots[approvedIndex];
  if (
    approved === undefined ||
    advisorRootFamily(approved) !== TurnPlanFamily.ManaTempo ||
    approved.sameTurnScoreWindowValue !== 0 ||
    !rootIsNonTactical(approved) ||
    advisorRootIsUnsafe(approved)
  ) {
    return undefined;
  }
  return bestOverrideIndex(
    roots,
    roots.map((_root, index) => index),
    (root, index) =>
      index !== approvedIndex &&
      advisorRootFamily(root) === TurnPlanFamily.ManaTempo &&
      isNonConcreteManaWindowRoot(root) &&
      root.rootRank === 0 &&
      root.ownDrainerVulnerable &&
      !root.ownDrainerWalkVulnerable &&
      saturatingScoreSubtract(approved.score, root.score) <= 256,
    (left, right) => compareRankedEvaluatedRootIndices(roots, left, right),
  );
}

function blackLateRecoveryProgressCompetitionOverride(
  game: MonsGame,
  roots: readonly EvaluatedRoot[],
  selectionIndices: readonly number[],
  approvedIndex: number,
  config: AutomoveConfig,
): number | undefined {
  if (
    !productionEnabled(config) ||
    game.activeColor !== Color.Black ||
    game.turnNumber < 12 ||
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
    advisorRootFamily(approved) !== TurnPlanFamily.DrainerSafetyRecovery ||
    !approved.spiritDevelopment ||
    !hasProgressSurface(approved) ||
    approved.spiritSameTurnScoreSetupNow ||
    approved.spiritOwnManaSetupNow ||
    approved.sameTurnScoreWindowValue > 0 ||
    !rootIsNonTactical(approved) ||
    advisorRootIsUnsafe(approved) ||
    approved.inputs.length < 3
  ) {
    return undefined;
  }
  return bestOverrideIndex(
    roots,
    selectionIndices,
    (challenger, index) =>
      index !== approvedIndex &&
      advisorRootFamily(challenger) === TurnPlanFamily.SpiritImpact &&
      challenger.spiritDevelopment &&
      !challenger.spiritSameTurnScoreSetupNow &&
      !challenger.spiritOwnManaSetupNow &&
      hasProgressSurface(challenger) &&
      rootIsNonTactical(challenger) &&
      challenger.sameTurnScoreWindowValue === 0 &&
      challenger.inputs.length >= 3 &&
      sameInputAt(challenger.inputs, approved.inputs, 0) &&
      sameInputAt(challenger.inputs, approved.inputs, 1) &&
      saturatingScoreSubtract(approved.score, challenger.score) <= 1_024 &&
      challenger.rootRank <= approved.rootRank + 2 &&
      rootProgressOrSetupBetter(challenger, approved),
    (left, right) => compareRankedEvaluatedRootIndices(roots, left, right),
  );
}

export {
  blackLateRecoveryProgressCompetitionOverride,
  blackLateWindowCompetitionOverride,
  blackLateWindowManaSafetyOverride,
};
