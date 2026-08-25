# Architecture

This is the maintainer contract for the source tree, automove behavior, and
immutable test data.

## Dependency direction

The published entrypoint exports the public API. The API adapts the rules
engine and requests automove suggestions; automove operates directly on engine
state. CLI programs exercise the engine without depending on the public façade.

```text
entrypoints -> api -> automove -> engine
                  \------------> engine
cli ---------------------------> engine
```

`api/types.ts` is the shared leaf used to preserve public and internal enum and
value identity. Engine and automove code may depend on that leaf, but neither
may depend on the API façade. Internal modules import implementations directly;
only the published entrypoint and the deliberate API-type identity boundaries
reexport another module.

The engine has five directed areas:

```text
game -> rules -> board -> model
  \----> codec ----^       ^
  \----> board ------------/
```

- `model` owns domain values, copying, equality, and stable keys.
- `board` owns geometry, variant configuration, and storage.
- `rules` owns legality, counters, and event application.
- `codec` translates between canonical serialized and in-memory values.
- `game` owns orchestration, staged input, history, and query caches.

`engine/model/domain.ts` has one type-only geometry dependency for the shared
location shape. The architecture test enforces all other engine direction,
acyclic imports, a flat automove directory, direct-import rules, at most 43
source files and 12,000 lines total, and an 800-line per-module ceiling. Modules
over the preferred 600-line limit must have a current cohesion reason in that
test.

## Automove

Automove is one flat packed-search implementation:

| Modules                                    | Responsibility                                                                      |
| ------------------------------------------ | ----------------------------------------------------------------------------------- |
| `allocation`, `board`, `state`             | Allocation failures, packed board values, mutable position state, undo, and hashing |
| `bridge`                                   | Lossless conversion between engine state, packed moves, and public inputs           |
| `moves`                                    | Deterministic complete-move generation and ordering                                 |
| `evaluation-weights`, `evaluation`         | Validated weights, lookup tables, attack data, and position scoring                 |
| `search-tuning`, `transposition`, `search` | Search limits, bounded caches, iterative deepening, and selective PVS/negamax       |
| `selector`                                 | Public preference profiles, deadlines, and optional searcher reuse                  |
| `suggestion`                               | Source-pure application checks and deterministic legal fallback                     |

The public profiles are fixed:

| Preference | Wall-clock budget | Fixed-clock node ceiling |
| ---------- | ----------------: | -----------------------: |
| Fast       |             16 ms |                   38,400 |
| Normal     |             75 ms |                  184,000 |
| Pro        |            460 ms |                2,000,000 |

Search is cooperative, so runtime, JIT, cache, and garbage-collection timing
can change a live selection. Reaching the deadline stops the search and retains
its supported nonzero move from the last completed iteration. If no such move is
available, the packed state is unsupported, allocation fails, or a selected move
cannot be applied, `suggestion` returns the first deterministic complete legal
move. Suggestions must remain legal and leave the source game byte-for-byte
unchanged.

The shipped search keeps aspiration windows for Fast and Normal, bounded
transposition data for selective nodes, deterministic ordering for commuting
mon sub-moves, cached evaluation and attack tables, and a winning regular-mana
step at any point in a turn. Quiet spirit pushes use ordinary quiet-move
ordering while tactical pushes retain reduction and pruning exemptions.
Evaluation models score shape, mana delivery, one- and two-point drainer trips,
attack exposure, and relative scoring tempo. Selective subtree transpositions
carry move ordering only when their score-bound direction is not known.

The active deterministic contract is
[automove decisions v16](../test-data/automove-decisions/v16/README.md): 13
states across all three preferences, or 39 decisions, with
`performance.now()` fixed at zero. The READMEs and manifests beside every
protected automove corpus contain the historical provenance; do not duplicate
or rewrite that history here.

The final quiet-spirit ordering change was accepted using held-out,
colour-swapped, fixed-node self-play before v16 was recorded:

| Mode   | Pairs |   Elo |   95% interval | Invalids / cutoffs |
| ------ | ----: | ----: | -------------: | -----------------: |
| Fast   | 1,280 | +21.5 |  +9.1 to +33.9 |              0 / 0 |
| Normal | 1,280 | +25.3 | +12.2 to +38.5 |              0 / 0 |
| Pro    |   320 | +50.3 | +24.6 to +76.5 |              0 / 0 |

An eight-block ABBA latency run over 64 midpoint states measured
candidate/baseline mean ratios of 0.997 for Fast, 0.998 for Normal, and 1.000
for Pro. These are historical rationale; the immutable decision corpus and
current repository checks are the executable contract.

## Validation, release, and data

`npm run check` is the canonical repository gate. It formats, lints,
type-checks, runs unit and corpus tests, replays complete games, verifies
protected data, builds the package, and validates its Node, browser, worker,
runtime, declaration, and tarball surfaces. The compressed rules regression
corpus is run through `./scripts/run-rules-tests.sh`; an integrity-only
complete-games check is available as
`node ./scripts/run-complete-games.mjs --check-only`.

Release preparation uses `npm run bump`, followed by
`npm run publish -- --check-only`; `npm run publish` performs the validated
publish from a clean worktree.

All existing payloads under `test-data/complete-games/v1/`,
`test-data/automove-decisions/`, and `test-data/compatibility-edge-cases/v1/`,
plus `test-data/rules-regressions.jsonl.gz`, are immutable. Never normalize,
deduplicate, reorder, regenerate, or replace them. Put a changed source corpus
in a new version directory and put derived reports under `target/` or the OS
temporary directory.
