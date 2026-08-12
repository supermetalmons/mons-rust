import type { Color } from "../../api/types.js";
import { MonsGame } from "../../engine/game/mons-game.js";
import { BOARD_SIZE, locationIndex } from "../../engine/board/geometry.js";
import type { AutomoveExecutionContext } from "../core/execution-context.js";
import {
  hash64FromLowWord,
  hash64Mul,
  hash64RotateLeft,
  hash64Xor,
  type Hash64,
} from "../core/hash64.js";
import { MAX_SCORE, MIN_SCORE, saturatingScoreAdd } from "../core/score-math.js";
import { actionIdentity } from "./action-rules.js";
import {
  ownDrainerSafetyScore,
  quickOrderScoreWithSearchHash,
  turnOracleContextWithSearchHash,
  type TurnUtilityEvalContext,
} from "./evaluation.js";
import {
  FNV_OFFSET_BASIS,
  FNV_PRIME,
  HASH64_ALL_ONES,
  HASH64_ALL_ONES_EXCEPT_LOW_BIT,
  TURN_PLAN_FAMILY_PRIORITY_ORDER,
  TurnPlanFamily,
  type MacroOpportunity,
  type OpportunityDelta,
  type TurnAction,
  type TurnEngineConfig,
  type TurnOpportunity,
  type TurnOracleContext,
} from "./model.js";
import { discoverTurnOpportunities, opportunityScore } from "./opportunities.js";
import {
  actionKeyTuple,
  compareActionKeys,
  compareNumber,
  familyRank,
} from "./ordering.js";

export function bundleChunkCapForConfig(config: TurnEngineConfig): number {
  return Math.min(Math.max(config.stepCap, 1), 6);
}

export function bundlePlanCapForConfig(config: TurnEngineConfig): number {
  return Math.min(Math.max(config.stepCap, 1), 4);
}

export function mergePlanFamily(
  current: TurnPlanFamily,
  next: TurnPlanFamily,
): TurnPlanFamily {
  return familyRank(next) < familyRank(current) ? next : current;
}

function macroFollowupFamilyAllowed(
  head: TurnPlanFamily,
  goal: TurnPlanFamily,
  candidate: TurnPlanFamily,
): boolean {
  if (candidate === goal || candidate === head) return true;
  switch (head) {
    case TurnPlanFamily.ImmediateScore:
      return (
        candidate === TurnPlanFamily.ImmediateScore ||
        candidate === TurnPlanFamily.DrainerSafetyRecovery ||
        candidate === TurnPlanFamily.SafeSupermanaProgress ||
        candidate === TurnPlanFamily.SafeOpponentManaProgress
      );
    case TurnPlanFamily.DenyOpponentWindow:
    case TurnPlanFamily.DrainerKill:
      return (
        candidate === TurnPlanFamily.ImmediateScore ||
        candidate === TurnPlanFamily.DenyOpponentWindow ||
        candidate === TurnPlanFamily.DrainerKill ||
        candidate === TurnPlanFamily.DrainerSafetyRecovery ||
        candidate === TurnPlanFamily.SafeSupermanaProgress ||
        candidate === TurnPlanFamily.SafeOpponentManaProgress
      );
    case TurnPlanFamily.DrainerSafetyRecovery:
      return (
        candidate === TurnPlanFamily.ImmediateScore ||
        candidate === TurnPlanFamily.DrainerSafetyRecovery ||
        candidate === TurnPlanFamily.SafeSupermanaProgress ||
        candidate === TurnPlanFamily.SafeOpponentManaProgress ||
        candidate === TurnPlanFamily.ManaTempo
      );
    case TurnPlanFamily.SpiritImpact:
      return (
        candidate === TurnPlanFamily.ImmediateScore ||
        candidate === TurnPlanFamily.DenyOpponentWindow ||
        candidate === TurnPlanFamily.SpiritImpact ||
        candidate === TurnPlanFamily.SafeSupermanaProgress ||
        candidate === TurnPlanFamily.SafeOpponentManaProgress ||
        candidate === TurnPlanFamily.DrainerSafetyRecovery
      );
    case TurnPlanFamily.SafeSupermanaProgress:
    case TurnPlanFamily.SafeOpponentManaProgress:
      return (
        candidate === TurnPlanFamily.ImmediateScore ||
        candidate === TurnPlanFamily.DrainerSafetyRecovery ||
        candidate === TurnPlanFamily.SafeSupermanaProgress ||
        candidate === TurnPlanFamily.SafeOpponentManaProgress ||
        candidate === TurnPlanFamily.DenyOpponentWindow ||
        candidate === TurnPlanFamily.SpiritImpact
      );
    case TurnPlanFamily.ManaTempo:
      return (
        candidate === TurnPlanFamily.ImmediateScore ||
        candidate === TurnPlanFamily.DrainerSafetyRecovery ||
        candidate === TurnPlanFamily.SafeSupermanaProgress ||
        candidate === TurnPlanFamily.SafeOpponentManaProgress ||
        candidate === TurnPlanFamily.SpiritImpact ||
        candidate === TurnPlanFamily.ManaTempo
      );
  }
}

