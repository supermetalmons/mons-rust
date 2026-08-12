import type { AutomoveExecutionContext } from "../core/execution-context.js";
import { MonKind, type Color } from "../../api/types.js";
import {
  isMonFainted,
  isSpiritTargetAllowed,
  itemMana,
  itemMon,
  otherColor,
} from "../../engine/model/domain.js";
import { MonsGame } from "../../engine/game/mons-game.js";
import {
  BOARD_SIZE,
  locationDistance,
  nearbyLocations,
  spiritReachableLocations,
} from "../../engine/board/geometry.js";
import { exactSearchStateHash } from "../exact/hash.js";
import { exactTurnTacticalProjectionWithSearchHash } from "../exact/turn-opportunity.js";
import { applyInputsForSearchWithEvents } from "../transitions/simulation.js";
import {
  distanceToNearestPool,
  manaMoveDestinationAllowed,
  spiritDestinationAllowed,
} from "./action-rules.js";
import { findAwakeDrainerLocation, ownDrainerSafetyScore } from "./evaluation.js";
import { TurnPlanFamily, type ActionSeed, type TurnEngineConfig } from "./model.js";
import { compareActionKeys, compareNumber } from "./ordering.js";
import { tacticalProjectionFlags } from "./opportunity-seeds-oracle.js";

function progressPriorityBonus(
  before: number | undefined,
  after: number | undefined,
): number {
  const beforeSteps = before ?? BOARD_SIZE * 3;
  const afterSteps = after ?? BOARD_SIZE * 3;
  return afterSteps >= beforeSteps ? 0 : (beforeSteps - afterSteps) * 220;
}

export function spiritImpactSeeds(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: TurnEngineConfig,
): ActionSeed[] {
  if (
    execution.session.checkpoint() ||
    !config.enableSpiritFamily ||
    !game.playerCanUseAction()
  )
    return [];
  const flags = tacticalProjectionFlags(true, true, true, true, true);
  const before = exactTurnTacticalProjectionWithSearchHash(
    execution,
    game,
    perspective,
    exactSearchStateHash(game),
    flags,
  );
  if (execution.session.checkpoint()) return [];
  const beforeSafety = ownDrainerSafetyScore(execution, game.board, perspective);
  const result: ActionSeed[] = [];
  for (const [spirit, item] of game.board.entries()) {
    if (execution.session.checkpoint()) return [];
    const mon = itemMon(item);
    if (
      mon?.color !== perspective ||
      mon.kind !== MonKind.Spirit ||
      isMonFainted(mon) ||
      game.board.squareAt(spirit).kind === "mon-base"
    ) {
      continue;
    }
    for (const target of spiritReachableLocations(spirit)) {
      if (execution.session.checkpoint()) return [];
      const targetItem = game.board.get(target);
      if (targetItem === undefined || !isSpiritTargetAllowed(targetItem)) continue;
      for (const destination of nearbyLocations(target)) {
        if (execution.session.checkpoint()) return [];
        if (!spiritDestinationAllowed(game.board, targetItem, destination)) continue;
        const applied = applyInputsForSearchWithEvents(game, [
          { kind: "location", location: spirit },
          { kind: "location", location: target },
          { kind: "location", location: destination },
        ]);
        if (applied === undefined) continue;
        let priority = 7_600;
        const targetMon = itemMon(targetItem);
        if (targetMon?.color === otherColor(perspective)) priority += 400;
        const targetMana = itemMana(targetItem);
        if (targetMana?.kind === "supermana") priority += 600;
        if (
          targetMana?.kind === "regular" &&
          targetMana.color === otherColor(perspective)
        ) {
          priority += 460;
        }
        const after = exactTurnTacticalProjectionWithSearchHash(
          execution,
          applied.game,
          perspective,
          exactSearchStateHash(applied.game),
          flags,
        );
        if (execution.session.checkpoint()) return [];
        if (after.sameTurnScoreWindowValue > before.sameTurnScoreWindowValue) {
          priority +=
            (after.sameTurnScoreWindowValue - before.sameTurnScoreWindowValue) * 280;
        }
        if (after.spiritAssistedScore) {
          priority += 900 + after.spiritAssistedScoreValue * 120;
        }
        if (after.safeSupermanaProgress) {
          priority +=
            700 +
            progressPriorityBonus(
              before.safeSupermanaProgressSteps,
              after.safeSupermanaProgressSteps,
            );
        }
        if (after.safeOpponentManaProgress) {
          priority +=
            760 +
            progressPriorityBonus(
              before.safeOpponentManaProgressSteps,
              after.safeOpponentManaProgressSteps,
            );
        }
        if (after.spiritAssistedDenial) {
          priority += 820 + after.spiritAssistedDenialValue * 140;
        }
        const afterSafety = ownDrainerSafetyScore(
          execution,
          applied.game.board,
          perspective,
        );
        if (afterSafety > beforeSafety) priority += (afterSafety - beforeSafety) * 160;
        priority +=
          Math.max(BOARD_SIZE - locationDistance(destination, target), 0) * 20;
        result.push({
          family: TurnPlanFamily.SpiritImpact,
          action: {
            kind: "spirit-shift",
            actor: spirit,
            target,
            destination,
          },
          priority,
        });
      }
    }
  }
  result.sort((left, right) => {
    const order = compareNumber(right.priority, left.priority);
    return order !== 0 ? order : compareActionKeys(left.action, right.action);
  });
  return result.slice(0, 12);
}

export function manaTempoSeeds(game: MonsGame, perspective: Color): ActionSeed[] {
  if (
    !game.playerCanMoveMana() ||
    findAwakeDrainerLocation(game.board, perspective) !== undefined
  ) {
    return [];
  }
  const result: ActionSeed[] = [];
  for (const [from, item] of game.board.entries()) {
    if (
      item.kind !== "mana" ||
      item.mana.kind !== "regular" ||
      item.mana.color !== perspective
    ) {
      continue;
    }
    for (const to of nearbyLocations(from)) {
      if (!manaMoveDestinationAllowed(game.board, to)) continue;
      const ownGain =
        distanceToNearestPool(from, perspective) -
        distanceToNearestPool(to, perspective);
      const opponent = otherColor(perspective);
      const opponentGain =
        distanceToNearestPool(from, opponent) - distanceToNearestPool(to, opponent);
      if (ownGain <= 0 || opponentGain > 0) continue;
      result.push({
        family: TurnPlanFamily.ManaTempo,
        action: { kind: "move-mana", from, to },
        priority: 6_900 + ownGain * 200 - Math.max(opponentGain, 0) * 200,
      });
    }
  }
  return result;
}
