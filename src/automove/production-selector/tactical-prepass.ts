import { TARGET_SCORE } from "../../engine/config.js";
import { Color, inputChainsEqual, type Input } from "../../engine/domain.js";
import type { MonsGame } from "../../engine/game.js";
import type { AutomoveExecutionContext } from "../execution-context.js";
import { compareTacticalRootCandidatesIgnoringScorePath } from "../root-focus.js";
import { saturatingScoreAdd, saturatingScoreSubtract } from "../score-math.js";
import type { RootCandidate } from "../root-candidates.js";
import { rootProgressStepsBetter } from "../root-selector.js";
import {
  AUTOMOVE_TURN_ENGINE_MODE,
  isPlainSpiritDevelopmentRoot,
  rootIsUnsafe,
  type AutomoveConfig,
} from "../selector-types.js";
import { productionIsEarlyWhiteTurnStart } from "../turn-engine-config.js";
import {
  TurnEngineMode,
  TurnPlanFamily,
  type TurnPlan,
} from "../turn-engine.js";
import {
  productionIsSafeEarlyBlackOpeningState,
  turnEngineModeUsesMacroPlans,
} from "./config.js";
import {
  ownDrainerVulnerableNextTurn,
  rootHasProgressSurface,
  valueAt,
} from "./shared.js";

function bestTacticalRootIndex(
  roots: readonly RootCandidate[],
  predicate: (root: RootCandidate) => boolean,
): number | undefined {
  let best: number | undefined;
  roots.forEach((root, index) => {
    if (
      predicate(root) &&
      (best === undefined ||
        compareTacticalRootCandidatesIgnoringScorePath(
          root,
          valueAt(roots, best),
        ) < 0)
    ) {
      best = index;
    }
  });
  return best;
}

export function forcedTacticalPrepassChoice(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  roots: readonly RootCandidate[],
  config: AutomoveConfig,
): Input[] | undefined {
  if (!config.policy.forcedTacticalPrepass || roots.length === 0) {
    return undefined;
  }
  const choose = (
    predicate: (root: RootCandidate) => boolean,
  ): Input[] | undefined => {
    const index = bestTacticalRootIndex(roots, predicate);
    return index === undefined ? undefined : [...valueAt(roots, index).inputs];
  };
  let choice = choose((root) => root.winsImmediately);
  if (choice !== undefined) return choice;

  const hasSupermanaScoring =
    config.policy.supermanaPrepassException &&
    roots.some((root) => root.scoresSupermanaThisTurn);
  const safeSupermanaPickup = (root: RootCandidate): boolean =>
    root.safeSupermanaPickupNow &&
    !root.ownDrainerVulnerable &&
    !root.manaHandoffToOpponent &&
    !root.winsImmediately &&
    !root.attacksOpponentDrainer;
  const hasSafeSupermanaPickup =
    config.policy.supermanaPrepassException && roots.some(safeSupermanaPickup);
  const hasException = hasSupermanaScoring || hasSafeSupermanaPickup;
  if (config.policy.supermanaPrepassException) {
    choice = choose((root) => root.scoresSupermanaThisTurn);
    if (choice !== undefined) return choice;
    choice = choose(safeSupermanaPickup);
    if (choice !== undefined) return choice;
  }
  if (!hasException) {
    choice = choose((root) => root.attacksOpponentDrainer);
    if (choice !== undefined) return choice;
  }
  if (
    !hasException &&
    ownDrainerVulnerableNextTurn(execution, game, perspective)
  ) {
    choice = choose((root) => !root.ownDrainerVulnerable);
    if (choice !== undefined) return choice;
  }
  const opponentScore =
    perspective === Color.White ? game.blackScore : game.whiteScore;
  if (TARGET_SCORE - opponentScore <= 1) {
    return choose((root) => root.classes.immediateScore);
  }
  return undefined;
}

