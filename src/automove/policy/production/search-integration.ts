import { Color } from "../../../api/types.js";
import { inputChainsEqual, type Input } from "../../../engine/model/domain.js";
import type { MonsGame } from "../../../engine/game/mons-game.js";
import { productionRootAdvisorPriorityInputs } from "../advisor/presearch.js";
import { productionRootPolicy } from "../advisor/index.js";
import type { AutomoveExecutionContext } from "../../core/execution-context.js";
import { pickRootWithReplyRiskGuard } from "../reply-risk/selection.js";
import { rootReplyRiskSnapshot } from "../reply-risk/snapshot.js";
import { rankRootCandidates } from "../../root/candidates.js";
import type { EvaluatedRoot, RootCandidate } from "../../root/types.js";
import { pickBaselineRootInputs } from "../../root/selection.js";
import type { RootSelectorOptions } from "../../root/selector-model.js";
import {
  focusRootCandidatesForSearch,
  searchRootCandidates,
  type SearchRootOptions,
} from "../../search/engine.js";
import { patchAutomoveConfig } from "../../config/patch.js";
import type { AutomoveConfig } from "../../config/types.js";
import { TurnPlanFamily, type TurnPlan } from "../../turn/model.js";
import {
  turnEngineCandidatePlan,
  turnEngineCandidatePlanFromAllowedHeads,
} from "../../turn/planner-cache.js";
import { utilityHasNonnegativeDenyGain } from "../../turn/ordering.js";
import { productionTurnEngineLive, turnEngineConfigForGame } from "./config.js";
import {
  turnEngineConfigFromAutomoveConfig,
  turnEngineRerankConfigFromAutomoveConfig as turnEngineRerankConfig,
} from "../../turn/config.js";
import {
  advisorConflictsWithChoice,
  allowedRerankOverrideCandidate,
  productionRootAdvisorPresearch,
} from "./advisor-support.js";
import {
  classifyTurnEngineRerankOverride,
  shouldInvokeTurnHeadRerank,
} from "./tactical-prepass.js";

function searchRootOptions(
  execution: AutomoveExecutionContext,
  perspective: Color,
  config: AutomoveConfig,
  priorityInputs: readonly (readonly Input[])[],
): SearchRootOptions {
  const options: SearchRootOptions =
    priorityInputs.length === 0 ? {} : { priorityInputs };
  if (!productionTurnEngineLive(config)) return options;
  const spiritConfig = turnEngineRerankConfig(config);
  const recoveryConfig = turnEngineConfigFromAutomoveConfig(config);
  return {
    ...options,
    qualifiesPlainSpiritPlan: (candidate: RootCandidate): boolean => {
      const plan = turnEngineCandidatePlan(
        execution,
        candidate.game,
        perspective,
        spiritConfig,
      );
      return (
        plan?.headFamily === TurnPlanFamily.SpiritImpact &&
        utilityHasNonnegativeDenyGain(plan.utility)
      );
    },
    qualifiesDrainerSafetyRecoveryPlan: (candidate: RootCandidate): boolean =>
      turnEngineCandidatePlan(execution, candidate.game, perspective, recoveryConfig)
        ?.headFamily === TurnPlanFamily.DrainerSafetyRecovery,
  };
}

/**
 * Runtime fallback observation seam. This intentionally repeats the production
 * advisor-before-focus order instead of exposing bare search results.
 */
export function focusedScoredRootsForRuntime(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  executionConfig: AutomoveConfig,
  useTranspositionTable = true,
): EvaluatedRoot[] {
  const sourceFen = game.fen();
  const config = patchAutomoveConfig(executionConfig, {
    search: { transpositionTable: useTranspositionTable },
  });
  const perspective = game.activeColor;
  const roots = rankRootCandidates(execution, game, perspective, config);
  if (execution.session.cancelled || roots.length === 0) return [];
  const enginePlan = config.planner.enabled
    ? turnEngineCandidatePlan(
        execution,
        game,
        perspective,
        turnEngineConfigForGame(game, config),
      )
    : undefined;
  const advisor = productionRootAdvisorPresearch(
    execution,
    game,
    perspective,
    config,
    roots,
    enginePlan,
  );
  const priority =
    advisor === undefined ? [] : productionRootAdvisorPriorityInputs(advisor);
  const searched = searchRootCandidates(
    execution,
    game,
    perspective,
    config,
    roots,
    searchRootOptions(execution, perspective, config, priority),
  );
  if (game.fen() !== sourceFen) {
    throw new Error("runtime root focus mutated its source game");
  }
  return [...searched.evaluations];
}

