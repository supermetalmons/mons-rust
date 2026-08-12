import { describe, expect, it } from "vitest";

import { Color, regularMana, type Mana } from "../../src/engine/model/domain.js";
import { locationIndex } from "../../src/engine/board/geometry.js";
import { actionIdentity } from "../../src/automove/turn/action-rules.js";
import type { TurnAction } from "../../src/automove/turn/model.js";

const FIRST = { i: 0, j: 0 } as const;
const SECOND = { i: 0, j: 1 } as const;
const THIRD = { i: 0, j: 2 } as const;

function key(tag: number, third = -2): string {
  return `${tag}:${locationIndex(FIRST)}:${locationIndex(SECOND)}:${third}`;
}

describe("turn action compilation contracts", () => {
  it("pins action identity tags, sentinels, and mana suffixes", () => {
    const regular = regularMana(Color.White);
    const supermana: Mana = { kind: "supermana" };
    const actions: readonly TurnAction[] = [
      { kind: "walk", actor: FIRST, to: SECOND },
      { kind: "attack", actor: FIRST, target: SECOND },
      {
        kind: "spirit-shift",
        actor: FIRST,
        target: SECOND,
        destination: THIRD,
      },
      { kind: "bomb", actor: FIRST, target: SECOND },
      { kind: "move-mana", from: FIRST, to: SECOND },
      { kind: "score-carry", actor: FIRST, wanted: regular, step: SECOND },
      { kind: "score-carry", actor: FIRST, wanted: supermana, step: SECOND },
      { kind: "safety-retreat", actor: FIRST, to: SECOND },
    ];

    expect(actions.map(actionIdentity)).toEqual([
      key(0),
      key(1),
      key(2, locationIndex(THIRD)),
      key(3),
      key(4),
      `${key(5)}:regular:${Color.White}`,
      `${key(5)}:supermana:s`,
      key(6),
    ]);
  });
});
