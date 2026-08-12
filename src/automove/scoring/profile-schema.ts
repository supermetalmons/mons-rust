export type EvaluationFormulaWeights = {
  readonly useHeuristicFormula: boolean;
  readonly includeRegularManaMoveWindows: boolean;
  readonly includeMatchPointWindow: boolean;
  readonly nextTurnWindowScaleBp: number;
  readonly doubleConfirmedScore: boolean;
};

export type MaterialWeights = {
  readonly confirmedScore: number;
  readonly faintedMon: number;
  readonly faintedDrainer: number;
  readonly faintedCooldownStep: number;
  readonly hasConsumable: number;
  readonly activeMon: number;
};

export type PositionWeights = {
  readonly drainerAtRisk: number;
  readonly manaCloseToSamePool: number;
  readonly monWithManaCloseToAnyPool: number;
  readonly extraForSupermana: number;
  readonly extraForOpponentsMana: number;
  readonly drainerCloseToMana: number;
  readonly drainerHoldingMana: number;
  readonly drainerCloseToOwnPool: number;
  readonly drainerCloseToSupermana: number;
  readonly monCloseToCenter: number;
  readonly spiritCloseToEnemy: number;
  readonly spiritOnOwnBasePenalty: number;
  readonly angelGuardingDrainer: number;
  readonly angelCloseToFriendlyDrainer: number;
};

export type ManaWeights = {
  readonly regularManaToOwnerPool: number;
  readonly regularManaDrainerControl: number;
  readonly supermanaDrainerControl: number;
  readonly supermanaRaceControl: number;
  readonly opponentManaDenial: number;
  readonly manaCarrierAtRisk: number;
  readonly manaCarrierGuarded: number;
  readonly manaCarrierOneStepFromPool: number;
  readonly supermanaCarrierOneStepFromPoolExtra: number;
  readonly immediateWinningCarrier: number;
};

export type RaceWeights = {
  readonly scoreRacePathProgress: number;
  readonly opponentScoreRacePathProgress: number;
  readonly scoreRaceMultiPath: number;
  readonly opponentScoreRaceMultiPath: number;
  readonly immediateScoreWindow: number;
  readonly opponentImmediateScoreWindow: number;
  readonly immediateScoreMultiWindow: number;
  readonly opponentImmediateScoreMultiWindow: number;
};

export type ThreatWeights = {
  readonly drainerBestManaPath: number;
  readonly drainerPickupScoreThisTurn: number;
  readonly manaCarrierScoreThisTurn: number;
  readonly drainerImmediateThreat: number;
  readonly spiritActionUtility: number;
  readonly drainerDangerBoolean: number;
  readonly manaCarrierDangerBoolean: number;
  readonly drainerWalkThreatBoolean: number;
  readonly manaCarrierWalkThreatBoolean: number;
  readonly opponentDrainerAttackBonus: number;
  readonly attackerCloseToOpponentDrainer: number;
};

export type ScoringWeights = {
  readonly id: string;
  readonly formula: EvaluationFormulaWeights;
  readonly material: MaterialWeights;
  readonly position: PositionWeights;
  readonly mana: ManaWeights;
  readonly race: RaceWeights;
  readonly threat: ThreatWeights;
};

export type ScoringProfileDefinition = {
  readonly id: string;
  readonly base?: ScoringWeights;
  readonly formula?: Partial<EvaluationFormulaWeights>;
  readonly material?: Partial<MaterialWeights>;
  readonly position?: Partial<PositionWeights>;
  readonly mana?: Partial<ManaWeights>;
  readonly race?: Partial<RaceWeights>;
  readonly threat?: Partial<ThreatWeights>;
};

export const PROFILE_KEYS = [
  "id",
  "formula",
  "material",
  "position",
  "mana",
  "race",
  "threat",
] as const satisfies readonly (keyof ScoringWeights)[];

export const FORMULA_KEYS = [
  "useHeuristicFormula",
  "includeRegularManaMoveWindows",
  "includeMatchPointWindow",
  "nextTurnWindowScaleBp",
  "doubleConfirmedScore",
] as const satisfies readonly (keyof EvaluationFormulaWeights)[];

export const MATERIAL_KEYS = [
  "confirmedScore",
  "faintedMon",
  "faintedDrainer",
  "faintedCooldownStep",
  "hasConsumable",
  "activeMon",
] as const satisfies readonly (keyof MaterialWeights)[];

export const POSITION_KEYS = [
  "drainerAtRisk",
  "manaCloseToSamePool",
  "monWithManaCloseToAnyPool",
  "extraForSupermana",
  "extraForOpponentsMana",
  "drainerCloseToMana",
  "drainerHoldingMana",
  "drainerCloseToOwnPool",
  "drainerCloseToSupermana",
  "monCloseToCenter",
  "spiritCloseToEnemy",
  "spiritOnOwnBasePenalty",
  "angelGuardingDrainer",
  "angelCloseToFriendlyDrainer",
] as const satisfies readonly (keyof PositionWeights)[];

export const MANA_KEYS = [
  "regularManaToOwnerPool",
  "regularManaDrainerControl",
  "supermanaDrainerControl",
  "supermanaRaceControl",
  "opponentManaDenial",
  "manaCarrierAtRisk",
  "manaCarrierGuarded",
  "manaCarrierOneStepFromPool",
  "supermanaCarrierOneStepFromPoolExtra",
  "immediateWinningCarrier",
] as const satisfies readonly (keyof ManaWeights)[];

export const RACE_KEYS = [
  "scoreRacePathProgress",
  "opponentScoreRacePathProgress",
  "scoreRaceMultiPath",
  "opponentScoreRaceMultiPath",
  "immediateScoreWindow",
  "opponentImmediateScoreWindow",
  "immediateScoreMultiWindow",
  "opponentImmediateScoreMultiWindow",
] as const satisfies readonly (keyof RaceWeights)[];

export const THREAT_KEYS = [
  "drainerBestManaPath",
  "drainerPickupScoreThisTurn",
  "manaCarrierScoreThisTurn",
  "drainerImmediateThreat",
  "spiritActionUtility",
  "drainerDangerBoolean",
  "manaCarrierDangerBoolean",
  "drainerWalkThreatBoolean",
  "manaCarrierWalkThreatBoolean",
  "opponentDrainerAttackBonus",
  "attackerCloseToOpponentDrainer",
] as const satisfies readonly (keyof ThreatWeights)[];

export type NumericSection =
  MaterialWeights | PositionWeights | ManaWeights | RaceWeights | ThreatWeights;
