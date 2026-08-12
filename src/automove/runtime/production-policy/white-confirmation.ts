import { Color } from "../../../api/types.js";
import { inputChainsEqual, type Input } from "../../../engine/model/domain.js";
import type { MonsGame } from "../../../engine/game/mons-game.js";
import { exactOpportunityContext } from "../../exact/turn-opportunity.js";
import type { AutomoveExecutionContext } from "../../core/execution-context.js";
import type { AutomoveConfig } from "../../config/types.js";
import { selectSearchInputs } from "../search-selection.js";
import {
  confirmBaselineContextEligible,
  productionRuntimeCompetition,
  safeQuietManaTempoRoot,
  searchOnlyBaselineConfig,
} from "./support.js";

export function selectWhiteConfirmBaselineTiebreakInputs(
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
    production.safeSupermanaProgressSteps === search.safeSupermanaProgressSteps &&
    production.safeOpponentManaProgressSteps === search.safeOpponentManaProgressSteps &&
    production.scorePathBestSteps === search.scorePathBestSteps &&
    safeQuietManaTempoRoot(production) &&
    safeQuietManaTempoRoot(search)
    ? inputs
    : undefined;
}

export function selectWhiteConfirmBaselineBetterInputs(
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
    production.safeSupermanaProgressSteps === search.safeSupermanaProgressSteps &&
    production.safeOpponentManaProgressSteps === search.safeOpponentManaProgressSteps &&
    production.scorePathBestSteps === search.scorePathBestSteps &&
    safeQuietManaTempoRoot(production) &&
    safeQuietManaTempoRoot(search)
    ? inputs
    : undefined;
}
