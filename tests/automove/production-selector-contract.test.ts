import fs from "node:fs";
import path from "node:path";

import ts from "typescript";
import { describe, expect, it, vi } from "vitest";

import { GameVariant } from "../../src/engine/board/config.js";
import { Color } from "../../src/engine/model/domain.js";
import { parseGameFen } from "../../src/engine/codec/game-board.js";
import { MonsGame } from "../../src/engine/game/mons-game.js";
import { exactOpportunityContext } from "../../src/automove/exact/turn-opportunity.js";
import { ProductionHeadGuardId } from "../../src/automove/policy/production/head-guard-order.js";
import { evaluateInitialHeadGuards } from "../../src/automove/policy/production/head-initial-guards.js";
import type { TurnEngineHeadAcceptanceContext } from "../../src/automove/policy/production/head-types.js";
import { commitPlanAndSeedFollowup } from "../../src/automove/policy/production/plan-commit.js";
import { turnEngineConfigForGame } from "../../src/automove/policy/production/config.js";
import { rankRootCandidates } from "../../src/automove/root/candidates.js";
import { rootFamily } from "../../src/automove/root/family.js";
import type { EvaluatedRoot } from "../../src/automove/root/types.js";
import {
  automoveConfigForGame,
  withProductionPlanner,
} from "../../src/automove/config/runtime.js";
import { patchAutomoveConfig } from "../../src/automove/config/patch.js";
import {
  EMPTY_TURN_UTILITY,
  TurnEngineMode,
  TurnPlanFamily,
  type TurnPlan,
} from "../../src/automove/turn/model.js";
import { turnEngineCachedStep } from "../../src/automove/turn/planner-cache.js";
import { createTestAutomoveExecutionContext } from "./execution-context.test-helper.js";

const productionSourceRoot = path.resolve(
  import.meta.dirname,
  "../../src/automove/policy/production",
);

type RuntimeDecision = {
  readonly when: string;
  readonly outcome: string;
};

const expectedRuntimeControlFlowSkeleton = [
  "call:initial=evaluateInitialHeadGuards(context)",
  'if:expression:initial.kind === "reject"->false',
  "call:facts=deriveOrderedHeadFacts(context, initial.facts)",
  "if:expression:firstRejectedManaAndRecoveryHeadGuard(context, facts) !== undefined->false",
  "call:decision=evaluateProjectedHeadGuards(execution, context, facts)",
  'if:expression:decision.kind === "accept"->true',
  'if:expression:decision.kind === "reject"->false',
  "return:acceptTurnEngineHeadByModeAndFamily(context, decision.policy)",
];

const expectedGuardEvaluationStrategy = {
  initial: [
    "binding(context):eager",
    "narrowUnsafeBlackManaScore:eager",
    "earlySafeManaBlocksSpirit:lazy",
    "blackTurnStartSafeManaBlocksPlainSpirit:lazy",
    "whiteTurnStartSafeManaBlocksPlainSpirit:lazy",
    "whiteLateSafeManaBlocksPlainSpirit:lazy",
    "blackNoActionVulnerableProgressHead:lazy",
    "blackEarlySameWindowManaHead:lazy",
    "blackNoActionWindowedVulnerableManaHead:lazy",
    "blackLateWindowedVulnerableManaHead:lazy",
    "pickupUpgrade:eager:hasPickupUpgrade(candidate, selected)",
    "blackEarlyProgressBlocksMana:lazy",
    "blackEarlyProgressBlocksNonConcreteWindow:lazy",
  ],
  manaAndRecovery: [
    "binding(context):eager",
    "binding(facts):eager",
    "earlyBlackSafeManaBlocksWeakerMana:lazy",
    "blackQuietManaBlocksLowerScoredMana:lazy",
    "whiteSameWindowManaBlocksLowerScoredMana:lazy",
    "whiteMidTurnManaBlocksLowerScoredWindowMana:lazy",
    "whiteMidTurnSpiritSetupBlocksWindowMana:lazy",
    "whiteTurnStartSpiritSetupBlocksWindowMana:lazy",
    "whiteSafeManaBlocksDeferredRecoveryProgress:lazy",
    "blackLateSafeProgressBlocksQuietMana:lazy",
    "blackRecoveryRootBlocksNonConcreteWindow:lazy",
    "whiteRecoveryRootBlocksNonConcreteWindow:lazy",
    "vulnerableWhiteManaHead:lazy",
  ],
  projected: [
    "binding(context):eager",
    "binding(facts):eager",
    "projectedSafe:eager:projectedPlanIsSafelyCompleted(execution, game, perspective, config, plan)",
    "projectedReplyNotWorse:eager:compareUtilityPrimaryAxes(plan.utility, selectedUtility)",
    "projectedHeadNotWorse:eager:compareUtilityPrimaryAxes(plan.headUtility, selectedUtility)",
    "narrowWhiteManaOnlyProgressTie:eager",
    "projectedProgressRegressesSafePickup:eager",
    "projectedDeferredRecoveryWithoutConcreteGain:eager",
    "safeRootBlocksPlainSpirit:eager",
    "safeRootBlocksPlainSpiritProgress:eager:isPlainSpiritDevelopmentRoot(candidate)",
    "plainSpiritSiblingRegresses:eager:isPlainSpiritDevelopmentRoot(selected)|isPlainSpiritDevelopmentRoot(candidate)",
    "projectedOverride:eager:utilityPassesOverrideGuard(plan.utility, selectedUtility)|utilitySupportsPrimaryAxesEvalTolerance(plan.utility, selectedUtility, 96)|utilityImprovesNonScoreOverrideAxes(plan.utility, selectedUtility)",
    "candidateUnsafeWithoutProjectedOverride:lazy",
    "selectedUtilityDominatesPlan:lazy",
    "allowNonConcreteWhiteProgress:eager:productionIsEarlyWhiteTurnStart(game)|isPlainSpiritDevelopmentRoot(selected)|utilitySupportsPrimaryAxesEvalTolerance(plan.utility, selectedUtility, 64)",
    "nonProgressHeadWithoutOverride:lazy",
    "drainerKillWithoutAttack:lazy",
    "whiteSetupRecoveryBlocksUtilityOverride:eager:saturatingScoreAdd(candidate.spiritSetupGain, 48)",
    "whiteVulnerableProgressBlocksImmediateScore:lazy",
    "allowGenericProductionOverride:eager:productionSecondaryAnalysisLive(config)",
    "spiritHeadWithoutImpact:lazy",
  ],
};

