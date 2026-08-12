import { BOARD_SIZE } from "../../engine/board/geometry.js";
import type { Event } from "../../engine/model/domain.js";
import type { Hash64 } from "../core/hash64.js";
import type {
  AutomoveConfig,
  MoveClassFlags as SelectorMoveClassFlags,
  RootObservation,
} from "../config/types.js";

export const UNKNOWN_PROGRESS_STEPS = BOARD_SIZE + 4;
export const UNKNOWN_SCORE_PATH_STEPS = BOARD_SIZE * 3;

export type SearchConfig = AutomoveConfig;
export type MoveClassFlags = SelectorMoveClassFlags;

export type RootCandidate = RootObservation & {
  readonly heuristic: number;
  readonly events: readonly Event[];
  readonly stateHash: Hash64;
};

export type EvaluatedRoot = RootCandidate & {
  readonly score: number;
  readonly nodesAfter: number;
};

export type RootCandidateDraft = Omit<RootCandidate, "rootRank">;
