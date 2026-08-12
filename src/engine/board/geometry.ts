export const BOARD_SIZE = 11;
export const BOARD_CELLS = BOARD_SIZE * BOARD_SIZE;
export const BOARD_CENTER_INDEX = Math.trunc(BOARD_SIZE / 2);
export const MAX_LOCATION_INDEX = BOARD_SIZE - 1;

export type Location = {
  readonly i: number;
  readonly j: number;
};

export function location(i: number, j: number): Location {
  return { i, j };
}

export function locationEquals(left: Location, right: Location): boolean {
  return left.i === right.i && left.j === right.j;
}

export function isValidLocation(value: Location): boolean {
  return (
    Number.isInteger(value.i) &&
    Number.isInteger(value.j) &&
    value.i >= 0 &&
    value.i < BOARD_SIZE &&
    value.j >= 0 &&
    value.j < BOARD_SIZE
  );
}

export function assertValidLocation(value: Location): void {
  if (!isValidLocation(value)) {
    throw new RangeError(
      `board location must use integer coordinates from 0 to ${MAX_LOCATION_INDEX}`,
    );
  }
}

export function locationIndex(value: Location): number {
  assertValidLocation(value);
  return value.i * BOARD_SIZE + value.j;
}

export function fromLocationIndex(index: number): Location {
  if (!Number.isInteger(index) || index < 0 || index >= BOARD_CELLS) {
    throw new RangeError(
      `board location index must be an integer from 0 to ${BOARD_CELLS - 1}`,
    );
  }
  return location(Math.trunc(index / BOARD_SIZE), index % BOARD_SIZE);
}

export function locationBetween(first: Location, second: Location): Location {
  return location(
    Math.trunc((first.i + second.i) / 2),
    Math.trunc((first.j + second.j) / 2),
  );
}

/** Chebyshev distance, which is the board's movement distance. */
export function locationDistance(from: Location, to: Location): number {
  return Math.max(Math.abs(to.i - from.i), Math.abs(to.j - from.j));
}

function nearbyLocationsFor(origin: Location, distance: number): readonly Location[] {
  const result: Location[] = [];
  for (let i = origin.i - distance; i <= origin.i + distance; i += 1) {
    for (let j = origin.j - distance; j <= origin.j + distance; j += 1) {
      const candidate = location(i, j);
      if (isValidLocation(candidate) && !locationEquals(candidate, origin)) {
        result.push(candidate);
      }
    }
  }
  return result;
}

function directionalReachabilityFor(
  origin: Location,
  deltas: readonly (readonly [number, number])[],
): readonly Location[] {
  const result: Location[] = [];
  for (const [deltaI, deltaJ] of deltas) {
    const candidate = location(origin.i + deltaI, origin.j + deltaJ);
    if (isValidLocation(candidate)) {
      result.push(candidate);
    }
  }
  return result;
}

function spiritReachabilityFor(origin: Location): readonly Location[] {
  const result: Location[] = [];
  for (let deltaI = -2; deltaI <= 2; deltaI += 1) {
    for (let deltaJ = -2; deltaJ <= 2; deltaJ += 1) {
      if (Math.max(Math.abs(deltaI), Math.abs(deltaJ)) !== 2) {
        continue;
      }
      const candidate = location(origin.i + deltaI, origin.j + deltaJ);
      if (isValidLocation(candidate)) {
        result.push(candidate);
      }
    }
  }
  return result;
}

function buildCache(
  factory: (origin: Location) => readonly Location[],
): readonly (readonly Location[])[] {
  return Object.freeze(
    Array.from({ length: BOARD_CELLS }, (_, index) =>
      Object.freeze(factory(fromLocationIndex(index)).map((at) => Object.freeze(at))),
    ),
  );
}

const NEARBY_LOCATIONS = buildCache((origin) => nearbyLocationsFor(origin, 1));
const BOMB_REACHABILITY = buildCache((origin) => nearbyLocationsFor(origin, 3));
const MYSTIC_REACHABILITY = buildCache((origin) =>
  directionalReachabilityFor(origin, [
    [-2, -2],
    [2, 2],
    [-2, 2],
    [2, -2],
  ]),
);
const DEMON_REACHABILITY = buildCache((origin) =>
  directionalReachabilityFor(origin, [
    [-2, 0],
    [2, 0],
    [0, 2],
    [0, -2],
  ]),
);
const SPIRIT_REACHABILITY = buildCache(spiritReachabilityFor);

function cachedLocations(
  cache: readonly (readonly Location[])[],
  origin: Location,
): readonly Location[] {
  if (!isValidLocation(origin)) {
    return [];
  }
  return cache[locationIndex(origin)] ?? [];
}

export function nearbyLocations(origin: Location): readonly Location[] {
  return cachedLocations(NEARBY_LOCATIONS, origin);
}

export function bombReachableLocations(origin: Location): readonly Location[] {
  return cachedLocations(BOMB_REACHABILITY, origin);
}

export function mysticReachableLocations(origin: Location): readonly Location[] {
  return cachedLocations(MYSTIC_REACHABILITY, origin);
}

export function demonReachableLocations(origin: Location): readonly Location[] {
  return cachedLocations(DEMON_REACHABILITY, origin);
}

export function spiritReachableLocations(origin: Location): readonly Location[] {
  return cachedLocations(SPIRIT_REACHABILITY, origin);
}

/** Every board location in row-major array order. */
export const ALL_LOCATIONS: readonly Location[] = Object.freeze(
  Array.from({ length: BOARD_CELLS }, (_, index) =>
    Object.freeze(fromLocationIndex(index)),
  ),
);
