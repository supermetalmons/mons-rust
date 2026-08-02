import { AutomoveCacheScope } from "./cache-scope.js";
import { SearchSession } from "./deadline.js";

export {
  AutomoveCacheScope,
  type BoundedAutomoveCache,
} from "./cache-scope.js";

type AutomoveCacheScopes = {
  readonly session: AutomoveCacheScope;
  readonly engine: AutomoveCacheScope;
};

export type AutomoveRandomSource = {
  nextUint32(): number;
};

export type AutomoveExecutionContext = {
  readonly session: SearchSession;
  readonly caches: AutomoveCacheScopes;
  readonly random: AutomoveRandomSource;
};

export function createAutomoveExecutionContext(
  session: SearchSession,
  engineCaches: AutomoveCacheScope,
  random: AutomoveRandomSource,
): AutomoveExecutionContext {
  if (engineCaches.lifetime !== "engine") {
    throw new TypeError("execution context requires an engine cache scope");
  }
  return Object.freeze({
    session,
    caches: Object.freeze({
      session: session.caches,
      engine: engineCaches,
    }),
    random,
  });
}
