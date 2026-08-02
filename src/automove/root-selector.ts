import { ACTIONS_PER_TURN, BOARD_SIZE } from "../engine/config.js";
import { Color, type Input } from "../engine/domain.js";
import type { MonsGame } from "../engine/game.js";
import { scoreForColor } from "../engine/legality.js";
import {
  MIN_SCORE,
  saturatingScoreAdd,
  saturatingScoreSubtract,
} from "./score-math.js";
import {
  rootProgressStepsBetter,
  rootScorePathStepsBetter,
} from "./root-focus.js";
import type { EvaluatedRoot } from "./search.js";
import {
  AUTOMOVE_TURN_ENGINE_MODE,
  isPlainSpiritDevelopmentRoot,
  rootIsUnsafe as isUnsafe,
  shouldPreferSpiritDevelopment,
  type AutomoveConfig,
} from "./selector-types.js";

const ROOT_SPIRIT_DEVELOPMENT_SCORE_MARGIN = 700;
const ROOT_POTION_HOLD_SCORE_MARGIN = 180;
const INTERVIEW_SOFT_PRIORITY_SCORE_MARGIN = 80;
const SPIRIT_SCORE_CHALLENGE_MARGIN = 40;

export { rootProgressStepsBetter };

type RootReplyRiskSnapshot = {
  readonly allowsImmediateOpponentWin: boolean;
};

export type RootSelectionContext = {
  readonly game: MonsGame;
  readonly roots: readonly EvaluatedRoot[];
  readonly candidateIndices: readonly number[];
  readonly perspective: Color;
  readonly config: AutomoveConfig;
};

export const ProductionCompetitionKind = Object.freeze({
  SafeProgress: "safe-progress",
  FollowupProgress: "followup-progress",
  RiskyScore: "risky-score",
  NegativeDeny: "negative-deny",
  Score: "score",
  Projection: "projection",
  RiskyRecovery: "risky-recovery",
} as const);

type ProductionCompetitionKind =
  (typeof ProductionCompetitionKind)[keyof typeof ProductionCompetitionKind];

const PRODUCTION_COMPETITION_KIND_ORDER = Object.freeze([
  ProductionCompetitionKind.SafeProgress,
  ProductionCompetitionKind.FollowupProgress,
  ProductionCompetitionKind.RiskyScore,
  ProductionCompetitionKind.NegativeDeny,
  ProductionCompetitionKind.Score,
  ProductionCompetitionKind.Projection,
  ProductionCompetitionKind.RiskyRecovery,
] as const satisfies readonly ProductionCompetitionKind[]);

export const ProductionComparisonPhase = Object.freeze({
  SpiritSetup: "spirit-setup",
  ProjectionChallenge: "projection-challenge",
  Projection: "projection",
  FollowupFloor: "followup-floor",
} as const);

type ProductionComparisonPhase =
  (typeof ProductionComparisonPhase)[keyof typeof ProductionComparisonPhase];

type ProductionRootRuleId =
  | `competition.${string}`
  | `safety-reentry.${string}`
  | `final-reentry.${string}`
  | `comparison.${string}`
  | `root-picker.${string}`;

type ProductionContinueResult = {
  readonly kind: "continue";
};

export type ProductionCompetitionResult =
  ProductionContinueResult | { readonly kind: "select" };

export type ProductionIndexSelectionResult =
  | ProductionContinueResult
  | {
      readonly kind: "select";
      readonly indices: readonly number[];
    };

type ProductionRootSelectionResult =
  | ProductionContinueResult
  | {
      readonly kind: "select";
      readonly index: number;
    };

export type ProductionComparisonResult =
  | ProductionContinueResult
  | {
      readonly kind: "compare";
      /** Positive means candidate wins, negative means incumbent wins. */
      readonly order: number;
    };

type ProductionRootComparisonContext = RootSelectionContext & {
  readonly candidateIndex: number;
  readonly incumbentIndex: number;
};

export type ProductionRootReentryContext = RootSelectionContext & {
  readonly selectedIndices: readonly number[];
};

type ProductionCompetitionRule = {
  readonly id: ProductionRootRuleId;
  readonly kind: ProductionCompetitionKind;
  readonly evaluate: (
    context: RootSelectionContext,
  ) => ProductionCompetitionResult;
};

type ProductionReentryRule = {
  readonly id: ProductionRootRuleId;
  readonly select: (
    context: ProductionRootReentryContext,
  ) => ProductionIndexSelectionResult;
};

