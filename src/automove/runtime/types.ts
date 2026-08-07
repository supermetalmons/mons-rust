import type { Input, Output } from "../../engine/domain.js";

export type AutomoveSuggestion = {
  readonly output: Output;
  readonly inputFen: string;
};

export type ProductionGuardResult =
  | { readonly kind: "continue" }
  | { readonly kind: "select"; readonly inputs: readonly Input[] };

export const CONTINUE_PRODUCTION_GUARD = Object.freeze({
  kind: "continue",
} as const satisfies ProductionGuardResult);

export function selectProductionGuard(
  inputs: readonly Input[],
): ProductionGuardResult {
  return { kind: "select", inputs };
}
