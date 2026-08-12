import { ACTIONS_PER_TURN } from "../../engine/board/config.js";
import { BOARD_SIZE } from "../../engine/board/geometry.js";
import { Color } from "../../api/types.js";
import type { MonsGame } from "../../engine/game/mons-game.js";
import { scoreForColor } from "../../engine/rules/legality.js";
import { MIN_SCORE, saturatingScoreAdd } from "../core/score-math.js";
import {
  AUTOMOVE_TURN_ENGINE_MODE,
  shouldPreferSpiritDevelopment,
  type AutomoveConfig,
} from "../config/types.js";
import type { EvaluatedRoot } from "./types.js";
import {
  ProductionCompetitionKind,
  PRODUCTION_COMPETITION_KIND_ORDER,
  ProductionComparisonPhase,
  type ProductionRootPolicy,
  type RootSelectionContext,
  type RootSelectorOptions,
} from "./selector-model.js";
import {
  compareProductionRules,
  compareRankedEvaluatedRootIndices,
} from "./evaluated-ordering.js";

const ROOT_SPIRIT_DEVELOPMENT_SCORE_MARGIN = 700;
const ROOT_POTION_HOLD_SCORE_MARGIN = 180;

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

function minValue(values: readonly number[]): number | undefined {
  let best: number | undefined;
  for (const value of values) best = best === undefined ? value : Math.min(best, value);
  return best;
}

function potionsForColor(game: MonsGame, color: Color): number {
  return color === Color.White ? game.whitePotionsCount : game.blackPotionsCount;
}

function shouldPreferPotionTakebackLines(game: MonsGame, perspective: Color): boolean {
  return (
    game.activeColor === perspective &&
    !game.isFirstTurn() &&
    !game.playerCanMoveMon() &&
    game.actionsUsedCount >= ACTIONS_PER_TURN &&
    game.playerCanMoveMana() &&
    potionsForColor(game, perspective) > 0
  );
}

function rootSpendsPotion(
  gameBefore: MonsGame,
  root: EvaluatedRoot,
  perspective: Color,
): boolean {
  return (
    potionsForColor(root.game, perspective) < potionsForColor(gameBefore, perspective)
  );
}

function rootPotionSpendCompensated(
  gameBefore: MonsGame,
  root: EvaluatedRoot,
  perspective: Color,
): boolean {
  return (
    root.winsImmediately ||
    root.attacksOpponentDrainer ||
    scoreForColor(root.game, perspective) >=
      saturatingScoreAdd(scoreForColor(gameBefore, perspective), 2) ||
    root.scoresSupermanaThisTurn ||
    root.scoresOpponentManaThisTurn ||
    (!root.ownDrainerVulnerable &&
      (root.supermanaProgress || root.opponentManaProgress))
  );
}

