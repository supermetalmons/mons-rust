import { describe, expect, it } from "vitest";

import {
  ADVISOR_SELECTION_RULES,
  applyAdvisorSelectionRules,
  type AdvisorSelectionRule,
  type AdvisorSelectionRuleContext,
} from "../../src/automove/advisor/selection-rules.js";
import { ProductionRootAdvisorReasonCode } from "../../src/automove/advisor/types.js";

const EXPECTED_RULE_IDS = [
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
] as const;

describe("advisor postsearch selection rules", () => {
  it("publishes all 41 stable IDs once in their policy order", () => {
    const ids = ADVISOR_SELECTION_RULES.map(({ id }) => id);

    expect(ids).toEqual(EXPECTED_RULE_IDS);
    expect(new Set(ids).size).toBe(41);
    expect(Object.isFrozen(ADVISOR_SELECTION_RULES)).toBe(true);
  });

  it("passes the current selection left-to-right and lets later rules override", () => {
    const seenIndices: number[] = [];
    const rules: readonly AdvisorSelectionRule[] = [
      {
        id: "continue",
        evaluate(context) {
          seenIndices.push(context.chosenIndex);
          return { kind: "continue" };
        },
      },
      {
        id: "family",
        evaluate(context) {
          seenIndices.push(context.chosenIndex);
          return {
            kind: "select",
            index: 4,
            reason: ProductionRootAdvisorReasonCode.ApprovedFamilyCompetition,
          };
        },
      },
      {
        id: "baseline",
        evaluate(context) {
          seenIndices.push(context.chosenIndex);
          return {
            kind: "select",
            index: 7,
            reason: ProductionRootAdvisorReasonCode.ApprovedBaselineSelector,
          };
        },
      },
    ];

    const selection = applyAdvisorSelectionRules(
      {} as Omit<AdvisorSelectionRuleContext, "chosenIndex">,
      {
        index: 2,
        reason: ProductionRootAdvisorReasonCode.ApprovedReplyRiskGuard,
      },
      rules,
    );

    expect(seenIndices).toEqual([2, 2, 4]);
    expect(selection).toEqual({
      index: 7,
      reason: ProductionRootAdvisorReasonCode.ApprovedBaselineSelector,
    });
  });
});
