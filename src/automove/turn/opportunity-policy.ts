import type { TurnPlanFamily } from "./model.js";

export function familyAllowed(
  allowedFamilies: readonly TurnPlanFamily[] | undefined,
  family: TurnPlanFamily,
): boolean {
  return allowedFamilies === undefined || allowedFamilies.includes(family);
}
