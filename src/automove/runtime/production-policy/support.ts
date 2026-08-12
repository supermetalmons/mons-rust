import { inputChainsEqual, type Input } from "../../../engine/model/domain.js";
import type { MonsGame } from "../../../engine/game/mons-game.js";
import type { ExactOpportunityContext } from "../../exact/types.js";
import type { AutomoveExecutionContext } from "../../core/execution-context.js";
import { replyRiskGuardShortlistIndices } from "../../policy/reply-risk/shortlist.js";
import type { RootCandidate } from "../../root/types.js";
import { rootFamily } from "../../root/family.js";
import { filteredRootCandidateIndices } from "../../root/filtering.js";
import type { EvaluatedRoot } from "../../root/types.js";
import { patchAutomoveConfig } from "../../config/patch.js";
import { withProductionPlanner } from "../../config/runtime.js";
import {
  focusedScoredRootsForRuntime,
  rootSelectorOptions,
} from "../../policy/production/search-integration.js";
import { turnEngineConfigFromAutomoveConfig } from "../../turn/config.js";
import { AUTOMOVE_TURN_ENGINE_MODE, type AutomoveConfig } from "../../config/types.js";
import { TurnPlanFamily, type TurnPlan, type TurnUtility } from "../../turn/model.js";
import { turnEngineEvaluatePlanWithReplies } from "../../turn/planner-cache.js";
import { turnEngineEvaluateStateUtility } from "../../turn/evaluation.js";
import {
  isOwnDrainerVulnerable,
  isOwnDrainerWalkVulnerable,
} from "../../root/vulnerability.js";

export function ownDrainerUnsafe(
  execution: AutomoveExecutionContext,
  game: MonsGame,
): boolean {
  return (
    isOwnDrainerVulnerable(execution, game, game.activeColor) ||
    isOwnDrainerWalkVulnerable(execution, game, game.activeColor)
  );
}

export function evaluateSelectedUtility(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  root: RootCandidate,
  config: AutomoveConfig,
): TurnUtility {
  const engineConfig = turnEngineConfigFromAutomoveConfig(config);
  const headUtility = turnEngineEvaluateStateUtility(
    execution,
    root.game,
    game,
    game.activeColor,
    engineConfig,
  );
  const family = rootFamily(root);
  const plan: TurnPlan = {
    actions: [],
    compiledChunks: [root.inputs],
    endGame: root.game.fork(),
    utility: headUtility,
    headUtility,
    headFamily: family,
    goalFamily: family,
    packageMeta: {
      scoreGain: 0,
      denyGain: 0,
      drainerSafetyDelta: 0,
      spiritOnlySetup: false,
      endsNonnegativeDrainerSafety: true,
      opponentImmediateWindowAfter: 0,
    },
  };
  return turnEngineEvaluatePlanWithReplies(
    execution,
    game,
    plan,
    game.activeColor,
    engineConfig,
  );
}

export function findRoot(
  roots: readonly RootCandidate[],
  inputs: readonly Input[],
): RootCandidate | undefined {
  return roots.find((root) => inputChainsEqual(root.inputs, inputs));
}

export function whiteDenyFallbackContextEligible(
  context: ExactOpportunityContext,
): boolean {
  return (
    !context.opponentCanWinImmediately &&
    context.delta.sameTurnScoreWindowValue === 1 &&
    context.delta.opponentWindowDenyGain === 1 &&
    !context.delta.drainerAttackAvailable &&
    context.delta.drainerSafety < 0
  );
}

export function safeQuietManaTempoRoot(root: EvaluatedRoot): boolean {
  return (
    !root.winsImmediately &&
    !root.attacksOpponentDrainer &&
    !root.ownDrainerVulnerable &&
    !root.ownDrainerWalkVulnerable &&
    !root.spiritDevelopment &&
    !root.spiritSameTurnScoreSetupNow &&
    !root.spiritOwnManaSetupNow &&
    !root.manaHandoffToOpponent &&
    !root.hasRoundtrip &&
    !root.scoresSupermanaThisTurn &&
    !root.scoresOpponentManaThisTurn &&
    !root.safeSupermanaPickupNow &&
    !root.safeOpponentManaPickupNow &&
    root.sameTurnScoreWindowValue === 0 &&
    !root.supermanaProgress &&
    !root.opponentManaProgress &&
    !root.classes.immediateScore &&
    !root.classes.drainerAttack &&
    !root.classes.drainerSafetyRecover &&
    !root.classes.carrierProgress &&
    !root.classes.material &&
    root.classes.quiet &&
    rootFamily(root) === TurnPlanFamily.ManaTempo
  );
}

export function confirmBaselineContextEligible(
  context: ExactOpportunityContext,
): boolean {
  return (
    !context.opponentCanWinImmediately &&
    context.delta.sameTurnScoreWindowValue === 0 &&
    context.delta.spiritGain === 0 &&
    context.delta.opponentWindowDenyGain === 0 &&
    !context.delta.drainerAttackAvailable &&
    context.delta.safeSupermanaProgressSteps === undefined &&
    context.delta.safeOpponentManaProgressSteps === undefined &&
    context.delta.drainerSafety >= 0
  );
}

export function productionRuntimeCompetition(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  base: AutomoveConfig,
  productionInputs: readonly Input[],
):
  | {
      readonly config: AutomoveConfig;
      readonly roots: readonly EvaluatedRoot[];
      readonly productionIndex: number;
      readonly candidateIndices: readonly number[];
      readonly shortlist: readonly number[];
    }
  | undefined {
  const config = withProductionPlanner(base);
  const roots = focusedScoredRootsForRuntime(execution, game, config, true);
  const productionIndex = roots.findIndex((root) =>
    inputChainsEqual(root.inputs, productionInputs),
  );
  if (productionIndex < 0) return undefined;
  const candidateIndices = filteredRootCandidateIndices(
    game,
    roots,
    game.activeColor,
    config,
    rootSelectorOptions(execution, config),
  );
  if (!candidateIndices.includes(productionIndex)) return undefined;
  const shortlist = replyRiskGuardShortlistIndices(
    execution,
    roots,
    candidateIndices,
    config,
  );
  if (!shortlist.includes(productionIndex)) return undefined;
  return {
    config,
    roots,
    productionIndex,
    candidateIndices,
    shortlist,
  };
}

export function searchOnlyBaselineConfig(production: AutomoveConfig): AutomoveConfig {
  return patchAutomoveConfig(production, {
    planner: {
      enabled: false,
      rerankHeads: true,
      mode: AUTOMOVE_TURN_ENGINE_MODE.Baseline,
    },
  });
}
