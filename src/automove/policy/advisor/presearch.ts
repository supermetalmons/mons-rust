import type { Input } from "../../../engine/model/domain.js";
import { Color } from "../../../api/types.js";
import { inputChainsEqual } from "../../../engine/model/domain.js";
import { MonsGame } from "../../../engine/game/mons-game.js";
import type { AutomoveExecutionContext } from "../../core/execution-context.js";
import type { RootCandidate } from "../../root/types.js";
import type { AutomoveConfig } from "../../config/types.js";
import { productionEnabled } from "../../config/types.js";
import type { TurnPlan } from "../../turn/model.js";
import { entry, productionRepresentativeSpecs, pushUnique } from "./support.js";
import type {
  ProductionAdvisorOptions,
  ProductionRootAdvisorDecision,
  ProductionRootAdvisorEntry,
} from "./types.js";
import { ProductionRootAdvisorReasonCode } from "./types.js";

import { evaluateInjectedRoot } from "./injected-root.js";
import {
  findRootMoveRepresentative,
  sameOpeningSetupRepresentative,
} from "./representatives.js";

export function productionRootAdvisorPresearch(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: AutomoveConfig,
  roots: RootCandidate[],
  engineHeadPlan?: TurnPlan,
  options: ProductionAdvisorOptions = {},
): ProductionRootAdvisorDecision | undefined {
  if (
    !productionEnabled(config) ||
    roots.length === 0 ||
    execution.session.checkpoint()
  ) {
    return undefined;
  }
  if (
    game.activeColor === Color.Black &&
    game.turnNumber === 2 &&
    game.monsMovesCount <= 1 &&
    game.playerCanUseAction() &&
    game.playerCanMoveMana()
  ) {
    return undefined;
  }
  const orderedShortlist: ProductionRootAdvisorEntry[] = [];
  const preservedFamilyRepresentatives: ProductionRootAdvisorEntry[] = [];
  const anchor = roots[0];
  if (anchor === undefined) return undefined;
  pushUnique(
    orderedShortlist,
    entry(anchor, ProductionRootAdvisorReasonCode.RankedRoot),
  );
  for (const [reason, predicate] of productionRepresentativeSpecs) {
    if (execution.session.checkpoint()) return undefined;
    const index = findRootMoveRepresentative(
      execution,
      game,
      roots,
      perspective,
      config,
      predicate,
    );
    const root = index === undefined ? undefined : roots[index];
    if (root === undefined) continue;
    const representative = entry(root, reason);
    pushUnique(preservedFamilyRepresentatives, representative);
    pushUnique(orderedShortlist, representative);
  }
  const setupIndex = sameOpeningSetupRepresentative(roots, config);
  const setup = setupIndex === undefined ? undefined : roots[setupIndex];
  if (setup !== undefined) {
    const representative = entry(
      setup,
      ProductionRootAdvisorReasonCode.PreserveSpiritRepresentative,
    );
    pushUnique(preservedFamilyRepresentatives, representative);
    pushUnique(orderedShortlist, representative);
  }
  const injectedRoot =
    engineHeadPlan === undefined
      ? undefined
      : evaluateInjectedRoot(
          execution,
          game,
          perspective,
          config,
          roots,
          engineHeadPlan,
          options,
        );
  if (injectedRoot?.admitted) {
    const injected = roots.find((root) =>
      inputChainsEqual(root.inputs, injectedRoot.inputs),
    );
    if (injected !== undefined) {
      pushUnique(
        orderedShortlist,
        entry(injected, ProductionRootAdvisorReasonCode.AdmitInjectedMacroRoot),
      );
    }
  }
  return execution.session.checkpoint()
    ? undefined
    : {
        orderedShortlist,
        preservedFamilyRepresentatives,
        approvedRoot: undefined,
        injectedRoot,
      };
}

export function productionRootAdvisorPriorityInputs(
  decision: ProductionRootAdvisorDecision,
): Input[][] {
  const result: Input[][] = [];
  if (decision.injectedRoot?.admitted) {
    result.push([...decision.injectedRoot.inputs]);
  }
  for (const representative of decision.preservedFamilyRepresentatives) {
    if (!result.some((inputs) => inputChainsEqual(inputs, representative.inputs))) {
      result.push([...representative.inputs]);
    }
  }
  return result;
}
