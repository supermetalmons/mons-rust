import { Board } from "../../engine/board/storage.js";
import { Color, MonKind } from "../../api/types.js";
import {
  isMonFainted,
  isSpiritTargetAllowed,
  itemMon,
  manaScore,
  otherColor,
} from "../../engine/model/domain.js";
import {
  locationEquals,
  nearbyLocations,
  spiritReachableLocations,
} from "../../engine/board/geometry.js";
import type { AutomoveExecutionContext } from "../core/execution-context.js";
import { Hash64Table } from "../core/hash64.js";
import {
  colorKey,
  EXACT_SPIRIT_SUMMARY_CACHE_MAX_ENTRIES,
  exactCaches,
  exactCacheTag,
} from "./cache.js";
import { exactBoardHash } from "./hash.js";
import { exactSecureSpecificManaStepsOnBoard } from "./secure-mana.js";
import {
  reachableSpiritPositions,
  spiritDestinationAllowed,
} from "./spirit-passive.js";
import {
  applySpiritMovePreviewInPlace,
  exactBestImmediateTacticalWindow,
  type ImmediateTacticalWindow,
  undoSpiritMovePreview,
} from "./spirit-tactical-support.js";
import { defaultSpiritSummary, type ExactSpiritSummary } from "./types.js";

export const EXACT_TACTICAL_SPIRIT_NEED_SCORE = 1 << 0;
export const EXACT_TACTICAL_SPIRIT_NEED_DENIAL = 1 << 1;
export const EXACT_TACTICAL_SPIRIT_NEED_PROGRESS = 1 << 2;
const EXACT_TACTICAL_SPIRIT_ALL_FIELDS =
  EXACT_TACTICAL_SPIRIT_NEED_SCORE |
  EXACT_TACTICAL_SPIRIT_NEED_DENIAL |
  EXACT_TACTICAL_SPIRIT_NEED_PROGRESS;

function tacticalSpiritSummaryForFields(
  summary: ExactSpiritSummary,
  fields: number,
): ExactSpiritSummary {
  const needScore = (fields & EXACT_TACTICAL_SPIRIT_NEED_SCORE) !== 0;
  const needDenial = (fields & EXACT_TACTICAL_SPIRIT_NEED_DENIAL) !== 0;
  const needProgress = (fields & EXACT_TACTICAL_SPIRIT_NEED_PROGRESS) !== 0;
  return {
    ...defaultSpiritSummary(),
    sameTurnScore: needScore && summary.sameTurnScore,
    sameTurnScoreValue: needScore ? summary.sameTurnScoreValue : 0,
    sameTurnOpponentManaScore: needDenial && summary.sameTurnOpponentManaScore,
    sameTurnOpponentManaScoreValue: needDenial
      ? summary.sameTurnOpponentManaScoreValue
      : 0,
    supermanaProgress: needProgress && summary.supermanaProgress,
    opponentManaProgress: needProgress && summary.opponentManaProgress,
  };
}

export function exactTacticalSpiritSummary(
  context: AutomoveExecutionContext,
  board: Board,
  color: Color,
  remainingMonMoves: number,
  canUseAction: boolean,
  fields: number,
): ExactSpiritSummary {
  fields &= EXACT_TACTICAL_SPIRIT_ALL_FIELDS;
  if (
    remainingMonMoves < 0 ||
    fields === 0 ||
    !canUseAction ||
    context.session.checkpoint()
  ) {
    return defaultSpiritSummary();
  }
  const boardHash = exactBoardHash(board);
  const cacheTag = exactCacheTag(
    colorKey(color),
    remainingMonMoves,
    Number(canUseAction),
    fields,
  );
  const cached =
    cacheTag === undefined
      ? undefined
      : exactCaches(context).spiritTacticalSummary.get(boardHash, cacheTag);
  if (cached !== undefined) return cached;
  for (
    let supersetFields = 1;
    supersetFields <= EXACT_TACTICAL_SPIRIT_ALL_FIELDS;
    supersetFields += 1
  ) {
    if (supersetFields === fields || (supersetFields & fields) !== fields) {
      continue;
    }
    const supersetTag = exactCacheTag(
      colorKey(color),
      remainingMonMoves,
      Number(canUseAction),
      supersetFields,
    );
    const superset =
      supersetTag === undefined
        ? undefined
        : exactCaches(context).spiritTacticalSummary.get(boardHash, supersetTag);
    if (superset === undefined) continue;
    const derived = tacticalSpiritSummaryForFields(superset, fields);
    if (!context.session.cacheWriteAllowed) return defaultSpiritSummary();
    if (cacheTag !== undefined) {
      exactCaches(context).spiritTacticalSummary.set(boardHash, derived, cacheTag);
    }
    return derived;
  }
  const result = exactTacticalSpiritSummaryUncached(
    context,
    board,
    color,
    remainingMonMoves,
    canUseAction,
    fields,
  );
  if (!context.session.cacheWriteAllowed) return defaultSpiritSummary();
  if (cacheTag !== undefined) {
    exactCaches(context).spiritTacticalSummary.set(boardHash, result, cacheTag);
  }
  return result;
}

