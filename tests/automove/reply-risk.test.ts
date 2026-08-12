import fs from "node:fs";
import path from "node:path";

import ts from "typescript";
import { describe, expect, it } from "vitest";

import {
  clearReplyRiskCache,
  selectedOverrideConfigKey,
} from "../../src/automove/policy/reply-risk/cache.js";
import { isBetterReplyRiskCandidate } from "../../src/automove/policy/reply-risk/arbitration.js";
import {
  contextualReplyRiskDecision,
  immediateReplyRiskDecision,
} from "../../src/automove/policy/reply-risk/arbitration-primary.js";
import {
  plainSpiritReplyRiskDecision,
  sharedSpiritFollowupDecision,
} from "../../src/automove/policy/reply-risk/arbitration-spirit.js";
import { finalReplyRiskDecision } from "../../src/automove/policy/reply-risk/arbitration-tiebreak.js";
import { rootReplyRiskSnapshot } from "../../src/automove/policy/reply-risk/snapshot.js";
import type { RootReplyRiskSnapshot } from "../../src/automove/policy/reply-risk/types.js";
import {
  automoveConfigForGame,
  withProductionPlanner,
} from "../../src/automove/config/runtime.js";
import { hash64 } from "../../src/automove/core/hash64.js";
import {
  UNKNOWN_PROGRESS_STEPS,
  UNKNOWN_SCORE_PATH_STEPS,
  type EvaluatedRoot,
  type MoveClassFlags,
} from "../../src/automove/root/types.js";
import { DEFAULT_SCORING_WEIGHTS } from "../../src/automove/scoring/presets.js";
import {
  TurnEngineMode,
  type TurnEngineConfig,
} from "../../src/automove/turn/model.js";
import { GameVariant } from "../../src/engine/board/config.js";
import { Color } from "../../src/engine/model/domain.js";
import { MonsGame } from "../../src/engine/game/mons-game.js";
import { createTestAutomoveExecutionContext } from "./execution-context.test-helper.js";

const replyRiskArbitrationPath = path.resolve(
  import.meta.dirname,
  "../../src/automove/policy/reply-risk/arbitration.ts",
);

function compactSource(node: ts.Node, source: ts.SourceFile): string {
  return node.getText(source).replace(/\s+/g, "").replace(/,\)/g, ")");
}

function returnedExpression(statement: ts.Statement): ts.Expression | undefined {
  if (ts.isReturnStatement(statement)) return statement.expression;
  if (!ts.isBlock(statement) || statement.statements.length !== 1) return undefined;
  const [onlyStatement] = statement.statements;
  return onlyStatement !== undefined && ts.isReturnStatement(onlyStatement)
    ? onlyStatement.expression
    : undefined;
}

function replyRiskArbitrationPhaseTrace(
  sourceText = fs.readFileSync(replyRiskArbitrationPath, "utf8"),
): string[] {
  const source = ts.createSourceFile(
    replyRiskArbitrationPath,
    sourceText,
    ts.ScriptTarget.Latest,
    true,
    ts.ScriptKind.TS,
  );
  const arbitrations = source.statements.filter(
    (statement): statement is ts.FunctionDeclaration =>
      ts.isFunctionDeclaration(statement) &&
      statement.name?.text === "isBetterReplyRiskCandidate",
  );
  const [arbitration] = arbitrations;
  if (arbitrations.length !== 1) {
    throw new Error("expected exactly one isBetterReplyRiskCandidate declaration");
  }
  if (arbitration?.body === undefined) {
    throw new Error("isBetterReplyRiskCandidate body not found");
  }

  const trace: string[] = [];
  for (const statement of arbitration.body.statements) {
    if (ts.isVariableStatement(statement)) {
      const [declaration] = statement.declarationList.declarations;
      if (
        statement.declarationList.declarations.length !== 1 ||
        declaration === undefined ||
        !ts.isIdentifier(declaration.name) ||
        declaration.initializer === undefined
      ) {
        throw new Error("unsupported arbitration variable declaration");
      }
      trace.push(
        `bind:${declaration.name.text}=${compactSource(declaration.initializer, source)}`,
      );
      continue;
    }
    if (ts.isIfStatement(statement)) {
      const returned = returnedExpression(statement.thenStatement);
      if (statement.elseStatement !== undefined || returned === undefined) {
        throw new Error("unsupported arbitration conditional");
      }
      trace.push(
        `guard:${compactSource(statement.expression, source)}=>return:${compactSource(returned, source)}`,
      );
      continue;
    }
    if (ts.isReturnStatement(statement) && statement.expression !== undefined) {
      trace.push(`return:${compactSource(statement.expression, source)}`);
      continue;
    }
    throw new Error(
      `unsupported arbitration statement: ${ts.SyntaxKind[statement.kind]}`,
    );
  }
  return trace;
}

