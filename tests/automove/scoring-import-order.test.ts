import fs from "node:fs";
import path from "node:path";

import ts from "typescript";
import { describe, expect, it, vi } from "vitest";

import { Color } from "../../src/api/types.js";
import { BUILT_IN_PROFILE_SIGNATURES } from "../../src/automove/scoring/built-in-profile-signatures.js";
import type { ScoringWeights } from "../../src/automove/scoring/profile-schema.js";
import type { MonsGame } from "../../src/engine/game/mons-game.js";

const repositoryRoot = path.resolve(import.meta.dirname, "../..");
const PROFILE_CONSTRUCTORS = new Set([
  "defineScoringProfile",
  "normalizeScoringProfile",
]);

function unwrappedExpression(expression: ts.Expression): ts.Expression {
  let current = expression;
  while (
    ts.isParenthesizedExpression(current) ||
    ts.isAsExpression(current) ||
    ts.isSatisfiesExpression(current) ||
    ts.isTypeAssertionExpression(current) ||
    ts.isNonNullExpression(current)
  ) {
    current = current.expression;
  }
  return current;
}

function calledIdentifier(call: ts.CallExpression): string | undefined {
  const expression = unwrappedExpression(call.expression);
  return ts.isIdentifier(expression) ? expression.text : undefined;
}

function profileIdExpression(call: ts.CallExpression): ts.Expression | undefined {
  const definition = call.arguments[0];
  if (definition === undefined || !ts.isObjectLiteralExpression(definition)) {
    return undefined;
  }
  for (const property of definition.properties) {
    if (ts.isShorthandPropertyAssignment(property) && property.name.text === "id") {
      return property.name;
    }
    if (
      ts.isPropertyAssignment(property) &&
      ((ts.isIdentifier(property.name) && property.name.text === "id") ||
        (ts.isStringLiteral(property.name) && property.name.text === "id"))
    ) {
      return unwrappedExpression(property.initializer);
    }
  }
  return undefined;
}

function literalText(expression: ts.Expression | undefined): string | undefined {
  if (expression === undefined) return undefined;
  const unwrapped = unwrappedExpression(expression);
  return ts.isStringLiteral(unwrapped) || ts.isNoSubstitutionTemplateLiteral(unwrapped)
    ? unwrapped.text
    : undefined;
}

function constructedProfileIds(relativePath: string): string[] {
  const filePath = path.join(repositoryRoot, relativePath);
  const source = ts.createSourceFile(
    filePath,
    fs.readFileSync(filePath, "utf8"),
    ts.ScriptTarget.Latest,
    true,
    ts.ScriptKind.TS,
  );
  const factories = new Map<string, number>();
  const factoryCalls = new Set<ts.CallExpression>();

  for (const statement of source.statements) {
    if (!ts.isFunctionDeclaration(statement) || statement.name === undefined) continue;
    const factoryName = statement.name.text;
    const parameters = statement.parameters.map((parameter) =>
      ts.isIdentifier(parameter.name) ? parameter.name.text : undefined,
    );
    const visit = (node: ts.Node): void => {
      if (
        ts.isCallExpression(node) &&
        PROFILE_CONSTRUCTORS.has(calledIdentifier(node) ?? "")
      ) {
        const id = profileIdExpression(node);
        const parameterIndex =
          id !== undefined && ts.isIdentifier(id) ? parameters.indexOf(id.text) : -1;
        if (parameterIndex >= 0) {
          factories.set(factoryName, parameterIndex);
          factoryCalls.add(node);
        }
      }
      ts.forEachChild(node, visit);
    };
    visit(statement);
  }

  const ids: string[] = [];
  const visit = (node: ts.Node): void => {
    if (ts.isCallExpression(node)) {
      const name = calledIdentifier(node);
      const factoryParameter = name === undefined ? undefined : factories.get(name);
      if (name !== undefined && PROFILE_CONSTRUCTORS.has(name)) {
        const id = literalText(profileIdExpression(node));
        if (id !== undefined) ids.push(id);
        else if (!factoryCalls.has(node)) {
          throw new Error(`unsupported profile id construction in ${relativePath}`);
        }
      } else if (factoryParameter !== undefined) {
        const id = literalText(node.arguments[factoryParameter]);
        if (id === undefined) {
          throw new Error(`non-literal profile factory id in ${relativePath}`);
        }
        ids.push(id);
      }
    }
    ts.forEachChild(node, visit);
  };
  visit(source);
  return ids;
}

function zeroSection(keys: readonly string[]): Record<string, number> {
  return Object.fromEntries(keys.map((key) => [key, 0]));
}

function conflictingProfile(id: string, base: ScoringWeights): ScoringWeights {
  return {
    ...base,
    id,
    material: { ...base.material, confirmedScore: 999 },
  };
}