export function acceptTurnEngineCachedStep(
  roots: readonly RootCandidate[],
  cachedInputs: readonly Input[],
  mode: TurnEngineMode,
): boolean {
  const index = roots.findIndex((root) =>
    inputChainsEqual(root.inputs, cachedInputs),
  );
  if (index < 0) return false;
  if (index === 0) return true;
  const top = roots[0];
  if (top === undefined) return false;
  if (top.winsImmediately) return false;
  const candidate = valueAt(roots, index);
  const gap = saturatingScoreSubtract(top.heuristic, candidate.heuristic);
  if (mode === TurnEngineMode.Baseline) {
    return index <= 2 && gap <= 96 && !rootIsUnsafe(candidate);
  }
  const candidateTactical =
    candidate.winsImmediately ||
    candidate.attacksOpponentDrainer ||
    candidate.classes.drainerSafetyRecover ||
    candidate.spiritSameTurnScoreSetupNow ||
    candidate.spiritOwnManaSetupNow ||
    candidate.spiritDevelopment ||
    candidate.scoresSupermanaThisTurn ||
    candidate.scoresOpponentManaThisTurn ||
    candidate.safeSupermanaPickupNow ||
    candidate.safeOpponentManaPickupNow ||
    rootProgressStepsBetter(
      candidate.safeSupermanaProgressSteps,
      top.safeSupermanaProgressSteps,
    ) ||
    rootProgressStepsBetter(
      candidate.safeOpponentManaProgressSteps,
      top.safeOpponentManaProgressSteps,
    );
  const candidateUnsafe = rootIsUnsafe(candidate);
  return (
    (!candidateUnsafe && index <= 4 && gap <= 128) ||
    (!candidateUnsafe && candidateTactical && index <= 8 && gap <= 224) ||
    (!candidateUnsafe && rootIsUnsafe(top) && index <= 10 && gap <= 256)
  );
}

export function shouldResumeTurnEngineCachedStep(
  roots: readonly RootCandidate[],
  cachedInputs: readonly Input[],
  mode: TurnEngineMode,
): boolean {
  return (
    turnEngineModeUsesMacroPlans(mode) &&
    acceptTurnEngineCachedStep(roots, cachedInputs, mode) &&
    inputChainsEqual(roots[0]?.inputs ?? [], cachedInputs)
  );
}

export function shouldSkipProductionHeadPlanForRootContext(
  game: MonsGame,
  roots: readonly RootCandidate[],
  config: AutomoveConfig,
): boolean {
  if (
    config.planner.mode !== AUTOMOVE_TURN_ENGINE_MODE.Production ||
    !config.planner.lowBudgetGuard ||
    roots.length === 0
  ) {
    return false;
  }
  if (productionIsEarlyWhiteTurnStart(game)) return false;
  if (
    !game.playerCanUseAction() &&
    !game.playerCanMoveMana() &&
    game.monsMovesCount >= 4
  ) {
    return true;
  }
  return (
    game.activeColor === Color.Black &&
    game.turnNumber === 2 &&
    game.monsMovesCount <= 1
  );
}

export function forcedLowBudgetTurnEnginePrepassChoice(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  roots: readonly RootCandidate[],
  plan: TurnPlan,
  config: AutomoveConfig,
): Input[] | undefined {
  if (
    config.planner.mode !== AUTOMOVE_TURN_ENGINE_MODE.Production ||
    !config.planner.lowBudgetGuard ||
    roots.length === 0
  ) {
    return undefined;
  }
  const lowBudgetOpening =
    productionIsEarlyWhiteTurnStart(game) ||
    productionIsSafeEarlyBlackOpeningState(execution, game);
  if (!lowBudgetOpening) return undefined;
  const inputs = plan.compiledChunks[0];
  if (inputs === undefined) return undefined;
  const index = roots.findIndex((root) =>
    inputChainsEqual(root.inputs, inputs),
  );
  if (index < 0) return undefined;
  const candidate = valueAt(roots, index);
  if (rootIsUnsafe(candidate)) return undefined;
  const top = roots[0];
  if (top === undefined) return undefined;
  if (
    top.winsImmediately ||
    top.attacksOpponentDrainer ||
    top.classes.drainerSafetyRecover
  ) {
    return undefined;
  }
  const gap = saturatingScoreSubtract(top.heuristic, candidate.heuristic);
  const progressFamily =
    plan.headFamily === TurnPlanFamily.SafeSupermanaProgress ||
    plan.headFamily === TurnPlanFamily.SafeOpponentManaProgress ||
    plan.headFamily === TurnPlanFamily.DrainerSafetyRecovery;
  const topPlainSpirit = isPlainSpiritDevelopmentRoot(top);
  const candidateProgress = rootHasProgressSurface(candidate);
  if (index === 0 && progressFamily) return [...candidate.inputs];
  if (
    productionIsEarlyWhiteTurnStart(game) &&
    topPlainSpirit &&
    !rootIsUnsafe(top) &&
    candidateProgress &&
    index <= 2 &&
    gap <= 96
  ) {
    return [...candidate.inputs];
  }
  if (
    productionIsSafeEarlyBlackOpeningState(execution, game) &&
    index <= 1 &&
    gap <= 96 &&
    (candidateProgress || candidate.spiritDevelopment)
  ) {
    return [...candidate.inputs];
  }
  return undefined;
}

