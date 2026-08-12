import type { AutomoveExecutionContext } from "../core/execution-context.js";
import { MonKind, type Color } from "../../api/types.js";
import { isMonFainted, itemMon } from "../../engine/model/domain.js";
import { MonsGame } from "../../engine/game/mons-game.js";
import {
  BOARD_SIZE,
  locationEquals,
  nearbyLocations,
} from "../../engine/board/geometry.js";
import {
  EXACT_TURN_TACTICAL_NEED_OPPONENT_MANA_PROGRESS,
  EXACT_TURN_TACTICAL_NEED_SCORE_WINDOW,
  EXACT_TURN_TACTICAL_NEED_SPIRIT_DENIAL,
  EXACT_TURN_TACTICAL_NEED_SPIRIT_SCORE,
  EXACT_TURN_TACTICAL_NEED_SUPERMANA_PROGRESS,
  type ExactOpportunityContext,
  type ExactTurnTacticalProjection,
} from "../exact/types.js";
import { exactSearchStateHash } from "../exact/hash.js";
import { exactStrategicAnalysisWithSearchHash } from "../exact/strategic.js";
import { exactTurnTacticalProjectionWithSearchHash } from "../exact/turn-opportunity.js";
import type { Hash64 } from "../core/hash64.js";
import { applyInputsForSearchWithEvents } from "../transitions/simulation.js";
import { remainingMovesForColor, walkDestinationPlausible } from "./action-rules.js";
import {
  findAwakeDrainerLocation,
  opponentCanWinImmediately,
  ownDrainerSafetyScore,
} from "./evaluation.js";
import { TurnPlanFamily, type ActionSeed, type TurnEngineConfig } from "./model.js";
import { familyAllowed } from "./opportunity-policy.js";

export function tacticalProjectionFlags(
  needSupermanaProgress: boolean,
  needOpponentManaProgress: boolean,
  needSpiritScore: boolean,
  needSpiritDenial: boolean,
  needScoreWindow: boolean,
): number {
  let flags = 0;
  if (needSupermanaProgress) flags |= EXACT_TURN_TACTICAL_NEED_SUPERMANA_PROGRESS;
  if (needOpponentManaProgress)
    flags |= EXACT_TURN_TACTICAL_NEED_OPPONENT_MANA_PROGRESS;
  if (needSpiritScore) flags |= EXACT_TURN_TACTICAL_NEED_SPIRIT_SCORE;
  if (needSpiritDenial) flags |= EXACT_TURN_TACTICAL_NEED_SPIRIT_DENIAL;
  if (needScoreWindow) flags |= EXACT_TURN_TACTICAL_NEED_SCORE_WINDOW;
  return flags;
}

type OracleWalkProjectionProfile =
  | "safe-progress-only"
  | "opponent-progress-only"
  | "drainer-opportunity"
  | "spirit-score-only"
  | "spirit-opportunity";

type OracleWalkActorCapabilities = {
  readonly canEmitSupermana: boolean;
  readonly canEmitOpponentMana: boolean;
  readonly canEmitSafety: boolean;
  readonly canEmitSpirit: boolean;
  readonly tacticalFlags: number;
  readonly projectionProfile: OracleWalkProjectionProfile | undefined;
  readonly needsScoreWindow: boolean;
};

function tacticalProjectionProfileFlags(profile: OracleWalkProjectionProfile): number {
  switch (profile) {
    case "safe-progress-only":
      return tacticalProjectionFlags(true, false, false, false, false);
    case "opponent-progress-only":
      return tacticalProjectionFlags(false, true, false, false, false);
    case "drainer-opportunity":
      return tacticalProjectionFlags(false, true, false, true, false);
    case "spirit-score-only":
      return tacticalProjectionFlags(false, false, true, false, false);
    case "spirit-opportunity":
      return tacticalProjectionFlags(true, false, true, false, false);
  }
}

function oracleWalkActorCapabilities(
  monKind: MonKind,
  allowSupermana: boolean,
  allowOpponentMana: boolean,
  allowSafety: boolean,
  allowSpirit: boolean,
): OracleWalkActorCapabilities {
  const canEmitSupermana = allowSupermana;
  const canEmitOpponentMana = allowOpponentMana && monKind !== MonKind.Spirit;
  const canEmitSafety = allowSafety;
  const canEmitSpirit = allowSpirit && monKind === MonKind.Spirit;
  const projectionProfile: OracleWalkProjectionProfile | undefined =
    canEmitSupermana && canEmitSpirit
      ? "spirit-opportunity"
      : canEmitSupermana
        ? "safe-progress-only"
        : canEmitOpponentMana && allowSpirit
          ? "drainer-opportunity"
          : canEmitOpponentMana
            ? "opponent-progress-only"
            : canEmitSpirit
              ? "spirit-score-only"
              : undefined;
  return {
    canEmitSupermana,
    canEmitOpponentMana,
    canEmitSafety,
    canEmitSpirit,
    tacticalFlags: tacticalProjectionFlags(
      canEmitSupermana,
      canEmitOpponentMana,
      canEmitSpirit,
      allowSpirit && canEmitOpponentMana,
      canEmitSupermana || canEmitSpirit,
    ),
    projectionProfile,
    needsScoreWindow: canEmitSupermana || canEmitSpirit,
  };
}

