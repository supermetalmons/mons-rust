import { MONS_MOVES_PER_TURN } from "../../engine/board/config.js";
import { Color, MonKind, type Mana } from "../../api/types.js";
import {
  isMonFainted,
  itemMon,
  manaEquals,
  otherColor,
  type Event,
} from "../../engine/model/domain.js";
import { MonsGame } from "../../engine/game/mons-game.js";
import {
  type Location,
  locationDistance,
  locationEquals,
} from "../../engine/board/geometry.js";
import { scoreForColor } from "../../engine/rules/legality.js";
import { saturatingScoreAdd, saturatingScoreSubtract } from "../core/score-math.js";
import { exactBoardHash } from "../exact/hash.js";
import { exactStrategicAnalysis } from "../exact/strategic.js";
import { exactTurnSummary } from "../exact/turn-opportunity.js";
import { isDrainerExactlySafeNextTurnOnBoardWithHash } from "../exact/drainer-safety.js";
import type { ExactTurnSummary } from "../exact/types.js";
import type { AutomoveExecutionContext } from "../core/execution-context.js";
import { hasAwakeSpiritOnBase } from "../config/types.js";
import { hasMaterialEvent } from "../transitions/event-classification.js";
import type { LegalInputTransition } from "../transitions/types.js";
import {
  approximateSameTurnScoreWindowValue,
  distanceToAnyPoolStepsForEfficiency as distanceToAnyPoolSteps,
  distanceToColorPoolStepsForEfficiency as distanceToColorPool,
} from "./move-efficiency.js";
import {
  UNKNOWN_PROGRESS_STEPS,
  type MoveClassFlags,
  type SearchConfig,
} from "./types.js";
import {
  approximateCanAttackOpponentDrainerThisTurn,
  isOwnDrainerVulnerable,
} from "./vulnerability.js";

function carrierSnapshot(game: MonsGame, color: Color): readonly [number, number] {
  let count = 0;
  let bestSteps = UNKNOWN_PROGRESS_STEPS;
  for (const [location, item] of game.board.entries()) {
    if (
      item.kind !== "mon-with-mana" ||
      item.mon.color !== color ||
      isMonFainted(item.mon)
    ) {
      continue;
    }
    count += 1;
    const edgeDistance = Math.max(
      Math.min(location.i, 10 - location.i),
      Math.min(location.j, 10 - location.j),
    );
    bestSteps = Math.min(bestSteps, edgeDistance);
  }
  return [count, bestSteps];
}

function hasCarrierProgress(before: MonsGame, after: MonsGame, color: Color): boolean {
  const [beforeCount, beforeSteps] = carrierSnapshot(before, color);
  const [afterCount, afterSteps] = carrierSnapshot(after, color);
  return (
    afterCount > beforeCount ||
    (afterSteps < beforeSteps && afterSteps < UNKNOWN_PROGRESS_STEPS)
  );
}

function spiritBase(game: MonsGame, color: Color): Location {
  return game.board.base({ kind: MonKind.Spirit, color, cooldown: 0 });
}

function hasAwakeSpiritOffBase(game: MonsGame, color: Color): boolean {
  const base = spiritBase(game, color);
  for (const [location, item] of game.board.entries()) {
    const mon = itemMon(item);
    if (
      !locationEquals(location, base) &&
      mon?.kind === MonKind.Spirit &&
      mon.color === color &&
      !isMonFainted(mon)
    ) {
      return true;
    }
  }
  return false;
}

export function hasSpiritDevelopment(
  before: MonsGame,
  after: MonsGame,
  color: Color,
  events: readonly Event[],
): boolean {
  return (
    events.some((event) => event.kind === "spirit-target-move") ||
    (hasAwakeSpiritOnBase(before, color) && hasAwakeSpiritOffBase(after, color))
  );
}

function isImmediateScore(
  before: MonsGame,
  after: MonsGame,
  actorColor: Color,
  events: readonly Event[],
): boolean {
  return (
    events.some((event) => event.kind === "mana-scored") ||
    scoreForColor(after, actorColor) > scoreForColor(before, actorColor)
  );
}

