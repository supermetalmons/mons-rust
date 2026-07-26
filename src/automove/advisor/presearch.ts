import { Color, inputChainsEqual } from "../../engine/domain.js";
import type { Event, Input } from "../../engine/domain.js";
import { MonsGame } from "../../engine/game.js";
import type { AutomoveExecutionContext } from "../execution-context.js";
import { hasRoundtripMonMove, manaHandoffPenalty } from "../move-efficiency.js";
import { isOwnDrainerVulnerable } from "../root-candidates.js";
import type { RootCandidate } from "../root-candidates.js";
import { rootFamily as advisorRootFamily } from "../root-family.js";
import { saturatingScoreSubtract } from "../score-math.js";
import {
  hasConcreteScoreSurface,
  hasProgressSurface,
  isPlainSpiritDevelopmentRoot,
  productionEnabled,
  rootIsUnsafe as advisorRootIsUnsafe,
} from "../selector-types.js";
import type { AutomoveConfig } from "../selector-types.js";
import { applyInputsForSearchWithEvents } from "../transitions.js";
import {
  TurnPlanFamily,
  compareUtilityPrimaryAxes,
  utilityImprovesNonScoreOverrideAxes,
  utilityPassesOverrideGuard,
  utilityStrictlyDominatesOverrideAxes,
  utilitySupportsPrimaryAxesEvalTolerance,
} from "../turn-engine.js";
import type { TurnPlan } from "../turn-engine.js";
import {
  advisorRootIsSafe,
  compareRankedRootMoveIndices,
  compareRootMoveSearchPriority,
  entry,
  pushUnique,
  rootMoveUtility,
  sameFirstInput,
  utilityCompetes,
} from "./support.js";
import { ProductionRootAdvisorReasonCode } from "./types.js";
import type {
  ProductionAdvisorOptions,
  ProductionInjectedRootAdvisorDecision,
  ProductionRootAdvisorDecision,
  ProductionRootAdvisorEntry,
} from "./types.js";

function findRootMoveRepresentative(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  roots: readonly RootCandidate[],
  perspective: Color,
  config: AutomoveConfig,
  predicate: (root: RootCandidate) => boolean,
): number | undefined {
  const anchor = roots[0];
  if (anchor === undefined) return undefined;
  if (predicate(anchor) && advisorRootIsSafe(anchor)) return undefined;
  const anchorUtility = rootMoveUtility(
    execution,
    game,
    anchor,
    perspective,
    config,
  );
  return roots
    .map((_root, index) => index)
    .filter((index) => {
      const candidate = roots[index];
      return (
        candidate !== undefined &&
        predicate(candidate) &&
        advisorRootIsSafe(candidate) &&
        utilityCompetes(
          rootMoveUtility(execution, game, candidate, perspective, config),
          anchorUtility,
        )
      );
    })
    .sort((left, right) => compareRankedRootMoveIndices(roots, left, right))[0];
}

function sameOpeningSetupRepresentative(
  roots: readonly RootCandidate[],
  config: AutomoveConfig,
): number | undefined {
  const anchor = roots[0];
  if (
    anchor === undefined ||
    !productionEnabled(config) ||
    !isPlainSpiritDevelopmentRoot(anchor) ||
    !advisorRootIsSafe(anchor)
  ) {
    return undefined;
  }
  return roots
    .map((_root, index) => index)
    .filter((index) => {
      const root = roots[index];
      return (
        root !== undefined &&
        advisorRootFamily(root) === TurnPlanFamily.SpiritImpact &&
        !isPlainSpiritDevelopmentRoot(root) &&
        sameFirstInput(root.inputs, anchor.inputs) &&
        root.efficiency === anchor.efficiency &&
        advisorRootIsSafe(root) &&
        !root.winsImmediately &&
        !root.attacksOpponentDrainer &&
        !root.scoresSupermanaThisTurn &&
        !root.scoresOpponentManaThisTurn &&
        !root.safeSupermanaPickupNow &&
        !root.safeOpponentManaPickupNow &&
        root.sameTurnScoreWindowValue === 0
      );
    })
    .sort((left, right) => compareRankedRootMoveIndices(roots, left, right))[0];
}