function rejectedDecision(when: string, reason: string): RuntimeDecision {
  return { when, outcome: `reject:${reason}` };
}

const expectedRuntimeDecisions = {
  initial: [
    rejectedDecision(
      "call:earlySafeManaBlocksSpirit()",
      ProductionHeadGuardId.EarlySafeManaBlocksSpirit,
    ),
    rejectedDecision(
      "call:blackTurnStartSafeManaBlocksPlainSpirit()",
      ProductionHeadGuardId.BlackTurnStartSafeManaBlocksPlainSpirit,
    ),
    rejectedDecision(
      "call:whiteTurnStartSafeManaBlocksPlainSpirit()",
      ProductionHeadGuardId.WhiteTurnStartSafeManaBlocksPlainSpirit,
    ),
    rejectedDecision(
      "call:whiteLateSafeManaBlocksPlainSpirit()",
      ProductionHeadGuardId.WhiteLateSafeManaBlocksPlainSpirit,
    ),
    rejectedDecision(
      "call:blackNoActionVulnerableProgressHead()",
      ProductionHeadGuardId.BlackNoActionVulnerableProgressHead,
    ),
    rejectedDecision(
      "call:blackEarlySameWindowManaHead()",
      ProductionHeadGuardId.BlackEarlySameWindowManaHead,
    ),
    rejectedDecision(
      "call:blackNoActionWindowedVulnerableManaHead()",
      ProductionHeadGuardId.BlackNoActionWindowedVulnerableManaHead,
    ),
    rejectedDecision(
      "call:blackLateWindowedVulnerableManaHead()",
      ProductionHeadGuardId.BlackLateWindowedVulnerableManaHead,
    ),
    rejectedDecision(
      "call:blackEarlyProgressBlocksMana()",
      ProductionHeadGuardId.BlackEarlyProgressBlocksMana,
    ),
    rejectedDecision(
      "call:blackEarlyProgressBlocksNonConcreteWindow()",
      ProductionHeadGuardId.BlackEarlyProgressBlocksNonConcreteWindow,
    ),
    { when: "otherwise", outcome: "continue" },
  ],
  manaAndRecovery: [
    rejectedDecision(
      "call:earlyBlackSafeManaBlocksWeakerMana()",
      ProductionHeadGuardId.EarlyBlackSafeManaBlocksWeakerMana,
    ),
    rejectedDecision(
      "call:blackQuietManaBlocksLowerScoredMana()",
      ProductionHeadGuardId.BlackQuietManaBlocksLowerScoredMana,
    ),
    rejectedDecision(
      "call:whiteSameWindowManaBlocksLowerScoredMana()",
      ProductionHeadGuardId.WhiteSameWindowManaBlocksLowerScoredMana,
    ),
    rejectedDecision(
      "call:whiteMidTurnManaBlocksLowerScoredWindowMana()",
      ProductionHeadGuardId.WhiteMidTurnManaBlocksLowerScoredWindowMana,
    ),
    rejectedDecision(
      "call:whiteMidTurnSpiritSetupBlocksWindowMana()",
      ProductionHeadGuardId.WhiteMidTurnSpiritSetupBlocksWindowMana,
    ),
    rejectedDecision(
      "call:whiteTurnStartSpiritSetupBlocksWindowMana()",
      ProductionHeadGuardId.WhiteTurnStartSpiritSetupBlocksWindowMana,
    ),
    rejectedDecision(
      "call:whiteSafeManaBlocksDeferredRecoveryProgress()",
      ProductionHeadGuardId.WhiteSafeManaBlocksDeferredRecoveryProgress,
    ),
    rejectedDecision(
      "call:vulnerableWhiteManaHead()",
      ProductionHeadGuardId.VulnerableWhiteManaHead,
    ),
    rejectedDecision(
      "call:blackLateSafeProgressBlocksQuietMana()",
      ProductionHeadGuardId.BlackLateSafeProgressBlocksQuietMana,
    ),
    rejectedDecision(
      "call:blackRecoveryRootBlocksNonConcreteWindow()",
      ProductionHeadGuardId.BlackRecoveryRootBlocksNonConcreteWindow,
    ),
    rejectedDecision(
      "call:whiteRecoveryRootBlocksNonConcreteWindow()",
      ProductionHeadGuardId.WhiteRecoveryRootBlocksNonConcreteWindow,
    ),
    { when: "otherwise", outcome: "continue" },
  ],
  projected: [
    rejectedDecision(
      "call:candidateUnsafeWithoutProjectedOverride()",
      ProductionHeadGuardId.CandidateUnsafeWithoutProjectedOverride,
    ),
    rejectedDecision(
      "call:selectedUtilityDominatesPlan()",
      ProductionHeadGuardId.SelectedUtilityDominatesPlan,
    ),
    { when: "value:whiteSpiritSetupGain", outcome: "accept" },
    rejectedDecision(
      "call:nonProgressHeadWithoutOverride()",
      ProductionHeadGuardId.NonProgressHeadWithoutOverride,
    ),
    rejectedDecision(
      "call:drainerKillWithoutAttack()",
      ProductionHeadGuardId.DrainerKillWithoutAttack,
    ),
    rejectedDecision(
      "call:whiteVulnerableProgressBlocksImmediateScore()",
      ProductionHeadGuardId.WhiteVulnerableProgressBlocksImmediateScore,
    ),
    {
      when: "expression:macroMode && !selected.winsImmediately && allowGenericProductionOverride && utilityPassesOverrideGuard(plan.utility, selectedUtility) && !whiteSetupRecoveryBlocksUtilityOverride && (!candidateUnsafe || selectedUnsafe)",
      outcome: "accept",
    },
    rejectedDecision(
      "call:spiritHeadWithoutImpact()",
      ProductionHeadGuardId.SpiritHeadWithoutImpact,
    ),
    { when: "value:projectedOverride", outcome: "accept" },
    { when: "otherwise", outcome: "delegate" },
  ],
} satisfies Record<string, readonly RuntimeDecision[]>;

