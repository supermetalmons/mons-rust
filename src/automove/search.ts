import { Color, MonKind, colorId, type Input } from "../engine/domain.js";
import { MonsGame } from "../engine/game.js";
import { saturatingScoreAdd, saturatingScoreSubtract } from "./score-math.js";
import { clearExactStateAnalysisCache, exactSearchStateHash } from "./exact.js";
import type { AutomoveExecutionContext } from "./execution-context.js";
import {
  HASH64_ZERO,
  Hash64Table,
  hash64CompareUnsigned,
  hash64Equals,
  hash64IsZero,
  type Hash64,
} from "./hash64.js";
import {
  classifyTransition,
  compareRootCandidates,
  hasProTacticalPotential,
  isOwnDrainerVulnerable,
  orderingEventBonus,
  rankRootCandidates,
  terminalSearchScore,
  type MoveClassFlags,
  type RootCandidate,
  type SearchConfig,
} from "./root-candidates.js";
import {
  clearMoveEfficiencyCache,
  moveEfficiencyDeltaFromBeforeSnapshot,
  moveEfficiencySnapshotWithHash,
} from "./move-efficiency.js";
import { focusedRootCandidates } from "./root-focus.js";
import {
  evaluatePreferabilityWithWeightsAndExactPolicy,
  scoringProfileId,
} from "./scoring.js";
import { patchAutomoveConfig } from "./selector-config.js";
import {
  enumerateLegalTransitions,
  isQuiescenceTacticalTransition,
} from "./transitions.js";

const TT_BEST_CHILD_BONUS = 2_400;
const CHILD_CLASS_SCORE_MARGIN = 110;
const PREFERABILITY_CACHE_MAX_ENTRIES = 32_768;

type TranspositionBound = "exact" | "lower" | "upper";

type TranspositionEntry = {
  readonly depth: number;
  readonly score: number;
  readonly bound: TranspositionBound;
  readonly bestChildHash: Hash64;
};

export type RankedChild = {
  readonly game: MonsGame;
  readonly hash: Hash64;
  readonly heuristic: number;
  readonly orderingEfficiency: number;
  readonly tacticalExtensionTrigger: boolean;
  readonly quietReductionCandidate: boolean;
  readonly classes: MoveClassFlags;
};

type SearchNodeAccounting = {
  visitedNodes: number;
  cacheHits: number;
  quiescenceNodes: number;
  extensionNodes: number;
};

type SearchContext = {
  readonly execution: AutomoveExecutionContext;
  readonly perspective: Color;
  readonly config: SearchConfig;
  readonly transposition: Hash64Table<TranspositionEntry>;
  readonly extensionNodeBudget: number;
  readonly stats: SearchNodeAccounting;
};

export type EvaluatedRoot = RootCandidate & {
  readonly score: number;
  readonly nodesAfter: number;
};

export type SearchRootOptions = {
  /** Advisor-selected roots, computed before the two-pass focus allocation. */
  readonly priorityInputs?: readonly (readonly Input[])[];
  /** A single prepass-selected input chain that must survive root focusing. */
  readonly forcedInputs?: readonly Input[];
  /** Production SpiritImpact/nonnegative-deny plan-family qualification. */
  readonly qualifiesPlainSpiritPlan?: (candidate: RootCandidate) => boolean;
  /** Production DrainerSafetyRecovery plan-family qualification. */
  readonly qualifiesDrainerSafetyRecoveryPlan?: (
    candidate: RootCandidate,
  ) => boolean;
};

export type SearchResult = {
  readonly best: EvaluatedRoot | undefined;
  readonly evaluations: readonly EvaluatedRoot[];
  readonly visitedNodes: number;
  readonly cacheHits: number;
  readonly timedOut: boolean;
};

type FocusedSearchRoots = {
  readonly candidates: readonly RootCandidate[];
  readonly scoutVisitedNodes: number;
};

const PREFERABILITY_CACHE = Symbol("preferability-cache");

function preferabilityCache(
  execution: AutomoveExecutionContext,
): Hash64Table<number> {
  return execution.caches.session.getOrCreate(
    PREFERABILITY_CACHE,
    () => new Hash64Table<number>(PREFERABILITY_CACHE_MAX_ENTRIES),
  );
}