export function macroFollowupFamilyBonus(
  head: TurnPlanFamily,
  goal: TurnPlanFamily,
  candidate: TurnPlanFamily,
): number {
  let bonus = 0;
  if (candidate === goal) bonus += 420;
  if (candidate === head) bonus += 220;
  if (candidate === TurnPlanFamily.ImmediateScore) bonus += 640;
  if (
    head === TurnPlanFamily.SpiritImpact &&
    (candidate === TurnPlanFamily.SpiritImpact ||
      candidate === TurnPlanFamily.ImmediateScore ||
      candidate === TurnPlanFamily.SafeSupermanaProgress ||
      candidate === TurnPlanFamily.SafeOpponentManaProgress)
  ) {
    bonus += 180;
  }
  if (
    (head === TurnPlanFamily.SafeSupermanaProgress ||
      head === TurnPlanFamily.SafeOpponentManaProgress) &&
    (candidate === TurnPlanFamily.SafeSupermanaProgress ||
      candidate === TurnPlanFamily.SafeOpponentManaProgress ||
      candidate === TurnPlanFamily.ImmediateScore)
  ) {
    bonus += 180;
  }
  if (
    head === TurnPlanFamily.DrainerSafetyRecovery &&
    (candidate === TurnPlanFamily.DrainerSafetyRecovery ||
      candidate === TurnPlanFamily.SafeSupermanaProgress ||
      candidate === TurnPlanFamily.SafeOpponentManaProgress ||
      candidate === TurnPlanFamily.ImmediateScore)
  ) {
    bonus += 160;
  }
  return bonus;
}

export function macroFollowupFamilies(
  head: TurnPlanFamily,
  goal: TurnPlanFamily,
): TurnPlanFamily[] {
  return TURN_PLAN_FAMILY_PRIORITY_ORDER.filter((candidate) =>
    macroFollowupFamilyAllowed(head, goal, candidate),
  );
}

export function macroFollowupSeedCandidates(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  gameHash: Hash64,
  perspective: Color,
  config: TurnEngineConfig,
  head: TurnPlanFamily,
  goal: TurnPlanFamily,
  usedActions: readonly TurnAction[],
): TurnOpportunity[] {
  const oracle = turnOracleContextWithSearchHash(
    execution,
    game,
    perspective,
    gameHash,
  );
  const emergency =
    oracle.opportunity.opponentCanWinImmediately ||
    oracle.opportunity.delta.drainerSafety < 0;
  const used = new Set(usedActions.map(actionIdentity));
  const candidates = discoverTurnOpportunities(
    execution,
    game,
    perspective,
    config,
    Math.max(Math.max(config.ownSeedCap, config.perNodeFamilyCap * 3), 8),
    macroFollowupFamilies(head, goal),
  ).filter((opportunity) => !used.has(actionIdentity(opportunity.action)));
  candidates.sort((left, right) => {
    const scoreOrder = compareNumber(
      opportunityScore(right, emergency) +
        macroFollowupFamilyBonus(head, goal, right.family),
      opportunityScore(left, emergency) +
        macroFollowupFamilyBonus(head, goal, left.family),
    );
    return scoreOrder !== 0 ? scoreOrder : compareActionKeys(left.action, right.action);
  });
  return candidates.slice(0, Math.max(Math.max(config.perNodeFamilyCap, 1) * 2, 4));
}

