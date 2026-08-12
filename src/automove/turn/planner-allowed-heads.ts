import { inputChainKey, type Input } from "../../engine/model/domain.js";
import type { Color } from "../../api/types.js";
import { MonsGame } from "../../engine/game/mons-game.js";
import type { AutomoveExecutionContext } from "../core/execution-context.js";
import {
  MIN_SCORE,
  saturatingScoreAdd,
  saturatingScoreMultiply,
} from "../core/score-math.js";
import { applyInputsForSearch } from "../transitions/simulation.js";
import { opponentCanWinImmediately, ownDrainerSafetyScore } from "./evaluation.js";
import { type TurnPlan, type TurnUtility } from "./model.js";
import {
  compareNumber,
  compareTuples,
  compareUtilityPrimaryAxes,
  turnEngineComparePlans,
} from "./ordering.js";

export function allowedRankMap(
  allowedFirstSteps: readonly (readonly Input[])[],
): ReadonlyMap<string, number> {
  const result = new Map<string, number>();
  allowedFirstSteps.forEach((inputs, rank) => {
    const key = inputChainKey(inputs);
    if (!result.has(key)) result.set(key, rank);
  });
  return result;
}

export type AllowedHeadSelectionMeta = {
  readonly rank: number;
  readonly allowedLength: number;
  readonly firstStepOpponentImmediateLoss: boolean;
  readonly firstStepDrainerSafety: number;
};

export function allowedHeadSelectionMeta(
  execution: AutomoveExecutionContext,
  root: MonsGame,
  plan: TurnPlan,
  perspective: Color,
  rank: number,
  allowedLength: number,
): AllowedHeadSelectionMeta {
  const first = plan.compiledChunks[0];
  const after = first === undefined ? undefined : applyInputsForSearch(root, first);
  return {
    rank,
    allowedLength,
    firstStepOpponentImmediateLoss:
      after !== undefined && opponentCanWinImmediately(execution, after, perspective),
    firstStepDrainerSafety:
      after === undefined
        ? Math.trunc(MIN_SCORE / 4)
        : ownDrainerSafetyScore(execution, after.board, perspective),
  };
}

function allowedHeadRankAdjustedEval(
  utility: TurnUtility,
  meta: AllowedHeadSelectionMeta,
): number {
  return saturatingScoreAdd(
    utility.evalScore,
    saturatingScoreMultiply(
      Math.min(Math.max(meta.allowedLength - meta.rank, 0), 96),
      12,
    ),
  );
}

export function compareAllowedHeadPlans(
  left: TurnPlan,
  leftMeta: AllowedHeadSelectionMeta,
  right: TurnPlan,
  rightMeta: AllowedHeadSelectionMeta,
): number {
  let order = compareUtilityPrimaryAxes(left.utility, right.utility);
  if (order !== 0) return order;
  order = compareTuples(
    [Number(!leftMeta.firstStepOpponentImmediateLoss), leftMeta.firstStepDrainerSafety],
    [
      Number(!rightMeta.firstStepOpponentImmediateLoss),
      rightMeta.firstStepDrainerSafety,
    ],
  );
  if (order !== 0) return order;
  order = compareNumber(
    allowedHeadRankAdjustedEval(left.utility, leftMeta),
    allowedHeadRankAdjustedEval(right.utility, rightMeta),
  );
  if (order !== 0) return order;
  order = turnEngineComparePlans(left, right);
  return order !== 0 ? order : compareNumber(rightMeta.rank, leftMeta.rank);
}
