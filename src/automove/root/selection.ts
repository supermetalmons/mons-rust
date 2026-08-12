import type { Color } from "../../api/types.js";
import type { Input } from "../../engine/model/domain.js";
import type { MonsGame } from "../../engine/game/mons-game.js";
import {
  MIN_SCORE,
  saturatingScoreAdd,
  saturatingScoreSubtract,
} from "../core/score-math.js";
import {
  AUTOMOVE_TURN_ENGINE_MODE,
  isPlainSpiritDevelopmentRoot,
  shouldPreferSpiritDevelopment,
  type AutomoveConfig,
} from "../config/types.js";
import { rootProgressStepsBetter, rootScorePathStepsBetter } from "./focus.js";
import type { EvaluatedRoot } from "./types.js";
import {
  ProductionComparisonPhase,
  type RootSelectionContext,
  type RootSelectorOptions,
} from "./selector-model.js";
import {
  compareProductionRules,
  bestScoredRootIndex,
  spiritScoreChallengeOrder,
} from "./evaluated-ordering.js";
import { filteredRootCandidateIndices } from "./filtering.js";

const INTERVIEW_SOFT_PRIORITY_SCORE_MARGIN = 80;

function valueAt<Value>(values: readonly Value[], index: number): Value {
  const value = values[index];
  if (value === undefined) {
    throw new RangeError(`root selector index ${index} is out of bounds`);
  }
  return value;
}

function assertRootIndex(
  roots: readonly EvaluatedRoot[],
  index: number,
  source: string,
): void {
  if (!Number.isInteger(index) || index < 0 || index >= roots.length) {
    throw new RangeError(`${source} selected an invalid root`);
  }
}

function maxValue(values: readonly number[], fallback: number): number {
  let best = fallback;
  for (const value of values) best = Math.max(best, value);
  return best;
}

function isProductionMode(config: AutomoveConfig): boolean {
  return config.planner.mode === AUTOMOVE_TURN_ENGINE_MODE.Production;
}

function selectionContext(
  game: MonsGame,
  roots: readonly EvaluatedRoot[],
  candidateIndices: readonly number[],
  perspective: Color,
  config: AutomoveConfig,
): RootSelectionContext {
  return { game, roots, candidateIndices, perspective, config };
}

