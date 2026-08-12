import { Color, MonKind, type Mana } from "../../api/types.js";
import {
  inputChainKey,
  itemMon,
  otherColor,
  type Event,
  type Input,
} from "../../engine/model/domain.js";
import { FOR_AUTOMOVE_START_INPUT_OPTIONS } from "../../engine/game/input-support.js";
import { MonsGame } from "../../engine/game/mons-game.js";
import { scoreForColor } from "../../engine/rules/legality.js";
import {
  TERMINAL_SEARCH_SCORE,
  clampHeuristicScore,
  saturatingScoreAdd,
  saturatingScoreSubtract,
} from "../core/score-math.js";
import type { AutomoveExecutionContext } from "../core/execution-context.js";
import { exactSearchStateHash } from "../exact/hash.js";
import { evaluatePreferabilityWithWeightsAndExactPolicy } from "../scoring/evaluator.js";
import { patchAutomoveConfig } from "../config/patch.js";
import {
  hasAwakeSpiritOnBase,
  shouldPreferSpiritDevelopment,
} from "../config/types.js";
import {
  hasRoundtripMonMove as hasRoundtrip,
  manaHandoffPenalty,
  moveEfficiencyDeltaFromBeforeSnapshot,
  moveEfficiencySnapshotWithHash,
  type MoveEfficiencySnapshot,
} from "./move-efficiency.js";
import { applyInputsForSearchWithEvents } from "../transitions/simulation.js";
import { enumerateLegalTransitions } from "../transitions/enumerate.js";
import type { LegalInputTransition } from "../transitions/types.js";
import { isOwnDrainerVulnerable } from "./vulnerability.js";
import {
  approximateActiveTurnSummary,
  classifyTransition,
  hasSpiritDevelopment,
  liveSpiritSetupGain,
  manaMovedToward,
  orderingEventBonus,
  picksUpMana,
  rootSoftPriority,
  rootTurnSummary,
  safeCarrierForMana,
  scoresMana,
  spiritManaSetup,
  spiritMovesManaToward,
} from "./observations.js";
import { compareRootCandidates, truncateWithClassCoverage } from "./ranking.js";
import {
  appendUniqueTransitions,
  canAttemptForcedDrainerAttackFallback,
  collectDrainerAttackInputs,
  collectTargetedDrainerSafetyInputs,
  collectTargetedSafeDrainerPickupInputs,
  collectTargetedSpiritSetupInputs,
  drainerSafetyFallbackCandidatesLimit,
  forcedAttackCandidatesLimit,
  genericRootFallbackEnumLimit,
  hasSpiritScoringManaSetup,
  safeDrainerPickupFallbackCandidatesLimit,
  spiritSetupFallbackCandidatesLimit,
  transitionHasSafeDrainerPickup,
} from "./targeted.js";
import {
  UNKNOWN_PROGRESS_STEPS,
  UNKNOWN_SCORE_PATH_STEPS,
  type RootCandidate,
  type RootCandidateDraft,
  type SearchConfig,
} from "./types.js";

type ExactLiteBudget = {
  rootCalls: number;
  staticCalls: number;
};

function rootTransitionRequiresExactLiteProgress(events: readonly Event[]): boolean {
  return events.some((event) =>
    [
      "mana-move",
      "mana-scored",
      "pickup-mana",
      "mana-dropped",
      "supermana-back-to-base",
    ].includes(event.kind),
  );
}

function rootTransitionRequiresExactLiteSpiritWindow(
  events: readonly Event[],
): boolean {
  return events.some(
    (event) =>
      event.kind === "spirit-target-move" ||
      (event.kind === "mon-move" && itemMon(event.item)?.kind === MonKind.Spirit),
  );
}

function transitionRequiresExactLite(events: readonly Event[]): boolean {
  return (
    rootTransitionRequiresExactLiteProgress(events) ||
    rootTransitionRequiresExactLiteSpiritWindow(events)
  );
}

