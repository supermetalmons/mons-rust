import { Color } from "../../engine/domain.js";
import type { MonsGame } from "../../engine/game.js";
import type { AutomoveExecutionContext } from "../execution-context.js";
import { saturatingScoreAdd } from "../score-math.js";
import { rootFamily } from "../root-family.js";
import type { EvaluatedRoot } from "../search.js";
import {
  hasProgressSurface,
  isPlainSpiritDevelopmentRoot,
  productionEnabled,
  rootIsUnsafe as isUnsafe,
} from "../selector-types.js";
import type { AutomoveConfig } from "../selector-types.js";
import { TurnPlanFamily } from "../turn-engine.js";
import { productionSecondaryAnalysisLive } from "./config.js";
import { spiritFollowupFloorScore } from "./projection.js";
import { rootProgressOrSetupBetter } from "./ranking.js";
import type { RootReplyRiskSnapshot } from "./types.js";
import {
  isProductionModeWhiteSpiritFollowupSetupPair,
  nonConcreteManaWindowRoot,
} from "./competition.js";
import { sameFirstInput } from "./sibling-ordering.js";

export function spiritFollowupFloorOrder(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  evaluations: readonly EvaluatedRoot[],
  candidateIndex: number,
  incumbentIndex: number,
  perspective: Color,
  config: AutomoveConfig,
  scores: Map<number, number>,
): number | undefined {
  const candidate = evaluations[candidateIndex];
  const incumbent = evaluations[incumbentIndex];
  if (
    !productionSecondaryAnalysisLive(config) ||
    game.turnNumber > 3 ||
    candidate === undefined ||
    incumbent === undefined ||
    !isPlainSpiritDevelopmentRoot(candidate) ||
    !isPlainSpiritDevelopmentRoot(incumbent) ||
    isUnsafe(candidate) ||
    isUnsafe(incumbent) ||
    candidate.manaHandoffToOpponent ||
    incumbent.manaHandoffToOpponent ||
    candidate.hasRoundtrip ||
    incumbent.hasRoundtrip ||
    candidate.supermanaProgress ||
    candidate.opponentManaProgress ||
    incumbent.supermanaProgress ||
    incumbent.opponentManaProgress ||
    candidate.sameTurnScoreWindowValue > 0 ||
    incumbent.sameTurnScoreWindowValue > 0 ||
    candidate.spiritSameTurnScoreSetupNow ||
    incumbent.spiritSameTurnScoreSetupNow ||
    candidate.spiritOwnManaSetupNow ||
    incumbent.spiritOwnManaSetupNow ||
    Math.abs(candidate.score - incumbent.score) > 224
  ) {
    return undefined;
  }
  const candidateScore =
    scores.get(candidateIndex) ??
    spiritFollowupFloorScore(execution, candidate.game, perspective, config);
  scores.set(candidateIndex, candidateScore);
  const incumbentScore =
    scores.get(incumbentIndex) ??
    spiritFollowupFloorScore(execution, incumbent.game, perspective, config);
  scores.set(incumbentIndex, incumbentScore);
  if (candidateScore >= saturatingScoreAdd(incumbentScore, 32)) return 1;
  if (incumbentScore >= saturatingScoreAdd(candidateScore, 32)) return -1;
  return 0;
}

export function isProductionModeBlackPlainSpiritFollowupSetupPair(
  game: MonsGame,
  plain: EvaluatedRoot,
  setup: EvaluatedRoot,
  config: AutomoveConfig,
): boolean {
  return (
    productionEnabled(config) &&
    game.activeColor === Color.Black &&
    game.turnNumber <= 4 &&
    isPlainSpiritDevelopmentRoot(plain) &&
    setup.spiritOwnManaSetupNow &&
    !setup.spiritSameTurnScoreSetupNow &&
    sameFirstInput(plain, setup) &&
    plain.ownDrainerVulnerable === setup.ownDrainerVulnerable &&
    plain.ownDrainerWalkVulnerable === setup.ownDrainerWalkVulnerable &&
    !plain.manaHandoffToOpponent &&
    !setup.manaHandoffToOpponent &&
    !plain.hasRoundtrip &&
    !setup.hasRoundtrip &&
    !plain.winsImmediately &&
    !setup.winsImmediately &&
    !plain.attacksOpponentDrainer &&
    !setup.attacksOpponentDrainer &&
    !plain.scoresSupermanaThisTurn &&
    !setup.scoresSupermanaThisTurn &&
    !plain.scoresOpponentManaThisTurn &&
    !setup.scoresOpponentManaThisTurn &&
    !plain.safeSupermanaPickupNow &&
    !setup.safeSupermanaPickupNow &&
    !plain.safeOpponentManaPickupNow &&
    !setup.safeOpponentManaPickupNow &&
    plain.sameTurnScoreWindowValue === 0 &&
    setup.sameTurnScoreWindowValue === 0 &&
    !plain.supermanaProgress &&
    !setup.supermanaProgress &&
    !plain.opponentManaProgress &&
    !setup.opponentManaProgress
  );
}