export function shouldInvokeTurnHeadRerank(
  roots: readonly RootCandidate[],
): boolean {
  const tactical = (root: RootCandidate): boolean =>
    root.winsImmediately ||
    root.attacksOpponentDrainer ||
    root.sameTurnScoreWindowValue >= 2 ||
    root.scoresSupermanaThisTurn ||
    root.scoresOpponentManaThisTurn ||
    root.safeSupermanaPickupNow ||
    root.safeOpponentManaPickupNow ||
    root.supermanaProgress ||
    root.opponentManaProgress ||
    root.spiritSameTurnScoreSetupNow ||
    root.spiritDevelopment ||
    root.classes.drainerSafetyRecover;
  return (
    roots[0] !== undefined &&
    (tactical(roots[0]) || roots.slice(0, 3).some(tactical))
  );
}

export function classifyTurnEngineRerankOverride(
  roots: readonly RootCandidate[],
  overrideInputs: readonly Input[],
): boolean {
  const index = roots.findIndex((root) =>
    inputChainsEqual(root.inputs, overrideInputs),
  );
  const top = roots[0];
  if (index < 0 || top === undefined) return false;
  const topUnsafe = rootIsUnsafe(top);
  if (index === 0) {
    const topTactical =
      top.winsImmediately ||
      top.attacksOpponentDrainer ||
      top.scoresSupermanaThisTurn ||
      top.scoresOpponentManaThisTurn ||
      top.safeSupermanaPickupNow ||
      top.safeOpponentManaPickupNow ||
      top.supermanaProgress ||
      top.opponentManaProgress ||
      top.sameTurnScoreWindowValue > 0 ||
      top.spiritSameTurnScoreSetupNow ||
      top.spiritDevelopment ||
      top.classes.drainerSafetyRecover;
    return topTactical && !topUnsafe;
  }
  const candidate = valueAt(roots, index);
  if (candidate.winsImmediately) return true;
  if (top.winsImmediately || rootIsUnsafe(candidate)) return false;
  const decisive =
    candidate.attacksOpponentDrainer ||
    candidate.scoresSupermanaThisTurn ||
    candidate.scoresOpponentManaThisTurn ||
    candidate.safeSupermanaPickupNow ||
    candidate.safeOpponentManaPickupNow;
  const soft =
    candidate.sameTurnScoreWindowValue > 0 ||
    candidate.spiritSameTurnScoreSetupNow;
  const gap = saturatingScoreSubtract(top.heuristic, candidate.heuristic);
  const materialAdvantage =
    (candidate.attacksOpponentDrainer && !top.attacksOpponentDrainer) ||
    ((candidate.scoresSupermanaThisTurn ||
      candidate.scoresOpponentManaThisTurn) &&
      !(top.scoresSupermanaThisTurn || top.scoresOpponentManaThisTurn)) ||
    ((candidate.safeSupermanaPickupNow ||
      candidate.safeOpponentManaPickupNow) &&
      !(top.safeSupermanaPickupNow || top.safeOpponentManaPickupNow)) ||
    (candidate.classes.drainerSafetyRecover &&
      !top.classes.drainerSafetyRecover &&
      top.ownDrainerVulnerable &&
      !candidate.ownDrainerVulnerable) ||
    candidate.sameTurnScoreWindowValue >
      saturatingScoreAdd(top.sameTurnScoreWindowValue, 1);
  const progressBetter =
    candidate.safeSupermanaProgressSteps < top.safeSupermanaProgressSteps ||
    candidate.safeOpponentManaProgressSteps < top.safeOpponentManaProgressSteps;
  const safer =
    (!candidate.ownDrainerVulnerable && top.ownDrainerVulnerable) ||
    (!candidate.manaHandoffToOpponent && top.manaHandoffToOpponent);
  const progress =
    candidate.supermanaProgress ||
    candidate.opponentManaProgress ||
    progressBetter;
  const progressOnly =
    progress &&
    !decisive &&
    !soft &&
    !candidate.classes.drainerSafetyRecover &&
    !candidate.spiritSameTurnScoreSetupNow;
  if (progressOnly) {
    return (safer || topUnsafe) && index <= 4 && gap <= 180;
  }
  const safetyProgressFallback =
    safer && progressBetter && index <= 5 && gap <= 220;
  if (decisive || soft || candidate.classes.drainerSafetyRecover || progress) {
    if (
      candidate.ownDrainerVulnerable &&
      !candidate.classes.drainerSafetyRecover
    ) {
      return false;
    }
    const signal =
      materialAdvantage ||
      (progressBetter && (safer || topUnsafe)) ||
      (candidate.classes.drainerSafetyRecover &&
        top.ownDrainerVulnerable &&
        !candidate.ownDrainerVulnerable) ||
      (candidate.spiritSameTurnScoreSetupNow &&
        candidate.sameTurnScoreWindowValue > top.sameTurnScoreWindowValue);
    return (
      (signal && index <= 6 && gap <= 520) ||
      (materialAdvantage && index <= 8 && gap <= 640)
    );
  }
  return safetyProgressFallback;
}
