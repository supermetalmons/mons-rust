import type { AutomoveExecutionContext } from "../../core/execution-context.js";
import type { EvaluatedRoot } from "../../root/types.js";
import type { AutomoveConfig } from "../../config/types.js";
import {
  contextualReplyRiskDecision,
  immediateReplyRiskDecision,
} from "./arbitration-primary.js";
import {
  plainSpiritReplyRiskDecision,
  sharedSpiritFollowupDecision,
} from "./arbitration-spirit.js";
import { finalReplyRiskDecision } from "./arbitration-tiebreak.js";
import type {
  FullReplyRiskComparisonContext,
  ReplyRiskComparisonContext,
  RootReplyRiskSnapshot,
} from "./types.js";

export function isBetterReplyRiskCandidate(
  execution: AutomoveExecutionContext,
  candidate: EvaluatedRoot,
  candidateSnapshot: RootReplyRiskSnapshot,
  incumbent: EvaluatedRoot,
  incumbentSnapshot: RootReplyRiskSnapshot,
  config: AutomoveConfig,
  context: ReplyRiskComparisonContext = {},
): boolean {
  const immediateDecision = immediateReplyRiskDecision(
    candidate,
    candidateSnapshot,
    incumbent,
    incumbentSnapshot,
  );
  if (immediateDecision !== undefined) return immediateDecision;

  const hasFullContext =
    context.game !== undefined &&
    context.evaluations !== undefined &&
    context.candidateIndex !== undefined &&
    context.incumbentIndex !== undefined &&
    context.perspective !== undefined;
  const fullContext = hasFullContext
    ? (context as FullReplyRiskComparisonContext)
    : undefined;
  const followupScores = context.spiritFollowupScores ?? new Map<number, number>();

  const contextualDecision = contextualReplyRiskDecision(
    execution,
    candidate,
    candidateSnapshot,
    incumbent,
    incumbentSnapshot,
    config,
    context,
    fullContext,
    followupScores,
  );
  if (contextualDecision !== undefined) return contextualDecision;

  const plainSpiritDecision = plainSpiritReplyRiskDecision(
    execution,
    candidate,
    candidateSnapshot,
    incumbent,
    incumbentSnapshot,
    config,
    context,
    fullContext,
    followupScores,
  );
  if (plainSpiritDecision !== undefined) return plainSpiritDecision;

  const followupDecision = sharedSpiritFollowupDecision(
    execution,
    config,
    fullContext,
    followupScores,
  );
  if (followupDecision !== undefined) return followupDecision;

  return finalReplyRiskDecision(
    candidate,
    candidateSnapshot,
    incumbent,
    incumbentSnapshot,
    config,
    context,
  );
}
