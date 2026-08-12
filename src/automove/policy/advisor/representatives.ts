import { Color } from "../../../api/types.js";
import { MonsGame } from "../../../engine/game/mons-game.js";
import type { AutomoveExecutionContext } from "../../core/execution-context.js";
import type { RootCandidate } from "../../root/types.js";
import { rootFamily as advisorRootFamily } from "../../root/family.js";
import type { AutomoveConfig } from "../../config/types.js";
import {
  hasConcreteScoreSurface,
  isPlainSpiritDevelopmentRoot,
  productionEnabled,
} from "../../config/types.js";
import { TurnPlanFamily } from "../../turn/model.js";
import {
  advisorRootIsSafe,
  compareRankedRootMoveIndices,
  rootMoveUtility,
  utilityCompetes,
} from "./support.js";
import { inputChainsShareFirstInput as sameFirstInput } from "../../../engine/model/domain.js";

export function findRootMoveRepresentative(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  roots: readonly RootCandidate[],
  perspective: Color,
  config: AutomoveConfig,
  predicate: (root: RootCandidate) => boolean,
): number | undefined {
  const anchor = roots[0];
  if (anchor === undefined) return undefined;
  if (predicate(anchor) && advisorRootIsSafe(anchor)) return undefined;
  const anchorUtility = rootMoveUtility(execution, game, anchor, perspective, config);
  return roots
    .map((_root, index) => index)
    .filter((index) => {
      const candidate = roots[index];
      return (
        candidate !== undefined &&
        predicate(candidate) &&
        advisorRootIsSafe(candidate) &&
        utilityCompetes(
          rootMoveUtility(execution, game, candidate, perspective, config),
          anchorUtility,
        )
      );
    })
    .sort((left, right) => compareRankedRootMoveIndices(roots, left, right))[0];
}

export function sameOpeningSetupRepresentative(
  roots: readonly RootCandidate[],
  config: AutomoveConfig,
): number | undefined {
  const anchor = roots[0];
  if (
    anchor === undefined ||
    !productionEnabled(config) ||
    !isPlainSpiritDevelopmentRoot(anchor) ||
    !advisorRootIsSafe(anchor)
  ) {
    return undefined;
  }
  return roots
    .map((_root, index) => index)
    .filter((index) => {
      const root = roots[index];
      return (
        root !== undefined &&
        advisorRootFamily(root) === TurnPlanFamily.SpiritImpact &&
        !isPlainSpiritDevelopmentRoot(root) &&
        sameFirstInput(root.inputs, anchor.inputs) &&
        root.efficiency === anchor.efficiency &&
        advisorRootIsSafe(root) &&
        !root.winsImmediately &&
        !root.attacksOpponentDrainer &&
        !root.scoresSupermanaThisTurn &&
        !root.scoresOpponentManaThisTurn &&
        !root.safeSupermanaPickupNow &&
        !root.safeOpponentManaPickupNow &&
        root.sameTurnScoreWindowValue === 0
      );
    })
    .sort((left, right) => compareRankedRootMoveIndices(roots, left, right))[0];
}

export function isBlackTurnSixPlainSpiritSetupPair(
  game: MonsGame,
  plain: RootCandidate,
  setup: RootCandidate,
  config: AutomoveConfig,
): boolean {
  return (
    productionEnabled(config) &&
    game.activeColor === Color.Black &&
    game.turnNumber === 6 &&
    game.monsMovesCount === 0 &&
    game.playerCanUseAction() &&
    game.playerCanMoveMana() &&
    advisorRootFamily(plain) === TurnPlanFamily.SpiritImpact &&
    advisorRootFamily(setup) === TurnPlanFamily.SpiritImpact &&
    isPlainSpiritDevelopmentRoot(plain) &&
    setup.spiritOwnManaSetupNow &&
    !setup.spiritSameTurnScoreSetupNow &&
    sameFirstInput(plain.inputs, setup.inputs) &&
    plain.ownDrainerVulnerable === setup.ownDrainerVulnerable &&
    plain.ownDrainerWalkVulnerable === setup.ownDrainerWalkVulnerable &&
    !plain.manaHandoffToOpponent &&
    !setup.manaHandoffToOpponent &&
    !plain.hasRoundtrip &&
    !setup.hasRoundtrip &&
    !hasConcreteScoreSurface(plain) &&
    !hasConcreteScoreSurface(setup) &&
    !plain.attacksOpponentDrainer &&
    !setup.attacksOpponentDrainer &&
    plain.sameTurnScoreWindowValue === 0 &&
    setup.sameTurnScoreWindowValue === 0 &&
    !plain.supermanaProgress &&
    !setup.supermanaProgress &&
    !plain.opponentManaProgress &&
    !setup.opponentManaProgress
  );
}
