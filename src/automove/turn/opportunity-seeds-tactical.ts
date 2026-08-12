import type { AutomoveExecutionContext } from "../core/execution-context.js";
import type { Color } from "../../api/types.js";
import {
  isMonFainted,
  itemMon,
  manaScore,
  otherColor,
} from "../../engine/model/domain.js";
import { MonsGame } from "../../engine/game/mons-game.js";
import { locationDistance, nearbyLocations } from "../../engine/board/geometry.js";
import { exactSearchStateHash } from "../exact/hash.js";
import {
  actorCanAttackFromItem,
  actorCanAttackTargetNow,
  actorCanBombFromItem,
  actorCanBombTargetNow,
  distanceToNearestPool,
  opponentDrainerKillIsHighValue,
  remainingMovesForColor,
} from "./action-rules.js";
import {
  activeTurnScoreWindowWithSearchHash,
  findAwakeDrainerLocation,
  opponentCanWinImmediately,
  ownDrainerSafetyScore,
} from "./evaluation.js";
import { TurnPlanFamily, type ActionSeed } from "./model.js";

export function immediateScoreSeeds(game: MonsGame, perspective: Color): ActionSeed[] {
  const result: ActionSeed[] = [];
  for (const [at, item] of game.board.entries()) {
    if (
      item.kind !== "mon-with-mana" ||
      item.mon.color !== perspective ||
      isMonFainted(item.mon)
    ) {
      continue;
    }
    const beforeDistance = distanceToNearestPool(at, perspective);
    for (const next of nearbyLocations(at)) {
      const afterDistance = distanceToNearestPool(next, perspective);
      if (afterDistance > beforeDistance) continue;
      result.push({
        family: TurnPlanFamily.ImmediateScore,
        action: {
          kind: "score-carry",
          actor: at,
          wanted: item.mana,
          step: next,
        },
        priority:
          9_800 +
          Math.max(beforeDistance - afterDistance, 0) * 180 +
          manaScore(item.mana, perspective) * 120,
      });
    }
  }
  return result;
}

export function denyWindowSeeds(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
): ActionSeed[] {
  const opponent = otherColor(perspective);
  const pressure = activeTurnScoreWindowWithSearchHash(
    execution,
    game,
    opponent,
    exactSearchStateHash(game),
  );
  if (pressure <= 0 && !opponentCanWinImmediately(execution, game, perspective))
    return [];
  const result = attackFamilySeeds(
    game,
    perspective,
    TurnPlanFamily.DenyOpponentWindow,
    9_400 + pressure * 240,
  );
  const drainer = findAwakeDrainerLocation(game.board, perspective);
  if (drainer === undefined) return result;
  const beforeSafety = ownDrainerSafetyScore(execution, game.board, perspective);
  const beforeDistance = distanceToNearestPool(drainer, perspective);
  for (const next of nearbyLocations(drainer)) {
    if (
      distanceToNearestPool(next, perspective) > beforeDistance + 1 &&
      beforeSafety >= 0
    ) {
      continue;
    }
    result.push({
      family: TurnPlanFamily.DenyOpponentWindow,
      action: { kind: "safety-retreat", actor: drainer, to: next },
      priority: 9_100 + Math.abs(beforeSafety) * 220,
    });
  }
  return result;
}

export function drainerKillSeeds(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
): ActionSeed[] {
  const target = findAwakeDrainerLocation(game.board, otherColor(perspective));
  return target === undefined ||
    !opponentDrainerKillIsHighValue(execution, game, perspective, target)
    ? []
    : attackFamilySeeds(game, perspective, TurnPlanFamily.DrainerKill, 9_000);
}

function attackFamilySeeds(
  game: MonsGame,
  perspective: Color,
  family: TurnPlanFamily,
  basePriority: number,
): ActionSeed[] {
  const target = findAwakeDrainerLocation(game.board, otherColor(perspective));
  if (target === undefined) return [];
  const result: ActionSeed[] = [];
  const canUseAction = game.playerCanUseAction();
  const remainingMoves = remainingMovesForColor(game, perspective);
  for (const [at, item] of game.board.entries()) {
    const mon = itemMon(item);
    if (mon?.color !== perspective || isMonFainted(mon)) continue;
    const canAttack = canUseAction && actorCanAttackFromItem(item);
    const canBomb = canUseAction && actorCanBombFromItem(item);
    if (
      canAttack &&
      actorCanAttackTargetNow(game.board, at, target, item, perspective)
    ) {
      result.push({
        family,
        action: { kind: "attack", actor: at, target },
        priority: basePriority,
      });
    }
    if (canBomb && actorCanBombTargetNow(game.board, at, target, item, perspective)) {
      result.push({
        family,
        action: { kind: "bomb", actor: at, target },
        priority: basePriority - 80,
      });
    }
    if (remainingMoves <= 0 || (!canAttack && !canBomb)) continue;
    for (const next of nearbyLocations(at)) {
      if (locationDistance(next, target) >= locationDistance(at, target)) continue;
      if (family === TurnPlanFamily.DrainerKill) {
        const preview = game.board.fork();
        preview.delete(at);
        preview.set(next, item);
        const threatensNow =
          (canAttack &&
            actorCanAttackTargetNow(preview, next, target, item, perspective)) ||
          (canBomb && actorCanBombTargetNow(preview, next, target, item, perspective));
        if (!threatensNow) continue;
      }
      result.push({
        family,
        action: { kind: "walk", actor: at, to: next },
        priority:
          basePriority -
          200 +
          (locationDistance(at, target) - locationDistance(next, target)) * 80,
      });
    }
  }
  return result;
}
