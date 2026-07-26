import { Color } from "../../engine/domain.js";
import type { MonsGame } from "../../engine/game.js";
import type { AutomoveExecutionContext } from "../execution-context.js";
import { exactOpportunityContext } from "../exact.js";
import { patchAutomoveConfig } from "../selector-config.js";
import {
  AUTOMOVE_TURN_ENGINE_MODE,
  type AutomoveConfig,
} from "../selector-types.js";
import {
  applyEarlyWhiteTurnEngineLimits,
  productionSecondaryAnalysisLive,
  productionIsEarlyWhiteTurnStart,
  turnEngineConfigFromAutomoveConfig,
  turnEngineModeFromAutomoveConfig,
  turnEngineRerankConfigFromAutomoveConfig,
} from "../turn-engine-config.js";
import { TurnEngineMode, type TurnEngineConfig } from "../turn-engine.js";

export {
  productionSecondaryAnalysisLive,
  turnEngineConfigFromAutomoveConfig,
  turnEngineModeFromAutomoveConfig as modeFromConfig,
  turnEngineRerankConfigFromAutomoveConfig as turnEngineRerankConfig,
};

export function turnEngineModeUsesMacroPlans(mode: TurnEngineMode): boolean {
  return mode === TurnEngineMode.Production;
}

export function productionTurnEngineLive(config: AutomoveConfig): boolean {
  return (
    config.planner.enabled &&
    config.planner.mode === AUTOMOVE_TURN_ENGINE_MODE.Production
  );
}

export function productionLowBudgetGuardLive(config: AutomoveConfig): boolean {
  return productionTurnEngineLive(config) && config.planner.lowBudgetGuard;
}

export function productionMidTurnTacticalGuardLive(
  config: AutomoveConfig,
): boolean {
  return (
    productionTurnEngineLive(config) && config.planner.midTurnTacticalGuard
  );
}

export function productionIsWhiteTurnOneManaOnlyFollowup(
  game: MonsGame,
): boolean {
  return (
    game.activeColor === Color.White &&
    game.turnNumber === 1 &&
    game.isFirstTurn() &&
    game.monsMovesCount === 1 &&
    !game.playerCanUseAction() &&
    !game.playerCanMoveMana()
  );
}

export function productionUseFreshLiveHeadPlan(
  game: MonsGame,
  config: AutomoveConfig,
): boolean {
  return (
    productionTurnEngineLive(config) &&
    game.activeColor === Color.White &&
    game.turnNumber >= 3 &&
    game.monsMovesCount === 0 &&
    game.playerCanUseAction() &&
    game.playerCanMoveMana()
  );
}

export function productionIsSafeEarlyBlackOpeningState(
  execution: AutomoveExecutionContext,
  game: MonsGame,
): boolean {
  if (
    game.activeColor !== Color.Black ||
    game.turnNumber !== 2 ||
    game.monsMovesCount !== 0 ||
    !game.playerCanUseAction() ||
    !game.playerCanMoveMana()
  ) {
    return false;
  }
  const context = exactOpportunityContext(execution, game, game.activeColor);
  return (
    !context.opponentCanWinImmediately &&
    context.delta.sameTurnScoreWindowValue <= 0 &&
    context.delta.opponentWindowDenyGain <= 0 &&
    !context.delta.drainerAttackAvailable &&
    context.delta.drainerSafety >= 2
  );
}

export function shouldSkipProductionLowBudgetState(
  execution: AutomoveExecutionContext,
  game: MonsGame,
): boolean {
  if (productionIsEarlyWhiteTurnStart(game)) return false;
  if (game.playerCanUseAction() || game.playerCanMoveMana()) return false;
  const context = exactOpportunityContext(execution, game, game.activeColor);
  return (
    !context.opponentCanWinImmediately &&
    context.delta.sameTurnScoreWindowValue <= 0 &&
    context.delta.opponentWindowDenyGain <= 0 &&
    !context.delta.drainerAttackAvailable &&
    context.delta.safeSupermanaProgressSteps === undefined &&
    context.delta.safeOpponentManaProgressSteps === undefined &&
    context.delta.drainerSafety >= 0
  );
}

export function shouldDisableProductionMidTurnTacticalEngine(
  execution: AutomoveExecutionContext,
  game: MonsGame,
): boolean {
  if (
    game.activeColor !== Color.White ||
    game.turnNumber !== 3 ||
    game.monsMovesCount === 0 ||
    game.monsMovesCount > 2 ||
    !game.playerCanUseAction() ||
    !game.playerCanMoveMana() ||
    productionIsEarlyWhiteTurnStart(game)
  ) {
    return false;
  }
  const context = exactOpportunityContext(execution, game, game.activeColor);
  return (
    !context.opponentCanWinImmediately &&
    context.delta.drainerSafety < 0 &&
    context.delta.sameTurnScoreWindowValue <= 0 &&
    context.delta.safeSupermanaProgressSteps === undefined &&
    context.delta.safeOpponentManaProgressSteps === undefined
  );
}

export function applyProductionLowBudgetSearchClamp(
  game: MonsGame,
  config: AutomoveConfig,
): AutomoveConfig {
  if (!productionLowBudgetGuardLive(config)) return config;
  if (
    game.activeColor !== Color.Black ||
    game.turnNumber !== 2 ||
    game.monsMovesCount > 1 ||
    (game.playerCanUseAction() && game.playerCanMoveMana())
  ) {
    return config;
  }
  return patchAutomoveConfig(config, {
    budget: {
      maxVisitedNodes: Math.min(config.budget.maxVisitedNodes, 6_000),
    },
    search: {
      rootBranchLimit: Math.min(Math.max(config.search.rootBranchLimit, 1), 12),
    },
    replyRisk: {
      replyLimit: Math.min(Math.max(config.replyRisk.replyLimit, 1), 12),
      nodeShareBp: Math.min(config.replyRisk.nodeShareBp, 1_200),
    },
  });
}

export function turnEngineConfigForGame(
  game: MonsGame,
  config: AutomoveConfig,
): TurnEngineConfig {
  const engine = turnEngineConfigFromAutomoveConfig(config);
  if (
    !turnEngineModeUsesMacroPlans(engine.mode) ||
    !config.planner.lowBudgetGuard ||
    !productionIsEarlyWhiteTurnStart(game)
  ) {
    return engine;
  }
  return applyEarlyWhiteTurnEngineLimits(engine);
}
