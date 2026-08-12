import type { AutomoveExecutionContext } from "../core/execution-context.js";
import type { Color } from "../../api/types.js";
import { MonsGame } from "../../engine/game/mons-game.js";
import { BOARD_SIZE } from "../../engine/board/geometry.js";
import { exactOpportunityContextWithSearchHash } from "../exact/turn-opportunity.js";
import { exactSearchStateHash } from "../exact/hash.js";
import type {
  ExactOpportunityBudget,
  ExactOpportunityContext,
} from "../exact/types.js";
import { actionIdentity } from "./action-rules.js";
import {
  OpportunityKind,
  TURN_PLAN_FAMILY_PRIORITY_ORDER,
  TurnEngineMode,
  TurnPlanFamily,
  type ActionSeed,
  type OpportunityBudget,
  type OpportunityDelta,
  type TurnAction,
  type TurnEngineConfig,
  type TurnOpportunity,
} from "./model.js";
import { compareActionKeys, compareNumber } from "./ordering.js";
import { familyAllowed } from "./opportunity-policy.js";

import {
  denyWindowSeeds,
  drainerKillSeeds,
  immediateScoreSeeds,
} from "./opportunity-seeds-tactical.js";
import {
  fallbackWalkSeeds,
  riskyRecoverySetupSeeds,
  safeOpponentManaProgressSeeds,
  safeSupermanaProgressSeeds,
  safetyRecoverySeeds,
} from "./opportunity-seeds-progress.js";
import { oracleWalkSeeds } from "./opportunity-seeds-oracle.js";
import { manaTempoSeeds, spiritImpactSeeds } from "./opportunity-seeds-spirit.js";

function opportunityKindForFamily(family: TurnPlanFamily): OpportunityKind {
  switch (family) {
    case TurnPlanFamily.ImmediateScore:
      return OpportunityKind.ImmediateScore;
    case TurnPlanFamily.DenyOpponentWindow:
      return OpportunityKind.TacticalDeny;
    case TurnPlanFamily.DrainerKill:
      return OpportunityKind.DrainerKill;
    case TurnPlanFamily.SafeSupermanaProgress:
      return OpportunityKind.SafeSupermanaProgress;
    case TurnPlanFamily.SafeOpponentManaProgress:
      return OpportunityKind.SafeOpponentManaProgress;
    case TurnPlanFamily.DrainerSafetyRecovery:
      return OpportunityKind.DrainerSafetyRecovery;
    case TurnPlanFamily.SpiritImpact:
      return OpportunityKind.SpiritImpact;
    case TurnPlanFamily.ManaTempo:
      return OpportunityKind.ManaTempo;
  }
}

function opportunityBudgetForAction(action: TurnAction): OpportunityBudget {
  switch (action.kind) {
    case "walk":
    case "safety-retreat":
    case "score-carry":
      return { monMovesNeeded: 1, needsAction: false, needsManaMove: false };
    case "attack":
    case "bomb":
    case "spirit-shift":
      return { monMovesNeeded: 0, needsAction: true, needsManaMove: false };
    case "move-mana":
      return { monMovesNeeded: 0, needsAction: false, needsManaMove: true };
  }
}

function budgetAllowsOpportunity(
  available: ExactOpportunityBudget,
  required: OpportunityBudget,
): boolean {
  return (
    required.monMovesNeeded <= available.remainingMonMoves &&
    (!required.needsAction || available.canUseAction) &&
    (!required.needsManaMove || available.canMoveMana)
  );
}

