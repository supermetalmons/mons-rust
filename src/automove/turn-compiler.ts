import { Board } from "../engine/board.js";
import type { AutomoveExecutionContext } from "./execution-context.js";
import { MONS_MOVES_PER_TURN, TARGET_SCORE } from "../engine/config.js";
import {
  Color,
  Consumable,
  MonKind,
  isMonFainted,
  itemConsumable,
  itemMana,
  itemMon,
  manaEquals,
  manaScore,
  otherColor,
  type Event,
  type Input,
  type Item,
  type Mana,
} from "../engine/domain.js";
import { MonsGame, FOR_AUTOMOVE_START_INPUT_OPTIONS } from "../engine/game.js";
import {
  BOARD_SIZE,
  bombReachableLocations,
  demonReachableLocations,
  locationBetween,
  locationDistance,
  locationEquals,
  locationIndex,
  mysticReachableLocations,
  type Location,
} from "../engine/geometry.js";
import { scoreForColor } from "../engine/legality.js";
import {
  saturatingScoreAdd,
  saturatingScoreMultiply,
  saturatingScoreSubtract,
} from "./score-math.js";
import {
  applyInputsForSearchWithEvents,
  compareInputChains,
  enumerateLegalTransitionsWithPriority,
  type LegalInputTransition,
} from "./transitions.js";
import {
  activeTurnScoreWindow,
  opponentCanWinImmediately,
  ownDrainerSafetyScore,
} from "./turn-evaluation.js";
import {
  TurnPlanFamily,
  TURN_ENGINE_COMPILE_LIMIT_MAX,
  actionKey,
  type ActionSeed,
  type TurnAction,
  type TurnEngineConfig,
} from "./turn-types.js";

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

export function compileLimitForConfig(config: TurnEngineConfig): number {
  return Math.min(
    Math.max(Math.max(config.ownSeedCap, config.opponentSeedCap) * 12, 24),
    96,
  );
}