type ProductionComparisonRule = {
  readonly id: ProductionRootRuleId;
  readonly phase: ProductionComparisonPhase;
  readonly compare: (
    context: ProductionRootComparisonContext,
  ) => ProductionComparisonResult;
};

export type ProductionRootPicker = {
  readonly id: ProductionRootRuleId;
  readonly select: (
    context: RootSelectionContext,
  ) => ProductionRootSelectionResult;
};

export type ProductionRootPolicy = {
  readonly competitionRules: readonly ProductionCompetitionRule[];
  readonly safetyReentryRules: readonly ProductionReentryRule[];
  readonly finalReentryRules: readonly ProductionReentryRule[];
  readonly comparisonRules: readonly ProductionComparisonRule[];
  readonly rootPicker?: ProductionRootPicker;
};

export type RootSelectorOptions = {
  readonly rootReplyRiskSnapshot?: (
    stateAfterMove: MonsGame,
    perspective: Color,
    config: AutomoveConfig,
    replyLimit: number,
    rootIndex: number,
  ) => RootReplyRiskSnapshot;
  readonly pickReplyRiskGuardedIndex?: (
    context: RootSelectionContext,
  ) => number | undefined;
  readonly productionPolicy?: ProductionRootPolicy;
  readonly checkpoint?: () => boolean;
  readonly cancelled?: () => boolean;
};

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
  for (const value of values)
    best = best === undefined ? value : Math.min(best, value);
  return best;
}

function potionsForColor(game: MonsGame, color: Color): number {
  return color === Color.White
    ? game.whitePotionsCount
    : game.blackPotionsCount;
}

