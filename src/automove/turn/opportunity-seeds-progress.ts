import type { AutomoveExecutionContext } from "../core/execution-context.js";
import type { Color, Mana } from "../../api/types.js";
import {
  isMonFainted,
  itemMon,
  manaEquals,
  otherColor,
} from "../../engine/model/domain.js";
import { MonsGame } from "../../engine/game/mons-game.js";
import {
  BOARD_SIZE,
  locationDistance,
  nearbyLocations,
} from "../../engine/board/geometry.js";
import {
  EXACT_TURN_TACTICAL_NEED_OPPONENT_MANA_PROGRESS,
  EXACT_TURN_TACTICAL_NEED_SCORE_WINDOW,
  EXACT_TURN_TACTICAL_NEED_SUPERMANA_PROGRESS,
} from "../exact/types.js";
import { exactBestScoreStepsOnBoard } from "../exact/mana-carrier.js";
import { exactSearchStateHash } from "../exact/hash.js";
import { exactSecureSpecificManaPathFrom } from "../exact/secure-mana.js";
import { exactTurnTacticalProjectionWithSearchHash } from "../exact/turn-opportunity.js";
import type { Hash64 } from "../core/hash64.js";
import { applyInputsForSearchWithEvents } from "../transitions/simulation.js";
import {
  distanceToNearestPool,
  nearestWantedManaLocation,
  remainingMovesForColor,
  walkDestinationPlausible,
} from "./action-rules.js";
import {
  findAwakeDrainerLocation,
  opponentCanWinImmediately,
  ownDrainerSafetyScore,
} from "./evaluation.js";
import {
  TurnEngineMode,
  TurnPlanFamily,
  type ActionSeed,
  type TurnEngineConfig,
} from "./model.js";

export function safeSupermanaProgressSeeds(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
): ActionSeed[] {
  return safeProgressSeeds(
    execution,
    game,
    perspective,
    { kind: "supermana" },
    TurnPlanFamily.SafeSupermanaProgress,
    8_900,
  );
}

export function safeOpponentManaProgressSeeds(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
): ActionSeed[] {
  return safeProgressSeeds(
    execution,
    game,
    perspective,
    { kind: "regular", color: otherColor(perspective) },
    TurnPlanFamily.SafeOpponentManaProgress,
    8_600,
  );
}

type SafeProgressExactSnapshot = {
  readonly progressSteps: number | undefined;
  readonly scorePathBestSteps: number | undefined;
  readonly sameTurnScoreWindowValue: number;
};

function safeProgressExactSnapshot(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  wanted: Mana,
  stateHash: Hash64,
): SafeProgressExactSnapshot {
  if (execution.session.checkpoint()) {
    return {
      progressSteps: undefined,
      scorePathBestSteps: undefined,
      sameTurnScoreWindowValue: 0,
    };
  }
  const opponent = otherColor(perspective);
  const flags =
    wanted.kind === "supermana"
      ? EXACT_TURN_TACTICAL_NEED_SUPERMANA_PROGRESS |
        EXACT_TURN_TACTICAL_NEED_SCORE_WINDOW
      : wanted.color === opponent
        ? EXACT_TURN_TACTICAL_NEED_OPPONENT_MANA_PROGRESS
        : 0;
  const projection = exactTurnTacticalProjectionWithSearchHash(
    execution,
    game,
    perspective,
    stateHash,
    flags,
  );
  if (execution.session.checkpoint()) {
    return {
      progressSteps: undefined,
      scorePathBestSteps: undefined,
      sameTurnScoreWindowValue: 0,
    };
  }
  return {
    progressSteps:
      wanted.kind === "supermana"
        ? projection.safeSupermanaProgressSteps
        : wanted.color === opponent
          ? projection.safeOpponentManaProgressSteps
          : undefined,
    scorePathBestSteps: exactBestScoreStepsOnBoard(execution, game.board, perspective),
    sameTurnScoreWindowValue: projection.sameTurnScoreWindowValue,
  };
}

