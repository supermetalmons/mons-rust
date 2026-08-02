import { productionRootAdvisorPostsearch } from "./advisor/postsearch.js";
import { buildRootPolicy } from "./advisor/root-policy-core.js";
import type { AutomoveExecutionContext } from "./execution-context.js";
import type {
  ProductionRootPicker,
  ProductionRootPolicy,
} from "./root-selector.js";
import type { AutomoveConfig } from "./selector-types.js";

export {
  productionRootAdvisorPresearch,
  productionRootAdvisorPriorityInputs,
} from "./advisor/presearch.js";

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
