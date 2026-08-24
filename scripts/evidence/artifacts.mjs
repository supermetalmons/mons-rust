import { createHash, randomUUID } from "node:crypto";
import {
  closeSync,
  constants as fsConstants,
  fsyncSync,
  fstatSync,
  linkSync,
  lstatSync,
  mkdirSync,
  openSync,
  readFileSync,
  realpathSync,
  unlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import {
  performanceStateBankAlgorithm,
  performanceStateBankColors,
  performanceStateBankSource,
} from "./performance-state-bank-schema.mjs";

const repositoryRoot = path.resolve(import.meta.dirname, "../..");
const protectedTestDataDirectory = path.join(repositoryRoot, "test-data");
const protectedAutomoveDecisionsDirectory = path.join(
  protectedTestDataDirectory,
  "automove-decisions",
);
const evidenceTargetDirectory = path.join(repositoryRoot, "target");
const temporaryDirectory = path.resolve(tmpdir());
const canonicalProtectedTestDataDirectory = realpathSync(protectedTestDataDirectory);
const protectedPerformanceBankIdentities = new Set(
  ["v1", "v4", "v5", "v6", "v7", "v8", "v9", "v10", "v11", "v12", "v13", "v14"].map(
    (version) =>
      regularFileIdentity(
        path.join(protectedAutomoveDecisionsDirectory, version, "decisions.jsonl"),
      ),
  ),
);
const canonicalEvidenceTargetDirectory = canonicalDestination(evidenceTargetDirectory);
const canonicalTemporaryDirectory = realpathSync(temporaryDirectory);
const sha256Pattern = /^[0-9a-f]{64}$/u;
let temporaryFileSequence = 0;

export function readStateBank(filePath) {
  return readStateBankArtifact(filePath).stateBank;
}

function readStateBankArtifact(filePath) {
  const artifact = readRegularFileArtifact(filePath, "state bank");
  const { bytes: sourceBytes, path: absolutePath } = artifact;
  const source = sourceBytes.toString("utf8");
  const states = [];
  const entries = [];
  const ids = new Set();
  for (const [index, line] of source.split(/\r?\n/u).entries()) {
    if (line.trim().length === 0) continue;
    let value;
    try {
      value = JSON.parse(line);
    } catch {
      throw new Error(`invalid JSON on state-bank line ${index + 1}`);
    }
    if (
      value === null ||
      typeof value !== "object" ||
      typeof value.id !== "string" ||
      value.id.length === 0 ||
      typeof value.fen !== "string" ||
      value.fen.length === 0
    ) {
      throw new Error(`invalid state on state-bank line ${index + 1}`);
    }
    if (
      value.variant !== undefined &&
      (typeof value.variant !== "string" || value.variant.length === 0)
    ) {
      throw new Error(`invalid variant metadata on state-bank line ${index + 1}`);
    }
    if (ids.has(value.id)) {
      throw new Error(`duplicate state-bank id: ${value.id}`);
    }
    ids.add(value.id);
    entries.push(value);
    states.push(
      Object.freeze({
        id: value.id,
        fen: value.fen,
        ...(value.variant === undefined ? {} : { variant: value.variant }),
        ...(value.activeColor === undefined ? {} : { activeColor: value.activeColor }),
      }),
    );
  }
  if (states.length === 0) {
    throw new Error("state bank is empty");
  }
  states.sort((left, right) => compareStrings(left.id, right.id));
  return {
    artifact,
    entries,
    source,
    stateBank: {
      path: absolutePath,
      bytes: sourceBytes.byteLength,
      sha256: artifact.sha256,
      states,
    },
  };
}

export function readPerformanceStateBank(filePath, manifestPath) {
  const parsed = readStateBankArtifact(filePath);
  const { stateBank } = parsed;
  if (protectedPerformanceBankIdentities.has(parsed.artifact.identity)) {
    if (manifestPath !== undefined) {
      throw new Error(
        "--state-manifest is not accepted for protected automove decision banks",
      );
    }
    return stateBank;
  }
  if (manifestPath === undefined) {
    throw new Error(
      "--state-manifest is required for performance state banks outside test-data/automove-decisions/",
    );
  }

  const absoluteManifestPath = path.resolve(manifestPath);
  let manifestArtifact;
  let manifest;
  try {
    manifestArtifact = readRegularFileArtifact(
      absoluteManifestPath,
      "performance state-bank manifest",
    );
    manifest = JSON.parse(manifestArtifact.bytes.toString("utf8"));
  } catch {
    throw new Error(
      `could not read performance state-bank manifest: ${absoluteManifestPath}`,
    );
  }
  if (!validGeneratedStateBankManifest(manifest, parsed)) {
    throw new Error(
      `performance state-bank manifest does not match state bank: ${absoluteManifestPath}`,
    );
  }
  return {
    ...stateBank,
    manifest: {
      path: absoluteManifestPath,
      sha256: manifestArtifact.sha256,
      sourceBundleSha256: manifest.source.bundle.sha256,
    },
  };
}

export function readRegularFileArtifact(filePath, label = "artifact") {
  const absolutePath = path.resolve(filePath);
  let descriptor;
  try {
    descriptor = openSync(absolutePath, fsConstants.O_RDONLY | fsConstants.O_NONBLOCK);
    const before = fstatSync(descriptor, { bigint: true });
    if (!before.isFile())
      throw new Error(`${label} is not a regular file: ${absolutePath}`);
    const bytes = readFileSync(descriptor);
    const after = fstatSync(descriptor, { bigint: true });
    if (!sameStableFile(before, after) || after.size !== BigInt(bytes.byteLength)) {
      throw new Error(`${label} changed while reading: ${absolutePath}`);
    }
    return Object.freeze({
      path: absolutePath,
      bytes,
      identity: fileIdentity(after),
      sha256: bytesSha256(bytes),
    });
  } finally {
    if (descriptor !== undefined) closeSync(descriptor);
  }
}

function validGeneratedStateBankManifest(manifest, parsed) {
  if (
    !hasExactKeys(manifest, [
      "schemaVersion",
      "kind",
      "algorithm",
      "source",
      "candidates",
      "selection",
      "output",
    ]) ||
    manifest.schemaVersion !== 1 ||
    manifest.kind !== "automove-performance-state-bank" ||
    !validGeneratedAlgorithm(manifest.algorithm) ||
    !validGeneratedSource(manifest.source) ||
    !validGeneratedCandidates(manifest.candidates) ||
    !validGeneratedSelection(manifest.selection, parsed.entries) ||
    !validGeneratedOutput(manifest.output, parsed) ||
    !validGeneratedEntries(parsed.entries, parsed.source)
  ) {
    return false;
  }
  return (
    manifest.candidates.eligible +
      manifest.candidates.excludedTerminal +
      manifest.candidates.excludedTooShort ===
    manifest.source.corpus.records
  );
}

function validGeneratedAlgorithm(algorithm) {
  if (
    !hasExactKeys(algorithm, [
      ...Object.keys(performanceStateBankAlgorithm),
      "sha256",
    ]) ||
    !sha256Pattern.test(algorithm.sha256 ?? "")
  ) {
    return false;
  }
  for (const [field, value] of Object.entries(performanceStateBankAlgorithm)) {
    if (algorithm[field] !== value) return false;
  }
  return (
    algorithm.sha256 ===
    bytesSha256(Buffer.from(JSON.stringify(performanceStateBankAlgorithm)))
  );
}

function validGeneratedSource(source) {
  if (
    !hasExactKeys(source, ["sha256", "corpus", "bundle"]) ||
    !hasExactKeys(source.corpus, ["path", "bytes", "sha256", "records"]) ||
    !hasExactKeys(source.bundle, ["sha256"]) ||
    !sha256Pattern.test(source.sha256 ?? "") ||
    !sha256Pattern.test(source.bundle.sha256 ?? "")
  ) {
    return false;
  }
  for (const [field, value] of Object.entries(performanceStateBankSource)) {
    if (source.corpus[field] !== value) return false;
  }
  return (
    source.sha256 ===
    bytesSha256(
      Buffer.from(
        JSON.stringify({
          bundleSha256: source.bundle.sha256,
          corpusSha256: source.corpus.sha256,
        }),
      ),
    )
  );
}

function validGeneratedCandidates(candidates) {
  return (
    hasExactKeys(candidates, ["eligible", "excludedTerminal", "excludedTooShort"]) &&
    [
      candidates.eligible,
      candidates.excludedTerminal,
      candidates.excludedTooShort,
    ].every(isNonnegativeSafeInteger) &&
    candidates.eligible >= performanceStateBankAlgorithm.statesPerColor * 2
  );
}

function validGeneratedSelection(selection, entries) {
  if (
    !hasExactKeys(selection, ["states", "colors", "variants"]) ||
    selection.states !== performanceStateBankAlgorithm.statesPerColor * 2 ||
    selection.states !== entries.length ||
    !hasExactKeys(selection.colors, performanceStateBankColors) ||
    performanceStateBankColors.some(
      (color) =>
        selection.colors[color] !== performanceStateBankAlgorithm.statesPerColor,
    ) ||
    !Array.isArray(selection.variants) ||
    selection.variants.length === 0 ||
    selection.variants.some(
      (variant) => typeof variant !== "string" || variant.length === 0,
    ) ||
    new Set(selection.variants).size !== selection.variants.length ||
    selection.variants.some(
      (variant, index) => index > 0 && selection.variants[index - 1] >= variant,
    )
  ) {
    return false;
  }
  const entryVariants = [...new Set(entries.map((entry) => entry.variant))].sort(
    compareStrings,
  );
  return (
    entryVariants.length === selection.variants.length &&
    entryVariants.every((variant, index) => selection.variants[index] === variant)
  );
}

function validGeneratedOutput(output, parsed) {
  return (
    hasExactKeys(output, [
      "format",
      "encoding",
      "trailingNewline",
      "bytes",
      "sha256",
    ]) &&
    output.format === "jsonl" &&
    output.encoding === "UTF-8" &&
    output.trailingNewline === true &&
    output.bytes === parsed.stateBank.bytes &&
    output.sha256 === parsed.stateBank.sha256
  );
}

function validGeneratedEntries(entries, source) {
  const lines = source.split("\n");
  if (
    source.includes("\r") ||
    !source.endsWith("\n") ||
    lines.length !== entries.length + 1 ||
    entries.some(
      (entry, index) =>
        lines[index] !==
        JSON.stringify({
          id: entry.id,
          fen: entry.fen,
          variant: entry.variant,
          activeColor: entry.activeColor,
          sourceLine: entry.sourceLine,
          midpointTurnIndex: entry.midpointTurnIndex,
          stateSha256: entry.stateSha256,
        }),
    )
  ) {
    return false;
  }
  const sourceLines = new Set();
  const colors = Object.fromEntries(
    performanceStateBankColors.map((color) => [color, 0]),
  );
  let previous;
  for (const entry of entries) {
    if (
      !hasExactKeys(entry, [
        "id",
        "fen",
        "variant",
        "activeColor",
        "sourceLine",
        "midpointTurnIndex",
        "stateSha256",
      ]) ||
      !performanceStateBankColors.includes(entry.activeColor) ||
      !Number.isSafeInteger(entry.sourceLine) ||
      entry.sourceLine < 1 ||
      entry.sourceLine > performanceStateBankSource.records ||
      sourceLines.has(entry.sourceLine) ||
      !Number.isSafeInteger(entry.midpointTurnIndex) ||
      entry.midpointTurnIndex < 1 ||
      !sha256Pattern.test(entry.stateSha256 ?? "") ||
      entry.stateSha256 !== bytesSha256(Buffer.from(entry.fen)) ||
      entry.id !==
        `midpoint-${entry.stateSha256}-${String(entry.sourceLine).padStart(10, "0")}` ||
      (previous !== undefined && compareGeneratedEntries(previous, entry) > 0)
    ) {
      return false;
    }
    sourceLines.add(entry.sourceLine);
    colors[entry.activeColor] += 1;
    previous = entry;
  }
  return performanceStateBankColors.every(
    (color) => colors[color] === performanceStateBankAlgorithm.statesPerColor,
  );
}

function compareGeneratedEntries(left, right) {
  return (
    compareStrings(left.stateSha256, right.stateSha256) ||
    left.sourceLine - right.sourceLine
  );
}

function hasExactKeys(value, keys) {
  return (
    value !== null &&
    typeof value === "object" &&
    !Array.isArray(value) &&
    Object.keys(value).length === keys.length &&
    keys.every((key) => Object.hasOwn(value, key))
  );
}

function isNonnegativeSafeInteger(value) {
  return Number.isSafeInteger(value) && value >= 0;
}

export function preflightJsonReportDestination(filePath) {
  const absolutePath = path.resolve(filePath);
  validateReportDestination(absolutePath);
  mkdirSync(path.dirname(absolutePath), { recursive: true });
  validateReportDestination(absolutePath);
  if (pathEntryExists(absolutePath)) {
    throw existingReportError(absolutePath);
  }
  return absolutePath;
}

export function createExclusiveEvidenceFile(filePath, content) {
  const staged = stageExclusiveEvidenceFile(filePath, content);
  try {
    return commitStagedEvidenceFile(staged);
  } catch (error) {
    removeEvidenceFileIfCreated(staged.temporaryFile);
    throw error;
  }
}

export function createExclusiveEvidencePair(
  firstPath,
  firstContent,
  commitMarkerPath,
  commitMarkerContent,
) {
  const stagedFiles = [];
  const createdFiles = [];
  try {
    stagedFiles.push(stageExclusiveEvidenceFile(firstPath, firstContent));
    stagedFiles.push(stageExclusiveEvidenceFile(commitMarkerPath, commitMarkerContent));
    for (const staged of stagedFiles) {
      createdFiles.push(commitStagedEvidenceFile(staged));
    }
    for (const created of createdFiles) assertEvidenceFileIdentity(created);
    return Object.freeze([...createdFiles]);
  } catch (error) {
    const cleanupErrors = [];
    for (const created of [...createdFiles].reverse()) {
      try {
        removeEvidenceFileIfCreated(created);
      } catch (cleanupError) {
        cleanupErrors.push(cleanupError);
      }
    }
    for (const staged of stagedFiles) {
      try {
        removeEvidenceFileIfCreated(staged.temporaryFile);
      } catch (cleanupError) {
        cleanupErrors.push(cleanupError);
      }
    }
    if (cleanupErrors.length > 0) {
      throw new AggregateError(
        [error, ...cleanupErrors],
        `failed to publish evidence pair: ${path.resolve(firstPath)}`,
      );
    }
    throw error;
  }
}

export function stageExclusiveEvidenceFile(filePath, content) {
  const destination = prepareExclusiveEvidenceDestination(filePath);
  const temporaryPath = temporaryEvidencePath(destination);
  let descriptor;
  try {
    descriptor = openSync(temporaryPath, "wx");
  } catch (error) {
    if (error instanceof Error && "code" in error && error.code === "EEXIST") {
      throw new Error(`temporary evidence file already exists: ${temporaryPath}`);
    }
    throw error;
  }
  try {
    if (fileIdentity(lstatSync(destination.parent)) !== destination.parentIdentity) {
      throw new Error(`evidence destination parent changed: ${destination.path}`);
    }
    writeFileSync(descriptor, content, { encoding: "utf8" });
    fsyncSync(descriptor);
    const identity = fileIdentity(fstatSync(descriptor));
    return Object.freeze({
      ...destination,
      temporaryFile: Object.freeze({ path: temporaryPath, identity }),
    });
  } catch (error) {
    const createdIdentity = fileIdentity(fstatSync(descriptor));
    closeSync(descriptor);
    descriptor = undefined;
    removeEvidenceFileIfCreated({
      path: temporaryPath,
      identity: createdIdentity,
    });
    throw error;
  } finally {
    if (descriptor !== undefined) closeSync(descriptor);
  }
}

function commitStagedEvidenceFile(staged) {
  assertEvidenceFileIdentity(staged.temporaryFile);
  if (fileIdentity(lstatSync(staged.parent)) !== staged.parentIdentity) {
    throw new Error(`evidence destination parent changed: ${staged.path}`);
  }
  try {
    linkSync(staged.temporaryFile.path, staged.path);
  } catch (error) {
    if (error instanceof Error && "code" in error && error.code === "EEXIST") {
      throw existingReportError(staged.path);
    }
    throw error;
  }
  const created = Object.freeze({
    path: staged.path,
    identity: staged.temporaryFile.identity,
  });
  try {
    if (fileIdentity(lstatSync(staged.parent)) !== staged.parentIdentity) {
      throw new Error(`evidence destination parent changed: ${staged.path}`);
    }
    assertEvidenceFileIdentity(created);
    removeEvidenceFileIfCreated(staged.temporaryFile);
    syncDirectory(staged.parent);
    return created;
  } catch (error) {
    removeEvidenceFileIfCreated(created);
    throw error;
  }
}

export function assertEvidenceFileIdentity(createdFile) {
  if (fileIdentityFromPath(createdFile.path) !== createdFile.identity) {
    throw new Error(`evidence file was replaced: ${createdFile.path}`);
  }
}

export function removeEvidenceFileIfCreated(createdFile) {
  const currentIdentity = fileIdentityFromPath(createdFile.path);
  if (currentIdentity === undefined) return false;
  if (currentIdentity !== createdFile.identity) {
    throw new Error(`refusing to remove replaced evidence file: ${createdFile.path}`);
  }
  unlinkSync(createdFile.path);
  return true;
}

function prepareExclusiveEvidenceDestination(filePath) {
  const absolutePath = path.resolve(filePath);
  validateReportDestination(absolutePath);
  mkdirSync(path.dirname(absolutePath), { recursive: true });
  const canonicalPath = canonicalDestination(absolutePath);
  validateReportDestination(canonicalPath);
  if (pathEntryExists(canonicalPath)) {
    throw existingReportError(canonicalPath);
  }
  const parent = realpathSync(path.dirname(canonicalPath));
  return {
    path: path.join(parent, path.basename(canonicalPath)),
    parent,
    parentIdentity: fileIdentity(lstatSync(parent)),
  };
}

function temporaryEvidencePath(destination) {
  temporaryFileSequence += 1;
  return path.join(
    destination.parent,
    `.${path.basename(destination.path)}.${process.pid}.${temporaryFileSequence}.${randomUUID()}.tmp`,
  );
}

function syncDirectory(directory) {
  const descriptor = openSync(directory, "r");
  try {
    fsyncSync(descriptor);
  } finally {
    closeSync(descriptor);
  }
}

export function writeJsonReport(filePath, report) {
  const absolutePath = preflightJsonReportDestination(filePath);
  return createExclusiveEvidenceFile(
    absolutePath,
    `${JSON.stringify(report, null, 2)}\n`,
  ).path;
}

export function bytesSha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function existingReportError(filePath) {
  return new Error(`refusing to overwrite existing evidence report: ${filePath}`);
}

function validateReportDestination(filePath) {
  const destination = canonicalDestination(filePath);
  if (
    isWithin(filePath, protectedTestDataDirectory) ||
    isWithin(destination, canonicalProtectedTestDataDirectory)
  ) {
    throw new Error("refusing to write an evidence report under test-data/");
  }
  const isTargetDestination =
    isWithin(filePath, evidenceTargetDirectory) &&
    isWithin(destination, canonicalEvidenceTargetDirectory);
  const isTemporaryDestination =
    (isWithin(filePath, temporaryDirectory) ||
      isWithin(filePath, canonicalTemporaryDirectory)) &&
    isWithin(destination, canonicalTemporaryDirectory);
  if (!isTargetDestination && !isTemporaryDestination) {
    throw new Error(
      "refusing to write an evidence report outside target/ or the OS temporary directory",
    );
  }
}

function isWithin(filePath, directoryPath) {
  const relative = path.relative(directoryPath, filePath);
  return relative === "" || (!relative.startsWith("..") && !path.isAbsolute(relative));
}

function canonicalDestination(filePath) {
  const suffix = [];
  let existingPath = filePath;
  while (!pathEntryExists(existingPath)) {
    const parent = path.dirname(existingPath);
    if (parent === existingPath) break;
    suffix.unshift(path.basename(existingPath));
    existingPath = parent;
  }
  return path.join(realpathSync(existingPath), ...suffix);
}

function pathEntryExists(filePath) {
  try {
    lstatSync(filePath);
    return true;
  } catch (error) {
    if (error instanceof Error && "code" in error && error.code === "ENOENT") {
      return false;
    }
    throw error;
  }
}

function fileIdentityFromPath(filePath) {
  try {
    return fileIdentity(lstatSync(filePath));
  } catch (error) {
    if (error instanceof Error && "code" in error && error.code === "ENOENT") {
      return undefined;
    }
    throw error;
  }
}

function regularFileIdentity(filePath) {
  const descriptor = openSync(filePath, "r");
  try {
    const stats = fstatSync(descriptor, { bigint: true });
    if (!stats.isFile())
      throw new Error(`protected state bank is not a regular file: ${filePath}`);
    return fileIdentity(stats);
  } finally {
    closeSync(descriptor);
  }
}

function sameStableFile(left, right) {
  return (
    left.dev === right.dev &&
    left.ino === right.ino &&
    left.mode === right.mode &&
    left.nlink === right.nlink &&
    left.size === right.size &&
    left.mtimeNs === right.mtimeNs &&
    left.ctimeNs === right.ctimeNs
  );
}

function fileIdentity(stats) {
  return `${stats.dev}:${stats.ino}`;
}

function compareStrings(left, right) {
  if (left < right) return -1;
  if (left > right) return 1;
  return 0;
}
