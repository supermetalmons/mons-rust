import type { AutomoveExecutionContext } from "../core/execution-context.js";
import { Color } from "../../api/types.js";
import type { Input } from "../../engine/model/domain.js";
import { MonsGame } from "../../engine/game/mons-game.js";
import { FOR_AUTOMOVE_START_INPUT_OPTIONS } from "../../engine/game/input-support.js";
import { locationIndex, type Location } from "../../engine/board/geometry.js";
import { applyInputsForSearchWithEvents } from "../transitions/simulation.js";
import { compareInputChains } from "../transitions/order.js";
import { enumerateLegalTransitionsWithPriority } from "../transitions/enumerate.js";
import type { LegalInputTransition } from "../transitions/types.js";
import {
  TurnPlanFamily,
  TURN_ENGINE_COMPILE_LIMIT_MAX,
  type ActionSeed,
  type TurnAction,
  type TurnEngineConfig,
} from "./model.js";
import { transitionMatchesAction, transitionScore } from "./compiler-match.js";

export class TransitionCompilePool {
  readonly #execution: AutomoveExecutionContext;
  public transitions: LegalInputTransition[];
  public limit: number;
  public readonly priorityLocations: readonly Location[];

  public constructor(
    execution: AutomoveExecutionContext,
    game: MonsGame,
    seeds: readonly ActionSeed[],
    config: TurnEngineConfig,
  ) {
    this.#execution = execution;
    this.limit = compileLimitForConfig(config);
    if (execution.session.checkpoint()) {
      this.transitions = [];
      this.priorityLocations = [];
      return;
    }
    const seen = new Set<number>();
    const priorityLocations: Location[] = [];
    for (const seed of seeds) {
      for (const at of actionPriorityLocations(seed.action)) {
        const key = locationIndex(at);
        if (!seen.has(key)) {
          seen.add(key);
          priorityLocations.push(at);
        }
      }
    }
    this.priorityLocations = priorityLocations;
    this.transitions = enumerateLegalTransitionsWithPriority(
      execution,
      game,
      this.limit,
      FOR_AUTOMOVE_START_INPUT_OPTIONS,
      priorityLocations,
    );
    if (execution.session.checkpoint()) this.transitions = [];
  }

  public expand(game: MonsGame): boolean {
    if (
      this.#execution.session.checkpoint() ||
      this.transitions.length < this.limit ||
      this.limit >= TURN_ENGINE_COMPILE_LIMIT_MAX
    ) {
      return false;
    }
    const nextLimit = Math.min(this.limit * 2, TURN_ENGINE_COMPILE_LIMIT_MAX);
    if (nextLimit <= this.limit) return false;
    const transitions = enumerateLegalTransitionsWithPriority(
      this.#execution,
      game,
      nextLimit,
      FOR_AUTOMOVE_START_INPUT_OPTIONS,
      this.priorityLocations,
    );
    if (this.#execution.session.checkpoint()) return false;
    this.transitions = transitions;
    this.limit = nextLimit;
    return true;
  }
}

function compileLimitForConfig(config: TurnEngineConfig): number {
  return Math.min(
    Math.max(Math.max(config.ownSeedCap, config.opponentSeedCap) * 12, 24),
    96,
  );
}

function directInputsForAction(action: TurnAction): Input[] {
  switch (action.kind) {
    case "walk":
    case "safety-retreat":
      return [
        { kind: "location", location: { ...action.actor } },
        { kind: "location", location: { ...action.to } },
      ];
    case "attack":
    case "bomb":
      return [
        { kind: "location", location: { ...action.actor } },
        { kind: "location", location: { ...action.target } },
      ];
    case "spirit-shift":
      return [
        { kind: "location", location: { ...action.actor } },
        { kind: "location", location: { ...action.target } },
        { kind: "location", location: { ...action.destination } },
      ];
    case "move-mana":
      return [
        { kind: "location", location: { ...action.from } },
        { kind: "location", location: { ...action.to } },
      ];
    case "score-carry":
      return [
        { kind: "location", location: { ...action.actor } },
        { kind: "location", location: { ...action.step } },
      ];
  }
}

function compileActionDirect(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  action: TurnAction,
): readonly [MonsGame, Input[]] | undefined {
  if (execution.session.checkpoint()) return undefined;
  const inputs = directInputsForAction(action);
  const result = applyInputsForSearchWithEvents(game, inputs);
  if (result === undefined || execution.session.checkpoint()) return undefined;
  if (
    !transitionMatchesAction(
      execution,
      game,
      result.game,
      result.events,
      perspective,
      action,
    )
  ) {
    return undefined;
  }
  return [result.game, inputs];
}

function bestTransitionForAction(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  action: TurnAction,
  transitions: readonly LegalInputTransition[],
): readonly [number, number] | undefined {
  if (execution.session.checkpoint()) return undefined;
  let best: readonly [number, number] | undefined;
  for (let index = 0; index < transitions.length; index += 1) {
    if (execution.session.checkpoint()) return undefined;
    const transition = transitions[index];
    if (
      transition === undefined ||
      !transitionMatchesAction(
        execution,
        game,
        transition.game,
        transition.events,
        perspective,
        action,
      )
    ) {
      continue;
    }
    const score = transitionScore(
      execution,
      game,
      transition.game,
      transition.events,
      perspective,
      action,
    );
    const bestTransition = best === undefined ? undefined : transitions[best[1]];
    if (
      best === undefined ||
      score > best[0] ||
      (score === best[0] &&
        bestTransition !== undefined &&
        compareInputChains(transition.inputs, bestTransition.inputs) < 0)
    ) {
      best = [score, index];
    }
  }
  return best;
}

function compileActionFromPoolFallback(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  action: TurnAction,
  pool: TransitionCompilePool,
): readonly [MonsGame, Input[]] | undefined {
  let best = bestTransitionForAction(
    execution,
    game,
    perspective,
    action,
    pool.transitions,
  );
  if (best === undefined && pool.expand(game)) {
    best = bestTransitionForAction(
      execution,
      game,
      perspective,
      action,
      pool.transitions,
    );
  }
  if (best === undefined || execution.session.checkpoint()) return undefined;
  const transition = pool.transitions[best[1]];
  return transition === undefined
    ? undefined
    : [transition.game.fork(), transition.inputs.slice()];
}

export function compileActionFromPool(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  action: TurnAction,
  pool: TransitionCompilePool,
): readonly [MonsGame, Input[]] | undefined {
  const direct = compileActionDirect(execution, game, perspective, action);
  if (direct !== undefined) return direct;
  return execution.session.cancelled
    ? undefined
    : compileActionFromPoolFallback(execution, game, perspective, action, pool);
}

export function compileAction(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  action: TurnAction,
  config: TurnEngineConfig,
): readonly [MonsGame, Input[]] | undefined {
  const direct = compileActionDirect(execution, game, perspective, action);
  if (direct !== undefined) return direct;
  if (execution.session.cancelled) return undefined;
  const pool = new TransitionCompilePool(
    execution,
    game,
    [{ family: TurnPlanFamily.ManaTempo, action, priority: 0 }],
    config,
  );
  return compileActionFromPoolFallback(execution, game, perspective, action, pool);
}

function actionPriorityLocations(action: TurnAction): readonly Location[] {
  switch (action.kind) {
    case "walk":
    case "safety-retreat":
      return [action.actor, action.to];
    case "attack":
    case "bomb":
      return [action.actor, action.target];
    case "spirit-shift":
      return [action.actor, action.target, action.destination];
    case "move-mana":
      return [action.from, action.to];
    case "score-carry":
      return [action.actor, action.step];
  }
}