function cachedPreferability(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: SearchConfig,
  stats: SearchNodeAccounting,
  stateHash = exactSearchStateHash(game),
): number {
  const cache = preferabilityCache(execution);
  const tag = colorId(perspective);
  const profileId = scoringProfileId(config.evaluation.weights);
  const cached = cache.get(stateHash, tag, undefined, profileId);
  if (cached !== undefined) {
    stats.cacheHits += 1;
    return cached;
  }
  const value = evaluatePreferabilityWithWeightsAndExactPolicy(
    execution,
    game,
    perspective,
    config.evaluation.weights,
    false,
  );
  if (execution.session.cacheWriteAllowed) {
    if (
      cache.size >= config.evaluation.cacheCapacity &&
      !cache.has(stateHash, tag, undefined, profileId)
    ) {
      cache.clear();
    }
    cache.set(stateHash, value, tag, undefined, profileId);
  }
  return value;
}

function classPriority(classes: MoveClassFlags): number {
  let score = 0;
  if (classes.immediateScore) score += 1_000;
  if (classes.drainerAttack) score += 700;
  if (classes.drainerSafetyRecover) score += 500;
  if (classes.carrierProgress) score += 220;
  if (classes.material) score += 80;
  return score;
}

function compareHashesDescending(left: Hash64, right: Hash64): number {
  return -hash64CompareUnsigned(left, right);
}

export function compareRankedChildren(
  left: RankedChild,
  right: RankedChild,
  maximizing: boolean,
): number {
  if (left.heuristic !== right.heuristic) {
    return maximizing
      ? right.heuristic - left.heuristic
      : left.heuristic - right.heuristic;
  }
  if (left.orderingEfficiency !== right.orderingEfficiency) {
    return right.orderingEfficiency - left.orderingEfficiency;
  }
  const classOrder = classPriority(right.classes) - classPriority(left.classes);
  return classOrder !== 0
    ? classOrder
    : compareHashesDescending(left.hash, right.hash);
}

function childWithinCoverageMargin(
  score: number,
  cutoff: number,
  maximizing: boolean,
): boolean {
  return maximizing
    ? saturatingScoreAdd(score, CHILD_CLASS_SCORE_MARGIN) >= cutoff
    : score <= saturatingScoreAdd(cutoff, CHILD_CLASS_SCORE_MARGIN);
}

export function isPriorityChild(child: RankedChild): boolean {
  return (
    child.classes.immediateScore ||
    child.classes.drainerAttack ||
    child.classes.drainerSafetyRecover ||
    child.classes.carrierProgress ||
    (child.orderingEfficiency > 0 && !child.classes.material)
  );
}

export function truncateChildrenWithCoverage(
  children: readonly RankedChild[],
  limit: number,
  maximizing: boolean,
  strictGuarantees = true,
): RankedChild[] {
  if (children.length <= limit || limit === 0) return [...children];
  const cutoff = children[limit - 1]?.heuristic ?? 0;
  const preserveIndex = children.findIndex(
    (child, index) =>
      index >= limit &&
      isPriorityChild(child) &&
      (strictGuarantees ||
        childWithinCoverageMargin(child.heuristic, cutoff, maximizing)),
  );
  if (preserveIndex < 0) return children.slice(0, limit);
  const selected = new Array<boolean>(children.length).fill(false);
  selected[preserveIndex] = true;
  let selectedCount = 1;
  for (let index = 0; index < selected.length; index += 1) {
    if (selectedCount >= limit) break;
    if (selected[index] === true) continue;
    selected[index] = true;
    selectedCount += 1;
  }
  return children.filter((_child, index) => selected[index] === true);
}

export function enforceTacticalChildTop2(
  children: RankedChild[],
  maximizing: boolean,
  strictGuarantees = true,
): void {
  if (children.length < 3 || children.slice(0, 2).some(isPriorityChild)) {
    return;
  }
  const secondScore = children[1]?.heuristic ?? 0;
  const replacementIndex = children.findIndex((child, index) => {
    if (index < 2 || !isPriorityChild(child)) return false;
    return (
      strictGuarantees ||
      childWithinCoverageMargin(child.heuristic, secondScore, maximizing)
    );
  });
  if (replacementIndex >= 2) {
    const second = children[1];
    const replacement = children[replacementIndex];
    if (second !== undefined && replacement !== undefined) {
      children[1] = replacement;
      children[replacementIndex] = second;
    }
  }
}

