import {
  Color,
  Modifier,
  inputChainsEqual,
  type Input,
} from "../../engine/domain.js";
import type { MonsGame } from "../../engine/game.js";
import {
  exactOpportunityContext,
  type ExactOpportunityContext,
} from "../exact.js";
import type { AutomoveExecutionContext } from "../execution-context.js";
import { replyRiskGuardShortlistIndices } from "../reply-risk.js";
import {
  isOwnDrainerVulnerable,
  isOwnDrainerWalkVulnerable,
  rankRootCandidates,
  type RootCandidate,
} from "../root-candidates.js";
import { rootFamily } from "../root-family.js";
import { filteredRootCandidateIndices } from "../root-selector.js";
import type { EvaluatedRoot } from "../search.js";
import {
  automoveConfigForGame,
  patchAutomoveConfig,
  withProductionPlanner,
} from "../selector-config.js";
import {
  focusedCandidateRankForRuntimeInputs,
  focusedScoredRootsForRuntime,
  rootSelectorOptions,
  turnEngineConfigFromAutomoveConfig,
} from "../production-selector.js";
import {
  AUTOMOVE_TURN_ENGINE_MODE,
  hasProgressSurface as rootHasProgressSurface,
  type AutomoveConfig,
} from "../selector-types.js";
import {
  TurnPlanFamily,
  turnEngineEvaluatePlanWithReplies,
  turnEngineEvaluateStateUtility,
  type TurnPlan,
  type TurnUtility,
  utilityHasNonnegativeDenyGain,
} from "../turn-engine.js";
import {
  selectSearchInputs,
  selectSearchInputsWithFreshPlanCache,
} from "./search-selection.js";
import {
  CONTINUE_PRODUCTION_GUARD,
  selectProductionGuard,
  type ProductionGuardResult,
} from "./types.js";

function ownDrainerUnsafe(
  execution: AutomoveExecutionContext,
  game: MonsGame,
): boolean {
  return (
    isOwnDrainerVulnerable(execution, game, game.activeColor) ||
    isOwnDrainerWalkVulnerable(execution, game, game.activeColor)
  );
}

function evaluateSelectedUtility(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  root: RootCandidate,
  config: AutomoveConfig,
): TurnUtility {
  const engineConfig = turnEngineConfigFromAutomoveConfig(config);
  const headUtility = turnEngineEvaluateStateUtility(
    execution,
    root.game,
    game,
    game.activeColor,
    engineConfig,
  );
  const family = rootFamily(root);
  const plan: TurnPlan = {
    actions: [],
    compiledChunks: [root.inputs],
    endGame: root.game.fork(),
    utility: headUtility,
    headUtility,
    headFamily: family,
    goalFamily: family,
    packageMeta: {
      scoreGain: 0,
      denyGain: 0,
      drainerSafetyDelta: 0,
      spiritOnlySetup: false,
      endsNonnegativeDrainerSafety: true,
      opponentImmediateWindowAfter: 0,
    },
  };
  return turnEngineEvaluatePlanWithReplies(
    execution,
    game,
    plan,
    game.activeColor,
    engineConfig,
  );
}

function selectEarlyWhiteFallbackInputs(
  execution: AutomoveExecutionContext,
  game: MonsGame,
): Input[] | undefined {
  const broadFallback =
    (game.activeColor === Color.White &&
      game.turnNumber <= 3 &&
      !game.playerCanUseAction() &&
      !game.playerCanMoveMana() &&
      (game.monsMovesCount === 0 || game.monsMovesCount === 3)) ||
    (game.activeColor === Color.White &&
      game.turnNumber === 1 &&
      game.monsMovesCount === 2 &&
      !game.playerCanUseAction() &&
      !game.playerCanMoveMana()) ||
    (game.activeColor === Color.White &&
      game.turnNumber === 3 &&
      game.monsMovesCount === 0 &&
      game.playerCanUseAction() &&
      game.playerCanMoveMana()) ||
    (game.activeColor === Color.White &&
      game.turnNumber === 3 &&
      game.monsMovesCount >= 3 &&
      game.playerCanUseAction() &&
      game.playerCanMoveMana());
  if (broadFallback) {
    return selectSearchInputs(
      execution,
      game,
      automoveConfigForGame(game, "pro"),
    );
  }

  const manaOnly =
    game.activeColor === Color.White &&
    game.turnNumber === 3 &&
    game.monsMovesCount === 1 &&
    !game.playerCanUseAction() &&
    game.playerCanMoveMana();
  const midTurn =
    game.activeColor === Color.White &&
    game.turnNumber === 3 &&
    game.monsMovesCount > 0 &&
    !manaOnly &&
    (game.playerCanUseAction() || game.playerCanMoveMana());
  if (!midTurn || !ownDrainerUnsafe(execution, game)) return undefined;
  return selectSearchInputs(
    execution,
    game,
    automoveConfigForGame(game, "fast"),
  );
}

