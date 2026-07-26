import { inputChainsShareFirstInput } from "../../engine/domain.js";
import { saturatingScoreAdd } from "../score-math.js";
import type { EvaluatedRoot } from "../search.js";
import {
  productionEnabled,
  rootIsUnsafe as isUnsafe,
} from "../selector-types.js";
import type { AutomoveConfig } from "../selector-types.js";
import { TurnPlanFamily, compareUtilityPrimaryAxes } from "../turn-engine.js";
import {
  isSafePlainSpiritPair,
  sameNonTacticalProgressLane,
} from "./ranking.js";
import type {
  RootReplyRiskSnapshot,
  TurnEngineRootProjection,
} from "./types.js";

export function safeProgressSiblingOrder(
  candidate: EvaluatedRoot,
  candidateSnapshot: RootReplyRiskSnapshot,
  incumbent: EvaluatedRoot,
  incumbentSnapshot: RootReplyRiskSnapshot,
  config: AutomoveConfig,
): number | undefined {
  if (
    !productionEnabled(config) ||
    !sameNonTacticalProgressLane(candidate, incumbent) ||
    candidateSnapshot.allowsImmediateOpponentWin ||
    incumbentSnapshot.allowsImmediateOpponentWin ||
    candidateSnapshot.opponentReachesMatchPoint ||
    incumbentSnapshot.opponentReachesMatchPoint
  ) {
    return undefined;
  }
  const candidateSafe = !isUnsafe(candidate);
  const incumbentSafe = !isUnsafe(incumbent);
  if (candidateSafe === incumbentSafe) return undefined;
  const candidateReplyCompetes =
    saturatingScoreAdd(candidateSnapshot.worstReplyScore, 240) >=
    incumbentSnapshot.worstReplyScore;
  const incumbentReplyCompetes =
    saturatingScoreAdd(incumbentSnapshot.worstReplyScore, 240) >=
    candidateSnapshot.worstReplyScore;
  if (candidateSafe && candidateReplyCompetes) return 1;
  if (incumbentSafe && incumbentReplyCompetes) return -1;
  return undefined;
}

export function riskyRecoveryProgressSiblingOrder(
  candidate: EvaluatedRoot,
  candidateSnapshot: RootReplyRiskSnapshot,
  incumbent: EvaluatedRoot,
  incumbentSnapshot: RootReplyRiskSnapshot,
  candidateProjection: TurnEngineRootProjection | undefined,
  incumbentProjection: TurnEngineRootProjection | undefined,
  config: AutomoveConfig,
): number | undefined {
  if (
    !productionEnabled(config) ||
    !sameNonTacticalProgressLane(candidate, incumbent)
  ) {
    return undefined;
  }
  const candidateUnsafe = isUnsafe(candidate);
  const incumbentUnsafe = isUnsafe(incumbent);
  if (
    candidateUnsafe === incumbentUnsafe ||
    candidateProjection === undefined ||
    incumbentProjection === undefined
  ) {
    return undefined;
  }
  const risky = candidateUnsafe ? candidate : incumbent;
  const riskySnapshot = candidateUnsafe ? candidateSnapshot : incumbentSnapshot;
  const riskyProjection = candidateUnsafe
    ? candidateProjection
    : incumbentProjection;
  const safe = candidateUnsafe ? incumbent : candidate;
  const safeSnapshot = candidateUnsafe ? incumbentSnapshot : candidateSnapshot;
  const safeProjection = candidateUnsafe
    ? incumbentProjection
    : candidateProjection;
  if (
    riskySnapshot.allowsImmediateOpponentWin ||
    riskySnapshot.opponentReachesMatchPoint ||
    safeSnapshot.allowsImmediateOpponentWin ||
    safeSnapshot.opponentReachesMatchPoint ||
    riskyProjection.plan.goalFamily !== TurnPlanFamily.ImmediateScore ||
    compareUtilityPrimaryAxes(
      riskyProjection.plan.utility,
      safeProjection.plan.utility,
    ) < 0 ||
    saturatingScoreAdd(riskySnapshot.worstReplyScore, 160) <
      safeSnapshot.worstReplyScore ||
    saturatingScoreAdd(risky.score, 32) < safe.score
  ) {
    return undefined;
  }
  return candidateUnsafe ? 1 : -1;
}

export function sameFirstInput(
  candidate: EvaluatedRoot,
  incumbent: EvaluatedRoot,
): boolean {
  return inputChainsShareFirstInput(candidate.inputs, incumbent.inputs);
}

