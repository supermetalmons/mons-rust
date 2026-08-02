import { readFileSync } from "node:fs";

import { describe, expect, it } from "vitest";

import { AutomoveEngine } from "../../src/automove/automove-engine.js";
import { suggestMove } from "../../src/automove/runtime.js";
import { enumerateLegalTransitions } from "../../src/automove/transitions.js";
import {
  loadPosition,
  moveToInputs,
  tryLoadPosition,
} from "../../src/automove/fast/bridge.js";
import { MAX_MOVES, generateMoves } from "../../src/automove/fast/moves.js";
import {
  FastPosition,
  applyFastMoveAndCheckRepresentability,
} from "../../src/automove/fast/position.js";
import {
  PRO_FAST_PROFILE,
  selectProFastInputs,
} from "../../src/automove/fast/index.js";
import { FastSearcher } from "../../src/automove/fast/search.js";
import { ALL_GAME_VARIANTS } from "../../src/engine/config.js";
import { inputArrayFen, parseInputArrayFen } from "../../src/engine/fen.js";
import { MonsGame } from "../../src/engine/game.js";
import { BOARD_CELLS } from "../../src/engine/geometry.js";
import { expectFastPositionInvariants } from "./fast.test-helper.js";

type DecisionState = {
  readonly id: string;
  readonly fen: string;
};

const V4_DECISION_STATES: readonly DecisionState[] = readFileSync(
  new URL(
    "../../test-data/automove-decisions/v4/decisions.jsonl",
    import.meta.url,
  ),
  "utf8",
)
  .trim()
  .split(/\r?\n/)
  .map((line) => {
    const state = JSON.parse(line) as Partial<DecisionState>;
    if (typeof state.id !== "string" || typeof state.fen !== "string") {
      throw new TypeError("automove decision state must contain id and fen");
    }
    return { id: state.id, fen: state.fen };
  });

function randomSource(seed: number) {
  let state = seed >>> 0 || 0x9e3779b9;
  return {
    nextUint32(): number {
      state ^= state << 13;
      state >>>= 0;
      state ^= state >>> 17;
      state ^= state << 5;
      state >>>= 0;
      return state;
    },
  };
}

function stateSignature(position: FastPosition): string {
  const parts: string[] = [];
  for (let index = 0; index < BOARD_CELLS; index += 1) {
    const cell = position.cells[index] ?? 0;
    if (cell !== 0) parts.push(`${index}:${cell}`);
  }
  parts.push(
    `w${position.whiteScore}`,
    `b${position.blackScore}`,
    `a${position.active}`,
    `m${position.monsMoves}`,
    `n${position.manaMoves}`,
    `u${position.actionsUsed}`,
    `p${position.potions[0]}/${position.potions[1]}`,
    `f${position.firstTurn ? 1 : 0}`,
    `l${[...position.monLocations].join("/")}`,
    `r${[...position.freeMana].join("/")}`,
    `c${position.manaCount}`,
    `i${[...position.manaIndices].slice(0, position.manaCount).join("/")}`,
    `h${position.hashLo}/${position.hashHi}`,
  );
  return parts.join(",");
}

