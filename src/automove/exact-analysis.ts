import { MONS_MOVES_PER_TURN, TARGET_SCORE } from "../engine/config.js";
import {
  Color,
  isMonFainted,
  type Mana,
  manaScore,
  otherColor,
} from "../engine/domain.js";
import { MonsGame } from "../engine/game.js";
import {
  canAttackTargetOnBoardWithHash,
  exactOwnDrainerSafetyScoreWithHash,
  findAwakeDrainer,
} from "./exact-attack.js";
import { colorKey, exactCaches, exactCacheTag } from "./exact-cache.js";
import { exactBoardHash, exactSearchStateHash } from "./exact-hash.js";
import {
  exactBestDrainerPickupPathWithHash,
  exactBestImmediateScoreOnBoard,
  exactBestScoreStepsOnBoard,
  exactCarrierStepsToAnyPoolWithHash,
  exactDrainerToAnyManaSteps,
  exactSecureSpecificManaStepsOnBoard,
} from "./exact-mana.js";
import {
  EXACT_TACTICAL_SPIRIT_NEED_DENIAL,
  EXACT_TACTICAL_SPIRIT_NEED_PROGRESS,
  EXACT_TACTICAL_SPIRIT_NEED_SCORE,
  exactPassiveSpiritSummary,
  exactTacticalSpiritSummary,
} from "./exact-spirit.js";
import {
  defaultColorSummary,
  defaultOpportunityContext,
  defaultSpiritSummary,
  defaultTurnSummary,
  defaultTurnTacticalProjection,
  EXACT_TURN_TACTICAL_ALL_FLAGS,
  EXACT_TURN_TACTICAL_NEED_OPPONENT_MANA_PROGRESS,
  EXACT_TURN_TACTICAL_NEED_SCORE_WINDOW,
  EXACT_TURN_TACTICAL_NEED_SPIRIT_DENIAL,
  EXACT_TURN_TACTICAL_NEED_SPIRIT_SCORE,
  EXACT_TURN_TACTICAL_NEED_SUPERMANA_PROGRESS,
  type ExactColorSummary,
  type ExactOpportunityBudget,
  type ExactOpportunityContext,
  ExactStrategicAnalysis,
  type ExactTurnSummary,
  type ExactTurnTacticalProjection,
} from "./exact-types.js";
import { type AutomoveExecutionContext } from "./execution-context.js";
import { type Hash64 } from "./hash64.js";

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
  const canUseAction =
    game.activeColor === color ? game.playerCanUseAction() : true;
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
        bestCarrierSteps === undefined
          ? steps
          : Math.min(bestCarrierSteps, steps);
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
  if (context.session.checkpointWithReserve(20))
    return new ExactStrategicAnalysis();
  const white = buildColorSummary(context, game, Color.White);
  if (context.session.checkpointWithReserve(20))
    return new ExactStrategicAnalysis();
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
  if (context.session.checkpointWithReserve(20))
    return new ExactStrategicAnalysis();
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

type ExactTurnProjectionFlags = number;

function exactSecureSpecificManaStepsThisTurn(
  context: AutomoveExecutionContext,
  game: MonsGame,
  color: Color,
  wanted: Mana,
): number | undefined {
  const remainingMoves =
    game.activeColor === color
      ? Math.max(MONS_MOVES_PER_TURN - game.monsMovesCount, 0)
      : MONS_MOVES_PER_TURN;
  return exactSecureSpecificManaStepsOnBoard(
    context,
    game.board,
    color,
    wanted,
    remainingMoves,
  );
}

function canAttackOpponentDrainerExactWithHash(
  context: AutomoveExecutionContext,
  game: MonsGame,
  color: Color,
  boardHash: Hash64,
): boolean {
  const target = findAwakeDrainer(game.board, otherColor(color));
  if (target === undefined) return false;
  return canAttackTargetOnBoardWithHash(
    context,
    game.board,
    boardHash,
    color,
    otherColor(color),
    target,
    game.activeColor === color
      ? Math.max(MONS_MOVES_PER_TURN - game.monsMovesCount, 0)
      : MONS_MOVES_PER_TURN,
    game.activeColor === color ? game.playerCanUseAction() : true,
  );
}