function opportunityDeltaForSeed(
  seed: ActionSeed,
  context: ExactOpportunityContext,
): OpportunityDelta {
  const unknown = BOARD_SIZE * 3;
  const supermanaProgressGain =
    seed.family === TurnPlanFamily.SafeSupermanaProgress
      ? Math.max(unknown - (context.delta.safeSupermanaProgressSteps ?? unknown), 0)
      : 0;
  const opponentManaProgressGain =
    seed.family === TurnPlanFamily.SafeOpponentManaProgress
      ? Math.max(unknown - (context.delta.safeOpponentManaProgressSteps ?? unknown), 0)
      : 0;
  return {
    sameTurnScoreWindowGain: context.delta.sameTurnScoreWindowValue,
    spiritGain:
      seed.family === TurnPlanFamily.SpiritImpact
        ? Math.max(context.delta.spiritGain, 1)
        : 0,
    opponentWindowDenyGain:
      seed.family === TurnPlanFamily.DenyOpponentWindow ||
      seed.family === TurnPlanFamily.DrainerKill
        ? Math.max(context.delta.opponentWindowDenyGain, 1)
        : 0,
    drainerAttack:
      (seed.family === TurnPlanFamily.DrainerKill ||
        seed.family === TurnPlanFamily.DenyOpponentWindow) &&
      context.delta.drainerAttackAvailable,
    drainerSafetyDelta:
      seed.family === TurnPlanFamily.DrainerSafetyRecovery
        ? Math.max(-context.delta.drainerSafety, 0)
        : 0,
    supermanaProgressGain,
    opponentManaProgressGain,
  };
}

function turnOpportunityFromSeed(
  seed: ActionSeed,
  context: ExactOpportunityContext,
): TurnOpportunity {
  return {
    kind: opportunityKindForFamily(seed.family),
    family: seed.family,
    action: seed.action,
    priority: seed.priority,
    budget: opportunityBudgetForAction(seed.action),
    delta: opportunityDeltaForSeed(seed, context),
  };
}

export function opportunityScore(
  opportunity: TurnOpportunity,
  emergency: boolean,
): number {
  const kindBonus = (() => {
    switch (opportunity.kind) {
      case OpportunityKind.ImmediateScore:
        return 12_000;
      case OpportunityKind.TacticalDeny:
        return 11_400;
      case OpportunityKind.DrainerKill:
        return 11_200;
      case OpportunityKind.DrainerSafetyRecovery:
        return 10_400;
      case OpportunityKind.SpiritImpact:
        return 9_800;
      case OpportunityKind.SafeSupermanaProgress:
        return 9_400;
      case OpportunityKind.SafeOpponentManaProgress:
        return 9_200;
      case OpportunityKind.ManaTempo:
        return 8_000;
    }
  })();
  const urgentKind =
    opportunity.kind === OpportunityKind.ImmediateScore ||
    opportunity.kind === OpportunityKind.TacticalDeny ||
    opportunity.kind === OpportunityKind.DrainerKill ||
    opportunity.kind === OpportunityKind.DrainerSafetyRecovery;
  return (
    opportunity.priority +
    kindBonus +
    (emergency && urgentKind ? 4_000 : 0) +
    opportunity.delta.sameTurnScoreWindowGain * 280 +
    opportunity.delta.spiritGain * 220 +
    opportunity.delta.opponentWindowDenyGain * 260 +
    opportunity.delta.drainerSafetyDelta * 240 +
    opportunity.delta.supermanaProgressGain * 40 +
    opportunity.delta.opponentManaProgressGain * 36 +
    (opportunity.delta.drainerAttack ? 800 : 0) -
    Math.max(opportunity.budget.monMovesNeeded, 0) * 120 -
    (opportunity.budget.needsAction ? 80 : 0) -
    (opportunity.budget.needsManaMove ? 40 : 0)
  );
}

