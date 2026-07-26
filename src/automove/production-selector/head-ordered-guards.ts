import type { AutomoveExecutionContext } from "../execution-context.js";
import {
  compareUtilityPrimaryAxes,
  utilityImprovesNonScoreOverrideAxes,
} from "../turn-engine.js";
import { acceptTurnEngineHeadByModeAndFamily } from "./head-family-policy.js";
import { evaluateInitialHeadGuards } from "./head-initial-guards.js";
import { manaAndRecoveryHeadGuardsReject } from "./head-mana-recovery-guards.js";
import { evaluateProjectedHeadGuards } from "./head-projected-guards.js";
import type {
  TurnEngineHeadAcceptanceContext,
  TurnEngineHeadInitialGuardFacts,
  TurnEngineHeadOrderedFacts,
} from "./head-types.js";

function deriveOrderedHeadFacts(
  context: TurnEngineHeadAcceptanceContext,
  initial: TurnEngineHeadInitialGuardFacts,
): TurnEngineHeadOrderedFacts {
  const { candidate, selected, plan, selectedUtilityValue } = context;
  const nearTieProgress =
    candidate.safeSupermanaProgressSteps ===
      selected.safeSupermanaProgressSteps &&
    candidate.safeOpponentManaProgressSteps ===
      selected.safeOpponentManaProgressSteps &&
    candidate.ownDrainerVulnerable === selected.ownDrainerVulnerable &&
    candidate.efficiency === selected.efficiency &&
    candidate.supermanaProgress === selected.supermanaProgress &&
    candidate.opponentManaProgress === selected.opponentManaProgress;
  const primaryAxesOrder = compareUtilityPrimaryAxes(
    plan.utility,
    selectedUtilityValue(),
  );
  const strategicAxesBetter = utilityImprovesNonScoreOverrideAxes(
    plan.utility,
    selectedUtilityValue(),
  );
  const selectedUtility = selectedUtilityValue();
  return {
    ...initial,
    nearTieProgress,
    primaryAxesOrder,
    strategicAxesBetter,
    selectedUtility,
  };
}

export function acceptTurnEngineHeadAfterOrderedGuards(
  execution: AutomoveExecutionContext,
  context: TurnEngineHeadAcceptanceContext,
): boolean {
  const initial = evaluateInitialHeadGuards(context);
  if (initial.kind === "reject") return false;

  const facts = deriveOrderedHeadFacts(context, initial.facts);
  if (manaAndRecoveryHeadGuardsReject(context, facts)) return false;

  const decision = evaluateProjectedHeadGuards(execution, context, facts);
  if (decision.kind === "accept") return true;
  if (decision.kind === "reject") return false;
  return acceptTurnEngineHeadByModeAndFamily(context, decision.policy);
}
