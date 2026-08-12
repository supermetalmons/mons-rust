import type { Color } from "../../api/types.js";
import type { Input } from "../../engine/model/domain.js";
import { MonsGame } from "../../engine/game/mons-game.js";
import type { AutomoveExecutionContext } from "../core/execution-context.js";
import { Hash64Set, Hash64Table, type Hash64 } from "../core/hash64.js";
import { exactSearchStateHash } from "../exact/hash.js";
import { compileAction } from "./compiler.js";
import {
  createTurnUtilityEvalContextWithSearchHash,
  evaluateStateUtilityWithSearchHash,
  ownDrainerSafetyScore,
  quickOrderScoreWithSearchHash,
  turnOracleContextWithSearchHash,
  type TurnUtilityEvalContext,
} from "./evaluation.js";
import {
  EMPTY_PACKAGE_META,
  LOCAL_HASH_COLLECTION_CAPACITY,
  PlanBuildStatus,
  TurnPlanFamily,
  type MacroOpportunity,
  type MacroPlanNode,
  type PlanGenerationResult,
  type TurnAction,
  type TurnEngineConfig,
  type TurnOpportunity,
  type TurnPlan,
} from "./model.js";
import { discoverTurnOpportunities } from "./opportunities.js";
import {
  compareChunks,
  compareNumber,
  compareTurnUtilities,
  familyRank,
  turnEngineComparePlans,
} from "./ordering.js";
import {
  bundleChunkCapForConfig,
  bundlePlanCapForConfig,
  macroFollowupFamilies,
  macroFollowupFamilyBonus,
  macroFollowupSeedCandidates,
  macroOpportunityDelta,
  macroPlanSignature,
  macroPriorityFromState,
  macroSignatureForActions,
  mergePlanFamily,
} from "./planner-macro-policy.js";
import { compareOrderedNodes } from "./planner-order.js";

