import { Color } from "../../../api/types.js";
import type { MonsGame } from "../../../engine/game/mons-game.js";
import type { AutomoveExecutionContext } from "../../core/execution-context.js";
import { MIN_SCORE } from "../../core/score-math.js";
import type { EvaluatedRoot } from "../../root/types.js";
import { isPlainSpiritDevelopmentRoot, productionEnabled } from "../../config/types.js";
import type { AutomoveConfig } from "../../config/types.js";
import {
  canTurnEngineProjectReplyRiskRoot,
  rootReplyRiskSnapshotWithProjection,
  turnEngineReplyRiskProjections,
} from "./projection.js";
import {
  rankedRootOrder,
  replyRiskGuardShortlistIndices,
  safePlainSpiritCompetition,
} from "./shortlist.js";
import { rootReplyRiskSnapshot } from "./snapshot.js";
import type { RootReplyRiskSnapshot } from "./types.js";
import { isBetterReplyRiskCandidate } from "./arbitration.js";
import {
  blackManaWindowProgressCompetition,
  closePositiveScoreCompetition,
  safeProgressCompetition,
  whiteSpiritFollowupSetupCompetition,
} from "./competition.js";
import {
  omittedSameOpeningSetupCompetes,
  sameOpeningSafeSetupPair,
} from "./sibling-ordering.js";
import {
  mixedPlainSpiritReplyFloorOrder,
  plainSpiritReplyRiskPick,
} from "./spirit-ordering.js";
import { productionWhiteTurnFourManaSiblingReentry } from "./reentry.js";