export function attacksDrainer(events: readonly Event[], actorColor: Color): boolean {
  return events.some(
    (event) =>
      event.kind === "mon-fainted" &&
      event.mon.kind === MonKind.Drainer &&
      event.mon.color === otherColor(actorColor),
  );
}

export function classifyTransition(
  context: AutomoveExecutionContext,
  before: MonsGame,
  transition: LegalInputTransition,
  actorColor: Color,
  vulnerableBefore = isOwnDrainerVulnerable(context, before, actorColor),
  vulnerableAfter = isOwnDrainerVulnerable(context, transition.game, actorColor),
): MoveClassFlags {
  const immediateScore = isImmediateScore(
    before,
    transition.game,
    actorColor,
    transition.events,
  );
  const drainerAttack = attacksDrainer(transition.events, actorColor);
  const drainerSafetyRecover = vulnerableBefore && !vulnerableAfter;
  const carrierProgress = hasCarrierProgress(before, transition.game, actorColor);
  const material = hasMaterialEvent(transition.events);
  const spiritDevelopment = hasSpiritDevelopment(
    before,
    transition.game,
    actorColor,
    transition.events,
  );
  return {
    immediateScore,
    drainerAttack,
    drainerSafetyRecover,
    carrierProgress,
    material,
    quiet:
      !immediateScore &&
      !drainerAttack &&
      !drainerSafetyRecover &&
      !carrierProgress &&
      !material &&
      !spiritDevelopment,
  };
}

export function orderingEventBonus(
  actorColor: Color,
  perspective: Color,
  events: readonly Event[],
): number {
  let bonus = 0;
  for (const event of events) {
    switch (event.kind) {
      case "mana-scored":
        bonus += actorColor === perspective ? 780 : -780;
        break;
      case "pickup-mana":
        bonus += actorColor === perspective ? 230 : -230;
        break;
      case "mon-fainted":
        bonus += event.mon.color === perspective ? -360 : 360;
        break;
      case "use-potion":
        bonus += actorColor === perspective ? -80 : 80;
        break;
      case "pickup-bomb":
      case "pickup-potion":
        bonus += actorColor === perspective ? 45 : -45;
        break;
      default:
        break;
    }
  }
  return bonus;
}

export function scoresMana(
  events: readonly Event[],
  predicate: (mana: Mana) => boolean,
): boolean {
  return events.some((event) => event.kind === "mana-scored" && predicate(event.mana));
}

export function picksUpMana(
  events: readonly Event[],
  predicate: (mana: Mana) => boolean,
): boolean {
  return events.some((event) => event.kind === "pickup-mana" && predicate(event.mana));
}

export function manaMovedToward(
  events: readonly Event[],
  color: Color,
  predicate: (mana: Mana) => boolean,
): boolean {
  return events.some(
    (event) =>
      event.kind === "mana-move" &&
      predicate(event.mana) &&
      distanceToColorPool(event.to, color) < distanceToColorPool(event.from, color),
  );
}

export function spiritMovesManaToward(
  events: readonly Event[],
  color: Color,
  predicate: (mana: Mana) => boolean,
): boolean {
  return events.some(
    (event) =>
      event.kind === "spirit-target-move" &&
      event.item.kind === "mana" &&
      predicate(event.item.mana) &&
      distanceToColorPool(event.to, color) < distanceToColorPool(event.from, color),
  );
}

export function safeCarrierForMana(
  context: AutomoveExecutionContext,
  game: MonsGame,
  color: Color,
  desiredMana: Mana,
): boolean {
  const boardHash = exactBoardHash(game.board);
  for (const [location, item] of game.board.entries()) {
    if (
      item.kind === "mon-with-mana" &&
      item.mon.kind === MonKind.Drainer &&
      item.mon.color === color &&
      !isMonFainted(item.mon) &&
      manaEquals(item.mana, desiredMana)
    ) {
      return isDrainerExactlySafeNextTurnOnBoardWithHash(
        context,
        game.board,
        boardHash,
        color,
        location,
      );
    }
  }
  return false;
}

