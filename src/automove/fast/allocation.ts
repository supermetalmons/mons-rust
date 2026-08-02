export const FAST_WORKSPACE_ALLOCATION_FAILED = new RangeError(
  "fast workspace allocation failed",
);

export function isFastWorkspaceAllocationFailure(
  error: unknown,
): error is RangeError {
  return error === FAST_WORKSPACE_ALLOCATION_FAILED;
}

export function rethrowFastWorkspaceAllocation(error: unknown): never {
  if (error instanceof RangeError) throw FAST_WORKSPACE_ALLOCATION_FAILED;
  throw error;
}
