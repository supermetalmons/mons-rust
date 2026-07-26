import { Color } from "../engine/domain.js";
import type { MonsGame } from "../engine/game.js";
import {
  AUTOMOVE_TURN_ENGINE_MODE,
  type AutomoveConfig,
} from "./selector-types.js";
import { TurnEngineMode, type TurnEngineConfig } from "./turn-types.js";

export function turnEngineModeFromAutomoveConfig(
  config: AutomoveConfig,
): TurnEngineMode {
  return config.planner.mode === AUTOMOVE_TURN_ENGINE_MODE.Production
    ? TurnEngineMode.Production
    : TurnEngineMode.Baseline;
}

function positiveCap(value: number): number {
  return Math.max(Math.trunc(value), 1);
}

export function turnEngineConfigFromAutomoveConfig(
  config: AutomoveConfig,
): TurnEngineConfig {
  return {
    mode: turnEngineModeFromAutomoveConfig(config),
    ownSeedCap: positiveCap(config.planner.ownSeedCap),
    ownBeam: positiveCap(config.planner.ownBeam),
    perNodeFamilyCap: positiveCap(config.planner.perNodeFamilyCap),
    stepCap: positiveCap(config.planner.stepCap),
    opponentSeedCap: positiveCap(config.planner.opponentSeedCap),
    opponentBeam: positiveCap(config.planner.opponentBeam),
    replySeedCap: positiveCap(config.planner.replySeedCap),
    replyBeam: positiveCap(config.planner.replyBeam),
    expansionCap: positiveCap(config.planner.expansionCap),
    enableSpiritFamily: config.planner.spiritFamily,
    scoringWeights: config.evaluation.weights,
    enableLazyOracleScoreWindowProjection: false,
  };
}

export function productionSecondaryAnalysisLive(
  config: AutomoveConfig,
): boolean {
  return (
    config.planner.enabled &&
    config.planner.mode === AUTOMOVE_TURN_ENGINE_MODE.Production &&
    config.planner.secondaryAnalysis
  );
}

export function productionIsEarlyWhiteTurnStart(game: MonsGame): boolean {
  return (
    game.activeColor === Color.White &&
    game.turnNumber <= 3 &&
    !game.playerCanUseAction() &&
    !game.playerCanMoveMana() &&
    (game.monsMovesCount === 0 || game.monsMovesCount === 3)
  );
}

export function applyEarlyWhiteTurnEngineLimits(
  config: TurnEngineConfig,
): TurnEngineConfig {
  return {
    ...config,
    ownSeedCap: Math.min(config.ownSeedCap, 14),
    ownBeam: Math.min(config.ownBeam, 5),
    perNodeFamilyCap: Math.min(config.perNodeFamilyCap, 4),
    stepCap: Math.min(config.stepCap, 6),
    opponentSeedCap: 1,
    opponentBeam: 1,
    replySeedCap: 1,
    replyBeam: 1,
    expansionCap: Math.min(config.expansionCap, 48),
  };
}

export function applyTurnEngineRerankLimits(
  config: TurnEngineConfig,
): TurnEngineConfig {
  const production = config.mode === TurnEngineMode.Production;
  return {
    ...config,
    ownSeedCap: Math.min(config.ownSeedCap, production ? 12 : 8),
    ownBeam: Math.min(config.ownBeam, production ? 5 : 4),
    perNodeFamilyCap: Math.min(config.perNodeFamilyCap, production ? 4 : 3),
    stepCap: Math.min(config.stepCap, 5),
    opponentSeedCap: Math.min(config.opponentSeedCap, production ? 6 : 4),
    opponentBeam: Math.min(config.opponentBeam, production ? 3 : 2),
    replySeedCap: Math.min(config.replySeedCap, production ? 3 : 2),
    replyBeam: production ? 2 : 1,
    expansionCap: Math.min(config.expansionCap, production ? 144 : 96),
  };
}

export function turnEngineRerankConfigFromAutomoveConfig(
  config: AutomoveConfig,
): TurnEngineConfig {
  return applyTurnEngineRerankLimits(
    turnEngineConfigFromAutomoveConfig(config),
  );
}