function shouldPreferPotionTakebackLines(
  game: MonsGame,
  perspective: Color,
): boolean {
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
    potionsForColor(root.game, perspective) <
    potionsForColor(gameBefore, perspective)
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
  const context = selectionContext(
    game,
    roots,
    candidateIndices,
    perspective,
    config,
  );
  for (const kind of PRODUCTION_COMPETITION_KIND_ORDER) {
    const kindCompetes =
      isProductionMode(config) &&
      evaluateProductionCompetitionRules(
        options.productionPolicy,
        kind,
        context,
      );
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
  if (
    candidates.some((index) => valueAt(roots, index).attacksOpponentDrainer)
  ) {
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
      !anyProductionCompetition(
        game,
        roots,
        candidates,
        perspective,
        config,
        options,
      )
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
          const preferred =
            safeSpiritReady.length > 0 ? safeSpiritReady : spiritReady;
          const keepsSpiritAndScores = candidates.some((index) => {
            const root = valueAt(roots, index);
            return (
              root.keepsAwakeSpiritOnBase &&
              scoreForColor(root.game, perspective) > scoreBefore
            );
          });
          const spiritLineScores = preferred.some(
            (index) =>
              scoreForColor(valueAt(roots, index).game, perspective) >
              scoreBefore,
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
        const context = selectionContext(
          game,
          roots,
          preSafety,
          perspective,
          config,
        );
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
      !anyProductionCompetition(
        game,
        roots,
        candidates,
        perspective,
        config,
        options,
      )
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
        valueAt(roots, index).score + ROOT_POTION_HOLD_SCORE_MARGIN >=
        bestScore,
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
        return (
          !root.manaHandoffToOpponent &&
          !root.hasRoundtrip &&
          !allowsLoss(index)
        );
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
    const context = selectionContext(
      game,
      roots,
      [...candidates],
      perspective,
      config,
    );
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

export function compareTacticalEvaluatedRoots(
  candidate: EvaluatedRoot,
  incumbent: EvaluatedRoot,
): number {
  const preferBoolean = (
    candidateValue: boolean,
    incumbentValue: boolean,
    preferTrue: boolean,
  ): number | undefined => {
    if (candidateValue === incumbentValue) return undefined;
    return candidateValue === preferTrue ? -1 : 1;
  };
  let order = preferBoolean(
    candidate.winsImmediately,
    incumbent.winsImmediately,
    true,
  );
  order ??= preferBoolean(
    candidate.attacksOpponentDrainer,
    incumbent.attacksOpponentDrainer,
    true,
  );
  order ??= preferBoolean(
    candidate.ownDrainerVulnerable,
    incumbent.ownDrainerVulnerable,
    false,
  );
  order ??= preferBoolean(
    candidate.classes.immediateScore,
    incumbent.classes.immediateScore,
    true,
  );
  order ??= preferBoolean(
    candidate.scoresSupermanaThisTurn,
    incumbent.scoresSupermanaThisTurn,
    true,
  );
  order ??= preferBoolean(
    candidate.scoresOpponentManaThisTurn,
    incumbent.scoresOpponentManaThisTurn,
    true,
  );
  order ??= preferBoolean(
    candidate.safeSupermanaPickupNow,
    incumbent.safeSupermanaPickupNow,
    true,
  );
  order ??= preferBoolean(
    candidate.safeOpponentManaPickupNow,
    incumbent.safeOpponentManaPickupNow,
    true,
  );
  if (order !== undefined) return order;
  if (
    candidate.sameTurnScoreWindowValue !== incumbent.sameTurnScoreWindowValue
  ) {
    return candidate.sameTurnScoreWindowValue >
      incumbent.sameTurnScoreWindowValue
      ? -1
      : 1;
  }
  order = preferBoolean(
    candidate.spiritSameTurnScoreSetupNow,
    incumbent.spiritSameTurnScoreSetupNow,
    true,
  );
  order ??= preferBoolean(
    candidate.spiritOwnManaSetupNow,
    incumbent.spiritOwnManaSetupNow,
    true,
  );
  if (order !== undefined) return order;
  if (
    candidate.spiritOwnManaSetupNow &&
    incumbent.spiritOwnManaSetupNow &&
    candidate.supermanaProgress &&
    incumbent.supermanaProgress &&
    candidate.safeSupermanaProgressSteps !==
      incumbent.safeSupermanaProgressSteps
  ) {
    return rootProgressStepsBetter(
      candidate.safeSupermanaProgressSteps,
      incumbent.safeSupermanaProgressSteps,
    )
      ? -1
      : 1;
  }
  if (
    candidate.spiritOwnManaSetupNow &&
    incumbent.spiritOwnManaSetupNow &&
    candidate.opponentManaProgress &&
    incumbent.opponentManaProgress &&
    candidate.safeOpponentManaProgressSteps !==
      incumbent.safeOpponentManaProgressSteps
  ) {
    return rootProgressStepsBetter(
      candidate.safeOpponentManaProgressSteps,
      incumbent.safeOpponentManaProgressSteps,
    )
      ? -1
      : 1;
  }
  if (
    candidate.spiritOwnManaSetupNow &&
    incumbent.spiritOwnManaSetupNow &&
    candidate.scorePathBestSteps !== incumbent.scorePathBestSteps
  ) {
    return rootScorePathStepsBetter(
      candidate.scorePathBestSteps,
      incumbent.scorePathBestSteps,
    )
      ? -1
      : 1;
  }
  order = preferBoolean(
    candidate.supermanaProgress,
    incumbent.supermanaProgress,
    true,
  );
  if (order !== undefined) return order;
  if (
    candidate.supermanaProgress &&
    incumbent.supermanaProgress &&
    candidate.safeSupermanaProgressSteps !==
      incumbent.safeSupermanaProgressSteps
  ) {
    return rootProgressStepsBetter(
      candidate.safeSupermanaProgressSteps,
      incumbent.safeSupermanaProgressSteps,
    )
      ? -1
      : 1;
  }
  order = preferBoolean(
    candidate.opponentManaProgress,
    incumbent.opponentManaProgress,
    true,
  );
  if (order !== undefined) return order;
  if (
    candidate.opponentManaProgress &&
    incumbent.opponentManaProgress &&
    candidate.safeOpponentManaProgressSteps !==
      incumbent.safeOpponentManaProgressSteps
  ) {
    return rootProgressStepsBetter(
      candidate.safeOpponentManaProgressSteps,
      incumbent.safeOpponentManaProgressSteps,
    )
      ? -1
      : 1;
  }
  order = preferBoolean(
    candidate.manaHandoffToOpponent,
    incumbent.manaHandoffToOpponent,
    false,
  );
  order ??= preferBoolean(
    candidate.hasRoundtrip,
    incumbent.hasRoundtrip,
    false,
  );
  order ??= preferBoolean(
    candidate.spiritDevelopment,
    incumbent.spiritDevelopment,
    true,
  );
  if (order !== undefined) return order;
  if (candidate.policyPriority !== incumbent.policyPriority) {
    return candidate.policyPriority > incumbent.policyPriority ? -1 : 1;
  }
  if (candidate.efficiency !== incumbent.efficiency) {
    return candidate.efficiency > incumbent.efficiency ? -1 : 1;
  }
  return 0;
}

export function compareRankedEvaluatedRootIndices(
  roots: readonly EvaluatedRoot[],
  candidateIndex: number,
  incumbentIndex: number,
): number {
  const candidate = valueAt(roots, candidateIndex);
  const incumbent = valueAt(roots, incumbentIndex);
  if (candidate.score !== incumbent.score) {
    return candidate.score > incumbent.score ? -1 : 1;
  }
  return (
    compareTacticalEvaluatedRoots(candidate, incumbent) ||
    candidateIndex - incumbentIndex
  );
}

function bestScoredRootIndex(
  roots: readonly EvaluatedRoot[],
  candidateIndices: readonly number[],
): number {
  let bestIndex = candidateIndices[0] ?? 0;
  for (const index of candidateIndices) {
    if (compareRankedEvaluatedRootIndices(roots, index, bestIndex) < 0) {
      bestIndex = index;
    }
  }
  return bestIndex;
}

/** Positive means candidate wins the challenge; negative means incumbent wins. */
export function spiritScoreChallengeOrder(
  candidate: EvaluatedRoot,
  incumbent: EvaluatedRoot,
): number | undefined {
  const candidatePlainSpirit = isPlainSpiritDevelopmentRoot(candidate);
  const incumbentPlainSpirit = isPlainSpiritDevelopmentRoot(incumbent);
  if (candidatePlainSpirit === incumbentPlainSpirit) return undefined;
  const challenger = candidatePlainSpirit ? incumbent : candidate;
  const spirit = candidatePlainSpirit ? candidate : incumbent;
  const candidateIsChallenger = !candidatePlainSpirit;
  if (
    isUnsafe(challenger) ||
    challenger.hasRoundtrip ||
    challenger.score <
      saturatingScoreAdd(spirit.score, SPIRIT_SCORE_CHALLENGE_MARGIN) ||
    challenger.sameTurnScoreWindowValue < spirit.sameTurnScoreWindowValue
  ) {
    return undefined;
  }
  return candidateIsChallenger ? 1 : -1;
}

function compareProductionRules(
  phase: ProductionComparisonPhase,
  context: ProductionRootComparisonContext,
  policy: ProductionRootPolicy | undefined,
): number | undefined {
  for (const rule of policy?.comparisonRules ?? []) {
    if (rule.phase !== phase) continue;
    const result = rule.compare(context);
    if (result.kind === "compare") {
      return result.order === 0 ? 0 : result.order > 0 ? 1 : -1;
    }
  }
  return undefined;
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
  const context = selectionContext(
    game,
    roots,
    candidateIndices,
    perspective,
    config,
  );
  if (config.replyRisk.enabled) {
    const guarded = options.pickReplyRiskGuardedIndex?.(context);
    if (guarded !== undefined) {
      assertRootIndex(roots, guarded, "reply-risk guard");
      if (!candidateIndices.includes(guarded)) {
        throw new RangeError(
          "reply-risk guard selected a root outside its shortlist",
        );
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
    if (
      evaluation.score + scoreMargin < bestScore &&
      !allowClosePlainSpiritSlack
    ) {
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
        (evaluation.spiritOwnManaSetupNow ||
          evaluation.spiritSameTurnScoreSetupNow));
    const spiritSameTurnSetupBetter =
      evaluation.spiritSameTurnScoreSetupNow &&
      !best.spiritSameTurnScoreSetupNow &&
      spiritSetupCompetes;
    const equalSpiritSameTurnSetup =
      evaluation.spiritSameTurnScoreSetupNow ===
      best.spiritSameTurnScoreSetupNow;
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
      evaluation.safeOpponentManaProgressSteps ===
        best.safeOpponentManaProgressSteps;
    const spiritSetupScorePathBetter =
      evaluation.spiritOwnManaSetupNow &&
      best.spiritOwnManaSetupNow &&
      rootScorePathStepsBetter(
        evaluation.scorePathBestSteps,
        best.scorePathBestSteps,
      );
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
      evaluation.safeOpponentManaProgressSteps ===
      best.safeOpponentManaProgressSteps;
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
      saturatingScoreAdd(
        best.policyPriority,
        INTERVIEW_SOFT_PRIORITY_SCORE_MARGIN,
      );
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
  const index = pickBaselineRootIndex(
    game,
    roots,
    perspective,
    config,
    options,
  );
  return index === undefined ? [] : [...valueAt(roots, index).inputs];
}
