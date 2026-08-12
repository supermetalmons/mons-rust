import { Color, MonKind, type Mana } from "../../api/types.js";
import {
  inputChainKey,
  isMonFainted,
  itemMon,
  manaEquals,
  otherColor,
  type Event,
  type Input,
} from "../../engine/model/domain.js";
import { FOR_AUTOMOVE_START_INPUT_OPTIONS } from "../../engine/game/input-support.js";
import { MonsGame } from "../../engine/game/mons-game.js";
import type { Location } from "../../engine/board/geometry.js";
import { canAttackOpponentDrainerThisTurn } from "../exact/turn-opportunity.js";
import { exactSecureSpecificManaPathFrom } from "../exact/secure-mana.js";
import { exactSearchStateHash } from "../exact/hash.js";
import type { AutomoveExecutionContext } from "../core/execution-context.js";
import { Hash64Set } from "../core/hash64.js";
import { applyInputsForSearchWithEvents } from "../transitions/simulation.js";
import { compareInputChains } from "../transitions/order.js";
import { enumerateLegalTransitionsLexicographicBounded } from "../transitions/enumerate.js";
import type { LegalInputTransition } from "../transitions/types.js";
import {
  attacksDrainer,
  safeCarrierForMana,
  spiritManaSetup,
  spiritMovesManaToward,
} from "./observations.js";
import { type SearchConfig } from "./types.js";
import {
  isOwnDrainerVulnerable,
  potentialDrainerAttackerLocations,
} from "./vulnerability.js";

const FORCED_ATTACK_FAST_CANDIDATES = 4;
const FORCED_ATTACK_NORMAL_CANDIDATES = 6;
const FORCED_ATTACK_FAST_NODE_BUDGET = 600;
const FORCED_ATTACK_NORMAL_NODE_BUDGET = 1_800;
const FORCED_ATTACK_FAST_ENUM_LIMIT = 220;
const FORCED_ATTACK_NORMAL_ENUM_LIMIT = 280;

export function appendUniqueTransitions(
  target: LegalInputTransition[],
  additions: readonly LegalInputTransition[],
): void {
  const seen = new Set(target.map((transition) => inputChainKey(transition.inputs)));
  for (const transition of additions) {
    if (seen.has(inputChainKey(transition.inputs))) continue;
    seen.add(inputChainKey(transition.inputs));
    target.push(transition);
  }
}

export function forcedAttackCandidatesLimit(config: SearchConfig): number {
  return config.budget.depth >= 3
    ? FORCED_ATTACK_NORMAL_CANDIDATES
    : FORCED_ATTACK_FAST_CANDIDATES;
}

function forcedAttackNodeBudget(config: SearchConfig): number {
  return config.budget.depth >= 3
    ? FORCED_ATTACK_NORMAL_NODE_BUDGET
    : FORCED_ATTACK_FAST_NODE_BUDGET;
}

function forcedAttackEnumLimit(config: SearchConfig): number {
  return config.budget.depth >= 3
    ? FORCED_ATTACK_NORMAL_ENUM_LIMIT
    : FORCED_ATTACK_FAST_ENUM_LIMIT;
}

export function spiritSetupFallbackCandidatesLimit(config: SearchConfig): number {
  return config.budget.depth >= 3 ? 8 : 4;
}

function spiritSetupFallbackEnumLimit(config: SearchConfig): number {
  return config.budget.depth >= 3 ? 256 : 128;
}

export function safeDrainerPickupFallbackCandidatesLimit(config: SearchConfig): number {
  return config.budget.depth >= 3 ? 8 : 4;
}

export function drainerSafetyFallbackCandidatesLimit(config: SearchConfig): number {
  return config.budget.depth >= 3 ? 8 : 4;
}

function drainerSafetyFallbackEnumLimit(config: SearchConfig): number {
  return config.budget.depth >= 3 ? 192 : 96;
}

export function genericRootFallbackEnumLimit(config: SearchConfig): number {
  return config.budget.depth >= 3 ? 24 : 12;
}

