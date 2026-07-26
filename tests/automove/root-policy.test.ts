import { describe, expect, it } from "vitest";

import { productionRootPolicy } from "../../src/automove/advisor.js";
import { buildRootPolicy } from "../../src/automove/advisor/root-policy-core.js";
import {
  PRODUCTION_COMPETITION_KIND_ORDER,
  ProductionComparisonPhase,
  ProductionCompetitionKind,
  compareProductionRules,
  evaluateProductionCompetitionRules,
  type ProductionRootPolicy,
  type RootSelectionContext,
} from "../../src/automove/root-selector.js";
import { automoveConfigForGame } from "../../src/automove/selector-config.js";
import { MonsGame } from "../../src/engine/game.js";
import { createTestAutomoveExecutionContext } from "./execution-context.test-helper.js";

describe("production root policy", () => {
  it("publishes frozen, unique rules in their semantic phase order", () => {
    const policy = buildRootPolicy(createTestAutomoveExecutionContext());
    const ids = [
      ...policy.competitionRules.map(({ id }) => id),
      ...policy.safetyReentryRules.map(({ id }) => id),
      ...policy.finalReentryRules.map(({ id }) => id),
      ...policy.comparisonRules.map(({ id }) => id),
    ];

    expect(policy.competitionRules.map(({ kind }) => kind)).toEqual(
      PRODUCTION_COMPETITION_KIND_ORDER,
    );
    expect(policy.finalReentryRules.map(({ id }) => id)).toEqual([
      "final-reentry.plain-spirit-progress",
      "final-reentry.risky-recovery",
    ]);
    expect(policy.comparisonRules.map(({ phase }) => phase)).toEqual([
      ProductionComparisonPhase.SpiritSetup,
      ProductionComparisonPhase.ProjectionChallenge,
      ProductionComparisonPhase.Projection,
      ProductionComparisonPhase.FollowupFloor,
    ]);
    expect(new Set(ids).size).toBe(ids.length);
    expect(Object.isFrozen(policy)).toBe(true);
    expect(Object.isFrozen(policy.competitionRules)).toBe(true);
    expect(Object.isFrozen(policy.safetyReentryRules)).toBe(true);
    expect(Object.isFrozen(policy.finalReentryRules)).toBe(true);
    expect(Object.isFrozen(policy.comparisonRules)).toBe(true);
    expect(
      [
        ...policy.competitionRules,
        ...policy.safetyReentryRules,
        ...policy.finalReentryRules,
        ...policy.comparisonRules,
      ].every(Object.isFrozen),
    ).toBe(true);
    expect(policy.rootPicker).toBeUndefined();
  });

  it("adds the root picker only at the public policy boundary", () => {
    const execution = createTestAutomoveExecutionContext();
    const game = new MonsGame();
    const policy = productionRootPolicy(
      execution,
      automoveConfigForGame(game, "pro"),
    );

    expect(policy.rootPicker?.id).toBe("root-picker.advisor-postsearch");
    expect(Object.isFrozen(policy.rootPicker)).toBe(true);
    expect(Object.isFrozen(policy)).toBe(true);
  });

  it("evaluates every competition rule of a kind without short-circuiting", () => {
    const visited: string[] = [];
    const policy: ProductionRootPolicy = {
      competitionRules: [
        {
          id: "competition.first",
          kind: ProductionCompetitionKind.Score,
          evaluate() {
            visited.push("first");
            return { kind: "select" };
          },
        },
        {
          id: "competition.second",
          kind: ProductionCompetitionKind.Score,
          evaluate() {
            visited.push("second");
            return { kind: "continue" };
          },
        },
        {
          id: "competition.other-kind",
          kind: ProductionCompetitionKind.Projection,
          evaluate() {
            visited.push("other-kind");
            return { kind: "select" };
          },
        },
      ],
      safetyReentryRules: [],
      finalReentryRules: [],
      comparisonRules: [],
    };

    expect(
      evaluateProductionCompetitionRules(
        policy,
        ProductionCompetitionKind.Score,
        {} as RootSelectionContext,
      ),
    ).toBe(true);
    expect(visited).toEqual(["first", "second"]);
  });

  it("uses the first defined comparison result in a phase", () => {
    const visited: string[] = [];
    const policy: ProductionRootPolicy = {
      competitionRules: [],
      safetyReentryRules: [],
      finalReentryRules: [],
      comparisonRules: [
        {
          id: "comparison.first",
          phase: ProductionComparisonPhase.Projection,
          compare() {
            visited.push("first");
            return { kind: "continue" };
          },
        },
        {
          id: "comparison.second",
          phase: ProductionComparisonPhase.Projection,
          compare() {
            visited.push("second");
            return { kind: "compare", order: 42 };
          },
        },
        {
          id: "comparison.third",
          phase: ProductionComparisonPhase.Projection,
          compare() {
            visited.push("third");
            return { kind: "compare", order: -1 };
          },
        },
      ],
    };

    expect(
      compareProductionRules(
        ProductionComparisonPhase.Projection,
        {} as Parameters<typeof compareProductionRules>[1],
        policy,
      ),
    ).toBe(1);
    expect(visited).toEqual(["first", "second"]);
  });
});