export function blackPlainSpiritFollowupReplyOrder(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  evaluations: readonly EvaluatedRoot[],
  candidateIndex: number,
  candidateSnapshot: RootReplyRiskSnapshot,
  incumbentIndex: number,
  incumbentSnapshot: RootReplyRiskSnapshot,
  perspective: Color,
  config: AutomoveConfig,
  scores: Map<number, number>,
): number | undefined {
  const candidate = evaluations[candidateIndex];
  const incumbent = evaluations[incumbentIndex];
  if (candidate === undefined || incumbent === undefined) return undefined;
  let plain: EvaluatedRoot;
  let plainSnapshot: RootReplyRiskSnapshot;
  let setup: EvaluatedRoot;
  let setupSnapshot: RootReplyRiskSnapshot;
  let plainIndex: number;
  let setupIndex: number;
  let candidateIsPlain: boolean;
  if (
    isProductionModeBlackPlainSpiritFollowupSetupPair(
      game,
      candidate,
      incumbent,
      config,
    )
  ) {
    plain = candidate;
    plainSnapshot = candidateSnapshot;
    setup = incumbent;
    setupSnapshot = incumbentSnapshot;
    plainIndex = candidateIndex;
    setupIndex = incumbentIndex;
    candidateIsPlain = true;
  } else if (
    isProductionModeBlackPlainSpiritFollowupSetupPair(
      game,
      incumbent,
      candidate,
      config,
    )
  ) {
    plain = incumbent;
    plainSnapshot = incumbentSnapshot;
    setup = candidate;
    setupSnapshot = candidateSnapshot;
    plainIndex = incumbentIndex;
    setupIndex = candidateIndex;
    candidateIsPlain = false;
  } else {
    return undefined;
  }
  if (
    plainSnapshot.allowsImmediateOpponentWin ||
    setupSnapshot.allowsImmediateOpponentWin ||
    plainSnapshot.opponentReachesMatchPoint ||
    setupSnapshot.opponentReachesMatchPoint
  ) {
    return undefined;
  }
  const plainFollowup =
    scores.get(plainIndex) ??
    spiritFollowupFloorScore(execution, plain.game, perspective, config);
  scores.set(plainIndex, plainFollowup);
  const setupFollowup =
    scores.get(setupIndex) ??
    spiritFollowupFloorScore(execution, setup.game, perspective, config);
  scores.set(setupIndex, setupFollowup);
  const setupHasCloseTopSeed =
    setup.rootRank <= plain.rootRank &&
    saturatingScoreAdd(setup.score, 64) >= plain.score &&
    setup.spiritSetupGain >= saturatingScoreAdd(plain.spiritSetupGain, 32) &&
    saturatingScoreAdd(setupSnapshot.worstReplyScore, 192) >=
      plainSnapshot.worstReplyScore &&
    saturatingScoreAdd(setupFollowup, 32) >= plainFollowup;
  if (setupHasCloseTopSeed) return candidateIsPlain ? -1 : 1;
  if (
    saturatingScoreAdd(plainSnapshot.worstReplyScore, 192) <
      setupSnapshot.worstReplyScore ||
    (plainFollowup < saturatingScoreAdd(setupFollowup, 32) &&
      plain.score < setup.score)
  ) {
    return undefined;
  }
  return candidateIsPlain ? 1 : -1;
}