export function pickBaselineRootIndexFromCandidateIndices(
  game: MonsGame,
  roots: readonly EvaluatedRoot[],
  candidateIndices: readonly number[],
  perspective: Color,
  config: AutomoveConfig,
  options: RootSelectorOptions = {},
): number | undefined {
  if (
    options.checkpoint?.() === true ||
    roots.length === 0 ||
    candidateIndices.length === 0
  ) {
    return undefined;
  }
  const context = selectionContext(game, roots, candidateIndices, perspective, config);
  if (config.replyRisk.enabled) {
    const guarded = options.pickReplyRiskGuardedIndex?.(context);
    if (guarded !== undefined) {
      assertRootIndex(roots, guarded, "reply-risk guard");
      if (!candidateIndices.includes(guarded)) {
        throw new RangeError("reply-risk guard selected a root outside its shortlist");
      }
      return options.cancelled?.() === true ? undefined : guarded;
    }
    if (options.cancelled?.() === true) return undefined;
  }

  const bestScore = maxValue(
    candidateIndices.map((index) => valueAt(roots, index).score),
    MIN_SCORE,
  );
  const scoreMargin = Math.max(config.policy.efficiencyScoreMargin, 0);
  let bestIndex = bestScoredRootIndex(roots, candidateIndices);
  let bestEfficiency = MIN_SCORE;
  let bestShortlistedScore = MIN_SCORE;
  const preferSpirit =
    config.policy.preferSpiritDevelopment &&
    shouldPreferSpiritDevelopment(game, perspective);
  const productionPlainSpiritProjectionTiebreak =
    isProductionMode(config) &&
    candidateIndices.filter((index) =>
      isPlainSpiritDevelopmentRoot(valueAt(roots, index)),
    ).length >= 2;

  for (const index of candidateIndices) {
    if (options.checkpoint?.() === true) return undefined;
    const evaluation = valueAt(roots, index);
    const best = valueAt(roots, bestIndex);
    const allowClosePlainSpiritSlack =
      isProductionMode(config) &&
      game.turnNumber <= 3 &&
      isPlainSpiritDevelopmentRoot(evaluation) &&
      isPlainSpiritDevelopmentRoot(best) &&
      saturatingScoreSubtract(bestScore, evaluation.score) <= 16;
    const spiritSetupOrder = isProductionMode(config)
      ? compareProductionRules(
          ProductionComparisonPhase.SpiritSetup,
          { ...context, candidateIndex: index, incumbentIndex: bestIndex },
          options.productionPolicy,
        )
      : undefined;
    const spiritSetupCompetes =
      !isProductionMode(config) ||
      spiritSetupOrder === undefined ||
      spiritSetupOrder >= 0;
    if (evaluation.score + scoreMargin < bestScore && !allowClosePlainSpiritSlack) {
      continue;
    }

    let spiritChallengeOrder =
      preferSpirit && evaluation.spiritDevelopment !== best.spiritDevelopment
        ? spiritScoreChallengeOrder(evaluation, best)
        : undefined;
    if (
      spiritChallengeOrder === undefined &&
      isProductionMode(config) &&
      preferSpirit &&
      evaluation.spiritDevelopment !== best.spiritDevelopment &&
      Math.abs(evaluation.score - best.score) <= 320
    ) {
      spiritChallengeOrder = compareProductionRules(
        ProductionComparisonPhase.ProjectionChallenge,
        { ...context, candidateIndex: index, incumbentIndex: bestIndex },
        options.productionPolicy,
      );
    }
    const spiritBetter =
      spiritChallengeOrder === undefined &&
      preferSpirit &&
      evaluation.spiritDevelopment &&
      !best.spiritDevelopment &&
      spiritSetupCompetes;
    const equalSpiritPreference =
      !preferSpirit ||
      evaluation.spiritDevelopment === best.spiritDevelopment ||
      spiritChallengeOrder !== undefined ||
      (isProductionMode(config) &&
        spiritSetupCompetes &&
        (evaluation.spiritOwnManaSetupNow || evaluation.spiritSameTurnScoreSetupNow));
    const spiritSameTurnSetupBetter =
      evaluation.spiritSameTurnScoreSetupNow &&
      !best.spiritSameTurnScoreSetupNow &&
      spiritSetupCompetes;
    const equalSpiritSameTurnSetup =
      evaluation.spiritSameTurnScoreSetupNow === best.spiritSameTurnScoreSetupNow;
    const spiritSetupBetter =
      evaluation.spiritOwnManaSetupNow &&
      !best.spiritOwnManaSetupNow &&
      spiritSetupCompetes;
    const equalSpiritSetup =
      evaluation.spiritOwnManaSetupNow === best.spiritOwnManaSetupNow;
    const spiritSetupGainBetter =
      preferSpirit &&
      evaluation.spiritDevelopment &&
      best.spiritDevelopment &&
      evaluation.spiritSetupGain > best.spiritSetupGain;
    const equalSpiritSetupGain =
      !preferSpirit ||
      !evaluation.spiritDevelopment ||
      !best.spiritDevelopment ||
      evaluation.spiritSetupGain === best.spiritSetupGain;
    const comparePlainSpiritProjection =
      productionPlainSpiritProjectionTiebreak &&
      isPlainSpiritDevelopmentRoot(evaluation) &&
      isPlainSpiritDevelopmentRoot(best);
    const spiritProjectionOrder =
      isProductionMode(config) &&
      (preferSpirit || comparePlainSpiritProjection) &&
      evaluation.spiritDevelopment &&
      best.spiritDevelopment
        ? compareProductionRules(
            ProductionComparisonPhase.Projection,
            { ...context, candidateIndex: index, incumbentIndex: bestIndex },
            options.productionPolicy,
          )
        : undefined;
    const spiritFollowupOrder = isProductionMode(config)
      ? compareProductionRules(
          ProductionComparisonPhase.FollowupFloor,
          { ...context, candidateIndex: index, incumbentIndex: bestIndex },
          options.productionPolicy,
        )
      : undefined;
    const spiritSetupSupermanaStepsBetter =
      evaluation.spiritOwnManaSetupNow &&
      best.spiritOwnManaSetupNow &&
      evaluation.supermanaProgress &&
      best.supermanaProgress &&
      rootProgressStepsBetter(
        evaluation.safeSupermanaProgressSteps,
        best.safeSupermanaProgressSteps,
      );
    const equalSpiritSetupSupermanaSteps =
      !evaluation.spiritOwnManaSetupNow ||
      !best.spiritOwnManaSetupNow ||
      !evaluation.supermanaProgress ||
      !best.supermanaProgress ||
      evaluation.safeSupermanaProgressSteps === best.safeSupermanaProgressSteps;
    const spiritSetupOpponentStepsBetter =
      evaluation.spiritOwnManaSetupNow &&
      best.spiritOwnManaSetupNow &&
      evaluation.opponentManaProgress &&
      best.opponentManaProgress &&
      rootProgressStepsBetter(
        evaluation.safeOpponentManaProgressSteps,
        best.safeOpponentManaProgressSteps,
      );
    const equalSpiritSetupOpponentSteps =
      !evaluation.spiritOwnManaSetupNow ||
      !best.spiritOwnManaSetupNow ||
      !evaluation.opponentManaProgress ||
      !best.opponentManaProgress ||
      evaluation.safeOpponentManaProgressSteps === best.safeOpponentManaProgressSteps;
    const spiritSetupScorePathBetter =
      evaluation.spiritOwnManaSetupNow &&
      best.spiritOwnManaSetupNow &&
      rootScorePathStepsBetter(evaluation.scorePathBestSteps, best.scorePathBestSteps);
    const equalSpiritSetupScorePath =
      !evaluation.spiritOwnManaSetupNow ||
      !best.spiritOwnManaSetupNow ||
      evaluation.scorePathBestSteps === best.scorePathBestSteps;
    const supermanaStepsBetter = rootProgressStepsBetter(
      evaluation.safeSupermanaProgressSteps,
      best.safeSupermanaProgressSteps,
    );
    const equalSupermanaSteps =
      evaluation.safeSupermanaProgressSteps === best.safeSupermanaProgressSteps;
    const opponentStepsBetter = rootProgressStepsBetter(
      evaluation.safeOpponentManaProgressSteps,
      best.safeOpponentManaProgressSteps,
    );
    const equalOpponentSteps =
      evaluation.safeOpponentManaProgressSteps === best.safeOpponentManaProgressSteps;
    const scoreWindowBetter =
      evaluation.sameTurnScoreWindowValue > best.sameTurnScoreWindowValue;
    const equalScoreWindow =
      evaluation.sameTurnScoreWindowValue === best.sameTurnScoreWindowValue;
    const handoffBetter =
      !evaluation.manaHandoffToOpponent && best.manaHandoffToOpponent;
    const equalHandoff =
      evaluation.manaHandoffToOpponent === best.manaHandoffToOpponent;
    const roundtripBetter = !evaluation.hasRoundtrip && best.hasRoundtrip;
    const equalRoundtrip = evaluation.hasRoundtrip === best.hasRoundtrip;
    const softBetter =
      evaluation.policyPriority >
      saturatingScoreAdd(best.policyPriority, INTERVIEW_SOFT_PRIORITY_SCORE_MARGIN);
    const softEqualOrDisabled =
      saturatingScoreAdd(
        evaluation.policyPriority,
        INTERVIEW_SOFT_PRIORITY_SCORE_MARGIN,
      ) >= best.policyPriority;
    const efficiencyOrScoreBetter =
      evaluation.efficiency > bestEfficiency ||
      (evaluation.efficiency === bestEfficiency &&
        evaluation.score > bestShortlistedScore);

    let tieBreakBetter: boolean;
    if ((spiritChallengeOrder ?? 0) > 0) tieBreakBetter = true;
    else if ((spiritChallengeOrder ?? 0) < 0) tieBreakBetter = false;
    else if (softBetter) tieBreakBetter = true;
    else if (!softEqualOrDisabled) tieBreakBetter = false;
    else if (scoreWindowBetter) tieBreakBetter = true;
    else if (!equalScoreWindow) tieBreakBetter = false;
    else if (spiritSameTurnSetupBetter) tieBreakBetter = true;
    else if (!equalSpiritSameTurnSetup) tieBreakBetter = false;
    else if (spiritSetupBetter) tieBreakBetter = true;
    else if (!equalSpiritSetup) tieBreakBetter = false;
    else if (spiritSetupGainBetter) tieBreakBetter = true;
    else if (!equalSpiritSetupGain) tieBreakBetter = false;
    else if ((spiritFollowupOrder ?? 0) > 0) tieBreakBetter = true;
    else if ((spiritFollowupOrder ?? 0) < 0) tieBreakBetter = false;
    else if ((spiritProjectionOrder ?? 0) > 0) tieBreakBetter = true;
    else if ((spiritProjectionOrder ?? 0) < 0) tieBreakBetter = false;
    else if (spiritSetupSupermanaStepsBetter) tieBreakBetter = true;
    else if (!equalSpiritSetupSupermanaSteps) tieBreakBetter = false;
    else if (spiritSetupOpponentStepsBetter) tieBreakBetter = true;
    else if (!equalSpiritSetupOpponentSteps) tieBreakBetter = false;
    else if (spiritSetupScorePathBetter) tieBreakBetter = true;
    else if (!equalSpiritSetupScorePath) tieBreakBetter = false;
    else if (supermanaStepsBetter) tieBreakBetter = true;
    else if (!equalSupermanaSteps) tieBreakBetter = false;
    else if (opponentStepsBetter) tieBreakBetter = true;
    else if (!equalOpponentSteps) tieBreakBetter = false;
    else if (handoffBetter) tieBreakBetter = true;
    else if (!equalHandoff) tieBreakBetter = false;
    else if (roundtripBetter) tieBreakBetter = true;
    else if (!equalRoundtrip) tieBreakBetter = false;
    else tieBreakBetter = efficiencyOrScoreBetter;

    if (spiritBetter || (equalSpiritPreference && tieBreakBetter)) {
      bestIndex = index;
      bestEfficiency = evaluation.efficiency;
      bestShortlistedScore = evaluation.score;
    }
  }
  return bestIndex;
}