function turnTacticalProjectionForFlags(
  projection: ExactTurnTacticalProjection,
  flags: number,
): ExactTurnTacticalProjection {
  const needSupermana =
    (flags & EXACT_TURN_TACTICAL_NEED_SUPERMANA_PROGRESS) !== 0;
  const needOpponentMana =
    (flags & EXACT_TURN_TACTICAL_NEED_OPPONENT_MANA_PROGRESS) !== 0;
  const needSpiritScore = (flags & EXACT_TURN_TACTICAL_NEED_SPIRIT_SCORE) !== 0;
  const needSpiritDenial =
    (flags & EXACT_TURN_TACTICAL_NEED_SPIRIT_DENIAL) !== 0;
  const needScoreWindow = (flags & EXACT_TURN_TACTICAL_NEED_SCORE_WINDOW) !== 0;
  const includeScoreWindowDenial =
    needScoreWindow && (needOpponentMana || needSpiritDenial);
  const includeSpiritScore = needSpiritScore || needScoreWindow;
  const includeSpiritDenial = needSpiritDenial || includeScoreWindowDenial;
  const safeSupermanaProgressSteps = needSupermana
    ? projection.safeSupermanaProgressSteps
    : undefined;
  const safeOpponentManaProgressSteps = needOpponentMana
    ? projection.safeOpponentManaProgressSteps
    : undefined;
  const spiritAssistedDenial =
    includeSpiritDenial && projection.spiritAssistedDenial;
  return {
    safeSupermanaProgress: safeSupermanaProgressSteps !== undefined,
    safeSupermanaProgressSteps,
    safeOpponentManaProgress:
      safeOpponentManaProgressSteps !== undefined || spiritAssistedDenial,
    safeOpponentManaProgressSteps,
    spiritAssistedScore: includeSpiritScore && projection.spiritAssistedScore,
    spiritAssistedScoreValue: includeSpiritScore
      ? projection.spiritAssistedScoreValue
      : 0,
    spiritAssistedDenial,
    spiritAssistedDenialValue: includeSpiritDenial
      ? projection.spiritAssistedDenialValue
      : 0,
    sameTurnScoreWindowValue: needScoreWindow
      ? projection.sameTurnScoreWindowValue
      : 0,
  };
}

