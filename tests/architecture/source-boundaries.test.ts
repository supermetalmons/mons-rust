import fs from "node:fs";
import path from "node:path";

import ts from "typescript";
import { describe, expect, it } from "vitest";

const repositoryRoot = path.resolve(import.meta.dirname, "../..");
const sourceRoot = path.join(repositoryRoot, "src");
const maximumSourceLines = 47_231;
const preferredMaximumModuleLines = 600;
const maximumModuleLines = 800;

const largeModuleReasons = new Map<string, string>([
  [
    "automove/packed/evaluation.ts",
    "One allocation-free pass over the packed position, where every term reads accumulators the same loops already build.",
  ],
  [
    "automove/packed/moves.ts",
    "One allocation-free generator pass per move family, sharing the packed move buffer and its ordering keys.",
  ],
  [
    "automove/packed/search.ts",
    "Allocation-neutral packed PVS and negamax with shared timeout and node accounting.",
  ],
  [
    "automove/root/move-efficiency.ts",
    "Order-sensitive board-delta observation and its tightly coupled cache projections.",
  ],
  [
    "automove/search/engine.ts",
    "Recursive bounded search and root orchestration share deadline and node ownership.",
  ],
  [
    "automove/root/focus.ts",
    "The complete order-sensitive focus, ranking, and tie-break chain stays together.",
  ],
  [
    "engine/game/event-compilation.ts",
    "Staged event compilation stays cohesive to preserve emitted event order.",
  ],
  [
    "automove/root/candidates.ts",
    "Root construction and ranking share deterministic enumeration state and order.",
  ],
  [
    "automove/packed/state.ts",
    "Allocation-neutral packed mutation, undo, and incremental hashing share state ownership.",
  ],
  [
    "automove/root/filtering.ts",
    "Ordered root filtering keeps production policy precedence in one chain.",
  ],
  [
    "engine/model/domain.ts",
    "Canonical domain values, copying, equality, keys, and API enum identity share one leaf.",
  ],
  [
    "automove/scoring/evaluator.ts",
    "The allocation-sensitive board scoring loop preserves arithmetic and memo order.",
  ],
  [
    "automove/policy/reply-risk/followup-ordering.ts",
    "Ordered reply arbitration keeps asymmetric precedence and tie-breaks together.",
  ],
  [
    "engine/game/mons-game.ts",
    "Game mutation, history, and query-cache invalidation have a single owner.",
  ],
]);

const engineAreaDependencies = new Map<string, ReadonlySet<string>>([
  ["model", new Set(["model"])],
  ["board", new Set(["board", "model"])],
  ["rules", new Set(["rules", "board", "model"])],
  ["codec", new Set(["codec", "board", "model"])],
  ["game", new Set(["game", "board", "codec", "model", "rules"])],
]);

const automoveAreaDependencies = new Map<string, ReadonlySet<string>>([
  ["core", new Set(["core"])],
  ["config", new Set(["config", "scoring"])],
  ["exact", new Set(["exact", "core"])],
  ["packed", new Set(["packed", "core"])],
  ["transitions", new Set(["transitions", "core"])],
  ["scoring", new Set(["scoring", "core", "exact"])],
  ["turn", new Set(["turn", "config", "core", "exact", "scoring", "transitions"])],
  [
    "root",
    new Set(["root", "config", "core", "exact", "scoring", "transitions", "turn"]),
  ],
  [
    "search",
    new Set(["search", "config", "core", "exact", "root", "scoring", "transitions"]),
  ],
  [
    "policy",
    new Set([
      "policy",
      "config",
      "core",
      "exact",
      "root",
      "scoring",
      "search",
      "transitions",
      "turn",
    ]),
  ],
  [
    "runtime",
    new Set([
      "runtime",
      "config",
      "core",
      "exact",
      "packed",
      "policy",
      "root",
      "search",
      "transitions",
      "turn",
    ]),
  ],
]);

type SourceGraph = ReadonlyMap<string, readonly string[]>;

function sourceFiles(directory = sourceRoot): string[] {
  return fs
    .readdirSync(directory, { withFileTypes: true })
    .flatMap((entry) => {
      const entryPath = path.join(directory, entry.name);
      return entry.isDirectory()
        ? sourceFiles(entryPath)
        : entryPath.endsWith(".ts")
          ? [entryPath]
          : [];
    })
    .sort();
}

function relativeSourcePath(filePath: string): string {
  return path.relative(sourceRoot, filePath).split(path.sep).join("/");
}