function withExactLiteBudgetedTransitionConfig(
  config: SearchConfig,
  perspective: Color,
  transition: LegalInputTransition,
  budget: ExactLiteBudget,
): SearchConfig {
  let rootCallBudget = config.budget.exactLiteRootCalls;
  let staticCallBudget = config.budget.exactLiteStaticCalls;
  if (
    config.search.exactLiteChecks &&
    rootCallBudget > 0 &&
    transitionRequiresExactLite(transition.events)
  ) {
    if (budget.rootCalls > 0) budget.rootCalls -= 1;
    else rootCallBudget = 0;
  }
  if (
    config.search.exactLiteChecks &&
    staticCallBudget > 0 &&
    transition.game.activeColor === perspective
  ) {
    if (budget.staticCalls > 0) budget.staticCalls -= 1;
    else staticCallBudget = 0;
  }
  rootCallBudget = Math.min(rootCallBudget, budget.rootCalls);
  staticCallBudget = Math.min(staticCallBudget, budget.staticCalls);
  return patchAutomoveConfig(config, {
    budget: {
      exactLiteRootCalls: rootCallBudget,
      exactLiteStaticCalls: staticCallBudget,
    },
    search: {
      exactRootAnalysis: config.search.exactLiteChecks
        ? rootCallBudget > 0 && transitionRequiresExactLite(transition.events)
        : config.search.exactRootAnalysis,
    },
  });
}

function rootCandidateSourceSnapshot(
  context: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
): MoveEfficiencySnapshot {
  const stateHash = exactSearchStateHash(game);
  return moveEfficiencySnapshotWithHash(
    context,
    game,
    perspective,
    false,
    false,
    stateHash,
  );
}

