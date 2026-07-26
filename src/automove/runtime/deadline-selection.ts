import type { Input } from "../../engine/domain.js";
import type { MonsGame } from "../../engine/game.js";
import { AUTOMOVE_SELECTOR_BUDGET_MS } from "../deadline.js";
import type { AutomoveExecutionContext } from "../execution-context.js";
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

const PRODUCTION_FAST_BANK_BUDGET_MS = 200;
const PRODUCTION_START_RESERVE_MS = 100;
const PRODUCTION_SELECTOR_BUDGET_MS = 550;

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

function selectProductionFastBankInputs(
  execution: AutomoveExecutionContext,
  game: MonsGame,
): Input[] | undefined {
  const fastConfig = automoveConfigForGame(game, "fast");
  const selected = execution.session.withCooperativeSubdeadline(
    PRODUCTION_FAST_BANK_BUDGET_MS,
    () => [...smartSearchBestInputs(execution, game, fastConfig, true)],
  );
  if (selected === undefined) clearTimedOutSelectionCaches(execution);
  return selected;
}

export function selectProductionInputsWithDeadline(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  base: AutomoveConfig,
): Input[] {
  const production = withProductionPlanner(base);
  return execution.session.withDeadlineIfAbsent(
    PRODUCTION_SELECTOR_BUDGET_MS,
    () => {
      const emergency = deterministicLegalFallbackInputs(game);
      if (execution.session.checkpoint()) return emergency;

      const fast = selectProductionFastBankInputs(execution, game) ?? [];
      const timeoutInputs =
        fast.length > 0 && !execution.session.checkpoint() ? fast : emergency;
      if (
        execution.session.checkpointWithReserve(PRODUCTION_START_RESERVE_MS)
      ) {
        return timeoutInputs;
      }

      const selected = selectProductionPolicyInputs(
        execution,
        game,
        base,
        production,
      );
      return selected.length === 0 || execution.session.checkpoint()
        ? timeoutInputs
        : selected;
    },
  );
}