const expectedGuardStatementTrace = {
  initial: expectedGuardStatementTraceFromLayout(
    expectedGuardEvaluationStrategy.initial,
    expectedRuntimeDecisions.initial,
    [
      ["declaration", 10],
      ["decision", 8],
      ["declaration", 3],
      ["decision", 3],
    ],
  ),
  manaAndRecovery: expectedGuardStatementTraceFromLayout(
    expectedGuardEvaluationStrategy.manaAndRecovery,
    expectedRuntimeDecisions.manaAndRecovery,
    [
      ["declaration", 13],
      ["decision", 12],
    ],
  ),
  projected: expectedGuardStatementTraceFromLayout(
    expectedGuardEvaluationStrategy.projected,
    expectedRuntimeDecisions.projected,
    [
      ["declaration", 13],
      ["decision", 1],
      ["declaration", 1],
      ["decision", 1],
      ["declaration", 1],
      ["decision", 1],
      ["declaration", 1],
      ["decision", 1],
      ["declaration", 1],
      ["decision", 1],
      ["declaration", 2],
      ["decision", 1],
      ["declaration", 1],
      ["decision", 1],
      ["declaration", 1],
      ["decision", 3],
    ],
  ),
};

function parsedFunction(filePath: string, sourceText: string, functionName: string) {
  const source = ts.createSourceFile(
    filePath,
    sourceText,
    ts.ScriptTarget.Latest,
    true,
    ts.ScriptKind.TS,
  );
  const declaration = source.statements.find(
    (statement): statement is ts.FunctionDeclaration =>
      ts.isFunctionDeclaration(statement) && statement.name?.text === functionName,
  );
  if (declaration?.body === undefined) {
    throw new Error(`missing production function ${functionName}`);
  }
  return { body: declaration.body, declaration, source };
}

function productionFunction(fileName: string, functionName: string) {
  const filePath = path.join(productionSourceRoot, fileName);
  return parsedFunction(filePath, fs.readFileSync(filePath, "utf8"), functionName);
}

