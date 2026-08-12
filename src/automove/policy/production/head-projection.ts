import type { Color } from "../../../api/types.js";
import type { MonsGame } from "../../../engine/game/mons-game.js";
import type { AutomoveExecutionContext } from "../../core/execution-context.js";
import { hasRoundtripMonMove, manaHandoffPenalty } from "../../root/move-efficiency.js";
import { turnEngineSelectedOverrideUtility } from "../reply-risk/projection.js";
import type { ReplyRiskHooks } from "../reply-risk/types.js";
import { rootFamily } from "../../root/family.js";
import type { EvaluatedRoot } from "../../root/types.js";
import type { AutomoveConfig } from "../../config/types.js";
import type { TurnPlan } from "../../turn/model.js";
import { applyInputsForSearchWithEvents } from "../../transitions/simulation.js";
import { ownDrainerVulnerableNextTurn } from "./shared.js";

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
  return nearCompletion && !handoff && !hasRoundtripMonMove(events) && !vulnerable;
}
