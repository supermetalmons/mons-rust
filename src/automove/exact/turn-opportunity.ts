import { MONS_MOVES_PER_TURN, TARGET_SCORE } from "../../engine/board/config.js";
import { Color, type Mana } from "../../api/types.js";
import { otherColor } from "../../engine/model/domain.js";
import { MonsGame } from "../../engine/game/mons-game.js";
import type { AutomoveExecutionContext } from "../core/execution-context.js";
import type { Hash64 } from "../core/hash64.js";
import { canAttackTargetOnBoardWithHash } from "./attack-reach.js";
import { colorKey, exactCaches, exactCacheTag } from "./cache.js";
import {
  exactOwnDrainerSafetyScoreWithHash,
  findAwakeDrainer,
} from "./drainer-safety.js";
import { exactBoardHash, exactSearchStateHash } from "./hash.js";
import {
  exactBestImmediateScoreOnBoard,
  exactBestScoreStepsOnBoard,
} from "./mana-carrier.js";
import { exactSecureSpecificManaStepsOnBoard } from "./secure-mana.js";
import {
  EXACT_TACTICAL_SPIRIT_NEED_DENIAL,
  EXACT_TACTICAL_SPIRIT_NEED_PROGRESS,
  EXACT_TACTICAL_SPIRIT_NEED_SCORE,
  exactTacticalSpiritSummary,
} from "./spirit-tactical.js";
import { exactStrategicAnalysisWithSearchHash } from "./strategic.js";
import {
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
  type ExactOpportunityBudget,
  type ExactOpportunityContext,
  type ExactTurnSummary,
  type ExactTurnTacticalProjection,
} from "./types.js";

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
  const needSupermana = (flags & EXACT_TURN_TACTICAL_NEED_SUPERMANA_PROGRESS) !== 0;
  const needOpponentMana =
    (flags & EXACT_TURN_TACTICAL_NEED_OPPONENT_MANA_PROGRESS) !== 0;
  const needSpiritScore = (flags & EXACT_TURN_TACTICAL_NEED_SPIRIT_SCORE) !== 0;
  const needSpiritDenial = (flags & EXACT_TURN_TACTICAL_NEED_SPIRIT_DENIAL) !== 0;
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
  const spiritAssistedDenial = includeSpiritDenial && projection.spiritAssistedDenial;
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
    sameTurnScoreWindowValue: needScoreWindow ? projection.sameTurnScoreWindowValue : 0,
  };
}

function buildExactTurnTacticalProjection(
  context: AutomoveExecutionContext,
  game: MonsGame,
  flags: number,
): ExactTurnTacticalProjection {
  if (context.session.checkpointWithReserve(20)) return defaultTurnTacticalProjection();
  const color = game.activeColor;
  const remainingMoves = Math.max(MONS_MOVES_PER_TURN - game.monsMovesCount, 0);
  const needSupermana = (flags & EXACT_TURN_TACTICAL_NEED_SUPERMANA_PROGRESS) !== 0;
  const needOpponentMana =
    (flags & EXACT_TURN_TACTICAL_NEED_OPPONENT_MANA_PROGRESS) !== 0;
  const needSpiritScore = (flags & EXACT_TURN_TACTICAL_NEED_SPIRIT_SCORE) !== 0;
  const needSpiritDenial = (flags & EXACT_TURN_TACTICAL_NEED_SPIRIT_DENIAL) !== 0;
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
  if (context.session.checkpointWithReserve(20)) return defaultTurnTacticalProjection();
  const safeSupermanaProgressSteps = needSupermana
    ? exactSecureSpecificManaStepsThisTurn(context, game, color, {
        kind: "supermana",
      })
    : undefined;
  if (context.session.checkpointWithReserve(20)) return defaultTurnTacticalProjection();
  const safeOpponentManaProgressSteps = needOpponentMana
    ? exactSecureSpecificManaStepsThisTurn(context, game, color, {
        kind: "regular",
        color: otherColor(color),
      })
    : undefined;
  if (context.session.checkpointWithReserve(20)) return defaultTurnTacticalProjection();
  const sameTurnScoreWindowValue = needScoreWindow
    ? Math.max(
        exactBestImmediateScoreOnBoard(context, game.board, color, remainingMoves),
        tacticalSpirit.sameTurnScoreValue,
        includeScoreWindowDenial ? tacticalSpirit.sameTurnOpponentManaScoreValue : 0,
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
        spiritAssistedDenialValue: tacticalSpirit.sameTurnOpponentManaScoreValue,
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
      if (!context.session.cacheWriteAllowed) return defaultTurnTacticalProjection();
      if (cacheTag !== undefined) {
        exactCaches(context).turnTacticalProjection.set(key, derived, cacheTag);
      }
      return derived;
    }
  }
  const built = buildExactTurnTacticalProjection(context, game, flags);
  if (!context.session.cacheWriteAllowed) return defaultTurnTacticalProjection();
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
  return context.session.checkpointWithReserve(20) ? defaultTurnSummary() : summary;
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
  if (context.session.checkpointWithReserve(20)) return defaultOpportunityContext();
  const drainerSafety = exactOwnDrainerSafetyScoreWithHash(
    context,
    game.board,
    boardHash,
    color,
  );
  if (context.session.checkpointWithReserve(20)) return defaultOpportunityContext();
  const opponent = otherColor(color);
  const opponentScore = opponent === Color.White ? game.whiteScore : game.blackScore;
  const opponentNeeded = Math.max(TARGET_SCORE - opponentScore, 0);
  const opponentImmediate = exactStrategicAnalysisWithSearchHash(
    context,
    game,
    key,
  ).colorSummary(opponent).immediateWindow.bestScore;
  if (context.session.checkpointWithReserve(20)) return defaultOpportunityContext();
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
