import type { MonsGame } from "../../../engine/game/mons-game.js";
import type { AutomoveConfig } from "../../config/types.js";
import {
  applyEarlyWhiteTurnEngineLimits,
  productionIsEarlyWhiteTurnStart,
  turnEngineConfigFromAutomoveConfig,
} from "../../turn/config.js";
import { TurnEngineMode, type TurnEngineConfig } from "../../turn/model.js";

export const SMART_ROOT_REPLY_RISK_WINNER_SPREAD_SKIP = 700;

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
