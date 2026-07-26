import { Color } from "../../engine/domain.js";
import { MonsGame } from "../../engine/game.js";
import type { AutomoveExecutionContext } from "../execution-context.js";
import {
  pickRootWithReplyRiskGuard,
  replyRiskGuardShortlistIndices,
  rootReplyRiskSnapshot,
} from "../reply-risk.js";
import { rootFamily as advisorRootFamily } from "../root-family.js";
import {
  compareRankedEvaluatedRootIndices,
  filteredRootCandidateIndices,
  pickBaselineRootIndexFromCandidateIndices,
} from "../root-selector.js";
import type { EvaluatedRoot } from "../search.js";
import {
  AUTOMOVE_TURN_ENGINE_MODE,
  isPlainSpiritDevelopmentRoot,
  productionEnabled,
} from "../selector-types.js";
import type { AutomoveConfig } from "../selector-types.js";
import { TurnPlanFamily } from "../turn-engine.js";
import { representativeCompetesInApproval } from "./approval.js";
import {
  collectAdvisorReentries,
  findScoredRepresentative,
  whiteFollowupRepresentative,
} from "./reentries.js";
import { buildRootPolicy } from "./root-policy-core.js";
import { applyAdvisorSelectionRules } from "./selection-rules.js";
import { entry, pushUnique, withoutReplyRiskGuard } from "./support.js";
import { ProductionRootAdvisorReasonCode } from "./types.js";
import type {
  ProductionRootAdvisorEntry,
  ProductionRootAdvisorPostsearchResult,
} from "./types.js";

function addAdvisorReentry(
  roots: readonly EvaluatedRoot[],
  decisionEntries: ProductionRootAdvisorEntry[],
  preserved: ProductionRootAdvisorEntry[],
  orderedShortlist: number[],
  selectionIndices: number[],
  approvalShortlist: number[],
  index: number,
  reason: ProductionRootAdvisorReasonCode = ProductionRootAdvisorReasonCode.OmittedRootReentry,
): void {
  const root = roots[index];
  if (root === undefined) return;
  const value = entry(root, reason);
  pushUnique(preserved, value);
  if (!orderedShortlist.includes(index)) orderedShortlist.push(index);
  if (!selectionIndices.includes(index)) selectionIndices.push(index);
  if (!approvalShortlist.includes(index)) approvalShortlist.push(index);
  pushUnique(decisionEntries, value);
}

