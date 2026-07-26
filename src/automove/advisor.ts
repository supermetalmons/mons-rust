import { productionRootAdvisorPostsearch } from "./advisor/postsearch.js";
import { buildRootPolicy } from "./advisor/root-policy-core.js";
import type { AutomoveExecutionContext } from "./execution-context.js";
import type {
  ProductionRootPicker,
  ProductionRootPolicy,
} from "./root-selector.js";
import type { AutomoveConfig } from "./selector-types.js";

export {
  ProductionRootAdvisorReasonCode,
  type ProductionAdvisorOptions,
  type ProductionInjectedRootAdvisorDecision,
  type ProductionRootAdvisorDecision,
  type ProductionRootAdvisorEntry,
  type ProductionRootAdvisorPostsearchResult,
} from "./advisor/types.js";
export {
  productionRootAdvisorPresearch,
  productionRootAdvisorPriorityInputs,
} from "./advisor/presearch.js";
export { productionRootAdvisorPostsearch };
export { rootFamily as advisorRootFamily } from "./root-family.js";
export { rootIsUnsafe as advisorRootIsUnsafe } from "./selector-types.js";

/** Builds the production policy seams consumed by `root-selector`. */
export function productionRootPolicy(
  execution: AutomoveExecutionContext,
  config: AutomoveConfig,
): ProductionRootPolicy {
  const rootPicker = Object.freeze({
    id: "root-picker.advisor-postsearch",
    select(context) {
      const index = productionRootAdvisorPostsearch(
        execution,
        context.game,
        context.roots,
        context.perspective,
        config,
      )?.index;
      return index === undefined
        ? ({ kind: "continue" } as const)
        : ({ kind: "select", index } as const);
    },
  } satisfies ProductionRootPicker);
  return Object.freeze({
    ...buildRootPolicy(execution),
    rootPicker,
  });
}