export function sameOpeningSafeSetupPair(
  candidate: EvaluatedRoot,
  incumbent: EvaluatedRoot,
  config: AutomoveConfig,
): boolean {
  return (
    productionEnabled(config) &&
    sameFirstInput(candidate, incumbent) &&
    candidate.efficiency === incumbent.efficiency &&
    Math.abs(candidate.score - incumbent.score) <= 128 &&
    !isUnsafe(candidate) &&
    !isUnsafe(incumbent) &&
    !candidate.manaHandoffToOpponent &&
    !incumbent.manaHandoffToOpponent &&
    !candidate.hasRoundtrip &&
    !incumbent.hasRoundtrip &&
    !candidate.winsImmediately &&
    !incumbent.winsImmediately &&
    !candidate.attacksOpponentDrainer &&
    !incumbent.attacksOpponentDrainer &&
    !candidate.scoresSupermanaThisTurn &&
    !incumbent.scoresSupermanaThisTurn &&
    !candidate.scoresOpponentManaThisTurn &&
    !incumbent.scoresOpponentManaThisTurn &&
    candidate.sameTurnScoreWindowValue === 0 &&
    incumbent.sameTurnScoreWindowValue === 0 &&
    !candidate.supermanaProgress &&
    !incumbent.supermanaProgress &&
    !candidate.opponentManaProgress &&
    !incumbent.opponentManaProgress
  );
}

export function omittedSameOpeningSetupCompetes(
  candidate: EvaluatedRoot,
  candidateSnapshot: RootReplyRiskSnapshot,
  incumbent: EvaluatedRoot,
  incumbentSnapshot: RootReplyRiskSnapshot,
  config: AutomoveConfig,
): boolean {
  return (
    productionEnabled(config) &&
    candidate.spiritOwnManaSetupNow &&
    !incumbent.spiritDevelopment &&
    !incumbent.spiritSameTurnScoreSetupNow &&
    !incumbent.spiritOwnManaSetupNow &&
    sameOpeningSafeSetupPair(candidate, incumbent, config) &&
    candidate.rootRank + 3 <= incumbent.rootRank &&
    saturatingScoreAdd(candidate.score, 128) >= incumbent.score &&
    saturatingScoreAdd(candidateSnapshot.worstReplyScore, 160) >=
      incumbentSnapshot.worstReplyScore
  );
}

export function safePickupOrder(
  candidate: EvaluatedRoot,
  candidateSnapshot: RootReplyRiskSnapshot,
  incumbent: EvaluatedRoot,
  incumbentSnapshot: RootReplyRiskSnapshot,
  config: AutomoveConfig,
): number | undefined {
  if (!productionEnabled(config)) return undefined;
  const candidatePickup =
    candidate.safeSupermanaPickupNow || candidate.safeOpponentManaPickupNow;
  const incumbentPickup =
    incumbent.safeSupermanaPickupNow || incumbent.safeOpponentManaPickupNow;
  if (candidatePickup === incumbentPickup) return undefined;
  const pickup = candidatePickup ? candidate : incumbent;
  const pickupSnapshot = candidatePickup
    ? candidateSnapshot
    : incumbentSnapshot;
  const other = candidatePickup ? incumbent : candidate;
  const otherSnapshot = candidatePickup ? incumbentSnapshot : candidateSnapshot;
  const otherProgressLike =
    other.supermanaProgress ||
    other.opponentManaProgress ||
    other.spiritOwnManaSetupNow ||
    other.spiritSameTurnScoreSetupNow ||
    other.spiritDevelopment;
  if (
    isUnsafe(pickup) ||
    pickupSnapshot.allowsImmediateOpponentWin ||
    pickupSnapshot.opponentReachesMatchPoint ||
    otherSnapshot.allowsImmediateOpponentWin ||
    otherSnapshot.opponentReachesMatchPoint ||
    pickup.manaHandoffToOpponent ||
    pickup.hasRoundtrip ||
    other.winsImmediately ||
    other.attacksOpponentDrainer ||
    !otherProgressLike ||
    saturatingScoreAdd(pickup.score, 144) < other.score ||
    saturatingScoreAdd(pickupSnapshot.worstReplyScore, 192) <
      otherSnapshot.worstReplyScore
  ) {
    return undefined;
  }
  return candidatePickup ? 1 : -1;
}

export function safePlainSpiritReplyRiskPair(
  candidate: EvaluatedRoot,
  candidateSnapshot: RootReplyRiskSnapshot,
  incumbent: EvaluatedRoot,
  incumbentSnapshot: RootReplyRiskSnapshot,
  config: AutomoveConfig,
): boolean {
  return (
    isSafePlainSpiritPair(candidate, incumbent, config) &&
    !candidateSnapshot.allowsImmediateOpponentWin &&
    !incumbentSnapshot.allowsImmediateOpponentWin &&
    !candidateSnapshot.opponentReachesMatchPoint &&
    !incumbentSnapshot.opponentReachesMatchPoint
  );
}