function awakeMonLocations(
  game: MonsGame,
  perspective: Color,
  kind?: MonKind,
): Location[] {
  const locations: Location[] = [];
  for (const [location, item] of game.board.entries()) {
    const mon = itemMon(item);
    if (
      mon?.color === perspective &&
      !isMonFainted(mon) &&
      (kind === undefined || mon.kind === kind)
    ) {
      locations.push(location);
    }
  }
  return locations;
}

export function hasSpiritScoringManaSetup(
  context: AutomoveExecutionContext,
  after: MonsGame,
  events: readonly Event[],
  perspective: Color,
): boolean {
  return (
    spiritMovesManaToward(
      events,
      perspective,
      (mana) => mana.kind === "regular" && mana.color === perspective,
    ) ||
    spiritManaSetup(context, after, events, perspective, {
      kind: "supermana",
    }) ||
    spiritManaSetup(context, after, events, perspective, {
      kind: "regular",
      color: otherColor(perspective),
    })
  );
}

export function collectTargetedSpiritSetupInputs(
  context: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: SearchConfig,
  maxCandidates: number,
): LegalInputTransition[] {
  if (context.session.checkpoint() || !game.playerCanUseAction()) return [];
  const spiritLocations = awakeMonLocations(game, perspective, MonKind.Spirit);
  if (spiritLocations.length === 0) return [];
  const limit = Math.max(1, maxCandidates);
  const collected: LegalInputTransition[] = [];
  for (const spiritLocation of spiritLocations) {
    if (context.session.checkpoint()) return [];
    if (collected.length >= limit) break;
    const transitions = enumerateLegalTransitionsLexicographicBounded(
      context,
      game,
      spiritSetupFallbackEnumLimit(config),
      FOR_AUTOMOVE_START_INPUT_OPTIONS,
      [spiritLocation],
    );
    if (context.session.cancelled) return [];
    for (const transition of transitions) {
      if (context.session.checkpoint()) return [];
      if (collected.length >= limit) break;
      if (
        hasSpiritScoringManaSetup(
          context,
          transition.game,
          transition.events,
          perspective,
        )
      ) {
        collected.push(transition);
      }
    }
  }
  return collected;
}

export function collectTargetedDrainerSafetyInputs(
  context: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: SearchConfig,
  maxCandidates: number,
): LegalInputTransition[] {
  if (context.session.checkpoint()) return [];
  const actorLocations = awakeMonLocations(game, perspective);
  if (actorLocations.length === 0) return [];
  const limit = Math.max(1, maxCandidates);
  const collected: LegalInputTransition[] = [];
  const seen = new Set<string>();
  for (const actorLocation of actorLocations) {
    if (context.session.checkpoint()) return [];
    if (collected.length >= limit) break;
    const transitions = enumerateLegalTransitionsLexicographicBounded(
      context,
      game,
      drainerSafetyFallbackEnumLimit(config),
      FOR_AUTOMOVE_START_INPUT_OPTIONS,
      [actorLocation],
    );
    if (context.session.cancelled) return [];
    for (const transition of transitions) {
      if (context.session.checkpoint()) return [];
      if (collected.length >= limit) break;
      if (isOwnDrainerVulnerable(context, transition.game, perspective)) continue;
      const key = inputChainKey(transition.inputs);
      if (!seen.has(key)) {
        seen.add(key);
        collected.push(transition);
      }
    }
  }
  collected.sort((left, right) => compareInputChains(left.inputs, right.inputs));
  return collected;
}

function eventsPickupWantedMana(events: readonly Event[], wanted: Mana): boolean {
  return events.some(
    (event) => event.kind === "pickup-mana" && manaEquals(event.mana, wanted),
  );
}

function eventsScoreWantedMana(events: readonly Event[], wanted: Mana): boolean {
  return events.some(
    (event) => event.kind === "mana-scored" && manaEquals(event.mana, wanted),
  );
}

export function transitionHasSafeDrainerPickup(
  context: AutomoveExecutionContext,
  transition: LegalInputTransition,
  perspective: Color,
  wanted: Mana,
): boolean {
  return (
    eventsPickupWantedMana(transition.events, wanted) &&
    safeCarrierForMana(context, transition.game, perspective, wanted)
  );
}

