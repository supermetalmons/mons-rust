import type { PlanNode } from "./model.js";
import { compareChunks, compareNumber } from "./ordering.js";

export function compareOrderedNodes(
  left: { readonly order: number; readonly node: PlanNode },
  right: { readonly order: number; readonly node: PlanNode },
): number {
  const order = compareNumber(right.order, left.order);
  return order !== 0
    ? order
    : compareChunks(left.node.compiledChunks, right.node.compiledChunks);
}
