import { Color, inputChainsEqual, type Input } from "../../engine/domain.js";
import type { MonsGame } from "../../engine/game.js";
import { productionRootAdvisorPresearch as runProductionRootAdvisorPresearch } from "../advisor.js";
import type { AutomoveExecutionContext } from "../execution-context.js";
import { hasRoundtripMonMove, manaHandoffPenalty } from "../move-efficiency.js";
import {
  turnEngineSelectedOverrideUtility,
  type ReplyRiskHooks,
} from "../reply-risk.js";
import {
  buildRootCandidateForInputs,
  rankRootCandidates,
  type RootCandidate,
} from "../root-candidates.js";
import { rootFamily } from "../root-family.js";
import type { EvaluatedRoot } from "../search.js";
import type { AutomoveConfig } from "../selector-types.js";
import {
  TurnEngineMode,
  turnEngineCommitPlan,
  turnEngineStoreCachedStep,
  type TurnEngineConfig,
  type TurnPlan,
} from "../turn-engine.js";
import {
  applyInputsForSearch,
  applyInputsForSearchWithEvents,
} from "../transitions.js";
import {
  turnEngineConfigForGame,
  turnEngineModeUsesMacroPlans,
} from "./config.js";
import { ownDrainerVulnerableNextTurn } from "./shared.js";
import { shouldResumeTurnEngineCachedStep } from "./tactical-prepass.js";

export function allowedRerankOverrideCandidate(
  roots: readonly RootCandidate[],
  inputs: readonly Input[],
): boolean {
  const root = roots.find((candidate) =>
    inputChainsEqual(candidate.inputs, inputs),
  );
  return (
    root !== undefined &&
    (root.winsImmediately ||
      root.attacksOpponentDrainer ||
      root.scoresSupermanaThisTurn ||
      root.scoresOpponentManaThisTurn ||
      root.safeSupermanaPickupNow ||
      root.safeOpponentManaPickupNow ||
      root.classes.drainerSafetyRecover ||
      root.sameTurnScoreWindowValue > 0 ||
      root.spiritSameTurnScoreSetupNow)
  );
}

export function productionRootAdvisorPresearch(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: AutomoveConfig,
  roots: RootCandidate[],
  plan: TurnPlan | undefined,
) {
  return runProductionRootAdvisorPresearch(
    execution,
    game,
    perspective,
    config,
    roots,
    plan,
    {
      buildInjectedRootCandidate: (
        candidateGame,
        candidatePerspective,
        _candidateConfig,
        inputs,
      ) =>
        buildRootCandidateForInputs(
          execution,
          candidateGame,
          candidatePerspective,
          config,
          inputs,
        ),
    },
  );
}

export function advisorConflictsWithChoice(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: AutomoveConfig,
  roots: readonly RootCandidate[],
  plan: TurnPlan | undefined,
  inputs: readonly Input[],
): boolean {
  const decision = productionRootAdvisorPresearch(
    execution,
    game,
    perspective,
    config,
    [...roots],
    plan,
  );
  const approved = decision?.approvedRoot?.inputs;
  return approved !== undefined && !inputChainsEqual(approved, inputs);
}

function seedTurnEngineFollowupCacheIfSafe(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: AutomoveConfig,
  mode: TurnEngineMode,
  plan: TurnPlan,
): void {
  if (!turnEngineModeUsesMacroPlans(mode) || plan.compiledChunks.length < 2) {
    return;
  }
  const first = plan.compiledChunks[0];
  const second = plan.compiledChunks[1];
  if (first === undefined || second === undefined) return;
  const afterFirst = applyInputsForSearch(game, first);
  if (afterFirst?.activeColor !== perspective) return;
  const afterConfig = turnEngineConfigForGame(afterFirst, config);
  const roots = rankRootCandidates(execution, afterFirst, perspective, config);
  if (!shouldResumeTurnEngineCachedStep(roots, second, mode)) return;
  turnEngineStoreCachedStep(execution, afterFirst, mode, afterConfig, second);
}

export function commitPlanAndSeedFollowup(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: AutomoveConfig,
  mode: TurnEngineMode,
  plan: TurnPlan,
  engineConfig: TurnEngineConfig,
): void {
  turnEngineCommitPlan(execution, game, perspective, mode, plan, engineConfig);
  seedTurnEngineFollowupCacheIfSafe(
    execution,
    game,
    perspective,
    config,
    mode,
    plan,
  );
}

export function turnEngineSelectedUtility(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  root: EvaluatedRoot,
  perspective: Color,
  config: AutomoveConfig,
  hooks?: ReplyRiskHooks,
) {
  return turnEngineSelectedOverrideUtility(
    execution,
    game,
    root,
    perspective,
    config,
    rootFamily(root),
    hooks,
  );
}

export function projectedPlanIsSafelyCompleted(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: AutomoveConfig,
  plan: TurnPlan,
): boolean {
  let projected = game.fork();
  const events = [];
  for (const chunk of plan.compiledChunks) {
    const applied = applyInputsForSearchWithEvents(projected, chunk);
    if (applied === undefined) return false;
    events.push(...applied.events);
    projected = applied.game;
  }
  const turnFinished =
    projected.winnerColor() !== undefined ||
    projected.activeColor !== perspective ||
    (!projected.playerCanMoveMon() &&
      !projected.playerCanUseAction() &&
      !projected.playerCanMoveMana());
  const nearCompletion =
    turnFinished ||
    plan.compiledChunks.length >= 4 ||
    !projected.playerCanMoveMon() ||
    (!projected.playerCanUseAction() && !projected.playerCanMoveMana());
  const handoff =
    manaHandoffPenalty(
      events,
      perspective,
      Math.max(config.evaluation.rootManaHandoffPenalty, 1),
    ) > 0;
  const vulnerable =
    projected.winnerColor() !== perspective &&
    ownDrainerVulnerableNextTurn(execution, projected, perspective);
  return (
    nearCompletion && !handoff && !hasRoundtripMonMove(events) && !vulnerable
  );
}
