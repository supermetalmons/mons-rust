import type { AutomoveExecutionContext } from "./execution-context.js";
import {
  Color,
  MonKind,
  isMonFainted,
  isSpiritTargetAllowed,
  itemMana,
  itemMon,
  manaEquals,
  manaScore,
  otherColor,
  type Mana,
} from "../engine/domain.js";
import { MonsGame } from "../engine/game.js";
import {
  BOARD_SIZE,
  locationDistance,
  locationEquals,
  nearbyLocations,
  spiritReachableLocations,
} from "../engine/geometry.js";
import {
  EXACT_TURN_TACTICAL_NEED_OPPONENT_MANA_PROGRESS,
  EXACT_TURN_TACTICAL_NEED_SCORE_WINDOW,
  EXACT_TURN_TACTICAL_NEED_SPIRIT_DENIAL,
  EXACT_TURN_TACTICAL_NEED_SPIRIT_SCORE,
  EXACT_TURN_TACTICAL_NEED_SUPERMANA_PROGRESS,
  exactBestScoreStepsOnBoard,
  exactOpportunityContextWithSearchHash,
  exactSearchStateHash,
  exactSecureSpecificManaPathFrom,
  exactStrategicAnalysisWithSearchHash,
  exactTurnTacticalProjectionWithSearchHash,
  type ExactOpportunityBudget,
  type ExactOpportunityContext,
  type ExactTurnTacticalProjection,
} from "./exact.js";
import type { Hash64 } from "./hash64.js";
import { applyInputsForSearchWithEvents } from "./transitions.js";
import {
  actionIdentity,
  actorCanAttackFromItem,
  actorCanAttackTargetNow,
  actorCanBombFromItem,
  actorCanBombTargetNow,
  distanceToNearestPool,
  manaMoveDestinationAllowed,
  nearestWantedManaLocation,
  opponentDrainerKillIsHighValue,
  remainingMovesForColor,
  spiritDestinationAllowed,
  walkDestinationPlausible,
} from "./turn-compiler.js";
import {
  activeTurnScoreWindowWithSearchHash,
  findAwakeDrainerLocation,
  opponentCanWinImmediately,
  ownDrainerSafetyScore,
} from "./turn-evaluation.js";
import {
  OpportunityKind,
  TURN_PLAN_FAMILY_PRIORITY_ORDER,
  TurnEngineMode,
  TurnPlanFamily,
  compareActionKeys,
  compareNumber,
  type ActionSeed,
  type OpportunityBudget,
  type OpportunityDelta,
  type TurnAction,
  type TurnEngineConfig,
  type TurnOpportunity,
} from "./turn-types.js";