/**
 * Exact runtime rank calculation for the negative-deny fallback. It performs
 * ranking, turn-engine/advisor injection, and the normal scout allocator, then
 * stops before the scored-root loop.
 */
export function focusedCandidateRankForRuntimeInputs(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  config: AutomoveConfig,
  inputs: readonly Input[],
): number | undefined {
  const sourceFen = game.fen();
  const perspective = game.activeColor;
  const roots = rankRootCandidates(execution, game, perspective, config);
  const enginePlan = config.planner.enabled
    ? turnEngineCandidatePlan(
        execution,
        game,
        perspective,
        turnEngineConfigForGame(game, config),
      )
    : undefined;
  const advisor = productionRootAdvisorPresearch(
    execution,
    game,
    perspective,
    config,
    roots,
    enginePlan,
  );
  const priority =
    advisor === undefined ? [] : productionRootAdvisorPriorityInputs(advisor);
  const focused = focusRootCandidatesForSearch(
    execution,
    game,
    perspective,
    config,
    roots,
    searchRootOptions(execution, perspective, config, priority),
    true,
  );
  if (game.fen() !== sourceFen) {
    throw new Error("runtime root-rank focus mutated its source game");
  }
  const rank = focused.candidates.findIndex((candidate) =>
    inputChainsEqual(candidate.inputs, inputs),
  );
  return rank < 0 ? undefined : rank;
}

export function rootSelectorOptions(
  execution: AutomoveExecutionContext,
  config: AutomoveConfig,
): RootSelectorOptions {
  return {
    checkpoint: () => execution.session.checkpoint(),
    cancelled: () => execution.session.cancelled,
    rootReplyRiskSnapshot: (state, perspective, _config, replyLimit) =>
      rootReplyRiskSnapshot(execution, state, perspective, config, replyLimit),
    pickReplyRiskGuardedIndex: (context) =>
      pickRootWithReplyRiskGuard(
        execution,
        context.game,
        context.roots,
        context.candidateIndices,
        context.perspective,
        config,
      ),
    productionPolicy: productionRootPolicy(execution, config),
  };
}

export function searchAndSelectRoot(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: AutomoveConfig,
  roots: readonly RootCandidate[],
  priorityInputs: readonly (readonly Input[])[],
): { inputs: Input[]; evaluations: EvaluatedRoot[] } {
  const result = searchRootCandidates(
    execution,
    game,
    perspective,
    config,
    roots,
    searchRootOptions(execution, perspective, config, priorityInputs),
  );
  if (execution.session.cancelled) return { inputs: [], evaluations: [] };
  const evaluations = [...result.evaluations];
  if (evaluations.length === 0) return { inputs: [], evaluations };
  return {
    inputs: pickBaselineRootInputs(
      game,
      evaluations,
      perspective,
      config,
      rootSelectorOptions(execution, config),
    ),
    evaluations,
  };
}

export function tryTurnHeadRerank(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: AutomoveConfig,
  roots: readonly RootCandidate[],
  existingPlan: TurnPlan | undefined,
): TurnPlan | undefined {
  if (
    roots.length <= 1 ||
    !config.planner.rerankHeads ||
    !shouldInvokeTurnHeadRerank(roots)
  ) {
    return undefined;
  }
  const allowed = roots.map((root) => root.inputs);
  const existingHead = existingPlan?.compiledChunks[0];
  if (
    existingPlan !== undefined &&
    existingHead !== undefined &&
    allowed.some((inputs) => inputChainsEqual(inputs, existingHead))
  ) {
    return existingPlan;
  }
  return turnEngineCandidatePlanFromAllowedHeads(
    execution,
    game,
    perspective,
    turnEngineRerankConfig(config),
    allowed,
  );
}

export function acceptedRerankInputs(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: AutomoveConfig,
  roots: readonly RootCandidate[],
  plan: TurnPlan | undefined,
  advisorPlan: TurnPlan | undefined,
): Input[] | undefined {
  const inputs = plan?.compiledChunks[0];
  if (
    inputs === undefined ||
    !classifyTurnEngineRerankOverride(roots, inputs) ||
    !allowedRerankOverrideCandidate(roots, inputs) ||
    advisorConflictsWithChoice(
      execution,
      game,
      perspective,
      config,
      roots,
      advisorPlan,
      inputs,
    )
  ) {
    return undefined;
  }
  return [...inputs];
}