export function earlyBlackPlainSpiritSiblingOrder(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  evaluations: readonly EvaluatedRoot[],
  candidateIndex: number,
  candidateSnapshot: RootReplyRiskSnapshot,
  incumbentIndex: number,
  incumbentSnapshot: RootReplyRiskSnapshot,
  perspective: Color,
  config: AutomoveConfig,
  scores: Map<number, number>,
): number | undefined {
  const candidate = evaluations[candidateIndex];
  const incumbent = evaluations[incumbentIndex];
  if (
    !productionEnabled(config) ||
    game.activeColor !== Color.Black ||
    game.turnNumber > 2 ||
    candidate === undefined ||
    incumbent === undefined ||
    !isPlainSpiritDevelopmentRoot(candidate) ||
    !isPlainSpiritDevelopmentRoot(incumbent) ||
    !sameFirstInput(candidate, incumbent) ||
    candidate.ownDrainerVulnerable !== incumbent.ownDrainerVulnerable ||
    candidate.manaHandoffToOpponent !== incumbent.manaHandoffToOpponent ||
    candidate.hasRoundtrip !== incumbent.hasRoundtrip ||
    candidate.spiritSameTurnScoreSetupNow ||
    incumbent.spiritSameTurnScoreSetupNow ||
    candidate.spiritOwnManaSetupNow ||
    incumbent.spiritOwnManaSetupNow ||
    candidate.winsImmediately ||
    incumbent.winsImmediately ||
    candidate.attacksOpponentDrainer ||
    incumbent.attacksOpponentDrainer ||
    candidateSnapshot.allowsImmediateOpponentWin ||
    incumbentSnapshot.allowsImmediateOpponentWin ||
    candidateSnapshot.opponentReachesMatchPoint ||
    incumbentSnapshot.opponentReachesMatchPoint
  ) {
    return undefined;
  }
  const candidateFollowup =
    scores.get(candidateIndex) ??
    spiritFollowupFloorScore(execution, candidate.game, perspective, config);
  scores.set(candidateIndex, candidateFollowup);
  const incumbentFollowup =
    scores.get(incumbentIndex) ??
    spiritFollowupFloorScore(execution, incumbent.game, perspective, config);
  scores.set(incumbentIndex, incumbentFollowup);
  const candidateBetter =
    candidateSnapshot.worstReplyScore >=
      saturatingScoreAdd(incumbentSnapshot.worstReplyScore, 96) &&
    saturatingScoreAdd(candidateFollowup, 32) >= incumbentFollowup &&
    saturatingScoreAdd(candidate.score, 48) >= incumbent.score;
  const incumbentBetter =
    incumbentSnapshot.worstReplyScore >=
      saturatingScoreAdd(candidateSnapshot.worstReplyScore, 96) &&
    saturatingScoreAdd(incumbentFollowup, 32) >= candidateFollowup &&
    saturatingScoreAdd(incumbent.score, 48) >= candidate.score;
  if (candidateBetter === incumbentBetter) {
    const candidateClose =
      candidateSnapshot.worstReplyScore <
        saturatingScoreAdd(incumbentSnapshot.worstReplyScore, 96) &&
      candidateFollowup < saturatingScoreAdd(incumbentFollowup, 96);
    const incumbentClose =
      incumbentSnapshot.worstReplyScore <
        saturatingScoreAdd(candidateSnapshot.worstReplyScore, 96) &&
      incumbentFollowup < saturatingScoreAdd(candidateFollowup, 96);
    if (
      candidateClose &&
      incumbentClose &&
      candidate.score !== incumbent.score
    ) {
      return candidate.score > incumbent.score ? 1 : -1;
    }
    return undefined;
  }
  return candidateBetter ? 1 : -1;
}

