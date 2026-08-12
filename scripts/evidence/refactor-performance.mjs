import path from "node:path";

import { checkAutomovePerformance } from "../check-automove-performance.mjs";
import { deriveAutomovePerformanceStates } from "../derive-automove-performance-states.mjs";
import { readRegularFileArtifact } from "./artifacts.mjs";
import { readPublicBundleArtifact } from "./options.mjs";

export const PRISTINE_BASELINE_SHA256 =
  "bc40e846cc5389aba69faf14b4c28bfaaf7773d7daf6ba64e640a87cbbc9d2fc";

const requiredModes = Object.freeze(["fast", "normal", "pro"]);
const requiredRepeat = 5;
const requiredV6States = 13;
const requiredMidpointStates = 64;
const requiredV6Sha256 =
  "32799d75bebfe49494770af1657ff232208244081f7eafe9adaa606b1a251ee1";
const requiredV6Path = path.resolve(
  import.meta.dirname,
  "../..",
  "test-data/automove-decisions/v6/decisions.jsonl",
);

export function assertRefactorPerformanceContract(
  v6Report,
  midpointReport,
  currentCandidateSha256,
) {
  assertFinalReport(v6Report, "v6", requiredV6States);
  assertFinalReport(midpointReport, "midpoint", requiredMidpointStates);

  if (
    v6Report.stateBank !== requiredV6Path ||
    v6Report.stateBankSha256 !== requiredV6Sha256
  ) {
    throw new Error("v6 performance report does not use the exact protected bank");
  }
  if (
    v6Report.stateBankManifest !== undefined ||
    v6Report.stateBankManifestSha256 !== undefined
  ) {
    throw new Error("v6 performance report must not use a state-bank manifest");
  }
  if (
    typeof midpointReport.stateBankManifest !== "string" ||
    midpointReport.stateBankManifest.length === 0 ||
    typeof midpointReport.stateBankManifestSha256 !== "string" ||
    midpointReport.stateBankManifestSha256.length === 0
  ) {
    throw new Error("midpoint performance report requires a state-bank manifest");
  }
  if (v6Report.candidateSha256 !== midpointReport.candidateSha256) {
    throw new Error("performance reports do not use the same candidate artifact");
  }
  if (v6Report.candidateSha256 !== currentCandidateSha256) {
    throw new Error(
      "performance reports do not use the supplied current candidate artifact",
    );
  }
}

export function assertDerivedMidpointStateBank(midpointReport, derivedStateBank) {
  const artifact = readRegularFileArtifact(
    midpointReport.stateBank,
    "midpoint performance state bank",
  );
  const expected = Buffer.from(derivedStateBank, "utf8");
  if (
    artifact.sha256 !== midpointReport.stateBankSha256 ||
    !artifact.bytes.equals(expected)
  ) {
    throw new Error(
      "midpoint performance state bank differs from fresh pristine-baseline derivation",
    );
  }
}

export function assertCurrentCandidateUnchanged(candidateArtifact) {
  let current;
  try {
    current = readPublicBundleArtifact(candidateArtifact.path, "current candidate");
  } catch {
    throw new Error("current candidate bundle changed while checking reports");
  }
  if (
    current.identity !== candidateArtifact.identity ||
    !current.bytes.equals(candidateArtifact.bytes)
  ) {
    throw new Error("current candidate bundle changed while checking reports");
  }
}

export async function checkRefactorPerformanceReports(
  v6Report,
  midpointReport,
  candidatePath,
) {
  const candidateArtifact = readPublicBundleArtifact(
    candidatePath,
    "current candidate",
  );
  assertRefactorPerformanceContract(v6Report, midpointReport, candidateArtifact.sha256);
  const v6 = checkAutomovePerformance(v6Report);
  const midpoint = checkAutomovePerformance(midpointReport);
  const derived = await deriveAutomovePerformanceStates(v6Report.baseline);
  if (derived.manifest.source.bundle.sha256 !== PRISTINE_BASELINE_SHA256) {
    throw new Error("fresh midpoint derivation did not use the pristine baseline");
  }
  assertDerivedMidpointStateBank(midpointReport, derived.stateBank);
  assertCurrentCandidateUnchanged(candidateArtifact);
  return Object.freeze({
    baselineSha256: PRISTINE_BASELINE_SHA256,
    candidateSha256: candidateArtifact.sha256,
    midpoint,
    v6,
  });
}

function assertFinalReport(report, label, expectedStates) {
  if (report === null || typeof report !== "object" || Array.isArray(report)) {
    throw new Error(`invalid ${label} performance report`);
  }
  if (report.baselineSha256 !== PRISTINE_BASELINE_SHA256) {
    throw new Error(`${label} performance report does not use the pristine baseline`);
  }
  if (report.candidateSha256 === PRISTINE_BASELINE_SHA256) {
    throw new Error(`${label} performance report compares the baseline with itself`);
  }
  if (
    report.config === null ||
    typeof report.config !== "object" ||
    Array.isArray(report.config) ||
    !sameArray(report.config.modes, requiredModes) ||
    report.config.repeat !== requiredRepeat ||
    report.config.order !== "ABBA" ||
    report.config.samplesPerBundlePerState !== requiredRepeat * 2
  ) {
    throw new Error(
      `${label} performance report must use fast,normal,pro with ABBA repeat 5`,
    );
  }
  if (report.states !== expectedStates) {
    throw new Error(
      `${label} performance report must contain exactly ${expectedStates} states`,
    );
  }
}

function sameArray(value, expected) {
  return (
    Array.isArray(value) &&
    value.length === expected.length &&
    value.every((item, index) => item === expected[index])
  );
}
