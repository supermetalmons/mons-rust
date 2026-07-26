import type { MonsGame } from "../../engine/game.js";
import { TERMINAL_SEARCH_SCORE } from "../score-math.js";
import type { AutomoveConfig } from "../selector-types.js";
import {
  applyEarlyWhiteTurnEngineLimits,
  productionSecondaryAnalysisLive,
  productionIsEarlyWhiteTurnStart,
  turnEngineConfigFromAutomoveConfig,
  turnEngineRerankConfigFromAutomoveConfig,
} from "../turn-engine-config.js";
import { TurnEngineMode, type TurnEngineConfig } from "../turn-engine.js";

export const SMART_TERMINAL_SCORE = TERMINAL_SEARCH_SCORE;
export const SMART_ROOT_REPLY_RISK_WINNER_SPREAD_SKIP = 700;

export {
  productionSecondaryAnalysisLive,
  turnEngineConfigFromAutomoveConfig as fullTurnEngineConfig,
  turnEngineRerankConfigFromAutomoveConfig as rerankTurnEngineConfig,
};

export function projectionTurnEngineConfig(
  game: MonsGame,
  config: AutomoveConfig,
): TurnEngineConfig {
  let engine = turnEngineConfigFromAutomoveConfig(config);
  if (
    engine.mode === TurnEngineMode.Production &&
    config.planner.lowBudgetGuard &&
    productionIsEarlyWhiteTurnStart(game)
  ) {
    engine = applyEarlyWhiteTurnEngineLimits(engine);
  }
  const production = engine.mode === TurnEngineMode.Production;
  return {
    ...engine,
    ownSeedCap: Math.min(engine.ownSeedCap, production ? 8 : 6),
    ownBeam: Math.min(engine.ownBeam, production ? 3 : 2),
    perNodeFamilyCap: Math.min(engine.perNodeFamilyCap, production ? 3 : 2),
    stepCap: Math.min(engine.stepCap, 4),
    opponentSeedCap: Math.min(engine.opponentSeedCap, production ? 2 : 1),
    opponentBeam: 1,
    replySeedCap: 1,
    replyBeam: 1,
    expansionCap: Math.min(engine.expansionCap, production ? 64 : 48),
  };
}
