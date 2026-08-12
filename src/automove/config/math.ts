export function clamp(value: number, minimum: number, maximum: number): number {
  return Math.min(maximum, Math.max(minimum, Math.trunc(value)));
}

export function subtractSaturating(value: number, amount: number): number {
  return Math.max(0, value - amount);
}

export function scaleFloor(
  value: number,
  numerator: number,
  denominator: number,
): number {
  return Math.trunc((value * numerator) / denominator);
}