export function spiritManaSetup(
  context: AutomoveExecutionContext,
  game: MonsGame,
  events: readonly Event[],
  color: Color,
  mana: Mana,
): boolean {
  return (
    spiritMovesManaToward(events, color, (candidate) => manaEquals(candidate, mana)) ||
    (events.some(
      (event) =>
        event.kind === "spirit-target-move" &&
        event.item.kind === "mana" &&
        manaEquals(event.item.mana, mana),
    ) &&
      safeCarrierForMana(context, game, color, mana))
  );
}

function approximateSpecificManaProgressSteps(
  game: MonsGame,
  color: Color,
  wanted: Mana,
): number | undefined {
  let bestSteps: number | undefined;
  for (const [drainerLocation, item] of game.board.entries()) {
    if (
      item.kind === "mon-with-mana" &&
      item.mon.color === color &&
      item.mon.kind === MonKind.Drainer &&
      !isMonFainted(item.mon) &&
      manaEquals(item.mana, wanted)
    ) {
      bestSteps = 0;
      continue;
    }
    if (item.kind !== "mon" && item.kind !== "mon-with-consumable") continue;
    if (
      item.mon.color !== color ||
      item.mon.kind !== MonKind.Drainer ||
      isMonFainted(item.mon)
    ) {
      continue;
    }
    for (const [manaLocation, manaItem] of game.board.entries()) {
      if (manaItem.kind !== "mana" || !manaEquals(manaItem.mana, wanted)) {
        continue;
      }
      const steps = locationDistance(drainerLocation, manaLocation);
      bestSteps = bestSteps === undefined ? steps : Math.min(bestSteps, steps);
    }
  }
  return bestSteps;
}

function approximateBestCarrierSteps(game: MonsGame, color: Color): number | undefined {
  let bestSteps: number | undefined;
  for (const [at, item] of game.board.entries()) {
    if (
      item.kind !== "mon-with-mana" ||
      item.mon.color !== color ||
      isMonFainted(item.mon)
    ) {
      continue;
    }
    const steps = Math.max(0, distanceToAnyPoolSteps(at) - 1);
    bestSteps = bestSteps === undefined ? steps : Math.min(bestSteps, steps);
  }
  return bestSteps;
}

export function approximateActiveTurnSummary(
  context: AutomoveExecutionContext,
  game: MonsGame,
  color: Color,
  allowExactStrategic = false,
): ExactTurnSummary {
  const strategic = allowExactStrategic
    ? exactStrategicAnalysis(context, game).colorSummary(color)
    : undefined;
  const remainingMoves = Math.max(0, MONS_MOVES_PER_TURN - game.monsMovesCount);
  const safeSupermanaProgressSteps = approximateSpecificManaProgressSteps(game, color, {
    kind: "supermana",
  });
  const safeOpponentManaProgressSteps = approximateSpecificManaProgressSteps(
    game,
    color,
    { kind: "regular", color: otherColor(color) },
  );
  return {
    canAttackOpponentDrainer: false,
    safeSupermanaProgress:
      safeSupermanaProgressSteps !== undefined &&
      safeSupermanaProgressSteps <= remainingMoves,
    safeSupermanaProgressSteps,
    safeOpponentManaProgress:
      safeOpponentManaProgressSteps !== undefined &&
      safeOpponentManaProgressSteps <= remainingMoves,
    safeOpponentManaProgressSteps,
    spiritAssistedSupermanaProgress: false,
    spiritAssistedOpponentManaProgress: false,
    spiritAssistedScore: false,
    spiritAssistedDenial: false,
    sameTurnScoreWindowValue:
      strategic?.immediateWindow.bestScore ??
      approximateSameTurnScoreWindowValue(game, color),
    scorePathBestSteps:
      strategic?.scorePathWindow.bestSteps ?? approximateBestCarrierSteps(game, color),
  };
}

export function hasProTacticalPotential(
  context: AutomoveExecutionContext,
  game: MonsGame,
): boolean {
  const activeColor = game.activeColor;
  const summary = approximateActiveTurnSummary(context, game, activeColor);
  return (
    summary.sameTurnScoreWindowValue > 0 ||
    approximateCanAttackOpponentDrainerThisTurn(game, activeColor) ||
    summary.safeSupermanaProgress ||
    summary.safeOpponentManaProgress
  );
}

