import type { Color } from "../../api/types.js";
import type { Input } from "../../engine/model/domain.js";
import type { AutomoveConfig } from "../config/types.js";

export type RootFocusMoveClassFlags = {
  readonly immediateScore: boolean;
  readonly drainerAttack: boolean;
  readonly drainerSafetyRecover: boolean;
  readonly carrierProgress: boolean;
  readonly material: boolean;
  readonly quiet: boolean;
};

export type RootFocusCandidate = {
  readonly inputs: readonly Input[];
  readonly game: { readonly activeColor: Color };
  readonly heuristic: number;
  readonly efficiency: number;
  readonly winsImmediately: boolean;
  readonly attacksOpponentDrainer: boolean;
  readonly ownDrainerVulnerable: boolean;
  readonly ownDrainerWalkVulnerable: boolean;
  readonly spiritDevelopment: boolean;
  readonly manaHandoffToOpponent: boolean;
  readonly hasRoundtrip: boolean;
  readonly scoresSupermanaThisTurn: boolean;
  readonly scoresOpponentManaThisTurn: boolean;
  readonly safeSupermanaPickupNow: boolean;
  readonly safeOpponentManaPickupNow: boolean;
  readonly safeSupermanaProgressSteps: number;
  readonly safeOpponentManaProgressSteps: number;
  readonly scorePathBestSteps: number;
  readonly sameTurnScoreWindowValue: number;
  readonly spiritSameTurnScoreSetupNow: boolean;
  readonly spiritOwnManaSetupNow: boolean;
  readonly supermanaProgress: boolean;
  readonly opponentManaProgress: boolean;
  readonly classes: RootFocusMoveClassFlags;
};

export type RootFocusConfig = AutomoveConfig;