export function earlyBlackManaProgressReplyOrder(
  game: MonsGame,
  candidate: EvaluatedRoot,
  candidateSnapshot: RootReplyRiskSnapshot,
  incumbent: EvaluatedRoot,
  incumbentSnapshot: RootReplyRiskSnapshot,
  config: AutomoveConfig,
): number | undefined {
  if (
    !productionEnabled(config) ||
    game.activeColor !== Color.Black ||
    game.turnNumber > 4 ||
    candidateSnapshot.allowsImmediateOpponentWin ||
    incumbentSnapshot.allowsImmediateOpponentWin ||
    candidateSnapshot.opponentReachesMatchPoint ||
    incumbentSnapshot.opponentReachesMatchPoint
  ) {
    return undefined;
  }
  const candidateFamily = rootFamily(candidate);
  const incumbentFamily = rootFamily(incumbent);
  const allowed = (family: TurnPlanFamily): boolean =>
    family === TurnPlanFamily.ManaTempo ||
    family === TurnPlanFamily.SafeSupermanaProgress ||
    family === TurnPlanFamily.SafeOpponentManaProgress;
  if (
    !allowed(candidateFamily) ||
    !allowed(incumbentFamily) ||
    Math.abs(candidate.rootRank - incumbent.rootRank) > 8
  ) {
    return undefined;
  }
  const candidateProgressSurface = hasProgressSurface(candidate);
  const incumbentProgressSurface = hasProgressSurface(incumbent);
  const candidateProgressBetter =
    rootProgressOrSetupBetter(candidate, incumbent) ||
    (candidateProgressSurface && !incumbentProgressSurface);
  const incumbentProgressBetter =
    rootProgressOrSetupBetter(incumbent, candidate) ||
    (incumbentProgressSurface && !candidateProgressSurface);
  if (candidateProgressBetter === incumbentProgressBetter) return undefined;

  const candidateWindow = nonConcreteManaWindowRoot(candidate);
  const incumbentWindow = nonConcreteManaWindowRoot(incumbent);
  if (candidateWindow !== incumbentWindow) {
    const window = candidateWindow ? candidate : incumbent;
    const windowSnapshot = candidateWindow
      ? candidateSnapshot
      : incumbentSnapshot;
    const progress = candidateWindow ? incumbent : candidate;
    const progressSnapshot = candidateWindow
      ? incumbentSnapshot
      : candidateSnapshot;
    if (
      window.sameTurnScoreWindowValue <= 1 &&
      progress.sameTurnScoreWindowValue === 0 &&
      rootFamily(progress) === TurnPlanFamily.ManaTempo &&
      progress.ownDrainerVulnerable === window.ownDrainerVulnerable &&
      progress.ownDrainerWalkVulnerable === window.ownDrainerWalkVulnerable &&
      !progress.manaHandoffToOpponent &&
      !progress.hasRoundtrip &&
      rootProgressOrSetupBetter(progress, window) &&
      saturatingScoreAdd(progressSnapshot.worstReplyScore, 96) >=
        windowSnapshot.worstReplyScore &&
      saturatingScoreAdd(progress.score, 192) >= window.score
    ) {
      return candidateWindow ? -1 : 1;
    }
  }

  const candidateReplyCompetes =
    saturatingScoreAdd(candidateSnapshot.worstReplyScore, 240) >=
    incumbentSnapshot.worstReplyScore;
  const incumbentReplyCompetes =
    saturatingScoreAdd(incumbentSnapshot.worstReplyScore, 240) >=
    candidateSnapshot.worstReplyScore;
  if (
    candidateProgressBetter &&
    candidate.score > incumbent.score &&
    candidateReplyCompetes
  ) {
    return 1;
  }
  if (
    incumbentProgressBetter &&
    incumbent.score > candidate.score &&
    incumbentReplyCompetes
  ) {
    return -1;
  }
  return undefined;
}