export function productionRootAdvisorPostsearch(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  roots: readonly EvaluatedRoot[],
  perspective: Color,
  config: AutomoveConfig,
): ProductionRootAdvisorPostsearchResult | undefined {
  if (
    !productionEnabled(config) ||
    roots.length === 0 ||
    execution.session.checkpoint()
  ) {
    return undefined;
  }
  const corePolicy = buildRootPolicy(execution);
  let candidateIndices = filteredRootCandidateIndices(
    game,
    roots,
    perspective,
    config,
    {
      rootReplyRiskSnapshot: (
        state,
        snapshotPerspective,
        _config,
        replyLimit,
      ) =>
        rootReplyRiskSnapshot(
          execution,
          state,
          snapshotPerspective,
          config,
          replyLimit,
        ),
      productionPolicy: corePolicy,
    },
  );
  if (candidateIndices.length === 0) {
    candidateIndices = roots.map((_root, index) => index);
  }
  let orderedShortlist = config.replyRisk.enabled
    ? replyRiskGuardShortlistIndices(execution, roots, candidateIndices, config)
    : [...candidateIndices];
  if (orderedShortlist.length === 0) {
    orderedShortlist = [...candidateIndices];
  }
  const replyRiskShortlist = [...orderedShortlist];
  const approvalShortlist = [...orderedShortlist];
  const selectionIndices = [...candidateIndices];
  const decisionEntries: ProductionRootAdvisorEntry[] = [];
  const preserved: ProductionRootAdvisorEntry[] = [];
  for (const index of orderedShortlist) {
    const root = roots[index];
    if (root !== undefined) {
      pushUnique(
        decisionEntries,
        entry(root, ProductionRootAdvisorReasonCode.ReplyRiskShortlist),
      );
    }
  }
  const followupScores = new Map<number, number>();
  const specs: readonly (readonly [
    ProductionRootAdvisorReasonCode,
    (root: EvaluatedRoot) => boolean,
  ])[] = [
    [
      ProductionRootAdvisorReasonCode.PreserveSpiritRepresentative,
      (root) => root.spiritSameTurnScoreSetupNow || root.spiritOwnManaSetupNow,
    ],
    [
      ProductionRootAdvisorReasonCode.PreserveSpiritRepresentative,
      isPlainSpiritDevelopmentRoot,
    ],
    [
      ProductionRootAdvisorReasonCode.PreserveSafeProgressRepresentative,
      (root) =>
        advisorRootFamily(root) === TurnPlanFamily.SafeSupermanaProgress,
    ],
    [
      ProductionRootAdvisorReasonCode.PreserveSafeProgressRepresentative,
      (root) =>
        advisorRootFamily(root) === TurnPlanFamily.SafeOpponentManaProgress,
    ],
    [
      ProductionRootAdvisorReasonCode.PreserveManaTempoRepresentative,
      (root) => advisorRootFamily(root) === TurnPlanFamily.ManaTempo,
    ],
  ];
  for (const [reason, predicate] of specs) {
    if (execution.session.checkpoint()) return undefined;
    const index = findScoredRepresentative(
      execution,
      game,
      roots,
      orderedShortlist,
      perspective,
      config,
      predicate,
    );
    const root = index === undefined ? undefined : roots[index];
    if (index === undefined || root === undefined) continue;
    const value = entry(root, reason);
    pushUnique(preserved, value);
    if (!orderedShortlist.includes(index)) orderedShortlist.push(index);
    if (!selectionIndices.includes(index)) selectionIndices.push(index);
    pushUnique(decisionEntries, value);
    if (
      representativeCompetesInApproval(
        execution,
        game,
        roots,
        orderedShortlist,
        index,
        reason,
        perspective,
        config,
        followupScores,
      ) &&
      !approvalShortlist.includes(index)
    ) {
      approvalShortlist.push(index);
    }
  }
  const followupRepresentative = whiteFollowupRepresentative(
    game,
    roots,
    orderedShortlist,
    config,
  );
  if (followupRepresentative !== undefined) {
    addAdvisorReentry(
      roots,
      decisionEntries,
      preserved,
      orderedShortlist,
      selectionIndices,
      approvalShortlist,
      followupRepresentative,
      ProductionRootAdvisorReasonCode.PreserveSpiritRepresentative,
    );
  }
  for (const index of collectAdvisorReentries(
    execution,
    game,
    roots,
    candidateIndices,
    orderedShortlist,
    perspective,
    config,
  )) {
    addAdvisorReentry(
      roots,
      decisionEntries,
      preserved,
      orderedShortlist,
      selectionIndices,
      approvalShortlist,
      index,
    );
  }
  selectionIndices.sort((left, right) =>
    compareRankedEvaluatedRootIndices(roots, left, right),
  );
  const baselineConfig = withoutReplyRiskGuard(
    config,
    AUTOMOVE_TURN_ENGINE_MODE.Baseline,
  );
  const baselineIndex = pickBaselineRootIndexFromCandidateIndices(
    game,
    roots,
    candidateIndices,
    perspective,
    baselineConfig,
  );
  const shortlistConfig = withoutReplyRiskGuard(config, config.planner.mode);
  let approvedIndex: number | undefined;
  let approvedReason: ProductionRootAdvisorReasonCode;
  if (config.replyRisk.enabled) {
    approvedIndex = pickRootWithReplyRiskGuard(
      execution,
      game,
      roots,
      approvalShortlist,
      perspective,
      config,
      selectionIndices,
    );
  }
  if (approvedIndex !== undefined) {
    approvedReason = ProductionRootAdvisorReasonCode.ApprovedReplyRiskGuard;
  } else {
    approvedIndex = pickBaselineRootIndexFromCandidateIndices(
      game,
      roots,
      selectionIndices,
      perspective,
      shortlistConfig,
      { productionPolicy: corePolicy },
    );
    approvedReason = ProductionRootAdvisorReasonCode.ApprovedBaselineSelector;
  }
  const selection = applyAdvisorSelectionRules(
    {
      execution,
      game,
      roots,
      replyRiskShortlist,
      selectionIndices,
      candidateIndices,
      perspective,
      config,
      baselineIndex,
    },
    {
      index: approvedIndex ?? orderedShortlist[0] ?? candidateIndices[0] ?? 0,
      reason: approvedReason,
    },
  );
  const chosenIndex = selection.index;
  approvedReason = selection.reason;
  const approvedRoot = roots[chosenIndex];
  if (approvedRoot === undefined || execution.session.checkpoint())
    return undefined;
  const approvedEntry = entry(approvedRoot, approvedReason);
  return {
    index: chosenIndex,
    decision: {
      orderedShortlist: decisionEntries,
      preservedFamilyRepresentatives: preserved,
      approvedRoot: approvedEntry,
      injectedRoot: undefined,
    },
  };
}
