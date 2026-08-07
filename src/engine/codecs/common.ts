import { isValidLocation, type Location } from "../geometry.js";

function parseNonnegativeInteger(text: string): number | undefined {
  if (!/^(?:0|[1-9]\d*)$/u.test(text)) return undefined;
  const parsed = Number(text);
  return Number.isSafeInteger(parsed) ? parsed : undefined;
}

export function isAscii(value: string): boolean {
  for (let index = 0; index < value.length; index += 1) {
    if (value.charCodeAt(index) > 0x7f) return false;
  }
  return true;
}

export function locationFen(location: Location): string {
  return `${location.i},${location.j}`;
}

export function parseLocationFen(fen: string): Location | undefined {
  const parts = fen.split(",");
  if (parts.length !== 2) {
    return undefined;
  }
  const iText = parts[0];
  const jText = parts[1];
  if (iText === undefined || jText === undefined) {
    return undefined;
  }
  const i = parseNonnegativeInteger(iText);
  const j = parseNonnegativeInteger(jText);
  if (i === undefined || j === undefined) return undefined;
  const parsed = { i, j };
  return isValidLocation(parsed) ? parsed : undefined;
}

export function compareAscii(left: string, right: string): number {
  if (left < right) {
    return -1;
  }
  return left > right ? 1 : 0;
}