function moduleSpecifiers(filePath: string): string[] {
  const source = ts.createSourceFile(
    filePath,
    fs.readFileSync(filePath, "utf8"),
    ts.ScriptTarget.Latest,
    true,
    ts.ScriptKind.TS,
  );
  const specifiers: string[] = [];
  const visit = (node: ts.Node): void => {
    if (
      (ts.isImportDeclaration(node) || ts.isExportDeclaration(node)) &&
      node.moduleSpecifier !== undefined &&
      ts.isStringLiteral(node.moduleSpecifier)
    ) {
      specifiers.push(node.moduleSpecifier.text);
    } else if (
      ts.isCallExpression(node) &&
      node.expression.kind === ts.SyntaxKind.ImportKeyword
    ) {
      const [argument] = node.arguments;
      if (argument !== undefined && ts.isStringLiteral(argument)) {
        specifiers.push(argument.text);
      }
    }
    ts.forEachChild(node, visit);
  };
  visit(source);
  return specifiers;
}

function reexportSpecifiers(filePath: string): string[] {
  const source = ts.createSourceFile(
    filePath,
    fs.readFileSync(filePath, "utf8"),
    ts.ScriptTarget.Latest,
    true,
    ts.ScriptKind.TS,
  );
  return source.statements
    .filter(
      (node): node is ts.ExportDeclaration =>
        ts.isExportDeclaration(node) &&
        node.moduleSpecifier !== undefined &&
        ts.isStringLiteral(node.moduleSpecifier),
    )
    .map((node) => (node.moduleSpecifier as ts.StringLiteral).text);
}

type ImportedBindingReexport = {
  readonly localName: string;
  readonly moduleSpecifier: string;
};

type AliasSource =
  | {
      readonly kind: "expression";
      readonly expression: ts.Expression;
      readonly at: number;
      readonly laterAssignment: boolean;
    }
  | {
      readonly kind: "projection";
      readonly base: AliasSource;
      readonly key: string | number;
      readonly at: number;
      readonly laterAssignment: boolean;
    }
  | {
      readonly kind: "ambiguous";
      readonly base: AliasSource;
      readonly at: number;
      readonly laterAssignment: boolean;
    };

type LocalBinding = {
  immutable: boolean;
  assigned: boolean;
  readonly sources: AliasSource[];
};

function unwrappedIdentifier(
  expression: ts.Expression | undefined,
): ts.Identifier | undefined {
  const current = unwrappedExpression(expression);
  return current !== undefined && ts.isIdentifier(current) ? current : undefined;
}

function unwrappedExpression(
  expression: ts.Expression | undefined,
): ts.Expression | undefined {
  let current = expression;
  while (
    current !== undefined &&
    (ts.isParenthesizedExpression(current) ||
      ts.isAsExpression(current) ||
      ts.isSatisfiesExpression(current) ||
      ts.isTypeAssertionExpression(current) ||
      ts.isNonNullExpression(current))
  ) {
    current = current.expression;
  }
  return current;
}

