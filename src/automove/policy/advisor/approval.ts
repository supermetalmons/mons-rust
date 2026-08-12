import { Color } from "../../../api/types.js";
import { MonsGame } from "../../../engine/game/mons-game.js";
import type { AutomoveExecutionContext } from "../../core/execution-context.js";
import { spiritFollowupFloorScore } from "../reply-risk/projection.js";
import { MIN_SCORE, saturatingScoreAdd } from "../../core/score-math.js";
import type { EvaluatedRoot } from "../../root/types.js";
import {
  hasConcreteScoreSurface,
  isPlainSpiritDevelopmentRoot,
} from "../../config/types.js";
import type { AutomoveConfig } from "../../config/types.js";
import { advisorRootIsSafe, memoizedByIndex } from "./support.js";
import { inputChainsShareFirstInput as sameFirstInput } from "../../../engine/model/domain.js";
import { ProductionRootAdvisorReasonCode } from "./types.js";

function blackPlainSpiritRepresentativeCompetes(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  roots: readonly EvaluatedRoot[],
  shortlist: readonly number[],
  plainIndex: number,
  perspective: Color,
  config: AutomoveConfig,
  followupScores: Map<number, number>,
): boolean {
  const plain = roots[plainIndex];
  if (plain === undefined || !isPlainSpiritDevelopmentRoot(plain)) return false;
  const followup = memoizedByIndex((index): number => {
    const root = roots[index];
    return root === undefined
      ? MIN_SCORE
      : spiritFollowupFloorScore(execution, root.game, perspective, config);
  }, followupScores);
  let competes = false;
  for (const index of shortlist) {
    const setup = roots[index];
    if (
      setup === undefined ||
      game.activeColor !== Color.Black ||
      game.turnNumber > 4 ||
      !setup.spiritOwnManaSetupNow ||
      setup.spiritSameTurnScoreSetupNow ||
      !sameFirstInput(plain.inputs, setup.inputs) ||
      plain.ownDrainerVulnerable !== setup.ownDrainerVulnerable ||
      plain.ownDrainerWalkVulnerable !== setup.ownDrainerWalkVulnerable ||
      !advisorRootIsSafe(plain) ||
      !advisorRootIsSafe(setup) ||
      hasConcreteScoreSurface(plain) ||
      hasConcreteScoreSurface(setup) ||
      plain.attacksOpponentDrainer ||
      setup.attacksOpponentDrainer ||
      plain.sameTurnScoreWindowValue !== 0 ||
      setup.sameTurnScoreWindowValue !== 0 ||
      plain.supermanaProgress ||
      setup.supermanaProgress ||
      plain.opponentManaProgress ||
      setup.opponentManaProgress
    ) {
      continue;
    }
    const setupHasCloseTopSeed =
      setup.rootRank <= plain.rootRank &&
      saturatingScoreAdd(setup.score, 64) >= plain.score &&
      setup.spiritSetupGain >= saturatingScoreAdd(plain.spiritSetupGain, 32);
    if (setupHasCloseTopSeed) return false;
    const plainFollowup = followup(plainIndex);
    const setupFollowup = followup(index);
    competes ||=
      plainFollowup >= saturatingScoreAdd(setupFollowup, 32) ||
      (plain.score >= setup.score && plainFollowup >= setupFollowup);
  }
  return competes;
}

function representativeCompetesInApproval(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  roots: readonly EvaluatedRoot[],
  shortlist: readonly number[],
  index: number,
  reason: ProductionRootAdvisorReasonCode,
  perspective: Color,
  config: AutomoveConfig,
  followupScores: Map<number, number>,
): boolean {
  const root = roots[index];
  if (root === undefined) return false;
  if (reason !== ProductionRootAdvisorReasonCode.PreserveSpiritRepresentative) {
    return true;
  }
  return (
    root.spiritSameTurnScoreSetupNow ||
    root.spiritOwnManaSetupNow ||
    blackPlainSpiritRepresentativeCompetes(
      execution,
      game,
      roots,
      shortlist,
      index,
      perspective,
      config,
      followupScores,
    )
  );
}

export { representativeCompetesInApproval };
