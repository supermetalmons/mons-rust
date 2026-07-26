import { Color, type Mana } from "../engine/domain.js";

export type ExactScorePathWindow = {
  readonly bestSteps: number | undefined;
  readonly multiPressure: number;
};

export type ExactImmediateScoreWindow = {
  readonly bestScore: number;
  readonly multiPressure: number;
};

export type ExactDrainerPickupPath = {
  readonly pathSteps: number;
  readonly totalMoves: number;
  readonly manaValue: number;
  readonly mana: Mana;
};

export type ExactSpiritSummary = {
  readonly utility: number;
  readonly sameTurnScore: boolean;
  readonly sameTurnScoreValue: number;
  readonly sameTurnOpponentManaScore: boolean;
  readonly sameTurnOpponentManaScoreValue: number;
  readonly supermanaProgress: boolean;
  readonly opponentManaProgress: boolean;
  readonly nextTurnSetupGain: number;
};

export type ExactColorSummary = {
  readonly scorePathWindow: ExactScorePathWindow;
  readonly immediateWindow: ExactImmediateScoreWindow;
  readonly bestDrainerPickup: ExactDrainerPickupPath | undefined;
  readonly bestCarrierSteps: number | undefined;
  readonly bestDrainerToManaSteps: number | undefined;
  readonly spirit: ExactSpiritSummary;
};

export type ExactTurnSummary = {
  readonly canAttackOpponentDrainer: boolean;
  readonly safeSupermanaProgress: boolean;
  readonly safeSupermanaProgressSteps: number | undefined;
  readonly safeOpponentManaProgress: boolean;
  readonly safeOpponentManaProgressSteps: number | undefined;
  readonly spiritAssistedSupermanaProgress: boolean;
  readonly spiritAssistedOpponentManaProgress: boolean;
  readonly spiritAssistedScore: boolean;
  readonly spiritAssistedDenial: boolean;
  readonly sameTurnScoreWindowValue: number;
  readonly scorePathBestSteps: number | undefined;
};

export type ExactTurnTacticalProjection = {
  readonly safeSupermanaProgress: boolean;
  readonly safeSupermanaProgressSteps: number | undefined;
  readonly safeOpponentManaProgress: boolean;
  readonly safeOpponentManaProgressSteps: number | undefined;
  readonly spiritAssistedScore: boolean;
  readonly spiritAssistedScoreValue: number;
  readonly spiritAssistedDenial: boolean;
  readonly spiritAssistedDenialValue: number;
  readonly sameTurnScoreWindowValue: number;
};

export const EXACT_TURN_TACTICAL_NEED_SUPERMANA_PROGRESS = 1 << 0;
export const EXACT_TURN_TACTICAL_NEED_OPPONENT_MANA_PROGRESS = 1 << 1;
export const EXACT_TURN_TACTICAL_NEED_SPIRIT_SCORE = 1 << 2;
export const EXACT_TURN_TACTICAL_NEED_SPIRIT_DENIAL = 1 << 3;
export const EXACT_TURN_TACTICAL_NEED_SCORE_WINDOW = 1 << 4;
export const EXACT_TURN_TACTICAL_ALL_FLAGS =
  EXACT_TURN_TACTICAL_NEED_SUPERMANA_PROGRESS |
  EXACT_TURN_TACTICAL_NEED_OPPONENT_MANA_PROGRESS |
  EXACT_TURN_TACTICAL_NEED_SPIRIT_SCORE |
  EXACT_TURN_TACTICAL_NEED_SPIRIT_DENIAL |
  EXACT_TURN_TACTICAL_NEED_SCORE_WINDOW;

export type ExactOpportunityBudget = {
  readonly remainingMonMoves: number;
  readonly canUseAction: boolean;
  readonly canMoveMana: boolean;
};

export type ExactOpportunityDelta = {
  readonly sameTurnScoreWindowValue: number;
  readonly spiritGain: number;
  readonly opponentWindowDenyGain: number;
  readonly drainerAttackAvailable: boolean;
  readonly drainerSafety: number;
  readonly safeSupermanaProgressSteps: number | undefined;
  readonly safeOpponentManaProgressSteps: number | undefined;
};

export type ExactOpportunityContext = {
  readonly budget: ExactOpportunityBudget;
  readonly turn: ExactTurnTacticalProjection;
  readonly delta: ExactOpportunityDelta;
  readonly opponentCanWinImmediately: boolean;
};

export function defaultSpiritSummary(): ExactSpiritSummary {
  return {
    utility: 0,
    sameTurnScore: false,
    sameTurnScoreValue: 0,
    sameTurnOpponentManaScore: false,
    sameTurnOpponentManaScoreValue: 0,
    supermanaProgress: false,
    opponentManaProgress: false,
    nextTurnSetupGain: 0,
  };
}

export function defaultColorSummary(): ExactColorSummary {
  return {
    scorePathWindow: { bestSteps: undefined, multiPressure: 0 },
    immediateWindow: { bestScore: 0, multiPressure: 0 },
    bestDrainerPickup: undefined,
    bestCarrierSteps: undefined,
    bestDrainerToManaSteps: undefined,
    spirit: defaultSpiritSummary(),
  };
}

export function defaultTurnSummary(): ExactTurnSummary {
  return {
    canAttackOpponentDrainer: false,
    safeSupermanaProgress: false,
    safeSupermanaProgressSteps: undefined,
    safeOpponentManaProgress: false,
    safeOpponentManaProgressSteps: undefined,
    spiritAssistedSupermanaProgress: false,
    spiritAssistedOpponentManaProgress: false,
    spiritAssistedScore: false,
    spiritAssistedDenial: false,
    sameTurnScoreWindowValue: 0,
    scorePathBestSteps: undefined,
  };
}

export function defaultTurnTacticalProjection(): ExactTurnTacticalProjection {
  return {
    safeSupermanaProgress: false,
    safeSupermanaProgressSteps: undefined,
    safeOpponentManaProgress: false,
    safeOpponentManaProgressSteps: undefined,
    spiritAssistedScore: false,
    spiritAssistedScoreValue: 0,
    spiritAssistedDenial: false,
    spiritAssistedDenialValue: 0,
    sameTurnScoreWindowValue: 0,
  };
}

export function defaultOpportunityContext(): ExactOpportunityContext {
  return {
    budget: { remainingMonMoves: 0, canUseAction: false, canMoveMana: false },
    turn: defaultTurnTacticalProjection(),
    delta: {
      sameTurnScoreWindowValue: 0,
      spiritGain: 0,
      opponentWindowDenyGain: 0,
      drainerAttackAvailable: false,
      drainerSafety: 0,
      safeSupermanaProgressSteps: undefined,
      safeOpponentManaProgressSteps: undefined,
    },
    opponentCanWinImmediately: false,
  };
}

export class ExactStrategicAnalysis {
  public readonly white: ExactColorSummary;
  public readonly black: ExactColorSummary;

  public constructor(
    white: ExactColorSummary = defaultColorSummary(),
    black: ExactColorSummary = defaultColorSummary(),
  ) {
    this.white = white;
    this.black = black;
  }

  public colorSummary(color: Color): ExactColorSummary {
    return color === Color.White ? this.white : this.black;
  }
}
