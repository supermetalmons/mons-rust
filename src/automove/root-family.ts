import {
  hasConcreteScoreSurface,
  type RootObservation,
} from "./selector-types.js";
import { TurnPlanFamily } from "./turn-engine.js";

export function rootFamily(root: RootObservation): TurnPlanFamily {
  if (hasConcreteScoreSurface(root)) {
    return TurnPlanFamily.ImmediateScore;
  }
  if (root.attacksOpponentDrainer) return TurnPlanFamily.DrainerKill;
  if (root.classes.drainerSafetyRecover) {
    return TurnPlanFamily.DrainerSafetyRecovery;
  }
  if (
    root.spiritSameTurnScoreSetupNow ||
    root.spiritOwnManaSetupNow ||
    root.spiritDevelopment
  ) {
    return TurnPlanFamily.SpiritImpact;
  }
  if (root.supermanaProgress) return TurnPlanFamily.SafeSupermanaProgress;
  if (root.opponentManaProgress) {
    return TurnPlanFamily.SafeOpponentManaProgress;
  }
  return TurnPlanFamily.ManaTempo;
}