function buildMacroFromHeadOpportunity(
  execution: AutomoveExecutionContext,
  root: MonsGame,
  perspective: Color,
  config: TurnEngineConfig,
  utilityContext: TurnUtilityEvalContext,
  opportunity: TurnOpportunity,
): MacroOpportunity | undefined {
  if (execution.session.checkpoint()) return undefined;
  const startOracle = turnOracleContextWithSearchHash(
    execution,
    root,
    perspective,
    utilityContext.startHash,
  );
  if (execution.session.cancelled) return undefined;
  const first = compileAction(execution, root, perspective, opportunity.action, config);
  if (first === undefined || execution.session.checkpoint()) return undefined;
  let current = first[0];
  let currentHash = exactSearchStateHash(current);
  const headUtility = evaluateStateUtilityWithSearchHash(
    utilityContext,
    current,
    currentHash,
  );
  const sessionAfterHeadEvaluation = execution.session;
  if (sessionAfterHeadEvaluation.cancelled) return undefined;
  const actions: TurnAction[] = [opportunity.action];
  const compiledChunks: Input[][] = [first[1]];
  let goalFamily = opportunity.family;
  const visitedStates = new Hash64Set(LOCAL_HASH_COLLECTION_CAPACITY);
  visitedStates.add(utilityContext.startHash);
  visitedStates.add(currentHash);

  while (
    current.activeColor === perspective &&
    current.winnerColor() === undefined &&
    compiledChunks.length < bundleChunkCapForConfig(config)
  ) {
    if (execution.session.checkpoint()) return undefined;
    const currentOracle = turnOracleContextWithSearchHash(
      execution,
      current,
      perspective,
      currentHash,
    );
    const sessionAfterCurrentOracle = execution.session;
    if (sessionAfterCurrentOracle.cancelled) return undefined;
    const currentUtility = evaluateStateUtilityWithSearchHash(
      utilityContext,
      current,
      currentHash,
    );
    const sessionAfterCurrentEvaluation = execution.session;
    if (sessionAfterCurrentEvaluation.cancelled) return undefined;
    const riskyTemporaryState =
      currentOracle.opportunity.delta.drainerSafety < 0 ||
      currentUtility.drainerSafety < 0 ||
      ownDrainerSafetyScore(execution, current.board, perspective) < 0;
    let best:
      | {
          readonly score: number;
          readonly opportunity: TurnOpportunity;
          readonly after: MonsGame;
          readonly afterHash: Hash64;
          readonly chunk: Input[];
          readonly goalFamily: TurnPlanFamily;
        }
      | undefined;

    for (const followup of macroFollowupSeedCandidates(
      execution,
      current,
      currentHash,
      perspective,
      config,
      opportunity.family,
      goalFamily,
      actions,
    )) {
      if (execution.session.checkpoint()) return undefined;
      const compiled = compileAction(
        execution,
        current,
        perspective,
        followup.action,
        config,
      );
      if (compiled === undefined || execution.session.checkpoint()) continue;
      const [after, chunk] = compiled;
      const afterHash = exactSearchStateHash(after);
      if (visitedStates.has(afterHash)) continue;
      const delta = macroOpportunityDelta(
        execution,
        current,
        after,
        afterHash,
        perspective,
        currentOracle,
      );
      const nextGoalFamily = mergePlanFamily(goalFamily, followup.family);
      const nextUtility = evaluateStateUtilityWithSearchHash(
        utilityContext,
        after,
        afterHash,
      );
      const sessionAfterFollowupEvaluation = execution.session;
      if (sessionAfterFollowupEvaluation.cancelled) return undefined;
      const improvementSignal =
        delta.sameTurnScoreWindowGain +
        delta.spiritGain +
        delta.opponentWindowDenyGain +
        Math.max(delta.drainerSafetyDelta, 0) +
        delta.supermanaProgressGain +
        delta.opponentManaProgressGain +
        (delta.drainerAttack ? 2 : 0);
      const temporaryRecoveryFollowup =
        riskyTemporaryState &&
        (followup.family === TurnPlanFamily.DrainerSafetyRecovery ||
          followup.family === TurnPlanFamily.ImmediateScore ||
          followup.family === TurnPlanFamily.SafeSupermanaProgress ||
          followup.family === TurnPlanFamily.SafeOpponentManaProgress);
      if (
        improvementSignal <= 0 &&
        compareTurnUtilities(nextUtility, currentUtility) <= 0 &&
        after.activeColor === perspective &&
        !temporaryRecoveryFollowup
      ) {
        continue;
      }
      const riskyBonus = riskyTemporaryState
        ? (() => {
            switch (followup.family) {
              case TurnPlanFamily.DrainerSafetyRecovery:
                return 960;
              case TurnPlanFamily.ImmediateScore:
                return 820;
              case TurnPlanFamily.SafeSupermanaProgress:
              case TurnPlanFamily.SafeOpponentManaProgress:
                return 360;
              case TurnPlanFamily.DenyOpponentWindow:
              case TurnPlanFamily.DrainerKill:
                return 220;
              case TurnPlanFamily.SpiritImpact:
              case TurnPlanFamily.ManaTempo:
                return 0;
            }
          })()
        : 0;
      const score =
        macroPriorityFromState(
          utilityContext,
          after,
          afterHash,
          nextGoalFamily,
          compiledChunks.length + 1,
          followup.priority +
            macroFollowupFamilyBonus(opportunity.family, goalFamily, followup.family),
        ) +
        riskyBonus +
        delta.sameTurnScoreWindowGain * 280 +
        delta.spiritGain * 220 +
        delta.opponentWindowDenyGain * 240 +
        Math.max(delta.drainerSafetyDelta, 0) * 200 +
        delta.supermanaProgressGain * 120 +
        delta.opponentManaProgressGain * 112 +
        (delta.drainerAttack ? 820 : 0);
      if (
        best === undefined ||
        score > best.score ||
        (score === best.score &&
          familyRank(nextGoalFamily) < familyRank(best.goalFamily))
      ) {
        best = {
          score,
          opportunity: followup,
          after,
          afterHash,
          chunk,
          goalFamily: nextGoalFamily,
        };
      }
    }
    if (best === undefined) break;
    actions.push(best.opportunity.action);
    compiledChunks.push(best.chunk);
    goalFamily = best.goalFamily;
    current = best.after;
    currentHash = best.afterHash;
    visitedStates.add(currentHash);
  }

  if (execution.session.checkpoint()) return undefined;
  const endSnapshot = { stateHash: currentHash };
  const delta = macroOpportunityDelta(
    execution,
    root,
    current,
    currentHash,
    perspective,
    startOracle,
  );
  const sessionAfterMacroDelta = execution.session;
  if (sessionAfterMacroDelta.cancelled) return undefined;
  return {
    headFamily: opportunity.family,
    goalFamily,
    priority: macroPriorityFromState(
      utilityContext,
      current,
      currentHash,
      goalFamily,
      compiledChunks.length,
      opportunity.priority,
    ),
    delta,
    actions,
    compiledChunks,
    endGame: current.fork(),
    endSnapshot,
    headUtility,
    signature: macroSignatureForActions(actions),
  };
}

