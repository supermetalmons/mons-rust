# Automove Knowledge

This document keeps only durable lessons that still matter for future automove work.

## Promoted Patterns That Held Up

- Strong root filtering matters more than wider raw enumeration when the filters are tactical and cheap.
- Drainer safety needs near-hard treatment in production search; soft penalties alone leave obvious blunders.
- Root reply-risk guard and efficiency tie-breaks are worth keeping because they remove many fake-good moves before deeper work.
- Production wasm must stay single-shot and predictable. Background or deferred search work is not release-safe yet.
- Opening-specific latency guardrails are necessary. A search that is strategically better but stalls on the first real reply is not promotable.
- Immutable reference baselines are useful. The Swift 2024 references remain valuable calibration points even when they are weaker than current runtime.

## Anti-Patterns That Repeatedly Failed

- Huge candidate catalogs create noise and slow iteration. Keep only baselines and candidates that still answer a live question.
- Shipping unvalidated experiment code is too risky, even when it looks locally promising.
- Heavy exact evaluation on both colors in every scored node is too expensive for production-facing reply turns.
- Ticked or staged wasm search that keeps working after the API returns is too risky without much stronger client-facing coverage.
- Generic meta-tooling for automated iteration was less useful than direct code, direct experiments, and clear pass/fail gates.

## Distilled Interview Guidance

These are the strongest recurring signals from `docs/automove-pro-strategy-interview.md`:

- Attack the opponent drainer whenever a real same-turn attack exists.
- Safe supermana and safe opponent-mana captures outrank routine tactical pressure.
- Do not leave your own drainer vulnerable unless the move wins immediately or scores a decisive mana now.
- Move spirit off base aggressively; spirit tempo is usually worth more than waiting.
- Spirit should help score, steal, or reposition mana toward your side, not drift into neutral/no-effect setups.
- Avoid mana movement that helps the opponent’s side of the board.
- Prefer shortest real routes and punish roundtrips or handoff-like progress that gives tempo away.

These should keep showing up as tactical fixtures, search priors, and production scoring checks.

## Current Exact-Path Gaps

The next strong improvement area is still exact active-turn tactics that are cheap enough for production:

- Can the active player attack the opponent drainer this turn?
- Can a drainer pick up supermana or opponent mana and score or land safely?
- Can spirit create a same-turn score, denial, or stronger carry route?
- Can the move leave a mana carrier or drainer on a square that is untouchable next turn?

The important constraint is cost:

- active-turn tactical exactness is valuable
- passive non-active followup evaluation must stay cheap
- cached tactical answers should be reused across root ranking, move-efficiency scoring, and tactical prepasses

## Good Next Directions

- Build a cached active-turn tactical query bundle keyed by search-state hash.
- Use that bundle to narrow root and child move sets before expensive scoring work.
- Add more interview-derived fixtures around spirit scoring, safe supermana, opponent-mana conversion, and drainer exposure.
- Keep passive summaries lightweight so exact tactics improve play without reintroducing client freezes.
