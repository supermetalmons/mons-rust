import { Color } from "../../../api/types.js";
import type { MonsGame } from "../../../engine/game/mons-game.js";
import { saturatingScoreAdd, saturatingScoreSubtract } from "../../core/score-math.js";
import { rootFamily } from "../../root/family.js";
import type { EvaluatedRoot } from "../../root/types.js";
import {
  isPlainSpiritDevelopmentRoot,
  productionEnabled,
  rootIsUnsafe as isUnsafe,
} from "../../config/types.js";
import type { AutomoveConfig } from "../../config/types.js";
import { TurnPlanFamily } from "../../turn/model.js";
import {
  rootProgressOrSetupBetter,
  sameNonTacticalProgressLane,
  shortlistHasPair,
} from "./shortlist.js";
import { sameFirstInput } from "./sibling-ordering.js";

export function safeProgressCompetition(
  evaluations: readonly EvaluatedRoot[],
  shortlist: readonly number[],
  config: AutomoveConfig,
): boolean {
  if (!productionEnabled(config) || shortlist.length < 2) return false;
  return shortlistHasPair(
    evaluations,
    shortlist,
    (candidate, incumbent) =>
      sameNonTacticalProgressLane(candidate, incumbent) &&
      isUnsafe(candidate) !== isUnsafe(incumbent),
  );
}

export function isProductionModeWhiteSpiritFollowupSetupPair(
  game: MonsGame,
  candidate: EvaluatedRoot,
  incumbent: EvaluatedRoot,
  config: AutomoveConfig,
): boolean {
  if (
    !productionEnabled(config) ||
    game.activeColor !== Color.White ||
    game.turnNumber > 3 ||
    game.monsMovesCount < 1 ||
    !game.playerCanUseAction() ||
    !game.playerCanMoveMana()
  ) {
    return false;
  }
  const candidateSetup =
    candidate.spiritOwnManaSetupNow &&
    candidate.opponentManaProgress &&
    !isPlainSpiritDevelopmentRoot(candidate);
  const incumbentSetup =
    incumbent.spiritOwnManaSetupNow &&
    incumbent.opponentManaProgress &&
    !isPlainSpiritDevelopmentRoot(incumbent);
  if (candidateSetup === incumbentSetup) return false;
  const pair = candidateSetup
    ? ([candidate, incumbent] as const)
    : ([incumbent, candidate] as const);
  const [setup, plain] = pair;
  if (!isPlainSpiritDevelopmentRoot(plain)) return false;
  return (
    sameFirstInput(setup, plain) &&
    setup.efficiency === plain.efficiency &&
    setup.ownDrainerVulnerable === plain.ownDrainerVulnerable &&
    setup.ownDrainerWalkVulnerable === plain.ownDrainerWalkVulnerable &&
    !setup.manaHandoffToOpponent &&
    !plain.manaHandoffToOpponent &&
    !setup.hasRoundtrip &&
    !plain.hasRoundtrip &&
    !setup.winsImmediately &&
    !plain.winsImmediately &&
    !setup.attacksOpponentDrainer &&
    !plain.attacksOpponentDrainer &&
    !setup.scoresSupermanaThisTurn &&
    !plain.scoresSupermanaThisTurn &&
    !setup.scoresOpponentManaThisTurn &&
    !plain.scoresOpponentManaThisTurn &&
    !setup.safeSupermanaPickupNow &&
    !plain.safeSupermanaPickupNow &&
    !setup.safeOpponentManaPickupNow &&
    !plain.safeOpponentManaPickupNow &&
    setup.sameTurnScoreWindowValue === 0 &&
    plain.sameTurnScoreWindowValue === 0 &&
    setup.supermanaProgress === plain.supermanaProgress &&
    setup.safeSupermanaProgressSteps === plain.safeSupermanaProgressSteps &&
    setup.opponentManaProgress === plain.opponentManaProgress &&
    setup.safeOpponentManaProgressSteps === plain.safeOpponentManaProgressSteps
  );
}

export function whiteSpiritFollowupSetupCompetition(
  game: MonsGame,
  evaluations: readonly EvaluatedRoot[],
  shortlist: readonly number[],
  config: AutomoveConfig,
): boolean {
  return shortlistHasPair(evaluations, shortlist, (candidate, incumbent) =>
    isProductionModeWhiteSpiritFollowupSetupPair(game, candidate, incumbent, config),
  );
}

export function nonConcreteManaWindowRoot(root: EvaluatedRoot): boolean {
  return (
    rootFamily(root) === TurnPlanFamily.ManaTempo &&
    root.sameTurnScoreWindowValue > 0 &&
    !root.winsImmediately &&
    !root.attacksOpponentDrainer &&
    !root.scoresSupermanaThisTurn &&
    !root.scoresOpponentManaThisTurn &&
    !root.safeSupermanaPickupNow &&
    !root.safeOpponentManaPickupNow &&
    !root.manaHandoffToOpponent &&
    !root.hasRoundtrip
  );
}

export function blackManaWindowProgressCompetition(
  game: MonsGame,
  evaluations: readonly EvaluatedRoot[],
  shortlist: readonly number[],
  config: AutomoveConfig,
): boolean {
  if (
    !productionEnabled(config) ||
    shortlist.length < 2 ||
    game.activeColor !== Color.Black ||
    game.turnNumber > 4
  ) {
    return false;
  }
  return shortlistHasPair(evaluations, shortlist, (candidate, incumbent) => {
    const candidateWindow = nonConcreteManaWindowRoot(candidate);
    const incumbentWindow = nonConcreteManaWindowRoot(incumbent);
    if (candidateWindow === incumbentWindow) return false;
    const window = candidateWindow ? candidate : incumbent;
    const progress = candidateWindow ? incumbent : candidate;
    return (
      window.sameTurnScoreWindowValue <= 1 &&
      progress.sameTurnScoreWindowValue === 0 &&
      rootFamily(progress) === TurnPlanFamily.ManaTempo &&
      progress.ownDrainerVulnerable === window.ownDrainerVulnerable &&
      progress.ownDrainerWalkVulnerable === window.ownDrainerWalkVulnerable &&
      !progress.manaHandoffToOpponent &&
      !progress.hasRoundtrip &&
      rootProgressOrSetupBetter(progress, window) &&
      saturatingScoreAdd(progress.score, 192) >= window.score
    );
  });
}

export function closePositiveScoreCompetition(
  evaluations: readonly EvaluatedRoot[],
  shortlist: readonly number[],
  config: AutomoveConfig,
): boolean {
  if (!productionEnabled(config) || shortlist.length < 2) return false;
  const scores = shortlist
    .map((index) => evaluations[index]?.score)
    .filter((score): score is number => score !== undefined && score >= 0)
    .sort((left, right) => right - left);
  return (
    scores.length >= 2 && saturatingScoreSubtract(scores[0] ?? 0, scores[1] ?? 0) <= 64
  );
}
