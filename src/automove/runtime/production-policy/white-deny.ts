import { Color } from "../../../api/types.js";
import { inputChainsEqual, type Input } from "../../../engine/model/domain.js";
import type { MonsGame } from "../../../engine/game/mons-game.js";
import { exactOpportunityContext } from "../../exact/turn-opportunity.js";
import type { AutomoveExecutionContext } from "../../core/execution-context.js";
import { rankRootCandidates } from "../../root/candidates.js";
import { automoveConfigForGame, withProductionPlanner } from "../../config/runtime.js";
import { patchAutomoveConfig } from "../../config/patch.js";
import {
  AUTOMOVE_TURN_ENGINE_MODE,
  hasProgressSurface as rootHasProgressSurface,
  type AutomoveConfig,
} from "../../config/types.js";
import { utilityHasNonnegativeDenyGain } from "../../turn/ordering.js";
import { focusedCandidateRankForRuntimeInputs } from "../../policy/production/search-integration.js";
import { selectSearchInputs } from "../search-selection.js";
import {
  evaluateSelectedUtility,
  findRoot,
  whiteDenyFallbackContextEligible,
} from "./support.js";

export function selectWhiteEarlyBaselineFallbackInputs(
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

export function selectWhiteNonnegativeDenyFallbackInputs(
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

export function selectWhiteNegativeDenyFallbackInputs(
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
  return focusedCandidateRankForRuntimeInputs(execution, game, searchOnly, inputs) === 0
    ? inputs
    : undefined;
}