export function isQuietReductionCandidate(
  orderingEfficiency: number,
  tacticalExtensionTrigger: boolean,
  classes: MoveClassFlags,
): boolean {
  return (
    !classes.material &&
    orderingEfficiency <= 0 &&
    !tacticalExtensionTrigger &&
    !classes.immediateScore &&
    !classes.drainerAttack &&
    !classes.drainerSafetyRecover &&
    !classes.carrierProgress
  );
}

export function isSelectiveExtensionCandidate(
  tacticalExtensionTrigger: boolean,
  orderingEfficiency: number,
  classes: MoveClassFlags,
): boolean {
  return (
    tacticalExtensionTrigger ||
    (orderingEfficiency > 0 && !classes.quiet && !classes.material)
  );
}

function rankedChildren(
  game: MonsGame,
  context: SearchContext,
  beforeStateHash: Hash64,
  preferredChildHash: Hash64 | undefined,
): RankedChild[] {
  if (context.execution.session.checkpoint()) return [];
  const maximizing = game.activeColor === context.perspective;
  const actorColor = game.activeColor;
  const beforeEfficiencySnapshot = moveEfficiencySnapshotWithHash(
    context.execution,
    game,
    context.perspective,
    false,
    false,
    beforeStateHash,
  );
  const ownDrainerVulnerableBefore = isOwnDrainerVulnerable(
    context.execution,
    game,
    actorColor,
  );
  const children: RankedChild[] = [];
  for (const transition of enumerateLegalTransitions(
    context.execution,
    game,
    context.config.search.nodeEnumerationLimit,
  )) {
    if (context.execution.session.checkpoint()) return [];
    const hash = exactSearchStateHash(transition.game);
    const ownDrainerVulnerableAfter = isOwnDrainerVulnerable(
      context.execution,
      transition.game,
      actorColor,
    );
    const classes = classifyTransition(
      context.execution,
      game,
      transition,
      actorColor,
      ownDrainerVulnerableBefore,
      ownDrainerVulnerableAfter,
    );
    const orderingEfficiency = moveEfficiencyDeltaFromBeforeSnapshot(
      context.execution,
      game,
      transition.game,
      actorColor,
      transition.events,
      beforeEfficiencySnapshot,
      hash,
      {
        isRoot: false,
        applyBacktrackPenalty: false,
        applyRootManaHandoffGuard: false,
        includeTacticalExact: false,
        includeStrategicExact: false,
        rootBacktrackPenalty: context.config.evaluation.rootBacktrackPenalty,
        rootManaHandoffPenalty:
          context.config.evaluation.rootManaHandoffPenalty,
      },
    );
    let heuristic =
      terminalSearchScore(
        transition.game,
        context.perspective,
        0,
        context.config.budget.depth,
      ) ??
      cachedPreferability(
        context.execution,
        transition.game,
        context.perspective,
        context.config,
        context.stats,
        hash,
      );
    heuristic = saturatingScoreAdd(
      heuristic,
      orderingEventBonus(actorColor, context.perspective, transition.events),
    );
    if (
      preferredChildHash !== undefined &&
      hash64Equals(hash, preferredChildHash)
    ) {
      heuristic = saturatingScoreAdd(heuristic, TT_BEST_CHILD_BONUS);
    }
    const tacticalExtensionTrigger =
      ownDrainerVulnerableBefore !== ownDrainerVulnerableAfter ||
      transition.events.some(
        (event) =>
          event.kind === "mana-scored" ||
          (event.kind === "mon-fainted" && event.mon.kind === MonKind.Drainer),
      );
    children.push({
      game: transition.game,
      hash,
      heuristic,
      orderingEfficiency,
      tacticalExtensionTrigger,
      quietReductionCandidate: isQuietReductionCandidate(
        orderingEfficiency,
        tacticalExtensionTrigger,
        classes,
      ),
      classes,
    });
  }
  children.sort((left, right) =>
    compareRankedChildren(left, right, maximizing),
  );
  enforceTacticalChildTop2(children, maximizing, true);
  return truncateChildrenWithCoverage(
    children,
    context.config.search.nodeBranchLimit,
    maximizing,
    true,
  );
}

