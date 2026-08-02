import type { Color } from "../../engine/domain.js";
import type { MonsGame } from "../../engine/game.js";
import type { AutomoveExecutionContext } from "../execution-context.js";
import type { EvaluatedRoot } from "../search.js";
import type { AutomoveConfig } from "../selector-types.js";
import {
  blackBaselineAlignmentOverride,
  blackTurnFourWindowManaSiblingOverride,
  blackTurnStartGuardedBaselineManaOverride,
  whiteTurnThreeAttackBridgeEscape,
  whiteTurnThreeBaselineAlignmentOverride,
} from "./policies/black-baseline.js";
import {
  blackFamilyCompetitionOverride,
  blackLateFollowupCompetitionOverride,
  blackLateReplyRiskSetupOverride,
  blackLateWeakWindowSafeProgressSetupOverride,
} from "./policies/black-family.js";
import {
  blackLateRecoveryProgressCompetitionOverride,
  blackLateWindowCompetitionOverride,
  blackLateWindowManaSafetyOverride,
} from "./policies/black-late.js";
import {
  blackEarlyPlainSpiritFollowupOverride,
  blackEarlySafeManaFollowupOverride,
  blackOpeningSetupSiblingOverride,
  blackTurnFourSetupClusterOverride,
  blackTurnFourVulnerableProgressManaOverride,
  blackTurnSixAttackVulnerableProgressManaOverride,
} from "./policies/black-opening.js";
import {
  blackNoActionManaSiblingOverride,
  blackNoActionProgressOverride,
  blackPlainSpiritSetupCompetitionOverride,
} from "./policies/black-progress.js";
import {
  blackSetupProgressCompetitionOverride,
  earlySameLaneHigherScoreOverride,
  whiteTurnFiveWeakWindowSetupOverride,
} from "./policies/cross-color.js";
import {
  blackTurnFourWeakWindowRecoveryOverride,
  whiteTurnThreeNoActionRecoveryOverride,
} from "./policies/no-action-recovery.js";
import {
  whiteEarlySafeProgressSetupCompetitionOverride,
  whiteEarlySetupCompetitionOverride,
  whiteLateFollowupCompetitionOverride,
} from "./policies/white-late.js";
import {
  whiteActionManaClusterOverride,
  whiteFollowupManaOverride,
  whiteManaCompetitionOverride,
  whiteNoActionSafeProgressManaOverride,
  whiteWindowProgressCompetitionOverride,
} from "./policies/white-mana-progress.js";
import {
  whiteEarlyFollowupSetupCompetitionOverride,
  whiteEarlyNoActionProgressCompetitionOverride,
  whiteEarlySetupSiblingProgressOverride,
  whiteManaOnlyCompetitionOverride,
  whiteSetupProgressCompetitionOverride,
  whiteTurnThreeSafeProgressSurfaceOverride,
} from "./policies/white-opening.js";
import { ProductionRootAdvisorReasonCode } from "./types.js";

type AdvisorSelectionRuleResult =
  | { readonly kind: "continue" }
  | {
      readonly kind: "select";
      readonly index: number;
      readonly reason: ProductionRootAdvisorReasonCode;
    };

type AdvisorSelection = {
  readonly index: number;
  readonly reason: ProductionRootAdvisorReasonCode;
};

type AdvisorSelectionRuleContext = {
  readonly execution: AutomoveExecutionContext;
  readonly game: MonsGame;
  readonly roots: readonly EvaluatedRoot[];
  readonly replyRiskShortlist: readonly number[];
  readonly selectionIndices: readonly number[];
  readonly candidateIndices: readonly number[];
  readonly chosenIndex: number;
  readonly perspective: Color;
  readonly config: AutomoveConfig;
  readonly baselineIndex: number | undefined;
};

type AdvisorSelectionRule<Id extends string = string> = {
  readonly id: Id;
  readonly evaluate: (
    context: AdvisorSelectionRuleContext,
  ) => AdvisorSelectionRuleResult;
};

type SelectionIndex = (
  context: AdvisorSelectionRuleContext,
) => number | undefined;