function safeProgressSeeds(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  wanted: Mana,
  family: TurnPlanFamily,
  basePriority: number,
): ActionSeed[] {
  if (execution.session.checkpoint()) return [];
  const drainer = findAwakeDrainerLocation(game.board, perspective);
  if (drainer === undefined) return [];
  const result: ActionSeed[] = [];
  const beforeExact = safeProgressExactSnapshot(
    execution,
    game,
    perspective,
    wanted,
    exactSearchStateHash(game),
  );
  if (execution.session.cancelled) return [];
  const beforeSafety = ownDrainerSafetyScore(execution, game.board, perspective);
  const path = exactSecureSpecificManaPathFrom(
    execution,
    game,
    perspective,
    drainer,
    wanted,
  );
  const pathStep = path?.[0];
  if (pathStep !== undefined) {
    result.push({
      family,
      action: { kind: "score-carry", actor: drainer, wanted, step: pathStep },
      priority: basePriority + Math.max(BOARD_SIZE * 2 - (path?.length ?? 0), 0) * 120,
    });
  }
  if (execution.session.checkpoint()) return [];
  if (remainingMovesForColor(game, perspective) > 0) {
    const target = nearestWantedManaLocation(game.board, wanted);
    if (target !== undefined) {
      const beforeDistance = locationDistance(drainer, target);
      const beforeSteps = beforeExact.progressSteps ?? BOARD_SIZE * 3;
      const beforeScorePath = beforeExact.scorePathBestSteps ?? BOARD_SIZE * 3;
      for (const next of nearbyLocations(drainer)) {
        if (execution.session.checkpoint()) return [];
        if (!walkDestinationPlausible(game.board, drainer, next)) continue;
        const applied = applyInputsForSearchWithEvents(game, [
          { kind: "location", location: drainer },
          { kind: "location", location: next },
        ]);
        if (
          applied === undefined ||
          opponentCanWinImmediately(execution, applied.game, perspective)
        ) {
          continue;
        }
        const afterExact = safeProgressExactSnapshot(
          execution,
          applied.game,
          perspective,
          wanted,
          exactSearchStateHash(applied.game),
        );
        const sessionAfterExactSnapshot = execution.session;
        if (sessionAfterExactSnapshot.cancelled) return [];
        const afterSafety = ownDrainerSafetyScore(
          execution,
          applied.game.board,
          perspective,
        );
        const afterSteps = afterExact.progressSteps ?? BOARD_SIZE * 3;
        const afterScorePath = afterExact.scorePathBestSteps ?? BOARD_SIZE * 3;
        const exactImproved =
          afterSteps < beforeSteps ||
          (afterSteps <= beforeSteps && afterScorePath < beforeScorePath);
        if (!exactImproved && afterSafety < beforeSafety) continue;
        let priority =
          basePriority -
          180 +
          Math.max(beforeDistance - locationDistance(next, target), 0) * 110 +
          (afterSafety - beforeSafety) * 120;
        if (exactImproved) {
          priority +=
            (beforeSteps - afterSteps) * 220 + (beforeScorePath - afterScorePath) * 180;
        }
        if (wanted.kind === "supermana" && afterExact.sameTurnScoreWindowValue > 0) {
          priority += afterExact.sameTurnScoreWindowValue * 260;
        }
        result.push({
          family,
          action: { kind: "walk", actor: drainer, to: next },
          priority,
        });
      }
    }
  }
  const drainerItem = game.board.get(drainer);
  if (drainerItem?.kind === "mon-with-mana" && manaEquals(drainerItem.mana, wanted)) {
    const beforeDistance = distanceToNearestPool(drainer, perspective);
    for (const next of nearbyLocations(drainer)) {
      const afterDistance = distanceToNearestPool(next, perspective);
      if (afterDistance > beforeDistance) continue;
      result.push({
        family,
        action: { kind: "score-carry", actor: drainer, wanted, step: next },
        priority: basePriority + Math.max(beforeDistance - afterDistance, 0) * 150,
      });
    }
  }
  return result;
}

export function safetyRecoverySeeds(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
): ActionSeed[] {
  const drainer = findAwakeDrainerLocation(game.board, perspective);
  if (drainer === undefined) return [];
  const beforeSafety = ownDrainerSafetyScore(execution, game.board, perspective);
  const result: ActionSeed[] = [];
  for (const next of nearbyLocations(drainer)) {
    const applied = applyInputsForSearchWithEvents(game, [
      { kind: "location", location: drainer },
      { kind: "location", location: next },
    ]);
    if (applied === undefined) continue;
    const afterSafety = ownDrainerSafetyScore(
      execution,
      applied.game.board,
      perspective,
    );
    if (afterSafety <= beforeSafety) continue;
    result.push({
      family: TurnPlanFamily.DrainerSafetyRecovery,
      action: { kind: "safety-retreat", actor: drainer, to: next },
      priority:
        8_300 + Math.abs(beforeSafety) * 220 + (afterSafety - beforeSafety) * 260,
    });
  }
  return result;
}