function directDecisionReturn(statement: ts.Statement): ts.ReturnStatement {
  if (ts.isReturnStatement(statement)) return statement;
  if (ts.isBlock(statement) && statement.statements.length === 1) {
    const [onlyStatement] = statement.statements;
    if (onlyStatement !== undefined && ts.isReturnStatement(onlyStatement)) {
      return onlyStatement;
    }
  }
  throw new Error(
    `unsupported nested production decision exit: ${statement.getText()}`,
  );
}

function propertyAssignment(
  value: ts.ObjectLiteralExpression,
  name: string,
): ts.PropertyAssignment | undefined {
  return value.properties.find(
    (property): property is ts.PropertyAssignment =>
      ts.isPropertyAssignment(property) &&
      ((ts.isIdentifier(property.name) && property.name.text === name) ||
        (ts.isStringLiteral(property.name) && property.name.text === name)),
  );
}

function productionHeadGuardReason(name: string): string {
  const reason = (ProductionHeadGuardId as Readonly<Record<string, string>>)[name];
  if (reason === undefined) {
    throw new Error(`unknown production rejection reason: ${name}`);
  }
  return reason;
}

function decisionOutcome(expression: ts.Expression | undefined): string {
  if (
    expression === undefined ||
    (ts.isIdentifier(expression) && expression.text === "undefined")
  ) {
    return "continue";
  }
  if (ts.isPropertyAccessExpression(expression)) {
    return `reject:${productionHeadGuardReason(expression.name.text)}`;
  }
  if (!ts.isObjectLiteralExpression(expression)) {
    throw new Error(`unsupported production decision: ${expression.getText()}`);
  }
  const kind = propertyAssignment(expression, "kind")?.initializer;
  if (kind === undefined || !ts.isStringLiteral(kind)) {
    throw new Error(`production decision has no literal kind: ${expression.getText()}`);
  }
  if (kind.text !== "reject") return kind.text;
  const reason = propertyAssignment(expression, "reason")?.initializer;
  if (reason === undefined || !ts.isPropertyAccessExpression(reason)) {
    throw new Error(`production rejection has no reason: ${expression.getText()}`);
  }
  return `reject:${productionHeadGuardReason(reason.name.text)}`;
}

function normalizedExpression(
  expression: ts.Expression,
  source: ts.SourceFile,
): string {
  return expression.getText(source).replace(/\s+/gu, " ");
}

function directCallText(
  expression: ts.Expression | undefined,
  source: ts.SourceFile,
): string {
  if (
    expression === undefined ||
    !ts.isCallExpression(expression) ||
    !ts.isIdentifier(expression.expression)
  ) {
    throw new Error(`unsupported production phase call: ${expression?.getText()}`);
  }
  return normalizedExpression(expression, source);
}

function guardEvaluationStrategy(
  statements: readonly ts.Statement[],
  source: ts.SourceFile,
): string[] {
  return statements.flatMap((statement): string[] => {
    if (!ts.isVariableStatement(statement)) return [];
    return statement.declarationList.declarations.map((declaration) => {
      const initializer = declaration.initializer;
      if (initializer === undefined) {
        throw new Error(
          `guard declaration has no initializer: ${declaration.getText()}`,
        );
      }
      const name = ts.isIdentifier(declaration.name)
        ? declaration.name.text
        : `binding(${normalizedExpression(initializer, source)})`;
      if (ts.isArrowFunction(initializer) || ts.isFunctionExpression(initializer)) {
        return `${name}:lazy`;
      }
      const calls: string[] = [];
      const visit = (node: ts.Node): void => {
        if (node !== initializer && ts.isFunctionLike(node)) return;
        if (ts.isCallExpression(node) && ts.isIdentifier(node.expression)) {
          calls.push(normalizedExpression(node, source));
        }
        ts.forEachChild(node, visit);
      };
      visit(initializer);
      return `${name}:eager${calls.length === 0 ? "" : `:${calls.join("|")}`}`;
    });
  });
}

function runtimeGuardEvaluationStrategy(
  fileName: string,
  functionName: string,
): string[] {
  const { body, source } = productionFunction(fileName, functionName);
  return guardEvaluationStrategy(body.statements, source);
}

function conditionLabel(condition: ts.Expression, source: ts.SourceFile): string {
  if (ts.isIdentifier(condition)) return `value:${condition.text}`;
  if (ts.isCallExpression(condition) && ts.isIdentifier(condition.expression)) {
    return `call:${directCallText(condition, source)}`;
  }
  return `expression:${normalizedExpression(condition, source)}`;
}

function decisionsFromStatements(
  statements: readonly ts.Statement[],
  source: ts.SourceFile,
): RuntimeDecision[] {
  return statements.flatMap((statement): RuntimeDecision[] => {
    if (ts.isVariableStatement(statement)) return [];
    if (ts.isIfStatement(statement)) {
      if (statement.elseStatement !== undefined) {
        throw new Error(`unsupported production decision else: ${statement.getText()}`);
      }
      const returned = directDecisionReturn(statement.thenStatement);
      return [
        {
          when: conditionLabel(statement.expression, source),
          outcome: decisionOutcome(returned.expression),
        },
      ];
    }
    if (ts.isReturnStatement(statement)) {
      return [{ when: "otherwise", outcome: decisionOutcome(statement.expression) }];
    }
    throw new Error(
      `unsupported top-level production decision: ${statement.getText()}`,
    );
  });
}