function buildExactTurnTacticalProjection(
  context: AutomoveExecutionContext,
  game: MonsGame,
  flags: number,
): ExactTurnTacticalProjection {
  if (context.session.checkpointWithReserve(20))
    return defaultTurnTacticalProjection();
  const color = game.activeColor;
  const remainingMoves = Math.max(MONS_MOVES_PER_TURN - game.monsMovesCount, 0);
  const needSupermana =
    (flags & EXACT_TURN_TACTICAL_NEED_SUPERMANA_PROGRESS) !== 0;
  const needOpponentMana =
    (flags & EXACT_TURN_TACTICAL_NEED_OPPONENT_MANA_PROGRESS) !== 0;
  const needSpiritScore = (flags & EXACT_TURN_TACTICAL_NEED_SPIRIT_SCORE) !== 0;
  const needSpiritDenial =
    (flags & EXACT_TURN_TACTICAL_NEED_SPIRIT_DENIAL) !== 0;
  const needScoreWindow = (flags & EXACT_TURN_TACTICAL_NEED_SCORE_WINDOW) !== 0;
  const includeScoreWindowDenial =
    needScoreWindow && (needOpponentMana || needSpiritDenial);
  let spiritFields = 0;
  if (needSpiritScore || needScoreWindow) {
    spiritFields |= EXACT_TACTICAL_SPIRIT_NEED_SCORE;
  }
  if (needSpiritDenial || includeScoreWindowDenial) {
    spiritFields |= EXACT_TACTICAL_SPIRIT_NEED_DENIAL;
  }
  const tacticalSpirit =
    spiritFields === 0
      ? defaultSpiritSummary()
      : exactTacticalSpiritSummary(
          context,
          game.board,
          color,
          remainingMoves,
          game.playerCanUseAction(),
          spiritFields,
        );
  if (context.session.checkpointWithReserve(20))
    return defaultTurnTacticalProjection();
  const safeSupermanaProgressSteps = needSupermana
    ? exactSecureSpecificManaStepsThisTurn(context, game, color, {
        kind: "supermana",
      })
    : undefined;
  if (context.session.checkpointWithReserve(20))
    return defaultTurnTacticalProjection();
  const safeOpponentManaProgressSteps = needOpponentMana
    ? exactSecureSpecificManaStepsThisTurn(context, game, color, {
        kind: "regular",
        color: otherColor(color),
      })
    : undefined;
  if (context.session.checkpointWithReserve(20))
    return defaultTurnTacticalProjection();
  const sameTurnScoreWindowValue = needScoreWindow
    ? Math.max(
        exactBestImmediateScoreOnBoard(
          context,
          game.board,
          color,
          remainingMoves,
        ),
        tacticalSpirit.sameTurnScoreValue,
        includeScoreWindowDenial
          ? tacticalSpirit.sameTurnOpponentManaScoreValue
          : 0,
      )
    : 0;
  return context.session.checkpointWithReserve(20)
    ? defaultTurnTacticalProjection()
    : {
        safeSupermanaProgress: safeSupermanaProgressSteps !== undefined,
        safeSupermanaProgressSteps,
        safeOpponentManaProgress:
          safeOpponentManaProgressSteps !== undefined ||
          tacticalSpirit.sameTurnOpponentManaScore,
        safeOpponentManaProgressSteps,
        spiritAssistedScore: tacticalSpirit.sameTurnScore,
        spiritAssistedScoreValue: tacticalSpirit.sameTurnScoreValue,
        spiritAssistedDenial: tacticalSpirit.sameTurnOpponentManaScore,
        spiritAssistedDenialValue:
          tacticalSpirit.sameTurnOpponentManaScoreValue,
        sameTurnScoreWindowValue,
      };
}

export function exactTurnTacticalProjectionWithSearchHash(
  context: AutomoveExecutionContext,
  game: MonsGame,
  color: Color,
  key: Hash64,
  flags: ExactTurnProjectionFlags,
): ExactTurnTacticalProjection {
  if (
    flags === 0 ||
    game.activeColor !== color ||
    context.session.checkpointWithReserve(20)
  ) {
    return defaultTurnTacticalProjection();
  }
  const remainingMoves = Math.max(MONS_MOVES_PER_TURN - game.monsMovesCount, 0);
  const cacheTag = exactCacheTag(
    colorKey(color),
    remainingMoves,
    game.playerCanUseAction() ? 1 : 0,
    flags,
  );
  const cached =
    cacheTag === undefined
      ? undefined
      : exactCaches(context).turnTacticalProjection.get(key, cacheTag);
  if (cached !== undefined) return cached;
  for (
    let supersetFlags = 1;
    supersetFlags <= EXACT_TURN_TACTICAL_ALL_FLAGS;
    supersetFlags += 1
  ) {
    if (supersetFlags === flags || (supersetFlags & flags) !== flags) {
      continue;
    }
    const supersetTag = exactCacheTag(
      colorKey(color),
      remainingMoves,
      game.playerCanUseAction() ? 1 : 0,
      supersetFlags,
    );
    const superset =
      supersetTag === undefined
        ? undefined
        : exactCaches(context).turnTacticalProjection.get(key, supersetTag);
    if (superset !== undefined) {
      const derived = turnTacticalProjectionForFlags(superset, flags);
      if (!context.session.cacheWriteAllowed)
        return defaultTurnTacticalProjection();
      if (cacheTag !== undefined) {
        exactCaches(context).turnTacticalProjection.set(key, derived, cacheTag);
      }
      return derived;
    }
  }
  const built = buildExactTurnTacticalProjection(context, game, flags);
  if (!context.session.cacheWriteAllowed)
    return defaultTurnTacticalProjection();
  if (cacheTag !== undefined) {
    exactCaches(context).turnTacticalProjection.set(key, built, cacheTag);
  }
  return built;
}

