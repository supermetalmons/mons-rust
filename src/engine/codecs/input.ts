import { MAX_INPUTS_PER_MOVE, type Input, Modifier } from "../domain.js";
import { isAscii, locationFen, parseLocationFen } from "./common.js";

export function modifierFen(modifier: Modifier): string {
  switch (modifier) {
    case Modifier.SelectPotion:
      return "p";
    case Modifier.SelectBomb:
      return "b";
  }
}

export function parseModifierFen(fen: string): Modifier | undefined {
  switch (fen) {
    case "p":
      return Modifier.SelectPotion;
    case "b":
      return Modifier.SelectBomb;
    default:
      return undefined;
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

export function parseInputFen(fen: string): Input | undefined {
  switch (fen[0]) {
    case "l": {
      const parsed = parseLocationFen(fen.slice(1));
      return parsed === undefined
        ? undefined
        : { kind: "location", location: parsed };
    }
    case "m": {
      const parsed = parseModifierFen(fen.slice(1));
      return parsed === undefined
        ? undefined
        : { kind: "modifier", modifier: parsed };
    }
    case "z":
      return fen === "z" ? { kind: "takeback" } : undefined;
    default:
      return undefined;
  }
}

export function inputArrayFen(inputs: readonly Input[]): string {
  return inputs.map(inputFen).join(";");
}

export function parseInputArrayFen(fen: string): Input[] | undefined {
  if (!isAscii(fen)) {
    return undefined;
  }
  if (fen === "") {
    return [];
  }
  const result: Input[] = [];
  for (const part of fen.split(";")) {
    if (result.length === MAX_INPUTS_PER_MOVE) return undefined;
    const input = parseInputFen(part);
    if (input === undefined) return undefined;
    result.push(input);
  }
  return result;
}
