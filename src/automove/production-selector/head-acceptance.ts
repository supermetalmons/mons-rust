import { Color, inputChainsEqual, type Input } from "../../engine/domain.js";
import type { MonsGame } from "../../engine/game.js";
import type { AutomoveExecutionContext } from "../execution-context.js";
import { exactOpportunityContext } from "../exact.js";
import {
  isProductionModeBlackPlainSpiritFollowupSetupPair,
  type ReplyRiskHooks,
} from "../reply-risk.js";
import { rootFamily } from "../root-family.js";
import { rootProgressStepsBetter } from "../root-selector.js";
import type { EvaluatedRoot } from "../search.js";
import { saturatingScoreAdd, saturatingScoreSubtract } from "../score-math.js";
import {
  hasConcreteScoreSurface as rootHasConcreteScoreSurface,
  isPlainSpiritDevelopmentRoot,
  rootIsUnsafe,
  type AutomoveConfig,
} from "../selector-types.js";
import {
  TurnPlanFamily,
  compareUtilityPrimaryAxes,
  utilityStrictlyDominatesOverrideAxes,
  utilitySupportsPrimaryAxesEvalTolerance,
  type TurnPlan,
} from "../turn-engine.js";
import { modeFromConfig, turnEngineModeUsesMacroPlans } from "./config.js";
import { acceptTurnEngineHeadAfterOrderedGuards } from "./head-ordered-guards.js";
import type { TurnEngineHeadAcceptanceContext } from "./head-types.js";
import { turnEngineSelectedUtility } from "./plan-support.js";
import {
  firstInputsEqual,
  isProductionModeNonConcreteManaWindowRoot,
  rootHasProgressSurface,
  valueAt,
} from "./shared.js";

function passesTurnEngineMacroDominanceGuard(
  context: TurnEngineHeadAcceptanceContext,
): boolean {
  const {
    game,
    plan,
    candidate,
    macroMode,
    selectedProgress,
    candidateFamily,
    selectedFamily,
    selectedUtilityValue,
    candidateUtilityValue,
    blackTurnSixRouteChangePlainSpirit,
    whiteSpiritSetupGain,
  } = context;
  if (!macroMode) return true;

  const selectedUtility = selectedUtilityValue();
  const candidateUtility = candidateUtilityValue();
  const blackNonConcreteWindowBlocksSpiritProgress =
    game.activeColor === Color.Black &&
    game.turnNumber <= 6 &&
    (plan.headFamily === TurnPlanFamily.SafeSupermanaProgress ||
      plan.headFamily === TurnPlanFamily.SafeOpponentManaProgress) &&
    plan.goalFamily === TurnPlanFamily.ImmediateScore &&
    candidateFamily === TurnPlanFamily.ManaTempo &&
    selectedFamily === TurnPlanFamily.SpiritImpact &&
    isProductionModeNonConcreteManaWindowRoot(candidate) &&
    selectedProgress &&
    compareUtilityPrimaryAxes(plan.headUtility, selectedUtility) < 0;
  if (blackNonConcreteWindowBlocksSpiritProgress) return false;
  const planDominates =
    compareUtilityPrimaryAxes(plan.utility, selectedUtility) > 0 &&
    (utilityStrictlyDominatesOverrideAxes(plan.utility, selectedUtility) ||
      utilityStrictlyDominatesOverrideAxes(plan.headUtility, selectedUtility));
  const candidateDominates =
    compareUtilityPrimaryAxes(candidateUtility, selectedUtility) > 0 &&
    utilityStrictlyDominatesOverrideAxes(candidateUtility, selectedUtility);
  return (
    blackTurnSixRouteChangePlainSpirit ||
    whiteSpiritSetupGain ||
    planDominates ||
    candidateDominates
  );
}

/**
 * Conservative post-search macro-plan gate. It preserves the selector's
 * family ordering, safety floor, score-gap caps, and completed-plan escape.
 */