function runtimeDecisions(fileName: string, functionName: string): RuntimeDecision[] {
  const { body, source } = productionFunction(fileName, functionName);
  return decisionsFromStatements(body.statements, source);
}

type GuardTraceSection = readonly [kind: "declaration" | "decision", count: number];

function expectedGuardStatementTraceFromLayout(
  declarations: readonly string[],
  decisions: readonly RuntimeDecision[],
  layout: readonly GuardTraceSection[],
): string[] {
  let declarationIndex = 0;
  let decisionIndex = 0;
  const trace = layout.flatMap(([kind, count]): string[] => {
    if (kind === "declaration") {
      const entries = declarations.slice(declarationIndex, declarationIndex + count);
      declarationIndex += count;
      return entries.map((entry) => `declaration:${entry}`);
    }
    const entries = decisions.slice(decisionIndex, decisionIndex + count);
    decisionIndex += count;
    return entries.map(({ when, outcome }) => `decision:${when}->${outcome}`);
  });
  if (declarationIndex !== declarations.length || decisionIndex !== decisions.length) {
    throw new Error("guard statement trace layout is incomplete");
  }
  return trace;
}

function guardStatementTrace(
  statements: readonly ts.Statement[],
  source: ts.SourceFile,
): string[] {
  return statements.flatMap((statement): string[] => {
    if (ts.isVariableStatement(statement)) {
      return guardEvaluationStrategy([statement], source).map(
        (entry) => `declaration:${entry}`,
      );
    }
    if (ts.isIfStatement(statement) || ts.isReturnStatement(statement)) {
      return decisionsFromStatements([statement], source).map(
        ({ when, outcome }) => `decision:${when}->${outcome}`,
      );
    }
    throw new Error(
      `unsupported top-level guard statement: ${statement.getText(source)}`,
    );
  });
}

function runtimeGuardStatementTrace(fileName: string, functionName: string): string[] {
  const { body, source } = productionFunction(fileName, functionName);
  return guardStatementTrace(body.statements, source);
}

function orderedHeadControlFlow(
  statements: readonly ts.Statement[],
  source: ts.SourceFile,
): string[] {
  return statements.flatMap((statement): string[] => {
    if (ts.isVariableStatement(statement)) {
      if (statement.declarationList.declarations.length !== 1) {
        throw new Error(
          `unsupported production phase declaration: ${statement.getText()}`,
        );
      }
      const declaration = statement.declarationList.declarations[0];
      if (declaration === undefined || !ts.isIdentifier(declaration.name)) {
        throw new Error(`unsupported production phase binding: ${statement.getText()}`);
      }
      return [
        `call:${declaration.name.text}=${directCallText(declaration.initializer, source)}`,
      ];
    }
    if (ts.isIfStatement(statement)) {
      if (statement.elseStatement !== undefined) {
        throw new Error(`unsupported production phase else: ${statement.getText()}`);
      }
      const returned = directDecisionReturn(statement.thenStatement);
      if (returned.expression === undefined) {
        throw new Error(`production phase exit has no value: ${statement.getText()}`);
      }
      return [
        `if:${conditionLabel(statement.expression, source)}->${returned.expression.getText(source)}`,
      ];
    }
    if (ts.isReturnStatement(statement)) {
      return [`return:${directCallText(statement.expression, source)}`];
    }
    throw new Error(`unsupported top-level production phase: ${statement.getText()}`);
  });
}

function runtimeControlFlowSkeleton(): string[] {
  const { body, source } = productionFunction(
    "head-ordered-guards.ts",
    "acceptTurnEngineHeadAfterOrderedGuards",
  );
  return orderedHeadControlFlow(body.statements, source);
}

function gameAtTurn(turnNumber: number): MonsGame {
  const initial = new MonsGame(false, GameVariant.Classic);
  const state = parseGameFen(initial.fen());
  if (state === undefined) throw new Error("initial game must parse");
  return MonsGame.newSimulationState({ ...state, turnNumber });
}

function neutralRoot(root: EvaluatedRoot): EvaluatedRoot {
  return {
    ...root,
    score: 0,
    efficiency: 0,
    winsImmediately: false,
    attacksOpponentDrainer: false,
    ownDrainerVulnerable: false,
    ownDrainerWalkVulnerable: false,
    spiritDevelopment: false,
    keepsAwakeSpiritOnBase: false,
    manaHandoffToOpponent: false,
    hasRoundtrip: false,
    scoresSupermanaThisTurn: false,
    scoresOpponentManaThisTurn: false,
    safeSupermanaPickupNow: false,
    safeOpponentManaPickupNow: false,
    safeSupermanaProgressSteps: 99,
    safeOpponentManaProgressSteps: 99,
    scorePathBestSteps: 99,
    sameTurnScoreWindowValue: 0,
    spiritSetupGain: 0,
    spiritSameTurnScoreSetupNow: false,
    spiritOwnManaSetupNow: false,
    supermanaProgress: false,
    opponentManaProgress: false,
    policyPriority: 0,
    classes: {
      immediateScore: false,
      drainerAttack: false,
      drainerSafetyRecover: false,
      carrierProgress: false,
      material: false,
      quiet: true,
    },
  };
}