function importedBindingReexportsFromSource(
  source: ts.SourceFile,
): ImportedBindingReexport[] {
  const imports = new Map<string, string>();
  const namespaceImports = new Map<string, string>();
  const localBindings = new Map<string, LocalBinding>();
  for (const statement of source.statements) {
    if (
      !ts.isImportDeclaration(statement) ||
      statement.importClause === undefined ||
      !ts.isStringLiteral(statement.moduleSpecifier)
    ) {
      continue;
    }
    const specifier = statement.moduleSpecifier.text;
    if (statement.importClause.name !== undefined) {
      imports.set(statement.importClause.name.text, specifier);
    }
    const bindings = statement.importClause.namedBindings;
    if (bindings === undefined) continue;
    if (ts.isNamespaceImport(bindings)) {
      imports.set(bindings.name.text, specifier);
      namespaceImports.set(bindings.name.text, specifier);
      continue;
    }
    for (const element of bindings.elements) {
      imports.set(element.name.text, specifier);
    }
  }
  const expressionSource = (
    expression: ts.Expression,
    at: number,
    laterAssignment: boolean,
  ): AliasSource => ({ kind: "expression", expression, at, laterAssignment });
  const projectedSource = (base: AliasSource, key: string | number): AliasSource => ({
    kind: "projection",
    base,
    key,
    at: base.at,
    laterAssignment: base.laterAssignment,
  });
  const ambiguousSource = (base: AliasSource): AliasSource => ({
    kind: "ambiguous",
    base,
    at: base.at,
    laterAssignment: base.laterAssignment,
  });
  const localBinding = (name: string, immutable: boolean): LocalBinding => {
    const existing = localBindings.get(name);
    if (existing !== undefined) {
      existing.immutable &&= immutable;
      return existing;
    }
    const created: LocalBinding = { immutable, assigned: false, sources: [] };
    localBindings.set(name, created);
    return created;
  };
  const staticPropertyName = (
    name: ts.PropertyName | undefined,
  ): string | number | undefined => {
    if (name === undefined) return undefined;
    if (ts.isIdentifier(name) || ts.isStringLiteralLike(name)) return name.text;
    if (ts.isNumericLiteral(name)) return Number(name.text);
    if (!ts.isComputedPropertyName(name)) return undefined;
    const expression = unwrappedExpression(name.expression);
    if (expression !== undefined && ts.isStringLiteralLike(expression)) {
      return expression.text;
    }
    return expression !== undefined && ts.isNumericLiteral(expression)
      ? Number(expression.text)
      : undefined;
  };
  const recordBindingName = (
    name: ts.BindingName,
    value: AliasSource | undefined,
    immutable: boolean,
    assigned: boolean,
  ): void => {
    if (ts.isIdentifier(name)) {
      const binding = localBinding(name.text, immutable);
      binding.assigned ||= assigned;
      if (value !== undefined) binding.sources.push(value);
      return;
    }
    if (value === undefined) return;
    if (ts.isObjectBindingPattern(name)) {
      for (const element of name.elements) {
        const key =
          element.propertyName === undefined && ts.isIdentifier(element.name)
            ? element.name.text
            : staticPropertyName(element.propertyName);
        const projected =
          element.dotDotDotToken !== undefined || key === undefined
            ? ambiguousSource(value)
            : projectedSource(value, key);
        recordBindingName(element.name, projected, immutable, assigned);
        if (element.initializer !== undefined) {
          recordBindingName(
            element.name,
            expressionSource(element.initializer, value.at, value.laterAssignment),
            immutable,
            assigned,
          );
        }
      }
      return;
    }
    for (const [index, element] of name.elements.entries()) {
      if (ts.isOmittedExpression(element)) continue;
      const projected =
        element.dotDotDotToken === undefined
          ? projectedSource(value, index)
          : ambiguousSource(value);
      recordBindingName(element.name, projected, immutable, assigned);
      if (element.initializer !== undefined) {
        recordBindingName(
          element.name,
          expressionSource(element.initializer, value.at, value.laterAssignment),
          immutable,
          assigned,
        );
      }
    }
  };
  for (const statement of source.statements) {
    if (!ts.isVariableStatement(statement)) continue;
    const immutable = (statement.declarationList.flags & ts.NodeFlags.Const) !== 0;
    for (const declaration of statement.declarationList.declarations) {
      recordBindingName(
        declaration.name,
        declaration.initializer === undefined
          ? undefined
          : expressionSource(
              declaration.initializer,
              declaration.getStart(source),
              false,
            ),
        immutable,
        false,
      );
    }
  }
  const recordAssignmentTarget = (target: ts.Expression, value: AliasSource): void => {
    const unwrapped = unwrappedExpression(target);
    if (unwrapped === undefined) return;
    if (ts.isIdentifier(unwrapped)) {
      const binding = localBindings.get(unwrapped.text);
      if (binding !== undefined) {
        binding.assigned = true;
        binding.sources.push(value);
      }
      return;
    }
    if (ts.isObjectLiteralExpression(unwrapped)) {
      for (const property of unwrapped.properties) {
        if (ts.isSpreadAssignment(property)) {
          recordAssignmentTarget(property.expression, ambiguousSource(value));
          continue;
        }
        if (ts.isShorthandPropertyAssignment(property)) {
          const projected = projectedSource(value, property.name.text);
          recordAssignmentTarget(property.name, projected);
          if (property.objectAssignmentInitializer !== undefined) {
            recordAssignmentTarget(
              property.name,
              expressionSource(property.objectAssignmentInitializer, value.at, true),
            );
          }
          continue;
        }
        if (!ts.isPropertyAssignment(property)) continue;
        const key = staticPropertyName(property.name);
        const projected =
          key === undefined ? ambiguousSource(value) : projectedSource(value, key);
        const assignmentTarget = unwrappedExpression(property.initializer);
        if (
          assignmentTarget !== undefined &&
          ts.isBinaryExpression(assignmentTarget) &&
          assignmentTarget.operatorToken.kind === ts.SyntaxKind.EqualsToken
        ) {
          recordAssignmentTarget(assignmentTarget.left, projected);
          recordAssignmentTarget(
            assignmentTarget.left,
            expressionSource(assignmentTarget.right, value.at, true),
          );
        } else {
          recordAssignmentTarget(property.initializer, projected);
        }
      }
      return;
    }
    if (!ts.isArrayLiteralExpression(unwrapped)) return;
    for (const [index, element] of unwrapped.elements.entries()) {
      if (ts.isOmittedExpression(element)) continue;
      if (ts.isSpreadElement(element)) {
        recordAssignmentTarget(element.expression, ambiguousSource(value));
        continue;
      }
      const projected = projectedSource(value, index);
      const assignmentTarget = unwrappedExpression(element);
      if (
        assignmentTarget !== undefined &&
        ts.isBinaryExpression(assignmentTarget) &&
        assignmentTarget.operatorToken.kind === ts.SyntaxKind.EqualsToken
      ) {
        recordAssignmentTarget(assignmentTarget.left, projected);
        recordAssignmentTarget(
          assignmentTarget.left,
          expressionSource(assignmentTarget.right, value.at, true),
        );
      } else {
        recordAssignmentTarget(element, projected);
      }
    }
  };
  for (const statement of source.statements) {
    if (!ts.isExpressionStatement(statement)) continue;
    const expression = unwrappedExpression(statement.expression);
    if (
      expression === undefined ||
      !ts.isBinaryExpression(expression) ||
      expression.operatorToken.kind !== ts.SyntaxKind.EqualsToken
    ) {
      continue;
    }
    recordAssignmentTarget(
      expression.left,
      expressionSource(expression.right, statement.getStart(source), true),
    );
  }
  const aliasCycle = (
    activeAliases: ReadonlySet<string>,
    name: string,
  ): Set<string> => {
    if (activeAliases.has(name)) {
      throw new Error(`cyclic local alias in ${source.fileName}: ${name}`);
    }
    return new Set([...activeAliases, name]);
  };
  const eligibleSources = (local: LocalBinding, at: number): AliasSource[] =>
    local.sources.filter(
      (candidate) => !candidate.laterAssignment || candidate.at <= at,
    );
  const namespaceModuleSpecifier = (
    aliasSource: AliasSource,
    activeAliases: ReadonlySet<string>,
  ): string | undefined => {
    if (aliasSource.kind !== "expression") return undefined;
    const identifier = unwrappedIdentifier(aliasSource.expression);
    if (identifier === undefined) return undefined;
    const importedModule = namespaceImports.get(identifier.text);
    if (importedModule !== undefined) return importedModule;
    const local = localBindings.get(identifier.text);
    if (local === undefined) return undefined;
    const cycle = aliasCycle(activeAliases, identifier.text);
    const modules = new Set(
      eligibleSources(local, aliasSource.at)
        .map((candidate) => namespaceModuleSpecifier(candidate, cycle))
        .filter((candidate) => candidate !== undefined),
    );
    if (modules.size > 0 && (!local.immutable || local.assigned)) {
      throw new Error(`mutable local alias in ${source.fileName}: ${identifier.text}`);
    }
    if (modules.size > 1) {
      throw new Error(
        `ambiguous imported alias in ${source.fileName}: ${identifier.text}`,
      );
    }
    return modules.values().next().value;
  };
  const literalProjectionSources = (
    aliasSource: AliasSource,
    key: string | number,
  ): AliasSource[] => {
    if (aliasSource.kind !== "expression") return [];
    const expression = unwrappedExpression(aliasSource.expression);
    if (expression === undefined) return [];
    if (ts.isObjectLiteralExpression(expression) && typeof key === "string") {
      for (let index = expression.properties.length - 1; index >= 0; index -= 1) {
        const property = expression.properties[index];
        if (property === undefined) continue;
        if (ts.isSpreadAssignment(property)) {
          return [
            projectedSource(
              expressionSource(property.expression, aliasSource.at, false),
              key,
            ),
          ];
        }
        if (staticPropertyName(property.name) !== key) continue;
        if (ts.isPropertyAssignment(property)) {
          return [expressionSource(property.initializer, aliasSource.at, false)];
        }
        if (ts.isShorthandPropertyAssignment(property)) {
          return [expressionSource(property.name, aliasSource.at, false)];
        }
        return [];
      }
      return [];
    }
    if (
      ts.isArrayLiteralExpression(expression) &&
      typeof key === "number" &&
      expression.elements.every((element) => !ts.isSpreadElement(element))
    ) {
      const element = expression.elements[key];
      return element === undefined || ts.isOmittedExpression(element)
        ? []
        : [expressionSource(element, aliasSource.at, false)];
    }
    return [];
  };
  const reexportSource = (
    aliasSource: AliasSource,
    activeAliases: ReadonlySet<string>,
  ): ImportedBindingReexport[] => {
    if (aliasSource.kind === "expression") {
      return reexport(aliasSource.expression, activeAliases, aliasSource.at);
    }
    if (aliasSource.kind === "ambiguous") {
      if (namespaceModuleSpecifier(aliasSource.base, activeAliases) !== undefined) {
        throw new Error(`ambiguous imported alias in ${source.fileName}`);
      }
      return [];
    }
    const moduleSpecifier = namespaceModuleSpecifier(aliasSource.base, activeAliases);
    if (moduleSpecifier !== undefined) {
      return [{ localName: String(aliasSource.key), moduleSpecifier }];
    }
    return literalProjectionSources(aliasSource.base, aliasSource.key).flatMap(
      (candidate) => reexportSource(candidate, activeAliases),
    );
  };
  const reexport = (
    expression: ts.Expression | undefined,
    activeAliases: ReadonlySet<string> = new Set(),
    at = Number.POSITIVE_INFINITY,
  ): ImportedBindingReexport[] => {
    const unwrapped = unwrappedExpression(expression);
    if (unwrapped === undefined) return [];
    if (ts.isIdentifier(unwrapped)) {
      const moduleSpecifier = imports.get(unwrapped.text);
      if (moduleSpecifier !== undefined) {
        return [{ localName: unwrapped.text, moduleSpecifier }];
      }
      const local = localBindings.get(unwrapped.text);
      if (local === undefined) return [];
      const result = eligibleSources(local, at).flatMap((candidate) =>
        reexportSource(candidate, aliasCycle(activeAliases, unwrapped.text)),
      );
      if (result.length > 0 && (!local.immutable || local.assigned)) {
        throw new Error(`mutable local alias in ${source.fileName}: ${unwrapped.text}`);
      }
      return result;
    }
    if (ts.isPropertyAccessExpression(unwrapped)) {
      const moduleSpecifier = namespaceModuleSpecifier(
        expressionSource(unwrapped.expression, at, false),
        activeAliases,
      );
      return moduleSpecifier === undefined
        ? []
        : [{ localName: unwrapped.name.text, moduleSpecifier }];
    }
    if (!ts.isElementAccessExpression(unwrapped)) return [];
    const moduleSpecifier = namespaceModuleSpecifier(
      expressionSource(unwrapped.expression, at, false),
      activeAliases,
    );
    if (moduleSpecifier === undefined) return [];
    const argument = unwrappedExpression(unwrapped.argumentExpression);
    if (
      argument === undefined ||
      (!ts.isStringLiteralLike(argument) && !ts.isNumericLiteral(argument))
    ) {
      throw new Error(`ambiguous imported alias in ${source.fileName}`);
    }
    return [{ localName: argument.text, moduleSpecifier }];
  };
  return source.statements.flatMap((statement): ImportedBindingReexport[] => {
    if (
      ts.isExportDeclaration(statement) &&
      statement.moduleSpecifier === undefined &&
      statement.exportClause !== undefined &&
      ts.isNamedExports(statement.exportClause)
    ) {
      return statement.exportClause.elements.flatMap((element) => {
        const localName = element.propertyName ?? element.name;
        return ts.isIdentifier(localName) ? reexport(localName) : [];
      });
    }
    if (
      ts.isVariableStatement(statement) &&
      statement.modifiers?.some(({ kind }) => kind === ts.SyntaxKind.ExportKeyword)
    ) {
      return statement.declarationList.declarations.flatMap(({ name }) => {
        const identifiers: ts.Identifier[] = [];
        const collect = (bindingName: ts.BindingName): void => {
          if (ts.isIdentifier(bindingName)) {
            identifiers.push(bindingName);
            return;
          }
          for (const element of bindingName.elements) {
            if (!ts.isOmittedExpression(element)) collect(element.name);
          }
        };
        collect(name);
        return identifiers.flatMap((identifier) => reexport(identifier));
      });
    }
    return ts.isExportAssignment(statement)
      ? reexport(statement.expression, new Set(), statement.getStart(source))
      : [];
  });
}