export function exactSameTurnScoreWindowWithSearchHash(
  context: AutomoveExecutionContext,
  game: MonsGame,
  color: Color,
  key: Hash64,
): number {
  return exactTurnTacticalProjectionWithSearchHash(
    context,
    game,
    color,
    key,
    EXACT_TURN_TACTICAL_NEED_SCORE_WINDOW,
  ).sameTurnScoreWindowValue;
}

function buildExactTurnSummary(
  context: AutomoveExecutionContext,
  game: MonsGame,
): ExactTurnSummary {
  if (context.session.checkpointWithReserve(20)) return defaultTurnSummary();
  const color = game.activeColor;
  const remainingMoves = Math.max(MONS_MOVES_PER_TURN - game.monsMovesCount, 0);
  const tacticalSpirit = exactTacticalSpiritSummary(
    context,
    game.board,
    color,
    remainingMoves,
    game.playerCanUseAction(),
    EXACT_TACTICAL_SPIRIT_NEED_SCORE |
      EXACT_TACTICAL_SPIRIT_NEED_DENIAL |
      EXACT_TACTICAL_SPIRIT_NEED_PROGRESS,
  );
  if (context.session.checkpointWithReserve(20)) return defaultTurnSummary();
  const safeSupermanaProgressSteps = exactSecureSpecificManaStepsThisTurn(
    context,
    game,
    color,
    { kind: "supermana" },
  );
  if (context.session.checkpointWithReserve(20)) return defaultTurnSummary();
  const safeOpponentManaProgressSteps = exactSecureSpecificManaStepsThisTurn(
    context,
    game,
    color,
    { kind: "regular", color: otherColor(color) },
  );
  if (context.session.checkpointWithReserve(20)) return defaultTurnSummary();
  const sameTurnScoreWindowValue = Math.max(
    exactBestImmediateScoreOnBoard(context, game.board, color, remainingMoves),
    tacticalSpirit.sameTurnScoreValue,
    tacticalSpirit.sameTurnOpponentManaScoreValue,
  );
  const boardHash = exactBoardHash(game.board);
  const summary: ExactTurnSummary = {
    canAttackOpponentDrainer: canAttackOpponentDrainerExactWithHash(
      context,
      game,
      color,
      boardHash,
    ),
    safeSupermanaProgress: safeSupermanaProgressSteps !== undefined,
    safeSupermanaProgressSteps,
    safeOpponentManaProgress:
      safeOpponentManaProgressSteps !== undefined ||
      tacticalSpirit.sameTurnOpponentManaScore,
    safeOpponentManaProgressSteps,
    spiritAssistedSupermanaProgress: tacticalSpirit.supermanaProgress,
    spiritAssistedOpponentManaProgress: tacticalSpirit.opponentManaProgress,
    spiritAssistedScore: tacticalSpirit.sameTurnScore,
    spiritAssistedDenial: tacticalSpirit.sameTurnOpponentManaScore,
    sameTurnScoreWindowValue,
    scorePathBestSteps: exactBestScoreStepsOnBoard(context, game.board, color),
  };
  return context.session.checkpointWithReserve(20)
    ? defaultTurnSummary()
    : summary;
}