function exactTacticalSpiritSummaryUncached(
  context: AutomoveExecutionContext,
  board: Board,
  color: Color,
  remainingMonMoves: number,
  canUseAction: boolean,
  fields: number,
): ExactSpiritSummary {
  if (!canUseAction || context.session.checkpoint()) return defaultSpiritSummary();
  const needScore = (fields & EXACT_TACTICAL_SPIRIT_NEED_SCORE) !== 0;
  const needDenial = (fields & EXACT_TACTICAL_SPIRIT_NEED_DENIAL) !== 0;
  const needProgress = (fields & EXACT_TACTICAL_SPIRIT_NEED_PROGRESS) !== 0;
  const before = exactBestImmediateTacticalWindow(
    context,
    board,
    color,
    remainingMonMoves,
    needScore,
    needDenial,
  );
  if (context.session.checkpoint()) return defaultSpiritSummary();
  const maxSameTurnScore = needScore ? manaScore({ kind: "supermana" }, color) : 0;
  const maxSameTurnOpponentScore = needDenial
    ? manaScore({ kind: "regular", color: otherColor(color) }, color)
    : 0;
  let best = defaultSpiritSummary();
  const afterWindowCache = new Hash64Table<ImmediateTacticalWindow>(
    EXACT_SPIRIT_SUMMARY_CACHE_MAX_ENTRIES,
  );
  for (const [location, spiritItem] of board.entries()) {
    if (context.session.checkpoint()) return defaultSpiritSummary();
    const spirit = itemMon(spiritItem);
    if (
      spirit?.color !== color ||
      spirit.kind !== MonKind.Spirit ||
      isMonFainted(spirit)
    ) {
      continue;
    }
    for (const [spiritPosition, spiritSteps] of reachableSpiritPositions(
      context,
      board,
      location,
      color,
      remainingMonMoves,
    )) {
      if (context.session.checkpoint()) return defaultSpiritSummary();
      if (board.squareAt(spiritPosition).kind === "mon-base") continue;
      const actionBoard = board.fork();
      if (!locationEquals(spiritPosition, location)) {
        actionBoard.delete(location);
        actionBoard.set(spiritPosition, spiritItem);
      }
      const remainingAfterAction = remainingMonMoves - spiritSteps;
      for (const target of spiritReachableLocations(spiritPosition)) {
        if (context.session.checkpoint()) return defaultSpiritSummary();
        const targetItem = actionBoard.get(target);
        if (targetItem === undefined || !isSpiritTargetAllowed(targetItem)) continue;
        for (const destination of nearbyLocations(target)) {
          if (context.session.checkpoint()) return defaultSpiritSummary();
          if (!spiritDestinationAllowed(actionBoard, targetItem, destination)) {
            continue;
          }
          const preview = applySpiritMovePreviewInPlace(
            actionBoard,
            target,
            targetItem,
            destination,
            color,
          );
          try {
            if (context.session.checkpoint()) return defaultSpiritSummary();
            const scoreFloor = Math.max(
              best.sameTurnScoreValue,
              before.bestScore,
              preview.scoreDelta,
            );
            const denialFloor = Math.max(
              best.sameTurnOpponentManaScoreValue,
              before.bestOpponentManaScore,
              preview.opponentManaScoreDelta,
            );
            const needAfterScore = needScore && scoreFloor < maxSameTurnScore;
            const needAfterDenial =
              needDenial && denialFloor < maxSameTurnOpponentScore;
            let after: ImmediateTacticalWindow = {
              bestScore: 0,
              bestOpponentManaScore: 0,
            };
            if (needAfterScore || needAfterDenial) {
              const minScore = needAfterScore ? scoreFloor + 1 : 1;
              const afterHash = exactBoardHash(actionBoard);
              const afterTag = exactCacheTag(
                colorKey(color),
                remainingAfterAction,
                minScore,
                Number(needAfterScore),
                Number(needAfterDenial),
              );
              const cachedAfter =
                afterTag === undefined
                  ? undefined
                  : afterWindowCache.get(afterHash, afterTag);
              if (cachedAfter !== undefined) {
                after = cachedAfter;
              } else {
                after = exactBestImmediateTacticalWindow(
                  context,
                  actionBoard,
                  color,
                  remainingAfterAction,
                  needAfterScore,
                  needAfterDenial,
                  minScore,
                );
                if (context.session.checkpoint()) return defaultSpiritSummary();
                if (afterTag !== undefined) {
                  afterWindowCache.set(afterHash, after, afterTag);
                }
              }
            }
            if (context.session.checkpoint()) return defaultSpiritSummary();
            const afterScore = Math.max(preview.scoreDelta, after.bestScore);
            const afterDenial = Math.max(
              preview.opponentManaScoreDelta,
              after.bestOpponentManaScore,
            );
            if (
              needScore &&
              (preview.scoreDelta > 0 || afterScore > before.bestScore)
            ) {
              best = {
                ...best,
                sameTurnScore: true,
                sameTurnScoreValue: Math.max(best.sameTurnScoreValue, afterScore),
              };
            }
            if (
              needDenial &&
              (preview.opponentManaScoreDelta > 0 ||
                afterDenial > before.bestOpponentManaScore)
            ) {
              best = {
                ...best,
                sameTurnOpponentManaScore: true,
                sameTurnOpponentManaScoreValue: Math.max(
                  best.sameTurnOpponentManaScoreValue,
                  afterDenial,
                ),
              };
            }
            if (needProgress && !best.supermanaProgress) {
              const movedSupermana =
                targetItem.kind === "mana" &&
                targetItem.mana.kind === "supermana" &&
                preview.scoreDelta > 0;
              let hasSupermanaProgress = movedSupermana;
              if (!hasSupermanaProgress) {
                hasSupermanaProgress =
                  exactSecureSpecificManaStepsOnBoard(
                    context,
                    actionBoard,
                    color,
                    { kind: "supermana" },
                    remainingAfterAction,
                  ) !== undefined;
                if (context.session.checkpoint()) return defaultSpiritSummary();
              }
              if (hasSupermanaProgress) {
                best = { ...best, supermanaProgress: true };
              }
            }
            if (needProgress && !best.opponentManaProgress) {
              let hasOpponentManaProgress = preview.opponentManaScoreDelta > 0;
              if (!hasOpponentManaProgress) {
                hasOpponentManaProgress =
                  exactSecureSpecificManaStepsOnBoard(
                    context,
                    actionBoard,
                    color,
                    { kind: "regular", color: otherColor(color) },
                    remainingAfterAction,
                  ) !== undefined;
                if (context.session.checkpoint()) return defaultSpiritSummary();
              }
              if (hasOpponentManaProgress) {
                best = { ...best, opponentManaProgress: true };
              }
            }
            if (context.session.checkpoint()) return defaultSpiritSummary();
            if (
              (!needScore || best.sameTurnScoreValue >= maxSameTurnScore) &&
              (!needDenial ||
                best.sameTurnOpponentManaScoreValue >= maxSameTurnOpponentScore) &&
              (!needProgress || (best.supermanaProgress && best.opponentManaProgress))
            ) {
              return best;
            }
          } finally {
            undoSpiritMovePreview(actionBoard, preview.undo);
          }
        }
      }
    }
  }
  return context.session.checkpoint() ? defaultSpiritSummary() : best;
}