function compareCanonicalPosition(
  game: MonsGame,
  label: string,
  engine: AutomoveEngine,
  buffer: Int32Array,
  packed: FastPosition,
  applied: FastPosition,
  expected: FastPosition,
): number {
  loadPosition(packed, game);
  expectFastPositionInvariants(packed, `${label}: packed load`);
  const count = generateMoves(packed, buffer);
  const generated: (readonly [string, number])[] = [];
  for (let index = 0; index < count; index += 1) {
    const move = buffer[index] ?? 0;
    generated.push([inputArrayFen(moveToInputs(move)), move]);
  }

  const reference: string[] = [];
  engine.run((execution) => {
    for (const transition of enumerateLegalTransitions(
      execution,
      game,
      100_000,
    )) {
      reference.push(inputArrayFen(transition.inputs));
    }
    return undefined;
  });

  const generatedInputs = generated.map(([inputFen]) => inputFen);
  expect(
    new Set(generatedInputs).size,
    `${label}: duplicate generated moves`,
  ).toBe(generatedInputs.length);
  expect(new Set(reference).size, `${label}: duplicate canonical moves`).toBe(
    reference.length,
  );
  expect([...generatedInputs].sort(), label).toEqual([...reference].sort());

  let checkedMoves = 0;
  for (const [inputFen, move] of generated) {
    applied.copyFrom(packed);
    expectFastPositionInvariants(applied, `${label} ${inputFen}: before`);
    const representable = applyFastMoveAndCheckRepresentability(applied, move);
    const inputs = parseInputArrayFen(inputFen);
    expect(inputs).toBeDefined();
    if (inputs === undefined) continue;
    const clone = game.fork();
    expect(clone.processInput(inputs, false, false).kind).toBe("events");
    const canonicalRepresentable = tryLoadPosition(expected, clone, 1);
    expect(representable, `${label} ${inputFen}`).toBe(canonicalRepresentable);
    if (!representable) continue;
    expectFastPositionInvariants(applied, `${label} ${inputFen}: applied`);
    expectFastPositionInvariants(expected, `${label} ${inputFen}: canonical`);
    expect(stateSignature(applied), `${label} ${inputFen}`).toBe(
      stateSignature(expected),
    );
    checkedMoves += 1;
  }
  return checkedMoves;
}