export function fallbackWalkSeeds(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
): ActionSeed[] {
  if (remainingMovesForColor(game, perspective) <= 0) return [];
  const result: ActionSeed[] = [];
  const beforeSafety = ownDrainerSafetyScore(execution, game.board, perspective);
  const drainer = findAwakeDrainerLocation(game.board, perspective);
  if (drainer !== undefined) {
    const beforePoolDistance = distanceToNearestPool(drainer, perspective);
    for (const next of nearbyLocations(drainer)) {
      if (!walkDestinationPlausible(game.board, drainer, next)) continue;
      const applied = applyInputsForSearchWithEvents(game, [
        { kind: "location", location: drainer },
        { kind: "location", location: next },
      ]);
      if (
        applied === undefined ||
        opponentCanWinImmediately(execution, applied.game, perspective)
      )
        continue;
      const afterSafety = ownDrainerSafetyScore(
        execution,
        applied.game.board,
        perspective,
      );
      if (afterSafety < beforeSafety) continue;
      const afterPoolDistance = distanceToNearestPool(next, perspective);
      result.push({
        family:
          afterSafety > beforeSafety
            ? TurnPlanFamily.DrainerSafetyRecovery
            : TurnPlanFamily.ManaTempo,
        action: { kind: "walk", actor: drainer, to: next },
        priority:
          7_200 +
          Math.max(beforePoolDistance - afterPoolDistance, 0) * 140 +
          (afterSafety - beforeSafety) * 240,
      });
    }
  }
  if (result.length !== 0) return result;
  for (const [actor, item] of game.board.entries()) {
    const mon = itemMon(item);
    if (mon?.color !== perspective || isMonFainted(mon)) continue;
    for (const to of nearbyLocations(actor)) {
      if (!walkDestinationPlausible(game.board, actor, to)) continue;
      const applied = applyInputsForSearchWithEvents(game, [
        { kind: "location", location: actor },
        { kind: "location", location: to },
      ]);
      if (
        applied === undefined ||
        opponentCanWinImmediately(execution, applied.game, perspective)
      )
        continue;
      result.push({
        family: TurnPlanFamily.ManaTempo,
        action: { kind: "walk", actor, to },
        priority: 6_800,
      });
    }
  }
  return result;
}

function bestFollowUpSafetyRecoveryPriority(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
): number | undefined {
  const drainer = findAwakeDrainerLocation(game.board, perspective);
  if (drainer === undefined) return undefined;
  const beforeSafety = ownDrainerSafetyScore(execution, game.board, perspective);
  let best: number | undefined;
  for (const next of nearbyLocations(drainer)) {
    if (!walkDestinationPlausible(game.board, drainer, next)) continue;
    const applied = applyInputsForSearchWithEvents(game, [
      { kind: "location", location: drainer },
      { kind: "location", location: next },
    ]);
    if (applied === undefined) continue;
    const afterSafety = ownDrainerSafetyScore(
      execution,
      applied.game.board,
      perspective,
    );
    if (afterSafety <= beforeSafety) continue;
    const priority =
      8_300 + Math.abs(beforeSafety) * 220 + (afterSafety - beforeSafety) * 260;
    best = best === undefined ? priority : Math.max(best, priority);
  }
  return best;
}

export function riskyRecoverySetupSeeds(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: TurnEngineConfig,
): ActionSeed[] {
  if (
    config.mode !== TurnEngineMode.Production ||
    remainingMovesForColor(game, perspective) <= 0
  ) {
    return [];
  }
  const drainer = findAwakeDrainerLocation(game.board, perspective);
  if (drainer === undefined) return [];
  const beforeSafety = ownDrainerSafetyScore(execution, game.board, perspective);
  const beforePoolDistance = distanceToNearestPool(drainer, perspective);
  const result: ActionSeed[] = [];
  for (const next of nearbyLocations(drainer)) {
    if (!walkDestinationPlausible(game.board, drainer, next)) continue;
    const applied = applyInputsForSearchWithEvents(game, [
      { kind: "location", location: drainer },
      { kind: "location", location: next },
    ]);
    if (
      applied === undefined ||
      opponentCanWinImmediately(execution, applied.game, perspective)
    )
      continue;
    const afterSafety = ownDrainerSafetyScore(
      execution,
      applied.game.board,
      perspective,
    );
    const afterPoolDistance = distanceToNearestPool(next, perspective);
    if (afterSafety >= beforeSafety || afterPoolDistance >= beforePoolDistance)
      continue;
    const recoveryPriority = bestFollowUpSafetyRecoveryPriority(
      execution,
      applied.game,
      perspective,
    );
    if (recoveryPriority === undefined) continue;
    result.push({
      family: TurnPlanFamily.ManaTempo,
      action: { kind: "walk", actor: drainer, to: next },
      priority:
        8_000 +
        Math.max(beforePoolDistance - afterPoolDistance, 0) * 260 +
        Math.trunc(recoveryPriority / 20) -
        Math.max(beforeSafety - afterSafety, 0) * 120,
    });
  }
  return result;
}
