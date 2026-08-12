import { Color } from "../../api/types.js";
import { manaScore, otherColor } from "../../engine/model/domain.js";
import { TARGET_SCORE } from "../../engine/board/config.js";
import { locationDistance } from "../../engine/board/geometry.js";
import type { ExactColorSummary } from "../exact/types.js";
import { colorSlot, type ScoringEvalContext } from "./context.js";
import type { ScoringWeights } from "./profile-schema.js";
import {
  bestDrainerPickupPathWithSnapshot,
  requireExactSummary,
} from "./threat-support.js";

type ScorePathWindow = {
  readonly bestSteps: number | undefined;
  readonly multiPressure: number;
};

type ImmediateScoreWindow = {
  readonly bestScore: number;
  readonly multiPressure: number;
};

export function scaleByBp(value: number, basisPoints: number): number {
  return Math.trunc((Math.trunc(value) * Math.trunc(basisPoints)) / 10_000);
}

function insertLowestStep(topSteps: number[], step: number): void {
  if (step >= (topSteps[2] ?? 0x7fff_ffff)) return;
  if (step < (topSteps[0] ?? 0x7fff_ffff)) {
    topSteps[2] = topSteps[1] ?? 0x7fff_ffff;
    topSteps[1] = topSteps[0] ?? 0x7fff_ffff;
    topSteps[0] = step;
  } else if (step < (topSteps[1] ?? 0x7fff_ffff)) {
    topSteps[2] = topSteps[1] ?? 0x7fff_ffff;
    topSteps[1] = step;
  } else {
    topSteps[2] = step;
  }
}

function insertTopScore(topScores: number[], value: number): void {
  if (value <= (topScores[2] ?? 0)) return;
  if (value > (topScores[0] ?? 0)) {
    topScores[2] = topScores[1] ?? 0;
    topScores[1] = topScores[0] ?? 0;
    topScores[0] = value;
  } else if (value > (topScores[1] ?? 0)) {
    topScores[2] = topScores[1] ?? 0;
    topScores[1] = value;
  } else {
    topScores[2] = value;
  }
}

function exactScorePathWindowForContext(
  context: ScoringEvalContext,
  color: Color,
  exactSummary: ExactColorSummary,
  includeRegularManaMoveWindows: boolean,
): ScorePathWindow {
  if (!includeRegularManaMoveWindows) return exactSummary.scorePathWindow;
  const summary = context.boardSummary();
  const candidateSteps = summary.regularManaScorePathSteps[colorSlot(color)];
  const bestSteps =
    candidateSteps === undefined
      ? exactSummary.scorePathWindow.bestSteps
      : exactSummary.scorePathWindow.bestSteps === undefined
        ? candidateSteps
        : Math.min(candidateSteps, exactSummary.scorePathWindow.bestSteps);
  return {
    bestSteps,
    multiPressure: exactSummary.scorePathWindow.multiPressure,
  };
}

function scorePathWindowToAnyPoolForContext(
  context: ScoringEvalContext,
  color: Color,
  includeDrainerPickups: boolean,
  includeRegularManaMoveWindows: boolean,
): ScorePathWindow {
  const summary = context.boardSummary();
  const topSteps = [0x7fff_ffff, 0x7fff_ffff, 0x7fff_ffff];
  for (const carrier of summary.liveManaCarriers[colorSlot(color)]) {
    insertLowestStep(topSteps, carrier.scoreSteps + 1);
  }
  if (includeDrainerPickups) {
    const snapshot = context.manaPathSnapshot();
    for (const location of summary.liveDrainerLocations[colorSlot(color)]) {
      const pickup = bestDrainerPickupPathWithSnapshot(snapshot, color, location);
      if (pickup !== undefined) insertLowestStep(topSteps, pickup[0] + 1);
    }
  }
  if (includeRegularManaMoveWindows) {
    const candidate = summary.regularManaScorePathSteps[colorSlot(color)];
    if (candidate !== undefined) insertLowestStep(topSteps, candidate);
  }
  const bestSteps = topSteps[0] === 0x7fff_ffff ? undefined : topSteps[0];
  let multiPressure = 0;
  if (topSteps[1] !== 0x7fff_ffff) {
    multiPressure = multiPressure + Math.trunc(70 / Math.max(1, topSteps[1] ?? 1));
  }
  if (topSteps[2] !== 0x7fff_ffff) {
    multiPressure = multiPressure + Math.trunc(40 / Math.max(1, topSteps[2] ?? 1));
  }
  return { bestSteps, multiPressure };
}

