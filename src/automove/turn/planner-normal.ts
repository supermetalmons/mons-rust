import type { Color } from "../../api/types.js";
import { MonsGame } from "../../engine/game/mons-game.js";
import type { AutomoveExecutionContext } from "../core/execution-context.js";
import { Hash64Table } from "../core/hash64.js";
import { exactSearchStateHash } from "../exact/hash.js";
import { TransitionCompilePool, compileActionFromPool } from "./compiler.js";
import {
  evaluateStateUtilityWithSearchHash,
  quickOrderScoreWithSearchHash,
  type TurnUtilityEvalContext,
} from "./evaluation.js";
import {
  EMPTY_PACKAGE_META,
  LOCAL_HASH_COLLECTION_CAPACITY,
  PlanBuildStatus,
  type PlanGenerationResult,
  type PlanNode,
  type TurnEngineConfig,
  type TurnPlan,
} from "./model.js";
import { generateActionSeeds } from "./opportunities.js";
import { turnEngineComparePlans } from "./ordering.js";
import { compareOrderedNodes } from "./planner-order.js";

export function generateTurnPlans(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: TurnEngineConfig,
  utilityContext: TurnUtilityEvalContext,
  seedCap: number,
  beamWidth: number,
  stepCap: number,
  expansionCap: number,
): PlanGenerationResult {
  if (execution.session.checkpoint()) return { status: PlanBuildStatus.BudgetExceeded };
  let expansions = 0;
  const seeds = generateActionSeeds(execution, game, perspective, config, seedCap);
  if (execution.session.checkpoint()) return { status: PlanBuildStatus.BudgetExceeded };
  if (seeds.length === 0) return { status: PlanBuildStatus.NoPlan };
  const compilePool = new TransitionCompilePool(execution, game, seeds, config);
  if (execution.session.checkpoint()) return { status: PlanBuildStatus.BudgetExceeded };
  const seen = new Hash64Table<number>(LOCAL_HASH_COLLECTION_CAPACITY);
  let frontier: { readonly order: number; readonly node: PlanNode }[] = [];

  for (const seed of seeds) {
    if (execution.session.checkpoint())
      return { status: PlanBuildStatus.BudgetExceeded };
    const compiled = compileActionFromPool(
      execution,
      game,
      perspective,
      seed.action,
      compilePool,
    );
    if (compiled === undefined) continue;
    if (execution.session.checkpoint())
      return { status: PlanBuildStatus.BudgetExceeded };
    expansions += 1;
    if (expansions > expansionCap) {
      return { status: PlanBuildStatus.BudgetExceeded };
    }
    const [after, chunk] = compiled;
    const hash = exactSearchStateHash(after);
    const order = quickOrderScoreWithSearchHash(
      utilityContext,
      after,
      hash,
      seed.family,
      1,
    );
    const existing = seen.get(hash);
    if (existing !== undefined && order <= existing) continue;
    seen.set(hash, order);
    const headUtility = evaluateStateUtilityWithSearchHash(utilityContext, after, hash);
    if (execution.session.cancelled) return { status: PlanBuildStatus.BudgetExceeded };
    frontier.push({
      order,
      node: {
        game: after,
        stateHash: hash,
        actions: [seed.action],
        compiledChunks: [chunk],
        headUtility,
        headFamily: seed.family,
        goalFamily: seed.family,
      },
    });
  }
  if (frontier.length === 0) return { status: PlanBuildStatus.NoPlan };
  frontier.sort(compareOrderedNodes);
  frontier = frontier.slice(0, Math.max(beamWidth, 1));
  const terminal: PlanNode[] = [];

  for (let step = 1; step < Math.max(stepCap, 1); step += 1) {
    if (execution.session.checkpoint())
      return { status: PlanBuildStatus.BudgetExceeded };
    const candidates: { readonly order: number; readonly node: PlanNode }[] = [];
    let expandedAny = false;
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
      const nextSeeds = generateActionSeeds(
        execution,
        node.game,
        perspective,
        config,
        seedCap,
      );
      if (execution.session.checkpoint())
        return { status: PlanBuildStatus.BudgetExceeded };
      if (nextSeeds.length === 0) {
        terminal.push(node);
        continue;
      }
      const nextPool = new TransitionCompilePool(
        execution,
        node.game,
        nextSeeds,
        config,
      );
      if (execution.session.checkpoint())
        return { status: PlanBuildStatus.BudgetExceeded };
      let nodeExpanded = false;
      for (const seed of nextSeeds) {
        if (execution.session.checkpoint())
          return { status: PlanBuildStatus.BudgetExceeded };
        const compiled = compileActionFromPool(
          execution,
          node.game,
          perspective,
          seed.action,
          nextPool,
        );
        if (compiled === undefined) continue;
        if (execution.session.checkpoint())
          return { status: PlanBuildStatus.BudgetExceeded };
        expansions += 1;
        if (expansions > expansionCap) {
          return { status: PlanBuildStatus.BudgetExceeded };
        }
        const [after, chunk] = compiled;
        const actions = [...node.actions, seed.action];
        const chunks = [...node.compiledChunks, chunk];
        const hash = exactSearchStateHash(after);
        const order = quickOrderScoreWithSearchHash(
          utilityContext,
          after,
          hash,
          node.goalFamily,
          actions.length,
        );
        if (execution.session.cancelled)
          return { status: PlanBuildStatus.BudgetExceeded };
        const existing = seen.get(hash);
        if (existing !== undefined && order <= existing) continue;
        seen.set(hash, order);
        candidates.push({
          order,
          node: {
            game: after,
            stateHash: hash,
            actions,
            compiledChunks: chunks,
            headUtility: node.headUtility,
            headFamily: node.headFamily,
            goalFamily: node.goalFamily,
          },
        });
        expandedAny = true;
        nodeExpanded = true;
      }
      if (!nodeExpanded) terminal.push(node);
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
  if (terminal.length === 0) return { status: PlanBuildStatus.NoPlan };
  const plans = terminal.map<TurnPlan>((node) => ({
    actions: [...node.actions],
    compiledChunks: node.compiledChunks.map((chunk) => chunk.slice()),
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
  }));
  if (execution.session.checkpoint()) return { status: PlanBuildStatus.BudgetExceeded };
  plans.sort((left, right) => -turnEngineComparePlans(left, right));
  return { status: "ok", plans };
}
