import { Board } from "../../engine/board/storage.js";
import type { AutomoveExecutionContext } from "../core/execution-context.js";
import { TARGET_SCORE } from "../../engine/board/config.js";
import { Color, MonKind, type Mana } from "../../api/types.js";
import {
  colorId,
  isMonFainted,
  itemMon,
  manaEquals,
  otherColor,
} from "../../engine/model/domain.js";
import type { MonsGame } from "../../engine/game/mons-game.js";
import { BOARD_SIZE, type Location } from "../../engine/board/geometry.js";
import { scoreForColor } from "../../engine/rules/legality.js";
import {
  MAX_SCORE,
  saturatingScoreAdd,
  saturatingScoreMultiply,
  saturatingScoreSubtract,
} from "../core/score-math.js";
import { defaultColorSummary, defaultOpportunityContext } from "../exact/types.js";
import { exactBoardHash, exactSearchStateHash } from "../exact/hash.js";
import {
  exactOpportunityContextWithSearchHash,
  exactSameTurnScoreWindowWithSearchHash,
} from "../exact/turn-opportunity.js";
import {
  exactOwnDrainerSafetyScoreWithHash,
  isDrainerExactlySafeNextTurnOnBoard,
} from "../exact/drainer-safety.js";
import { exactStrategicAnalysisWithSearchHash } from "../exact/strategic.js";
import type { Hash64 } from "../core/hash64.js";
import { evaluatePreferabilityWithWeightsAndExactPolicy } from "../scoring/evaluator.js";
import { turnEngineCaches } from "./cache.js";
import {
  createUtilityCacheIdentity,
  utilityCacheKey,
  type UtilityCacheIdentity,
} from "./fingerprint.js";
import {
  EMPTY_TURN_UTILITY,
  TurnPlanFamily,
  createTurnUtility,
  type TurnEngineConfig,
  type TurnOracleContext,
  type TurnUtility,
} from "./model.js";

export type TurnUtilityEvalContext = Readonly<{
  execution: AutomoveExecutionContext;
  start: MonsGame;
  startHash: Hash64;
  perspective: Color;
  config: TurnEngineConfig;
  cacheIdentity: UtilityCacheIdentity;
}>;

export function createTurnUtilityEvalContext(
  execution: AutomoveExecutionContext,
  start: MonsGame,
  perspective: Color,
  config: TurnEngineConfig,
): TurnUtilityEvalContext {
  return createTurnUtilityEvalContextWithSearchHash(
    execution,
    start,
    exactSearchStateHash(start),
    perspective,
    config,
  );
}

export function createTurnUtilityEvalContextWithSearchHash(
  execution: AutomoveExecutionContext,
  start: MonsGame,
  startHash: Hash64,
  perspective: Color,
  config: TurnEngineConfig,
): TurnUtilityEvalContext {
  return {
    execution,
    start,
    startHash,
    perspective,
    config,
    cacheIdentity: createUtilityCacheIdentity(startHash, perspective, config),
  };
}

export function turnEngineEvaluateStateUtility(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  start: MonsGame,
  perspective: Color,
  config: TurnEngineConfig,
): TurnUtility {
  if (execution.session.checkpoint()) return EMPTY_TURN_UTILITY;
  const utility = evaluateStateUtilityFromInputs(
    execution,
    game,
    start,
    perspective,
    config,
  );
  return execution.session.checkpoint() ? EMPTY_TURN_UTILITY : utility;
}

function evaluateStateUtilityFromInputs(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  start: MonsGame,
  perspective: Color,
  config: TurnEngineConfig,
): TurnUtility {
  if (execution.session.checkpoint()) return EMPTY_TURN_UTILITY;
  const context = createTurnUtilityEvalContext(execution, start, perspective, config);
  return evaluateStateUtilityAfterCheckpoint(context, game, exactSearchStateHash(game));
}

export function turnOracleContextWithSearchHash(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  stateHash: Hash64,
): TurnOracleContext {
  if (execution.session.checkpoint()) return emptyOracleContext();
  return turnOracleContextAfterCheckpoint(execution, game, perspective, stateHash);
}

function turnOracleContextAfterCheckpoint(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  stateHash: Hash64,
): TurnOracleContext {
  const cached = turnEngineCaches(execution).oracle.get(
    stateHash,
    colorId(perspective),
  );
  if (cached !== undefined)
    return execution.session.checkpoint() ? emptyOracleContext() : cached;
  const analysis = exactStrategicAnalysisWithSearchHash(execution, game, stateHash);
  if (execution.session.checkpoint()) return emptyOracleContext();
  const opportunity = exactOpportunityContextWithSearchHash(
    execution,
    game,
    perspective,
    stateHash,
  );
  if (execution.session.checkpoint()) return emptyOracleContext();
  const built: TurnOracleContext = {
    opportunity,
    strategic: analysis.colorSummary(perspective),
    opponentImmediateWindow: analysis.colorSummary(otherColor(perspective))
      .immediateWindow.bestScore,
  };
  if (execution.session.cacheWriteAllowed) {
    turnEngineCaches(execution).oracle.set(stateHash, built, colorId(perspective));
  }
  return built;
}