function exactImmediateScoreWindowForContext(
  context: ScoringEvalContext,
  color: Color,
  exactSummary: ExactColorSummary,
  allowManaMove: boolean,
): ImmediateScoreWindow {
  if (!allowManaMove) return exactSummary.immediateWindow;
  const regularScore = context.boardSummary().regularManaMoveScores[colorSlot(color)];
  return {
    bestScore: Math.max(exactSummary.immediateWindow.bestScore, regularScore),
    multiPressure: exactSummary.immediateWindow.multiPressure,
  };
}

function immediateScoreWindowSummaryForContext(
  context: ScoringEvalContext,
  color: Color,
  remainingMonMoves: number,
  includeDrainerPickups: boolean,
  includeRegularManaMoveWindows: boolean,
  allowManaMove: boolean,
): ImmediateScoreWindow {
  if (remainingMonMoves <= 0) return { bestScore: 0, multiPressure: 0 };
  const summary = context.boardSummary();
  const topScores = [0, 0, 0];
  for (const carrier of summary.liveManaCarriers[colorSlot(color)]) {
    if (carrier.scoreSteps <= remainingMonMoves) {
      insertTopScore(topScores, manaScore(carrier.mana, color));
    }
  }
  if (includeDrainerPickups) {
    const snapshot = context.manaPathSnapshot();
    for (const location of summary.liveDrainerLocations[colorSlot(color)]) {
      let bestPickupScore = 0;
      for (const candidate of snapshot.candidates) {
        const pickupSteps = locationDistance(location, candidate.location);
        if (pickupSteps + candidate.scoreSteps <= remainingMonMoves) {
          bestPickupScore = Math.max(bestPickupScore, manaScore(candidate.mana, color));
        }
      }
      if (bestPickupScore > 0) insertTopScore(topScores, bestPickupScore);
    }
  }
  if (includeRegularManaMoveWindows && allowManaMove) {
    const regularScore = summary.regularManaMoveScores[colorSlot(color)];
    if (regularScore > 0) insertTopScore(topScores, regularScore);
  }
  return {
    bestScore: topScores[0] ?? 0,
    multiPressure: (topScores[1] ?? 0) * 70 + (topScores[2] ?? 0) * 35,
  };
}

export function scorePreferabilityRaceWindows(
  color: Color,
  weights: ScoringWeights,
  context: ScoringEvalContext,
  useHeuristicFormula: boolean,
  includeRegularManaMoveWindows: boolean,
  myExactSummary: ExactColorSummary | undefined,
  opponentExactSummary: ExactColorSummary | undefined,
  initialScore: number,
): number {
  let score = initialScore;
  const myScorePathWindow = useHeuristicFormula
    ? scorePathWindowToAnyPoolForContext(
        context,
        color,
        false,
        includeRegularManaMoveWindows,
      )
    : exactScorePathWindowForContext(
        context,
        color,
        requireExactSummary(myExactSummary),
        includeRegularManaMoveWindows,
      );
  const opponentScorePathWindow = useHeuristicFormula
    ? scorePathWindowToAnyPoolForContext(
        context,
        otherColor(color),
        false,
        includeRegularManaMoveWindows,
      )
    : exactScorePathWindowForContext(
        context,
        otherColor(color),
        requireExactSummary(opponentExactSummary),
        includeRegularManaMoveWindows,
      );
  if (myScorePathWindow.bestSteps !== undefined) {
    score =
      score +
      scaleByBp(
        Math.trunc(
          weights.race.scoreRacePathProgress / Math.max(1, myScorePathWindow.bestSteps),
        ),
        10_000,
      );
    if (!useHeuristicFormula) {
      score =
        score +
        scaleByBp(
          Math.trunc(
            (weights.race.scoreRaceMultiPath * myScorePathWindow.multiPressure) / 100,
          ),
          10_000,
        );
    }
  }
  if (opponentScorePathWindow.bestSteps !== undefined) {
    score =
      score +
      -scaleByBp(
        Math.trunc(
          weights.race.opponentScoreRacePathProgress /
            Math.max(1, opponentScorePathWindow.bestSteps),
        ),
        10_000,
      );
    if (!useHeuristicFormula) {
      score =
        score +
        -scaleByBp(
          Math.trunc(
            (weights.race.opponentScoreRaceMultiPath *
              opponentScorePathWindow.multiPressure) /
              100,
          ),
          10_000,
        );
    }
  }
  return score;
}