function isBlackTurnSixPlainSpiritSetupPair(
  game: MonsGame,
  plain: RootCandidate,
  setup: RootCandidate,
  config: AutomoveConfig,
): boolean {
  return (
    productionEnabled(config) &&
    game.activeColor === Color.Black &&
    game.turnNumber === 6 &&
    game.monsMovesCount === 0 &&
    game.playerCanUseAction() &&
    game.playerCanMoveMana() &&
    advisorRootFamily(plain) === TurnPlanFamily.SpiritImpact &&
    advisorRootFamily(setup) === TurnPlanFamily.SpiritImpact &&
    isPlainSpiritDevelopmentRoot(plain) &&
    setup.spiritOwnManaSetupNow &&
    !setup.spiritSameTurnScoreSetupNow &&
    sameFirstInput(plain.inputs, setup.inputs) &&
    plain.ownDrainerVulnerable === setup.ownDrainerVulnerable &&
    plain.ownDrainerWalkVulnerable === setup.ownDrainerWalkVulnerable &&
    !plain.manaHandoffToOpponent &&
    !setup.manaHandoffToOpponent &&
    !plain.hasRoundtrip &&
    !setup.hasRoundtrip &&
    !hasConcreteScoreSurface(plain) &&
    !hasConcreteScoreSurface(setup) &&
    !plain.attacksOpponentDrainer &&
    !setup.attacksOpponentDrainer &&
    plain.sameTurnScoreWindowValue === 0 &&
    setup.sameTurnScoreWindowValue === 0 &&
    !plain.supermanaProgress &&
    !setup.supermanaProgress &&
    !plain.opponentManaProgress &&
    !setup.opponentManaProgress
  );
}

function projectedPlanState(
  game: MonsGame,
  plan: TurnPlan,
): { readonly game: MonsGame; readonly events: readonly Event[] } | undefined {
  let state = game.fork();
  const events: Event[] = [];
  for (const chunk of plan.compiledChunks) {
    const applied = applyInputsForSearchWithEvents(state, chunk);
    if (applied === undefined) return undefined;
    state = applied.game;
    events.push(...applied.events);
  }
  return { game: state, events };
}

