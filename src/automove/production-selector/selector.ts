import { inputChainsEqual, type Input } from "../../engine/domain.js";
import type { MonsGame } from "../../engine/game.js";
import { productionRootAdvisorPriorityInputs } from "../advisor.js";
import type { AutomoveExecutionContext } from "../execution-context.js";
import { clearExactStateAnalysisCache } from "../exact.js";
import { clearReplyRiskCache } from "../reply-risk.js";
import { rankRootCandidates } from "../root-candidates.js";
import { clearSearchCaches } from "../search.js";
import { patchAutomoveConfig } from "../selector-config.js";
import {
  AUTOMOVE_TURN_ENGINE_MODE,
  type AutomoveConfig,
} from "../selector-types.js";
import {
  TurnEngineMode,
  clearTurnEnginePlanCache,
  turnEngineCachedStep,
  turnEngineCandidatePlan,
  turnEngineCandidatePlanLive,
  turnEngineNextInputsFromAllowedHeads,
} from "../turn-engine.js";
import {
  applyProductionLowBudgetSearchClamp,
  modeFromConfig,
  productionIsSafeEarlyBlackOpeningState,
  productionIsWhiteTurnOneManaOnlyFollowup,
  productionLowBudgetGuardLive,
  productionMidTurnTacticalGuardLive,
  productionUseFreshLiveHeadPlan,
  shouldDisableProductionMidTurnTacticalEngine,
  shouldSkipProductionLowBudgetState,
  turnEngineConfigForGame,
  turnEngineRerankConfig,
} from "./config.js";
import { acceptTurnEngineHeadAfterSearch } from "./head-acceptance.js";
import {
  advisorConflictsWithChoice,
  allowedRerankOverrideCandidate,
  commitPlanAndSeedFollowup,
  productionRootAdvisorPresearch,
} from "./plan-support.js";
import {
  acceptedRerankInputs,
  searchAndSelectRoot,
  tryTurnHeadRerank,
} from "./search-integration.js";
import {
  acceptTurnEngineCachedStep,
  classifyTurnEngineRerankOverride,
  forcedLowBudgetTurnEnginePrepassChoice,
  forcedTacticalPrepassChoice,
  shouldInvokeTurnHeadRerank,
  shouldResumeTurnEngineCachedStep,
  shouldSkipProductionHeadPlanForRootContext,
} from "./tactical-prepass.js";