function progressStepGain(
  before: number | undefined,
  after: number | undefined,
): number {
  const unknown = BOARD_SIZE * 3;
  return Math.max((before ?? unknown) - (after ?? unknown), 0);
}

export function macroOpportunityDelta(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  endGame: MonsGame,
  endHash: Hash64,
  perspective: Color,
  startOracle: TurnOracleContext,
): OpportunityDelta {
  if (execution.session.checkpoint()) return emptyOpportunityDelta();
  const endOracle = turnOracleContextWithSearchHash(
    execution,
    endGame,
    perspective,
    endHash,
  );
  if (execution.session.cancelled) return emptyOpportunityDelta();
  return {
    sameTurnScoreWindowGain: Math.max(
      endOracle.strategic.immediateWindow.bestScore -
        startOracle.strategic.immediateWindow.bestScore,
      0,
    ),
    spiritGain: Math.max(
      endOracle.strategic.spirit.nextTurnSetupGain -
        startOracle.strategic.spirit.nextTurnSetupGain,
      endOracle.strategic.spirit.utility - startOracle.strategic.spirit.utility,
      0,
    ),
    opponentWindowDenyGain: Math.max(
      startOracle.opponentImmediateWindow - endOracle.opponentImmediateWindow,
      0,
    ),
    drainerAttack: endOracle.opportunity.delta.drainerAttackAvailable,
    drainerSafetyDelta:
      ownDrainerSafetyScore(execution, endGame.board, perspective) -
      ownDrainerSafetyScore(execution, game.board, perspective),
    supermanaProgressGain: progressStepGain(
      startOracle.opportunity.delta.safeSupermanaProgressSteps,
      endOracle.opportunity.delta.safeSupermanaProgressSteps,
    ),
    opponentManaProgressGain: progressStepGain(
      startOracle.opportunity.delta.safeOpponentManaProgressSteps,
      endOracle.opportunity.delta.safeOpponentManaProgressSteps,
    ),
  };
}

function emptyOpportunityDelta(): OpportunityDelta {
  return {
    sameTurnScoreWindowGain: 0,
    spiritGain: 0,
    opponentWindowDenyGain: 0,
    drainerAttack: false,
    drainerSafetyDelta: 0,
    supermanaProgressGain: 0,
    opponentManaProgressGain: 0,
  };
}

export function macroPriorityFromState(
  utilityContext: TurnUtilityEvalContext,
  endGame: MonsGame,
  endHash: Hash64,
  family: TurnPlanFamily,
  chunkCount: number,
  priorityHint: number,
): number {
  return saturatingScoreAdd(
    priorityHint,
    Math.max(
      MIN_SCORE,
      Math.min(
        MAX_SCORE,
        Math.trunc(
          quickOrderScoreWithSearchHash(
            utilityContext,
            endGame,
            endHash,
            family,
            chunkCount,
          ) / 1_024,
        ),
      ),
    ),
  );
}

function macroSignatureMix(hash: Hash64, value: Hash64): Hash64 {
  return hash64RotateLeft(hash64Mul(hash64Xor(hash, value), FNV_PRIME), 11);
}

export function macroSignatureForActions(actions: readonly TurnAction[]): Hash64 {
  let hash = FNV_OFFSET_BASIS;
  for (const action of actions) {
    const [tag, first, second, third] = actionKeyTuple(action);
    hash = macroSignatureMix(hash, hash64FromLowWord(tag));
    hash = macroSignatureMix(hash, hash64FromLowWord(locationIndex(first)));
    hash = macroSignatureMix(
      hash,
      second === undefined ? HASH64_ALL_ONES : hash64FromLowWord(locationIndex(second)),
    );
    hash = macroSignatureMix(
      hash,
      third === undefined
        ? HASH64_ALL_ONES_EXCEPT_LOW_BIT
        : hash64FromLowWord(locationIndex(third)),
    );
  }
  return hash;
}

export function macroPlanSignature(
  previous: Hash64,
  opportunity: MacroOpportunity,
): Hash64 {
  return macroSignatureMix(
    macroSignatureMix(previous, opportunity.endSnapshot.stateHash),
    opportunity.signature,
  );
}