function importedBindingReexports(filePath: string): ImportedBindingReexport[] {
  return importedBindingReexportsFromSource(
    ts.createSourceFile(
      filePath,
      fs.readFileSync(filePath, "utf8"),
      ts.ScriptTarget.Latest,
      true,
      ts.ScriptKind.TS,
    ),
  );
}

function resolveSourceImport(from: string, specifier: string): string | undefined {
  const unresolved = path.resolve(path.dirname(from), specifier);
  const candidates = specifier.endsWith(".js")
    ? [`${unresolved.slice(0, -3)}.ts`]
    : [unresolved, `${unresolved}.ts`, path.join(unresolved, "index.ts")];
  const resolved = candidates.find((candidate) => fs.existsSync(candidate));
  if (resolved === undefined) {
    throw new Error(`${relativeSourcePath(from)} imports ${specifier}`);
  }
  const relative = relativeSourcePath(resolved);
  return relative.startsWith("../") ? undefined : relative;
}

function sourceGraph(): SourceGraph {
  return new Map(
    sourceFiles().map((filePath) => [
      relativeSourcePath(filePath),
      moduleSpecifiers(filePath)
        .filter((specifier) => specifier.startsWith("."))
        .map((specifier) => resolveSourceImport(filePath, specifier))
        .filter((dependency) => dependency !== undefined),
    ]),
  );
}

