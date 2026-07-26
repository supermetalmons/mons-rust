import { describe, expect, it } from "vitest";

import {
  automoveConfigForGame,
  patchAutomoveConfig,
  validateAutomoveConfig,
} from "../../src/automove/selector-config.js";
import {
  DEFAULT_SCORING_WEIGHTS,
  defineScoringProfile,
} from "../../src/automove/scoring.js";
import { ALL_GAME_VARIANTS } from "../../src/engine/config.js";
import { MonsGame } from "../../src/engine/game.js";

const SECTION_NAMES = Object.freeze([
  "budget",
  "search",
  "planner",
  "evaluation",
  "replyRisk",
  "policy",
] as const);

const CHANGED_WEIGHTS = defineScoringProfile({
  id: "test.selector-config.changed",
  base: DEFAULT_SCORING_WEIGHTS,
  material: {
    activeMon: DEFAULT_SCORING_WEIGHTS.material.activeMon + 1,
  },
});

describe("nested automove configuration", () => {
  it("resolves every built-in as a deeply readonly six-section value", () => {
    for (const variant of ALL_GAME_VARIANTS) {
      const game = new MonsGame(false, variant);
      for (const preference of ["fast", "normal", "pro"] as const) {
        const config = automoveConfigForGame(game, preference);

        expect(() => validateAutomoveConfig(config)).not.toThrow();
        expect(Object.keys(config)).toEqual(SECTION_NAMES);
        expect(Object.isFrozen(config)).toBe(true);
        for (const sectionName of SECTION_NAMES) {
          expect(Object.isFrozen(config[sectionName])).toBe(true);
          expect(Object.getPrototypeOf(config[sectionName])).toBe(
            Object.prototype,
          );
        }
        expect(Object.isFrozen(config.evaluation.weights)).toBe(true);
        expect(Object.isFrozen(config.evaluation.weights.material)).toBe(true);
        expect("depth" in config).toBe(false);
        expect("scoringWeights" in config).toBe(false);
        expect("enableTurnEngineSelector" in config).toBe(false);
      }
    }
  });

  it("freezes only patched sections and leaves the source untouched", () => {
    const game = new MonsGame(false, ALL_GAME_VARIANTS[0]);
    const source = automoveConfigForGame(game, "fast");
    const patched = patchAutomoveConfig(source, {
      search: {
        rootBranchLimit: source.search.rootBranchLimit + 1,
      },
    });

    expect(patched).not.toBe(source);
    expect(patched.search).not.toBe(source.search);
    expect(patched.budget).toBe(source.budget);
    expect(patched.planner).toBe(source.planner);
    expect(patched.evaluation).toBe(source.evaluation);
    expect(patched.replyRisk).toBe(source.replyRisk);
    expect(patched.policy).toBe(source.policy);
    expect(Object.isFrozen(patched.search)).toBe(true);
    expect(source.search.rootBranchLimit).not.toBe(
      patched.search.rootBranchLimit,
    );
    expect(
      Reflect.set(patched.search, "rootBranchLimit", Number.MAX_SAFE_INTEGER),
    ).toBe(false);
  });

  it("uses the scoring profile identity owned by the weights", () => {
    const game = new MonsGame(false, ALL_GAME_VARIANTS[0]);
    const source = automoveConfigForGame(game, "normal");
    const patched = patchAutomoveConfig(source, {
      evaluation: { weights: CHANGED_WEIGHTS },
    });

    expect(patched.evaluation).not.toBe(source.evaluation);
    expect(patched.evaluation.weights).toBe(CHANGED_WEIGHTS);
    expect(patched.evaluation.weights.id).not.toBe(
      source.evaluation.weights.id,
    );
  });

  it("snapshots mutable caller-owned scoring profiles", () => {
    const source = automoveConfigForGame(
      new MonsGame(false, ALL_GAME_VARIANTS[0]),
      "normal",
    );
    const mutableWeights = {
      id: "test.selector-config.mutable",
      formula: { ...DEFAULT_SCORING_WEIGHTS.formula },
      material: { ...DEFAULT_SCORING_WEIGHTS.material },
      position: { ...DEFAULT_SCORING_WEIGHTS.position },
      mana: { ...DEFAULT_SCORING_WEIGHTS.mana },
      race: { ...DEFAULT_SCORING_WEIGHTS.race },
      threat: { ...DEFAULT_SCORING_WEIGHTS.threat },
    };
    const patched = patchAutomoveConfig(source, {
      evaluation: { weights: mutableWeights },
    });
    const capturedActiveMon = patched.evaluation.weights.material.activeMon;

    mutableWeights.material.activeMon += 100;

    expect(patched.evaluation.weights).not.toBe(mutableWeights);
    expect(patched.evaluation.weights.material.activeMon).toBe(
      capturedActiveMon,
    );
    expect(Object.isFrozen(patched.evaluation.weights)).toBe(true);
    expect(Object.isFrozen(patched.evaluation.weights.material)).toBe(true);
  });

  it("rejects invalid patches without modifying the source", () => {
    const game = new MonsGame(false, ALL_GAME_VARIANTS[0]);
    const source = automoveConfigForGame(game, "pro");
    const originalDepth = source.budget.depth;

    expect(() =>
      patchAutomoveConfig(source, {
        budget: { depth: -1 },
      }),
    ).toThrow(RangeError);
    expect(source.budget.depth).toBe(originalDepth);
  });

  it("rejects malformed runtime shapes and unsafe ranges", () => {
    const source = automoveConfigForGame(
      new MonsGame(false, ALL_GAME_VARIANTS[0]),
      "normal",
    );

    expect(() =>
      validateAutomoveConfig({
        ...source,
        unexpected: true,
      }),
    ).toThrow(/must contain exactly/u);
    expect(() =>
      patchAutomoveConfig(source, {
        search: { quietReductions: 1 },
      } as unknown as Parameters<typeof patchAutomoveConfig>[1]),
    ).toThrow(/must be a boolean/u);
    expect(() =>
      patchAutomoveConfig(source, {
        planner: { mode: "experimental" },
      } as unknown as Parameters<typeof patchAutomoveConfig>[1]),
    ).toThrow(/unknown planner mode/u);
    expect(() =>
      patchAutomoveConfig(source, {
        replyRisk: { nodeShareBp: 10_001 },
      }),
    ).toThrow(/basis-point/u);
    expect(() =>
      patchAutomoveConfig(source, {
        search: { transpositionCapacity: 1_000_001 },
      }),
    ).toThrow(/1000000/u);
    expect(() =>
      patchAutomoveConfig(source, {
        search: { transpositionCapacity: 0 },
      }),
    ).toThrow(/must be positive/u);
  });
});