const QUIET_ROOT_CLASSES: MoveClassFlags = Object.freeze({
  immediateScore: false,
  drainerAttack: false,
  drainerSafetyRecover: false,
  carrierProgress: false,
  material: false,
  quiet: true,
});

type ReplyRiskRootOverrides = Omit<Partial<EvaluatedRoot>, "classes"> & {
  readonly classes?: Partial<MoveClassFlags>;
};

function replyRiskRoot(overrides: ReplyRiskRootOverrides = {}): EvaluatedRoot {
  const { classes, ...rootOverrides } = overrides;
  return {
    rootRank: 0,
    inputs: [],
    game: new MonsGame(false, GameVariant.Classic),
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
    safeSupermanaProgressSteps: UNKNOWN_PROGRESS_STEPS,
    safeOpponentManaProgressSteps: UNKNOWN_PROGRESS_STEPS,
    scorePathBestSteps: UNKNOWN_SCORE_PATH_STEPS,
    sameTurnScoreWindowValue: 0,
    spiritSetupGain: 0,
    spiritSameTurnScoreSetupNow: false,
    spiritOwnManaSetupNow: false,
    supermanaProgress: false,
    opponentManaProgress: false,
    policyPriority: 0,
    classes: { ...QUIET_ROOT_CLASSES, ...classes },
    heuristic: 0,
    events: [],
    stateHash: hash64(0, 0),
    score: 0,
    nodesAfter: 0,
    ...rootOverrides,
  };
}

const safeReplyRiskSnapshot: RootReplyRiskSnapshot = Object.freeze({
  allowsImmediateOpponentWin: false,
  opponentReachesMatchPoint: false,
  worstReplyScore: 0,
});

const unsafeReplyRiskSnapshot: RootReplyRiskSnapshot = Object.freeze({
  ...safeReplyRiskSnapshot,
  allowsImmediateOpponentWin: true,
});

describe("reply-risk execution caches", () => {
  it("reuses snapshots only within the owning search session", () => {
    const firstExecution = createTestAutomoveExecutionContext();
    const secondExecution = createTestAutomoveExecutionContext();
    const game = new MonsGame(false, GameVariant.Classic);
    const config = automoveConfigForGame(game, "fast");

    const first = rootReplyRiskSnapshot(firstExecution, game, Color.White, config, 4);
    expect(rootReplyRiskSnapshot(firstExecution, game, Color.White, config, 4)).toBe(
      first,
    );
    expect(firstExecution.caches.session.entryCount).toBeGreaterThan(0);
    expect(secondExecution.caches.session.entryCount).toBe(0);

    const isolated = rootReplyRiskSnapshot(
      secondExecution,
      game,
      Color.White,
      config,
      4,
    );
    expect(isolated).toEqual(first);
    expect(isolated).not.toBe(first);

    const entriesBeforeClear = firstExecution.caches.session.entryCount;
    clearReplyRiskCache(firstExecution);
    expect(firstExecution.caches.session.entryCount).toBeLessThan(entriesBeforeClear);
  });

  it("keys selected overrides by every behavior-changing planner switch", () => {
    const base = {
      mode: TurnEngineMode.Baseline,
      ownSeedCap: 8,
      ownBeam: 3,
      perNodeFamilyCap: 3,
      stepCap: 4,
      opponentSeedCap: 2,
      opponentBeam: 1,
      replySeedCap: 1,
      replyBeam: 1,
      expansionCap: 64,
      enableSpiritFamily: true,
      scoringWeights: DEFAULT_SCORING_WEIGHTS,
      enableLazyOracleScoreWindowProjection: false,
    } satisfies TurnEngineConfig;
    const keys = [
      selectedOverrideConfigKey(base, false, false),
      selectedOverrideConfigKey(
        { ...base, mode: TurnEngineMode.Production },
        false,
        false,
      ),
      selectedOverrideConfigKey(base, false, true),
      selectedOverrideConfigKey(
        { ...base, enableLazyOracleScoreWindowProjection: true },
        false,
        false,
      ),
      selectedOverrideConfigKey(base, true, false),
    ];

    expect(keys.every((key) => key !== undefined)).toBe(true);
    expect(new Set(keys.map((key) => `${key?.hi}:${key?.lo}`)).size).toBe(keys.length);
  });
});