function staticScore(
  game: MonsGame,
  stateHash: Hash64,
  context: SearchContext,
): number {
  const terminal = terminalSearchScore(
    game,
    context.perspective,
    0,
    context.config.budget.depth,
  );
  return (
    terminal ??
    cachedPreferability(
      context.execution,
      game,
      context.perspective,
      context.config,
      context.stats,
      stateHash,
    )
  );
}

function quiescenceScore(
  game: MonsGame,
  stateHash: Hash64,
  alpha: number,
  beta: number,
  context: SearchContext,
): number {
  const standPat = staticScore(game, stateHash, context);
  const maximizing = game.activeColor === context.perspective;
  if ((maximizing && standPat >= beta) || (!maximizing && standPat <= alpha)) {
    return standPat;
  }
  context.stats.quiescenceNodes += 1;
  let best = standPat;
  let window = maximizing ? Math.max(alpha, best) : Math.min(beta, best);
  for (const transition of enumerateLegalTransitions(
    context.execution,
    game,
    Math.min(
      context.config.search.quiescenceEnumerationLimit,
      context.config.search.nodeEnumerationLimit,
    ),
  )) {
    if (
      context.stats.visitedNodes >= context.config.budget.maxVisitedNodes ||
      context.execution.session.checkpoint()
    ) {
      break;
    }
    if (!isQuiescenceTacticalTransition(transition.events)) continue;
    context.stats.visitedNodes += 1;
    const transitionHash = exactSearchStateHash(transition.game);
    const score = cachedPreferability(
      context.execution,
      transition.game,
      context.perspective,
      context.config,
      context.stats,
      transitionHash,
    );
    best = maximizing ? Math.max(best, score) : Math.min(best, score);
    window = maximizing ? Math.max(window, best) : Math.min(window, best);
    if ((maximizing && window >= beta) || (!maximizing && window <= alpha)) {
      break;
    }
  }
  return best;
}

function boundedSearch(
  game: MonsGame,
  stateHash: Hash64,
  depth: number,
  alphaValue: number,
  betaValue: number,
  extensionsRemaining: number,
  context: SearchContext,
): number {
  const terminal = terminalSearchScore(
    game,
    context.perspective,
    depth,
    context.config.budget.depth,
  );
  if (terminal !== undefined) return terminal;
  if (context.execution.session.checkpoint()) return 0;
  if (context.stats.visitedNodes >= context.config.budget.maxVisitedNodes) {
    return staticScore(game, stateHash, context);
  }
  if (depth <= 0) {
    if (
      context.config.search.quiescence &&
      context.stats.quiescenceNodes < context.config.budget.quiescenceNodes
    ) {
      return quiescenceScore(game, stateHash, alphaValue, betaValue, context);
    }
    return staticScore(game, stateHash, context);
  }

  let alpha = alphaValue;
  let beta = betaValue;
  const alphaBefore = alpha;
  const betaBefore = beta;

  let preferredChildHash: Hash64 | undefined;
  const entry = context.config.search.transpositionTable
    ? context.transposition.get(stateHash)
    : undefined;
  if (entry !== undefined) {
    context.stats.cacheHits += 1;
    if (!hash64IsZero(entry.bestChildHash)) {
      preferredChildHash = entry.bestChildHash;
    }
    if (entry.depth >= depth) {
      if (entry.bound === "exact") return entry.score;
      if (entry.bound === "lower") alpha = Math.max(alpha, entry.score);
      else beta = Math.min(beta, entry.score);
      if (alpha >= beta) return entry.score;
    }
  }

  const maximizing = game.activeColor === context.perspective;
  if (
    context.config.search.futilityPruning &&
    depth === 1 &&
    !hasProTacticalPotential(context.execution, game)
  ) {
    const evaluation = staticScore(game, stateHash, context);
    if (
      (maximizing &&
        saturatingScoreAdd(evaluation, context.config.search.futilityMargin) <
          alpha) ||
      (!maximizing &&
        saturatingScoreSubtract(
          evaluation,
          context.config.search.futilityMargin,
        ) > beta)
    ) {
      return evaluation;
    }
  }

  const children = rankedChildren(game, context, stateHash, preferredChildHash);
  if (children.length === 0) return staticScore(game, stateHash, context);
  let value = maximizing ? -0x8000_0000 : 0x7fff_ffff;
  let bestChildHash = HASH64_ZERO;
  let stoppedByBudget = false;
  for (const child of children) {
    if (context.stats.visitedNodes >= context.config.budget.maxVisitedNodes) {
      stoppedByBudget = true;
      break;
    }
    let childDepth = Math.max(0, depth - 1);
    let childExtensions = extensionsRemaining;
    if (
      context.config.search.selectiveExtensions &&
      isSelectiveExtensionCandidate(
        child.tacticalExtensionTrigger,
        child.orderingEfficiency,
        child.classes,
      ) &&
      childExtensions > 0 &&
      (context.extensionNodeBudget === 0 ||
        context.stats.extensionNodes < context.extensionNodeBudget)
    ) {
      childDepth = depth;
      childExtensions -= 1;
      if (context.extensionNodeBudget > 0) {
        context.stats.extensionNodes += 1;
      }
    } else if (
      context.config.search.quietReductions &&
      child.quietReductionCandidate &&
      depth >= context.config.search.quietReductionDepthThreshold
    ) {
      childDepth = Math.max(0, depth - 2);
    }
    context.stats.visitedNodes += 1;
    const score = boundedSearch(
      child.game,
      child.hash,
      childDepth,
      alpha,
      beta,
      childExtensions,
      context,
    );
    if ((maximizing && score > value) || (!maximizing && score < value)) {
      value = score;
      bestChildHash = child.hash;
    }
    if (maximizing) alpha = Math.max(alpha, value);
    else beta = Math.min(beta, value);
    if (alpha >= beta || context.execution.session.checkpoint()) break;
  }
  if (value === -0x8000_0000 || value === 0x7fff_ffff) {
    value = staticScore(game, stateHash, context);
  }

  if (
    context.config.search.transpositionTable &&
    !stoppedByBudget &&
    context.execution.session.cacheWriteAllowed
  ) {
    const bound: TranspositionBound =
      value <= alphaBefore ? "upper" : value >= betaBefore ? "lower" : "exact";
    if (
      context.transposition.size >=
        context.config.search.transpositionCapacity &&
      !context.transposition.has(stateHash)
    ) {
      context.transposition.clear();
    }
    context.transposition.set(stateHash, {
      depth,
      score: value,
      bound,
      bestChildHash,
    });
  }
  return value;
}

