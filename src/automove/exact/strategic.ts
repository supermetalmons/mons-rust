import { MONS_MOVES_PER_TURN } from "../../engine/board/config.js";
import { Color } from "../../api/types.js";
import { isMonFainted, manaScore } from "../../engine/model/domain.js";
import { MonsGame } from "../../engine/game/mons-game.js";
import type { AutomoveExecutionContext } from "../core/execution-context.js";
import type { Hash64 } from "../core/hash64.js";
import { exactCaches } from "./cache.js";
import { findAwakeDrainer } from "./drainer-safety.js";
import { exactBoardHash, exactSearchStateHash } from "./hash.js";
import {
  exactBestDrainerPickupPathWithHash,
  exactCarrierStepsToAnyPoolWithHash,
  exactDrainerToAnyManaSteps,
} from "./mana-carrier.js";
import { exactPassiveSpiritSummary } from "./spirit-passive.js";
import {
  defaultColorSummary,
  type ExactColorSummary,
  ExactStrategicAnalysis,
} from "./types.js";

function multiPressureFromSteps(steps: readonly number[]): number {
  const second = steps[1];
  const third = steps[2];
  return (
    (second === undefined ? 0 : Math.trunc(70 / Math.max(second, 1))) +
    (third === undefined ? 0 : Math.trunc(40 / Math.max(third, 1)))
  );
}

function multiPressureFromScores(scores: readonly number[]): number {
  return (scores[1] ?? 0) * 70 + (scores[2] ?? 0) * 35;
}

function buildColorSummary(
  context: AutomoveExecutionContext,
  game: MonsGame,
  color: Color,
): ExactColorSummary {
  if (context.session.checkpointWithReserve(20)) return defaultColorSummary();
  const fullTurnMoves =
    game.activeColor === color
      ? Math.max(MONS_MOVES_PER_TURN - game.monsMovesCount, 0)
      : MONS_MOVES_PER_TURN;
  const canUseAction = game.activeColor === color ? game.playerCanUseAction() : true;
  const boardHash = exactBoardHash(game.board);
  const carrierSteps: number[] = [];
  let bestCarrierSteps: number | undefined;
  for (const [location, item] of game.board.entries()) {
    if (context.session.checkpoint()) return defaultColorSummary();
    if (
      item.kind !== "mon-with-mana" ||
      item.mon.color !== color ||
      isMonFainted(item.mon)
    ) {
      continue;
    }
    const steps = exactCarrierStepsToAnyPoolWithHash(
      context,
      game.board,
      location,
      item.mana,
      boardHash,
    );
    if (steps !== undefined) {
      bestCarrierSteps =
        bestCarrierSteps === undefined ? steps : Math.min(bestCarrierSteps, steps);
      carrierSteps.push(steps);
    }
  }
  const drainer = findAwakeDrainer(game.board, color);
  const bestDrainerPickup =
    drainer === undefined
      ? undefined
      : exactBestDrainerPickupPathWithHash(
          context,
          game.board,
          color,
          drainer,
          undefined,
          boardHash,
        );
  if (context.session.checkpointWithReserve(20)) return defaultColorSummary();
  const bestDrainerToManaSteps =
    drainer === undefined
      ? undefined
      : exactDrainerToAnyManaSteps(context, game.board, color, drainer);
  if (bestDrainerPickup !== undefined) {
    carrierSteps.push(bestDrainerPickup.totalMoves);
  }
  carrierSteps.sort((left, right) => left - right);
  const uniqueCarrierSteps = carrierSteps.filter(
    (value, index) => index === 0 || carrierSteps[index - 1] !== value,
  );
  const immediateScores: number[] = [];
  for (const [location, item] of game.board.entries()) {
    if (context.session.checkpoint()) return defaultColorSummary();
    if (
      item.kind !== "mon-with-mana" ||
      item.mon.color !== color ||
      isMonFainted(item.mon)
    ) {
      continue;
    }
    const steps = exactCarrierStepsToAnyPoolWithHash(
      context,
      game.board,
      location,
      item.mana,
      boardHash,
      fullTurnMoves,
    );
    if (steps !== undefined && steps <= fullTurnMoves) {
      immediateScores.push(manaScore(item.mana, color));
    }
  }
  if (
    bestDrainerPickup !== undefined &&
    bestDrainerPickup.totalMoves <= fullTurnMoves
  ) {
    immediateScores.push(bestDrainerPickup.manaValue);
  }
  const spirit = exactPassiveSpiritSummary(
    context,
    game.board,
    color,
    fullTurnMoves,
    canUseAction,
  );
  if (context.session.checkpointWithReserve(20)) return defaultColorSummary();
  immediateScores.sort((left, right) => right - left);
  return {
    scorePathWindow: {
      bestSteps: uniqueCarrierSteps[0],
      multiPressure: multiPressureFromSteps(uniqueCarrierSteps),
    },
    immediateWindow: {
      bestScore: immediateScores[0] ?? 0,
      multiPressure: multiPressureFromScores(immediateScores),
    },
    bestDrainerPickup,
    bestCarrierSteps,
    bestDrainerToManaSteps,
    spirit,
  };
}

function buildExactStrategicAnalysis(
  context: AutomoveExecutionContext,
  game: MonsGame,
): ExactStrategicAnalysis {
  if (context.session.checkpointWithReserve(20)) return new ExactStrategicAnalysis();
  const white = buildColorSummary(context, game, Color.White);
  if (context.session.checkpointWithReserve(20)) return new ExactStrategicAnalysis();
  const black = buildColorSummary(context, game, Color.Black);
  return context.session.checkpointWithReserve(20)
    ? new ExactStrategicAnalysis()
    : new ExactStrategicAnalysis(white, black);
}

export function exactStrategicAnalysisWithSearchHash(
  context: AutomoveExecutionContext,
  game: MonsGame,
  key: Hash64,
): ExactStrategicAnalysis {
  if (context.session.checkpointWithReserve(20)) return new ExactStrategicAnalysis();
  const cached = exactCaches(context).strategicAnalysis.get(key);
  if (cached !== undefined) return cached;
  const built = buildExactStrategicAnalysis(context, game);
  if (!context.session.cacheWriteAllowed) return new ExactStrategicAnalysis();
  exactCaches(context).strategicAnalysis.set(key, built);
  return built;
}

export function exactStrategicAnalysis(
  context: AutomoveExecutionContext,
  game: MonsGame,
): ExactStrategicAnalysis {
  return exactStrategicAnalysisWithSearchHash(
    context,
    game,
    exactSearchStateHash(game),
  );
}