function cycleIn(graph: SourceGraph): string[] | undefined {
  const complete = new Set<string>();
  const active = new Map<string, number>();
  const pathStack: string[] = [];
  const visit = (filePath: string): string[] | undefined => {
    if (complete.has(filePath)) return undefined;
    const cycleStart = active.get(filePath);
    if (cycleStart !== undefined) {
      return [...pathStack.slice(cycleStart), filePath];
    }
    active.set(filePath, pathStack.length);
    pathStack.push(filePath);
    for (const dependency of graph.get(filePath) ?? []) {
      const cycle = visit(dependency);
      if (cycle !== undefined) return cycle;
    }
    pathStack.pop();
    active.delete(filePath);
    complete.add(filePath);
    return undefined;
  };
  for (const filePath of graph.keys()) {
    const cycle = visit(filePath);
    if (cycle !== undefined) return cycle;
  }
  return undefined;
}

function subareaGraph(graph: SourceGraph, area: string): SourceGraph {
  const dependencies = new Map<string, Set<string>>();
  for (const [source, sourceDependencies] of graph) {
    if (topLevelArea(source) !== area) continue;
    const sourceSubarea = subarea(source);
    if (sourceSubarea === undefined) continue;
    const subareaDependencies = dependencies.get(sourceSubarea) ?? new Set<string>();
    dependencies.set(sourceSubarea, subareaDependencies);
    for (const dependency of sourceDependencies) {
      if (topLevelArea(dependency) !== area) continue;
      const dependencySubarea = subarea(dependency);
      if (dependencySubarea !== undefined && dependencySubarea !== sourceSubarea) {
        subareaDependencies.add(dependencySubarea);
      }
    }
  }
  return new Map(
    [...dependencies].map(([source, sourceDependencies]) => [
      source,
      [...sourceDependencies].sort(),
    ]),
  );
}