function createSearchContext(
  execution: AutomoveExecutionContext,
  perspective: Color,
  config: SearchConfig,
): SearchContext {
  return {
    execution,
    perspective,
    config,
    transposition: new Hash64Table<TranspositionEntry>(
      config.search.transpositionCapacity,
    ),
    extensionNodeBudget: config.search.selectiveExtensions
      ? Math.max(
          1,
          Math.floor(
            (config.budget.maxVisitedNodes *
              config.budget.extensionNodeShareBp) /
              10_000,
          ),
        )
      : 0,
    stats: {
      visitedNodes: 0,
      cacheHits: 0,
      quiescenceNodes: 0,
      extensionNodes: 0,
    },
  };
}

function candidateWithCurrentStateHash(
  candidate: RootCandidate,
): RootCandidate {
  const stateHash = exactSearchStateHash(candidate.game);
  return hash64Equals(stateHash, candidate.stateHash)
    ? candidate
    : { ...candidate, stateHash };
}

function betterEvaluation(
  candidate: EvaluatedRoot,
  incumbent: EvaluatedRoot,
): boolean {
  return (
    candidate.score > incumbent.score ||
    (candidate.score === incumbent.score &&
      compareRootCandidates(candidate, incumbent) < 0)
  );
}

type FocusedEvaluationRun = {
  readonly evaluations: readonly EvaluatedRoot[];
  readonly best: EvaluatedRoot | undefined;
};

function evaluateFocusedRoots(
  candidates: readonly RootCandidate[],
  context: SearchContext,
): FocusedEvaluationRun {
  const evaluations: EvaluatedRoot[] = [];
  let best: EvaluatedRoot | undefined;
  let alpha = -0x8000_0000;
  for (const candidate of candidates) {
    if (
      context.stats.visitedNodes >= context.config.budget.maxVisitedNodes ||
      context.execution.session.checkpoint()
    ) {
      break;
    }
    context.stats.visitedNodes += 1;
    const score =
      context.config.budget.depth <= 1
        ? candidate.heuristic
        : boundedSearch(
            candidate.game,
            candidate.stateHash,
            context.config.budget.depth - 1,
            alpha,
            0x7fff_ffff,
            context.config.search.maxExtensionsPerPath,
            context,
          );
    const evaluation: EvaluatedRoot = {
      ...candidate,
      score,
      nodesAfter: context.stats.visitedNodes,
    };
    evaluations.push(evaluation);
    if (best === undefined || betterEvaluation(evaluation, best)) {
      best = evaluation;
    }
    alpha = Math.max(alpha, score);
  }
  return { evaluations, best };
}

