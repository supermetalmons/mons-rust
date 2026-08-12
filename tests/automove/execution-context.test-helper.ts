import {
  SearchSession,
  type MonotonicClock,
} from "../../src/automove/core/deadline.js";
import { AutomoveCacheScope } from "../../src/automove/core/cache-scope.js";
import {
  createAutomoveExecutionContext,
  type AutomoveExecutionContext,
} from "../../src/automove/core/execution-context.js";

const TEST_RANDOM_SOURCE = Object.freeze({
  nextUint32: () => 0,
});

export function createTestAutomoveExecutionContext(
  clock: MonotonicClock = () => 0,
): AutomoveExecutionContext {
  return createAutomoveExecutionContext(
    new SearchSession({ clock }),
    new AutomoveCacheScope("engine"),
    TEST_RANDOM_SOURCE,
  );
}
