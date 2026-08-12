import { productionRootAdvisorPostsearch } from "./postsearch.js";
import { buildRootPolicy } from "./root-policy.js";
import type { AutomoveExecutionContext } from "../../core/execution-context.js";
import type {
  ProductionRootPicker,
  ProductionRootPolicy,
} from "../../root/selector-model.js";
import type { AutomoveConfig } from "../../config/types.js";

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
