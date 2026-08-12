import { Color, MonKind } from "../../../api/types.js";
import { isMonFainted, itemMon } from "../../../engine/model/domain.js";
import type { MonsGame } from "../../../engine/game/mons-game.js";
import type { AutomoveExecutionContext } from "../../core/execution-context.js";
import { isDrainerExactlySafeNextTurnOnBoard } from "../../exact/drainer-safety.js";
import type { RootCandidate } from "../../root/types.js";
import { rootFamily } from "../../root/family.js";
import type { EvaluatedRoot } from "../../root/types.js";
import {
  hasConcreteScoreSurface as rootHasConcreteScoreSurface,
  hasProgressSurface,
} from "../../config/types.js";
import { TurnPlanFamily } from "../../turn/model.js";

export function valueAt<Value>(values: readonly Value[], index: number): Value {
  const value = values[index];
  if (value === undefined) {
    throw new RangeError(`production selector index ${index} is out of bounds`);
  }
  return value;
}

export function hasPickupUpgrade(
  candidate: RootCandidate,
  selected: RootCandidate,
): boolean {
  return (
    (candidate.safeSupermanaPickupNow && !selected.safeSupermanaPickupNow) ||
    (candidate.safeOpponentManaPickupNow && !selected.safeOpponentManaPickupNow)
  );
}

export function rootHasProgressSurface(root: RootCandidate): boolean {
  return (
    hasProgressSurface(root) ||
    root.scoresSupermanaThisTurn ||
    root.scoresOpponentManaThisTurn
  );
}

export function isProductionModeNonConcreteManaWindowRoot(
  root: EvaluatedRoot,
): boolean {
  return (
    rootFamily(root) === TurnPlanFamily.ManaTempo &&
    root.sameTurnScoreWindowValue > 0 &&
    root.sameTurnScoreWindowValue <= 1 &&
    !rootHasConcreteScoreSurface(root) &&
    !root.attacksOpponentDrainer &&
    !root.manaHandoffToOpponent &&
    !root.hasRoundtrip
  );
}

export function ownDrainerVulnerableNextTurn(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
): boolean {
  for (const [location, item] of game.board.entries()) {
    const mon = itemMon(item);
    if (mon?.kind !== MonKind.Drainer || mon.color !== perspective) continue;
    if (isMonFainted(mon)) return true;
    if (game.isFirstTurn()) return false;
    return !isDrainerExactlySafeNextTurnOnBoard(
      execution,
      game.board,
      perspective,
      location,
    );
  }
  return false;
}