function buildRootCandidate(
  context: AutomoveExecutionContext,
  before: MonsGame,
  transition: LegalInputTransition,
  perspective: Color,
  config: SearchConfig,
  vulnerableBefore: boolean,
  getSourceSnapshot: () => MoveEfficiencySnapshot,
): RootCandidateDraft | undefined {
  if (context.session.checkpoint()) return undefined;
  const after = transition.game;
  const stateHash = exactSearchStateHash(after);
  const efficiency = moveEfficiencyDeltaFromBeforeSnapshot(
    context,
    before,
    after,
    perspective,
    transition.events,
    getSourceSnapshot(),
    stateHash,
    {
      isRoot: true,
      applyBacktrackPenalty: true,
      applyRootManaHandoffGuard: true,
      includeTacticalExact: false,
      includeStrategicExact: false,
      rootBacktrackPenalty: config.evaluation.rootBacktrackPenalty,
      rootManaHandoffPenalty: config.evaluation.rootManaHandoffPenalty,
    },
  );
  if (context.session.checkpoint()) return undefined;
  const ownDrainerVulnerable = isOwnDrainerVulnerable(context, after, perspective);
  const ownDrainerWalkVulnerable = false;
  const classes = classifyTransition(
    context,
    before,
    transition,
    perspective,
    vulnerableBefore,
    ownDrainerVulnerable,
  );
  const scoresSupermanaThisTurn = scoresMana(
    transition.events,
    (mana) => mana.kind === "supermana",
  );
  const scoresOpponentManaThisTurn = scoresMana(
    transition.events,
    (mana) => mana.kind === "regular" && mana.color !== perspective,
  );
  const picksSupermana = picksUpMana(
    transition.events,
    (mana) => mana.kind === "supermana",
  );
  const picksOpponentMana = picksUpMana(
    transition.events,
    (mana) => mana.kind === "regular" && mana.color !== perspective,
  );
  const summary = rootTurnSummary(
    context,
    after,
    perspective,
    config.search.exactRootAnalysis,
    config.search.exactLiteChecks && config.budget.exactLiteStaticCalls > 0,
  );
  const safeSupermanaPickupNow =
    picksSupermana &&
    safeCarrierForMana(context, after, perspective, { kind: "supermana" });
  const safeOpponentManaPickupNow =
    picksOpponentMana &&
    safeCarrierForMana(context, after, perspective, {
      kind: "regular",
      color: otherColor(perspective),
    });
  const spiritSupermanaSetup = spiritManaSetup(
    context,
    after,
    transition.events,
    perspective,
    { kind: "supermana" },
  );
  const spiritOpponentManaSetup = spiritManaSetup(
    context,
    after,
    transition.events,
    perspective,
    { kind: "regular", color: otherColor(perspective) },
  );
  const supermanaProgress =
    scoresSupermanaThisTurn ||
    picksSupermana ||
    manaMovedToward(
      transition.events,
      perspective,
      (mana) => mana.kind === "supermana",
    ) ||
    spiritSupermanaSetup ||
    (summary?.safeSupermanaProgress ?? false) ||
    (summary?.spiritAssistedSupermanaProgress ?? false);
  const opponentManaProgress =
    scoresOpponentManaThisTurn ||
    picksOpponentMana ||
    manaMovedToward(
      transition.events,
      perspective,
      (mana) => mana.kind === "regular" && mana.color !== perspective,
    ) ||
    spiritOpponentManaSetup ||
    (summary?.safeOpponentManaProgress ?? false) ||
    (summary?.spiritAssistedOpponentManaProgress ?? false) ||
    (summary?.spiritAssistedDenial ?? false);
  const safeSupermanaProgressSteps =
    summary?.safeSupermanaProgressSteps ?? UNKNOWN_PROGRESS_STEPS;
  const safeOpponentManaProgressSteps =
    summary?.safeOpponentManaProgressSteps ?? UNKNOWN_PROGRESS_STEPS;
  const scorePathBestSteps = summary?.scorePathBestSteps ?? UNKNOWN_SCORE_PATH_STEPS;
  const sameTurnScoreWindowValue = summary?.sameTurnScoreWindowValue ?? 0;
  const spiritDevelopment = hasSpiritDevelopment(
    before,
    after,
    perspective,
    transition.events,
  );
  const spiritSameTurnScoreSetupNow =
    transition.events.some((event) => event.kind === "spirit-target-move") &&
    after.activeColor === perspective &&
    sameTurnScoreWindowValue > 0;
  const spiritOwnManaSetupNow =
    spiritMovesManaToward(
      transition.events,
      perspective,
      (mana) => mana.kind === "regular" && mana.color === perspective,
    ) ||
    spiritSupermanaSetup ||
    spiritOpponentManaSetup;
  const spiritSetupGain = liveSpiritSetupGain(
    summary,
    spiritDevelopment,
    spiritSameTurnScoreSetupNow,
    spiritOwnManaSetupNow,
  );
  const winsImmediately = after.winnerColor() === perspective;
  const attacksOpponentDrainer = classes.drainerAttack;
  const rootCompensatesHandoff =
    winsImmediately ||
    attacksOpponentDrainer ||
    scoresSupermanaThisTurn ||
    scoresOpponentManaThisTurn ||
    summary?.spiritAssistedScore === true;
  const manaHandoffToOpponent =
    !rootCompensatesHandoff &&
    manaHandoffPenalty(
      transition.events,
      perspective,
      Math.max(0, config.evaluation.rootManaHandoffPenalty),
    ) > 0;
  const roundtrip = hasRoundtrip(transition.events);
  const policyPriority = rootSoftPriority(config, {
    supermanaProgress,
    opponentManaProgress,
    safeSupermanaProgressSteps,
    safeOpponentManaProgressSteps,
    scoresSupermanaThisTurn,
    scoresOpponentManaThisTurn,
    ownDrainerVulnerable,
    manaHandoffToOpponent,
    hasRoundtrip: roundtrip,
  });
  const terminalScore = terminalSearchScore(
    after,
    perspective,
    Math.max(0, config.budget.depth - 1),
    config.budget.depth,
  );
  let heuristic =
    terminalScore ??
    evaluatePreferabilityWithWeightsAndExactPolicy(
      context,
      after,
      perspective,
      config.evaluation.weights,
      false,
    );
  heuristic = saturatingScoreAdd(
    heuristic,
    orderingEventBonus(before.activeColor, perspective, transition.events),
  );
  heuristic = saturatingScoreAdd(heuristic, policyPriority);
  const spentPotion = transition.events.some((event) => event.kind === "use-potion");
  const compensatedPotion =
    winsImmediately ||
    attacksOpponentDrainer ||
    scoreForColor(after, perspective) >= scoreForColor(before, perspective) + 2 ||
    scoresSupermanaThisTurn ||
    scoresOpponentManaThisTurn ||
    summary?.spiritAssistedScore === true ||
    (!ownDrainerVulnerable && (supermanaProgress || opponentManaProgress));
  if (spentPotion && !compensatedPotion) {
    heuristic = saturatingScoreSubtract(
      heuristic,
      Math.max(0, config.evaluation.potionSpendPenalty),
    );
  }
  if (terminalScore === undefined) {
    heuristic = clampHeuristicScore(heuristic);
  }
  return {
    inputs: transition.inputs,
    game: after,
    events: transition.events,
    stateHash,
    heuristic,
    efficiency,
    winsImmediately,
    attacksOpponentDrainer,
    ownDrainerVulnerable,
    ownDrainerWalkVulnerable,
    spiritDevelopment,
    keepsAwakeSpiritOnBase:
      hasAwakeSpiritOnBase(before, perspective) &&
      hasAwakeSpiritOnBase(after, perspective),
    manaHandoffToOpponent,
    hasRoundtrip: roundtrip,
    scoresSupermanaThisTurn,
    scoresOpponentManaThisTurn,
    safeSupermanaPickupNow,
    safeOpponentManaPickupNow,
    safeSupermanaProgressSteps,
    safeOpponentManaProgressSteps,
    scorePathBestSteps,
    sameTurnScoreWindowValue,
    spiritSetupGain,
    spiritSameTurnScoreSetupNow,
    spiritOwnManaSetupNow,
    supermanaProgress,
    opponentManaProgress,
    policyPriority,
    classes,
  };
}