export function acceptTurnEngineHeadAfterSearch(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: AutomoveConfig,
  roots: readonly EvaluatedRoot[],
  selectedInputs: readonly Input[],
  plan: TurnPlan,
  hooks?: ReplyRiskHooks,
): boolean {
  const candidateInputs = plan.compiledChunks[0];
  if (candidateInputs === undefined) return false;
  const candidateIndex = roots.findIndex((root) =>
    inputChainsEqual(root.inputs, candidateInputs),
  );
  const selectedIndex = roots.findIndex((root) =>
    inputChainsEqual(root.inputs, selectedInputs),
  );
  if (candidateIndex < 0 || selectedIndex < 0) return false;
  if (candidateIndex === selectedIndex) return true;
  const candidate = valueAt(roots, candidateIndex);
  const selected = valueAt(roots, selectedIndex);
  if (selected.winsImmediately && !candidate.winsImmediately) return false;

  const macroMode = turnEngineModeUsesMacroPlans(modeFromConfig(config));
  const candidateUnsafe = rootIsUnsafe(candidate);
  const selectedUnsafe = rootIsUnsafe(selected);
  const candidateProgress = rootHasProgressSurface(candidate);
  const selectedProgress = rootHasProgressSurface(selected);
  const exactContext = exactOpportunityContext(
    execution,
    game,
    game.activeColor,
  );
  const scoreGap = saturatingScoreSubtract(selected.score, candidate.score);
  const sameTurnWindowBetter =
    candidate.sameTurnScoreWindowValue > selected.sameTurnScoreWindowValue;
  const drainerAttackBetter =
    candidate.attacksOpponentDrainer && !selected.attacksOpponentDrainer;
  const scoresNowBetter =
    (candidate.scoresSupermanaThisTurn ||
      candidate.scoresOpponentManaThisTurn) &&
    !(selected.scoresSupermanaThisTurn || selected.scoresOpponentManaThisTurn);
  const safetyRecoverBetter =
    candidate.classes.drainerSafetyRecover &&
    !selected.classes.drainerSafetyRecover &&
    selected.ownDrainerVulnerable &&
    !candidate.ownDrainerVulnerable;
  const spiritWindowBetter =
    candidate.spiritSameTurnScoreSetupNow &&
    !selected.spiritSameTurnScoreSetupNow &&
    candidate.sameTurnScoreWindowValue >= selected.sameTurnScoreWindowValue;
  const spiritDevelopmentBetter =
    candidate.spiritDevelopment && !selected.spiritDevelopment;
  const candidateSpiritTactical =
    candidate.spiritSameTurnScoreSetupNow ||
    candidate.sameTurnScoreWindowValue > 0 ||
    candidate.attacksOpponentDrainer ||
    candidate.scoresSupermanaThisTurn ||
    candidate.scoresOpponentManaThisTurn ||
    candidate.safeSupermanaPickupNow ||
    candidate.safeOpponentManaPickupNow;
  const progressBetter =
    (rootProgressStepsBetter(
      candidate.safeSupermanaProgressSteps,
      selected.safeSupermanaProgressSteps,
    ) ||
      rootProgressStepsBetter(
        candidate.safeOpponentManaProgressSteps,
        selected.safeOpponentManaProgressSteps,
      )) &&
    !selected.winsImmediately &&
    !selected.attacksOpponentDrainer &&
    !selected.spiritSameTurnScoreSetupNow;
  const selectedSpiritPhase =
    selected.spiritDevelopment ||
    selected.spiritSameTurnScoreSetupNow ||
    selected.spiritOwnManaSetupNow;
  const candidateFamily = rootFamily(candidate);
  const selectedFamily = rootFamily(selected);
  let selectedUtilityCache: ReturnType<
    typeof turnEngineSelectedUtility
  > | null = null;
  const selectedUtilityValue = (): ReturnType<
    typeof turnEngineSelectedUtility
  > => {
    selectedUtilityCache ??= turnEngineSelectedUtility(
      execution,
      game,
      selected,
      perspective,
      config,
      hooks,
    );
    return selectedUtilityCache;
  };
  let candidateUtilityCache: ReturnType<
    typeof turnEngineSelectedUtility
  > | null = null;
  const candidateUtilityValue = (): ReturnType<
    typeof turnEngineSelectedUtility
  > => {
    candidateUtilityCache ??= turnEngineSelectedUtility(
      execution,
      game,
      candidate,
      perspective,
      config,
      hooks,
    );
    return candidateUtilityCache;
  };
  const blackSpiritPair =
    macroMode &&
    plan.headFamily === TurnPlanFamily.SpiritImpact &&
    isProductionModeBlackPlainSpiritFollowupSetupPair(
      game,
      candidate,
      selected,
      config,
    ) &&
    candidate.score > selected.score;
  const whiteSpiritSetupGain =
    macroMode &&
    game.activeColor === Color.White &&
    plan.headFamily === TurnPlanFamily.SpiritImpact &&
    !selectedUnsafe &&
    !candidateUnsafe &&
    candidate.spiritOwnManaSetupNow &&
    !candidate.spiritSameTurnScoreSetupNow &&
    !selected.spiritOwnManaSetupNow &&
    !selected.spiritSameTurnScoreSetupNow &&
    selected.spiritDevelopment &&
    candidate.spiritDevelopment &&
    candidate.spiritSetupGain >=
      saturatingScoreAdd(selected.spiritSetupGain, 32) &&
    candidate.safeSupermanaProgressSteps <=
      selected.safeSupermanaProgressSteps &&
    candidate.safeOpponentManaProgressSteps <=
      selected.safeOpponentManaProgressSteps &&
    candidate.ownDrainerVulnerable === selected.ownDrainerVulnerable &&
    candidate.manaHandoffToOpponent === selected.manaHandoffToOpponent &&
    candidate.hasRoundtrip === selected.hasRoundtrip &&
    !scoresNowBetter &&
    !drainerAttackBetter &&
    scoreGap <= 96;
  const blackTurnSixRouteChangePlainSpirit =
    macroMode &&
    game.activeColor === Color.Black &&
    game.turnNumber === 6 &&
    game.monsMovesCount === 0 &&
    game.playerCanUseAction() &&
    game.playerCanMoveMana() &&
    plan.headFamily === TurnPlanFamily.SpiritImpact &&
    plan.goalFamily === TurnPlanFamily.SpiritImpact &&
    candidateIndex <= 1 &&
    candidateUnsafe === selectedUnsafe &&
    isPlainSpiritDevelopmentRoot(candidate) &&
    !candidateProgress &&
    !candidateSpiritTactical &&
    selectedFamily === TurnPlanFamily.ManaTempo &&
    selectedUnsafe &&
    !selected.spiritDevelopment &&
    !selected.spiritSameTurnScoreSetupNow &&
    !selected.spiritOwnManaSetupNow &&
    !selectedProgress &&
    !rootHasConcreteScoreSurface(selected) &&
    selected.sameTurnScoreWindowValue === 0 &&
    !firstInputsEqual(candidate.inputs, selected.inputs) &&
    candidate.ownDrainerVulnerable === selected.ownDrainerVulnerable &&
    candidate.ownDrainerWalkVulnerable === selected.ownDrainerWalkVulnerable &&
    candidate.manaHandoffToOpponent === selected.manaHandoffToOpponent &&
    candidate.hasRoundtrip === selected.hasRoundtrip &&
    scoreGap <= 1_024 &&
    utilitySupportsPrimaryAxesEvalTolerance(
      candidateUtilityValue(),
      selectedUtilityValue(),
      64,
    );

  const context: TurnEngineHeadAcceptanceContext = {
    game,
    perspective,
    config,
    plan,
    candidateIndex,
    candidate,
    selected,
    macroMode,
    candidateUnsafe,
    selectedUnsafe,
    candidateProgress,
    selectedProgress,
    exactContext,
    scoreGap,
    sameTurnWindowBetter,
    drainerAttackBetter,
    scoresNowBetter,
    safetyRecoverBetter,
    spiritWindowBetter,
    spiritDevelopmentBetter,
    candidateSpiritTactical,
    progressBetter,
    selectedSpiritPhase,
    candidateFamily,
    selectedFamily,
    selectedUtilityValue,
    candidateUtilityValue,
    blackSpiritPair,
    whiteSpiritSetupGain,
    blackTurnSixRouteChangePlainSpirit,
  };
  if (!passesTurnEngineMacroDominanceGuard(context)) return false;
  return acceptTurnEngineHeadAfterOrderedGuards(execution, context);
}