export function directInputsForAction(action: TurnAction): Input[] {
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

export function compileActionDirect(
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

export function bestTransitionForAction(
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
    const bestTransition =
      best === undefined ? undefined : transitions[best[1]];
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

export function compileActionFromPoolFallback(
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
  return compileActionFromPoolFallback(
    execution,
    game,
    perspective,
    action,
    pool,
  );
}

export function actionPriorityLocations(
  action: TurnAction,
): readonly Location[] {
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

export function movedActorTo(
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

export function attackEventsMatch(
  events: readonly Event[],
  actor: Location,
  target: Location,
  perspective: Color,
): boolean {
  return events.some((event) => {
    if (event.kind === "mystic-action" || event.kind === "demon-action") {
      return (
        locationEquals(event.from, actor) && locationEquals(event.to, target)
      );
    }
    return (
      event.kind === "mon-fainted" &&
      event.mon.color === otherColor(perspective) &&
      locationEquals(event.to, target)
    );
  });
}

export function actorOrSuccessorCarries(
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
      return attackEventsMatch(
        events,
        action.actor,
        action.target,
        perspective,
      );
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
            event.kind === "mana-scored" &&
            manaEquals(event.mana, action.wanted),
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

export function eventsIncludeNonWalkAction(events: readonly Event[]): boolean {
  return events.some(
    (event) =>
      event.kind === "mystic-action" ||
      event.kind === "demon-action" ||
      event.kind === "bomb-attack" ||
      event.kind === "spirit-target-move",
  );
}

export function eventsIncludeAnyFaint(
  events: readonly Event[],
  perspective: Color,
): boolean {
  return events.some(
    (event) =>
      event.kind === "mon-fainted" &&
      event.mon.color === otherColor(perspective),
  );
}

export function eventsIncludeOpponentDrainerFaint(
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

export function actorCanAttackFromItem(item: Item): boolean {
  const mon = itemMon(item);
  return (
    mon !== undefined &&
    (mon.kind === MonKind.Mystic || mon.kind === MonKind.Demon)
  );
}

export function actorCanBombFromItem(item: Item): boolean {
  const mon = itemMon(item);
  return (
    mon !== undefined &&
    !isMonFainted(mon) &&
    item.kind === "mon-with-consumable" &&
    item.consumable === Consumable.Bomb
  );
}

export function locationGuardedByAngel(
  angelLocation: Location | undefined,
  at: Location,
): boolean {
  return (
    angelLocation !== undefined && locationDistance(angelLocation, at) === 1
  );
}

export function demonAttackPathClear(
  board: Board,
  from: Location,
  target: Location,
): boolean {
  const middle = locationBetween(from, target);
  const square = board.squareAt(middle);
  return (
    board.get(middle) === undefined &&
    square.kind !== "supermana-base" &&
    square.kind !== "mon-base"
  );
}

export function actorCanAttackTargetNow(
  board: Board,
  actor: Location,
  target: Location,
  item: Item,
  perspective: Color,
): boolean {
  if (board.squareAt(actor).kind === "mon-base") return false;
  const targetItem = board.get(target);
  const targetMon = targetItem === undefined ? undefined : itemMon(targetItem);
  if (
    targetMon?.color !== otherColor(perspective) ||
    isMonFainted(targetMon) ||
    locationGuardedByAngel(
      board.findAwakeAngel(otherColor(perspective)),
      target,
    )
  ) {
    return false;
  }
  const mon = itemMon(item);
  if (mon?.kind === MonKind.Mystic) {
    return mysticReachableLocations(actor).some((at) =>
      locationEquals(at, target),
    );
  }
  return (
    mon?.kind === MonKind.Demon &&
    demonReachableLocations(actor).some((at) => locationEquals(at, target)) &&
    demonAttackPathClear(board, actor, target)
  );
}

export function actorCanBombTargetNow(
  board: Board,
  actor: Location,
  target: Location,
  item: Item,
  perspective: Color,
): boolean {
  if (!bombReachableLocations(actor).some((at) => locationEquals(at, target)))
    return false;
  if (!actorCanBombFromItem(item) || itemMon(item)?.color !== perspective)
    return false;
  const targetItem = board.get(target);
  const targetMon = targetItem === undefined ? undefined : itemMon(targetItem);
  return (
    targetMon?.color === otherColor(perspective) && !isMonFainted(targetMon)
  );
}

export function spiritDestinationAllowed(
  board: Board,
  targetItem: Item,
  destination: Location,
): boolean {
  const destinationItem = board.get(destination);
  const square = board.squareAt(destination);
  const targetMon = itemMon(targetItem);
  const targetMana = itemMana(targetItem);
  let validDestination: boolean;
  if (destinationItem === undefined) {
    validDestination = true;
  } else if (destinationItem.kind === "mon") {
    if (itemMon(targetItem) !== undefined) validDestination = false;
    else if (targetItem.kind === "mana") {
      validDestination =
        destinationItem.mon.kind === MonKind.Drainer &&
        !isMonFainted(destinationItem.mon);
    } else {
      validDestination =
        targetItem.kind === "consumable" &&
        targetItem.consumable === Consumable.BombOrPotion;
    }
  } else if (destinationItem.kind === "mana") {
    validDestination =
      targetMon?.kind === MonKind.Drainer && !isMonFainted(targetMon);
  } else if (
    destinationItem.kind === "mon-with-mana" ||
    destinationItem.kind === "mon-with-consumable"
  ) {
    validDestination =
      targetItem.kind === "consumable" &&
      targetItem.consumable === Consumable.BombOrPotion;
  } else if (destinationItem.consumable === Consumable.BombOrPotion) {
    validDestination = itemMon(targetItem) !== undefined;
  } else {
    validDestination = false;
  }
  if (!validDestination) return false;
  switch (square.kind) {
    case "regular":
    case "consumable-base":
    case "mana-base":
    case "mana-pool":
      return true;
    case "supermana-base":
      return (
        targetMana?.kind === "supermana" ||
        (targetMana === undefined && targetMon?.kind === MonKind.Drainer)
      );
    case "mon-base":
      return (
        targetMon?.kind === square.monKind &&
        targetMon.color === square.color &&
        targetMana === undefined &&
        itemConsumable(targetItem) === undefined
      );
  }
}

export function manaMoveDestinationAllowed(
  board: Board,
  destination: Location,
): boolean {
  const item = board.get(destination);
  const square = board.squareAt(destination);
  const ordinarySquare =
    square.kind === "regular" ||
    square.kind === "consumable-base" ||
    square.kind === "mana-base" ||
    square.kind === "mana-pool";
  if (item === undefined) return ordinarySquare;
  return (
    item.kind === "mon" &&
    ordinarySquare &&
    item.mon.kind === MonKind.Drainer &&
    !isMonFainted(item.mon)
  );
}

export function nearestWantedManaLocation(
  board: Board,
  wanted: Mana,
): Location | undefined {
  for (const [at, item] of board.entries()) {
    if (item.kind === "mana" && manaEquals(item.mana, wanted)) return at;
  }
  return undefined;
}

export function walkDestinationPlausible(
  board: Board,
  actor: Location,
  destination: Location,
): boolean {
  const actorItem = board.get(actor);
  const actorMon = actorItem === undefined ? undefined : itemMon(actorItem);
  if (actorMon === undefined) return false;
  const destinationItem = board.get(destination);
  if (destinationItem !== undefined && itemMon(destinationItem) !== undefined)
    return false;
  const square = board.squareAt(destination);
  switch (square.kind) {
    case "regular":
    case "consumable-base":
    case "mana-base":
    case "mana-pool":
      return true;
    case "supermana-base":
      return actorMon.kind === MonKind.Drainer;
    case "mon-base":
      return (
        actorMon.kind === square.monKind && actorMon.color === square.color
      );
  }
}

export function remainingMovesForColor(game: MonsGame, color: Color): number {
  return game.activeColor === color
    ? Math.max(MONS_MOVES_PER_TURN - game.monsMovesCount, 0)
    : MONS_MOVES_PER_TURN;
}

export function distanceToNearestPool(at: Location, color: Color): number {
  const row = color === Color.Black ? 0 : BOARD_SIZE - 1;
  return Math.min(
    locationDistance(at, { i: row, j: 0 }),
    locationDistance(at, { i: row, j: BOARD_SIZE - 1 }),
  );
}

export function opponentDrainerKillIsHighValue(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  target: Location,
): boolean {
  const opponent = otherColor(perspective);
  return (
    ownDrainerSafetyScore(execution, game.board, perspective) < 0 ||
    activeTurnScoreWindow(execution, game, opponent) > 0 ||
    scoreForColor(game, opponent) >= TARGET_SCORE - 2 ||
    game.board.get(target)?.kind === "mon-with-mana" ||
    distanceToNearestPool(target, opponent) <= 3
  );
}

export function actionIdentity(action: TurnAction): string {
  if (action.kind !== "score-carry") return actionKey(action);
  return `${actionKey(action)}:${action.wanted.kind}:${
    action.wanted.kind === "regular" ? action.wanted.color : "s"
  }`;
}