function immediateOpponentWin(
  root: EvaluatedRoot,
  rootIndex: number,
  perspective: Color,
  config: AutomoveConfig,
  options: RootSelectorOptions,
): boolean {
  const replyLimit = Math.max(config.replyRisk.antiHelpReplyLimit, 1);
  return (
    options.rootReplyRiskSnapshot?.(
      root.game,
      perspective,
      config,
      replyLimit,
      rootIndex,
    ).allowsImmediateOpponentWin ?? false
  );
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

function evaluateProductionCompetitionRules(
  policy: ProductionRootPolicy | undefined,
  kind: ProductionCompetitionKind,
  context: RootSelectionContext,
): boolean {
  let selected = false;
  for (const rule of policy?.competitionRules ?? []) {
    if (rule.kind !== kind) continue;
    if (rule.evaluate(context).kind === "select") selected = true;
  }
  return selected;
}

function anyProductionCompetition(
  game: MonsGame,
  roots: readonly EvaluatedRoot[],
  candidateIndices: readonly number[],
  perspective: Color,
  config: AutomoveConfig,
  options: RootSelectorOptions,
  negativeDenyIsOverridden?: () => boolean,
): boolean {
  let competes = false;
  const context = selectionContext(game, roots, candidateIndices, perspective, config);
  for (const kind of PRODUCTION_COMPETITION_KIND_ORDER) {
    const kindCompetes =
      isProductionMode(config) &&
      evaluateProductionCompetitionRules(options.productionPolicy, kind, context);
    if (
      kindCompetes &&
      (kind !== "negative-deny" || negativeDenyIsOverridden?.() !== true)
    ) {
      competes = true;
    }
  }
  return competes;
}

function spiritSetupOverridesNegativeDeny(
  context: RootSelectionContext,
  spiritSetupIndices: readonly number[],
  policy: ProductionRootPolicy | undefined,
): boolean {
  const nonSpiritIndices = context.candidateIndices.filter(
    (index) => !valueAt(context.roots, index).spiritDevelopment,
  );
  return spiritSetupIndices.some((spiritIndex) =>
    nonSpiritIndices.every((index) => {
      const order = compareProductionRules(
        ProductionComparisonPhase.SpiritSetup,
        {
          ...context,
          candidateIndex: spiritIndex,
          incumbentIndex: index,
        },
        policy,
      );
      return order !== undefined && order > 0;
    }),
  );
}

function retainBestKnownSteps(
  roots: readonly EvaluatedRoot[],
  indices: readonly number[],
  steps: (root: EvaluatedRoot) => number,
  unknown: number,
): number[] {
  const best = minValue(
    indices
      .map((index) => steps(valueAt(roots, index)))
      .filter((value) => value < unknown),
  );
  return best === undefined
    ? [...indices]
    : indices.filter((index) => steps(valueAt(roots, index)) === best);
}

export function filteredRootCandidateIndices(
  game: MonsGame,
  roots: readonly EvaluatedRoot[],
  perspective: Color,
  config: AutomoveConfig,
  options: RootSelectorOptions = {},
): number[] {
  if (roots.length === 0) return [];
  let candidates = roots.map((_root, index) => index);
  let forcedAttackApplied = false;

  if (candidates.some((index) => valueAt(roots, index).winsImmediately)) {
    return candidates.filter((index) => valueAt(roots, index).winsImmediately);
  }
  if (candidates.some((index) => valueAt(roots, index).attacksOpponentDrainer)) {
    candidates = candidates.filter(
      (index) => valueAt(roots, index).attacksOpponentDrainer,
    );
    forcedAttackApplied = true;
  }

  if (
    !forcedAttackApplied &&
    !candidates.some((index) => valueAt(roots, index).classes.immediateScore)
  ) {
    const bestWindow = maxValue(
      candidates.flatMap((index) => {
        const root = valueAt(roots, index);
        return !root.ownDrainerVulnerable && !root.manaHandoffToOpponent
          ? [root.sameTurnScoreWindowValue]
          : [];
      }),
      0,
    );
    if (bestWindow > 0) {
      const windows = candidates.filter((index) => {
        const root = valueAt(roots, index);
        return (
          root.sameTurnScoreWindowValue === bestWindow &&
          !root.ownDrainerVulnerable &&
          !root.manaHandoffToOpponent
        );
      });
      if (windows.length > 0) candidates = windows;
    }
  }

  if (
    !forcedAttackApplied &&
    candidates.some((index) => valueAt(roots, index).safeSupermanaPickupNow)
  ) {
    candidates = candidates.filter((index) => {
      const root = valueAt(roots, index);
      return root.scoresSupermanaThisTurn || root.safeSupermanaPickupNow;
    });
  } else if (
    !forcedAttackApplied &&
    candidates.some((index) => valueAt(roots, index).safeOpponentManaPickupNow)
  ) {
    candidates = candidates.filter((index) => {
      const root = valueAt(roots, index);
      return root.scoresOpponentManaThisTurn || root.safeOpponentManaPickupNow;
    });
  }

  if (
    !forcedAttackApplied &&
    !candidates.some((index) => valueAt(roots, index).classes.immediateScore)
  ) {
    const bestSpiritWindow = maxValue(
      candidates.flatMap((index) => {
        const root = valueAt(roots, index);
        return root.spiritSameTurnScoreSetupNow &&
          !root.ownDrainerVulnerable &&
          !root.manaHandoffToOpponent
          ? [root.sameTurnScoreWindowValue]
          : [];
      }),
      0,
    );
    const bestNonSpiritWindow = maxValue(
      candidates.flatMap((index) => {
        const root = valueAt(roots, index);
        return !root.spiritSameTurnScoreSetupNow &&
          !root.ownDrainerVulnerable &&
          !root.manaHandoffToOpponent
          ? [root.sameTurnScoreWindowValue]
          : [];
      }),
      0,
    );
    if (bestSpiritWindow > bestNonSpiritWindow) {
      candidates = candidates.filter((index) => {
        const root = valueAt(roots, index);
        return (
          root.spiritSameTurnScoreSetupNow &&
          root.sameTurnScoreWindowValue === bestSpiritWindow &&
          !root.ownDrainerVulnerable &&
          !root.manaHandoffToOpponent
        );
      });
    }
  }

  if (
    !forcedAttackApplied &&
    !candidates.some((index) => valueAt(roots, index).classes.immediateScore) &&
    candidates.some((index) => {
      const root = valueAt(roots, index);
      return (
        root.sameTurnScoreWindowValue > 0 &&
        !root.ownDrainerVulnerable &&
        !root.manaHandoffToOpponent
      );
    })
  ) {
    const bestWindow = maxValue(
      candidates.map((index) => valueAt(roots, index).sameTurnScoreWindowValue),
      0,
    );
    if (bestWindow > 0) {
      const windows = candidates.filter((index) => {
        const root = valueAt(roots, index);
        return (
          root.sameTurnScoreWindowValue === bestWindow &&
          !root.ownDrainerVulnerable &&
          !root.manaHandoffToOpponent
        );
      });
      if (windows.length > 0) candidates = windows;
    }
  }

  if (!forcedAttackApplied) {
    const spiritSetups = candidates.filter((index) => {
      const root = valueAt(roots, index);
      return (
        root.spiritOwnManaSetupNow &&
        !root.ownDrainerVulnerable &&
        !root.manaHandoffToOpponent
      );
    });
    if (
      spiritSetups.length > 0 &&
      !anyProductionCompetition(
        game,
        roots,
        candidates,
        perspective,
        config,
        options,
        () =>
          spiritSetupOverridesNegativeDeny(
            selectionContext(game, roots, candidates, perspective, config),
            spiritSetups,
            options.productionPolicy,
          ),
      )
    ) {
      const supermana = spiritSetups.filter(
        (index) => valueAt(roots, index).supermanaProgress,
      );
      if (supermana.length > 0) {
        candidates = retainBestKnownSteps(
          roots,
          supermana,
          (root) => root.safeSupermanaProgressSteps,
          BOARD_SIZE + 4,
        );
      } else {
        const opponentMana = spiritSetups.filter(
          (index) => valueAt(roots, index).opponentManaProgress,
        );
        candidates =
          opponentMana.length > 0
            ? retainBestKnownSteps(
                roots,
                opponentMana,
                (root) => root.safeOpponentManaProgressSteps,
                BOARD_SIZE + 4,
              )
            : retainBestKnownSteps(
                roots,
                spiritSetups,
                (root) => root.scorePathBestSteps,
                BOARD_SIZE * 3,
              );
      }
    }
  }

  if (
    config.policy.hardSpiritDeployment &&
    !forcedAttackApplied &&
    shouldPreferSpiritDevelopment(game, perspective)
  ) {
    const hasSafeHighValuePickup = candidates.some((index) => {
      const root = valueAt(roots, index);
      return (
        root.scoresSupermanaThisTurn ||
        root.scoresOpponentManaThisTurn ||
        root.safeSupermanaPickupNow ||
        root.safeOpponentManaPickupNow
      );
    });
    if (
      !hasSafeHighValuePickup &&
      !anyProductionCompetition(game, roots, candidates, perspective, config, options)
    ) {
      const spiritSetups = candidates.filter((index) => {
        const root = valueAt(roots, index);
        return (
          root.spiritOwnManaSetupNow &&
          !root.ownDrainerVulnerable &&
          !root.manaHandoffToOpponent
        );
      });
      if (spiritSetups.length > 0) {
        candidates = spiritSetups;
      } else {
        const scoreBefore = scoreForColor(game, perspective);
        const spiritReady = candidates.filter(
          (index) => !valueAt(roots, index).keepsAwakeSpiritOnBase,
        );
        if (spiritReady.length > 0) {
          const safeSpiritReady = spiritReady.filter((index) => {
            const root = valueAt(roots, index);
            return !root.ownDrainerVulnerable && !root.manaHandoffToOpponent;
          });
          const preferred = safeSpiritReady.length > 0 ? safeSpiritReady : spiritReady;
          const keepsSpiritAndScores = candidates.some((index) => {
            const root = valueAt(roots, index);
            return (
              root.keepsAwakeSpiritOnBase &&
              scoreForColor(root.game, perspective) > scoreBefore
            );
          });
          const spiritLineScores = preferred.some(
            (index) =>
              scoreForColor(valueAt(roots, index).game, perspective) > scoreBefore,
          );
          if (!keepsSpiritAndScores || spiritLineScores) candidates = preferred;
        }
      }
    }
  }

  if (!forcedAttackApplied) {
    const preSafety = [...candidates];
    const bestScore = maxValue(
      candidates.map((index) => valueAt(roots, index).score),
      MIN_SCORE,
    );
    const margin = Math.max(config.policy.drainerSafetyScoreMargin, 0);
    const safer = candidates.filter((index) => {
      const root = valueAt(roots, index);
      return !root.ownDrainerVulnerable && root.score + margin >= bestScore;
    });
    if (safer.length > 0) {
      candidates = safer;
      if (isProductionMode(config)) {
        const context = selectionContext(game, roots, preSafety, perspective, config);
        for (const rule of options.productionPolicy?.safetyReentryRules ?? []) {
          const result = rule.select({
            ...context,
            selectedIndices: candidates,
          });
          if (result.kind === "continue") continue;
          for (const index of result.indices) {
            assertRootIndex(roots, index, `Production rule ${rule.id}`);
            if (!preSafety.includes(index)) {
              throw new RangeError(
                `Production rule ${rule.id} selected a root outside the prefilter`,
              );
            }
            if (!candidates.includes(index)) candidates.push(index);
          }
        }
      }
    }
  }

  if (
    config.policy.preferSpiritDevelopment &&
    shouldPreferSpiritDevelopment(game, perspective) &&
    candidates.some((index) => valueAt(roots, index).spiritDevelopment)
  ) {
    const hasSafeHighValuePickup = candidates.some((index) => {
      const root = valueAt(roots, index);
      return (
        root.scoresSupermanaThisTurn ||
        root.scoresOpponentManaThisTurn ||
        root.safeSupermanaPickupNow ||
        root.safeOpponentManaPickupNow
      );
    });
    if (
      !hasSafeHighValuePickup &&
      !anyProductionCompetition(game, roots, candidates, perspective, config, options)
    ) {
      const bestScore = maxValue(
        candidates.map((index) => valueAt(roots, index).score),
        MIN_SCORE,
      );
      const spiritSetups = candidates.filter((index) => {
        const root = valueAt(roots, index);
        return (
          root.spiritOwnManaSetupNow &&
          root.score + ROOT_SPIRIT_DEVELOPMENT_SCORE_MARGIN >= bestScore
        );
      });
      if (spiritSetups.length > 0) {
        candidates = spiritSetups;
      } else {
        const spirit = candidates.filter((index) => {
          const root = valueAt(roots, index);
          return (
            root.spiritDevelopment &&
            root.score + ROOT_SPIRIT_DEVELOPMENT_SCORE_MARGIN >= bestScore
          );
        });
        if (spirit.length > 0) candidates = spirit;
      }
    }
  }

  if (
    !forcedAttackApplied &&
    candidates.length > 1 &&
    shouldPreferPotionTakebackLines(game, perspective)
  ) {
    const bestScore = maxValue(
      candidates.map((index) => valueAt(roots, index).score),
      MIN_SCORE,
    );
    const nearBest = candidates.filter(
      (index) =>
        valueAt(roots, index).score + ROOT_POTION_HOLD_SCORE_MARGIN >= bestScore,
    );
    if (nearBest.length > 1) {
      const quickLoss = new Map<number, boolean>();
      const allowsLoss = (index: number): boolean => {
        const cached = quickLoss.get(index);
        if (cached !== undefined) return cached;
        const result = immediateOpponentWin(
          valueAt(roots, index),
          index,
          perspective,
          config,
          options,
        );
        quickLoss.set(index, result);
        return result;
      };
      const hasNonPotionNonLosing = nearBest.some((index) => {
        const root = valueAt(roots, index);
        return !rootSpendsPotion(game, root, perspective) && !allowsLoss(index);
      });
      if (hasNonPotionNonLosing) {
        const nearBestSet = new Set(nearBest);
        const strict = candidates.filter((index) => {
          const root = valueAt(roots, index);
          return (
            root.winsImmediately ||
            !nearBestSet.has(index) ||
            !rootSpendsPotion(game, root, perspective) ||
            rootPotionSpendCompensated(game, root, perspective)
          );
        });
        if (strict.length > 0) candidates = strict;
      }
    }
  }

  if (candidates.length > 1) {
    const bestScore = maxValue(
      candidates.map((index) => valueAt(roots, index).score),
      MIN_SCORE,
    );
    const margin = Math.max(config.replyRisk.antiHelpScoreMargin, 0);
    const nearBest = candidates.filter(
      (index) => valueAt(roots, index).score + margin >= bestScore,
    );
    if (nearBest.length > 1) {
      const quickLoss = new Map<number, boolean>();
      const allowsLoss = (index: number): boolean => {
        const cached = quickLoss.get(index);
        if (cached !== undefined) return cached;
        const result = immediateOpponentWin(
          valueAt(roots, index),
          index,
          perspective,
          config,
          options,
        );
        quickLoss.set(index, result);
        return result;
      };
      const hasCleanNonLosing = nearBest.some((index) => {
        const root = valueAt(roots, index);
        return !root.manaHandoffToOpponent && !root.hasRoundtrip && !allowsLoss(index);
      });
      if (hasCleanNonLosing) {
        const nearBestSet = new Set(nearBest);
        const strict = candidates.filter((index) => {
          const root = valueAt(roots, index);
          return (
            root.winsImmediately ||
            !nearBestSet.has(index) ||
            (!root.manaHandoffToOpponent && !root.hasRoundtrip)
          );
        });
        if (strict.length > 0) candidates = strict;
      }
    }
  }

  if (isProductionMode(config)) {
    const context = selectionContext(game, roots, [...candidates], perspective, config);
    let reentered = false;
    for (const rule of options.productionPolicy?.finalReentryRules ?? []) {
      const result = rule.select({
        ...context,
        selectedIndices: candidates,
      });
      if (result.kind === "continue") continue;
      for (const index of result.indices) {
        assertRootIndex(roots, index, `Production rule ${rule.id}`);
        if (!candidates.includes(index)) {
          candidates.push(index);
          reentered = true;
        }
      }
    }
    if (reentered) {
      candidates.sort((left, right) =>
        compareRankedEvaluatedRootIndices(roots, left, right),
      );
    }
  }
  return candidates;
}