function smartSearchBestInputsInternal(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  initialConfig: AutomoveConfig,
  useTranspositionTable: boolean,
): Input[] {
  if (execution.session.checkpoint()) return [];
  clearSearchCaches(execution);
  clearReplyRiskCache(execution);
  let config = patchAutomoveConfig(initialConfig, {
    search: { transpositionTable: useTranspositionTable },
  });
  const perspective = game.activeColor;
  const liveEngineConfig = turnEngineConfigForGame(game, config);
  let precheckedCached: Input[] | undefined;
  if (productionLowBudgetGuardLive(config)) {
    precheckedCached = turnEngineCachedStep(execution, game, liveEngineConfig);
    if (
      precheckedCached === undefined &&
      shouldSkipProductionLowBudgetState(execution, game)
    ) {
      config = patchAutomoveConfig(config, {
        planner: { enabled: false },
      });
    }
  }
  if (
    productionMidTurnTacticalGuardLive(config) &&
    shouldDisableProductionMidTurnTacticalEngine(execution, game)
  ) {
    precheckedCached = undefined;
    config = patchAutomoveConfig(config, {
      planner: { enabled: false },
    });
  }
  if (
    productionLowBudgetGuardLive(config) &&
    productionIsSafeEarlyBlackOpeningState(execution, game)
  ) {
    config = patchAutomoveConfig(config, {
      planner: {
        secondaryAnalysis: false,
        selectedFollowupProjection: false,
      },
    });
  }
  config = applyProductionLowBudgetSearchClamp(game, config);
  if (
    !config.planner.enabled &&
    config.planner.mode === AUTOMOVE_TURN_ENGINE_MODE.Production &&
    productionIsWhiteTurnOneManaOnlyFollowup(game)
  ) {
    return smartSearchBestInputsInternal(
      execution,
      game,
      patchAutomoveConfig(config, {
        planner: { mode: AUTOMOVE_TURN_ENGINE_MODE.Baseline },
      }),
      useTranspositionTable,
    );
  }

  const roots = rankRootCandidates(execution, game, perspective, config);
  if (execution.session.cancelled || roots.length === 0) return [];
  if (config.planner.enabled) {
    const mode = modeFromConfig(config);
    const cachedCandidate =
      precheckedCached ??
      turnEngineCachedStep(execution, game, liveEngineConfig);
    const cached =
      cachedCandidate !== undefined &&
      acceptTurnEngineCachedStep(roots, cachedCandidate, mode)
        ? cachedCandidate
        : undefined;
    const engineConfig = turnEngineConfigForGame(game, config);
    const skipHead = shouldSkipProductionHeadPlanForRootContext(
      game,
      roots,
      config,
    );
    const headPlan = skipHead
      ? undefined
      : productionUseFreshLiveHeadPlan(game, config)
        ? turnEngineCandidatePlanLive(
            execution,
            game,
            perspective,
            engineConfig,
          )
        : turnEngineCandidatePlan(execution, game, perspective, engineConfig);
    if (execution.session.checkpoint()) return [];

    if (mode === TurnEngineMode.Production && cached !== undefined) {
      if (shouldResumeTurnEngineCachedStep(roots, cached, mode)) {
        return [...cached];
      }
      if (
        headPlan?.compiledChunks[0] !== undefined &&
        inputChainsEqual(headPlan.compiledChunks[0], cached)
      ) {
        commitPlanAndSeedFollowup(
          execution,
          game,
          perspective,
          config,
          mode,
          headPlan,
          engineConfig,
        );
        return [...cached];
      }
    }

    const rerankPlan = tryTurnHeadRerank(
      execution,
      game,
      perspective,
      config,
      roots,
      headPlan,
    );
    const rerankInputs = acceptedRerankInputs(
      execution,
      game,
      perspective,
      config,
      roots,
      rerankPlan,
      headPlan,
    );
    if (rerankInputs !== undefined && rerankPlan !== undefined) {
      commitPlanAndSeedFollowup(
        execution,
        game,
        perspective,
        config,
        mode,
        rerankPlan,
        engineConfig,
      );
      return rerankInputs;
    }

    const forced = forcedTacticalPrepassChoice(
      execution,
      game,
      perspective,
      roots,
      config,
    );
    if (forced !== undefined) {
      if (
        headPlan?.compiledChunks[0] !== undefined &&
        inputChainsEqual(headPlan.compiledChunks[0], forced)
      ) {
        commitPlanAndSeedFollowup(
          execution,
          game,
          perspective,
          config,
          mode,
          headPlan,
          engineConfig,
        );
      }
      return forced;
    }
    if (headPlan !== undefined) {
      const lowBudget = forcedLowBudgetTurnEnginePrepassChoice(
        execution,
        game,
        roots,
        headPlan,
        config,
      );
      if (lowBudget !== undefined) {
        commitPlanAndSeedFollowup(
          execution,
          game,
          perspective,
          config,
          mode,
          headPlan,
          engineConfig,
        );
        return lowBudget;
      }
    }

    const advisor = productionRootAdvisorPresearch(
      execution,
      game,
      perspective,
      config,
      roots,
      headPlan,
    );
    const priority =
      advisor === undefined ? [] : productionRootAdvisorPriorityInputs(advisor);
    const searched = searchAndSelectRoot(
      execution,
      game,
      perspective,
      config,
      roots,
      priority,
    );
    if (searched.inputs.length === 0) return [];
    let selected = searched.inputs;
    if (
      headPlan !== undefined &&
      acceptTurnEngineHeadAfterSearch(
        execution,
        game,
        perspective,
        config,
        searched.evaluations,
        selected,
        headPlan,
      )
    ) {
      selected = [...(headPlan.compiledChunks[0] ?? selected)];
    }
    if (cached !== undefined && inputChainsEqual(cached, selected)) {
      return [...selected];
    }
    if (
      headPlan?.compiledChunks[0] !== undefined &&
      inputChainsEqual(headPlan.compiledChunks[0], selected)
    ) {
      commitPlanAndSeedFollowup(
        execution,
        game,
        perspective,
        config,
        mode,
        headPlan,
        engineConfig,
      );
    }
    return [...selected];
  }

  if (
    roots.length > 1 &&
    config.planner.rerankHeads &&
    shouldInvokeTurnHeadRerank(roots)
  ) {
    const allowed = roots.map((root) => root.inputs);
    const inputs = turnEngineNextInputsFromAllowedHeads(
      execution,
      game,
      perspective,
      modeFromConfig(config),
      turnEngineRerankConfig(config),
      allowed,
    );
    if (
      inputs !== undefined &&
      classifyTurnEngineRerankOverride(roots, inputs) &&
      allowedRerankOverrideCandidate(roots, inputs) &&
      !advisorConflictsWithChoice(
        execution,
        game,
        perspective,
        config,
        roots,
        undefined,
        inputs,
      )
    ) {
      return [...inputs];
    }
  }
  const forced = forcedTacticalPrepassChoice(
    execution,
    game,
    perspective,
    roots,
    config,
  );
  if (forced !== undefined) return forced;
  const advisor = productionRootAdvisorPresearch(
    execution,
    game,
    perspective,
    config,
    roots,
    undefined,
  );
  const priority =
    advisor === undefined ? [] : productionRootAdvisorPriorityInputs(advisor);
  return searchAndSelectRoot(
    execution,
    game,
    perspective,
    config,
    roots,
    priority,
  ).inputs;
}

export function clearProductionSelectorCaches(
  execution: AutomoveExecutionContext,
): void {
  clearExactStateAnalysisCache(execution);
  clearSearchCaches(execution);
  clearReplyRiskCache(execution);
  clearTurnEnginePlanCache(execution);
}

export function smartSearchBestInputs(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  executionConfig: AutomoveConfig,
  useTranspositionTable = true,
): Input[] {
  const sourceFen = game.fen();
  const result = smartSearchBestInputsInternal(
    execution,
    game,
    executionConfig,
    useTranspositionTable,
  );
  if (game.fen() !== sourceFen) {
    throw new Error("smart search mutated its source game");
  }
  return [...result];
}