function pickBaselineRootIndex(
  game: MonsGame,
  roots: readonly EvaluatedRoot[],
  perspective: Color,
  config: AutomoveConfig,
  options: RootSelectorOptions = {},
): number | undefined {
  if (options.checkpoint?.() === true || roots.length === 0) return undefined;
  if (isProductionMode(config)) {
    const all = roots.map((_root, index) => index);
    const picker = options.productionPolicy?.rootPicker;
    if (picker !== undefined) {
      const result = picker.select(
        selectionContext(game, roots, all, perspective, config),
      );
      if (result.kind === "select") {
        assertRootIndex(roots, result.index, `Production rule ${picker.id}`);
        return options.cancelled?.() === true ? undefined : result.index;
      }
    }
  }
  let candidates = filteredRootCandidateIndices(
    game,
    roots,
    perspective,
    config,
    options,
  );
  if (candidates.length === 0) candidates = roots.map((_root, index) => index);
  return pickBaselineRootIndexFromCandidateIndices(
    game,
    roots,
    candidates,
    perspective,
    config,
    options,
  );
}

export function pickBaselineRootInputs(
  game: MonsGame,
  roots: readonly EvaluatedRoot[],
  perspective: Color,
  config: AutomoveConfig,
  options: RootSelectorOptions = {},
): Input[] {
  const index = pickBaselineRootIndex(game, roots, perspective, config, options);
  return index === undefined ? [] : [...valueAt(roots, index).inputs];
}