describe("reply-risk arbitration", () => {
  it("keeps the ordered arbitration phases and candidate-first argument polarity", () => {
    expect(replyRiskArbitrationPhaseTrace()).toEqual([
      "bind:immediateDecision=immediateReplyRiskDecision(candidate,candidateSnapshot,incumbent,incumbentSnapshot)",
      "guard:immediateDecision!==undefined=>return:immediateDecision",
      "bind:hasFullContext=context.game!==undefined&&context.evaluations!==undefined&&context.candidateIndex!==undefined&&context.incumbentIndex!==undefined&&context.perspective!==undefined",
      "bind:fullContext=hasFullContext?(contextasFullReplyRiskComparisonContext):undefined",
      "bind:followupScores=context.spiritFollowupScores??newMap<number,number>()",
      "bind:contextualDecision=contextualReplyRiskDecision(execution,candidate,candidateSnapshot,incumbent,incumbentSnapshot,config,context,fullContext,followupScores)",
      "guard:contextualDecision!==undefined=>return:contextualDecision",
      "bind:plainSpiritDecision=plainSpiritReplyRiskDecision(execution,candidate,candidateSnapshot,incumbent,incumbentSnapshot,config,context,fullContext,followupScores)",
      "guard:plainSpiritDecision!==undefined=>return:plainSpiritDecision",
      "bind:followupDecision=sharedSpiritFollowupDecision(execution,config,fullContext,followupScores)",
      "guard:followupDecision!==undefined=>return:followupDecision",
      "return:finalReplyRiskDecision(candidate,candidateSnapshot,incumbent,incumbentSnapshot,config,context)",
    ]);
  });

  it("fails closed when unrecognized top-level arbitration work is introduced", () => {
    const source = fs.readFileSync(replyRiskArbitrationPath, "utf8");
    const mutated = source.replace(
      "  const plainSpiritDecision =",
      "  recordArbitrationDecision(contextualDecision);\n\n  const plainSpiritDecision =",
    );

    expect(mutated).not.toBe(source);
    expect(() => replyRiskArbitrationPhaseTrace(mutated)).toThrow(
      "unsupported arbitration statement: ExpressionStatement",
    );
  });

  it("lets immediate safety preempt an opposing contextual progress preference", () => {
    const execution = createTestAutomoveExecutionContext();
    const game = new MonsGame(false, GameVariant.Classic);
    const config = withProductionPlanner(automoveConfigForGame(game, "fast"));
    const candidate = replyRiskRoot({ game });
    const incumbent = replyRiskRoot({
      game,
      classes: { carrierProgress: true, quiet: false },
    });

    expect(
      immediateReplyRiskDecision(
        candidate,
        safeReplyRiskSnapshot,
        incumbent,
        unsafeReplyRiskSnapshot,
      ),
    ).toBe(true);
    expect(
      contextualReplyRiskDecision(
        execution,
        candidate,
        safeReplyRiskSnapshot,
        incumbent,
        unsafeReplyRiskSnapshot,
        config,
        {},
        undefined,
        new Map(),
      ),
    ).toBe(false);
    expect(
      isBetterReplyRiskCandidate(
        execution,
        candidate,
        safeReplyRiskSnapshot,
        incumbent,
        unsafeReplyRiskSnapshot,
        config,
      ),
    ).toBe(true);
    expect(
      isBetterReplyRiskCandidate(
        execution,
        incumbent,
        unsafeReplyRiskSnapshot,
        candidate,
        safeReplyRiskSnapshot,
        config,
      ),
    ).toBe(false);
  });

  it("lets contextual progress preempt the opposite plain-Spirit score decision", () => {
    const execution = createTestAutomoveExecutionContext();
    const game = new MonsGame(false, GameVariant.Classic);
    const config = withProductionPlanner(automoveConfigForGame(game, "fast"));
    const candidate = replyRiskRoot({
      game,
      spiritDevelopment: true,
      score: -10,
      classes: { carrierProgress: true, quiet: false },
    });
    const incumbent = replyRiskRoot({ game, spiritDevelopment: true, score: 10 });

    expect(
      contextualReplyRiskDecision(
        execution,
        candidate,
        safeReplyRiskSnapshot,
        incumbent,
        safeReplyRiskSnapshot,
        config,
        {},
        undefined,
        new Map(),
      ),
    ).toBe(true);
    expect(
      plainSpiritReplyRiskDecision(
        execution,
        candidate,
        safeReplyRiskSnapshot,
        incumbent,
        safeReplyRiskSnapshot,
        config,
        {},
        undefined,
        new Map(),
      ),
    ).toBe(false);
    expect(
      isBetterReplyRiskCandidate(
        execution,
        candidate,
        safeReplyRiskSnapshot,
        incumbent,
        safeReplyRiskSnapshot,
        config,
      ),
    ).toBe(true);
    expect(
      isBetterReplyRiskCandidate(
        execution,
        incumbent,
        safeReplyRiskSnapshot,
        candidate,
        safeReplyRiskSnapshot,
        config,
      ),
    ).toBe(false);
  });

  it("lets a shared followup preempt the opposite final policy tiebreak", () => {
    const execution = createTestAutomoveExecutionContext();
    const game = new MonsGame(false, GameVariant.Classic);
    const config = withProductionPlanner(automoveConfigForGame(game, "fast"));
    const candidate = replyRiskRoot({ game, spiritDevelopment: true });
    const incumbent = replyRiskRoot({
      game,
      spiritDevelopment: true,
      policyPriority: 1,
    });
    const evaluations = [candidate, incumbent];
    const followupScores = new Map([
      [0, 100],
      [1, 0],
    ]);
    const context = {
      game,
      evaluations,
      candidateIndex: 0,
      incumbentIndex: 1,
      perspective: Color.White,
      spiritFollowupScores: followupScores,
    } as const;

    expect(
      contextualReplyRiskDecision(
        execution,
        candidate,
        unsafeReplyRiskSnapshot,
        incumbent,
        unsafeReplyRiskSnapshot,
        config,
        context,
        context,
        followupScores,
      ),
    ).toBeUndefined();
    expect(
      plainSpiritReplyRiskDecision(
        execution,
        candidate,
        unsafeReplyRiskSnapshot,
        incumbent,
        unsafeReplyRiskSnapshot,
        config,
        context,
        context,
        followupScores,
      ),
    ).toBeUndefined();
    expect(
      sharedSpiritFollowupDecision(execution, config, context, followupScores),
    ).toBe(true);
    expect(
      finalReplyRiskDecision(
        candidate,
        unsafeReplyRiskSnapshot,
        incumbent,
        unsafeReplyRiskSnapshot,
        config,
        context,
      ),
    ).toBe(false);
    expect(
      isBetterReplyRiskCandidate(
        execution,
        candidate,
        unsafeReplyRiskSnapshot,
        incumbent,
        unsafeReplyRiskSnapshot,
        config,
        context,
      ),
    ).toBe(true);
    expect(
      isBetterReplyRiskCandidate(
        execution,
        incumbent,
        unsafeReplyRiskSnapshot,
        candidate,
        unsafeReplyRiskSnapshot,
        config,
        {
          ...context,
          candidateIndex: 1,
          incumbentIndex: 0,
        },
      ),
    ).toBe(false);
  });

  it("keeps immediate decisions asymmetric and leaves exact final ties with the incumbent", () => {
    const neutral = replyRiskRoot();
    const winner = replyRiskRoot({ winsImmediately: true });

    expect(
      immediateReplyRiskDecision(
        winner,
        safeReplyRiskSnapshot,
        neutral,
        safeReplyRiskSnapshot,
      ),
    ).toBe(true);
    expect(
      immediateReplyRiskDecision(
        neutral,
        safeReplyRiskSnapshot,
        winner,
        safeReplyRiskSnapshot,
      ),
    ).toBe(false);
    expect(
      immediateReplyRiskDecision(
        neutral,
        safeReplyRiskSnapshot,
        neutral,
        unsafeReplyRiskSnapshot,
      ),
    ).toBe(true);
    expect(
      immediateReplyRiskDecision(
        neutral,
        unsafeReplyRiskSnapshot,
        neutral,
        safeReplyRiskSnapshot,
      ),
    ).toBe(false);

    const config = automoveConfigForGame(
      new MonsGame(false, GameVariant.Classic),
      "fast",
    );
    expect(
      finalReplyRiskDecision(
        replyRiskRoot({ policyPriority: 1 }),
        safeReplyRiskSnapshot,
        neutral,
        safeReplyRiskSnapshot,
        config,
        {},
      ),
    ).toBe(true);
    expect(
      finalReplyRiskDecision(
        neutral,
        safeReplyRiskSnapshot,
        replyRiskRoot({ policyPriority: 1 }),
        safeReplyRiskSnapshot,
        config,
        {},
      ),
    ).toBe(false);
    expect(
      finalReplyRiskDecision(
        neutral,
        safeReplyRiskSnapshot,
        replyRiskRoot(),
        safeReplyRiskSnapshot,
        config,
        {},
      ),
    ).toBe(false);
  });
});