function injectedCandidatePassesRootGate(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  roots: readonly RootCandidate[],
  candidate: RootCandidate,
  candidateWasEnumerated: boolean,
  perspective: Color,
  config: AutomoveConfig,
  plan: TurnPlan,
): boolean {
  const top = roots[0];
  if (
    top === undefined ||
    (top.winsImmediately && !candidate.winsImmediately)
  ) {
    return false;
  }
  const topUnsafe = advisorRootIsUnsafe(top);
  const candidateUnsafe = advisorRootIsUnsafe(candidate);
  const topUtility = rootMoveUtility(execution, game, top, perspective, config);
  const projected = projectedPlanState(game, plan);
  const projectedFinished =
    projected !== undefined &&
    (projected.game.winnerColor() !== undefined ||
      projected.game.activeColor !== perspective ||
      (!projected.game.playerCanMoveMon() &&
        !projected.game.playerCanUseAction() &&
        !projected.game.playerCanMoveMana()));
  const projectedNearCompletion =
    projected !== undefined &&
    (projectedFinished ||
      plan.compiledChunks.length >= 4 ||
      !projected.game.playerCanMoveMon() ||
      (!projected.game.playerCanUseAction() &&
        !projected.game.playerCanMoveMana()));
  const projectedHandoff =
    projected !== undefined &&
    manaHandoffPenalty(
      projected.events,
      perspective,
      Math.max(config.evaluation.rootManaHandoffPenalty, 1),
    ) > 0;
  const projectedRoundtrip =
    projected !== undefined && hasRoundtripMonMove(projected.events);
  const projectedVulnerable =
    projected !== undefined &&
    projected.game.winnerColor() !== perspective &&
    isOwnDrainerVulnerable(execution, projected.game, perspective);
  let completedPlanOverride =
    projectedNearCompletion &&
    !projectedHandoff &&
    !projectedRoundtrip &&
    !projectedVulnerable &&
    plan.compiledChunks.length > 1 &&
    plan.goalFamily !== TurnPlanFamily.ManaTempo &&
    (compareUtilityPrimaryAxes(plan.utility, topUtility) >= 0 ||
      utilityPassesOverrideGuard(plan.utility, topUtility) ||
      utilitySupportsPrimaryAxesEvalTolerance(plan.utility, topUtility, 160) ||
      (plan.goalFamily === TurnPlanFamily.ImmediateScore &&
        utilityImprovesNonScoreOverrideAxes(plan.utility, topUtility)));
  const progressHead =
    plan.headFamily === TurnPlanFamily.SafeSupermanaProgress ||
    plan.headFamily === TurnPlanFamily.SafeOpponentManaProgress;
  const progressToScore =
    progressHead &&
    plan.goalFamily === TurnPlanFamily.ImmediateScore &&
    plan.compiledChunks.length > 1;
  const regressesTopSurface =
    !candidateWasEnumerated &&
    progressToScore &&
    !candidateUnsafe &&
    roots.slice(0, 3).some((root) => {
      const surface =
        root.spiritDevelopment ||
        root.spiritSameTurnScoreSetupNow ||
        root.spiritOwnManaSetupNow ||
        root.safeSupermanaPickupNow ||
        root.safeOpponentManaPickupNow ||
        root.winsImmediately ||
        root.scoresSupermanaThisTurn ||
        root.scoresOpponentManaThisTurn;
      return (
        !advisorRootIsUnsafe(root) &&
        surface &&
        compareUtilityPrimaryAxes(
          plan.headUtility,
          rootMoveUtility(execution, game, root, perspective, config),
        ) < 0
      );
    });
  const blocksConcreteSpirit =
    !candidateWasEnumerated &&
    progressToScore &&
    roots
      .slice(0, 3)
      .some(
        (root) =>
          root.spiritSameTurnScoreSetupNow ||
          root.spiritOwnManaSetupNow ||
          root.sameTurnScoreWindowValue > 0,
      );
  const duplicatesSafeProgress =
    !candidateWasEnumerated &&
    progressToScore &&
    roots
      .slice(0, 3)
      .some((root) => !advisorRootIsUnsafe(root) && hasProgressSurface(root));
  const blocksSafeNonProgressTop =
    !candidateWasEnumerated &&
    progressToScore &&
    !topUnsafe &&
    !hasConcreteScoreSurface(top) &&
    !top.attacksOpponentDrainer &&
    !top.spiritDevelopment &&
    !top.spiritSameTurnScoreSetupNow &&
    !top.spiritOwnManaSetupNow &&
    !hasProgressSurface(top);
  const replacesProgressClusterWithWindow =
    !candidateWasEnumerated &&
    progressToScore &&
    !candidate.winsImmediately &&
    !candidate.attacksOpponentDrainer &&
    !candidate.safeSupermanaPickupNow &&
    !candidate.safeOpponentManaPickupNow &&
    !candidate.spiritDevelopment &&
    !candidate.spiritSameTurnScoreSetupNow &&
    !candidate.spiritOwnManaSetupNow &&
    candidate.sameTurnScoreWindowValue > 0 &&
    !candidate.supermanaProgress &&
    !candidate.opponentManaProgress &&
    roots
      .slice(0, 6)
      .filter(
        (root) =>
          hasProgressSurface(root) &&
          root.sameTurnScoreWindowValue === 0 &&
          !root.spiritSameTurnScoreSetupNow &&
          !root.spiritOwnManaSetupNow,
      ).length >= 3;
  const regressesPlainSpiritCluster =
    candidateWasEnumerated &&
    plan.headFamily === TurnPlanFamily.SpiritImpact &&
    plan.goalFamily === TurnPlanFamily.ImmediateScore &&
    isPlainSpiritDevelopmentRoot(candidate) &&
    !candidate.attacksOpponentDrainer &&
    !hasConcreteScoreSurface(candidate) &&
    roots.slice(0, 3).some((root) => {
      if (
        inputChainsEqual(root.inputs, candidate.inputs) ||
        !isPlainSpiritDevelopmentRoot(root) ||
        root.attacksOpponentDrainer ||
        hasConcreteScoreSurface(root)
      ) {
        return false;
      }
      return (
        root.spiritSetupGain >= candidate.spiritSetupGain &&
        root.safeSupermanaProgressSteps <=
          candidate.safeSupermanaProgressSteps &&
        root.safeOpponentManaProgressSteps <=
          candidate.safeOpponentManaProgressSteps &&
        compareUtilityPrimaryAxes(
          rootMoveUtility(execution, game, root, perspective, config),
          plan.headUtility,
        ) >= 0
      );
    });
  completedPlanOverride =
    completedPlanOverride &&
    !regressesTopSurface &&
    !blocksConcreteSpirit &&
    !duplicatesSafeProgress &&
    !blocksSafeNonProgressTop;
  const utilityOverride =
    roots.slice(0, 3).every((root) => {
      const utility = rootMoveUtility(
        execution,
        game,
        root,
        perspective,
        config,
      );
      return (
        utilityPassesOverrideGuard(plan.utility, utility) &&
        (!candidateUnsafe || advisorRootIsUnsafe(root))
      );
    }) || completedPlanOverride;
  const candidateSpiritTactical =
    candidate.spiritSameTurnScoreSetupNow ||
    candidate.sameTurnScoreWindowValue > 0 ||
    candidate.attacksOpponentDrainer ||
    hasConcreteScoreSurface(candidate);
  const allowBlackTurnSixPlainSpirit =
    plan.headFamily === TurnPlanFamily.SpiritImpact &&
    roots
      .slice(0, 4)
      .filter((root) =>
        isBlackTurnSixPlainSpiritSetupPair(game, candidate, root, config),
      ).length >= 2;
  if (replacesProgressClusterWithWindow || regressesPlainSpiritCluster) {
    return false;
  }
  if (
    plan.headFamily === TurnPlanFamily.DrainerKill &&
    !candidate.attacksOpponentDrainer
  ) {
    return false;
  }
  if (
    progressHead &&
    !hasProgressSurface(candidate) &&
    !completedPlanOverride
  ) {
    return false;
  }
  if (
    !utilityOverride &&
    plan.headFamily === TurnPlanFamily.SpiritImpact &&
    !candidateSpiritTactical &&
    !candidate.spiritOwnManaSetupNow &&
    !allowBlackTurnSixPlainSpirit
  ) {
    return false;
  }
  const topSpiritSurface = roots
    .slice(0, 3)
    .some(
      (root) =>
        root.spiritDevelopment ||
        root.spiritSameTurnScoreSetupNow ||
        root.spiritOwnManaSetupNow,
    );
  if (
    plan.headFamily === TurnPlanFamily.SpiritImpact &&
    !topSpiritSurface &&
    !candidateSpiritTactical &&
    !candidate.spiritOwnManaSetupNow &&
    !utilityOverride &&
    !allowBlackTurnSixPlainSpirit
  ) {
    return false;
  }
  return !candidateUnsafe || topUnsafe || utilityOverride;
}

