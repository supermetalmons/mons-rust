export const ProductionHeadGuardId = Object.freeze({
  EarlySafeManaBlocksSpirit: "early-safe-mana-blocks-spirit",
  BlackTurnStartSafeManaBlocksPlainSpirit:
    "black-turn-start-safe-mana-blocks-plain-spirit",
  WhiteTurnStartSafeManaBlocksPlainSpirit:
    "white-turn-start-safe-mana-blocks-plain-spirit",
  WhiteLateSafeManaBlocksPlainSpirit: "white-late-safe-mana-blocks-plain-spirit",
  BlackNoActionVulnerableProgressHead: "black-no-action-vulnerable-progress-head",
  BlackEarlySameWindowManaHead: "black-early-same-window-mana-head",
  BlackNoActionWindowedVulnerableManaHead:
    "black-no-action-windowed-vulnerable-mana-head",
  BlackLateWindowedVulnerableManaHead: "black-late-windowed-vulnerable-mana-head",
  BlackEarlyProgressBlocksMana: "black-early-progress-blocks-mana",
  BlackEarlyProgressBlocksNonConcreteWindow:
    "black-early-progress-blocks-non-concrete-window",
  EarlyBlackSafeManaBlocksWeakerMana: "early-black-safe-mana-blocks-weaker-mana",
  BlackQuietManaBlocksLowerScoredMana: "black-quiet-mana-blocks-lower-scored-mana",
  WhiteSameWindowManaBlocksLowerScoredMana:
    "white-same-window-mana-blocks-lower-scored-mana",
  WhiteMidTurnManaBlocksLowerScoredWindowMana:
    "white-mid-turn-mana-blocks-lower-scored-window-mana",
  WhiteMidTurnSpiritSetupBlocksWindowMana:
    "white-mid-turn-spirit-setup-blocks-window-mana",
  WhiteTurnStartSpiritSetupBlocksWindowMana:
    "white-turn-start-spirit-setup-blocks-window-mana",
  WhiteSafeManaBlocksDeferredRecoveryProgress:
    "white-safe-mana-blocks-deferred-recovery-progress",
  VulnerableWhiteManaHead: "vulnerable-white-mana-head",
  BlackLateSafeProgressBlocksQuietMana: "black-late-safe-progress-blocks-quiet-mana",
  BlackRecoveryRootBlocksNonConcreteWindow:
    "black-recovery-root-blocks-non-concrete-window",
  WhiteRecoveryRootBlocksNonConcreteWindow:
    "white-recovery-root-blocks-non-concrete-window",
  CandidateUnsafeWithoutProjectedOverride:
    "candidate-unsafe-without-projected-override",
  SelectedUtilityDominatesPlan: "selected-utility-dominates-plan",
  NonProgressHeadWithoutOverride: "non-progress-head-without-override",
  DrainerKillWithoutAttack: "drainer-kill-without-attack",
  WhiteVulnerableProgressBlocksImmediateScore:
    "white-vulnerable-progress-blocks-immediate-score",
  SpiritHeadWithoutImpact: "spirit-head-without-impact",
});

export type ProductionHeadGuardId =
  (typeof ProductionHeadGuardId)[keyof typeof ProductionHeadGuardId];
