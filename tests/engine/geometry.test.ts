import { describe, expect, it } from "vitest";

import { Board } from "../../src/engine/board/storage.js";
import {
  ALL_LOCATIONS,
  fromLocationIndex,
  locationIndex,
  nearbyLocations,
} from "../../src/engine/board/geometry.js";

describe("strict board geometry", () => {
  it("rejects coordinates that used to alias a valid array slot", () => {
    const board = new Board();

    expect(() => locationIndex({ i: -1, j: 11 })).toThrow(RangeError);
    expect(() => board.squareAt({ i: -1, j: 11 })).toThrow(RangeError);
    expect(board.get({ i: -1, j: 11 })).toBeUndefined();
  });

  it("rejects invalid positional indexes", () => {
    for (const index of [-1, 121, 1.5, Number.NaN]) {
      expect(() => fromLocationIndex(index), String(index)).toThrow(RangeError);
    }

    expect(fromLocationIndex(0)).toEqual({ i: 0, j: 0 });
    expect(fromLocationIndex(120)).toEqual({ i: 10, j: 10 });
  });

  it("keeps shared geometry and square tables immutable", () => {
    const nearby = nearbyLocations({ i: 5, j: 5 });

    expect(Object.isFrozen(ALL_LOCATIONS)).toBe(true);
    expect(ALL_LOCATIONS.every(Object.isFrozen)).toBe(true);
    expect(Object.isFrozen(nearby)).toBe(true);
    expect(nearby.every(Object.isFrozen)).toBe(true);
    expect(Object.isFrozen(new Board().squareAt({ i: 0, j: 3 }))).toBe(true);
  });
});