function evaluateInjectedRoot(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  config: AutomoveConfig,
  roots: RootCandidate[],
  plan: TurnPlan,
  options: ProductionAdvisorOptions,
): ProductionInjectedRootAdvisorDecision | undefined {
  const firstChunk = plan.compiledChunks[0];
  if (firstChunk === undefined) return undefined;
  const candidateInputs = [...firstChunk];
  const existing = roots.find((root) =>
    inputChainsEqual(root.inputs, candidateInputs),
  );
  const candidate =
    existing ??
    options.buildInjectedRootCandidate?.(
      game,
      perspective,
      config,
      candidateInputs,
      plan,
    );
  const rejected = (): ProductionInjectedRootAdvisorDecision => ({
    inputs: candidateInputs,
    family: plan.headFamily,
    admitted: false,
    reason: ProductionRootAdvisorReasonCode.RejectInjectedMacroRoot,
  });
  if (
    candidate === undefined ||
    !injectedCandidatePassesRootGate(
      execution,
      game,
      roots,
      candidate,
      existing !== undefined,
      perspective,
      config,
      plan,
    )
  ) {
    return rejected();
  }
  const simulated = existing === undefined ? [...roots, candidate] : [...roots];
  if (existing === undefined) simulated.sort(compareRootMoveSearchPriority);
  const top = roots[0];
  const candidateIndex = simulated.findIndex((root) =>
    inputChainsEqual(root.inputs, candidateInputs),
  );
  const simulatedCandidate = simulated[candidateIndex];
  if (top === undefined || simulatedCandidate === undefined) return rejected();
  let admitted = inputChainsEqual(simulatedCandidate.inputs, top.inputs);
  if (!admitted) {
    const incumbentUtility = rootMoveUtility(
      execution,
      game,
      top,
      perspective,
      config,
    );
    const candidateUtility = rootMoveUtility(
      execution,
      game,
      simulatedCandidate,
      perspective,
      config,
    );
    const strictPrimaryWin =
      compareUtilityPrimaryAxes(plan.utility, incumbentUtility) > 0 ||
      compareUtilityPrimaryAxes(plan.headUtility, incumbentUtility) > 0 ||
      compareUtilityPrimaryAxes(candidateUtility, incumbentUtility) > 0 ||
      utilityStrictlyDominatesOverrideAxes(plan.utility, incumbentUtility) ||
      utilityStrictlyDominatesOverrideAxes(candidateUtility, incumbentUtility);
    const resolvesSurface =
      (simulatedCandidate.winsImmediately && !top.winsImmediately) ||
      (simulatedCandidate.attacksOpponentDrainer &&
        !top.attacksOpponentDrainer) ||
      ((simulatedCandidate.scoresSupermanaThisTurn ||
        simulatedCandidate.scoresOpponentManaThisTurn) &&
        !(top.scoresSupermanaThisTurn || top.scoresOpponentManaThisTurn)) ||
      (simulatedCandidate.sameTurnScoreWindowValue >
        top.sameTurnScoreWindowValue &&
        simulatedCandidate.sameTurnScoreWindowValue > 0) ||
      (!advisorRootIsUnsafe(simulatedCandidate) && advisorRootIsUnsafe(top)) ||
      (simulatedCandidate.classes.drainerSafetyRecover &&
        top.ownDrainerVulnerable &&
        !top.classes.drainerSafetyRecover) ||
      (hasProgressSurface(simulatedCandidate) &&
        !hasProgressSurface(top) &&
        !advisorRootIsUnsafe(simulatedCandidate));
    const sameOpeningFollowup =
      plan.headFamily === TurnPlanFamily.SpiritImpact &&
      plan.goalFamily === TurnPlanFamily.SpiritImpact &&
      roots
        .slice(0, 4)
        .filter((root) =>
          isBlackTurnSixPlainSpiritSetupPair(
            game,
            simulatedCandidate,
            root,
            config,
          ),
        ).length >= 2 &&
      compareUtilityPrimaryAxes(candidateUtility, incumbentUtility) >= 0 &&
      saturatingScoreSubtract(top.heuristic, simulatedCandidate.heuristic) <= 8;
    admitted = strictPrimaryWin || resolvesSurface || sameOpeningFollowup;
  }
  if (admitted && existing === undefined) {
    roots.splice(0, roots.length, ...simulated);
  }
  return {
    inputs: candidateInputs,
    family: advisorRootFamily(simulatedCandidate),
    admitted,
    reason: admitted
      ? ProductionRootAdvisorReasonCode.AdmitInjectedMacroRoot
      : ProductionRootAdvisorReasonCode.RejectInjectedMacroRoot,
  };
}

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
  const specs: readonly (readonly [
    ProductionRootAdvisorReasonCode,
    (root: RootCandidate) => boolean,
  ])[] = [
    [
      ProductionRootAdvisorReasonCode.PreserveSpiritRepresentative,
      (root) => root.spiritSameTurnScoreSetupNow || root.spiritOwnManaSetupNow,
    ],
    [
      ProductionRootAdvisorReasonCode.PreserveSpiritRepresentative,
      (root) => isPlainSpiritDevelopmentRoot(root),
    ],
    [
      ProductionRootAdvisorReasonCode.PreserveSafeProgressRepresentative,
      (root) =>
        advisorRootFamily(root) === TurnPlanFamily.SafeSupermanaProgress,
    ],
    [
      ProductionRootAdvisorReasonCode.PreserveSafeProgressRepresentative,
      (root) =>
        advisorRootFamily(root) === TurnPlanFamily.SafeOpponentManaProgress,
    ],
    [
      ProductionRootAdvisorReasonCode.PreserveManaTempoRepresentative,
      (root) => advisorRootFamily(root) === TurnPlanFamily.ManaTempo,
    ],
  ];
  for (const [reason, predicate] of specs) {
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
    if (
      !result.some((inputs) => inputChainsEqual(inputs, representative.inputs))
    ) {
      result.push([...representative.inputs]);
    }
  }
  return result;
}