function strategicSpiritSignalWithSearchHash(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  stateHash: Hash64,
): readonly [number, number] {
  const spirit = exactStrategicAnalysisWithSearchHash(
    execution,
    game,
    stateHash,
  ).colorSummary(perspective).spirit;
  return [spirit.nextTurnSetupGain, spirit.utility];
}

export function oracleWalkSeeds(
  execution: AutomoveExecutionContext,
  game: MonsGame,
  perspective: Color,
  context: ExactOpportunityContext,
  allowedFamilies: readonly TurnPlanFamily[] | undefined,
  config: TurnEngineConfig,
): ActionSeed[] {
  if (execution.session.checkpoint() || remainingMovesForColor(game, perspective) <= 0)
    return [];

  const allowSupermana = familyAllowed(
    allowedFamilies,
    TurnPlanFamily.SafeSupermanaProgress,
  );
  const allowOpponentMana = familyAllowed(
    allowedFamilies,
    TurnPlanFamily.SafeOpponentManaProgress,
  );
  const allowSafety = familyAllowed(
    allowedFamilies,
    TurnPlanFamily.DrainerSafetyRecovery,
  );
  const allowSpirit = familyAllowed(allowedFamilies, TurnPlanFamily.SpiritImpact);
  if (!allowSupermana && !allowOpponentMana && !allowSafety && !allowSpirit) {
    return [];
  }

  const before = context.turn;
  let beforeSpirit: readonly [number, number] = [0, 0];
  if (allowSpirit) {
    beforeSpirit = strategicSpiritSignalWithSearchHash(
      execution,
      game,
      perspective,
      exactSearchStateHash(game),
    );
  }
  if (execution.session.checkpoint()) return [];
  const beforeSafety =
    allowSupermana || allowSafety
      ? ownDrainerSafetyScore(execution, game.board, perspective)
      : 0;
  const beforeSuperSteps = before.safeSupermanaProgressSteps ?? BOARD_SIZE * 3;
  const beforeOpponentSteps = before.safeOpponentManaProgressSteps ?? BOARD_SIZE * 3;
  const ownDrainer = findAwakeDrainerLocation(game.board, perspective);
  const result: ActionSeed[] = [];
  const useLazyScoreWindowProjection = config.enableLazyOracleScoreWindowProjection;

  for (const [actor, item] of game.board.entries()) {
    if (execution.session.checkpoint()) return [];
    const mon = itemMon(item);
    if (
      mon?.color !== perspective ||
      isMonFainted(mon) ||
      (ownDrainer !== undefined && locationEquals(actor, ownDrainer))
    ) {
      continue;
    }
    const capabilities = oracleWalkActorCapabilities(
      mon.kind,
      allowSupermana,
      allowOpponentMana,
      allowSafety,
      allowSpirit,
    );
    if (
      !capabilities.canEmitSupermana &&
      !capabilities.canEmitOpponentMana &&
      !capabilities.canEmitSafety &&
      !capabilities.canEmitSpirit
    ) {
      continue;
    }
    for (const to of nearbyLocations(actor)) {
      if (execution.session.checkpoint()) return [];
      if (!walkDestinationPlausible(game.board, actor, to)) continue;
      const applied = applyInputsForSearchWithEvents(game, [
        { kind: "location", location: actor },
        { kind: "location", location: to },
      ]);
      if (
        applied === undefined ||
        opponentCanWinImmediately(execution, applied.game, perspective)
      )
        continue;

      const needAfterSpirit = capabilities.canEmitSpirit;
      const needAfterTurn = useLazyScoreWindowProjection
        ? capabilities.projectionProfile !== undefined
        : capabilities.tacticalFlags !== 0;
      const needAfterScoreWindow =
        useLazyScoreWindowProjection && capabilities.needsScoreWindow;
      const afterHash =
        needAfterTurn || needAfterScoreWindow || needAfterSpirit
          ? exactSearchStateHash(applied.game)
          : undefined;
      let after: ExactTurnTacticalProjection | undefined;
      if (needAfterTurn) {
        if (afterHash === undefined) {
          throw new Error("oracle walk projection requires a state hash");
        }
        let flags = capabilities.tacticalFlags;
        if (useLazyScoreWindowProjection) {
          if (capabilities.projectionProfile === undefined) {
            throw new Error("lazy oracle projection requires a profile");
          }
          flags = tacticalProjectionProfileFlags(capabilities.projectionProfile);
        }
        after = exactTurnTacticalProjectionWithSearchHash(
          execution,
          applied.game,
          perspective,
          afterHash,
          flags,
        );
      }
      if (execution.session.cancelled) return [];
      let afterSpirit: readonly [number, number] = [0, 0];
      if (needAfterSpirit) {
        if (afterHash === undefined) {
          throw new Error("oracle Spirit analysis requires a state hash");
        }
        afterSpirit = strategicSpiritSignalWithSearchHash(
          execution,
          applied.game,
          perspective,
          afterHash,
        );
      }
      const afterSafety =
        capabilities.canEmitSupermana || capabilities.canEmitSafety
          ? ownDrainerSafetyScore(execution, applied.game.board, perspective)
          : beforeSafety;
      const afterSuperSteps =
        after === undefined
          ? beforeSuperSteps
          : (after.safeSupermanaProgressSteps ?? BOARD_SIZE * 3);
      const afterOpponentSteps =
        after === undefined
          ? beforeOpponentSteps
          : (after.safeOpponentManaProgressSteps ?? BOARD_SIZE * 3);
      let afterScoreWindowValue: number | undefined;
      const loadAfterScoreWindow = (): number => {
        if (afterScoreWindowValue !== undefined) return afterScoreWindowValue;
        if (useLazyScoreWindowProjection) {
          if (afterHash === undefined) {
            throw new Error("oracle score-window projection requires a state hash");
          }
          afterScoreWindowValue = exactTurnTacticalProjectionWithSearchHash(
            execution,
            applied.game,
            perspective,
            afterHash,
            EXACT_TURN_TACTICAL_NEED_SCORE_WINDOW,
          ).sameTurnScoreWindowValue;
        } else {
          afterScoreWindowValue = after?.sameTurnScoreWindowValue ?? 0;
        }
        return afterScoreWindowValue;
      };

      if (capabilities.canEmitSupermana && afterSuperSteps < beforeSuperSteps) {
        result.push({
          family: TurnPlanFamily.SafeSupermanaProgress,
          action: { kind: "walk", actor, to },
          priority:
            8_250 +
            (beforeSuperSteps - afterSuperSteps) * 240 +
            (afterSafety - beforeSafety) * 100 +
            loadAfterScoreWindow() * 160,
        });
      }

      const opponentProgressImproved =
        capabilities.canEmitOpponentMana && afterOpponentSteps < beforeOpponentSteps;
      const spiritDenialImproved =
        allowSpirit &&
        (capabilities.canEmitSpirit || capabilities.canEmitOpponentMana) &&
        (after?.spiritAssistedDenialValue ?? 0) > before.spiritAssistedDenialValue;
      if (opponentProgressImproved || spiritDenialImproved) {
        const family =
          mon.kind === MonKind.Spirit
            ? TurnPlanFamily.SpiritImpact
            : TurnPlanFamily.SafeOpponentManaProgress;
        if (familyAllowed(allowedFamilies, family)) {
          result.push({
            family,
            action: { kind: "walk", actor, to },
            priority:
              8_000 +
              (opponentProgressImproved
                ? Math.max(beforeOpponentSteps - afterOpponentSteps, 0) * 240
                : 0) +
              (spiritDenialImproved
                ? Math.max(
                    (after?.spiritAssistedDenialValue ?? 0) -
                      before.spiritAssistedDenialValue,
                    0,
                  ) * 180
                : 0),
          });
        }
      }

      if (capabilities.canEmitSpirit) {
        const setupDelta = afterSpirit[0] - beforeSpirit[0];
        const utilityDelta = afterSpirit[1] - beforeSpirit[1];
        const spiritSetupImproved = setupDelta > 0 || utilityDelta > 0;
        const spiritScoreBaseImproved =
          (after?.spiritAssistedScoreValue ?? 0) > before.spiritAssistedScoreValue;
        const spiritScoreWindowImproved = useLazyScoreWindowProjection
          ? !spiritScoreBaseImproved &&
            !spiritSetupImproved &&
            loadAfterScoreWindow() > before.sameTurnScoreWindowValue
          : (after?.sameTurnScoreWindowValue ?? 0) > before.sameTurnScoreWindowValue;
        if (
          spiritScoreBaseImproved ||
          spiritScoreWindowImproved ||
          spiritSetupImproved
        ) {
          const scoreDelta =
            (after?.spiritAssistedScoreValue ?? 0) - before.spiritAssistedScoreValue;
          const windowDelta = loadAfterScoreWindow() - before.sameTurnScoreWindowValue;
          result.push({
            family: TurnPlanFamily.SpiritImpact,
            action: { kind: "walk", actor, to },
            priority:
              8_100 +
              scoreDelta * 200 +
              windowDelta * 220 +
              setupDelta * 320 +
              utilityDelta * 180,
          });
        }
      }

      if (capabilities.canEmitSafety && afterSafety > beforeSafety) {
        result.push({
          family: TurnPlanFamily.DrainerSafetyRecovery,
          action: { kind: "walk", actor, to },
          priority: 8_050 + (afterSafety - beforeSafety) * 260,
        });
      }
    }
  }
  return result;
}