export function opportunityKindForFamily(
  family: TurnPlanFamily,
): OpportunityKind {
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

export function opportunityBudgetForAction(
  action: TurnAction,
): OpportunityBudget {
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

export function budgetAllowsOpportunity(
  available: ExactOpportunityBudget,
  required: OpportunityBudget,
): boolean {
  return (
    required.monMovesNeeded <= available.remainingMonMoves &&
    (!required.needsAction || available.canUseAction) &&
    (!required.needsManaMove || available.canMoveMana)
  );
}

export function opportunityDeltaForSeed(
  seed: ActionSeed,
  context: ExactOpportunityContext,
): OpportunityDelta {
  const unknown = BOARD_SIZE * 3;
  const supermanaProgressGain =
    seed.family === TurnPlanFamily.SafeSupermanaProgress
      ? Math.max(
          unknown - (context.delta.safeSupermanaProgressSteps ?? unknown),
          0,
        )
      : 0;
  const opponentManaProgressGain =
    seed.family === TurnPlanFamily.SafeOpponentManaProgress
      ? Math.max(
          unknown - (context.delta.safeOpponentManaProgressSteps ?? unknown),
          0,
        )
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

export function turnOpportunityFromSeed(
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

export function familyAllowed(
  allowedFamilies: readonly TurnPlanFamily[] | undefined,
  family: TurnPlanFamily,
): boolean {
  return allowedFamilies === undefined || allowedFamilies.includes(family);
}

export function discoverTurnOpportunities(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: TurnEngineConfig,
  opportunityCap: number,
  allowedFamilies?: readonly TurnPlanFamily[],
): TurnOpportunity[] {
  if (execution.session.checkpoint() || game.activeColor !== perspective)
    return [];
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
    seeds.push(
      ...riskyRecoverySetupSeeds(execution, game, perspective, config),
    );
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
  for (
    let round = 0;
    round < Math.max(config.perNodeFamilyCap, 1);
    round += 1
  ) {
    if (execution.session.checkpoint()) return [];
    let addedAny = false;
    for (const family of TURN_PLAN_FAMILY_PRIORITY_ORDER) {
      const familyOpportunities = perFamily.get(family);
      if (familyOpportunities === undefined) continue;
      let index = indices.get(family) ?? 0;
      while (index < familyOpportunities.length) {
        const candidate = familyOpportunities[index];
        index += 1;
        if (
          candidate !== undefined &&
          !seen.has(actionIdentity(candidate.action))
        ) {
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
  if (execution.session.checkpoint() || game.activeColor !== perspective)
    return [];
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
  for (
    let round = 0;
    round < Math.max(config.perNodeFamilyCap, 1);
    round += 1
  ) {
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

export function immediateScoreSeeds(
  game: MonsGame,
  perspective: Color,
): ActionSeed[] {
  const result: ActionSeed[] = [];
  for (const [at, item] of game.board.entries()) {
    if (
      item.kind !== "mon-with-mana" ||
      item.mon.color !== perspective ||
      isMonFainted(item.mon)
    ) {
      continue;
    }
    const beforeDistance = distanceToNearestPool(at, perspective);
    for (const next of nearbyLocations(at)) {
      const afterDistance = distanceToNearestPool(next, perspective);
      if (afterDistance > beforeDistance) continue;
      result.push({
        family: TurnPlanFamily.ImmediateScore,
        action: {
          kind: "score-carry",
          actor: at,
          wanted: item.mana,
          step: next,
        },
        priority:
          9_800 +
          Math.max(beforeDistance - afterDistance, 0) * 180 +
          manaScore(item.mana, perspective) * 120,
      });
    }
  }
  return result;
}

export function denyWindowSeeds(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
): ActionSeed[] {
  const opponent = otherColor(perspective);
  const pressure = activeTurnScoreWindowWithSearchHash(
    execution,
    game,
    opponent,
    exactSearchStateHash(game),
  );
  if (pressure <= 0 && !opponentCanWinImmediately(execution, game, perspective))
    return [];
  const result = attackFamilySeeds(
    game,
    perspective,
    TurnPlanFamily.DenyOpponentWindow,
    9_400 + pressure * 240,
  );
  const drainer = findAwakeDrainerLocation(game.board, perspective);
  if (drainer === undefined) return result;
  const beforeSafety = ownDrainerSafetyScore(
    execution,
    game.board,
    perspective,
  );
  const beforeDistance = distanceToNearestPool(drainer, perspective);
  for (const next of nearbyLocations(drainer)) {
    if (
      distanceToNearestPool(next, perspective) > beforeDistance + 1 &&
      beforeSafety >= 0
    ) {
      continue;
    }
    result.push({
      family: TurnPlanFamily.DenyOpponentWindow,
      action: { kind: "safety-retreat", actor: drainer, to: next },
      priority: 9_100 + Math.abs(beforeSafety) * 220,
    });
  }
  return result;
}

export function drainerKillSeeds(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
): ActionSeed[] {
  const target = findAwakeDrainerLocation(game.board, otherColor(perspective));
  return target === undefined ||
    !opponentDrainerKillIsHighValue(execution, game, perspective, target)
    ? []
    : attackFamilySeeds(game, perspective, TurnPlanFamily.DrainerKill, 9_000);
}

export function attackFamilySeeds(
  game: MonsGame,
  perspective: Color,
  family: TurnPlanFamily,
  basePriority: number,
): ActionSeed[] {
  const target = findAwakeDrainerLocation(game.board, otherColor(perspective));
  if (target === undefined) return [];
  const result: ActionSeed[] = [];
  const canUseAction = game.playerCanUseAction();
  const remainingMoves = remainingMovesForColor(game, perspective);
  for (const [at, item] of game.board.entries()) {
    const mon = itemMon(item);
    if (mon?.color !== perspective || isMonFainted(mon)) continue;
    const canAttack = canUseAction && actorCanAttackFromItem(item);
    const canBomb = canUseAction && actorCanBombFromItem(item);
    if (
      canAttack &&
      actorCanAttackTargetNow(game.board, at, target, item, perspective)
    ) {
      result.push({
        family,
        action: { kind: "attack", actor: at, target },
        priority: basePriority,
      });
    }
    if (
      canBomb &&
      actorCanBombTargetNow(game.board, at, target, item, perspective)
    ) {
      result.push({
        family,
        action: { kind: "bomb", actor: at, target },
        priority: basePriority - 80,
      });
    }
    if (remainingMoves <= 0 || (!canAttack && !canBomb)) continue;
    for (const next of nearbyLocations(at)) {
      if (locationDistance(next, target) >= locationDistance(at, target))
        continue;
      if (family === TurnPlanFamily.DrainerKill) {
        const preview = game.board.fork();
        preview.delete(at);
        preview.set(next, item);
        const threatensNow =
          (canAttack &&
            actorCanAttackTargetNow(
              preview,
              next,
              target,
              item,
              perspective,
            )) ||
          (canBomb &&
            actorCanBombTargetNow(preview, next, target, item, perspective));
        if (!threatensNow) continue;
      }
      result.push({
        family,
        action: { kind: "walk", actor: at, to: next },
        priority:
          basePriority -
          200 +
          (locationDistance(at, target) - locationDistance(next, target)) * 80,
      });
    }
  }
  return result;
}

export function safeSupermanaProgressSeeds(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
): ActionSeed[] {
  return safeProgressSeeds(
    execution,
    game,
    perspective,
    { kind: "supermana" },
    TurnPlanFamily.SafeSupermanaProgress,
    8_900,
  );
}

export function safeOpponentManaProgressSeeds(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
): ActionSeed[] {
  return safeProgressSeeds(
    execution,
    game,
    perspective,
    { kind: "regular", color: otherColor(perspective) },
    TurnPlanFamily.SafeOpponentManaProgress,
    8_600,
  );
}

export type SafeProgressExactSnapshot = {
  readonly progressSteps: number | undefined;
  readonly scorePathBestSteps: number | undefined;
  readonly sameTurnScoreWindowValue: number;
};

export function safeProgressExactSnapshot(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  wanted: Mana,
  stateHash: Hash64,
): SafeProgressExactSnapshot {
  if (execution.session.checkpoint()) {
    return {
      progressSteps: undefined,
      scorePathBestSteps: undefined,
      sameTurnScoreWindowValue: 0,
    };
  }
  const opponent = otherColor(perspective);
  const flags =
    wanted.kind === "supermana"
      ? EXACT_TURN_TACTICAL_NEED_SUPERMANA_PROGRESS |
        EXACT_TURN_TACTICAL_NEED_SCORE_WINDOW
      : wanted.color === opponent
        ? EXACT_TURN_TACTICAL_NEED_OPPONENT_MANA_PROGRESS
        : 0;
  const projection = exactTurnTacticalProjectionWithSearchHash(
    execution,
    game,
    perspective,
    stateHash,
    flags,
  );
  if (execution.session.checkpoint()) {
    return {
      progressSteps: undefined,
      scorePathBestSteps: undefined,
      sameTurnScoreWindowValue: 0,
    };
  }
  return {
    progressSteps:
      wanted.kind === "supermana"
        ? projection.safeSupermanaProgressSteps
        : wanted.color === opponent
          ? projection.safeOpponentManaProgressSteps
          : undefined,
    scorePathBestSteps: exactBestScoreStepsOnBoard(
      execution,
      game.board,
      perspective,
    ),
    sameTurnScoreWindowValue: projection.sameTurnScoreWindowValue,
  };
}

export function safeProgressSeeds(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  wanted: Mana,
  family: TurnPlanFamily,
  basePriority: number,
): ActionSeed[] {
  if (execution.session.checkpoint()) return [];
  const drainer = findAwakeDrainerLocation(game.board, perspective);
  if (drainer === undefined) return [];
  const result: ActionSeed[] = [];
  const beforeExact = safeProgressExactSnapshot(
    execution,
    game,
    perspective,
    wanted,
    exactSearchStateHash(game),
  );
  if (execution.session.cancelled) return [];
  const beforeSafety = ownDrainerSafetyScore(
    execution,
    game.board,
    perspective,
  );
  const path = exactSecureSpecificManaPathFrom(
    execution,
    game,
    perspective,
    drainer,
    wanted,
  );
  const pathStep = path?.[0];
  if (pathStep !== undefined) {
    result.push({
      family,
      action: { kind: "score-carry", actor: drainer, wanted, step: pathStep },
      priority:
        basePriority + Math.max(BOARD_SIZE * 2 - (path?.length ?? 0), 0) * 120,
    });
  }
  if (execution.session.checkpoint()) return [];
  if (remainingMovesForColor(game, perspective) > 0) {
    const target = nearestWantedManaLocation(game.board, wanted);
    if (target !== undefined) {
      const beforeDistance = locationDistance(drainer, target);
      const beforeSteps = beforeExact.progressSteps ?? BOARD_SIZE * 3;
      const beforeScorePath = beforeExact.scorePathBestSteps ?? BOARD_SIZE * 3;
      for (const next of nearbyLocations(drainer)) {
        if (execution.session.checkpoint()) return [];
        if (!walkDestinationPlausible(game.board, drainer, next)) continue;
        const applied = applyInputsForSearchWithEvents(game, [
          { kind: "location", location: drainer },
          { kind: "location", location: next },
        ]);
        if (
          applied === undefined ||
          opponentCanWinImmediately(execution, applied.game, perspective)
        ) {
          continue;
        }
        const afterExact = safeProgressExactSnapshot(
          execution,
          applied.game,
          perspective,
          wanted,
          exactSearchStateHash(applied.game),
        );
        const sessionAfterExactSnapshot = execution.session;
        if (sessionAfterExactSnapshot.cancelled) return [];
        const afterSafety = ownDrainerSafetyScore(
          execution,
          applied.game.board,
          perspective,
        );
        const afterSteps = afterExact.progressSteps ?? BOARD_SIZE * 3;
        const afterScorePath = afterExact.scorePathBestSteps ?? BOARD_SIZE * 3;
        const exactImproved =
          afterSteps < beforeSteps ||
          (afterSteps <= beforeSteps && afterScorePath < beforeScorePath);
        if (!exactImproved && afterSafety < beforeSafety) continue;
        let priority =
          basePriority -
          180 +
          Math.max(beforeDistance - locationDistance(next, target), 0) * 110 +
          (afterSafety - beforeSafety) * 120;
        if (exactImproved) {
          priority +=
            (beforeSteps - afterSteps) * 220 +
            (beforeScorePath - afterScorePath) * 180;
        }
        if (
          wanted.kind === "supermana" &&
          afterExact.sameTurnScoreWindowValue > 0
        ) {
          priority += afterExact.sameTurnScoreWindowValue * 260;
        }
        result.push({
          family,
          action: { kind: "walk", actor: drainer, to: next },
          priority,
        });
      }
    }
  }
  const drainerItem = game.board.get(drainer);
  if (
    drainerItem?.kind === "mon-with-mana" &&
    manaEquals(drainerItem.mana, wanted)
  ) {
    const beforeDistance = distanceToNearestPool(drainer, perspective);
    for (const next of nearbyLocations(drainer)) {
      const afterDistance = distanceToNearestPool(next, perspective);
      if (afterDistance > beforeDistance) continue;
      result.push({
        family,
        action: { kind: "score-carry", actor: drainer, wanted, step: next },
        priority:
          basePriority + Math.max(beforeDistance - afterDistance, 0) * 150,
      });
    }
  }
  return result;
}

export function safetyRecoverySeeds(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
): ActionSeed[] {
  const drainer = findAwakeDrainerLocation(game.board, perspective);
  if (drainer === undefined) return [];
  const beforeSafety = ownDrainerSafetyScore(
    execution,
    game.board,
    perspective,
  );
  const result: ActionSeed[] = [];
  for (const next of nearbyLocations(drainer)) {
    const applied = applyInputsForSearchWithEvents(game, [
      { kind: "location", location: drainer },
      { kind: "location", location: next },
    ]);
    if (applied === undefined) continue;
    const afterSafety = ownDrainerSafetyScore(
      execution,
      applied.game.board,
      perspective,
    );
    if (afterSafety <= beforeSafety) continue;
    result.push({
      family: TurnPlanFamily.DrainerSafetyRecovery,
      action: { kind: "safety-retreat", actor: drainer, to: next },
      priority:
        8_300 +
        Math.abs(beforeSafety) * 220 +
        (afterSafety - beforeSafety) * 260,
    });
  }
  return result;
}

export function fallbackWalkSeeds(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
): ActionSeed[] {
  if (remainingMovesForColor(game, perspective) <= 0) return [];
  const result: ActionSeed[] = [];
  const beforeSafety = ownDrainerSafetyScore(
    execution,
    game.board,
    perspective,
  );
  const drainer = findAwakeDrainerLocation(game.board, perspective);
  if (drainer !== undefined) {
    const beforePoolDistance = distanceToNearestPool(drainer, perspective);
    for (const next of nearbyLocations(drainer)) {
      if (!walkDestinationPlausible(game.board, drainer, next)) continue;
      const applied = applyInputsForSearchWithEvents(game, [
        { kind: "location", location: drainer },
        { kind: "location", location: next },
      ]);
      if (
        applied === undefined ||
        opponentCanWinImmediately(execution, applied.game, perspective)
      )
        continue;
      const afterSafety = ownDrainerSafetyScore(
        execution,
        applied.game.board,
        perspective,
      );
      if (afterSafety < beforeSafety) continue;
      const afterPoolDistance = distanceToNearestPool(next, perspective);
      result.push({
        family:
          afterSafety > beforeSafety
            ? TurnPlanFamily.DrainerSafetyRecovery
            : TurnPlanFamily.ManaTempo,
        action: { kind: "walk", actor: drainer, to: next },
        priority:
          7_200 +
          Math.max(beforePoolDistance - afterPoolDistance, 0) * 140 +
          (afterSafety - beforeSafety) * 240,
      });
    }
  }
  if (result.length !== 0) return result;
  for (const [actor, item] of game.board.entries()) {
    const mon = itemMon(item);
    if (mon?.color !== perspective || isMonFainted(mon)) continue;
    for (const to of nearbyLocations(actor)) {
      if (!walkDestinationPlausible(game.board, actor, to)) continue;
      const applied = applyInputsForSearchWithEvents(game, [
        { kind: "location", location: actor },
        { kind: "location", location: to },
      ]);
      if (
        applied === undefined ||
        opponentCanWinImmediately(execution, applied.game, perspective)
      )
        continue;
      result.push({
        family: TurnPlanFamily.ManaTempo,
        action: { kind: "walk", actor, to },
        priority: 6_800,
      });
    }
  }
  return result;
}

export function bestFollowUpSafetyRecoveryPriority(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
): number | undefined {
  const drainer = findAwakeDrainerLocation(game.board, perspective);
  if (drainer === undefined) return undefined;
  const beforeSafety = ownDrainerSafetyScore(
    execution,
    game.board,
    perspective,
  );
  let best: number | undefined;
  for (const next of nearbyLocations(drainer)) {
    if (!walkDestinationPlausible(game.board, drainer, next)) continue;
    const applied = applyInputsForSearchWithEvents(game, [
      { kind: "location", location: drainer },
      { kind: "location", location: next },
    ]);
    if (applied === undefined) continue;
    const afterSafety = ownDrainerSafetyScore(
      execution,
      applied.game.board,
      perspective,
    );
    if (afterSafety <= beforeSafety) continue;
    const priority =
      8_300 + Math.abs(beforeSafety) * 220 + (afterSafety - beforeSafety) * 260;
    best = best === undefined ? priority : Math.max(best, priority);
  }
  return best;
}

export function riskyRecoverySetupSeeds(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: TurnEngineConfig,
): ActionSeed[] {
  if (
    config.mode !== TurnEngineMode.Production ||
    remainingMovesForColor(game, perspective) <= 0
  ) {
    return [];
  }
  const drainer = findAwakeDrainerLocation(game.board, perspective);
  if (drainer === undefined) return [];
  const beforeSafety = ownDrainerSafetyScore(
    execution,
    game.board,
    perspective,
  );
  const beforePoolDistance = distanceToNearestPool(drainer, perspective);
  const result: ActionSeed[] = [];
  for (const next of nearbyLocations(drainer)) {
    if (!walkDestinationPlausible(game.board, drainer, next)) continue;
    const applied = applyInputsForSearchWithEvents(game, [
      { kind: "location", location: drainer },
      { kind: "location", location: next },
    ]);
    if (
      applied === undefined ||
      opponentCanWinImmediately(execution, applied.game, perspective)
    )
      continue;
    const afterSafety = ownDrainerSafetyScore(
      execution,
      applied.game.board,
      perspective,
    );
    const afterPoolDistance = distanceToNearestPool(next, perspective);
    if (afterSafety >= beforeSafety || afterPoolDistance >= beforePoolDistance)
      continue;
    const recoveryPriority = bestFollowUpSafetyRecoveryPriority(
      execution,
      applied.game,
      perspective,
    );
    if (recoveryPriority === undefined) continue;
    result.push({
      family: TurnPlanFamily.ManaTempo,
      action: { kind: "walk", actor: drainer, to: next },
      priority:
        8_000 +
        Math.max(beforePoolDistance - afterPoolDistance, 0) * 260 +
        Math.trunc(recoveryPriority / 20) -
        Math.max(beforeSafety - afterSafety, 0) * 120,
    });
  }
  return result;
}

export function tacticalProjectionFlags(
  needSupermanaProgress: boolean,
  needOpponentManaProgress: boolean,
  needSpiritScore: boolean,
  needSpiritDenial: boolean,
  needScoreWindow: boolean,
): number {
  let flags = 0;
  if (needSupermanaProgress)
    flags |= EXACT_TURN_TACTICAL_NEED_SUPERMANA_PROGRESS;
  if (needOpponentManaProgress)
    flags |= EXACT_TURN_TACTICAL_NEED_OPPONENT_MANA_PROGRESS;
  if (needSpiritScore) flags |= EXACT_TURN_TACTICAL_NEED_SPIRIT_SCORE;
  if (needSpiritDenial) flags |= EXACT_TURN_TACTICAL_NEED_SPIRIT_DENIAL;
  if (needScoreWindow) flags |= EXACT_TURN_TACTICAL_NEED_SCORE_WINDOW;
  return flags;
}

export type OracleWalkProjectionProfile =
  | "safe-progress-only"
  | "opponent-progress-only"
  | "drainer-opportunity"
  | "spirit-score-only"
  | "spirit-opportunity";

export type OracleWalkActorCapabilities = {
  readonly canEmitSupermana: boolean;
  readonly canEmitOpponentMana: boolean;
  readonly canEmitSafety: boolean;
  readonly canEmitSpirit: boolean;
  readonly tacticalFlags: number;
  readonly projectionProfile: OracleWalkProjectionProfile | undefined;
  readonly needsScoreWindow: boolean;
};

export function tacticalProjectionProfileFlags(
  profile: OracleWalkProjectionProfile,
): number {
  switch (profile) {
    case "safe-progress-only":
      return tacticalProjectionFlags(true, false, false, false, false);
    case "opponent-progress-only":
      return tacticalProjectionFlags(false, true, false, false, false);
    case "drainer-opportunity":
      return tacticalProjectionFlags(false, true, false, true, false);
    case "spirit-score-only":
      return tacticalProjectionFlags(false, false, true, false, false);
    case "spirit-opportunity":
      return tacticalProjectionFlags(true, false, true, false, false);
  }
}

export function oracleWalkActorCapabilities(
  monKind: MonKind,
  allowSupermana: boolean,
  allowOpponentMana: boolean,
  allowSafety: boolean,
  allowSpirit: boolean,
): OracleWalkActorCapabilities {
  const canEmitSupermana = allowSupermana;
  const canEmitOpponentMana = allowOpponentMana && monKind !== MonKind.Spirit;
  const canEmitSafety = allowSafety;
  const canEmitSpirit = allowSpirit && monKind === MonKind.Spirit;
  const projectionProfile: OracleWalkProjectionProfile | undefined =
    canEmitSupermana && canEmitSpirit
      ? "spirit-opportunity"
      : canEmitSupermana
        ? "safe-progress-only"
        : canEmitOpponentMana && allowSpirit
          ? "drainer-opportunity"
          : canEmitOpponentMana
            ? "opponent-progress-only"
            : canEmitSpirit
              ? "spirit-score-only"
              : undefined;
  return {
    canEmitSupermana,
    canEmitOpponentMana,
    canEmitSafety,
    canEmitSpirit,
    tacticalFlags: tacticalProjectionFlags(
      canEmitSupermana,
      canEmitOpponentMana,
      canEmitSpirit,
      allowSpirit && canEmitOpponentMana,
      canEmitSupermana || canEmitSpirit,
    ),
    projectionProfile,
    needsScoreWindow: canEmitSupermana || canEmitSpirit,
  };
}

export function strategicSpiritSignalWithSearchHash(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  stateHash: Hash64,
): readonly [number, number] {
  const spirit = exactStrategicAnalysisWithSearchHash(
    execution,
    game,
    stateHash,
  ).colorSummary(perspective).spirit;
  return [spirit.nextTurnSetupGain, spirit.utility];
}

export function oracleWalkSeeds(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  context: ExactOpportunityContext,
  allowedFamilies: readonly TurnPlanFamily[] | undefined,
  config: TurnEngineConfig,
): ActionSeed[] {
  if (
    execution.session.checkpoint() ||
    remainingMovesForColor(game, perspective) <= 0
  )
    return [];

  const allowSupermana = familyAllowed(
    allowedFamilies,
    TurnPlanFamily.SafeSupermanaProgress,
  );
  const allowOpponentMana = familyAllowed(
    allowedFamilies,
    TurnPlanFamily.SafeOpponentManaProgress,
  );
  const allowSafety = familyAllowed(
    allowedFamilies,
    TurnPlanFamily.DrainerSafetyRecovery,
  );
  const allowSpirit = familyAllowed(
    allowedFamilies,
    TurnPlanFamily.SpiritImpact,
  );
  if (!allowSupermana && !allowOpponentMana && !allowSafety && !allowSpirit) {
    return [];
  }

  const before = context.turn;
  let beforeSpirit: readonly [number, number] = [0, 0];
  if (allowSpirit) {
    beforeSpirit = strategicSpiritSignalWithSearchHash(
      execution,
      game,
      perspective,
      exactSearchStateHash(game),
    );
  }
  if (execution.session.checkpoint()) return [];
  const beforeSafety =
    allowSupermana || allowSafety
      ? ownDrainerSafetyScore(execution, game.board, perspective)
      : 0;
  const beforeSuperSteps = before.safeSupermanaProgressSteps ?? BOARD_SIZE * 3;
  const beforeOpponentSteps =
    before.safeOpponentManaProgressSteps ?? BOARD_SIZE * 3;
  const ownDrainer = findAwakeDrainerLocation(game.board, perspective);
  const result: ActionSeed[] = [];
  const useLazyScoreWindowProjection =
    config.enableLazyOracleScoreWindowProjection;

  for (const [actor, item] of game.board.entries()) {
    if (execution.session.checkpoint()) return [];
    const mon = itemMon(item);
    if (
      mon?.color !== perspective ||
      isMonFainted(mon) ||
      (ownDrainer !== undefined && locationEquals(actor, ownDrainer))
    ) {
      continue;
    }
    const capabilities = oracleWalkActorCapabilities(
      mon.kind,
      allowSupermana,
      allowOpponentMana,
      allowSafety,
      allowSpirit,
    );
    if (
      !capabilities.canEmitSupermana &&
      !capabilities.canEmitOpponentMana &&
      !capabilities.canEmitSafety &&
      !capabilities.canEmitSpirit
    ) {
      continue;
    }
    for (const to of nearbyLocations(actor)) {
      if (execution.session.checkpoint()) return [];
      if (!walkDestinationPlausible(game.board, actor, to)) continue;
      const applied = applyInputsForSearchWithEvents(game, [
        { kind: "location", location: actor },
        { kind: "location", location: to },
      ]);
      if (
        applied === undefined ||
        opponentCanWinImmediately(execution, applied.game, perspective)
      )
        continue;

      const needAfterSpirit = capabilities.canEmitSpirit;
      const needAfterTurn = useLazyScoreWindowProjection
        ? capabilities.projectionProfile !== undefined
        : capabilities.tacticalFlags !== 0;
      const needAfterScoreWindow =
        useLazyScoreWindowProjection && capabilities.needsScoreWindow;
      const afterHash =
        needAfterTurn || needAfterScoreWindow || needAfterSpirit
          ? exactSearchStateHash(applied.game)
          : undefined;
      let after: ExactTurnTacticalProjection | undefined;
      if (needAfterTurn) {
        if (afterHash === undefined) {
          throw new Error("oracle walk projection requires a state hash");
        }
        let flags = capabilities.tacticalFlags;
        if (useLazyScoreWindowProjection) {
          if (capabilities.projectionProfile === undefined) {
            throw new Error("lazy oracle projection requires a profile");
          }
          flags = tacticalProjectionProfileFlags(
            capabilities.projectionProfile,
          );
        }
        after = exactTurnTacticalProjectionWithSearchHash(
          execution,
          applied.game,
          perspective,
          afterHash,
          flags,
        );
      }
      if (execution.session.cancelled) return [];
      let afterSpirit: readonly [number, number] = [0, 0];
      if (needAfterSpirit) {
        if (afterHash === undefined) {
          throw new Error("oracle Spirit analysis requires a state hash");
        }
        afterSpirit = strategicSpiritSignalWithSearchHash(
          execution,
          applied.game,
          perspective,
          afterHash,
        );
      }
      const afterSafety =
        capabilities.canEmitSupermana || capabilities.canEmitSafety
          ? ownDrainerSafetyScore(execution, applied.game.board, perspective)
          : beforeSafety;
      const afterSuperSteps =
        after === undefined
          ? beforeSuperSteps
          : (after.safeSupermanaProgressSteps ?? BOARD_SIZE * 3);
      const afterOpponentSteps =
        after === undefined
          ? beforeOpponentSteps
          : (after.safeOpponentManaProgressSteps ?? BOARD_SIZE * 3);
      let afterScoreWindowValue: number | undefined;
      const loadAfterScoreWindow = (): number => {
        if (afterScoreWindowValue !== undefined) return afterScoreWindowValue;
        if (useLazyScoreWindowProjection) {
          if (afterHash === undefined) {
            throw new Error(
              "oracle score-window projection requires a state hash",
            );
          }
          afterScoreWindowValue = exactTurnTacticalProjectionWithSearchHash(
            execution,
            applied.game,
            perspective,
            afterHash,
            EXACT_TURN_TACTICAL_NEED_SCORE_WINDOW,
          ).sameTurnScoreWindowValue;
        } else {
          afterScoreWindowValue = after?.sameTurnScoreWindowValue ?? 0;
        }
        return afterScoreWindowValue;
      };

      if (capabilities.canEmitSupermana && afterSuperSteps < beforeSuperSteps) {
        result.push({
          family: TurnPlanFamily.SafeSupermanaProgress,
          action: { kind: "walk", actor, to },
          priority:
            8_250 +
            (beforeSuperSteps - afterSuperSteps) * 240 +
            (afterSafety - beforeSafety) * 100 +
            loadAfterScoreWindow() * 160,
        });
      }

      const opponentProgressImproved =
        capabilities.canEmitOpponentMana &&
        afterOpponentSteps < beforeOpponentSteps;
      const spiritDenialImproved =
        allowSpirit &&
        (capabilities.canEmitSpirit || capabilities.canEmitOpponentMana) &&
        (after?.spiritAssistedDenialValue ?? 0) >
          before.spiritAssistedDenialValue;
      if (opponentProgressImproved || spiritDenialImproved) {
        const family =
          mon.kind === MonKind.Spirit
            ? TurnPlanFamily.SpiritImpact
            : TurnPlanFamily.SafeOpponentManaProgress;
        if (familyAllowed(allowedFamilies, family)) {
          result.push({
            family,
            action: { kind: "walk", actor, to },
            priority:
              8_000 +
              (opponentProgressImproved
                ? Math.max(beforeOpponentSteps - afterOpponentSteps, 0) * 240
                : 0) +
              (spiritDenialImproved
                ? Math.max(
                    (after?.spiritAssistedDenialValue ?? 0) -
                      before.spiritAssistedDenialValue,
                    0,
                  ) * 180
                : 0),
          });
        }
      }

      if (capabilities.canEmitSpirit) {
        const setupDelta = afterSpirit[0] - beforeSpirit[0];
        const utilityDelta = afterSpirit[1] - beforeSpirit[1];
        const spiritSetupImproved = setupDelta > 0 || utilityDelta > 0;
        const spiritScoreBaseImproved =
          (after?.spiritAssistedScoreValue ?? 0) >
          before.spiritAssistedScoreValue;
        const spiritScoreWindowImproved = useLazyScoreWindowProjection
          ? !spiritScoreBaseImproved &&
            !spiritSetupImproved &&
            loadAfterScoreWindow() > before.sameTurnScoreWindowValue
          : (after?.sameTurnScoreWindowValue ?? 0) >
            before.sameTurnScoreWindowValue;
        if (
          spiritScoreBaseImproved ||
          spiritScoreWindowImproved ||
          spiritSetupImproved
        ) {
          const scoreDelta =
            (after?.spiritAssistedScoreValue ?? 0) -
            before.spiritAssistedScoreValue;
          const windowDelta =
            loadAfterScoreWindow() - before.sameTurnScoreWindowValue;
          result.push({
            family: TurnPlanFamily.SpiritImpact,
            action: { kind: "walk", actor, to },
            priority:
              8_100 +
              scoreDelta * 200 +
              windowDelta * 220 +
              setupDelta * 320 +
              utilityDelta * 180,
          });
        }
      }

      if (capabilities.canEmitSafety && afterSafety > beforeSafety) {
        result.push({
          family: TurnPlanFamily.DrainerSafetyRecovery,
          action: { kind: "walk", actor, to },
          priority: 8_050 + (afterSafety - beforeSafety) * 260,
        });
      }
    }
  }
  return result;
}

export function progressPriorityBonus(
  before: number | undefined,
  after: number | undefined,
): number {
  const beforeSteps = before ?? BOARD_SIZE * 3;
  const afterSteps = after ?? BOARD_SIZE * 3;
  return afterSteps >= beforeSteps ? 0 : (beforeSteps - afterSteps) * 220;
}

export function spiritImpactSeeds(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: TurnEngineConfig,
): ActionSeed[] {
  if (
    execution.session.checkpoint() ||
    !config.enableSpiritFamily ||
    !game.playerCanUseAction()
  )
    return [];
  const flags = tacticalProjectionFlags(true, true, true, true, true);
  const before = exactTurnTacticalProjectionWithSearchHash(
    execution,
    game,
    perspective,
    exactSearchStateHash(game),
    flags,
  );
  if (execution.session.checkpoint()) return [];
  const beforeSafety = ownDrainerSafetyScore(
    execution,
    game.board,
    perspective,
  );
  const result: ActionSeed[] = [];
  for (const [spirit, item] of game.board.entries()) {
    if (execution.session.checkpoint()) return [];
    const mon = itemMon(item);
    if (
      mon?.color !== perspective ||
      mon.kind !== MonKind.Spirit ||
      isMonFainted(mon) ||
      game.board.squareAt(spirit).kind === "mon-base"
    ) {
      continue;
    }
    for (const target of spiritReachableLocations(spirit)) {
      if (execution.session.checkpoint()) return [];
      const targetItem = game.board.get(target);
      if (targetItem === undefined || !isSpiritTargetAllowed(targetItem))
        continue;
      for (const destination of nearbyLocations(target)) {
        if (execution.session.checkpoint()) return [];
        if (!spiritDestinationAllowed(game.board, targetItem, destination))
          continue;
        const applied = applyInputsForSearchWithEvents(game, [
          { kind: "location", location: spirit },
          { kind: "location", location: target },
          { kind: "location", location: destination },
        ]);
        if (applied === undefined) continue;
        let priority = 7_600;
        const targetMon = itemMon(targetItem);
        if (targetMon?.color === otherColor(perspective)) priority += 400;
        const targetMana = itemMana(targetItem);
        if (targetMana?.kind === "supermana") priority += 600;
        if (
          targetMana?.kind === "regular" &&
          targetMana.color === otherColor(perspective)
        ) {
          priority += 460;
        }
        const after = exactTurnTacticalProjectionWithSearchHash(
          execution,
          applied.game,
          perspective,
          exactSearchStateHash(applied.game),
          flags,
        );
        if (execution.session.checkpoint()) return [];
        if (after.sameTurnScoreWindowValue > before.sameTurnScoreWindowValue) {
          priority +=
            (after.sameTurnScoreWindowValue - before.sameTurnScoreWindowValue) *
            280;
        }
        if (after.spiritAssistedScore) {
          priority += 900 + after.spiritAssistedScoreValue * 120;
        }
        if (after.safeSupermanaProgress) {
          priority +=
            700 +
            progressPriorityBonus(
              before.safeSupermanaProgressSteps,
              after.safeSupermanaProgressSteps,
            );
        }
        if (after.safeOpponentManaProgress) {
          priority +=
            760 +
            progressPriorityBonus(
              before.safeOpponentManaProgressSteps,
              after.safeOpponentManaProgressSteps,
            );
        }
        if (after.spiritAssistedDenial) {
          priority += 820 + after.spiritAssistedDenialValue * 140;
        }
        const afterSafety = ownDrainerSafetyScore(
          execution,
          applied.game.board,
          perspective,
        );
        if (afterSafety > beforeSafety)
          priority += (afterSafety - beforeSafety) * 160;
        priority +=
          Math.max(BOARD_SIZE - locationDistance(destination, target), 0) * 20;
        result.push({
          family: TurnPlanFamily.SpiritImpact,
          action: {
            kind: "spirit-shift",
            actor: spirit,
            target,
            destination,
          },
          priority,
        });
      }
    }
  }
  result.sort((left, right) => {
    const order = compareNumber(right.priority, left.priority);
    return order !== 0 ? order : compareActionKeys(left.action, right.action);
  });
  return result.slice(0, 12);
}

export function manaTempoSeeds(
  game: MonsGame,
  perspective: Color,
): ActionSeed[] {
  if (
    !game.playerCanMoveMana() ||
    findAwakeDrainerLocation(game.board, perspective) !== undefined
  ) {
    return [];
  }
  const result: ActionSeed[] = [];
  for (const [from, item] of game.board.entries()) {
    if (
      item.kind !== "mana" ||
      item.mana.kind !== "regular" ||
      item.mana.color !== perspective
    ) {
      continue;
    }
    for (const to of nearbyLocations(from)) {
      if (!manaMoveDestinationAllowed(game.board, to)) continue;
      const ownGain =
        distanceToNearestPool(from, perspective) -
        distanceToNearestPool(to, perspective);
      const opponent = otherColor(perspective);
      const opponentGain =
        distanceToNearestPool(from, opponent) -
        distanceToNearestPool(to, opponent);
      if (ownGain <= 0 || opponentGain > 0) continue;
      result.push({
        family: TurnPlanFamily.ManaTempo,
        action: { kind: "move-mana", from, to },
        priority: 6_900 + ownGain * 200 - Math.max(opponentGain, 0) * 200,
      });
    }
  }
  return result;
}