export function rootTurnSummary(
  context: AutomoveExecutionContext,
  game: MonsGame,
  color: Color,
  exactRootEnabled: boolean,
  exactStaticEnabled: boolean,
): ExactTurnSummary | undefined {
  if (game.activeColor !== color) return undefined;
  return exactRootEnabled
    ? exactTurnSummary(context, game, color)
    : approximateActiveTurnSummary(context, game, color, exactStaticEnabled);
}

export function liveSpiritSetupGain(
  summary: ExactTurnSummary | undefined,
  spiritDevelopment: boolean,
  spiritSameTurnScoreSetupNow: boolean,
  spiritOwnManaSetupNow: boolean,
): number {
  let gain = 0;
  if (spiritSameTurnScoreSetupNow) gain += 80;
  if (spiritOwnManaSetupNow) gain += 48;
  if (spiritDevelopment) gain += 24;
  if (summary?.spiritAssistedScore) gain += 72;
  if (summary?.spiritAssistedDenial) gain += 40;
  if (summary?.spiritAssistedSupermanaProgress) gain += 36;
  if (summary?.spiritAssistedOpponentManaProgress) gain += 36;
  gain += Math.max(0, summary?.sameTurnScoreWindowValue ?? 0) * 20;
  if (summary?.safeSupermanaProgressSteps !== undefined) {
    gain += Math.max(0, 12 - Math.min(12, summary.safeSupermanaProgressSteps)) * 4;
  }
  if (summary?.safeOpponentManaProgressSteps !== undefined) {
    gain += Math.max(0, 12 - Math.min(12, summary.safeOpponentManaProgressSteps)) * 4;
  }
  if (summary?.scorePathBestSteps !== undefined) {
    gain += Math.max(0, 8 - Math.min(8, summary.scorePathBestSteps)) * 3;
  }
  return gain;
}

function rootProgressBonus(steps: number, perStep: number): number {
  if (steps >= UNKNOWN_PROGRESS_STEPS || perStep <= 0) return 0;
  const clampedSteps = Math.max(0, Math.min(MONS_MOVES_PER_TURN, steps));
  return (MONS_MOVES_PER_TURN - clampedSteps) * perStep;
}

export function rootSoftPriority(
  config: SearchConfig,
  values: {
    readonly supermanaProgress: boolean;
    readonly opponentManaProgress: boolean;
    readonly safeSupermanaProgressSteps: number;
    readonly safeOpponentManaProgressSteps: number;
    readonly scoresSupermanaThisTurn: boolean;
    readonly scoresOpponentManaThisTurn: boolean;
    readonly ownDrainerVulnerable: boolean;
    readonly manaHandoffToOpponent: boolean;
    readonly hasRoundtrip: boolean;
  },
): number {
  let score = 0;
  if (values.scoresSupermanaThisTurn) {
    score = saturatingScoreAdd(
      score,
      Math.max(0, config.evaluation.supermanaScoreBonus),
    );
  } else if (values.supermanaProgress && !values.ownDrainerVulnerable) {
    score = saturatingScoreAdd(
      score,
      Math.max(0, config.evaluation.supermanaProgressBonus),
    );
    score = saturatingScoreAdd(
      score,
      rootProgressBonus(values.safeSupermanaProgressSteps, 8),
    );
  }
  if (values.scoresOpponentManaThisTurn) {
    score = saturatingScoreAdd(
      score,
      Math.max(0, config.evaluation.opponentManaScoreBonus),
    );
  } else if (values.opponentManaProgress && !values.ownDrainerVulnerable) {
    score = saturatingScoreAdd(
      score,
      Math.max(0, config.evaluation.opponentManaProgressBonus),
    );
    score = saturatingScoreAdd(
      score,
      rootProgressBonus(values.safeOpponentManaProgressSteps, 6),
    );
  }
  if (values.manaHandoffToOpponent) {
    score = saturatingScoreSubtract(
      score,
      Math.max(0, config.evaluation.softManaHandoffPenalty),
    );
  }
  if (values.hasRoundtrip) {
    score = saturatingScoreSubtract(
      score,
      Math.max(0, config.evaluation.softRoundtripPenalty),
    );
  }
  return score;
}