export function earlyBlackPlainSpiritManaReplyOrder(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  evaluations: readonly EvaluatedRoot[],
  candidateIndex: number,
  candidateSnapshot: RootReplyRiskSnapshot,
  incumbentIndex: number,
  incumbentSnapshot: RootReplyRiskSnapshot,
  perspective: Color,
  config: AutomoveConfig,
  scores: Map<number, number>,
): number | undefined {
  const candidate = evaluations[candidateIndex];
  const incumbent = evaluations[incumbentIndex];
  if (
    !productionEnabled(config) ||
    game.activeColor !== Color.Black ||
    game.turnNumber > 4 ||
    candidate === undefined ||
    incumbent === undefined
  ) {
    return undefined;
  }
  const candidatePlain = isPlainSpiritDevelopmentRoot(candidate);
  const incumbentPlain = isPlainSpiritDevelopmentRoot(incumbent);
  const candidateMana = rootFamily(candidate) === TurnPlanFamily.ManaTempo;
  const incumbentMana = rootFamily(incumbent) === TurnPlanFamily.ManaTempo;
  if (candidatePlain === incumbentPlain || candidateMana === incumbentMana) {
    return undefined;
  }
  const plain = candidatePlain ? candidate : incumbent;
  const plainSnapshot = candidatePlain ? candidateSnapshot : incumbentSnapshot;
  const plainIndex = candidatePlain ? candidateIndex : incumbentIndex;
  const mana = candidatePlain ? incumbent : candidate;
  const manaSnapshot = candidatePlain ? incumbentSnapshot : candidateSnapshot;
  const manaIndex = candidatePlain ? incumbentIndex : candidateIndex;
  if (
    !sameFirstInput(plain, mana) ||
    plain.ownDrainerVulnerable !== mana.ownDrainerVulnerable ||
    plain.ownDrainerWalkVulnerable !== mana.ownDrainerWalkVulnerable ||
    plainSnapshot.allowsImmediateOpponentWin ||
    manaSnapshot.allowsImmediateOpponentWin ||
    plainSnapshot.opponentReachesMatchPoint ||
    manaSnapshot.opponentReachesMatchPoint ||
    plain.manaHandoffToOpponent ||
    mana.manaHandoffToOpponent ||
    plain.hasRoundtrip ||
    mana.hasRoundtrip ||
    plain.winsImmediately ||
    mana.winsImmediately ||
    plain.attacksOpponentDrainer ||
    mana.attacksOpponentDrainer ||
    plain.sameTurnScoreWindowValue > 0 ||
    mana.sameTurnScoreWindowValue > 0 ||
    plain.scoresSupermanaThisTurn ||
    mana.scoresSupermanaThisTurn ||
    plain.scoresOpponentManaThisTurn ||
    mana.scoresOpponentManaThisTurn ||
    plain.safeSupermanaPickupNow ||
    mana.safeSupermanaPickupNow ||
    plain.safeOpponentManaPickupNow ||
    mana.safeOpponentManaPickupNow ||
    plain.spiritSameTurnScoreSetupNow ||
    plain.spiritOwnManaSetupNow ||
    mana.spiritSameTurnScoreSetupNow ||
    mana.spiritOwnManaSetupNow ||
    mana.supermanaProgress ||
    mana.opponentManaProgress ||
    saturatingScoreAdd(plain.score, 24) < mana.score ||
    saturatingScoreAdd(plainSnapshot.worstReplyScore, 192) <
      manaSnapshot.worstReplyScore
  ) {
    return undefined;
  }
  const plainFollowup =
    scores.get(plainIndex) ??
    spiritFollowupFloorScore(execution, plain.game, perspective, config);
  scores.set(plainIndex, plainFollowup);
  const manaFollowup =
    scores.get(manaIndex) ??
    spiritFollowupFloorScore(execution, mana.game, perspective, config);
  scores.set(manaIndex, manaFollowup);
  if (saturatingScoreAdd(plainFollowup, 32) < manaFollowup) {
    return undefined;
  }
  return candidatePlain ? 1 : -1;
}

export function safeNonSpiritFollowupOrder(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  evaluations: readonly EvaluatedRoot[],
  candidateIndex: number,
  candidateSnapshot: RootReplyRiskSnapshot,
  incumbentIndex: number,
  incumbentSnapshot: RootReplyRiskSnapshot,
  perspective: Color,
  config: AutomoveConfig,
  scores: Map<number, number>,
): number | undefined {
  if (!productionSecondaryAnalysisLive(config)) return undefined;
  const candidate = evaluations[candidateIndex];
  const incumbent = evaluations[incumbentIndex];
  if (candidate === undefined || incumbent === undefined) return undefined;
  const candidatePlain = isPlainSpiritDevelopmentRoot(candidate);
  const incumbentPlain = isPlainSpiritDevelopmentRoot(incumbent);
  if (candidatePlain === incumbentPlain) return undefined;
  const challenger = candidatePlain ? incumbent : candidate;
  const challengerSnapshot = candidatePlain
    ? incumbentSnapshot
    : candidateSnapshot;
  const challengerIndex = candidatePlain ? incumbentIndex : candidateIndex;
  const spirit = candidatePlain ? candidate : incumbent;
  const spiritSnapshot = candidatePlain ? candidateSnapshot : incumbentSnapshot;
  const spiritIndex = candidatePlain ? candidateIndex : incumbentIndex;
  if (
    isUnsafe(challenger) ||
    isUnsafe(spirit) ||
    challengerSnapshot.allowsImmediateOpponentWin ||
    spiritSnapshot.allowsImmediateOpponentWin ||
    challengerSnapshot.opponentReachesMatchPoint ||
    spiritSnapshot.opponentReachesMatchPoint ||
    challenger.manaHandoffToOpponent ||
    spirit.manaHandoffToOpponent ||
    challenger.hasRoundtrip ||
    spirit.hasRoundtrip ||
    challenger.ownDrainerVulnerable ||
    spirit.ownDrainerVulnerable ||
    challenger.ownDrainerWalkVulnerable ||
    spirit.ownDrainerWalkVulnerable ||
    challenger.spiritSameTurnScoreSetupNow ||
    challenger.spiritOwnManaSetupNow ||
    spirit.spiritSameTurnScoreSetupNow ||
    spirit.spiritOwnManaSetupNow ||
    challenger.winsImmediately ||
    spirit.winsImmediately ||
    challenger.attacksOpponentDrainer ||
    spirit.attacksOpponentDrainer ||
    challenger.sameTurnScoreWindowValue > 0 ||
    spirit.sameTurnScoreWindowValue > 0
  ) {
    return undefined;
  }
  const challengerScore =
    scores.get(challengerIndex) ??
    spiritFollowupFloorScore(execution, challenger.game, perspective, config);
  scores.set(challengerIndex, challengerScore);
  const spiritScore =
    scores.get(spiritIndex) ??
    spiritFollowupFloorScore(execution, spirit.game, perspective, config);
  scores.set(spiritIndex, spiritScore);
  const standardFloorCompetes =
    saturatingScoreAdd(challengerSnapshot.worstReplyScore, 192) >=
    spiritSnapshot.worstReplyScore;
  const relaxedOpeningTempoCompetes =
    game.turnNumber <= 2 &&
    !challenger.supermanaProgress &&
    !challenger.opponentManaProgress &&
    challenger.score >= saturatingScoreAdd(spirit.score, 48) &&
    challengerSnapshot.worstReplyScore >= 96 &&
    challengerScore >= spiritScore;
  if (
    saturatingScoreAdd(challenger.score, 32) < spirit.score ||
    (!standardFloorCompetes && !relaxedOpeningTempoCompetes)
  ) {
    return undefined;
  }
  if (
    challengerScore >= saturatingScoreAdd(spiritScore, 48) ||
    relaxedOpeningTempoCompetes
  ) {
    return candidatePlain ? -1 : 1;
  }
  return undefined;
}