function discoverMacroOpportunities(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: TurnEngineConfig,
  utilityContext: TurnUtilityEvalContext,
  opportunityCap: number,
  allowedFamilies?: readonly TurnPlanFamily[],
): MacroOpportunity[] {
  if (execution.session.checkpoint()) return [];
  const macros: MacroOpportunity[] = [];
  const seen = new Hash64Set(LOCAL_HASH_COLLECTION_CAPACITY);
  const opportunities = discoverTurnOpportunities(
    execution,
    game,
    perspective,
    config,
    Math.max(Math.max(opportunityCap, config.perNodeFamilyCap * 3), 8),
    allowedFamilies,
  );
  if (execution.session.checkpoint()) return [];
  for (const opportunity of opportunities) {
    if (execution.session.checkpoint()) return [];
    const bundle = buildMacroFromHeadOpportunity(
      execution,
      game,
      perspective,
      config,
      utilityContext,
      opportunity,
    );
    if (bundle === undefined) {
      if (execution.session.cancelled) return [];
      continue;
    }
    if (seen.has(bundle.endSnapshot.stateHash, 0, bundle.signature)) continue;
    seen.add(bundle.endSnapshot.stateHash, 0, bundle.signature);
    macros.push(bundle);
    if (macros.length >= Math.max(opportunityCap, 1)) break;
  }
  macros.sort((left, right) => {
    const score = (value: MacroOpportunity): number =>
      value.priority +
      value.delta.sameTurnScoreWindowGain * 280 +
      value.delta.spiritGain * 220 +
      value.delta.opponentWindowDenyGain * 240 +
      value.delta.drainerSafetyDelta * 220 +
      value.delta.supermanaProgressGain * 120 +
      value.delta.opponentManaProgressGain * 112 +
      (value.delta.drainerAttack ? 820 : 0) +
      (bundleChunkCapForConfig(config) - value.compiledChunks.length) * 8;
    const order = compareNumber(score(right), score(left));
    if (order !== 0) return order;
    const familyOrder = compareNumber(
      familyRank(left.goalFamily),
      familyRank(right.goalFamily),
    );
    return familyOrder !== 0
      ? familyOrder
      : compareChunks(left.compiledChunks, right.compiledChunks);
  });
  return execution.session.checkpoint()
    ? []
    : macros.slice(0, Math.max(opportunityCap, 1));
}

function macroNodeToPlan(
  utilityContext: TurnUtilityEvalContext,
  node: MacroPlanNode,
): TurnPlan {
  return {
    actions: node.actions,
    compiledChunks: node.compiledChunks,
    endGame: node.game.fork(),
    utility: evaluateStateUtilityWithSearchHash(
      utilityContext,
      node.game,
      node.stateHash,
    ),
    headUtility: node.headUtility,
    headFamily: node.headFamily,
    goalFamily: node.goalFamily,
    packageMeta: EMPTY_PACKAGE_META,
  };
}

