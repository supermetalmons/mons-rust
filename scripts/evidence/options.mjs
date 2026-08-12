import { spawnSync } from "node:child_process";
import path from "node:path";
import vm from "node:vm";

import ts from "typescript";

import { readRegularFileArtifact } from "./artifacts.mjs";

const NativeEvalError = EvalError;
const NativeFunction = Function;
const NativeString = String;
const hostConsoleLog = console.log.bind(console);

export const AUTOMOVE_MODES = Object.freeze(["fast", "normal", "pro"]);
export const INVALID_REASONS = Object.freeze([
  "no-suggestion",
  "source-mutation",
  "selector-error",
  "illegal-replay",
]);
export const MAX_PERFORMANCE_REPEAT = 100;
export const DISALLOW_STRING_CODE_GENERATION_FLAG =
  "--disallow-code-generation-from-strings";
export const EXPERIMENTAL_VM_MODULES_FLAG = "--experimental-vm-modules";

let importSequence = 0;

export function parseCliOptions(arguments_, allowedOptions) {
  const allowed = new Set([...allowedOptions, "help"]);
  const options = new Map();
  for (let index = 0; index < arguments_.length; index += 1) {
    const argument = arguments_[index];
    if (!argument?.startsWith("--")) {
      throw new Error(`unexpected positional argument: ${String(argument)}`);
    }
    const equalsIndex = argument.indexOf("=");
    const name = argument.slice(2, equalsIndex < 0 ? undefined : equalsIndex);
    if (!allowed.has(name)) {
      throw new Error(`unknown option: --${name}`);
    }
    if (options.has(name)) {
      throw new Error(`duplicate option: --${name}`);
    }
    if (name === "help") {
      if (equalsIndex >= 0) {
        throw new Error("--help does not accept a value");
      }
      options.set(name, true);
      continue;
    }
    const value =
      equalsIndex >= 0 ? argument.slice(equalsIndex + 1) : arguments_[index + 1];
    if (value === undefined || value.startsWith("--")) {
      throw new Error(`missing value for --${name}`);
    }
    if (equalsIndex < 0) index += 1;
    options.set(name, value);
  }
  return options;
}

export function requiredOption(options, name) {
  const value = options.get(name);
  if (typeof value !== "string" || value.length === 0) {
    throw new Error(`missing required option: --${name}`);
  }
  return value;
}

export function positiveIntegerOption(options, name, defaultValue, maximum) {
  if (
    !Number.isSafeInteger(defaultValue) ||
    defaultValue < 1 ||
    !Number.isSafeInteger(maximum) ||
    maximum < defaultValue
  ) {
    throw new Error(`invalid numeric bounds for --${name}`);
  }
  const value = options.get(name);
  if (value === undefined) return defaultValue;
  if (typeof value !== "string" || !/^[1-9]\d*$/.test(value)) {
    throw new Error(`--${name} must be a positive safe integer`);
  }
  const parsed = Number(value);
  if (!Number.isSafeInteger(parsed)) {
    throw new Error(`--${name} must be a positive safe integer`);
  }
  if (parsed > maximum) {
    throw new Error(`--${name} must be at most ${maximum}`);
  }
  return parsed;
}

export function selectModes(value = "all") {
  const requested = parseList(value, "modes");
  if (requested.length === 1 && requested[0] === "all") {
    return [...AUTOMOVE_MODES];
  }
  const requestedSet = new Set(requested);
  for (const mode of requestedSet) {
    if (!AUTOMOVE_MODES.includes(mode)) {
      throw new Error(`unsupported automove mode: ${mode}`);
    }
  }
  return AUTOMOVE_MODES.filter((mode) => requestedSet.has(mode));
}

export function selectVariants(value, baselineBundle, candidateBundle) {
  const requested = parseList(value ?? "all", "variants");
  const baselineVariants = baselineBundle.variants;
  const candidateVariants = new Set(candidateBundle.variants);
  const aliases = new Map(
    baselineBundle.variantEntries.flatMap(([key, item]) => [
      [key, item],
      [item, item],
    ]),
  );
  let variants;
  if (requested.length === 1 && requested[0] === "all") {
    variants = [...baselineVariants];
  } else {
    const requestedVariants = requested.map((item) => {
      const variant = aliases.get(item);
      if (variant === undefined) {
        throw new Error(`unsupported game variant: ${item}`);
      }
      return variant;
    });
    const requestedSet = new Set(requestedVariants);
    variants = baselineVariants.filter((variant) => requestedSet.has(variant));
  }
  for (const variant of variants) {
    if (!candidateVariants.has(variant)) {
      throw new Error(`candidate bundle does not export variant: ${variant}`);
    }
  }
  return variants;
}

