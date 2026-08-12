import { Color } from "../../../../../api/types.js";
import { MonsGame } from "../../../../../engine/game/mons-game.js";
import { exactOpportunityContext } from "../../../../exact/turn-opportunity.js";
import type { AutomoveExecutionContext } from "../../../../core/execution-context.js";
import { rootFamily as advisorRootFamily } from "../../../../root/family.js";
import type { EvaluatedRoot } from "../../../../root/types.js";
import {
  productionEnabled,
  rootIsUnsafe as advisorRootIsUnsafe,
} from "../../../../config/types.js";
import type { AutomoveConfig } from "../../../../config/types.js";
import { TurnPlanFamily } from "../../../../turn/model.js";
import { rootIsNonTactical } from "../black/baseline.js";
import { isNonConcreteManaWindowRoot } from "../white/mana-progress.js";
import { bestOverrideIndex, compareRootRankThenRanked } from "../../support.js";
import { inputChainsShareFirstInput as sameFirstInput } from "../../../../../engine/model/domain.js";

type NoActionRecoveryPolicy = {
  readonly color: Color;
  readonly turnNumber: number;
  readonly monsMoves:
    | { readonly kind: "exact"; readonly count: number }
    | { readonly kind: "minimum"; readonly count: number };
  readonly rejectDrainerAttack: boolean;
  readonly requireSameProgressSteps: boolean;
};

function noActionRecoveryOverride(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  roots: readonly EvaluatedRoot[],
  approvedIndex: number,
  policy: NoActionRecoveryPolicy,
  config: AutomoveConfig,
): number | undefined {
  if (
    !productionEnabled(config) ||
    game.activeColor !== policy.color ||
    game.turnNumber !== policy.turnNumber ||
    (policy.monsMoves.kind === "exact"
      ? game.monsMovesCount !== policy.monsMoves.count
      : game.monsMovesCount < policy.monsMoves.count) ||
    game.playerCanUseAction() ||
    !game.playerCanMoveMana() ||
    roots.length === 0
  ) {
    return undefined;
  }
  const exact = exactOpportunityContext(execution, game, game.activeColor);
  if (
    exact.delta.sameTurnScoreWindowValue > 1 ||
    exact.delta.opponentWindowDenyGain > 1 ||
    (exact.delta.sameTurnScoreWindowValue === 0 &&
      exact.delta.opponentWindowDenyGain === 0) ||
    (policy.rejectDrainerAttack && exact.delta.drainerAttackAvailable) ||
    exact.delta.drainerSafety >= 0
  ) {
    return undefined;
  }
  const approved = roots[approvedIndex];
  if (
    approved === undefined ||
    !isNonConcreteManaWindowRoot(approved) ||
    !approved.ownDrainerVulnerable ||
    approved.ownDrainerWalkVulnerable ||
    approved.spiritDevelopment ||
    approved.spiritSameTurnScoreSetupNow ||
    approved.spiritOwnManaSetupNow
  ) {
    return undefined;
  }
  return bestOverrideIndex(
    roots,
    roots.map((_root, index) => index),
    (root, index) =>
      index !== approvedIndex &&
      advisorRootFamily(root) === TurnPlanFamily.DrainerSafetyRecovery &&
      root.classes.drainerSafetyRecover &&
      !advisorRootIsUnsafe(root) &&
      !root.ownDrainerVulnerable &&
      !root.ownDrainerWalkVulnerable &&
      !root.spiritDevelopment &&
      !root.spiritSameTurnScoreSetupNow &&
      !root.spiritOwnManaSetupNow &&
      rootIsNonTactical(root) &&
      root.sameTurnScoreWindowValue === 0 &&
      sameFirstInput(root.inputs, approved.inputs) &&
      (!policy.requireSameProgressSteps ||
        (root.safeSupermanaProgressSteps === approved.safeSupermanaProgressSteps &&
          root.safeOpponentManaProgressSteps ===
            approved.safeOpponentManaProgressSteps)) &&
      root.rootRank <= approved.rootRank &&
      root.scorePathBestSteps > approved.scorePathBestSteps,
    (left, right) => compareRootRankThenRanked(roots, left, right),
  );
}

function whiteTurnThreeNoActionRecoveryOverride(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  roots: readonly EvaluatedRoot[],
  approvedIndex: number,
  config: AutomoveConfig,
): number | undefined {
  return noActionRecoveryOverride(
    execution,
    game,
    roots,
    approvedIndex,
    {
      color: Color.White,
      turnNumber: 3,
      monsMoves: { kind: "exact", count: 0 },
      rejectDrainerAttack: false,
      requireSameProgressSteps: false,
    },
    config,
  );
}

function blackTurnFourWeakWindowRecoveryOverride(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  roots: readonly EvaluatedRoot[],
  approvedIndex: number,
  config: AutomoveConfig,
): number | undefined {
  return noActionRecoveryOverride(
    execution,
    game,
    roots,
    approvedIndex,
    {
      color: Color.Black,
      turnNumber: 4,
      monsMoves: { kind: "minimum", count: 1 },
      rejectDrainerAttack: true,
      requireSameProgressSteps: true,
    },
    config,
  );
}

export {
  blackTurnFourWeakWindowRecoveryOverride,
  whiteTurnThreeNoActionRecoveryOverride,
};
