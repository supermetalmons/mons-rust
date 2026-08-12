import { describe, expect, it } from "vitest";

import type { AutomoveConfig } from "../../src/automove/config/types.js";
import * as advisor from "../../src/automove/policy/advisor/index.js";
import * as rootPolicy from "../../src/automove/policy/advisor/root-policy.js";
import {
  ADVISOR_SELECTION_RULES,
  applyAdvisorSelectionRuleRegistry,
  type AdvisorSelectionRule,
  type AdvisorSelectionRuleContext,
} from "../../src/automove/policy/advisor/selection/registry.js";
import { ProductionRootAdvisorReasonCode } from "../../src/automove/policy/advisor/types.js";
import { createTestAutomoveExecutionContext } from "./execution-context.test-helper.js";

describe("production advisor contracts", () => {
  it("pins the ordered root policy registry IDs", () => {
    const execution = createTestAutomoveExecutionContext();
    const policy = rootPolicy.buildRootPolicy(execution);
    expect(policy.competitionRules.map(({ id }) => id)).toEqual([
      "competition.safe-progress",
      "competition.followup-progress",
      "competition.risky-score",
      "competition.negative-deny",
      "competition.score",
      "competition.projection",
      "competition.risky-recovery",
    ]);
    expect(policy.safetyReentryRules.map(({ id }) => id)).toEqual([
      "safety-reentry.recovery-and-progress",
    ]);
    expect(policy.finalReentryRules.map(({ id }) => id)).toEqual([
      "final-reentry.plain-spirit-progress",
      "final-reentry.risky-recovery",
    ]);
    expect(policy.comparisonRules.map(({ id }) => id)).toEqual([
      "comparison.spirit-setup",
      "comparison.projection-challenge",
      "comparison.projection",
      "comparison.followup-floor",
    ]);
    expect(Object.isFrozen(policy)).toBe(true);
    expect(
      [
        ...policy.competitionRules,
        ...policy.safetyReentryRules,
        ...policy.finalReentryRules,
        ...policy.comparisonRules,
      ].every(Object.isFrozen),
    ).toBe(true);
    expect(
      advisor.productionRootPolicy(execution, {} as AutomoveConfig).rootPicker?.id,
    ).toBe("root-picker.advisor-postsearch");
  });

  it("pins selection rule order", () => {
    expect(ADVISOR_SELECTION_RULES.map(({ id }) => id)).toEqual([
      "black-family-competition",
      "black-opening-setup-sibling",
      "black-early-safe-mana-followup",
      "black-early-plain-spirit-followup",
      "black-turn-four-vulnerable-progress-mana",
      "black-turn-six-attack-vulnerable-progress-mana",
      "black-turn-four-setup-cluster",
      "black-opening-early-same-lane-higher-score",
      "black-setup-progress-competition",
      "black-plain-spirit-setup-competition",
      "black-no-action-progress",
      "black-no-action-mana-sibling",
      "black-turn-four-window-mana-sibling",
      "black-turn-four-weak-window-recovery",
      "black-baseline-alignment",
      "black-turn-start-guarded-baseline-mana",
      "black-late-window-mana-safety",
      "black-late-window-competition",
      "black-late-recovery-progress-competition",
      "white-followup-mana",
      "white-mana-competition",
      "white-no-action-safe-progress-mana",
      "white-window-progress-competition",
      "white-action-mana-cluster",
      "white-setup-progress-competition",
      "white-early-followup-setup-competition",
      "white-opening-early-same-lane-higher-score",
      "white-early-setup-sibling-progress",
      "white-early-safe-progress-setup-competition",
      "white-early-no-action-progress-competition",
      "white-turn-three-safe-progress-surface",
      "white-turn-three-baseline-alignment",
      "white-turn-three-attack-bridge-escape",
      "white-turn-three-no-action-recovery",
      "white-mana-only-competition",
      "white-turn-five-weak-window-setup",
      "white-early-setup-competition",
      "white-late-followup-competition",
      "black-late-followup-competition",
      "black-late-reply-risk-setup",
      "black-late-weak-window-safe-progress-setup",
    ]);
    expect(Object.isFrozen(ADVISOR_SELECTION_RULES)).toBe(true);
    expect(ADVISOR_SELECTION_RULES.every(Object.isFrozen)).toBe(true);
  });

  it("keeps evaluating in order and lets later selections override", () => {
    const visited: string[] = [];
    const rules: readonly AdvisorSelectionRule[] = [
      {
        id: "first",
        evaluate(context) {
          visited.push(`first:${context.chosenIndex}`);
          return {
            kind: "select",
            index: 2,
            reason: ProductionRootAdvisorReasonCode.ApprovedFamilyCompetition,
          };
        },
      },
      {
        id: "continue",
        evaluate(context) {
          visited.push(`continue:${context.chosenIndex}`);
          return { kind: "continue" };
        },
      },
      {
        id: "last",
        evaluate(context) {
          visited.push(`last:${context.chosenIndex}`);
          return {
            kind: "select",
            index: 4,
            reason: ProductionRootAdvisorReasonCode.ApprovedBaselineSelector,
          };
        },
      },
    ];
    expect(visited).toEqual([]);
    const result = applyAdvisorSelectionRuleRegistry(
      {} as Omit<AdvisorSelectionRuleContext, "chosenIndex">,
      {
        index: 0,
        reason: ProductionRootAdvisorReasonCode.RankedRoot,
      },
      rules,
    );
    expect(visited).toEqual(["first:0", "continue:2", "last:2"]);
    expect(result).toEqual({
      index: 4,
      reason: ProductionRootAdvisorReasonCode.ApprovedBaselineSelector,
    });
  });
});