function selectScoreWindowTacticalFallbackInputs(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  base: AutomoveConfig,
): Input[] | undefined {
  const eligible =
    game.activeColor === Color.White &&
    game.turnNumber === 3 &&
    (game.monsMovesCount === 1 || game.monsMovesCount === 2) &&
    game.playerCanUseAction() &&
    game.playerCanMoveMana();
  if (!eligible) return undefined;
  const context = exactOpportunityContext(execution, game, game.activeColor);
  if (context.delta.sameTurnScoreWindowValue <= 0) return undefined;
  return selectSearchInputsWithFreshPlanCache(
    execution,
    game,
    withProductionPlanner(base),
  );
}

function findRoot(
  roots: readonly RootCandidate[],
  inputs: readonly Input[],
): RootCandidate | undefined {
  return roots.find((root) => inputChainsEqual(root.inputs, inputs));
}

function whiteDenyFallbackContextEligible(
  context: ExactOpportunityContext,
): boolean {
  return (
    !context.opponentCanWinImmediately &&
    context.delta.sameTurnScoreWindowValue === 1 &&
    context.delta.opponentWindowDenyGain === 1 &&
    !context.delta.drainerAttackAvailable &&
    context.delta.drainerSafety < 0
  );
}

function selectWhiteEarlyBaselineFallbackInputs(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  base: AutomoveConfig,
  productionInputs: readonly Input[],
): Input[] | undefined {
  const eligible =
    game.activeColor === Color.White &&
    game.turnNumber === 5 &&
    game.monsMovesCount === 0 &&
    game.playerCanUseAction() &&
    game.playerCanMoveMana() &&
    productionInputs.length > 0;
  if (!eligible) return undefined;
  const context = exactOpportunityContext(execution, game, game.activeColor);
  if (!whiteDenyFallbackContextEligible(context)) return undefined;

  const production = withProductionPlanner(base);
  const selected = findRoot(
    rankRootCandidates(execution, game, game.activeColor, production),
    productionInputs,
  );
  if (
    selected === undefined ||
    selected.winsImmediately ||
    selected.attacksOpponentDrainer ||
    selected.spiritDevelopment ||
    selected.spiritSameTurnScoreSetupNow ||
    selected.spiritOwnManaSetupNow ||
    selected.scoresSupermanaThisTurn ||
    selected.scoresOpponentManaThisTurn ||
    selected.safeSupermanaPickupNow ||
    selected.safeOpponentManaPickupNow ||
    selected.supermanaProgress ||
    selected.opponentManaProgress ||
    !selected.ownDrainerVulnerable ||
    selected.ownDrainerWalkVulnerable ||
    selected.manaHandoffToOpponent ||
    selected.hasRoundtrip ||
    selected.sameTurnScoreWindowValue !== 1
  ) {
    return undefined;
  }

  const fallbackConfig = automoveConfigForGame(game, "pro");
  const inputs = selectSearchInputs(execution, game, fallbackConfig);
  if (inputs.length === 0 || inputChainsEqual(inputs, productionInputs)) {
    return undefined;
  }
  const fallback = findRoot(
    rankRootCandidates(execution, game, game.activeColor, fallbackConfig),
    inputs,
  );
  if (
    fallback === undefined ||
    !fallback.spiritDevelopment ||
    fallback.spiritSameTurnScoreSetupNow ||
    !rootHasProgressSurface(fallback) ||
    fallback.winsImmediately ||
    fallback.attacksOpponentDrainer ||
    fallback.scoresSupermanaThisTurn ||
    fallback.scoresOpponentManaThisTurn ||
    fallback.safeSupermanaPickupNow ||
    fallback.safeOpponentManaPickupNow ||
    fallback.manaHandoffToOpponent ||
    fallback.hasRoundtrip ||
    !fallback.ownDrainerVulnerable ||
    fallback.ownDrainerWalkVulnerable ||
    fallback.sameTurnScoreWindowValue !== 0
  ) {
    return undefined;
  }
  return inputs;
}

