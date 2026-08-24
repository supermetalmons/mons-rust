import type { Location } from "../board/geometry.js";

export function locationFen(location: Location): string {
  return `${location.i},${location.j}`;
}

export function compareAscii(left: string, right: string): number {
  if (left < right) {
    return -1;
  }
  return left > right ? 1 : 0;
}