describe("packed-state automove search", () => {
  it("matches engine move generation and transitions on every variant", () => {
    const buffer = new Int32Array(MAX_MOVES);
    const packed = new FastPosition();
    const applied = new FastPosition();
    const expected = new FastPosition();
    let checkedPositions = 0;
    let checkedMoves = 0;

    for (const variant of ALL_GAME_VARIANTS) {
      const game = new MonsGame(true, variant);
      const engine = new AutomoveEngine({ randomSource: randomSource(17) });
      for (let ply = 0; ply < 40; ply += 1) {
        if (game.winnerColor() !== undefined) break;

        checkedMoves += compareCanonicalPosition(
          game,
          `${variant} ply ${ply}`,
          engine,
          buffer,
          packed,
          applied,
          expected,
        );
        checkedPositions += 1;

        const inputFen = engine.run((execution) => {
          const suggestion = suggestMove(execution, game, "random");
          return suggestion.output.kind === "events"
            ? suggestion.inputFen
            : undefined;
        });
        if (inputFen === undefined) break;
        const inputs = parseInputArrayFen(inputFen);
        if (inputs === undefined) break;
        if (game.processInput(inputs, false, false).kind !== "events") break;
      }
    }

    const corpusEngine = new AutomoveEngine({ randomSource: randomSource(29) });
    for (const state of V4_DECISION_STATES) {
      const game = MonsGame.fromFen(state.fen, true);
      expect(game, state.id).toBeDefined();
      if (game === undefined) continue;
      checkedMoves += compareCanonicalPosition(
        game,
        `automove-decisions/v4 ${state.id}`,
        corpusEngine,
        buffer,
        packed,
        applied,
        expected,
      );
      checkedPositions += 1;
    }

    expect(checkedPositions).toBeGreaterThan(400);
    expect(checkedMoves).toBeGreaterThan(10_000);
  }, 120_000);

  it("verifies reduced fail-highs at full depth", () => {
    const game = MonsGame.fromFen(
      "0 0 b 0 0 3 0 0 2 n03y0xs0xn06/n06d0xa0xe0xn02/n11/n04xxmn01xxmn04/n04xxmxxmxxmn04/xxQn04xxUn04xxQ/n04xxMxxMxxMn04/n04xxMn01xxMn04/n03E0xn07/n04A0xn01D0xn04/n06S0xY0xn03 5",
      true,
    );
    expect(game).toBeDefined();
    if (game === undefined) return;

    const searcher = new FastSearcher();
    loadPosition(searcher.root, game);
    const outcome = searcher.search(
      {
        maxDepth: 4,
        maxNodes: 1_000_000,
        tuning: {
          lateMoveReduction: true,
          lateMoveIndex: 3,
          lateMoveDeepIndex: 8,
          moveCountPruning: true,
          moveCountDepth: 3,
          moveCountBase: 4,
          moveCountFactor: 5,
          futilityMargin: 900,
        },
      },
      () => false,
    );

    expect(outcome.depth).toBe(4);
    expect(outcome.score).toBe(2260);
    expect(outcome.supported).toBe(true);
    expect(inputArrayFen(moveToInputs(outcome.move))).toBe("l0,4;l1,4");
  });

  it("does not accept a selectively derived terminal score as proven", () => {
    const game = MonsGame.fromFen(
      "0 3 b 0 0 0 0 0 100 n11/n02xxmy0xn03xxUn01xxmn01/n01xxmn02s0xxxmn05/n04a0xn06/n05A0xn05/n11/n01d0xn05xxMn03/n06xxMn01xxMn02/xxMn03S0xD0xn04e0x/n03E0xn06Y0x/n11",
      true,
    );
    expect(game).toBeDefined();
    if (game === undefined) return;

    const searcher = new FastSearcher();
    loadPosition(searcher.root, game);
    const outcome = searcher.search(
      { maxDepth: 8, maxNodes: 120_000 },
      () => false,
    );

    expect(outcome).toEqual({
      move: 16_725_528,
      score: 999_996,
      depth: 5,
      nodes: 120_000,
      supported: true,
    });
    expect(
      game.fork().processInput(moveToInputs(outcome.move), false, false).kind,
    ).toBe("events");
  });

  it("still stops at depth one for a genuine immediate win", () => {
    const game = MonsGame.fromFen(
      "0 3 b 0 0 0 0 0 108 n01d0Mn09/n06e0xn02xxmn01/s0xn03y0xn01xxMn04/n10a0x/n04A0xn03xxmn02/n05xxUn05/n02xxMn05Y0xn02/n02D0Mn01xxMn03S0xn02/n11/n02E0xn08/n11",
      true,
    );
    expect(game).toBeDefined();
    if (game === undefined) return;

    const winningColor = game.activeColor;
    const searcher = new FastSearcher();
    loadPosition(searcher.root, game);
    const outcome = searcher.search(
      { maxDepth: 8, maxNodes: 120_000 },
      () => false,
    );

    expect(outcome).toEqual({
      move: 1_204,
      score: 1_000_000,
      depth: 1,
      nodes: 62,
      supported: true,
    });
    const replay = game.fork();
    expect(
      replay.processInput(moveToInputs(outcome.move), false, false).kind,
    ).toBe("events");
    expect(replay.winnerColor()).toBe(winningColor);
  });

  it("selects an applicable Pro move for every variant opening", () => {
    let now = 0;
    const engine = new AutomoveEngine({ clock: () => now++ });
    for (const variant of ALL_GAME_VARIANTS) {
      const game = new MonsGame(true, variant);
      const inputs = engine.run((execution) =>
        selectProFastInputs(execution, game),
      );
      expect(inputs, variant).toBeDefined();
      if (inputs === undefined) continue;
      expect(inputs.length, variant).toBeGreaterThan(0);
      expect(game.fork().processInput(inputs, false, false).kind, variant).toBe(
        "events",
      );
    }
  }, 60_000);

  it("bounds the search when the host clock never advances", () => {
    const game = new MonsGame(true, ALL_GAME_VARIANTS[0]);
    let clockReads = 0;
    const engine = new AutomoveEngine({
      clock: () => {
        clockReads += 1;
        return 0;
      },
    });
    const inputs = engine.run((execution) =>
      selectProFastInputs(execution, game),
    );
    expect(inputs).toBeDefined();
    if (inputs === undefined) return;
    expect(inputs.length).toBeGreaterThan(0);
    expect(clockReads).toBeGreaterThan(0);
    expect(clockReads).toBeLessThanOrEqual(
      Math.ceil(PRO_FAST_PROFILE.maxNodes / 512) * 2 + 16,
    );
  }, 30_000);
});