function selectWhiteNonnegativeDenyFallbackInputs(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  base: AutomoveConfig,
  productionInputs: readonly Input[],
): Input[] | undefined {
  const eligible =
    game.activeColor === Color.White &&
    game.turnNumber === 3 &&
    game.monsMovesCount === 1 &&
    !game.playerCanUseAction() &&
    game.playerCanMoveMana() &&
    productionInputs.length > 0;
  if (!eligible) return undefined;
  const context = exactOpportunityContext(execution, game, game.activeColor);
  if (!whiteDenyFallbackContextEligible(context)) return undefined;
  const production = withProductionPlanner(base);
  const selected = findRoot(
    rankRootCandidates(execution, game, game.activeColor, production),
    productionInputs,
  );
  if (
    selected === undefined ||
    !utilityHasNonnegativeDenyGain(
      evaluateSelectedUtility(execution, game, selected, production),
    )
  ) {
    return undefined;
  }
  const searchOnly = patchAutomoveConfig(production, {
    planner: {
      enabled: false,
      rerankHeads: true,
      mode: AUTOMOVE_TURN_ENGINE_MODE.Baseline,
    },
  });
  const inputs = selectSearchInputs(execution, game, searchOnly);
  return inputs.length === 0 || inputChainsEqual(inputs, productionInputs)
    ? undefined
    : inputs;
}

function selectWhiteNegativeDenyFallbackInputs(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  base: AutomoveConfig,
  productionInputs: readonly Input[],
): Input[] | undefined {
  const eligible =
    game.activeColor === Color.White &&
    game.turnNumber === 3 &&
    game.monsMovesCount === 1 &&
    !game.playerCanUseAction() &&
    game.playerCanMoveMana() &&
    productionInputs.length > 0;
  if (!eligible) return undefined;
  const context = exactOpportunityContext(execution, game, game.activeColor);
  if (!whiteDenyFallbackContextEligible(context)) return undefined;
  const production = withProductionPlanner(base);
  const selected = findRoot(
    rankRootCandidates(execution, game, game.activeColor, production),
    productionInputs,
  );
  if (
    selected === undefined ||
    utilityHasNonnegativeDenyGain(
      evaluateSelectedUtility(execution, game, selected, production),
    )
  ) {
    return undefined;
  }
  const productionConfig = automoveConfigForGame(game, "pro");
  const searchOnly = patchAutomoveConfig(production, {
    planner: {
      enabled: false,
      rerankHeads: true,
      ownSeedCap: productionConfig.planner.ownSeedCap,
      ownBeam: productionConfig.planner.ownBeam,
      perNodeFamilyCap: productionConfig.planner.perNodeFamilyCap,
      stepCap: productionConfig.planner.stepCap,
    },
  });
  const inputs = selectSearchInputs(execution, game, searchOnly);
  if (inputs.length === 0 || inputChainsEqual(inputs, productionInputs)) {
    return undefined;
  }
  return focusedCandidateRankForRuntimeInputs(
    execution,
    game,
    searchOnly,
    inputs,
  ) === 0
    ? inputs
    : undefined;
}

function safeQuietManaTempoRoot(root: EvaluatedRoot): boolean {
  return (
    !root.winsImmediately &&
    !root.attacksOpponentDrainer &&
    !root.ownDrainerVulnerable &&
    !root.ownDrainerWalkVulnerable &&
    !root.spiritDevelopment &&
    !root.spiritSameTurnScoreSetupNow &&
    !root.spiritOwnManaSetupNow &&
    !root.manaHandoffToOpponent &&
    !root.hasRoundtrip &&
    !root.scoresSupermanaThisTurn &&
    !root.scoresOpponentManaThisTurn &&
    !root.safeSupermanaPickupNow &&
    !root.safeOpponentManaPickupNow &&
    root.sameTurnScoreWindowValue === 0 &&
    !root.supermanaProgress &&
    !root.opponentManaProgress &&
    !root.classes.immediateScore &&
    !root.classes.drainerAttack &&
    !root.classes.drainerSafetyRecover &&
    !root.classes.carrierProgress &&
    !root.classes.material &&
    root.classes.quiet &&
    rootFamily(root) === TurnPlanFamily.ManaTempo
  );
}