export async function loadPublicBundle(filePath, role) {
  assertSecureBundleExecutionAvailable(role);
  const artifact = readPublicBundleArtifact(filePath, role);
  importSequence += 1;
  const { context, sandbox } = createBundleContext();
  const module = new vm.SourceTextModule(artifact.bytes.toString("utf8"), {
    context,
    identifier: `${artifact.path}#automoveEvidenceInstance=${encodeURIComponent(`${role}-${importSequence}`)}`,
    importModuleDynamically() {
      throw new Error(`${role} bundle attempted a dynamic import`);
    },
  });
  Object.freeze(sandbox);
  await module.link(() => {
    throw new Error(`${role} bundle attempted a static import`);
  });
  await module.evaluate();
  const { namespace } = module;
  let currentArtifact;
  try {
    currentArtifact = readPublicBundleArtifact(filePath, role);
  } catch {
    throw new Error(`${role} bundle changed while loading`);
  }
  if (
    currentArtifact.identity !== artifact.identity ||
    !currentArtifact.bytes.equals(artifact.bytes)
  ) {
    throw new Error(`${role} bundle changed while loading`);
  }
  if (typeof namespace.Game !== "function") {
    throw new Error(`${role} bundle does not export Game`);
  }
  if (namespace.GameVariant === null || typeof namespace.GameVariant !== "object") {
    throw new Error(`${role} bundle does not export GameVariant`);
  }
  const variantEntries = Object.entries(namespace.GameVariant);
  if (
    variantEntries.length === 0 ||
    variantEntries.some(([key, value]) => key.length === 0 || typeof value !== "string")
  ) {
    throw new Error(`${role} bundle exports an invalid GameVariant`);
  }
  const variants = [...new Set(variantEntries.map(([, value]) => value))];
  return Object.freeze({
    Game: namespace.Game,
    path: artifact.path,
    sha256: artifact.sha256,
    variantEntries: Object.freeze(variantEntries),
    variants: Object.freeze(variants),
  });
}

export async function runBundleExecutingCli(main) {
  if (isolatedModuleExecutionIsAvailable() && stringCodeGenerationIsDisabledInHost()) {
    await main();
    return;
  }
  const scriptPath = process.argv[1];
  if (scriptPath === undefined) {
    throw new Error("bundle-executing CLI path is unavailable");
  }
  const result = spawnSync(
    process.execPath,
    [
      ...process.execArgv,
      EXPERIMENTAL_VM_MODULES_FLAG,
      DISALLOW_STRING_CODE_GENERATION_FLAG,
      "--disable-warning=ExperimentalWarning",
      scriptPath,
      ...process.argv.slice(2),
    ],
    { stdio: "inherit" },
  );
  if (result.error !== undefined) throw result.error;
  if (result.signal !== null) {
    throw new Error(`bundle-executing CLI terminated by ${result.signal}`);
  }
  process.exitCode = result.status ?? 1;
}

function isolatedModuleExecutionIsAvailable() {
  if (typeof vm.SourceTextModule !== "function") return false;
  let context;
  try {
    context = vm.createContext(Object.create(null), {
      codeGeneration: { strings: false, wasm: false },
    });
    vm.runInContext('Function("return undefined")', context);
  } catch (error) {
    if (!isNamedError(error, "EvalError")) return false;
  }
  try {
    vm.runInContext(
      "new WebAssembly.Module(Uint8Array.of(0, 97, 115, 109, 1, 0, 0, 0))",
      context,
    );
  } catch (error) {
    return isNamedError(error, "CompileError");
  }
  return false;
}

function stringCodeGenerationIsDisabledInHost() {
  try {
    NativeFunction("return undefined");
  } catch (error) {
    return error instanceof NativeEvalError;
  }
  return false;
}

function assertSecureBundleExecutionAvailable(role) {
  if (!isolatedModuleExecutionIsAvailable()) {
    throw new Error(
      `${role} bundle execution requires ${EXPERIMENTAL_VM_MODULES_FLAG}`,
    );
  }
  if (!stringCodeGenerationIsDisabledInHost()) {
    throw new Error(
      `${role} bundle execution requires ${DISALLOW_STRING_CODE_GENERATION_FLAG}`,
    );
  }
}

function createBundleContext() {
  const consoleFacade = Object.freeze(
    Object.assign(Object.create(null), {
      log: hostFacade((...values) => {
        hostConsoleLog(values.map((value) => NativeString(value)).join(" "));
      }),
    }),
  );
  const performanceFacade = Object.freeze(
    Object.assign(Object.create(null), {
      now: hostFacade(() => globalThis.performance.now()),
    }),
  );
  const cryptoFacade = Object.freeze(
    Object.assign(Object.create(null), {
      getRandomValues: hostFacade((array) => globalThis.crypto.getRandomValues(array)),
    }),
  );
  const sandbox = Object.assign(Object.create(null), {
    console: consoleFacade,
    crypto: cryptoFacade,
    performance: performanceFacade,
  });
  const context = vm.createContext(sandbox, {
    codeGeneration: { strings: false, wasm: false },
  });
  return { context, sandbox };
}

