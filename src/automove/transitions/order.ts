import {
  MODIFIER_RANK,
  type Input,
  type NextInput,
} from "../../engine/model/domain.js";
import type { Location } from "../../engine/board/geometry.js";

function compareNumber(left: number, right: number): number {
  return left < right ? -1 : left > right ? 1 : 0;
}

function inputTag(value: Input): number {
  switch (value.kind) {
    case "takeback":
      return 0;
    case "location":
      return 1;
    case "modifier":
      return 2;
  }
}

export function compareLocations(left: Location, right: Location): number {
  return compareNumber(left.i, right.i) || compareNumber(left.j, right.j);
}

export function compareNextInputs(left: NextInput, right: NextInput): number {
  return compareInputs(left.input, right.input);
}

/** Stable input order: Takeback, Location(i,j), then Modifier. */
export function compareInputs(left: Input, right: Input): number {
  const tagOrder = compareNumber(inputTag(left), inputTag(right));
  if (tagOrder !== 0) return tagOrder;
  if (left.kind === "location" && right.kind === "location") {
    return compareLocations(left.location, right.location);
  }
  if (left.kind === "modifier" && right.kind === "modifier") {
    return compareNumber(MODIFIER_RANK[left.modifier], MODIFIER_RANK[right.modifier]);
  }
  return 0;
}

export function compareInputChains(
  left: readonly Input[],
  right: readonly Input[],
): number {
  const sharedLength = Math.min(left.length, right.length);
  for (let index = 0; index < sharedLength; index += 1) {
    const leftInput = left[index];
    const rightInput = right[index];
    if (leftInput === undefined || rightInput === undefined) break;
    const order = compareInputs(leftInput, rightInput);
    if (order !== 0) return order;
  }
  return compareNumber(left.length, right.length);
}

export function lexicographicValues<T>(
  values: readonly T[],
  compare: (left: T, right: T) => number,
): readonly T[] {
  for (let index = 1; index < values.length; index += 1) {
    const previous = values[index - 1];
    const current = values[index];
    if (
      previous !== undefined &&
      current !== undefined &&
      compare(previous, current) > 0
    ) {
      return [...values].sort(compare);
    }
  }
  return values;
}