function confirmBaselineContextEligible(
  context: ExactOpportunityContext,
): boolean {
  return (
    !context.opponentCanWinImmediately &&
    context.delta.sameTurnScoreWindowValue === 0 &&
    context.delta.spiritGain === 0 &&
    context.delta.opponentWindowDenyGain === 0 &&
    !context.delta.drainerAttackAvailable &&
    context.delta.safeSupermanaProgressSteps === undefined &&
    context.delta.safeOpponentManaProgressSteps === undefined &&
    context.delta.drainerSafety >= 0
  );
}

function productionRuntimeCompetition(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  base: AutomoveConfig,
  productionInputs: readonly Input[],
):
  | {
      readonly config: AutomoveConfig;
      readonly roots: readonly EvaluatedRoot[];
      readonly productionIndex: number;
      readonly candidateIndices: readonly number[];
      readonly shortlist: readonly number[];
    }
  | undefined {
  const config = withProductionPlanner(base);
  const roots = focusedScoredRootsForRuntime(execution, game, config, true);
  const productionIndex = roots.findIndex((root) =>
    inputChainsEqual(root.inputs, productionInputs),
  );
  if (productionIndex < 0) return undefined;
  const candidateIndices = filteredRootCandidateIndices(
    game,
    roots,
    game.activeColor,
    config,
    rootSelectorOptions(execution, config),
  );
  if (!candidateIndices.includes(productionIndex)) return undefined;
  const shortlist = replyRiskGuardShortlistIndices(
    execution,
    roots,
    candidateIndices,
    config,
  );
  if (!shortlist.includes(productionIndex)) return undefined;
  return {
    config,
    roots,
    productionIndex,
    candidateIndices,
    shortlist,
  };
}

function searchOnlyBaselineConfig(production: AutomoveConfig): AutomoveConfig {
  return patchAutomoveConfig(production, {
    planner: {
      enabled: false,
      rerankHeads: true,
      mode: AUTOMOVE_TURN_ENGINE_MODE.Baseline,
    },
  });
}

function selectWhiteConfirmBaselineTiebreakInputs(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  base: AutomoveConfig,
  productionInputs: readonly Input[],
): Input[] | undefined {
  const eligible =
    game.activeColor === Color.White &&
    game.turnNumber === 3 &&
    game.monsMovesCount === 2 &&
    !game.playerCanUseAction() &&
    game.playerCanMoveMana() &&
    productionInputs.length > 0;
  if (
    !eligible ||
    !confirmBaselineContextEligible(
      exactOpportunityContext(execution, game, game.activeColor),
    )
  ) {
    return undefined;
  }
  const competition = productionRuntimeCompetition(
    execution,
    game,
    base,
    productionInputs,
  );
  if (
    competition?.candidateIndices.length !== 2 ||
    competition.shortlist.length !== competition.candidateIndices.length
  ) {
    return undefined;
  }
  const inputs = selectSearchInputs(
    execution,
    game,
    searchOnlyBaselineConfig(competition.config),
  );
  if (inputs.length === 0 || inputChainsEqual(inputs, productionInputs)) {
    return undefined;
  }
  const searchIndex = competition.roots.findIndex((root) =>
    inputChainsEqual(root.inputs, inputs),
  );
  if (
    !competition.candidateIndices.includes(searchIndex) ||
    !competition.shortlist.includes(searchIndex)
  ) {
    return undefined;
  }
  const production = competition.roots[competition.productionIndex];
  const search = competition.roots[searchIndex];
  if (production === undefined || search === undefined) return undefined;
  return production.score === search.score &&
    production.spiritSetupGain === search.spiritSetupGain &&
    production.safeSupermanaProgressSteps ===
      search.safeSupermanaProgressSteps &&
    production.safeOpponentManaProgressSteps ===
      search.safeOpponentManaProgressSteps &&
    production.scorePathBestSteps === search.scorePathBestSteps &&
    safeQuietManaTempoRoot(production) &&
    safeQuietManaTempoRoot(search)
    ? inputs
    : undefined;
}

