import { isPlainSpiritDevelopmentRoot } from "../../config/types.js";
import {
  productionIsEarlyWhiteTurnStart,
  productionSecondaryAnalysisLive,
  turnEngineModeFromAutomoveConfig as modeFromConfig,
} from "../../turn/config.js";
import { TurnEngineMode, TurnPlanFamily } from "../../turn/model.js";
import {
  compareTurnUtilities,
  compareUtilityPrimaryAxes,
  utilityHasScoreDeltaForce,
  utilityPassesOverrideGuard,
  utilityStrictlyDominatesOverrideAxes,
  utilitySupportsFamilyFallback,
  utilitySupportsPrimaryAxesEvalTolerance,
} from "../../turn/ordering.js";
import type {
  TurnEngineHeadAcceptanceContext,
  TurnEngineHeadFamilyPolicyContext,
} from "./head-types.js";
import { hasPickupUpgrade } from "./shared.js";

export function acceptTurnEngineHeadByModeAndFamily(
  context: TurnEngineHeadAcceptanceContext,
  policy: TurnEngineHeadFamilyPolicyContext,
): boolean {
  const {
    game,
    config,
    plan,
    candidateIndex,
    candidate,
    selected,
    candidateUnsafe,
    selectedUnsafe,
    candidateProgress,
    selectedProgress,
    scoreGap,
    sameTurnWindowBetter,
    drainerAttackBetter,
    scoresNowBetter,
    safetyRecoverBetter,
    spiritWindowBetter,
    spiritDevelopmentBetter,
    progressBetter,
    selectedSpiritPhase,
    selectedFamily,
    blackSpiritPair,
    blackTurnSixRouteChangePlainSpirit,
  } = context;
  const {
    selectedUtility,
    pickupUpgrade,
    strategicAxesBetter,
    projectedDeferredRecoveryWithoutConcreteGain,
    safeRootBlocksPlainSpirit,
    safeRootBlocksPlainSpiritProgress,
    plainSpiritSiblingRegresses,
    allowNonConcreteWhiteProgress,
    whiteSetupRecoveryBlocksUtilityOverride,
  } = policy;
  switch (modeFromConfig(config)) {
    case TurnEngineMode.Baseline:
      switch (plan.headFamily) {
        case TurnPlanFamily.ImmediateScore:
          return (
            (candidate.winsImmediately || scoresNowBetter || sameTurnWindowBetter) &&
            scoreGap <= 280
          );
        case TurnPlanFamily.DenyOpponentWindow:
          return sameTurnWindowBetter || safetyRecoverBetter || drainerAttackBetter;
        case TurnPlanFamily.DrainerKill:
          return drainerAttackBetter && scoreGap <= 180;
        case TurnPlanFamily.DrainerSafetyRecovery:
          return (
            safetyRecoverBetter &&
            compareTurnUtilities(plan.utility, selectedUtility) >= 0 &&
            scoreGap <= 140
          );
        case TurnPlanFamily.SpiritImpact:
          return (
            candidateIndex <= (plan.compiledChunks.length > 1 ? 12 : 6) &&
            scoreGap <= 120 &&
            (spiritWindowBetter ||
              spiritDevelopmentBetter ||
              (selectedSpiritPhase &&
                compareTurnUtilities(plan.utility, selectedUtility) >= 0))
          );
        case TurnPlanFamily.SafeSupermanaProgress:
        case TurnPlanFamily.SafeOpponentManaProgress:
          return (
            !selectedSpiritPhase &&
            candidateIndex <= (plan.compiledChunks.length > 1 ? 3 : 1) &&
            scoreGap <= 80 &&
            (progressBetter ||
              compareTurnUtilities(plan.utility, selectedUtility) >= 0 ||
              (candidate.safeSupermanaProgressSteps ===
                selected.safeSupermanaProgressSteps &&
                candidate.safeOpponentManaProgressSteps ===
                  selected.safeOpponentManaProgressSteps &&
                candidate.ownDrainerVulnerable === selected.ownDrainerVulnerable &&
                candidate.efficiency === selected.efficiency &&
                candidate.supermanaProgress === selected.supermanaProgress &&
                candidate.opponentManaProgress === selected.opponentManaProgress &&
                scoreGap <= 32))
          );
        case TurnPlanFamily.ManaTempo:
          return false;
        default:
          return false;
      }
    case TurnEngineMode.Production:
      switch (plan.headFamily) {
        case TurnPlanFamily.ImmediateScore:
          return (
            (candidate.winsImmediately ||
              scoresNowBetter ||
              sameTurnWindowBetter ||
              (candidateIndex <= 16 &&
                utilityPassesOverrideGuard(plan.utility, selectedUtility) &&
                (!candidateUnsafe || selectedUnsafe))) &&
            scoreGap <= 360
          );
        case TurnPlanFamily.DenyOpponentWindow:
          return (
            sameTurnWindowBetter ||
            safetyRecoverBetter ||
            drainerAttackBetter ||
            (candidateIndex <= 16 &&
              scoreGap <= 220 &&
              !candidateUnsafe &&
              utilityPassesOverrideGuard(plan.utility, selectedUtility))
          );
        case TurnPlanFamily.DrainerKill:
          return (
            candidate.attacksOpponentDrainer &&
            candidateIndex <= 16 &&
            scoreGap <= 260 &&
            (drainerAttackBetter ||
              compareTurnUtilities(plan.utility, selectedUtility) >= 0)
          );
        case TurnPlanFamily.DrainerSafetyRecovery:
          return (
            candidate.classes.drainerSafetyRecover &&
            candidateIndex <= 16 &&
            scoreGap <= 240 &&
            (safetyRecoverBetter ||
              (selected.ownDrainerVulnerable && !candidate.ownDrainerVulnerable) ||
              (productionSecondaryAnalysisLive(config) &&
                utilitySupportsFamilyFallback(plan.utility, selectedUtility) &&
                (selectedUnsafe || !candidateUnsafe) &&
                !whiteSetupRecoveryBlocksUtilityOverride))
          );
        case TurnPlanFamily.SpiritImpact: {
          const engineNotWorse = utilitySupportsFamilyFallback(
            plan.utility,
            selectedUtility,
          );
          const engineBetter = utilityPassesOverrideGuard(
            plan.utility,
            selectedUtility,
          );
          if (
            safeRootBlocksPlainSpirit ||
            safeRootBlocksPlainSpiritProgress ||
            plainSpiritSiblingRegresses
          ) {
            return false;
          }
          const selectedConcreteSpiritSetup =
            selected.spiritSameTurnScoreSetupNow ||
            selected.spiritOwnManaSetupNow ||
            selected.sameTurnScoreWindowValue > 0;
          if (
            selectedConcreteSpiritSetup &&
            !blackSpiritPair &&
            !selectedUnsafe &&
            !candidateUnsafe &&
            !candidate.spiritSameTurnScoreSetupNow &&
            !candidate.spiritOwnManaSetupNow &&
            candidate.sameTurnScoreWindowValue <= selected.sameTurnScoreWindowValue &&
            candidate.spiritSetupGain <= selected.spiritSetupGain &&
            !scoresNowBetter &&
            !drainerAttackBetter
          ) {
            return false;
          }
          const spiritHeadOverride =
            blackTurnSixRouteChangePlainSpirit ||
            (scoreGap <= 220 &&
              (spiritWindowBetter ||
                spiritDevelopmentBetter ||
                (candidate.spiritOwnManaSetupNow &&
                  !selected.spiritOwnManaSetupNow &&
                  engineNotWorse) ||
                engineBetter ||
                (selectedSpiritPhase && engineNotWorse)));
          return (
            candidateIndex <= (plan.compiledChunks.length > 1 ? 16 : 10) &&
            spiritHeadOverride
          );
        }
        case TurnPlanFamily.SafeSupermanaProgress:
        case TurnPlanFamily.SafeOpponentManaProgress: {
          const primaryAxes = compareUtilityPrimaryAxes(plan.utility, selectedUtility);
          const engineNotWorse =
            (candidateProgress || selectedUnsafe) &&
            utilitySupportsFamilyFallback(plan.utility, selectedUtility);
          const selectedSafeNonProgress =
            !selectedUnsafe && !selectedSpiritPhase && !selectedProgress;
          const selectedSafeProgress = !selectedUnsafe && selectedProgress;
          const selectedProgressFamily =
            selectedFamily === TurnPlanFamily.SafeSupermanaProgress ||
            selectedFamily === TurnPlanFamily.SafeOpponentManaProgress;
          const unsafeProgressHasMaterialOverride =
            strategicAxesBetter ||
            utilityHasScoreDeltaForce(plan.utility, selectedUtility, 220);
          if (
            selectedUnsafe &&
            candidateUnsafe &&
            !selectedSpiritPhase &&
            !selectedProgress &&
            candidateProgress &&
            scoreGap > 0 &&
            !pickupUpgrade &&
            !safetyRecoverBetter &&
            !scoresNowBetter &&
            !drainerAttackBetter &&
            !sameTurnWindowBetter &&
            !unsafeProgressHasMaterialOverride
          ) {
            return false;
          }
          if (projectedDeferredRecoveryWithoutConcreteGain) return false;
          const whitePlainSpiritProgress =
            allowNonConcreteWhiteProgress ||
            (productionIsEarlyWhiteTurnStart(game) &&
              isPlainSpiritDevelopmentRoot(selected) &&
              !selectedUnsafe &&
              !candidateUnsafe &&
              candidateProgress &&
              candidateIndex <= 2 &&
              scoreGap <= 96 &&
              (primaryAxes > 0 ||
                (primaryAxes >= 0 &&
                  utilitySupportsPrimaryAxesEvalTolerance(
                    plan.utility,
                    selectedUtility,
                    64,
                  ))));
          const allowSoftProgress =
            !selectedSpiritPhase || selectedUnsafe || whitePlainSpiritProgress;
          const largeSafeLead = scoreGap > 48 && !selectedUnsafe && !candidateUnsafe;
          const searchLeadGuard =
            (!selectedSafeProgress && !selectedSafeNonProgress) ||
            !largeSafeLead ||
            strategicAxesBetter ||
            pickupUpgrade;
          const safeFallback =
            !candidateUnsafe &&
            engineNotWorse &&
            (selectedUnsafe || (!selectedSpiritPhase && !selectedProgress)) &&
            (!selectedSafeNonProgress ||
              scoreGap <= 32 ||
              strategicAxesBetter ||
              pickupUpgrade);
          const engineBetter =
            engineNotWorse &&
            utilityStrictlyDominatesOverrideAxes(plan.utility, selectedUtility) &&
            (!selectedSafeNonProgress || strategicAxesBetter);
          const nearTie =
            candidate.safeSupermanaProgressSteps ===
              selected.safeSupermanaProgressSteps &&
            candidate.safeOpponentManaProgressSteps ===
              selected.safeOpponentManaProgressSteps &&
            candidate.ownDrainerVulnerable === selected.ownDrainerVulnerable &&
            candidate.efficiency === selected.efficiency &&
            candidate.supermanaProgress === selected.supermanaProgress &&
            candidate.opponentManaProgress === selected.opponentManaProgress;
          return (
            candidateIndex <= (plan.compiledChunks.length > 1 ? 12 : 6) &&
            scoreGap <= 220 &&
            ((allowSoftProgress && progressBetter && searchLeadGuard) ||
              whitePlainSpiritProgress ||
              pickupUpgrade ||
              (engineBetter &&
                (!largeSafeLead ||
                  strategicAxesBetter ||
                  hasPickupUpgrade(candidate, selected)) &&
                (!selectedSafeProgress ||
                  strategicAxesBetter ||
                  hasPickupUpgrade(candidate, selected) ||
                  (progressBetter && !selectedSpiritPhase))) ||
              safeFallback ||
              (allowSoftProgress &&
                nearTie &&
                scoreGap <= (selectedProgressFamily ? 32 : 64) &&
                searchLeadGuard))
          );
        }
        case TurnPlanFamily.ManaTempo:
          return false;
      }
  }
}
