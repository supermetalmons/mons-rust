import {
  type Input,
  type Item,
  type NextInput,
  type NextInputKind,
  type Square,
} from "../model/domain.js";
import { locationDistance, locationEquals, type Location } from "../board/geometry.js";

export type SuggestedStartInputOptions = {
  readonly includeManaStartsWithPotionAction: boolean;
};

export const DEFAULT_SUGGESTED_START_INPUT_OPTIONS: SuggestedStartInputOptions =
  Object.freeze({
    includeManaStartsWithPotionAction: false,
  });

export const FOR_AUTOMOVE_START_INPUT_OPTIONS: SuggestedStartInputOptions =
  Object.freeze({
    includeManaStartsWithPotionAction: true,
  });

export function nextInput(
  input: Input,
  kind: NextInputKind,
  actorMonItem?: Item,
): NextInput {
  return actorMonItem === undefined ? { input, kind } : { input, kind, actorMonItem };
}

export function firstOptionFromEachKindGroup(
  options: readonly NextInput[],
): NextInput[] {
  const result: NextInput[] = [];
  let previousKind: NextInput["kind"] | undefined;
  for (const option of options) {
    if (option.kind === previousKind) continue;
    result.push(option);
    previousKind = option.kind;
  }
  return result;
}

export function regularSquareForMovement(square: Square): boolean {
  switch (square.kind) {
    case "regular":
    case "consumable-base":
    case "mana-base":
    case "mana-pool":
      return true;
    case "supermana-base":
    case "mon-base":
      return false;
  }
}

export function isLocationGuardedByAngelLocation(
  angelLocation: Location | undefined,
  targetLocation: Location,
): boolean {
  return (
    angelLocation !== undefined && locationDistance(angelLocation, targetLocation) === 1
  );
}

export function nextInputsFromLocations(
  locations: readonly Location[],
  kind: NextInputKind,
  specific: Location | undefined,
  filter: (location: Location) => boolean,
): NextInput[] {
  if (specific !== undefined) {
    for (const candidate of locations) {
      if (!locationEquals(candidate, specific)) continue;
      return filter(specific)
        ? [nextInput({ kind: "location", location: specific }, kind)]
        : [];
    }
    return [];
  }
  const options: NextInput[] = [];
  for (const at of locations) {
    if (!filter(at)) continue;
    options.push(nextInput({ kind: "location", location: at }, kind));
  }
  return options;
}
