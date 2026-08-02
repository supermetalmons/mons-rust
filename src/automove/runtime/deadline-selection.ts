import type { Input } from "../../engine/domain.js";
import type { MonsGame } from "../../engine/game.js";
import { AUTOMOVE_SELECTOR_BUDGET_MS } from "../deadline.js";
import type { AutomoveExecutionContext } from "../execution-context.js";
import { selectProFastSelection } from "../fast/index.js";
import { smartSearchBestInputs } from "../production-selector.js";
import {
  automoveConfigForGame,
  withProductionPlanner,
} from "../selector-config.js";
import type { AutomoveConfig } from "../selector-types.js";
import { deterministicLegalFallbackInputs } from "./input-selection.js";
import { selectProductionPolicyInputs } from "./production-policy.js";
import {
  clearTimedOutSelectionCaches,
  selectSearchInputs,
} from "./search-selection.js";

const FAST_PREFERENCE_BANK_BUDGET_MS = 200;
const PRODUCTION_START_RESERVE_MS = 100;
export const PRODUCTION_SELECTOR_BUDGET_MS = 550;

export function selectStrategicSearchInputsWithDeadline(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  config: AutomoveConfig,
): Input[] {
  return execution.session.withDeadlineIfAbsent(
    AUTOMOVE_SELECTOR_BUDGET_MS,
    () => {
      const fallback = deterministicLegalFallbackInputs(game);
      if (execution.session.checkpoint()) return fallback;
      const selected = selectSearchInputs(execution, game, config);
      return selected.length === 0 || execution.session.checkpoint()
        ? fallback
        : selected;
    },
  );
}

function selectFastPreferenceBankInputs(
  execution: AutomoveExecutionContext,
  game: MonsGame,
): Input[] | undefined {
  const fastPreferenceConfig = automoveConfigForGame(game, "fast");
  const selected = execution.session.withCooperativeSubdeadline(
    FAST_PREFERENCE_BANK_BUDGET_MS,
    () => [
      ...smartSearchBestInputs(execution, game, fastPreferenceConfig, true),
    ],
  );
  if (selected === undefined) clearTimedOutSelectionCaches(execution);
  return selected;
}

function selectCanonicalProductionInputs(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  packedProFallbackInputs: Input[],
): Input[] {
  let timeoutInputs =
    packedProFallbackInputs.length > 0
      ? packedProFallbackInputs
      : deterministicLegalFallbackInputs(game);
  if (execution.session.checkpoint()) return timeoutInputs;

  const fastBankInputs = selectFastPreferenceBankInputs(execution, game);
  if (fastBankInputs !== undefined && fastBankInputs.length > 0) {
    if (execution.session.checkpoint()) return timeoutInputs;
    timeoutInputs = fastBankInputs;
  }
  if (execution.session.checkpointWithReserve(PRODUCTION_START_RESERVE_MS)) {
    return timeoutInputs;
  }

  const base = automoveConfigForGame(game, "pro");
  const production = withProductionPlanner(base);
  const selected = selectProductionPolicyInputs(
    execution,
    game,
    base,
    production,
  );
  return selected.length === 0 || execution.session.checkpoint()
    ? timeoutInputs
    : selected;
}

export function selectProductionInputsWithDeadline(
  execution: AutomoveExecutionContext,
  game: MonsGame,
): Input[] {
  return execution.session.withDeadlineIfAbsent(
    PRODUCTION_SELECTOR_BUDGET_MS,
    () => {
      const packedProSelection = selectProFastSelection(execution, game);
      return packedProSelection.kind === "supported"
        ? packedProSelection.inputs
        : selectCanonicalProductionInputs(
            execution,
            game,
            packedProSelection.fallbackInputs,
          );
    },
  );
}