function selectWhiteConfirmBaselineBetterInputs(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  base: AutomoveConfig,
  productionInputs: readonly Input[],
): Input[] | undefined {
  const eligible =
    game.activeColor === Color.White &&
    game.turnNumber === 3 &&
    game.monsMovesCount >= 3 &&
    !game.playerCanUseAction() &&
    game.playerCanMoveMana() &&
    productionInputs.length > 0;
  if (
    !eligible ||
    !confirmBaselineContextEligible(
      exactOpportunityContext(execution, game, game.activeColor),
    )
  ) {
    return undefined;
  }
  const competition = productionRuntimeCompetition(
    execution,
    game,
    base,
    productionInputs,
  );
  if (competition === undefined) return undefined;
  const inputs = selectSearchInputs(
    execution,
    game,
    searchOnlyBaselineConfig(competition.config),
  );
  if (inputs.length === 0 || inputChainsEqual(inputs, productionInputs)) {
    return undefined;
  }
  const searchIndex = competition.roots.findIndex((root) =>
    inputChainsEqual(root.inputs, inputs),
  );
  if (
    !competition.candidateIndices.includes(searchIndex) ||
    !competition.shortlist.includes(searchIndex)
  ) {
    return undefined;
  }
  const production = competition.roots[competition.productionIndex];
  const search = competition.roots[searchIndex];
  if (production === undefined || search === undefined) return undefined;
  return search.score >= production.score &&
    search.rootRank < production.rootRank &&
    production.spiritSetupGain === search.spiritSetupGain &&
    production.safeSupermanaProgressSteps ===
      search.safeSupermanaProgressSteps &&
    production.safeOpponentManaProgressSteps ===
      search.safeOpponentManaProgressSteps &&
    production.scorePathBestSteps === search.scorePathBestSteps &&
    safeQuietManaTempoRoot(production) &&
    safeQuietManaTempoRoot(search)
    ? inputs
    : undefined;
}

function selectUnconditionalBlackFallbackInputs(
  execution: AutomoveExecutionContext,
  game: MonsGame,
): Input[] | undefined {
  const eligible =
    (game.activeColor === Color.Black &&
      game.turnNumber === 2 &&
      game.monsMovesCount === 0 &&
      game.playerCanUseAction() &&
      game.playerCanMoveMana()) ||
    (game.activeColor === Color.Black &&
      game.turnNumber === 2 &&
      game.monsMovesCount > 0 &&
      !game.playerCanUseAction() &&
      game.playerCanMoveMana()) ||
    (game.activeColor === Color.Black &&
      game.turnNumber === 4 &&
      game.monsMovesCount === 0 &&
      game.playerCanUseAction() &&
      game.playerCanMoveMana());
  return eligible
    ? selectSearchInputs(execution, game, automoveConfigForGame(game, "pro"))
    : undefined;
}

function selectLateBlackFallbackInputs(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  productionInputs: readonly Input[],
): Input[] | undefined {
  if (productionInputs.length === 0) return undefined;
  const transitionTurn =
    game.activeColor === Color.Black &&
    game.turnNumber === 4 &&
    game.monsMovesCount === 2 &&
    game.playerCanUseAction() &&
    game.playerCanMoveMana();
  const midTurn =
    game.activeColor === Color.Black &&
    game.turnNumber >= 4 &&
    game.monsMovesCount >= 3 &&
    game.playerCanUseAction() &&
    game.playerCanMoveMana();
  if (!transitionTurn && !midTurn) return undefined;
  const inputs = selectSearchInputs(
    execution,
    game,
    automoveConfigForGame(game, "pro"),
  );
  if (inputs.length === 0 || inputChainsEqual(inputs, productionInputs)) {
    return undefined;
  }
  if (
    transitionTurn &&
    inputs.length === 3 &&
    inputs[2]?.kind === "modifier" &&
    inputs[2].modifier === Modifier.SelectBomb
  ) {
    return inputs;
  }
  return midTurn ? inputs : undefined;
}

type ProductionPreselectionGuardId =
  | "early-white-fallback"
  | "score-window-tactical-fallback"
  | "unconditional-black-fallback";

type ProductionFallbackGuardId =
  | "white-early-baseline-fallback"
  | "white-nonnegative-deny-fallback"
  | "white-negative-deny-fallback"
  | "white-confirm-baseline-tiebreak"
  | "white-confirm-baseline-better"
  | "late-black-fallback";

export type ProductionPreselectionGuard = {
  readonly id: ProductionPreselectionGuardId;
  evaluate(
    execution: AutomoveExecutionContext,
    game: MonsGame,
    base: AutomoveConfig,
  ): ProductionGuardResult;
};

export type ProductionFallbackGuard = {
  readonly id: ProductionFallbackGuardId;
  evaluate(
    execution: AutomoveExecutionContext,
    game: MonsGame,
    base: AutomoveConfig,
    productionInputs: readonly Input[],
  ): ProductionGuardResult;
};

function guardResult(
  inputs: readonly Input[] | undefined,
): ProductionGuardResult {
  return inputs === undefined
    ? CONTINUE_PRODUCTION_GUARD
    : selectProductionGuard(inputs);
}

