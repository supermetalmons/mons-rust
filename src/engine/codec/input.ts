import type { Location } from "../board/geometry.js";
import { MAX_INPUTS_PER_MOVE, type Input, Modifier } from "../model/domain.js";
import { locationFen } from "./common.js";

const ASCII_ZERO = 48;
const ASCII_NINE = 57;
const ASCII_COMMA = 44;
const ASCII_SEMICOLON = 59;
const ASCII_LOCATION = 108;
const ASCII_MODIFIER = 109;
const ASCII_TAKEBACK = 122;
const ASCII_POTION = 112;
const ASCII_BOMB = 98;

function modifierFen(modifier: Modifier): string {
  switch (modifier) {
    case Modifier.SelectPotion:
      return "p";
    case Modifier.SelectBomb:
      return "b";
  }
}

export function inputFen(input: Input): string {
  switch (input.kind) {
    case "takeback":
      return "z";
    case "location":
      return `l${locationFen(input.location)}`;
    case "modifier":
      return `m${modifierFen(input.modifier)}`;
  }
}

function parseCoordinate(fen: string, start: number, end: number): number | undefined {
  if (start >= end) return undefined;
  const first = fen.charCodeAt(start);
  if (first < ASCII_ZERO || first > ASCII_NINE) return undefined;
  if (
    first === ASCII_ZERO + 1 &&
    start + 1 < end &&
    fen.charCodeAt(start + 1) === ASCII_ZERO
  ) {
    return 10;
  }
  return first - ASCII_ZERO;
}

function coordinateEnd(start: number, value: number): number {
  return start + (value === 10 ? 2 : 1);
}

function parseLocation(fen: string, start: number, end: number): Location | undefined {
  const i = parseCoordinate(fen, start, end);
  if (i === undefined) return undefined;
  const comma = coordinateEnd(start, i);
  if (comma >= end || fen.charCodeAt(comma) !== ASCII_COMMA) return undefined;

  const jStart = comma + 1;
  const j = parseCoordinate(fen, jStart, end);
  if (j === undefined || coordinateEnd(jStart, j) !== end) return undefined;
  return { i, j };
}

function parseInputFenRange(
  fen: string,
  start: number,
  end: number,
): Input | undefined {
  switch (fen.charCodeAt(start)) {
    case ASCII_LOCATION: {
      const location = parseLocation(fen, start + 1, end);
      return location === undefined ? undefined : { kind: "location", location };
    }
    case ASCII_MODIFIER:
      if (end !== start + 2) return undefined;
      if (fen.charCodeAt(start + 1) === ASCII_POTION) {
        return { kind: "modifier", modifier: Modifier.SelectPotion };
      }
      return fen.charCodeAt(start + 1) === ASCII_BOMB
        ? { kind: "modifier", modifier: Modifier.SelectBomb }
        : undefined;
    case ASCII_TAKEBACK:
      return end === start + 1 ? { kind: "takeback" } : undefined;
    default:
      return undefined;
  }
}

export function parseInputFen(fen: string): Input | undefined {
  return parseInputFenRange(fen, 0, fen.length);
}

export function inputArrayFen(inputs: readonly Input[]): string {
  return inputs.map(inputFen).join(";");
}

export function parseInputArrayFen(fen: unknown): Input[] | undefined {
  if (typeof fen !== "string") {
    throw new TypeError("input FEN must be a string");
  }
  if (fen === "") {
    return [];
  }
  const result: Input[] = [];
  let start = 0;
  for (let index = 0; index <= fen.length; index += 1) {
    if (index < fen.length && fen.charCodeAt(index) !== ASCII_SEMICOLON) {
      continue;
    }
    if (result.length === MAX_INPUTS_PER_MOVE) return undefined;
    const input = parseInputFenRange(fen, start, index);
    if (input === undefined) return undefined;
    result.push(input);
    start = index + 1;
  }
  return result;
}