function initialSafeManaGuardContext(): TurnEngineHeadAcceptanceContext {
  const execution = createTestAutomoveExecutionContext();
  const game = gameAtTurn(3);
  const config = patchAutomoveConfig(
    withProductionPlanner(automoveConfigForGame(game, "pro")),
    {
      planner: {
        secondaryAnalysis: false,
        selectedFollowupProjection: false,
      },
    },
  );
  const roots = rankRootCandidates(execution, game, game.activeColor, config).slice(
    0,
    2,
  );
  const candidateRoot = roots[1];
  const selectedRoot = roots[0];
  if (candidateRoot === undefined || selectedRoot === undefined) {
    throw new Error("production guard contract requires two roots");
  }
  const candidate: EvaluatedRoot = {
    ...neutralRoot({ ...candidateRoot, score: 0, nodesAfter: 0 }),
    spiritDevelopment: true,
    spiritOwnManaSetupNow: true,
  };
  const selected = neutralRoot({ ...selectedRoot, score: 0, nodesAfter: 0 });
  const plan: TurnPlan = {
    actions: [],
    compiledChunks: [candidate.inputs],
    endGame: candidate.game,
    utility: EMPTY_TURN_UTILITY,
    headUtility: EMPTY_TURN_UTILITY,
    headFamily: TurnPlanFamily.SpiritImpact,
    goalFamily: TurnPlanFamily.SpiritImpact,
    packageMeta: {
      scoreGain: 0,
      denyGain: 0,
      drainerSafetyDelta: 0,
      spiritOnlySetup: false,
      endsNonnegativeDrainerSafety: true,
      opponentImmediateWindowAfter: 0,
    },
  };
  return {
    game,
    perspective: Color.White,
    config,
    plan,
    candidateIndex: 1,
    candidate,
    selected,
    macroMode: true,
    candidateUnsafe: false,
    selectedUnsafe: false,
    candidateProgress: false,
    selectedProgress: false,
    exactContext: exactOpportunityContext(execution, game, game.activeColor),
    scoreGap: 0,
    sameTurnWindowBetter: false,
    drainerAttackBetter: false,
    scoresNowBetter: false,
    safetyRecoverBetter: false,
    spiritWindowBetter: false,
    spiritDevelopmentBetter: true,
    candidateSpiritTactical: false,
    progressBetter: false,
    selectedSpiritPhase: false,
    candidateFamily: rootFamily(candidate),
    selectedFamily: rootFamily(selected),
    selectedUtilityValue: vi.fn(() => EMPTY_TURN_UTILITY),
    candidateUtilityValue: vi.fn(() => EMPTY_TURN_UTILITY),
    blackSpiritPair: false,
    whiteSpiritSetupGain: false,
    blackTurnSixRouteChangePlainSpirit: false,
  };
}