export function discoverTurnOpportunities(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: TurnEngineConfig,
  opportunityCap: number,
  allowedFamilies?: readonly TurnPlanFamily[],
): TurnOpportunity[] {
  if (execution.session.checkpoint() || game.activeColor !== perspective) return [];
  const context = exactOpportunityContextWithSearchHash(
    execution,
    game,
    perspective,
    exactSearchStateHash(game),
  );
  if (execution.session.checkpoint()) return [];
  const emergency =
    context.opponentCanWinImmediately || context.delta.drainerSafety < 0;
  const seeds: ActionSeed[] = [];
  if (familyAllowed(allowedFamilies, TurnPlanFamily.ImmediateScore)) {
    seeds.push(...immediateScoreSeeds(game, perspective));
  }
  if (execution.session.checkpoint()) return [];
  if (familyAllowed(allowedFamilies, TurnPlanFamily.DenyOpponentWindow)) {
    seeds.push(...denyWindowSeeds(execution, game, perspective));
  }
  if (execution.session.checkpoint()) return [];
  if (familyAllowed(allowedFamilies, TurnPlanFamily.DrainerKill)) {
    seeds.push(...drainerKillSeeds(execution, game, perspective));
  }
  if (execution.session.checkpoint()) return [];
  if (familyAllowed(allowedFamilies, TurnPlanFamily.SafeSupermanaProgress)) {
    seeds.push(...safeSupermanaProgressSeeds(execution, game, perspective));
  }
  if (execution.session.checkpoint()) return [];
  if (familyAllowed(allowedFamilies, TurnPlanFamily.SafeOpponentManaProgress)) {
    seeds.push(...safeOpponentManaProgressSeeds(execution, game, perspective));
  }
  if (execution.session.checkpoint()) return [];
  if (familyAllowed(allowedFamilies, TurnPlanFamily.DrainerSafetyRecovery)) {
    seeds.push(...safetyRecoverySeeds(execution, game, perspective));
  }
  if (execution.session.checkpoint()) return [];
  if (familyAllowed(allowedFamilies, TurnPlanFamily.ManaTempo)) {
    seeds.push(...riskyRecoverySetupSeeds(execution, game, perspective, config));
  }
  if (execution.session.checkpoint()) return [];
  if (
    familyAllowed(allowedFamilies, TurnPlanFamily.SafeSupermanaProgress) ||
    familyAllowed(allowedFamilies, TurnPlanFamily.SafeOpponentManaProgress) ||
    familyAllowed(allowedFamilies, TurnPlanFamily.DrainerSafetyRecovery) ||
    familyAllowed(allowedFamilies, TurnPlanFamily.SpiritImpact)
  ) {
    seeds.push(
      ...oracleWalkSeeds(
        execution,
        game,
        perspective,
        context,
        allowedFamilies,
        config,
      ),
    );
  }
  if (execution.session.checkpoint()) return [];
  if (familyAllowed(allowedFamilies, TurnPlanFamily.SpiritImpact)) {
    seeds.push(...spiritImpactSeeds(execution, game, perspective, config));
  }
  if (execution.session.checkpoint()) return [];
  if (familyAllowed(allowedFamilies, TurnPlanFamily.ManaTempo)) {
    seeds.push(...manaTempoSeeds(game, perspective));
  }
  if (execution.session.checkpoint()) return [];
  if (
    familyAllowed(allowedFamilies, TurnPlanFamily.ManaTempo) ||
    familyAllowed(allowedFamilies, TurnPlanFamily.DrainerSafetyRecovery)
  ) {
    seeds.push(
      ...fallbackWalkSeeds(execution, game, perspective).filter((seed) =>
        familyAllowed(allowedFamilies, seed.family),
      ),
    );
  }
  if (execution.session.checkpoint()) return [];

  const perFamily = new Map<TurnPlanFamily, TurnOpportunity[]>();
  for (const seed of seeds) {
    if (execution.session.checkpoint()) return [];
    const opportunity = turnOpportunityFromSeed(seed, context);
    if (!budgetAllowsOpportunity(context.budget, opportunity.budget)) continue;
    if (
      emergency &&
      opportunity.kind === OpportunityKind.ManaTempo &&
      !opportunity.delta.drainerAttack &&
      opportunity.delta.drainerSafetyDelta <= 0
    ) {
      continue;
    }
    const list = perFamily.get(opportunity.family) ?? [];
    list.push(opportunity);
    perFamily.set(opportunity.family, list);
  }
  for (const opportunities of perFamily.values()) {
    opportunities.sort((left, right) => {
      const scoreOrder = compareNumber(
        opportunityScore(right, emergency),
        opportunityScore(left, emergency),
      );
      return scoreOrder !== 0
        ? scoreOrder
        : compareActionKeys(left.action, right.action);
    });
  }

  const indices = new Map<TurnPlanFamily, number>();
  const seen = new Set<string>();
  const result: TurnOpportunity[] = [];
  for (let round = 0; round < Math.max(config.perNodeFamilyCap, 1); round += 1) {
    if (execution.session.checkpoint()) return [];
    let addedAny = false;
    for (const family of TURN_PLAN_FAMILY_PRIORITY_ORDER) {
      const familyOpportunities = perFamily.get(family);
      if (familyOpportunities === undefined) continue;
      let index = indices.get(family) ?? 0;
      while (index < familyOpportunities.length) {
        const candidate = familyOpportunities[index];
        index += 1;
        if (candidate !== undefined && !seen.has(actionIdentity(candidate.action))) {
          seen.add(actionIdentity(candidate.action));
          result.push(candidate);
          addedAny = true;
          break;
        }
      }
      indices.set(family, index);
      if (result.length >= Math.max(opportunityCap, 1)) return result;
    }
    if (!addedAny) break;
  }
  return result;
}