export function evaluateSearchScore(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  depth: number,
  config: SearchConfig,
): number {
  const context = createSearchContext(execution, perspective, config);
  const stateHash = exactSearchStateHash(game);
  return boundedSearch(
    game,
    stateHash,
    Math.max(0, depth),
    -0x8000_0000,
    0x7fff_ffff,
    config.search.maxExtensionsPerPath,
    context,
  );
}

/**
 * Run only the two-pass root allocator. The scout uses the same shared
 * search context and cumulative node accounting as the full root search, but
 * this seam deliberately stops before the later scored-root loop.
 */
export function focusRootCandidatesForSearch(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: SearchConfig,
  suppliedCandidates?: readonly RootCandidate[],
  options: SearchRootOptions = {},
  useTranspositionTable = config.search.transpositionTable,
): FocusedSearchRoots {
  const sourceFen = game.fen();
  const sourceCandidates = suppliedCandidates
    ? suppliedCandidates.map(candidateWithCurrentStateHash)
    : rankRootCandidates(execution, game, perspective, config);
  let scoutContext: SearchContext | undefined;
  const focused = focusedRootCandidates({
    rootMoves: sourceCandidates,
    perspective,
    config,
    useTranspositionTable,
    ...(options.priorityInputs === undefined
      ? {}
      : { priorityInputs: options.priorityInputs }),
    ...(options.forcedInputs === undefined
      ? {}
      : { forcedInputs: options.forcedInputs }),
    ...(options.qualifiesPlainSpiritPlan === undefined
      ? {}
      : { qualifiesPlainSpiritPlan: options.qualifiesPlainSpiritPlan }),
    ...(options.qualifiesDrainerSafetyRecoveryPlan === undefined
      ? {}
      : {
          qualifiesDrainerSafetyRecoveryPlan:
            options.qualifiesDrainerSafetyRecoveryPlan,
        }),
    evaluateDeeperScout: (scout) => {
      scoutContext ??= createSearchContext(
        execution,
        perspective,
        patchAutomoveConfig(scout.config, {
          search: { transpositionTable: scout.useTranspositionTable },
        }),
      );
      scoutContext.stats.visitedNodes = Math.max(
        scoutContext.stats.visitedNodes,
        scout.visitedNodes,
      );
      const score = boundedSearch(
        scout.candidate.game,
        scout.candidate.stateHash,
        scout.depth,
        scout.alpha,
        0x7fff_ffff,
        0,
        scoutContext,
      );
      return {
        score,
        visitedNodes: scoutContext.stats.visitedNodes,
      };
    },
    checkpoint: () => execution.session.checkpoint(),
    cancelled: () => execution.session.cancelled,
  });
  if (game.fen() !== sourceFen) {
    throw new Error("root focus mutated its source game");
  }
  return {
    candidates: focused.candidates,
    scoutVisitedNodes: focused.scoutVisitedNodes,
  };
}

export function searchRootCandidates(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: SearchConfig,
  suppliedCandidates?: readonly RootCandidate[],
  options: SearchRootOptions = {},
): SearchResult {
  const sourceFen = game.fen();
  const focused = focusRootCandidatesForSearch(
    execution,
    game,
    perspective,
    config,
    suppliedCandidates,
    options,
  );
  const candidates = focused.candidates;
  const context = createSearchContext(execution, perspective, config);
  context.stats.visitedNodes = focused.scoutVisitedNodes;
  const { evaluations, best } = evaluateFocusedRoots(candidates, context);
  if (game.fen() !== sourceFen) {
    throw new Error("bounded search mutated its source game");
  }
  return {
    best,
    evaluations,
    visitedNodes: context.stats.visitedNodes,
    cacheHits: context.stats.cacheHits,
    timedOut: context.execution.session.cancelled,
  };
}

export function clearSearchCaches(execution: AutomoveExecutionContext): void {
  preferabilityCache(execution).clear();
  clearMoveEfficiencyCache(execution);
  clearExactStateAnalysisCache(execution);
}
