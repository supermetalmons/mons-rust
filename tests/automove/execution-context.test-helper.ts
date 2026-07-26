import {
  SearchSession,
  type MonotonicClock,
} from "../../src/automove/deadline.js";
import {
  AutomoveCacheScope,
  createAutomoveExecutionContext,
  type AutomoveExecutionContext,
} from "../../src/automove/execution-context.js";

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