function emptyOracleContext(): TurnOracleContext {
  return {
    opportunity: defaultOpportunityContext(),
    strategic: defaultColorSummary(),
    opponentImmediateWindow: 0,
  };
}

export function activeTurnScoreWindowWithSearchHash(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  color: Color,
  stateHash: Hash64,
): number {
  if (execution.session.checkpoint()) return 0;
  const value = exactSameTurnScoreWindowWithSearchHash(
    execution,
    game,
    color,
    stateHash,
  );
  return execution.session.checkpoint() ? 0 : value;
}

export function activeTurnScoreWindow(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  color: Color,
): number {
  return activeTurnScoreWindowWithSearchHash(
    execution,
    game,
    color,
    exactSearchStateHash(game),
  );
}

function winnerState(game: MonsGame, perspective: Color): number {
  const winner = game.winnerColor();
  return winner === undefined ? 0 : winner === perspective ? 2 : -2;
}

export function opponentCanWinImmediately(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
): boolean {
  if (
    game.winnerColor() !== undefined ||
    game.activeColor !== otherColor(perspective)
  ) {
    return false;
  }
  return opponentCanWinImmediatelyWithSearchHash(
    execution,
    game,
    perspective,
    exactSearchStateHash(game),
  );
}

export function opponentCanWinImmediatelyWithSearchHash(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  stateHash: Hash64,
): boolean {
  const opponent = otherColor(perspective);
  if (game.winnerColor() !== undefined || game.activeColor !== opponent) return false;
  const needed = saturatingScoreSubtract(TARGET_SCORE, scoreForColor(game, opponent));
  return (
    needed <= 0 ||
    activeTurnScoreWindowWithSearchHash(execution, game, opponent, stateHash) >= needed
  );
}

export function ownDrainerSafetyScore(
  execution: AutomoveExecutionContext,
  board: Board,
  color: Color,
): number {
  return exactOwnDrainerSafetyScoreWithHash(
    execution,
    board,
    exactBoardHash(board),
    color,
  );
}

export function findAwakeDrainerLocation(
  board: Board,
  color: Color,
): Location | undefined {
  for (const [at, item] of board.entries()) {
    const mon = itemMon(item);
    if (mon?.color === color && mon.kind === MonKind.Drainer && !isMonFainted(mon))
      return at;
  }
  return undefined;
}

function ownDrainerCarriesSafeMana(
  execution: AutomoveExecutionContext,
  board: Board,
  color: Color,
  wanted: Mana,
): boolean {
  const at = findAwakeDrainerLocation(board, color);
  if (at === undefined) return false;
  const item = board.get(at);
  return (
    item?.kind === "mon-with-mana" &&
    manaEquals(item.mana, wanted) &&
    isDrainerExactlySafeNextTurnOnBoard(execution, board, color, at)
  );
}

export function evaluateStateUtilityWithSearchHash(
  context: TurnUtilityEvalContext,
  game: MonsGame,
  stateHash: Hash64,
): TurnUtility {
  if (context.execution.session.checkpoint()) return EMPTY_TURN_UTILITY;
  return evaluateStateUtilityAfterCheckpoint(context, game, stateHash);
}