function topLevelArea(filePath: string): string {
  return filePath.split("/", 1)[0] ?? filePath;
}

function subarea(filePath: string): string | undefined {
  return filePath.split("/")[1];
}

function subareaViolation(
  source: string,
  dependency: string,
  area: "engine" | "automove",
  allowedDependencies: ReadonlyMap<string, ReadonlySet<string>>,
): string | undefined {
  if (topLevelArea(source) !== area || topLevelArea(dependency) !== area) {
    return undefined;
  }
  if (
    area === "engine" &&
    source === "engine/model/domain.ts" &&
    dependency === "engine/board/geometry.ts"
  ) {
    return undefined;
  }
  const sourceSubarea = subarea(source);
  const dependencySubarea = subarea(dependency);
  if (
    sourceSubarea === undefined ||
    dependencySubarea === undefined ||
    !allowedDependencies.get(sourceSubarea)?.has(dependencySubarea)
  ) {
    return `${source} -> ${dependency}`;
  }
  return undefined;
}

const productionSourceGraph = sourceGraph();

describe("source architecture", () => {
  it("keeps the production import graph acyclic", () => {
    const cycle = cycleIn(productionSourceGraph);
    expect(cycle?.join(" -> "), "source import cycle").toBeUndefined();
  });

  it("keeps layer dependencies directed and api/types as the shared leaf", () => {
    const graph = productionSourceGraph;
    const violations: string[] = [];
    for (const [source, dependencies] of graph) {
      const sourceArea = topLevelArea(source);
      for (const dependency of dependencies) {
        const dependencyArea = topLevelArea(dependency);
        const allowed =
          sourceArea === dependencyArea ||
          (sourceArea === "entrypoints" && dependencyArea === "api") ||
          (sourceArea === "api" &&
            (dependencyArea === "engine" || dependencyArea === "automove")) ||
          (sourceArea === "automove" && dependencyArea === "engine") ||
          dependency === "api/types.ts" ||
          (sourceArea === "cli" && dependencyArea === "engine");
        if (!allowed) violations.push(`${source} -> ${dependency}`);
      }
    }
    expect(violations).toEqual([]);
    expect(graph.get("api/types.ts")).toEqual([]);
  });

  it("keeps engine and automove subarea dependencies directed", () => {
    const graph = productionSourceGraph;
    const violations: string[] = [];
    for (const [source, dependencies] of graph) {
      for (const dependency of dependencies) {
        for (const [area, allowed] of [
          ["engine", engineAreaDependencies],
          ["automove", automoveAreaDependencies],
        ] as const) {
          const violation = subareaViolation(source, dependency, area, allowed);
          if (violation !== undefined) violations.push(violation);
        }
      }
    }
    expect(violations).toEqual([]);
    expect(
      cycleIn(subareaGraph(graph, "automove"))?.join(" -> "),
      "automove subarea cycle",
    ).toBeUndefined();
  });

  it("uses direct implementation imports outside the published and contract leaves", () => {
    const directReexportViolations = sourceFiles().flatMap((filePath) => {
      const source = relativeSourcePath(filePath);
      return reexportSpecifiers(filePath)
        .filter(
          (specifier) =>
            source !== "entrypoints/mons-rules.ts" &&
            !(
              source === "engine/model/domain.ts" && specifier === "../../api/types.js"
            ),
        )
        .map((specifier) => `${source} -> ${specifier}`);
    });
    const allowedImportedBindingReexports = new Set([
      "engine/board/config.ts -> ../../api/types.js#GameVariant",
      "engine/model/domain.ts -> ../../api/types.js#Color",
      "engine/model/domain.ts -> ../../api/types.js#Consumable",
      "engine/model/domain.ts -> ../../api/types.js#Modifier",
      "engine/model/domain.ts -> ../../api/types.js#MonKind",
    ]);
    const importedBindingViolations = sourceFiles().flatMap((filePath) => {
      const source = relativeSourcePath(filePath);
      return importedBindingReexports(filePath)
        .map(
          ({ localName, moduleSpecifier }) =>
            `${source} -> ${moduleSpecifier}#${localName}`,
        )
        .filter((reexport) => !allowedImportedBindingReexports.has(reexport));
    });

    expect(directReexportViolations).toEqual([]);
    expect(importedBindingViolations).toEqual([]);
  });

  it("detects wrapped variable and default aliases of imported bindings", () => {
    const source = ts.createSourceFile(
      "import-reexport-mutations.ts",
      `import { defaultAlias, variableAlias } from "./implementation.js";
       export const alias = ((variableAlias as unknown) satisfies unknown);
       export default ((defaultAlias as unknown) satisfies unknown);`,
      ts.ScriptTarget.Latest,
      true,
      ts.ScriptKind.TS,
    );

    expect(importedBindingReexportsFromSource(source)).toEqual([
      { localName: "variableAlias", moduleSpecifier: "./implementation.js" },
      { localName: "defaultAlias", moduleSpecifier: "./implementation.js" },
    ]);
  });

  it("detects wrapped variable and default aliases of namespace-import properties", () => {
    const source = ts.createSourceFile(
      "namespace-import-reexport-mutations.ts",
      `import * as implementation from "./implementation.js";
       export const alias = ((implementation.variableAlias as unknown) satisfies unknown);
       export default ((implementation.defaultAlias as unknown) satisfies unknown);`,
      ts.ScriptTarget.Latest,
      true,
      ts.ScriptKind.TS,
    );

    expect(importedBindingReexportsFromSource(source)).toEqual([
      { localName: "variableAlias", moduleSpecifier: "./implementation.js" },
      { localName: "defaultAlias", moduleSpecifier: "./implementation.js" },
    ]);
  });

  it("detects imported bindings through immutable local alias chains", () => {
    const source = ts.createSourceFile(
      "local-import-reexport-mutations.ts",
      `import { implementation } from "./implementation.js";
       const first = (implementation as unknown);
       const second = (first satisfies unknown);
       export { second as alias };`,
      ts.ScriptTarget.Latest,
      true,
      ts.ScriptKind.TS,
    );

    expect(importedBindingReexportsFromSource(source)).toEqual([
      { localName: "implementation", moduleSpecifier: "./implementation.js" },
    ]);
  });

  it("detects namespace bindings through immutable destructuring", () => {
    const source = ts.createSourceFile(
      "namespace-destructuring-reexport-mutations.ts",
      `import * as implementation from "./implementation.js";
       const { variableAlias: alias } = implementation;
       export const { defaultAlias } = implementation;
       export { alias };`,
      ts.ScriptTarget.Latest,
      true,
      ts.ScriptKind.TS,
    );

    expect(importedBindingReexportsFromSource(source)).toEqual([
      { localName: "defaultAlias", moduleSpecifier: "./implementation.js" },
      { localName: "variableAlias", moduleSpecifier: "./implementation.js" },
    ]);
  });

  it("detects imports selected from object and array literals", () => {
    const source = ts.createSourceFile(
      "literal-destructuring-reexport-mutations.ts",
      `import { first, second } from "./implementation.js";
       const { value: objectAlias } = { value: first };
       const [arrayAlias] = [second];
       export { objectAlias, arrayAlias };`,
      ts.ScriptTarget.Latest,
      true,
      ts.ScriptKind.TS,
    );

    expect(importedBindingReexportsFromSource(source)).toEqual([
      { localName: "first", moduleSpecifier: "./implementation.js" },
      { localName: "second", moduleSpecifier: "./implementation.js" },
    ]);
  });

  it("rejects imported bindings introduced by later assignments", () => {
    const direct = ts.createSourceFile(
      "later-assignment-reexport-mutations.ts",
      `import { implementation } from "./implementation.js";
       let alias;
       alias = implementation;
       export { alias };`,
      ts.ScriptTarget.Latest,
      true,
      ts.ScriptKind.TS,
    );
    const destructured = ts.createSourceFile(
      "later-destructuring-reexport-mutations.ts",
      `import * as implementation from "./implementation.js";
       let alias;
       ({ value: alias } = implementation);
       export { alias };`,
      ts.ScriptTarget.Latest,
      true,
      ts.ScriptKind.TS,
    );

    expect(() => importedBindingReexportsFromSource(direct)).toThrow(
      "mutable local alias",
    );
    expect(() => importedBindingReexportsFromSource(destructured)).toThrow(
      "mutable local alias",
    );
  });

  it("does not flag ordinary destructured or mutable exports", () => {
    const source = ts.createSourceFile(
      "ordinary-local-export-mutations.ts",
      `const local = { value: 1 };
       export const { value } = local;
       export let counter = 0;
       counter = 1;`,
      ts.ScriptTarget.Latest,
      true,
      ts.ScriptKind.TS,
    );

    expect(importedBindingReexportsFromSource(source)).toEqual([]);
  });

  it("does not let a later assignment contaminate an earlier snapshot", () => {
    const source = ts.createSourceFile(
      "later-assignment-snapshot-mutations.ts",
      `import { implementation } from "./implementation.js";
       let local;
       const snapshot = local;
       local = implementation;
       export { snapshot };`,
      ts.ScriptTarget.Latest,
      true,
      ts.ScriptKind.TS,
    );

    expect(importedBindingReexportsFromSource(source)).toEqual([]);
  });

  it("rejects ambiguous namespace destructuring that reaches an export", () => {
    const rest = ts.createSourceFile(
      "namespace-rest-reexport-mutations.ts",
      `import * as implementation from "./implementation.js";
       const { ...alias } = implementation;
       export { alias };`,
      ts.ScriptTarget.Latest,
      true,
      ts.ScriptKind.TS,
    );
    const computed = ts.createSourceFile(
      "namespace-computed-reexport-mutations.ts",
      `import * as implementation from "./implementation.js";
       const key = "value";
       const { [key]: alias } = implementation;
       export { alias };`,
      ts.ScriptTarget.Latest,
      true,
      ts.ScriptKind.TS,
    );

    expect(() => importedBindingReexportsFromSource(rest)).toThrow(
      "ambiguous imported alias",
    );
    expect(() => importedBindingReexportsFromSource(computed)).toThrow(
      "ambiguous imported alias",
    );
  });

  it("rejects cyclic and mutable local alias chains", () => {
    const cyclic = ts.createSourceFile(
      "cyclic-import-reexport-mutations.ts",
      `const first = second;
       const second = first;
       export { first as alias };`,
      ts.ScriptTarget.Latest,
      true,
      ts.ScriptKind.TS,
    );
    const mutable = ts.createSourceFile(
      "mutable-import-reexport-mutations.ts",
      `import { implementation } from "./implementation.js";
       let local = implementation;
       export { local as alias };`,
      ts.ScriptTarget.Latest,
      true,
      ts.ScriptKind.TS,
    );

    expect(() => importedBindingReexportsFromSource(cyclic)).toThrow(
      "cyclic local alias",
    );
    expect(() => importedBindingReexportsFromSource(mutable)).toThrow(
      "mutable local alias",
    );
  });

  it("keeps the reorganized source tree within its physical size limits", () => {
    const modules = sourceFiles().map((filePath) => ({
      filePath: relativeSourcePath(filePath),
      lines: fs.readFileSync(filePath, "utf8").split("\n").length - 1,
    }));
    const oversized = modules
      .filter(({ lines }) => lines > maximumModuleLines)
      .map(({ filePath, lines }) => `${filePath}: ${lines}`);
    const undocumentedLargeModules = modules
      .filter(
        ({ filePath, lines }) =>
          lines > preferredMaximumModuleLines && !largeModuleReasons.has(filePath),
      )
      .map(({ filePath, lines }) => `${filePath}: ${lines}`);
    const staleLargeModuleReasons = [...largeModuleReasons]
      .filter(([filePath, reason]) => {
        const module = modules.find((candidate) => candidate.filePath === filePath);
        return (
          reason.trim().length === 0 ||
          module === undefined ||
          module.lines <= preferredMaximumModuleLines
        );
      })
      .map(([filePath]) => filePath);
    const total = modules.reduce((sum, { lines }) => sum + lines, 0);

    expect(oversized).toEqual([]);
    expect(undocumentedLargeModules).toEqual([]);
    expect(staleLargeModuleReasons).toEqual([]);
    expect(total).toBeLessThanOrEqual(maximumSourceLines);
  });
});