export function whiteSpiritFollowupSetupReplyOrder(
  game: MonsGame,
  candidate: EvaluatedRoot,
  candidateSnapshot: RootReplyRiskSnapshot,
  incumbent: EvaluatedRoot,
  incumbentSnapshot: RootReplyRiskSnapshot,
  config: AutomoveConfig,
): number | undefined {
  if (
    !isProductionModeWhiteSpiritFollowupSetupPair(
      game,
      candidate,
      incumbent,
      config,
    )
  ) {
    return undefined;
  }
  const candidateSetup =
    candidate.spiritOwnManaSetupNow &&
    candidate.opponentManaProgress &&
    !isPlainSpiritDevelopmentRoot(candidate);
  const setup = candidateSetup ? candidate : incumbent;
  const setupSnapshot = candidateSetup ? candidateSnapshot : incumbentSnapshot;
  const plain = candidateSetup ? incumbent : candidate;
  const plainSnapshot = candidateSetup ? incumbentSnapshot : candidateSnapshot;
  if (
    !sameFirstInput(setup, plain) ||
    setup.efficiency !== plain.efficiency ||
    setup.ownDrainerVulnerable !== plain.ownDrainerVulnerable ||
    setup.ownDrainerWalkVulnerable !== plain.ownDrainerWalkVulnerable ||
    setup.manaHandoffToOpponent ||
    plain.manaHandoffToOpponent ||
    setup.hasRoundtrip ||
    plain.hasRoundtrip ||
    setup.winsImmediately ||
    plain.winsImmediately ||
    setup.attacksOpponentDrainer ||
    plain.attacksOpponentDrainer ||
    setup.scoresSupermanaThisTurn ||
    plain.scoresSupermanaThisTurn ||
    setup.scoresOpponentManaThisTurn ||
    plain.scoresOpponentManaThisTurn ||
    setup.safeSupermanaPickupNow ||
    plain.safeSupermanaPickupNow ||
    setup.safeOpponentManaPickupNow ||
    plain.safeOpponentManaPickupNow ||
    setup.sameTurnScoreWindowValue > 0 ||
    plain.sameTurnScoreWindowValue > 0 ||
    setup.supermanaProgress ||
    plain.supermanaProgress ||
    setupSnapshot.allowsImmediateOpponentWin ||
    plainSnapshot.allowsImmediateOpponentWin ||
    setupSnapshot.opponentReachesMatchPoint ||
    plainSnapshot.opponentReachesMatchPoint
  ) {
    return undefined;
  }
  const floorMargin =
    game.turnNumber === 3 && game.monsMovesCount === 1 ? 192 : 96;
  if (
    saturatingScoreAdd(setup.score, 96) < plain.score ||
    saturatingScoreAdd(setupSnapshot.worstReplyScore, floorMargin) <
      plainSnapshot.worstReplyScore ||
    setup.rootRank > plain.rootRank + 8
  ) {
    return undefined;
  }
  return candidateSetup ? 1 : -1;
}