const CONTINUE: AdvisorSelectionRuleResult = Object.freeze({
  kind: "continue",
});

function selectionRule<const Id extends string>(
  id: Id,
  reason: ProductionRootAdvisorReasonCode,
  select: SelectionIndex,
): AdvisorSelectionRule<Id> {
  return Object.freeze({
    id,
    evaluate(context): AdvisorSelectionRuleResult {
      const index = select(context);
      return index === undefined ? CONTINUE : { kind: "select", index, reason };
    },
  });
}

function familyRule<const Id extends string>(
  id: Id,
  select: SelectionIndex,
): AdvisorSelectionRule<Id> {
  return selectionRule(
    id,
    ProductionRootAdvisorReasonCode.ApprovedFamilyCompetition,
    select,
  );
}

function baselineRule<const Id extends string>(
  id: Id,
  select: SelectionIndex,
): AdvisorSelectionRule<Id> {
  return selectionRule(
    id,
    ProductionRootAdvisorReasonCode.ApprovedBaselineSelector,
    select,
  );
}

const ADVISOR_SELECTION_RULES = Object.freeze([
  familyRule("black-family-competition", (context) =>
    blackFamilyCompetitionOverride(
      context.execution,
      context.game,
      context.roots,
      context.replyRiskShortlist,
      context.selectionIndices,
      context.chosenIndex,
      context.perspective,
      context.config,
    ),
  ),
  familyRule("black-opening-setup-sibling", (context) =>
    blackOpeningSetupSiblingOverride(
      context.game,
      context.roots,
      context.selectionIndices,
      context.chosenIndex,
      context.config,
    ),
  ),
  familyRule("black-early-safe-mana-followup", (context) =>
    blackEarlySafeManaFollowupOverride(
      context.execution,
      context.game,
      context.roots,
      context.selectionIndices,
      context.chosenIndex,
      context.config,
    ),
  ),
  familyRule("black-early-plain-spirit-followup", (context) =>
    blackEarlyPlainSpiritFollowupOverride(
      context.execution,
      context.game,
      context.roots,
      context.selectionIndices,
      context.chosenIndex,
      context.config,
    ),
  ),
  familyRule("black-turn-four-vulnerable-progress-mana", (context) =>
    blackTurnFourVulnerableProgressManaOverride(
      context.execution,
      context.game,
      context.roots,
      context.selectionIndices,
      context.chosenIndex,
      context.config,
    ),
  ),
  familyRule("black-turn-six-attack-vulnerable-progress-mana", (context) =>
    blackTurnSixAttackVulnerableProgressManaOverride(
      context.execution,
      context.game,
      context.roots,
      context.selectionIndices,
      context.chosenIndex,
      context.config,
    ),
  ),
  familyRule("black-turn-four-setup-cluster", (context) =>
    blackTurnFourSetupClusterOverride(
      context.execution,
      context.game,
      context.roots,
      context.selectionIndices,
      context.chosenIndex,
      context.config,
    ),
  ),
  familyRule("black-opening-early-same-lane-higher-score", (context) =>
    earlySameLaneHigherScoreOverride(
      context.execution,
      context.game,
      context.roots,
      context.selectionIndices,
      context.chosenIndex,
      context.config,
    ),
  ),
  familyRule("black-setup-progress-competition", (context) =>
    blackSetupProgressCompetitionOverride(
      context.execution,
      context.game,
      context.roots,
      context.selectionIndices,
      context.chosenIndex,
      context.perspective,
      context.config,
    ),
  ),
  familyRule("black-plain-spirit-setup-competition", (context) =>
    blackPlainSpiritSetupCompetitionOverride(
      context.execution,
      context.game,
      context.roots,
      context.selectionIndices,
      context.chosenIndex,
      context.config,
    ),
  ),
  familyRule("black-no-action-progress", (context) =>
    blackNoActionProgressOverride(
      context.execution,
      context.game,
      context.roots,
      context.selectionIndices,
      context.chosenIndex,
      context.config,
    ),
  ),
  familyRule("black-no-action-mana-sibling", (context) =>
    blackNoActionManaSiblingOverride(
      context.game,
      context.roots,
      context.selectionIndices,
      context.chosenIndex,
      context.config,
    ),
  ),
  familyRule("black-turn-four-window-mana-sibling", (context) =>
    blackTurnFourWindowManaSiblingOverride(
      context.execution,
      context.game,
      context.roots,
      context.selectionIndices,
      context.chosenIndex,
      context.config,
    ),
  ),
  familyRule("black-turn-four-weak-window-recovery", (context) =>
    blackTurnFourWeakWindowRecoveryOverride(
      context.execution,
      context.game,
      context.roots,
      context.chosenIndex,
      context.config,
    ),
  ),
  baselineRule("black-baseline-alignment", (context) =>
    context.baselineIndex === undefined
      ? undefined
      : blackBaselineAlignmentOverride(
          context.execution,
          context.game,
          context.roots,
          context.selectionIndices,
          context.chosenIndex,
          context.baselineIndex,
          context.config,
        ),
  ),
  baselineRule("black-turn-start-guarded-baseline-mana", (context) =>
    blackTurnStartGuardedBaselineManaOverride(
      context.execution,
      context.game,
      context.roots,
      context.candidateIndices,
      context.chosenIndex,
      context.config,
    ),
  ),
  familyRule("black-late-window-mana-safety", (context) =>
    blackLateWindowManaSafetyOverride(
      context.game,
      context.roots,
      context.selectionIndices,
      context.chosenIndex,
      context.config,
    ),
  ),
  familyRule("black-late-window-competition", (context) =>
    blackLateWindowCompetitionOverride(
      context.execution,
      context.game,
      context.roots,
      context.chosenIndex,
      context.config,
    ),
  ),
  familyRule("black-late-recovery-progress-competition", (context) =>
    blackLateRecoveryProgressCompetitionOverride(
      context.game,
      context.roots,
      context.selectionIndices,
      context.chosenIndex,
      context.config,
    ),
  ),
  familyRule("white-followup-mana", (context) =>
    whiteFollowupManaOverride(
      context.execution,
      context.game,
      context.roots,
      context.selectionIndices,
      context.chosenIndex,
      context.perspective,
      context.config,
    ),
  ),
  familyRule("white-mana-competition", (context) =>
    whiteManaCompetitionOverride(
      context.execution,
      context.game,
      context.roots,
      context.selectionIndices,
      context.chosenIndex,
      context.perspective,
      context.config,
    ),
  ),
  familyRule("white-no-action-safe-progress-mana", (context) =>
    whiteNoActionSafeProgressManaOverride(
      context.execution,
      context.game,
      context.roots,
      context.selectionIndices,
      context.chosenIndex,
      context.config,
    ),
  ),
  familyRule("white-window-progress-competition", (context) =>
    whiteWindowProgressCompetitionOverride(
      context.execution,
      context.game,
      context.roots,
      context.selectionIndices,
      context.chosenIndex,
      context.perspective,
      context.config,
    ),
  ),
  familyRule("white-action-mana-cluster", (context) =>
    whiteActionManaClusterOverride(
      context.game,
      context.roots,
      context.selectionIndices,
      context.chosenIndex,
      context.config,
    ),
  ),
  familyRule("white-setup-progress-competition", (context) =>
    whiteSetupProgressCompetitionOverride(
      context.execution,
      context.game,
      context.roots,
      context.selectionIndices,
      context.chosenIndex,
      context.perspective,
      context.config,
    ),
  ),
  familyRule("white-early-followup-setup-competition", (context) =>
    whiteEarlyFollowupSetupCompetitionOverride(
      context.game,
      context.roots,
      context.selectionIndices,
      context.chosenIndex,
      context.config,
    ),
  ),
  familyRule("white-opening-early-same-lane-higher-score", (context) =>
    earlySameLaneHigherScoreOverride(
      context.execution,
      context.game,
      context.roots,
      context.selectionIndices,
      context.chosenIndex,
      context.config,
    ),
  ),
  familyRule("white-early-setup-sibling-progress", (context) =>
    whiteEarlySetupSiblingProgressOverride(
      context.execution,
      context.game,
      context.roots,
      context.chosenIndex,
      context.config,
    ),
  ),
  familyRule("white-early-safe-progress-setup-competition", (context) =>
    whiteEarlySafeProgressSetupCompetitionOverride(
      context.game,
      context.roots,
      context.selectionIndices,
      context.chosenIndex,
      context.config,
    ),
  ),
  familyRule("white-early-no-action-progress-competition", (context) =>
    whiteEarlyNoActionProgressCompetitionOverride(
      context.game,
      context.roots,
      context.selectionIndices,
      context.chosenIndex,
      context.config,
    ),
  ),
  familyRule("white-turn-three-safe-progress-surface", (context) =>
    whiteTurnThreeSafeProgressSurfaceOverride(
      context.game,
      context.roots,
      context.selectionIndices,
      context.chosenIndex,
      context.config,
    ),
  ),
  baselineRule("white-turn-three-baseline-alignment", (context) =>
    context.baselineIndex === undefined
      ? undefined
      : whiteTurnThreeBaselineAlignmentOverride(
          context.execution,
          context.game,
          context.roots,
          context.selectionIndices,
          context.chosenIndex,
          context.baselineIndex,
          context.config,
        ),
  ),
  familyRule("white-turn-three-attack-bridge-escape", (context) =>
    whiteTurnThreeAttackBridgeEscape(
      context.execution,
      context.game,
      context.roots,
      context.selectionIndices,
      context.chosenIndex,
      context.config,
    ),
  ),
  familyRule("white-turn-three-no-action-recovery", (context) =>
    whiteTurnThreeNoActionRecoveryOverride(
      context.execution,
      context.game,
      context.roots,
      context.chosenIndex,
      context.config,
    ),
  ),
  familyRule("white-mana-only-competition", (context) =>
    whiteManaOnlyCompetitionOverride(
      context.game,
      context.roots,
      context.selectionIndices,
      context.chosenIndex,
      context.config,
    ),
  ),
  familyRule("white-turn-five-weak-window-setup", (context) =>
    whiteTurnFiveWeakWindowSetupOverride(
      context.execution,
      context.game,
      context.roots,
      context.selectionIndices,
      context.chosenIndex,
      context.config,
    ),
  ),
  familyRule("white-early-setup-competition", (context) =>
    whiteEarlySetupCompetitionOverride(
      context.execution,
      context.game,
      context.roots,
      context.selectionIndices,
      context.chosenIndex,
      context.config,
    ),
  ),
  familyRule("white-late-followup-competition", (context) =>
    whiteLateFollowupCompetitionOverride(
      context.game,
      context.roots,
      context.chosenIndex,
      context.config,
    ),
  ),
  familyRule("black-late-followup-competition", (context) =>
    blackLateFollowupCompetitionOverride(
      context.game,
      context.roots,
      context.selectionIndices,
      context.chosenIndex,
      context.config,
    ),
  ),
  familyRule("black-late-reply-risk-setup", (context) =>
    blackLateReplyRiskSetupOverride(
      context.execution,
      context.game,
      context.roots,
      context.replyRiskShortlist,
      context.chosenIndex,
      context.config,
    ),
  ),
  familyRule("black-late-weak-window-safe-progress-setup", (context) =>
    blackLateWeakWindowSafeProgressSetupOverride(
      context.execution,
      context.game,
      context.roots,
      context.replyRiskShortlist,
      context.chosenIndex,
      context.perspective,
      context.config,
    ),
  ),
] as const satisfies readonly AdvisorSelectionRule[]);

export function applyAdvisorSelectionRules(
  context: Omit<AdvisorSelectionRuleContext, "chosenIndex">,
  initialSelection: AdvisorSelection,
): AdvisorSelection {
  let selection = initialSelection;
  for (const rule of ADVISOR_SELECTION_RULES) {
    const result = rule.evaluate({
      ...context,
      chosenIndex: selection.index,
    });
    if (result.kind === "select") {
      selection = { index: result.index, reason: result.reason };
    }
  }
  return selection;
}
