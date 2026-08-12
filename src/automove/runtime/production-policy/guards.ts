import type { Input } from "../../../engine/model/domain.js";
import type { MonsGame } from "../../../engine/game/mons-game.js";
import type { AutomoveExecutionContext } from "../../core/execution-context.js";
import type { AutomoveConfig } from "../../config/types.js";
import { selectLateBlackFallbackInputs } from "./black-fallback.js";
import {
  selectEarlyWhiteFallbackInputs,
  selectScoreWindowTacticalFallbackInputs,
  selectUnconditionalBlackFallbackInputs,
} from "./preselection.js";
import {
  selectWhiteConfirmBaselineBetterInputs,
  selectWhiteConfirmBaselineTiebreakInputs,
} from "./white-confirmation.js";
import {
  selectWhiteEarlyBaselineFallbackInputs,
  selectWhiteNegativeDenyFallbackInputs,
  selectWhiteNonnegativeDenyFallbackInputs,
} from "./white-deny.js";
import {
  CONTINUE_PRODUCTION_GUARD,
  selectProductionGuard,
  type ProductionGuardResult,
} from "../types.js";

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

type ProductionPreselectionGuard = {
  readonly id: ProductionPreselectionGuardId;
  evaluate(
    execution: AutomoveExecutionContext,
    game: MonsGame,
    base: AutomoveConfig,
  ): ProductionGuardResult;
};

type ProductionFallbackGuard = {
  readonly id: ProductionFallbackGuardId;
  evaluate(
    execution: AutomoveExecutionContext,
    game: MonsGame,
    base: AutomoveConfig,
    productionInputs: readonly Input[],
  ): ProductionGuardResult;
};

function guardResult(inputs: readonly Input[] | undefined): ProductionGuardResult {
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
      guardResult(selectScoreWindowTacticalFallbackInputs(execution, game, base)),
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
        selectWhiteEarlyBaselineFallbackInputs(execution, game, base, productionInputs),
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
        selectWhiteNegativeDenyFallbackInputs(execution, game, base, productionInputs),
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
        selectWhiteConfirmBaselineBetterInputs(execution, game, base, productionInputs),
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