export const PRODUCTION_PRESELECTION_GUARDS = Object.freeze([
  Object.freeze({
    id: "early-white-fallback",
    evaluate: (
      execution: AutomoveExecutionContext,
      game: MonsGame,
    ): ProductionGuardResult =>
      guardResult(selectEarlyWhiteFallbackInputs(execution, game)),
  }),
  Object.freeze({
    id: "score-window-tactical-fallback",
    evaluate: (
      execution: AutomoveExecutionContext,
      game: MonsGame,
      base: AutomoveConfig,
    ): ProductionGuardResult =>
      guardResult(
        selectScoreWindowTacticalFallbackInputs(execution, game, base),
      ),
  }),
  Object.freeze({
    id: "unconditional-black-fallback",
    evaluate: (
      execution: AutomoveExecutionContext,
      game: MonsGame,
    ): ProductionGuardResult =>
      guardResult(selectUnconditionalBlackFallbackInputs(execution, game)),
  }),
] as const satisfies readonly ProductionPreselectionGuard[]);

export const PRODUCTION_FALLBACK_GUARDS = Object.freeze([
  Object.freeze({
    id: "white-early-baseline-fallback",
    evaluate: (
      execution: AutomoveExecutionContext,
      game: MonsGame,
      base: AutomoveConfig,
      productionInputs: readonly Input[],
    ): ProductionGuardResult =>
      guardResult(
        selectWhiteEarlyBaselineFallbackInputs(
          execution,
          game,
          base,
          productionInputs,
        ),
      ),
  }),
  Object.freeze({
    id: "white-nonnegative-deny-fallback",
    evaluate: (
      execution: AutomoveExecutionContext,
      game: MonsGame,
      base: AutomoveConfig,
      productionInputs: readonly Input[],
    ): ProductionGuardResult =>
      guardResult(
        selectWhiteNonnegativeDenyFallbackInputs(
          execution,
          game,
          base,
          productionInputs,
        ),
      ),
  }),
  Object.freeze({
    id: "white-negative-deny-fallback",
    evaluate: (
      execution: AutomoveExecutionContext,
      game: MonsGame,
      base: AutomoveConfig,
      productionInputs: readonly Input[],
    ): ProductionGuardResult =>
      guardResult(
        selectWhiteNegativeDenyFallbackInputs(
          execution,
          game,
          base,
          productionInputs,
        ),
      ),
  }),
  Object.freeze({
    id: "white-confirm-baseline-tiebreak",
    evaluate: (
      execution: AutomoveExecutionContext,
      game: MonsGame,
      base: AutomoveConfig,
      productionInputs: readonly Input[],
    ): ProductionGuardResult =>
      guardResult(
        selectWhiteConfirmBaselineTiebreakInputs(
          execution,
          game,
          base,
          productionInputs,
        ),
      ),
  }),
  Object.freeze({
    id: "white-confirm-baseline-better",
    evaluate: (
      execution: AutomoveExecutionContext,
      game: MonsGame,
      base: AutomoveConfig,
      productionInputs: readonly Input[],
    ): ProductionGuardResult =>
      guardResult(
        selectWhiteConfirmBaselineBetterInputs(
          execution,
          game,
          base,
          productionInputs,
        ),
      ),
  }),
  Object.freeze({
    id: "late-black-fallback",
    evaluate: (
      execution: AutomoveExecutionContext,
      game: MonsGame,
      base: AutomoveConfig,
      productionInputs: readonly Input[],
    ): ProductionGuardResult => {
      void base;
      return guardResult(
        selectLateBlackFallbackInputs(execution, game, productionInputs),
      );
    },
  }),
] as const satisfies readonly ProductionFallbackGuard[]);

export function selectProductionPolicyInputs(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  base: AutomoveConfig,
  production: AutomoveConfig,
): Input[] {
  for (const guard of PRODUCTION_PRESELECTION_GUARDS) {
    if (execution.session.checkpoint()) return [];
    const result = guard.evaluate(execution, game, base);
    if (result.kind === "select") return [...result.inputs];
  }

  const productionInputs = selectSearchInputsWithFreshPlanCache(
    execution,
    game,
    production,
  );
  if (execution.session.checkpoint()) return [];

  for (const guard of PRODUCTION_FALLBACK_GUARDS) {
    if (execution.session.checkpoint()) return [];
    const result = guard.evaluate(execution, game, base, productionInputs);
    if (result.kind === "select") return [...result.inputs];
  }
  return productionInputs;
}