export function generateMacroPlans(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: TurnEngineConfig,
  utilityContext: TurnUtilityEvalContext,
  opportunityCap: number,
  beamWidth: number,
  bundleCap: number,
  expansionCap: number,
): PlanGenerationResult {
  if (execution.session.checkpoint()) return { status: PlanBuildStatus.BudgetExceeded };
  let expansions = 0;
  let budgetExhausted = false;
  const cap = Math.min(Math.max(bundleCap, 1), bundlePlanCapForConfig(config));
  const opportunities = discoverMacroOpportunities(
    execution,
    game,
    perspective,
    config,
    utilityContext,
    opportunityCap,
  );
  if (execution.session.checkpoint()) return { status: PlanBuildStatus.BudgetExceeded };
  if (opportunities.length === 0) return { status: PlanBuildStatus.NoPlan };
  const seen = new Hash64Table<number>(LOCAL_HASH_COLLECTION_CAPACITY);
  let frontier: { readonly order: number; readonly node: MacroPlanNode }[] = [];
  for (const opportunity of opportunities) {
    if (execution.session.checkpoint())
      return { status: PlanBuildStatus.BudgetExceeded };
    expansions += 1;
    if (expansions > expansionCap) {
      budgetExhausted = true;
      break;
    }
    const order = quickOrderScoreWithSearchHash(
      utilityContext,
      opportunity.endGame,
      opportunity.endSnapshot.stateHash,
      opportunity.goalFamily,
      opportunity.compiledChunks.length,
    );
    if (execution.session.cancelled) return { status: PlanBuildStatus.BudgetExceeded };
    const existing = seen.get(
      opportunity.endSnapshot.stateHash,
      0,
      opportunity.signature,
    );
    if (existing !== undefined && order <= existing) continue;
    seen.set(opportunity.endSnapshot.stateHash, order, 0, opportunity.signature);
    frontier.push({
      order,
      node: {
        game: opportunity.endGame,
        stateHash: opportunity.endSnapshot.stateHash,
        actions: opportunity.actions,
        compiledChunks: opportunity.compiledChunks,
        headUtility: opportunity.headUtility,
        headFamily: opportunity.headFamily,
        goalFamily: opportunity.goalFamily,
        signature: opportunity.signature,
      },
    });
  }
  if (frontier.length === 0) {
    return {
      status: budgetExhausted ? PlanBuildStatus.BudgetExceeded : PlanBuildStatus.NoPlan,
    };
  }
  frontier.sort(compareOrderedNodes);
  frontier = frontier.slice(0, Math.max(beamWidth, 1));
  const terminal: MacroPlanNode[] = [];

  for (let round = 1; round < cap; round += 1) {
    if (execution.session.checkpoint())
      return { status: PlanBuildStatus.BudgetExceeded };
    const candidates: {
      readonly order: number;
      readonly node: MacroPlanNode;
    }[] = [];
    let expandedAny = false;
    let stopExpansion = false;
    for (const current of frontier) {
      const node = current.node;
      if (execution.session.checkpoint())
        return { status: PlanBuildStatus.BudgetExceeded };
      if (
        node.game.winnerColor() !== undefined ||
        node.game.activeColor !== perspective
      ) {
        terminal.push(node);
        continue;
      }
      const followupUtilityContext = createTurnUtilityEvalContextWithSearchHash(
        execution,
        node.game,
        node.stateHash,
        perspective,
        config,
      );
      const followups = discoverMacroOpportunities(
        execution,
        node.game,
        perspective,
        config,
        followupUtilityContext,
        opportunityCap,
        macroFollowupFamilies(node.headFamily, node.goalFamily),
      );
      if (execution.session.checkpoint())
        return { status: PlanBuildStatus.BudgetExceeded };
      if (followups.length === 0) {
        terminal.push(node);
        continue;
      }
      let nodeExpanded = false;
      for (const opportunity of followups) {
        if (execution.session.checkpoint())
          return { status: PlanBuildStatus.BudgetExceeded };
        expansions += 1;
        if (expansions > expansionCap) {
          terminal.push(node);
          budgetExhausted = true;
          stopExpansion = true;
          break;
        }
        const actions = [...node.actions, ...opportunity.actions];
        const chunks = [...node.compiledChunks, ...opportunity.compiledChunks];
        const goalFamily = mergePlanFamily(node.goalFamily, opportunity.goalFamily);
        const signature = macroPlanSignature(node.signature, opportunity);
        const order = quickOrderScoreWithSearchHash(
          utilityContext,
          opportunity.endGame,
          opportunity.endSnapshot.stateHash,
          goalFamily,
          chunks.length,
        );
        if (execution.session.cancelled)
          return { status: PlanBuildStatus.BudgetExceeded };
        const existing = seen.get(opportunity.endSnapshot.stateHash, 0, signature);
        if (existing !== undefined && order <= existing) continue;
        seen.set(opportunity.endSnapshot.stateHash, order, 0, signature);
        candidates.push({
          order,
          node: {
            game: opportunity.endGame,
            stateHash: opportunity.endSnapshot.stateHash,
            actions,
            compiledChunks: chunks,
            headUtility: node.headUtility,
            headFamily: node.headFamily,
            goalFamily,
            signature,
          },
        });
        expandedAny = true;
        nodeExpanded = true;
      }
      if (stopExpansion) break;
      if (!nodeExpanded) terminal.push(node);
    }
    if (stopExpansion) {
      candidates.sort(compareOrderedNodes);
      frontier = candidates.slice(0, Math.max(beamWidth, 1));
      break;
    }
    if (!expandedAny || candidates.length === 0) {
      frontier = [];
      break;
    }
    candidates.sort(compareOrderedNodes);
    frontier = candidates.slice(0, Math.max(beamWidth, 1));
  }
  terminal.push(...frontier.map(({ node }) => node));
  if (execution.session.checkpoint()) return { status: PlanBuildStatus.BudgetExceeded };
  if (terminal.length === 0) {
    return {
      status: budgetExhausted ? PlanBuildStatus.BudgetExceeded : PlanBuildStatus.NoPlan,
    };
  }
  const plans = terminal.map((node) => macroNodeToPlan(utilityContext, node));
  if (execution.session.checkpoint()) return { status: PlanBuildStatus.BudgetExceeded };
  plans.sort((left, right) => -turnEngineComparePlans(left, right));
  return { status: "ok", plans };
}
