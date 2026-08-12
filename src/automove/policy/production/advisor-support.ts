import { Color } from "../../../api/types.js";
import { inputChainsEqual, type Input } from "../../../engine/model/domain.js";
import type { MonsGame } from "../../../engine/game/mons-game.js";
import { productionRootAdvisorPresearch as runProductionRootAdvisorPresearch } from "../advisor/presearch.js";
import type { AutomoveExecutionContext } from "../../core/execution-context.js";
import { buildRootCandidateForInputs } from "../../root/candidates.js";
import type { RootCandidate } from "../../root/types.js";
import type { AutomoveConfig } from "../../config/types.js";
import type { TurnPlan } from "../../turn/model.js";

export function allowedRerankOverrideCandidate(
  roots: readonly RootCandidate[],
  inputs: readonly Input[],
): boolean {
  const root = roots.find((candidate) => inputChainsEqual(candidate.inputs, inputs));
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