/** Build a scored root candidate when the advisor has no engine head. */
export function buildRootCandidateForInputs(
  context: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: SearchConfig,
  inputs: readonly Input[],
): RootCandidate | undefined {
  const copiedInputs = [...inputs];
  const applied = applyInputsForSearchWithEvents(game, copiedInputs);
  if (applied === undefined) return undefined;
  const candidate = buildRootCandidate(
    context,
    game,
    {
      inputs: copiedInputs,
      game: applied.game,
      events: applied.events,
    },
    perspective,
    config,
    isOwnDrainerVulnerable(context, game, perspective),
    () => rootCandidateSourceSnapshot(context, game, perspective),
  );
  return candidate === undefined ? undefined : { ...candidate, rootRank: 0 };
}

export function rankRootCandidates(
  context: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: SearchConfig,
): RootCandidate[] {
  if (context.session.checkpoint()) return [];
  const sourceFen = game.fen();
  const vulnerableBefore = isOwnDrainerVulnerable(context, game, perspective);
  const rootTransitions = enumerateLegalTransitions(
    context,
    game,
    config.search.rootEnumerationLimit,
    FOR_AUTOMOVE_START_INPUT_OPTIONS,
  );
  if (
    vulnerableBefore &&
    !rootTransitions.some(
      (transition) => !isOwnDrainerVulnerable(context, transition.game, perspective),
    )
  ) {
    appendUniqueTransitions(
      rootTransitions,
      collectTargetedDrainerSafetyInputs(
        context,
        game,
        perspective,
        config,
        drainerSafetyFallbackCandidatesLimit(config),
      ),
    );
  }
  if (context.session.checkpoint()) return [];

  const turnBefore = approximateActiveTurnSummary(context, game, perspective, false);
  const spiritSetupGainBefore = liveSpiritSetupGain(turnBefore, false, false, false);
  if (context.session.checkpoint()) return [];
  const supermana: Mana = { kind: "supermana" };
  if (
    turnBefore.safeSupermanaProgress &&
    !rootTransitions.some((transition) =>
      transitionHasSafeDrainerPickup(context, transition, perspective, supermana),
    )
  ) {
    appendUniqueTransitions(
      rootTransitions,
      collectTargetedSafeDrainerPickupInputs(
        context,
        game,
        perspective,
        safeDrainerPickupFallbackCandidatesLimit(config),
        supermana,
      ),
    );
  }
  if (context.session.checkpoint()) return [];
  const opponentMana: Mana = {
    kind: "regular",
    color: otherColor(perspective),
  };
  if (
    turnBefore.safeOpponentManaProgress &&
    !rootTransitions.some((transition) =>
      transitionHasSafeDrainerPickup(context, transition, perspective, opponentMana),
    )
  ) {
    appendUniqueTransitions(
      rootTransitions,
      collectTargetedSafeDrainerPickupInputs(
        context,
        game,
        perspective,
        safeDrainerPickupFallbackCandidatesLimit(config),
        opponentMana,
      ),
    );
  }
  if (context.session.checkpoint()) return [];
  if (
    (config.policy.hardSpiritDeployment || config.policy.preferSpiritDevelopment) &&
    (shouldPreferSpiritDevelopment(game, perspective) ||
      spiritSetupGainBefore > 0 ||
      turnBefore.spiritAssistedSupermanaProgress ||
      turnBefore.spiritAssistedOpponentManaProgress) &&
    !rootTransitions.some((transition) =>
      hasSpiritScoringManaSetup(
        context,
        transition.game,
        transition.events,
        perspective,
      ),
    )
  ) {
    appendUniqueTransitions(
      rootTransitions,
      collectTargetedSpiritSetupInputs(
        context,
        game,
        perspective,
        config,
        spiritSetupFallbackCandidatesLimit(config),
      ),
    );
  }
  if (context.session.cancelled) return [];

  const exactLiteBudget: ExactLiteBudget = {
    rootCalls: config.budget.exactLiteRootCalls,
    staticCalls: config.budget.exactLiteStaticCalls,
  };
  let sourceSnapshot: MoveEfficiencySnapshot | undefined;
  const getSourceSnapshot = (): MoveEfficiencySnapshot => {
    sourceSnapshot ??= rootCandidateSourceSnapshot(context, game, perspective);
    return sourceSnapshot;
  };
  const drafts: RootCandidateDraft[] = [];
  const appendCandidates = (transitions: readonly LegalInputTransition[]): boolean => {
    for (const transition of transitions) {
      if (context.session.checkpoint()) return false;
      const transitionConfig = withExactLiteBudgetedTransitionConfig(
        config,
        perspective,
        transition,
        exactLiteBudget,
      );
      const candidate = buildRootCandidate(
        context,
        game,
        transition,
        perspective,
        transitionConfig,
        vulnerableBefore,
        getSourceSnapshot,
      );
      if (context.session.checkpoint()) return false;
      if (candidate !== undefined) drafts.push(candidate);
    }
    return true;
  };
  if (!appendCandidates(rootTransitions)) return [];
  if (drafts.length === 0) {
    const fallbackTransitions = enumerateLegalTransitions(
      context,
      game,
      genericRootFallbackEnumLimit(config),
      FOR_AUTOMOVE_START_INPUT_OPTIONS,
    );
    if (context.session.checkpoint() || !appendCandidates(fallbackTransitions))
      return [];
  }
  let ranked = drafts.map((candidate, rank): RootCandidate => ({
    ...candidate,
    rootRank: rank,
  }));
  ranked.sort(compareRootCandidates);

  let hasWinningCandidate = ranked.some((candidate) => candidate.winsImmediately);
  let forcedAttackInputKeys: Set<string> | undefined;
  if (
    !hasWinningCandidate &&
    !ranked.some((candidate) => candidate.attacksOpponentDrainer) &&
    canAttemptForcedDrainerAttackFallback(game, perspective)
  ) {
    const fallbackTransitions = collectDrainerAttackInputs(
      context,
      game,
      perspective,
      config,
      forcedAttackCandidatesLimit(config),
      config.policy.targetedDrainerAttackFallback,
    );
    if (context.session.checkpoint()) return [];
    if (fallbackTransitions.length > 0) {
      forcedAttackInputKeys = new Set(
        fallbackTransitions.map((transition) => inputChainKey(transition.inputs)),
      );
      const seen = new Set(ranked.map((candidate) => inputChainKey(candidate.inputs)));
      for (const transition of fallbackTransitions) {
        if (context.session.checkpoint()) return [];
        const key = inputChainKey(transition.inputs);
        if (seen.has(key)) continue;
        seen.add(key);
        const transitionConfig = withExactLiteBudgetedTransitionConfig(
          config,
          perspective,
          transition,
          exactLiteBudget,
        );
        const draft = buildRootCandidate(
          context,
          game,
          transition,
          perspective,
          transitionConfig,
          vulnerableBefore,
          getSourceSnapshot,
        );
        if (context.session.checkpoint()) return [];
        if (draft !== undefined) {
          ranked.push({ ...draft, rootRank: 0 });
        }
      }
      ranked.sort(compareRootCandidates);
      hasWinningCandidate = ranked.some((candidate) => candidate.winsImmediately);
    }
  }

  if (
    !hasWinningCandidate &&
    ranked.some((candidate) => candidate.attacksOpponentDrainer)
  ) {
    ranked = ranked.filter((candidate) => candidate.attacksOpponentDrainer);
  } else if (forcedAttackInputKeys !== undefined) {
    ranked = ranked.filter((candidate) =>
      forcedAttackInputKeys.has(inputChainKey(candidate.inputs)),
    );
  }
  ranked = truncateWithClassCoverage(ranked, config.search.rootBranchLimit);
  ranked = ranked.map((candidate, rank) => ({
    ...candidate,
    rootRank: rank,
  }));
  if (game.fen() !== sourceFen) {
    throw new Error("root candidate enumeration mutated its source game");
  }
  return context.session.checkpoint() ? [] : ranked;
}

export function terminalSearchScore(
  game: MonsGame,
  perspective: Color,
  depth: number,
  searchDepth: number,
): number | undefined {
  const winner = game.winnerColor();
  if (winner === undefined) return undefined;
  const ply = Math.max(0, searchDepth - depth);
  return winner === perspective
    ? TERMINAL_SEARCH_SCORE - ply
    : -TERMINAL_SEARCH_SCORE + ply;
}