describe("scoring profile module initialization", () => {
  it("protects every built-in id across every registry entry point", async () => {
    vi.resetModules();
    const schema = await import("../../src/automove/scoring/profile-schema.js");
    const validation = await import("../../src/automove/scoring/profile-validation.js");
    const builtInProfileIds = [
      ...constructedProfileIds("src/automove/scoring/presets.ts"),
      ...constructedProfileIds("src/automove/config/profiles.ts"),
    ];
    expect(new Set(builtInProfileIds).size).toBe(builtInProfileIds.length);
    expect(Object.keys(BUILT_IN_PROFILE_SIGNATURES).toSorted()).toEqual(
      builtInProfileIds.toSorted(),
    );
    expect("defineBuiltInScoringProfile" in validation).toBe(false);
    expect("registerBuiltInRootScoringProfile" in validation).toBe(false);
    const base = {
      id: "test.import-order-base",
      formula: {
        useHeuristicFormula: true,
        includeRegularManaMoveWindows: false,
        includeMatchPointWindow: false,
        nextTurnWindowScaleBp: 5_000,
        doubleConfirmedScore: true,
      },
      material: zeroSection(schema.MATERIAL_KEYS),
      position: zeroSection(schema.POSITION_KEYS),
      mana: zeroSection(schema.MANA_KEYS),
      race: zeroSection(schema.RACE_KEYS),
      threat: zeroSection(schema.THREAT_KEYS),
    } as ScoringWeights;
    const custom = validation.defineScoringProfile({
      id: "test.before-presets",
      base,
    });
    expect(custom.id).toBe("test.before-presets");

    const expectDirectConflict = (id: string): void => {
      const message = `scoring profile id ${id} is already registered`;
      const hostile = conflictingProfile(id, base);
      expect(() =>
        validation.defineScoringProfile({
          id,
          base,
          material: { confirmedScore: 999 },
        }),
      ).toThrow(message);
      expect(() => validation.validateScoringProfile(hostile)).toThrow(message);
      expect(() => validation.normalizeScoringProfile(hostile)).toThrow(message);
    };

    for (const id of builtInProfileIds) {
      expectDirectConflict(id);
    }

    const mutableProfile = {
      ...base,
      id: "test.mutable-before-presets",
      material: { ...base.material, activeMon: 5 },
    };
    const normalized = validation.normalizeScoringProfile(mutableProfile);
    expect(normalized).not.toBe(mutableProfile);
    expect(Object.isFrozen(normalized)).toBe(true);
    for (const section of [
      normalized.formula,
      normalized.material,
      normalized.position,
      normalized.mana,
      normalized.race,
      normalized.threat,
    ]) {
      expect(Object.isFrozen(section)).toBe(true);
    }
    mutableProfile.material.activeMon = 77;
    expect(normalized.material.activeMon).toBe(5);

    const { automoveConfigFromPreference } =
      await import("../../src/automove/config/presets.js");
    const { defineAutomoveConfig, patchAutomoveConfig } =
      await import("../../src/automove/config/patch.js");
    const baseConfig = automoveConfigFromPreference("fast");
    expect(
      patchAutomoveConfig(baseConfig, {
        evaluation: { weights: custom },
      }).evaluation.weights,
    ).toBe(custom);
    expect(
      defineAutomoveConfig({
        ...baseConfig,
        evaluation: { ...baseConfig.evaluation, weights: custom },
      }).evaluation.weights,
    ).toBe(custom);
    for (const id of builtInProfileIds) {
      const message = `scoring profile id ${id} is already registered`;
      const hostile = conflictingProfile(id, base);
      expect(() =>
        patchAutomoveConfig(baseConfig, {
          evaluation: { weights: hostile },
        }),
      ).toThrow(message);
      expect(() =>
        defineAutomoveConfig({
          ...baseConfig,
          evaluation: { ...baseConfig.evaluation, weights: hostile },
        }),
      ).toThrow(message);
    }

    const presets = await import("../../src/automove/scoring/presets.js");
    const runtimeProfiles = await import("../../src/automove/config/profiles.js");
    for (const id of builtInProfileIds) {
      expectDirectConflict(id);
    }
    expect(presets.DEFAULT_SCORING_WEIGHTS.id).toBe("default");
    expect(Object.isFrozen(presets.DEFAULT_SCORING_WEIGHTS)).toBe(true);
    expect(validation.normalizeScoringProfile(presets.DEFAULT_SCORING_WEIGHTS)).toBe(
      presets.DEFAULT_SCORING_WEIGHTS,
    );
    expect(presets.BALANCED_DISTANCE_SCORING_WEIGHTS.id).toBe("balanced-distance");

    for (const [phase, whiteScore, blackScore] of [
      ["balanced", 0, 0],
      ["tactical", 0, 3],
      ["tactical-aggressive", 0, 4],
      ["finisher", 3, 0],
      ["finisher-aggressive", 4, 0],
    ] as const) {
      const game = {
        activeColor: Color.White,
        whiteScore,
        blackScore,
      } as MonsGame;
      const walk = runtimeProfiles.runtimePhaseAdaptiveWalkThreatMediumScoringProfile(
        game,
        3,
      );
      const attacker =
        runtimeProfiles.runtimePhaseAdaptiveAttackerProximityScoringProfile(game, 3);
      expect(walk).toBe(
        runtimeProfiles.runtimePhaseAdaptiveWalkThreatMediumScoringProfile(game, 3),
      );
      expect(attacker).toBe(
        runtimeProfiles.runtimePhaseAdaptiveAttackerProximityScoringProfile(game, 3),
      );
      expect(walk.weights.id).toBe(`runtime-normal-walk-${phase}`);
      expect(attacker.weights.id).toBe(`runtime-normal-attacker-${phase}`);
      expect(Object.isFrozen(walk)).toBe(true);
      expect(Object.isFrozen(attacker)).toBe(true);
      expect(Object.isFrozen(walk.weights)).toBe(true);
      expect(Object.isFrozen(attacker.weights)).toBe(true);
    }
  });
});