function evaluateStateUtilityAfterCheckpoint(
  context: TurnUtilityEvalContext,
  game: MonsGame,
  stateHash: Hash64,
): TurnUtility {
  const { execution, start, perspective, config } = context;
  const key = utilityCacheKey(stateHash, context.cacheIdentity);
  const cached = turnEngineCaches(execution).utility.get(
    key.stateHash,
    key.startTag,
    key.configFingerprint,
    key.startLow,
  );
  if (cached !== undefined)
    return execution.session.checkpoint() ? EMPTY_TURN_UTILITY : cached;

  const myScore = scoreForColor(game, perspective);
  const startScore = scoreForColor(start, perspective);
  const scoreDelta = saturatingScoreSubtract(myScore, startScore);
  const oracle = turnOracleContextWithSearchHash(
    execution,
    game,
    perspective,
    stateHash,
  );
  if (execution.session.cancelled) return EMPTY_TURN_UTILITY;
  const bestSteps = oracle.strategic.scorePathWindow.bestSteps;
  const pathBonus =
    bestSteps === undefined
      ? 0
      : saturatingScoreMultiply(Math.max(BOARD_SIZE * 3 - bestSteps, 0), 22);
  const immediateBonus = saturatingScoreAdd(
    saturatingScoreMultiply(oracle.strategic.immediateWindow.bestScore, 110),
    saturatingScoreMultiply(oracle.strategic.immediateWindow.multiPressure, 18),
  );
  const supermana: Mana = { kind: "supermana" };
  const opponentMana: Mana = {
    kind: "regular",
    color: otherColor(perspective),
  };
  const safeSupermanaBonus = ownDrainerCarriesSafeMana(
    execution,
    game.board,
    perspective,
    supermana,
  )
    ? 380
    : 0;
  const safeOpponentManaBonus = ownDrainerCarriesSafeMana(
    execution,
    game.board,
    perspective,
    opponentMana,
  )
    ? 300
    : 0;
  const opponent = otherColor(perspective);
  const opponentWindowBefore = activeTurnScoreWindowWithSearchHash(
    execution,
    start,
    opponent,
    context.startHash,
  );
  const sessionAfterOpponentWindowBefore = execution.session;
  if (sessionAfterOpponentWindowBefore.cancelled) return EMPTY_TURN_UTILITY;
  const opponentWindowAfter =
    game.activeColor === opponent
      ? activeTurnScoreWindowWithSearchHash(execution, game, opponent, stateHash)
      : oracle.opponentImmediateWindow;
  const sessionAfterOpponentWindowAfter = execution.session;
  if (sessionAfterOpponentWindowAfter.cancelled) return EMPTY_TURN_UTILITY;
  const denyGain = saturatingScoreSubtract(opponentWindowBefore, opponentWindowAfter);
  const drainerSafety = ownDrainerSafetyScore(execution, game.board, perspective);
  const unsafeProgressPenalty =
    drainerSafety < 0
      ? saturatingScoreMultiply(Math.min(Math.abs(drainerSafety), MAX_SCORE), 900)
      : 0;
  const opponentNeededBefore = saturatingScoreSubtract(
    TARGET_SCORE,
    scoreForColor(start, opponent),
  );
  const opponentNeededAfter = saturatingScoreSubtract(
    TARGET_SCORE,
    scoreForColor(game, opponent),
  );
  const deniedImmediateWindow =
    opponentNeededBefore > 0 &&
    opponentWindowBefore >= opponentNeededBefore &&
    (opponentNeededAfter <= 0 || opponentWindowAfter < opponentNeededAfter);
  const evalScore = evaluatePreferabilityWithWeightsAndExactPolicy(
    execution,
    game,
    perspective,
    config.scoringWeights,
    false,
  );
  if (execution.session.checkpoint()) return EMPTY_TURN_UTILITY;
  const utility = createTurnUtility({
    winState: winnerState(game, perspective),
    avoidImmediateLoss: opponentCanWinImmediatelyWithSearchHash(
      execution,
      game,
      perspective,
      stateHash,
    )
      ? -1
      : 1,
    scoreDelta: saturatingScoreSubtract(
      saturatingScoreAdd(
        saturatingScoreAdd(
          saturatingScoreAdd(
            saturatingScoreAdd(saturatingScoreMultiply(scoreDelta, 2_400), pathBonus),
            immediateBonus,
          ),
          safeSupermanaBonus,
        ),
        safeOpponentManaBonus,
      ),
      unsafeProgressPenalty,
    ),
    denyGain: saturatingScoreAdd(
      saturatingScoreMultiply(denyGain, 220),
      deniedImmediateWindow ? 1_500 : 0,
    ),
    drainerAttack:
      findAwakeDrainerLocation(game.board, otherColor(perspective)) === undefined
        ? 1
        : 0,
    drainerSafety,
    evalScore,
  });
  if (execution.session.cacheWriteAllowed) {
    turnEngineCaches(execution).utility.set(
      key.stateHash,
      utility,
      key.startTag,
      key.configFingerprint,
      key.startLow,
    );
  }
  return utility;
}

export function quickOrderScoreWithSearchHash(
  context: TurnUtilityEvalContext,
  game: MonsGame,
  stateHash: Hash64,
  family: TurnPlanFamily,
  stepLength: number,
): number {
  return quickOrderScoreForUtility(
    evaluateStateUtilityWithSearchHash(context, game, stateHash),
    family,
    stepLength,
  );
}

function quickOrderScoreForUtility(
  utility: TurnUtility,
  family: TurnPlanFamily,
  stepLength: number,
): number {
  const familyBonus = (() => {
    switch (family) {
      case TurnPlanFamily.ImmediateScore:
        return 1_000;
      case TurnPlanFamily.DenyOpponentWindow:
        return 960;
      case TurnPlanFamily.DrainerKill:
        return 920;
      case TurnPlanFamily.DrainerSafetyRecovery:
        return 860;
      case TurnPlanFamily.SpiritImpact:
        return 820;
      case TurnPlanFamily.SafeSupermanaProgress:
        return 760;
      case TurnPlanFamily.SafeOpponentManaProgress:
        return 720;
      case TurnPlanFamily.ManaTempo:
        return 560;
    }
  })();
  return (
    utility.winState * 10_000_000 +
    utility.avoidImmediateLoss * 5_000_000 +
    utility.scoreDelta +
    utility.denyGain +
    utility.drainerAttack * 3_500 +
    utility.drainerSafety * 2_200 +
    Math.trunc(utility.evalScore / 8) +
    familyBonus * 2_000 -
    stepLength * 350
  );
}