export function scorePreferabilityImmediateWindows(
  color: Color,
  weights: ScoringWeights,
  context: ScoringEvalContext,
  useHeuristicFormula: boolean,
  includeRegularManaMoveWindows: boolean,
  includeMatchPointWindow: boolean,
  nextTurnWindowScaleBp: number,
  remainingMonMovesForActive: number,
  myExactSummary: ExactColorSummary | undefined,
  opponentExactSummary: ExactColorSummary | undefined,
  myScoreNow: number,
  opponentScoreNow: number,
  initialScore: number,
): number {
  const game = context.game;
  let score = initialScore;
  if (game.activeColor === color) {
    const immediateWindow = useHeuristicFormula
      ? immediateScoreWindowSummaryForContext(
          context,
          color,
          remainingMonMovesForActive,
          false,
          includeRegularManaMoveWindows,
          includeRegularManaMoveWindows && game.playerCanMoveMana(),
        )
      : exactImmediateScoreWindowForContext(
          context,
          color,
          requireExactSummary(myExactSummary),
          includeRegularManaMoveWindows && game.playerCanMoveMana(),
        );
    score =
      score +
      scaleByBp(weights.race.immediateScoreWindow * immediateWindow.bestScore, 10_000);
    if (!useHeuristicFormula) {
      score =
        score +
        scaleByBp(
          Math.trunc(
            (weights.race.immediateScoreMultiWindow * immediateWindow.multiPressure) /
              100,
          ),
          10_000,
        );
      const opponentNextTurnWindow = exactImmediateScoreWindowForContext(
        context,
        otherColor(color),
        requireExactSummary(opponentExactSummary),
        includeRegularManaMoveWindows,
      );
      score =
        score +
        -scaleByBp(
          Math.trunc(
            (weights.race.opponentImmediateScoreWindow *
              opponentNextTurnWindow.bestScore *
              nextTurnWindowScaleBp) /
              10_000,
          ),
          10_000,
        );
      score =
        score +
        -scaleByBp(
          Math.trunc(
            (weights.race.opponentImmediateScoreMultiWindow *
              opponentNextTurnWindow.multiPressure *
              nextTurnWindowScaleBp) /
              1_000_000,
          ),
          10_000,
        );
      if (includeMatchPointWindow) {
        if (myScoreNow + immediateWindow.bestScore >= TARGET_SCORE) {
          score = score + weights.mana.immediateWinningCarrier;
        }
        if (opponentScoreNow + opponentNextTurnWindow.bestScore >= TARGET_SCORE) {
          score = score + -weights.mana.immediateWinningCarrier;
        }
      }
    }
  } else {
    const opponentImmediateWindow = useHeuristicFormula
      ? immediateScoreWindowSummaryForContext(
          context,
          otherColor(color),
          remainingMonMovesForActive,
          false,
          includeRegularManaMoveWindows,
          includeRegularManaMoveWindows && game.playerCanMoveMana(),
        )
      : exactImmediateScoreWindowForContext(
          context,
          otherColor(color),
          requireExactSummary(opponentExactSummary),
          includeRegularManaMoveWindows && game.playerCanMoveMana(),
        );
    score =
      score +
      -scaleByBp(
        weights.race.opponentImmediateScoreWindow * opponentImmediateWindow.bestScore,
        10_000,
      );
    if (!useHeuristicFormula) {
      score =
        score +
        -scaleByBp(
          Math.trunc(
            (weights.race.opponentImmediateScoreMultiWindow *
              opponentImmediateWindow.multiPressure) /
              100,
          ),
          10_000,
        );
      const myNextTurnWindow = exactImmediateScoreWindowForContext(
        context,
        color,
        requireExactSummary(myExactSummary),
        includeRegularManaMoveWindows,
      );
      score =
        score +
        scaleByBp(
          Math.trunc(
            (weights.race.immediateScoreWindow *
              myNextTurnWindow.bestScore *
              nextTurnWindowScaleBp) /
              10_000,
          ),
          10_000,
        );
      score =
        score +
        scaleByBp(
          Math.trunc(
            (weights.race.immediateScoreMultiWindow *
              myNextTurnWindow.multiPressure *
              nextTurnWindowScaleBp) /
              1_000_000,
          ),
          10_000,
        );
      if (includeMatchPointWindow) {
        if (opponentScoreNow + opponentImmediateWindow.bestScore >= TARGET_SCORE) {
          score = score + -weights.mana.immediateWinningCarrier;
        }
        if (myScoreNow + myNextTurnWindow.bestScore >= TARGET_SCORE) {
          score = score + weights.mana.immediateWinningCarrier;
        }
      }
    }
  }
  return score;
}