describe("production selector contracts", () => {
  it("pins actual head evaluator phase and decision precedence", () => {
    expect(runtimeControlFlowSkeleton()).toEqual(expectedRuntimeControlFlowSkeleton);
    expect(
      runtimeGuardEvaluationStrategy(
        "head-initial-guards.ts",
        "evaluateInitialHeadGuards",
      ),
    ).toEqual(expectedGuardEvaluationStrategy.initial);
    expect(
      runtimeGuardEvaluationStrategy(
        "head-mana-recovery-guards.ts",
        "firstRejectedManaAndRecoveryHeadGuard",
      ),
    ).toEqual(expectedGuardEvaluationStrategy.manaAndRecovery);
    expect(
      runtimeGuardEvaluationStrategy(
        "head-projected-guards.ts",
        "evaluateProjectedHeadGuards",
      ),
    ).toEqual(expectedGuardEvaluationStrategy.projected);
    expect(
      runtimeDecisions("head-initial-guards.ts", "evaluateInitialHeadGuards"),
    ).toEqual(expectedRuntimeDecisions.initial);
    expect(
      runtimeDecisions(
        "head-mana-recovery-guards.ts",
        "firstRejectedManaAndRecoveryHeadGuard",
      ),
    ).toEqual(expectedRuntimeDecisions.manaAndRecovery);
    expect(
      runtimeDecisions("head-projected-guards.ts", "evaluateProjectedHeadGuards"),
    ).toEqual(expectedRuntimeDecisions.projected);
    expect(
      runtimeGuardStatementTrace("head-initial-guards.ts", "evaluateInitialHeadGuards"),
    ).toEqual(expectedGuardStatementTrace.initial);
    expect(
      runtimeGuardStatementTrace(
        "head-mana-recovery-guards.ts",
        "firstRejectedManaAndRecoveryHeadGuard",
      ),
    ).toEqual(expectedGuardStatementTrace.manaAndRecovery);
    expect(
      runtimeGuardStatementTrace(
        "head-projected-guards.ts",
        "evaluateProjectedHeadGuards",
      ),
    ).toEqual(expectedGuardStatementTrace.projected);
    expect(Object.isFrozen(ProductionHeadGuardId)).toBe(true);
  });

  it("fails closed for reordered production phases and nested exits", () => {
    const { body, source } = productionFunction(
      "head-ordered-guards.ts",
      "acceptTurnEngineHeadAfterOrderedGuards",
    );
    const statements = [...body.statements];
    const initialExit = statements[1];
    const factDerivation = statements[2];
    if (initialExit === undefined || factDerivation === undefined) {
      throw new Error("production phase skeleton is incomplete");
    }
    statements[1] = factDerivation;
    statements[2] = initialExit;
    expect(orderedHeadControlFlow(statements, source)).not.toEqual(
      expectedRuntimeControlFlowSkeleton,
    );

    const nested = parsedFunction(
      "nested-production-phase.ts",
      `function accept() {
        const initial = evaluateInitialHeadGuards(context);
        if (initial.kind === "reject") {
          if (initial.reason !== undefined) return false;
        }
        return acceptTurnEngineHeadByModeAndFamily(context, initial.policy);
      }`,
      "accept",
    );
    expect(() => orderedHeadControlFlow(nested.body.statements, nested.source)).toThrow(
      "unsupported nested production decision exit",
    );
  });

  it("detects changed phase arguments and guard evaluation strategy", () => {
    const orderedFilePath = path.join(productionSourceRoot, "head-ordered-guards.ts");
    const orderedSourceText = fs.readFileSync(orderedFilePath, "utf8");
    const changedArgumentsText = orderedSourceText.replace(
      "deriveOrderedHeadFacts(context, initial.facts)",
      "deriveOrderedHeadFacts(context, { ...initial.facts, pickupUpgrade: false })",
    );
    expect(changedArgumentsText).not.toBe(orderedSourceText);
    const changedArguments = parsedFunction(
      orderedFilePath,
      changedArgumentsText,
      "acceptTurnEngineHeadAfterOrderedGuards",
    );
    expect(
      orderedHeadControlFlow(changedArguments.body.statements, changedArguments.source),
    ).not.toEqual(expectedRuntimeControlFlowSkeleton);

    const initialFilePath = path.join(productionSourceRoot, "head-initial-guards.ts");
    const initialSourceText = fs.readFileSync(initialFilePath, "utf8");
    const eagerGuardText = initialSourceText.replace(
      "if (earlySafeManaBlocksSpirit()) {",
      "if (earlySafeManaBlocksSpirit) {",
    );
    expect(eagerGuardText).not.toBe(initialSourceText);
    const eagerGuard = parsedFunction(
      initialFilePath,
      eagerGuardText,
      "evaluateInitialHeadGuards",
    );
    const eagerDecisions = decisionsFromStatements(
      eagerGuard.body.statements,
      eagerGuard.source,
    );
    expect(eagerDecisions[0]?.when).toBe("value:earlySafeManaBlocksSpirit");
    expect(eagerDecisions).not.toEqual(
      runtimeDecisions("head-initial-guards.ts", "evaluateInitialHeadGuards"),
    );

    const initial = parsedFunction(
      initialFilePath,
      initialSourceText,
      "evaluateInitialHeadGuards",
    );
    const movedEagerStatements = [...initial.body.statements];
    const pickupUpgradeIndex = movedEagerStatements.findIndex(
      (statement) =>
        ts.isVariableStatement(statement) &&
        statement.declarationList.declarations.some(
          (declaration) =>
            ts.isIdentifier(declaration.name) &&
            declaration.name.text === "pickupUpgrade",
        ),
    );
    const firstExitIndex = movedEagerStatements.findIndex(ts.isIfStatement);
    if (pickupUpgradeIndex < 0 || firstExitIndex < 0) {
      throw new Error("initial guard statement trace is incomplete");
    }
    const [pickupUpgrade] = movedEagerStatements.splice(pickupUpgradeIndex, 1);
    if (pickupUpgrade === undefined) {
      throw new Error("pickup upgrade declaration is missing");
    }
    movedEagerStatements.splice(firstExitIndex, 0, pickupUpgrade);
    expect(guardEvaluationStrategy(movedEagerStatements, initial.source)).toEqual(
      expectedGuardEvaluationStrategy.initial,
    );
    expect(decisionsFromStatements(movedEagerStatements, initial.source)).toEqual(
      expectedRuntimeDecisions.initial,
    );
    expect(guardStatementTrace(movedEagerStatements, initial.source)).not.toEqual(
      expectedGuardStatementTrace.initial,
    );

    const projectedFilePath = path.join(
      productionSourceRoot,
      "head-projected-guards.ts",
    );
    const projectedSourceText = fs.readFileSync(projectedFilePath, "utf8");
    const projected = parsedFunction(
      projectedFilePath,
      projectedSourceText,
      "evaluateProjectedHeadGuards",
    );
    const movedStatements = [...projected.body.statements];
    const declarationIndex = (name: string): number =>
      movedStatements.findIndex(
        (statement) =>
          ts.isVariableStatement(statement) &&
          statement.declarationList.declarations.some(
            (declaration) =>
              ts.isIdentifier(declaration.name) && declaration.name.text === name,
          ),
      );
    const projectedSafeIndex = declarationIndex("projectedSafe");
    const projectedReplyIndex = declarationIndex("projectedReplyNotWorse");
    if (projectedSafeIndex < 0 || projectedReplyIndex < 0) {
      throw new Error("projected guard strategy declarations are missing");
    }
    const projectedSafe = movedStatements[projectedSafeIndex];
    const projectedReply = movedStatements[projectedReplyIndex];
    if (projectedSafe === undefined || projectedReply === undefined) {
      throw new Error("projected guard strategy declarations are incomplete");
    }
    movedStatements[projectedSafeIndex] = projectedReply;
    movedStatements[projectedReplyIndex] = projectedSafe;
    expect(guardEvaluationStrategy(movedStatements, projected.source)).not.toEqual(
      expectedGuardEvaluationStrategy.projected,
    );

    const eagerStrategyText = projectedSourceText
      .replace(
        `const candidateUnsafeWithoutProjectedOverride = (): boolean =>
    candidateUnsafe && !selectedUnsafe && !projectedOverride;`,
        `const candidateUnsafeWithoutProjectedOverride =
    candidateUnsafe && !selectedUnsafe && !projectedOverride;`,
      )
      .replace(
        "if (candidateUnsafeWithoutProjectedOverride()) {",
        "if (candidateUnsafeWithoutProjectedOverride) {",
      );
    expect(eagerStrategyText).not.toBe(projectedSourceText);
    const eagerStrategy = parsedFunction(
      projectedFilePath,
      eagerStrategyText,
      "evaluateProjectedHeadGuards",
    );
    expect(
      guardEvaluationStrategy(eagerStrategy.body.statements, eagerStrategy.source),
    ).not.toEqual(expectedGuardEvaluationStrategy.projected);
  });

  it("reports the first ordered rejection reason without evaluating later utility", () => {
    const context = initialSafeManaGuardContext();

    expect(evaluateInitialHeadGuards(context)).toEqual({
      kind: "reject",
      reason: ProductionHeadGuardId.EarlySafeManaBlocksSpirit,
    });
    expect(context.selectedUtilityValue).not.toHaveBeenCalled();
    expect(context.candidateUtilityValue).not.toHaveBeenCalled();
  });

  it("writes a committed head before reuse and never writes after timeout", () => {
    const game = new MonsGame(false, GameVariant.Classic);
    const config = withProductionPlanner(automoveConfigForGame(game, "pro"));
    const sourceExecution = createTestAutomoveExecutionContext();
    const candidate = rankRootCandidates(
      sourceExecution,
      game,
      game.activeColor,
      config,
    )[0];
    if (candidate === undefined) throw new Error("initial root must exist");
    const plan: TurnPlan = {
      actions: [],
      compiledChunks: [candidate.inputs],
      endGame: candidate.game,
      utility: EMPTY_TURN_UTILITY,
      headUtility: EMPTY_TURN_UTILITY,
      headFamily: rootFamily(candidate),
      goalFamily: rootFamily(candidate),
      packageMeta: {
        scoreGain: 0,
        denyGain: 0,
        drainerSafetyDelta: 0,
        spiritOnlySetup: false,
        endsNonnegativeDrainerSafety: true,
        opponentImmediateWindowAfter: 0,
      },
    };
    const engineConfig = turnEngineConfigForGame(game, config);
    const execution = createTestAutomoveExecutionContext();

    commitPlanAndSeedFollowup(
      execution,
      game,
      game.activeColor,
      config,
      TurnEngineMode.Production,
      plan,
      engineConfig,
    );

    expect(turnEngineCachedStep(execution, game, engineConfig)).toEqual(
      candidate.inputs,
    );

    const timedOutExecution = createTestAutomoveExecutionContext();
    timedOutExecution.session.withDeadlineIfAbsent(0, () => {
      commitPlanAndSeedFollowup(
        timedOutExecution,
        game,
        game.activeColor,
        config,
        TurnEngineMode.Production,
        plan,
        engineConfig,
      );
    });
    expect(timedOutExecution.caches.engine.entryCount).toBe(0);
  });
});