export function collectTargetedSafeDrainerPickupInputs(
  context: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  maxCandidates: number,
  wanted: Mana,
): LegalInputTransition[] {
  if (context.session.checkpoint() || !game.playerCanMoveMon()) return [];
  const drainerLocations = awakeMonLocations(game, perspective, MonKind.Drainer);
  if (drainerLocations.length === 0) return [];
  const limit = Math.max(1, maxCandidates);
  const collected: LegalInputTransition[] = [];
  const seen = new Set<string>();
  for (const drainerLocation of drainerLocations) {
    if (context.session.checkpoint()) return [];
    if (collected.length >= limit) break;
    const path = exactSecureSpecificManaPathFrom(
      context,
      game,
      perspective,
      drainerLocation,
      wanted,
    );
    if (context.session.cancelled) return [];
    if (path === undefined || path.length === 0) continue;
    const inputs: Input[] = [
      {
        kind: "location",
        location: { i: drainerLocation.i, j: drainerLocation.j },
      },
      ...path.map((location): Input => ({
        kind: "location",
        location: { i: location.i, j: location.j },
      })),
    ];
    const applied = applyInputsForSearchWithEvents(game, inputs);
    if (applied === undefined) continue;
    if (
      eventsPickupWantedMana(applied.events, wanted) &&
      (safeCarrierForMana(context, applied.game, perspective, wanted) ||
        eventsScoreWantedMana(applied.events, wanted))
    ) {
      const key = inputChainKey(inputs);
      if (!seen.has(key)) {
        seen.add(key);
        collected.push({
          inputs,
          game: applied.game,
          events: applied.events,
        });
      }
    }
  }
  collected.sort((left, right) => compareInputChains(left.inputs, right.inputs));
  return collected;
}

export function canAttemptForcedDrainerAttackFallback(
  game: MonsGame,
  perspective: Color,
): boolean {
  return (
    game.playerCanMoveMon() &&
    potentialDrainerAttackerLocations(game, perspective).length > 0
  );
}

function canAttackOpponentDrainerBeforeTurnEnds(
  context: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  budget: { remaining: number },
  memo: Hash64Set,
): boolean {
  if (
    context.session.checkpoint() ||
    game.activeColor !== perspective ||
    budget.remaining === 0
  ) {
    return false;
  }
  const stateHash = exactSearchStateHash(game);
  if (memo.has(stateHash)) return true;
  budget.remaining = Math.max(0, budget.remaining - 1);
  const canAttack = canAttackOpponentDrainerThisTurn(context, game, perspective);
  if (canAttack && context.session.cacheWriteAllowed) memo.add(stateHash);
  return canAttack;
}

export function collectDrainerAttackInputs(
  context: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: SearchConfig,
  maxCandidates: number,
  targeted: boolean,
): LegalInputTransition[] {
  if (context.session.checkpoint()) return [];
  const attackerLocations = targeted
    ? potentialDrainerAttackerLocations(game, perspective)
    : undefined;
  if (targeted && attackerLocations?.length === 0) return [];
  const multiplier = targeted ? 2 : 1;
  const enumLimit = forcedAttackEnumLimit(config) * multiplier;
  const budget = {
    remaining: forcedAttackNodeBudget(config) * multiplier,
  };
  const memo = new Hash64Set(Math.max(1, budget.remaining));
  const transitions = enumerateLegalTransitionsLexicographicBounded(
    context,
    game,
    enumLimit,
    FOR_AUTOMOVE_START_INPUT_OPTIONS,
    attackerLocations,
  );
  if (context.session.checkpoint()) return [];
  const limit = Math.max(1, maxCandidates);
  const collected: LegalInputTransition[] = [];
  for (const transition of transitions) {
    if (context.session.checkpoint()) return [];
    if (collected.length >= limit) break;
    if (attacksDrainer(transition.events, perspective)) {
      collected.push(transition);
      continue;
    }
    if (
      transition.game.activeColor === perspective &&
      canAttackOpponentDrainerBeforeTurnEnds(
        context,
        transition.game,
        perspective,
        budget,
        memo,
      )
    ) {
      collected.push(transition);
    }
  }
  return collected;
}
