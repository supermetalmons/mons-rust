import type { Event } from "../../engine/model/domain.js";

const MATERIAL_EVENT_KINDS: readonly Event["kind"][] = Object.freeze([
  "mana-scored",
  "pickup-mana",
  "mon-fainted",
  "use-potion",
  "pickup-bomb",
  "pickup-potion",
  "bomb-attack",
  "bomb-explosion",
]);

const QUIESCENCE_TACTICAL_EVENT_KINDS: readonly Event["kind"][] = Object.freeze([
  "mana-scored",
  "pickup-mana",
  "mon-fainted",
  "use-potion",
  "bomb-attack",
  "bomb-explosion",
  "spirit-target-move",
  "supermana-back-to-base",
]);

function hasEventKind(
  events: readonly Event[],
  kinds: readonly Event["kind"][],
): boolean {
  return events.some((event) => kinds.includes(event.kind));
}

export function hasMaterialEvent(events: readonly Event[]): boolean {
  return hasEventKind(events, MATERIAL_EVENT_KINDS);
}

export function isQuiescenceTacticalTransition(events: readonly Event[]): boolean {
  return hasEventKind(events, QUIESCENCE_TACTICAL_EVENT_KINDS);
}