function hostFacade(callback) {
  return Object.freeze(Object.setPrototypeOf(callback, null));
}

function isNamedError(error, name) {
  return error !== null && typeof error === "object" && error.name === name;
}

export function readPublicBundleArtifact(filePath, role) {
  const artifact = readRegularFileArtifact(filePath, `${role} bundle`);
  const source = artifact.bytes.toString("utf8");
  const sourceFile = ts.createSourceFile(
    path.basename(artifact.path),
    source,
    ts.ScriptTarget.Latest,
    true,
    ts.ScriptKind.JS,
  );
  if (sourceFile.parseDiagnostics.length > 0) {
    throw new Error(`${role} bundle is not valid JavaScript`);
  }
  const exports = new Set();
  let invalidSpecifier;
  const visit = (node) => {
    const dynamicCode = dynamicCodeCapability(node);
    if (dynamicCode !== undefined) {
      invalidSpecifier ??= `dynamic code ${dynamicCode}`;
    }
    if (
      (ts.isImportDeclaration(node) || ts.isExportDeclaration(node)) &&
      node.moduleSpecifier !== undefined &&
      ts.isStringLiteralLike(node.moduleSpecifier)
    ) {
      invalidSpecifier ??= node.moduleSpecifier.text;
    }
    if (
      ts.isCallExpression(node) &&
      node.expression.kind === ts.SyntaxKind.ImportKeyword &&
      node.arguments.length > 0
    ) {
      const [specifier] = node.arguments;
      if (ts.isStringLiteralLike(specifier)) {
        invalidSpecifier ??= specifier.text;
      } else {
        invalidSpecifier ??= "dynamic import expression";
      }
    }
    if (
      ts.isCallExpression(node) &&
      ts.isIdentifier(node.expression) &&
      node.expression.text === "require" &&
      node.arguments.length > 0
    ) {
      const [specifier] = node.arguments;
      if (ts.isStringLiteralLike(specifier)) {
        invalidSpecifier ??= specifier.text;
      } else {
        invalidSpecifier ??= "dynamic require expression";
      }
    }
    if (ts.isMetaProperty(node) && node.keywordToken === ts.SyntaxKind.ImportKeyword) {
      invalidSpecifier ??= "import.meta";
    }
    collectPublicExports(node, exports);
    ts.forEachChild(node, visit);
  };
  visit(sourceFile);
  if (invalidSpecifier !== undefined) {
    throw new Error(`${role} bundle is not self-contained: ${invalidSpecifier}`);
  }
  if (!exports.has("Game") || !exports.has("GameVariant")) {
    throw new Error(`${role} bundle does not declare Game and GameVariant exports`);
  }
  return artifact;
}

function dynamicCodeCapability(node) {
  if (
    ts.isIdentifier(node) &&
    (node.text === "eval" || node.text === "Function" || node.text === "constructor")
  ) {
    return node.text;
  }
  if (
    ts.isElementAccessExpression(node) &&
    node.argumentExpression !== undefined &&
    ts.isStringLiteralLike(node.argumentExpression) &&
    (node.argumentExpression.text === "eval" ||
      node.argumentExpression.text === "Function" ||
      node.argumentExpression.text === "constructor")
  ) {
    return node.argumentExpression.text;
  }
  return undefined;
}

function collectPublicExports(node, exports) {
  if (ts.isExportDeclaration(node) && node.exportClause !== undefined) {
    if (ts.isNamedExports(node.exportClause)) {
      for (const element of node.exportClause.elements) exports.add(element.name.text);
    }
    return;
  }
  if (!hasExportModifier(node)) return;
  if (ts.isVariableStatement(node)) {
    for (const declaration of node.declarationList.declarations) {
      if (ts.isIdentifier(declaration.name)) exports.add(declaration.name.text);
    }
    return;
  }
  if (node.name !== undefined && ts.isIdentifier(node.name))
    exports.add(node.name.text);
}

function hasExportModifier(node) {
  return (
    node.modifiers?.some(
      (modifier) => modifier.kind === ts.SyntaxKind.ExportKeyword,
    ) === true
  );
}

export function initialFen(bundle, variant) {
  const game = new bundle.Game({ variant });
  const fen = game.toFen();
  if (typeof fen !== "string" || fen.length === 0) {
    throw new Error(`bundle produced an invalid initial FEN for ${variant}`);
  }
  return fen;
}

function parseList(value, optionName) {
  const items = value.split(",").map((item) => item.trim());
  if (items.length === 0 || items.some((item) => item.length === 0)) {
    throw new Error(`--${optionName} must be a non-empty comma-separated list`);
  }
  if (new Set(items).size !== items.length) {
    throw new Error(`--${optionName} contains duplicate values`);
  }
  if (items.includes("all") && items.length !== 1) {
    throw new Error(`--${optionName}=all cannot be combined with other values`);
  }
  return items;
}
