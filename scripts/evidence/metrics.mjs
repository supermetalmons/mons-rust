export function emptyInvalidCounts() {
  return {
    total: 0,
    noSuggestion: 0,
    sourceMutation: 0,
    selectorError: 0,
    illegalReplay: 0,
  };
}

export function addInvalid(counts, reason) {
  counts.total += 1;
  if (reason === "no-suggestion") counts.noSuggestion += 1;
  if (reason === "source-mutation") counts.sourceMutation += 1;
  if (reason === "selector-error") counts.selectorError += 1;
  if (reason === "illegal-replay") counts.illegalReplay += 1;
}

export function mergeInvalidCounts(items) {
  const merged = emptyInvalidCounts();
  for (const item of items) {
    merged.total += item.total;
    merged.noSuggestion += item.noSuggestion;
    merged.sourceMutation += item.sourceMutation;
    merged.selectorError += item.selectorError;
    merged.illegalReplay += item.illegalReplay;
  }
  return merged;
}

export function timingSummary(values) {
  const sorted = values.map(roundNumber).sort((left, right) => left - right);
  if (sorted.length === 0) {
    return {
      count: 0,
      totalMs: 0,
      meanMs: null,
      medianMs: null,
      p95Ms: null,
      maxMs: null,
    };
  }
  const total = sorted.reduce((sum, value) => sum + value, 0);
  const middle = Math.floor(sorted.length / 2);
  const median =
    sorted.length % 2 === 0
      ? (sorted[middle - 1] + sorted[middle]) / 2
      : sorted[middle];
  const p95Index = Math.max(0, Math.ceil(sorted.length * 0.95) - 1);
  return {
    count: sorted.length,
    totalMs: roundNumber(total),
    meanMs: roundNumber(total / sorted.length),
    medianMs: roundNumber(median),
    p95Ms: sorted[p95Index],
    maxMs: sorted[sorted.length - 1],
  };
}

export function timingRatios(candidate, baseline) {
  return {
    mean: safeRatio(candidate.meanMs, baseline.meanMs),
    median: safeRatio(candidate.medianMs, baseline.medianMs),
    p95: safeRatio(candidate.p95Ms, baseline.p95Ms),
    max: safeRatio(candidate.maxMs, baseline.maxMs),
  };
}

export function roundNumber(value) {
  return Math.round(value * 1_000_000) / 1_000_000;
}

function safeRatio(numerator, denominator) {
  if (
    typeof numerator !== "number" ||
    typeof denominator !== "number" ||
    denominator === 0
  ) {
    return null;
  }
  return roundNumber(numerator / denominator);
}