export function generateActionSeeds(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: TurnEngineConfig,
  seedCap: number,
): ActionSeed[] {
  if (execution.session.checkpoint() || game.activeColor !== perspective) return [];
  if (config.mode === TurnEngineMode.Production) {
    const seeds = discoverTurnOpportunities(
      execution,
      game,
      perspective,
      config,
      seedCap,
    ).map(({ family, action, priority }) => ({ family, action, priority }));
    return execution.session.checkpoint() ? [] : seeds;
  }

  const seeds: ActionSeed[] = [];
  seeds.push(...immediateScoreSeeds(game, perspective));
  if (execution.session.checkpoint()) return [];
  seeds.push(...denyWindowSeeds(execution, game, perspective));
  if (execution.session.checkpoint()) return [];
  seeds.push(...drainerKillSeeds(execution, game, perspective));
  if (execution.session.checkpoint()) return [];
  seeds.push(...safeSupermanaProgressSeeds(execution, game, perspective));
  if (execution.session.checkpoint()) return [];
  seeds.push(...safeOpponentManaProgressSeeds(execution, game, perspective));
  if (execution.session.checkpoint()) return [];
  seeds.push(...safetyRecoverySeeds(execution, game, perspective));
  if (execution.session.checkpoint()) return [];
  seeds.push(
    ...oracleWalkSeeds(
      execution,
      game,
      perspective,
      exactOpportunityContextWithSearchHash(
        execution,
        game,
        perspective,
        exactSearchStateHash(game),
      ),
      undefined,
      config,
    ),
  );
  if (execution.session.checkpoint()) return [];
  seeds.push(...spiritImpactSeeds(execution, game, perspective, config));
  if (execution.session.checkpoint()) return [];
  seeds.push(...manaTempoSeeds(game, perspective));
  if (execution.session.checkpoint()) return [];
  const perFamily = new Map<TurnPlanFamily, ActionSeed[]>();
  for (const seed of seeds) {
    if (execution.session.checkpoint()) return [];
    const list = perFamily.get(seed.family) ?? [];
    list.push(seed);
    perFamily.set(seed.family, list);
  }
  for (const familySeeds of perFamily.values()) {
    familySeeds.sort((left, right) => {
      const order = compareNumber(right.priority, left.priority);
      return order !== 0 ? order : compareActionKeys(left.action, right.action);
    });
  }
  const seen = new Set<string>();
  const indices = new Map<TurnPlanFamily, number>();
  const result: ActionSeed[] = [];
  for (let round = 0; round < Math.max(config.perNodeFamilyCap, 1); round += 1) {
    if (execution.session.checkpoint()) return [];
    let addedAny = false;
    for (const family of TURN_PLAN_FAMILY_PRIORITY_ORDER) {
      const list = perFamily.get(family);
      if (list === undefined) continue;
      let index = indices.get(family) ?? 0;
      while (index < list.length) {
        const seed = list[index];
        index += 1;
        if (seed !== undefined && !seen.has(actionIdentity(seed.action))) {
          seen.add(actionIdentity(seed.action));
          result.push(seed);
          addedAny = true;
          break;
        }
      }
      indices.set(family, index);
      if (result.length >= Math.max(seedCap, 1)) return result;
    }
    if (!addedAny) break;
  }
  return result;
}
