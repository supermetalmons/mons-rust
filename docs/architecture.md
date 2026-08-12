# Architecture

This document records the intended boundaries for ongoing maintenance. It is a
directional design, not a claim that every source file has already reached its
final location.

## Dependency direction

The published entrypoint is a narrow export surface over the public API. The
public API composes the rules engine and automove system. Engine and automove
modules share only `api/types.ts`, the contract leaf that preserves public and
internal enum identity; the engine does not depend on the API façade or
automove.

```text
entrypoints -> api -> automove -> engine
                  \------------> engine
cli ---------------------------> engine
```

Dependency arrows point from importer to dependency. Within the engine, game
orchestration depends on rules and codecs, which depend on board and model
primitives:

```text
game -> rules -> board -> model -> api/types
  \----> codec ----^       ^
  \----> board ------------/
```

Model code owns domain values. Board code owns geometry, configuration, and
storage. Rules code owns legality and event application. Codecs translate
between serialized and in-memory representations. Game code coordinates those
pieces. `model/domain.ts` has one type-only dependency on board geometry for the
shared location shape. Lower layers should not import the API façade or
automove modules, and façade files should remain compatibility boundaries
rather than alternate implementations.

Within automove, runtime modules compose policy; policy consumes search; search
consumes root selection; and root selection consumes turn planning. Those
areas may also consume the lower configuration, core, exact, scoring, and
transition leaves allowed by the architecture test. The intended high-level
direction is:

```text
runtime -> policy -> search -> root -> turn
   |          |        |        |       |
   +----------+--------+--------+------> config, core, exact, scoring, transitions
   +-----------------------------------> packed
```

`root/types.ts` owns the shared root candidate and evaluated-root shapes;
search produces evaluated roots but root modules never depend on search.

Moving a file should preserve this direction and keep selector ordering,
tie-breaks, time checkpoints, and public suggestions unchanged.

Modules should normally stay at or below 600 physical lines. The architecture
test records and verifies the small set of 600–800-line exceptions together
with their hot-path or state-ownership reason; no source module may exceed 800
lines.

## Tooling boundaries

Top-level files in `scripts/` are stable developer command paths. When a command
needs substantial implementation, the top-level file remains a thin façade:

- automove evidence support is grouped under `scripts/evidence/`;
- package assertions are grouped under `scripts/package-check/`;
- generated reports and derived evidence belong under `target/` or the OS
  temporary directory.

The complete-games integrity-only command is:

```sh
node ./scripts/run-complete-games.mjs --check-only
```

`node ./scripts/check-package.mjs .` validates the ESM manifest, declarations,
tar surface, runtime and type consumers, browser and worker bundles, and the
intentional CommonJS rejection.

## Test and corpus contracts

Tests mirror the public API, engine, automove, CLI, corpus, and script
boundaries. Behavior-preserving refactors should use the narrowest focused
tests first and finish with the canonical checks.

Files under `test-data/complete-games/v1/` and the protected regression corpora
are immutable inputs. Cleanup work must not normalize, regenerate, reorder, or
rewrite them. The compressed rules corpus remains owned by
`scripts/run-rules-tests.sh`; derived artifacts stay outside `test-data/`.