export function pickRootWithReplyRiskGuard(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  evaluations: readonly EvaluatedRoot[],
  indices: readonly number[],
  perspective: Color,
  config: AutomoveConfig,
  candidateIndices: readonly number[] = indices,
): number | undefined {
  if (!config.replyRisk.enabled || execution.session.checkpoint()) {
    return undefined;
  }
  let shortlist = replyRiskGuardShortlistIndices(
    execution,
    evaluations,
    indices,
    config,
  );
  if (shortlist.length === 0) return undefined;
  const reentry = productionWhiteTurnFourManaSiblingReentry(
    execution,
    game,
    evaluations,
    shortlist,
    perspective,
    config,
  );
  if (reentry !== undefined && !shortlist.includes(reentry)) {
    shortlist = [...shortlist, reentry].sort(
      (left, right) => -rankedRootOrder(evaluations, left, right),
    );
  }
  const rootNodeBudget = Math.max(
    shortlist.length,
    1,
    Math.trunc(
      (config.budget.maxVisitedNodes * Math.max(0, config.replyRisk.nodeShareBp)) /
        10_000,
    ),
  );
  const perRootReplyLimit = Math.min(
    Math.max(1, config.replyRisk.replyLimit),
    Math.max(1, Math.trunc(rootNodeBudget / shortlist.length)),
  );
  const projections = turnEngineReplyRiskProjections(
    execution,
    evaluations,
    shortlist,
    perspective,
    config,
  );
  const snapshots = new Map<number, RootReplyRiskSnapshot>();
  for (const index of shortlist) {
    if (execution.session.checkpoint()) return undefined;
    const evaluation = evaluations[index];
    if (evaluation === undefined) continue;
    const projection = projections.get(index);
    if (projection !== undefined) {
      snapshots.set(
        index,
        rootReplyRiskSnapshotWithProjection(
          execution,
          evaluation,
          projection,
          perspective,
          config,
          perRootReplyLimit,
        ),
      );
      continue;
    }
    const canUseFallbackProjection =
      config.planner.enabled &&
      productionEnabled(config) &&
      canTurnEngineProjectReplyRiskRoot(evaluation, perspective) &&
      !isPlainSpiritDevelopmentRoot(evaluation);
    const projected = canUseFallbackProjection ? evaluation.game : undefined;
    snapshots.set(
      index,
      rootReplyRiskSnapshot(
        execution,
        projected ?? evaluation.game,
        perspective,
        config,
        perRootReplyLimit,
      ),
    );
  }
  const spiritFollowupScores = new Map<number, number>();
  const bestPlainSpiritIndex = plainSpiritReplyRiskPick(
    execution,
    game,
    evaluations,
    shortlist,
    snapshots,
    projections,
    perspective,
    config,
    spiritFollowupScores,
  );
  if (
    shortlist.every((index) => {
      const root = evaluations[index];
      return root !== undefined && isPlainSpiritDevelopmentRoot(root);
    })
  ) {
    return bestPlainSpiritIndex;
  }
  if (
    productionEnabled(config) &&
    shortlist.every((index) => {
      const value = snapshots.get(index);
      return (
        value !== undefined &&
        !value.allowsImmediateOpponentWin &&
        !value.opponentReachesMatchPoint
      );
    }) &&
    !safePlainSpiritCompetition(evaluations, shortlist, config) &&
    !whiteSpiritFollowupSetupCompetition(game, evaluations, shortlist, config) &&
    !safeProgressCompetition(evaluations, shortlist, config) &&
    !blackManaWindowProgressCompetition(game, evaluations, shortlist, config) &&
    !closePositiveScoreCompetition(evaluations, shortlist, config)
  ) {
    const ordered = [...shortlist].sort((left, right) => {
      const leftFloor = snapshots.get(left)?.worstReplyScore ?? MIN_SCORE;
      const rightFloor = snapshots.get(right)?.worstReplyScore ?? MIN_SCORE;
      return rightFloor - leftFloor || -rankedRootOrder(evaluations, left, right);
    });
    const first = ordered[0];
    const second = ordered[1];
    if (
      first !== undefined &&
      (second === undefined ||
        (snapshots.get(first)?.worstReplyScore ?? MIN_SCORE) >
          (snapshots.get(second)?.worstReplyScore ?? MIN_SCORE))
    ) {
      return first;
    }
  }
  let bestIndex =
    bestPlainSpiritIndex ?? shortlist.find((index) => snapshots.has(index));
  if (bestIndex === undefined) return undefined;
  for (const index of shortlist) {
    if (index === bestIndex) continue;
    const candidate = evaluations[index];
    const incumbent = evaluations[bestIndex];
    const candidateSnapshot = snapshots.get(index);
    const incumbentSnapshot = snapshots.get(bestIndex);
    if (
      candidate !== undefined &&
      incumbent !== undefined &&
      candidateSnapshot !== undefined &&
      incumbentSnapshot !== undefined &&
      isBetterReplyRiskCandidate(
        execution,
        candidate,
        candidateSnapshot,
        incumbent,
        incumbentSnapshot,
        config,
        {
          candidateProjection: projections.get(index),
          incumbentProjection: projections.get(bestIndex),
          game,
          evaluations,
          candidateIndex: index,
          incumbentIndex: bestIndex,
          perspective,
          spiritFollowupScores,
        },
      )
    ) {
      bestIndex = index;
    }
  }
  if (
    productionEnabled(config) &&
    bestPlainSpiritIndex !== undefined &&
    bestPlainSpiritIndex !== bestIndex
  ) {
    const plainSnapshot = snapshots.get(bestPlainSpiritIndex);
    const bestSnapshot = snapshots.get(bestIndex);
    const plainProjection = projections.get(bestPlainSpiritIndex);
    const bestProjection = projections.get(bestIndex);
    if (
      plainSnapshot !== undefined &&
      bestSnapshot !== undefined &&
      plainProjection !== undefined &&
      bestProjection !== undefined &&
      (mixedPlainSpiritReplyFloorOrder(
        plainSnapshot,
        plainProjection,
        bestSnapshot,
        bestProjection,
        config,
      ) ?? -1) > 0
    ) {
      bestIndex = bestPlainSpiritIndex;
    }
  }
  const bestRoot = evaluations[bestIndex];
  if (
    productionEnabled(config) &&
    bestRoot !== undefined &&
    !bestRoot.spiritDevelopment &&
    !bestRoot.spiritSameTurnScoreSetupNow &&
    !bestRoot.spiritOwnManaSetupNow &&
    bestIndex >= 4
  ) {
    const omitted = candidateIndices
      .filter((index) => !shortlist.includes(index))
      .filter((index) => {
        const root = evaluations[index];
        return (
          root !== undefined &&
          root.spiritOwnManaSetupNow &&
          root.rootRank + 3 <= bestRoot.rootRank &&
          sameOpeningSafeSetupPair(root, bestRoot, config)
        );
      })
      .sort((left, right) => -rankedRootOrder(evaluations, left, right))[0];
    if (omitted !== undefined) {
      const candidate = evaluations[omitted];
      const incumbentSnapshot = snapshots.get(bestIndex);
      if (candidate !== undefined && incumbentSnapshot !== undefined) {
        const candidateSnapshot = rootReplyRiskSnapshotWithProjection(
          execution,
          candidate,
          undefined,
          perspective,
          config,
          perRootReplyLimit,
        );
        if (
          omittedSameOpeningSetupCompetes(
            candidate,
            candidateSnapshot,
            bestRoot,
            incumbentSnapshot,
            config,
          ) ||
          isBetterReplyRiskCandidate(
            execution,
            candidate,
            candidateSnapshot,
            bestRoot,
            incumbentSnapshot,
            config,
            {
              incumbentProjection: projections.get(bestIndex),
              game,
              evaluations,
              candidateIndex: omitted,
              incumbentIndex: bestIndex,
              perspective,
              spiritFollowupScores,
            },
          )
        ) {
          bestIndex = omitted;
        }
      }
    }
  }
  return execution.session.checkpoint() ? undefined : bestIndex;
}
