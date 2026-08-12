import type { AutomoveExecutionContext } from "../core/execution-context.js";
import { Color, MonKind, type Mana } from "../../api/types.js";
import {
  isMonFainted,
  manaEquals,
  manaScore,
  otherColor,
  type Event,
} from "../../engine/model/domain.js";
import type { MonsGame } from "../../engine/game/mons-game.js";
import {
  locationDistance,
  locationEquals,
  type Location,
} from "../../engine/board/geometry.js";
import { scoreForColor } from "../../engine/rules/legality.js";
import {
  saturatingScoreAdd,
  saturatingScoreMultiply,
  saturatingScoreSubtract,
} from "../core/score-math.js";
import { distanceToNearestPool } from "./action-rules.js";
import { opponentCanWinImmediately, ownDrainerSafetyScore } from "./evaluation.js";
import type { TurnAction } from "./model.js";

function movedActorTo(
  events: readonly Event[],
  actor: Location,
  to: Location,
): boolean {
  return events.some(
    (event) =>
      (event.kind === "mon-move" || event.kind === "demon-additional-step") &&
      locationEquals(event.from, actor) &&
      locationEquals(event.to, to),
  );
}

function attackEventsMatch(
  events: readonly Event[],
  actor: Location,
  target: Location,
  perspective: Color,
): boolean {
  return events.some((event) => {
    if (event.kind === "mystic-action" || event.kind === "demon-action") {
      return locationEquals(event.from, actor) && locationEquals(event.to, target);
    }
    return (
      event.kind === "mon-fainted" &&
      event.mon.color === otherColor(perspective) &&
      locationEquals(event.to, target)
    );
  });
}

function actorOrSuccessorCarries(
  after: MonsGame,
  perspective: Color,
  wanted: Mana,
): boolean {
  for (const [, item] of after.board.entries()) {
    if (
      item.kind === "mon-with-mana" &&
      item.mon.color === perspective &&
      !isMonFainted(item.mon) &&
      manaEquals(item.mana, wanted)
    ) {
      return true;
    }
  }
  return false;
}

export function transitionMatchesAction(
  execution: AutomoveExecutionContext,
  before: MonsGame,
  after: MonsGame,
  events: readonly Event[],
  perspective: Color,
  action: TurnAction,
): boolean {
  switch (action.kind) {
    case "walk":
      return (
        movedActorTo(events, action.actor, action.to) &&
        !eventsIncludeNonWalkAction(events)
      );
    case "attack":
      return attackEventsMatch(events, action.actor, action.target, perspective);
    case "spirit-shift":
      return events.some(
        (event) =>
          event.kind === "spirit-target-move" &&
          locationEquals(event.by, action.actor) &&
          locationEquals(event.from, action.target) &&
          locationEquals(event.to, action.destination),
      );
    case "bomb":
      return events.some(
        (event) =>
          event.kind === "bomb-attack" &&
          locationEquals(event.from, action.actor) &&
          locationEquals(event.to, action.target),
      );
    case "move-mana":
      return events.some(
        (event) =>
          event.kind === "mana-move" &&
          locationEquals(event.from, action.from) &&
          locationEquals(event.to, action.to),
      );
    case "score-carry":
      return (
        movedActorTo(events, action.actor, action.step) &&
        (events.some(
          (event) =>
            event.kind === "mana-scored" && manaEquals(event.mana, action.wanted),
        ) ||
          actorOrSuccessorCarries(after, perspective, action.wanted))
      );
    case "safety-retreat":
      return (
        movedActorTo(events, action.actor, action.to) &&
        ownDrainerSafetyScore(execution, after.board, perspective) >
          ownDrainerSafetyScore(execution, before.board, perspective)
      );
  }
}

export function transitionScore(
  execution: AutomoveExecutionContext,
  before: MonsGame,
  after: MonsGame,
  events: readonly Event[],
  perspective: Color,
  action: TurnAction,
): number {
  let score = saturatingScoreMultiply(
    saturatingScoreSubtract(
      scoreForColor(after, perspective),
      scoreForColor(before, perspective),
    ),
    500,
  );
  score = saturatingScoreAdd(
    score,
    saturatingScoreMultiply(
      ownDrainerSafetyScore(execution, after.board, perspective),
      180,
    ),
  );
  if (
    !opponentCanWinImmediately(execution, before, perspective) &&
    opponentCanWinImmediately(execution, after, perspective)
  ) {
    score = saturatingScoreSubtract(score, 2_200);
  }
  switch (action.kind) {
    case "walk":
      score = saturatingScoreAdd(
        score,
        saturatingScoreMultiply(locationDistance(action.actor, action.to), -20),
      );
      break;
    case "attack":
      if (eventsIncludeOpponentDrainerFaint(events, perspective))
        score = saturatingScoreAdd(score, 1_600);
      if (eventsIncludeAnyFaint(events, perspective))
        score = saturatingScoreAdd(score, 800);
      break;
    case "spirit-shift":
      if (events.some((event) => event.kind === "mana-scored"))
        score = saturatingScoreAdd(score, 1_000);
      if (events.some((event) => event.kind === "spirit-target-move"))
        score = saturatingScoreAdd(score, 600);
      break;
    case "bomb":
      if (eventsIncludeAnyFaint(events, perspective))
        score = saturatingScoreAdd(score, 1_000);
      break;
    case "move-mana":
      score = saturatingScoreAdd(
        score,
        saturatingScoreMultiply(
          saturatingScoreSubtract(
            distanceToNearestPool(action.from, perspective),
            distanceToNearestPool(action.to, perspective),
          ),
          160,
        ),
      );
      break;
    case "score-carry":
      score = saturatingScoreAdd(
        score,
        saturatingScoreMultiply(manaScore(action.wanted, perspective), 200),
      );
      break;
    case "safety-retreat":
      score = saturatingScoreAdd(
        score,
        saturatingScoreMultiply(
          ownDrainerSafetyScore(execution, after.board, perspective),
          260,
        ),
      );
      break;
  }
  return score;
}

function eventsIncludeNonWalkAction(events: readonly Event[]): boolean {
  return events.some(
    (event) =>
      event.kind === "mystic-action" ||
      event.kind === "demon-action" ||
      event.kind === "bomb-attack" ||
      event.kind === "spirit-target-move",
  );
}

function eventsIncludeAnyFaint(events: readonly Event[], perspective: Color): boolean {
  return events.some(
    (event) =>
      event.kind === "mon-fainted" && event.mon.color === otherColor(perspective),
  );
}

function eventsIncludeOpponentDrainerFaint(
  events: readonly Event[],
  perspective: Color,
): boolean {
  return events.some(
    (event) =>
      event.kind === "mon-fainted" &&
      event.mon.color === otherColor(perspective) &&
      event.mon.kind === MonKind.Drainer,
  );
}
