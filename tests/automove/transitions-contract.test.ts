import { describe, expect, it } from "vitest";

import {
  Color,
  type Event,
  type Input,
  type Output,
} from "../../src/engine/model/domain.js";
import { inputArrayFen } from "../../src/engine/codec/input.js";
import type { MonsGame } from "../../src/engine/game/mons-game.js";
import {
  enumerateLegalTransitions,
  enumerateLegalTransitionsLexicographicBounded,
} from "../../src/automove/transitions/enumerate.js";
import { createTestAutomoveExecutionContext } from "./execution-context.test-helper.js";

const ENGINE_FIRST = { i: 1, j: 0 } as const;
const LEXICOGRAPHIC_FIRST = { i: 0, j: 0 } as const;
const TERMINAL_EVENTS: readonly Event[] = [{ kind: "next-turn", color: Color.Black }];

function reverseOrderedGame(): MonsGame {
  const game = {
    fork(): MonsGame {
      return game as unknown as MonsGame;
    },
    processInputWithStartOptions(inputs: readonly Input[]): Output {
      return inputs.length === 0
        ? {
            kind: "locations-to-start-from",
            locations: [ENGINE_FIRST, LEXICOGRAPHIC_FIRST],
          }
        : { kind: "events", events: TERMINAL_EVENTS };
    },
    forkAndApplyEventsForSimulation(events: readonly Event[]) {
      return { game: game as unknown as MonsGame, events: [...events] };
    },
  };
  return game as unknown as MonsGame;
}

describe("automove transition traversal contract", () => {
  it("bounds engine traversal before sorting but lexicographic traversal after sorting", () => {
    const execution = createTestAutomoveExecutionContext();
    const game = reverseOrderedGame();

    const ordinary = enumerateLegalTransitions(execution, game, 1);
    const lexicographic = enumerateLegalTransitionsLexicographicBounded(
      execution,
      game,
      1,
    );

    expect(ordinary.map(({ inputs }) => inputArrayFen(inputs))).toEqual(["l1,0"]);
    expect(lexicographic.map(({ inputs }) => inputArrayFen(inputs))).toEqual(["l0,0"]);
  });
});