function exactTurnSummaryWithSearchHash(
  context: AutomoveExecutionContext,
  game: MonsGame,
  color: Color,
  key: Hash64,
): ExactTurnSummary {
  if (game.activeColor !== color || context.session.checkpointWithReserve(20)) {
    return defaultTurnSummary();
  }
  const cached = exactCaches(context).turnSummary.get(key);
  if (cached !== undefined) return cached;
  const built = buildExactTurnSummary(context, game);
  if (!context.session.cacheWriteAllowed) return defaultTurnSummary();
  exactCaches(context).turnSummary.set(key, built);
  return built;
}

export function exactTurnSummary(
  context: AutomoveExecutionContext,
  game: MonsGame,
  color: Color,
): ExactTurnSummary {
  return exactTurnSummaryWithSearchHash(
    context,
    game,
    color,
    exactSearchStateHash(game),
  );
}

export function canAttackOpponentDrainerThisTurn(
  context: AutomoveExecutionContext,
  game: MonsGame,
  color: Color,
): boolean {
  return exactTurnSummary(context, game, color).canAttackOpponentDrainer;
}

export function exactOpportunityContextWithSearchHash(
  context: AutomoveExecutionContext,
  game: MonsGame,
  color: Color,
  key: Hash64,
): ExactOpportunityContext {
  if (game.activeColor !== color || context.session.checkpointWithReserve(20)) {
    return defaultOpportunityContext();
  }
  const budget: ExactOpportunityBudget = {
    remainingMonMoves: Math.max(MONS_MOVES_PER_TURN - game.monsMovesCount, 0),
    canUseAction: game.playerCanUseAction(),
    canMoveMana: game.playerCanMoveMana(),
  };
  const boardHash = exactBoardHash(game.board);
  const turn = exactTurnTacticalProjectionWithSearchHash(
    context,
    game,
    color,
    key,
    EXACT_TURN_TACTICAL_ALL_FLAGS,
  );
  if (context.session.checkpointWithReserve(20))
    return defaultOpportunityContext();
  const drainerSafety = exactOwnDrainerSafetyScoreWithHash(
    context,
    game.board,
    boardHash,
    color,
  );
  if (context.session.checkpointWithReserve(20))
    return defaultOpportunityContext();
  const opponent = otherColor(color);
  const opponentScore =
    opponent === Color.White ? game.whiteScore : game.blackScore;
  const opponentNeeded = Math.max(TARGET_SCORE - opponentScore, 0);
  const opponentImmediate = exactStrategicAnalysisWithSearchHash(
    context,
    game,
    key,
  ).colorSummary(opponent).immediateWindow.bestScore;
  if (context.session.checkpointWithReserve(20))
    return defaultOpportunityContext();
  const opponentCanWinImmediately =
    opponentNeeded > 0 && opponentImmediate >= opponentNeeded;
  const opponentWindowDenyGain =
    opponentNeeded > 0 && turn.sameTurnScoreWindowValue > 0
      ? Math.min(turn.sameTurnScoreWindowValue, opponentNeeded)
      : 0;
  const opportunityContext: ExactOpportunityContext = {
    budget,
    turn,
    delta: {
      sameTurnScoreWindowValue: turn.sameTurnScoreWindowValue,
      spiritGain: Math.max(
        turn.spiritAssistedScoreValue,
        turn.spiritAssistedDenialValue,
      ),
      opponentWindowDenyGain,
      drainerAttackAvailable: canAttackOpponentDrainerExactWithHash(
        context,
        game,
        color,
        boardHash,
      ),
      drainerSafety,
      safeSupermanaProgressSteps: turn.safeSupermanaProgressSteps,
      safeOpponentManaProgressSteps: turn.safeOpponentManaProgressSteps,
    },
    opponentCanWinImmediately,
  };
  return context.session.checkpointWithReserve(20)
    ? defaultOpportunityContext()
    : opportunityContext;
}

export function exactOpportunityContext(
  context: AutomoveExecutionContext,
  game: MonsGame,
  color: Color,
): ExactOpportunityContext {
  return exactOpportunityContextWithSearchHash(
    context,
    game,
    color,
    exactSearchStateHash(game),
  );
}
