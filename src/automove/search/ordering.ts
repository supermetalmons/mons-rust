import type { MonsGame } from "../../engine/game/mons-game.js";
import { saturatingScoreAdd } from "../core/score-math.js";
import { hash64CompareUnsigned, type Hash64 } from "../core/hash64.js";
import type { MoveClassFlags } from "../root/types.js";

const CHILD_CLASS_SCORE_MARGIN = 110;

export type RankedChild = {
  readonly game: MonsGame;
  readonly hash: Hash64;
  readonly heuristic: number;
  readonly orderingEfficiency: number;
  readonly tacticalExtensionTrigger: boolean;
  readonly quietReductionCandidate: boolean;
  readonly classes: MoveClassFlags;
};

function classPriority(classes: MoveClassFlags): number {
  let score = 0;
  if (classes.immediateScore) score += 1_000;
  if (classes.drainerAttack) score += 700;
  if (classes.drainerSafetyRecover) score += 500;
  if (classes.carrierProgress) score += 220;
  if (classes.material) score += 80;
  return score;
}

export function compareRankedChildren(
  left: RankedChild,
  right: RankedChild,
  maximizing: boolean,
): number {
  if (left.heuristic !== right.heuristic) {
    return maximizing
      ? right.heuristic - left.heuristic
      : left.heuristic - right.heuristic;
  }
  if (left.orderingEfficiency !== right.orderingEfficiency) {
    return right.orderingEfficiency - left.orderingEfficiency;
  }
  const classOrder = classPriority(right.classes) - classPriority(left.classes);
  return classOrder !== 0 ? classOrder : -hash64CompareUnsigned(left.hash, right.hash);
}

function childWithinCoverageMargin(
  score: number,
  cutoff: number,
  maximizing: boolean,
): boolean {
  return maximizing
    ? saturatingScoreAdd(score, CHILD_CLASS_SCORE_MARGIN) >= cutoff
    : score <= saturatingScoreAdd(cutoff, CHILD_CLASS_SCORE_MARGIN);
}

export function isPriorityChild(child: RankedChild): boolean {
  return (
    child.classes.immediateScore ||
    child.classes.drainerAttack ||
    child.classes.drainerSafetyRecover ||
    child.classes.carrierProgress ||
    (child.orderingEfficiency > 0 && !child.classes.material)
  );
}

export function truncateChildrenWithCoverage(
  children: readonly RankedChild[],
  limit: number,
  maximizing: boolean,
  strictGuarantees = true,
): RankedChild[] {
  if (children.length <= limit || limit === 0) return [...children];
  const cutoff = children[limit - 1]?.heuristic ?? 0;
  const preserveIndex = children.findIndex(
    (child, index) =>
      index >= limit &&
      isPriorityChild(child) &&
      (strictGuarantees ||
        childWithinCoverageMargin(child.heuristic, cutoff, maximizing)),
  );
  if (preserveIndex < 0) return children.slice(0, limit);
  const selected = new Array<boolean>(children.length).fill(false);
  selected[preserveIndex] = true;
  let selectedCount = 1;
  for (let index = 0; index < selected.length; index += 1) {
    if (selectedCount >= limit) break;
    if (selected[index] === true) continue;
    selected[index] = true;
    selectedCount += 1;
  }
  return children.filter((_child, index) => selected[index] === true);
}

export function enforceTacticalChildTop2(
  children: RankedChild[],
  maximizing: boolean,
  strictGuarantees = true,
): void {
  if (children.length < 3 || children.slice(0, 2).some(isPriorityChild)) return;
  const secondScore = children[1]?.heuristic ?? 0;
  const replacementIndex = children.findIndex((child, index) => {
    if (index < 2 || !isPriorityChild(child)) return false;
    return (
      strictGuarantees ||
      childWithinCoverageMargin(child.heuristic, secondScore, maximizing)
    );
  });
  if (replacementIndex >= 2) {
    const second = children[1];
    const replacement = children[replacementIndex];
    if (second !== undefined && replacement !== undefined) {
      children[1] = replacement;
      children[replacementIndex] = second;
    }
  }
}

export function isQuietReductionCandidate(
  orderingEfficiency: number,
  tacticalExtensionTrigger: boolean,
  classes: MoveClassFlags,
): boolean {
  return (
    !classes.material &&
    orderingEfficiency <= 0 &&
    !tacticalExtensionTrigger &&
    !classes.immediateScore &&
    !classes.drainerAttack &&
    !classes.drainerSafetyRecover &&
    !classes.carrierProgress
  );
}

export function isSelectiveExtensionCandidate(
  tacticalExtensionTrigger: boolean,
  orderingEfficiency: number,
  classes: MoveClassFlags,
): boolean {
  return (
    tacticalExtensionTrigger ||
    (orderingEfficiency > 0 && !classes.quiet && !classes.material)
  );
}
