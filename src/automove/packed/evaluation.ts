import { BOARD_CELLS, BOARD_SIZE } from "../../engine/board/geometry.js";
import { MONS_MOVES_PER_TURN, TARGET_SCORE } from "../../engine/board/config.js";
import {
  CENTER_ROW_DISTANCE,
  CONS_BOMB,
  DEMON_TARGETS,
  KIND_ANGEL,
  KIND_DEMON,
  KIND_DRAINER,
  KIND_MYSTIC,
  KIND_SPIRIT,
  MANA_BLACK,
  MANA_SUPER,
  MANA_WHITE,
  COLOR_COUNT,
  MON_KIND_COUNT,
  MYSTIC_TARGETS,
  OCC_MANA,
  OCC_MON,
  OWN_POOL_DISTANCE,
  POOL_DISTANCE,
  SQ_MON_BASE,
  SQ_SUPERMANA_BASE,
  SUPERMANA_BASE_INDEX,
  cellConsumable,
  cellCooldown,
  cellMana,
  cellOccupancy,
  chebyshev,
  damagingAttackCanComplete,
  i32,
  manaScoreValue,
  midpoint,
  monId,
  u16,
  u8,
} from "./board.js";
import { rethrowFastWorkspaceAllocation } from "./allocation.js";
import { awakeAngelGuards, manaMoveAllowed, type FastPosition } from "./state.js";
import {
  DISTANCE_TABLE_SIZE,
  THREAT_BUCKETS,
  THREAT_BUCKET_EXPOSED,
  THREAT_BUCKET_FEW,
  THREAT_BUCKET_SPARE,
  THREAT_SPARE_MOVES,
  THREAT_WALK_STRIDE,
  RACE_LATE_NEED,
  RACE_MAX_TURNS,
  RACE_SPAN,
  SCORE_SHAPE_STRIDE,
  UNREACHABLE_DISTANCE,
  createEvalTables,
  normalizeEvalWeights,
  type EvalTables,
  type EvalWeights,
} from "./evaluation-weights.js";

export const WIN_VALUE = 1_000_000;

function ownPoolDistance(index: number, color: number): number {
  return u8(OWN_POOL_DISTANCE, color * BOARD_CELLS + index);
}

const POOL_INDICES = Int32Array.of(
  0,
  BOARD_SIZE - 1,
  BOARD_CELLS - BOARD_SIZE,
  BOARD_CELLS - 1,
);

function hasAdjacentScoringPool(position: FastPosition, index: number): boolean {
  for (let slot = 0; slot < POOL_INDICES.length; slot += 1) {
    const pool = i32(POOL_INDICES, slot);
    if (chebyshev(index, pool) === 1 && manaMoveAllowed(position, pool)) {
      return true;
    }
  }
  return false;
}

// What a fused trip is worth: the turn bucket it lands in, less the steps still to walk.
function tripValue(
  tables: EvalTables,
  bucket: Int32Array,
  distance: number,
  budget: number,
): number {
  return (
    i32(bucket, distance > budget ? distance - budget : 0) -
    i32(tables.tripStep, distance)
  );
}

function centerDistance(index: number): number {
  return u8(CENTER_ROW_DISTANCE, index);
}

type AttackTables = {
  readonly mysticSteps: Uint8Array;
  readonly demonStarts: Int32Array;
  readonly demonCounts: Int32Array;
  readonly demonOrigins: Int32Array;
  readonly demonBetween: Int32Array;
};

const ATTACK_TABLES = new WeakMap<object, AttackTables>();
// Every evaluation of one search shares a single squares array, so a pointer check answers
// the lookup without touching the weak map on the hot path.
let lastAttackSquares: ArrayLike<number> | undefined;
let lastAttackTables: AttackTables | undefined;

function sameSquareContent(
  previous: ArrayLike<number> | undefined,
  current: ArrayLike<number>,
): boolean {
  if (previous?.length !== current.length) return false;
  for (let index = 0; index < current.length; index += 1) {
    if (u8(previous, index) !== u8(current, index)) return false;
  }
  return true;
}

function buildAttackTables(squares: ArrayLike<number>): AttackTables {
  try {
    const mysticSteps = new Uint8Array(BOARD_CELLS * BOARD_CELLS).fill(
      UNREACHABLE_DISTANCE,
    );
    const demonStarts = new Int32Array(BOARD_CELLS);
    const demonCounts = new Int32Array(BOARD_CELLS);
    const origins: number[] = [];
    const betweens: number[] = [];
    for (let at = 0; at < BOARD_CELLS; at += 1) {
      const mysticStart = i32(MYSTIC_TARGETS.starts, at);
      const mysticCount = i32(MYSTIC_TARGETS.counts, at);
      for (let index = 0; index < BOARD_CELLS; index += 1) {
        let best = UNREACHABLE_DISTANCE;
        for (let offset = 0; offset < mysticCount; offset += 1) {
          const origin = i32(MYSTIC_TARGETS.list, mysticStart + offset);
          if (u8(squares, origin) >= SQ_MON_BASE) continue;
          const steps = chebyshev(index, origin);
          if (steps < best) best = steps;
        }
        mysticSteps[at * BOARD_CELLS + index] = best;
      }
      demonStarts[at] = origins.length;
      const demonStart = i32(DEMON_TARGETS.starts, at);
      const demonCount = i32(DEMON_TARGETS.counts, at);
      for (let offset = 0; offset < demonCount; offset += 1) {
        const origin = i32(DEMON_TARGETS.list, demonStart + offset);
        if (u8(squares, origin) >= SQ_MON_BASE) continue;
        const between = midpoint(at, origin);
        const betweenSquare = u8(squares, between);
        if (betweenSquare === SQ_SUPERMANA_BASE || betweenSquare >= SQ_MON_BASE) {
          continue;
        }
        origins.push(origin);
        betweens.push(between);
      }
      demonCounts[at] = origins.length - i32(demonStarts, at);
    }
    return {
      mysticSteps,
      demonStarts,
      demonCounts,
      demonOrigins: new Int32Array(origins),
      demonBetween: new Int32Array(betweens),
    };
  } catch (error) {
    rethrowFastWorkspaceAllocation(error);
  }
}

// The static part of the attack-origin filter depends only on the variant's squares, so
// it is precomputed once per squares array: mystic origins reduce to a min-distance table
// and demon origins to the (origin, between) pairs whose squares allow the attack.
function attackTablesFor(squares: ArrayLike<number>): AttackTables {
  if (squares === lastAttackSquares && lastAttackTables !== undefined) {
    return lastAttackTables;
  }
  const cached = ATTACK_TABLES.get(squares);
  if (cached !== undefined) {
    lastAttackSquares = squares;
    lastAttackTables = cached;
    return cached;
  }
  // Each selection call arrives with a fresh squares array - `fastSquaresForVariant` slices the
  // canonical table and `FastPosition.reset` copies that slice again - so the identity caches
  // above never hit across calls and the mystic table below was rebuilt every time. The tables
  // are a pure function of the squares content, so one comparison stands in for the rebuild.
  // This requires that a published squares array is never written in place: every writer
  // allocates a fresh array, and an in-place edit would let one array donate stale tables to a
  // different array whose content matches the edited state.
  if (lastAttackTables !== undefined && sameSquareContent(lastAttackSquares, squares)) {
    const reused = lastAttackTables;
    ATTACK_TABLES.set(squares, reused);
    lastAttackSquares = squares;
    return reused;
  }
  const tables = buildAttackTables(squares);
  ATTACK_TABLES.set(squares, tables);
  lastAttackSquares = squares;
  lastAttackTables = tables;
  return tables;
}

function estimatedAttackSteps(
  position: FastPosition,
  at: number,
  ownerColor: number,
  guarded: boolean,
): number {
  if (!damagingAttackCanComplete(u16(position.cells, at))) {
    return UNREACHABLE_DISTANCE;
  }
  const enemy = ownerColor ^ 1;
  let best = UNREACHABLE_DISTANCE;
  const attackTables = attackTablesFor(position.squares);

  for (let kind = 0; kind < MON_KIND_COUNT; kind += 1) {
    const index = i32(position.monLocations, monId(kind, enemy));
    if (index < 0) continue;
    const cell = u16(position.cells, index);
    if (cellOccupancy(cell) !== OCC_MON) continue;
    if (cellConsumable(cell) === CONS_BOMB) {
      const reach = chebyshev(index, at) - 3;
      const steps = reach < 0 ? 0 : reach;
      if (steps === 0) return 0;
      if (steps < best) best = steps;
      continue;
    }
    if (cellCooldown(cell) !== 0) continue;
    if (cellConsumable(cell) !== 0) continue;
    if (guarded) continue;
    if (cellMana(cell) !== 0) continue;
    if (kind === KIND_MYSTIC) {
      const steps = u8(attackTables.mysticSteps, at * BOARD_CELLS + index);
      if (steps === 0) return 0;
      if (steps < best) best = steps;
      continue;
    }
    if (kind !== KIND_DEMON) continue;
    const targetStart = i32(attackTables.demonStarts, at);
    const targetCount = i32(attackTables.demonCounts, at);
    for (let offset = 0; offset < targetCount; offset += 1) {
      const between = i32(attackTables.demonBetween, targetStart + offset);
      if (between !== index && u16(position.cells, between) !== 0) continue;
      const steps = chebyshev(
        index,
        i32(attackTables.demonOrigins, targetStart + offset),
      );
      if (steps === 0) return 0;
      if (steps < best) best = steps;
    }
  }
  return best;
}

export function evaluatePosition(position: FastPosition, weights: EvalWeights): number {
  return evaluateWithTables(position, createEvalTables(normalizeEvalWeights(weights)));
}

export function evaluateWithTables(
  position: FastPosition,
  tables: EvalTables,
  winsNextTurnThreat = 0,
): number {
  const weights = tables.weights;
  let value =
    (position.whiteScore - position.blackScore) * weights.scoreUnit +
    i32(
      tables.scoreShape,
      position.whiteScore * SCORE_SHAPE_STRIDE + position.blackScore,
    ) +
    (i32(position.potions, 0) - i32(position.potions, 1)) * weights.potion;

  const drainerWhite = i32(position.monLocations, monId(KIND_DRAINER, 0));
  const drainerBlack = i32(position.monLocations, monId(KIND_DRAINER, 1));
  const drainerReadyWhite =
    drainerWhite >= 0 && cellCooldown(u16(position.cells, drainerWhite)) === 0;
  const drainerReadyBlack =
    drainerBlack >= 0 && cellCooldown(u16(position.cells, drainerBlack)) === 0;
  let nearestManaWhite = UNREACHABLE_DISTANCE;
  let nearestManaBlack = UNREACHABLE_DISTANCE;
  let shortestPickupScoreWhite = UNREACHABLE_DISTANCE;
  let shortestPickupScoreBlack = UNREACHABLE_DISTANCE;
  // The fused trip is tracked once per point class: the nearest item is not the best plan when
  // a step or two further along stands one worth double.
  let twoPointPickupWhite = UNREACHABLE_DISTANCE;
  let twoPointPickupBlack = UNREACHABLE_DISTANCE;
  const remaining = MONS_MOVES_PER_TURN - position.monsMoves;
  const windowTracking = winsNextTurnThreat !== 0;
  const needWhite = TARGET_SCORE - position.whiteScore;
  const needBlack = TARGET_SCORE - position.blackScore;
  let manaStepWin = 0;
  let manaStepTurnsWhite = RACE_MAX_TURNS;
  let manaStepTurnsBlack = RACE_MAX_TURNS;
  let tripTurnsWhite = RACE_MAX_TURNS;
  let tripTurnsBlack = RACE_MAX_TURNS;
  let pickupNextWhite = false;
  let pickupNextBlack = false;
  if (windowTracking) {
    if (drainerWhite >= 0) {
      const cell = u16(position.cells, drainerWhite);
      pickupNextWhite =
        cellCooldown(cell) <= 1 && cellMana(cell) === 0 && cellConsumable(cell) === 0;
    }
    if (drainerBlack >= 0) {
      const cell = u16(position.cells, drainerBlack);
      pickupNextBlack =
        cellCooldown(cell) <= 1 && cellMana(cell) === 0 && cellConsumable(cell) === 0;
    }
  }
  let pickupBestPointsWhite = 0;
  let pickupBestPointsBlack = 0;
  let pickupBestAdjacentOwnWhite = false;
  let pickupBestAdjacentOwnBlack = false;
  let manaMoveCountWhite = 0;
  let manaMoveCountBlack = 0;
  let carrierNextPointsWhite = 0;
  let carrierNextPointsBlack = 0;

  for (let slot = 0; slot < position.manaCount; slot += 1) {
    const index = i32(position.manaIndices, slot);
    const cell = u16(position.cells, index);
    if (cellOccupancy(cell) !== OCC_MANA) continue;
    const mana = cellMana(cell);
    const distanceWhite = drainerReadyWhite
      ? chebyshev(drainerWhite, index)
      : UNREACHABLE_DISTANCE;
    const distanceBlack = drainerReadyBlack
      ? chebyshev(drainerBlack, index)
      : UNREACHABLE_DISTANCE;
    if (distanceWhite < nearestManaWhite) nearestManaWhite = distanceWhite;
    if (distanceBlack < nearestManaBlack) nearestManaBlack = distanceBlack;
    const scoreDistance = u8(POOL_DISTANCE, index);
    if (drainerReadyWhite) {
      const pickupScoreDistance = distanceWhite + scoreDistance;
      if (pickupScoreDistance < shortestPickupScoreWhite) {
        shortestPickupScoreWhite = pickupScoreDistance;
      }
      // Two points for either side means the supermana or the other side's mana, so the test
      // is one comparison against the owning code rather than a scoring lookup.
      if (mana !== MANA_WHITE && pickupScoreDistance < twoPointPickupWhite) {
        twoPointPickupWhite = pickupScoreDistance;
      }
    }
    if (drainerReadyBlack) {
      const pickupScoreDistance = distanceBlack + scoreDistance;
      if (pickupScoreDistance < shortestPickupScoreBlack) {
        shortestPickupScoreBlack = pickupScoreDistance;
      }
      if (mana !== MANA_BLACK && pickupScoreDistance < twoPointPickupBlack) {
        twoPointPickupBlack = pickupScoreDistance;
      }
    }
    if (windowTracking) {
      let manaMoveTarget = false;
      if (
        mana !== MANA_SUPER &&
        scoreDistance <= 1 &&
        hasAdjacentScoringPool(position, index)
      ) {
        manaMoveTarget = true;
        if (mana - 1 === 0) {
          if (manaMoveCountWhite < 2) manaMoveCountWhite += 1;
        } else if (manaMoveCountBlack < 2) {
          manaMoveCountBlack += 1;
        }
      }
      if (pickupNextWhite) {
        const distance = drainerReadyWhite
          ? distanceWhite
          : chebyshev(drainerWhite, index);
        if (distance + scoreDistance <= MONS_MOVES_PER_TURN) {
          const points = manaScoreValue(mana, 0);
          if (points > pickupBestPointsWhite) {
            pickupBestPointsWhite = points;
            pickupBestAdjacentOwnWhite = manaMoveTarget && mana - 1 === 0;
          } else if (
            points === pickupBestPointsWhite &&
            !(manaMoveTarget && mana - 1 === 0)
          ) {
            pickupBestAdjacentOwnWhite = false;
          }
        }
      }
      if (pickupNextBlack) {
        const distance = drainerReadyBlack
          ? distanceBlack
          : chebyshev(drainerBlack, index);
        if (distance + scoreDistance <= MONS_MOVES_PER_TURN) {
          const points = manaScoreValue(mana, 1);
          if (points > pickupBestPointsBlack) {
            pickupBestPointsBlack = points;
            pickupBestAdjacentOwnBlack = manaMoveTarget && mana - 1 === 1;
          } else if (
            points === pickupBestPointsBlack &&
            !(manaMoveTarget && mana - 1 === 1)
          ) {
            pickupBestAdjacentOwnBlack = false;
          }
        }
      }
    }
    let control = distanceBlack - distanceWhite;
    if (control > 4) control = 4;
    if (control < -4) control = -4;
    if (weights.manaPointsAttraction !== 0) {
      const attraction = tables.manaPointsAttraction;
      if (drainerReadyWhite) {
        value += i32(
          attraction,
          manaScoreValue(mana, 0) * DISTANCE_TABLE_SIZE + distanceWhite,
        );
      }
      if (drainerReadyBlack) {
        value -= i32(
          attraction,
          manaScoreValue(mana, 1) * DISTANCE_TABLE_SIZE + distanceBlack,
        );
      }
    }
    if (mana === MANA_SUPER) {
      value += weights.supermanaDrainerControl * control;
    } else {
      const ownerColor = mana - 1;
      const ownerSign = ownerColor === 0 ? 1 : -1;
      value +=
        ownerSign * i32(tables.manaToOwnerPool, ownPoolDistance(index, ownerColor));
      value += ownerSign * i32(tables.manaToNearestPool, scoreDistance);
      value += weights.manaDrainerControl * control;
      // One point from winning, with own mana already inside mana-step reach of a pool, is
      // the one need-by-distance cell the additive form cannot price.
      if (scoreDistance <= 2 && (ownerColor === 0 ? needWhite : needBlack) <= 1) {
        manaStepWin |= ownerColor === 0 ? 1 : 2;
      }
      if (ownerColor === 0) {
        if (scoreDistance < manaStepTurnsWhite) manaStepTurnsWhite = scoreDistance;
      } else if (scoreDistance < manaStepTurnsBlack) {
        manaStepTurnsBlack = scoreDistance;
      }
    }
  }

  if (manaStepWin !== 0) {
    value +=
      weights.manaStepWinThreat *
      (((manaStepWin & 1) === 0 ? 0 : 1) - ((manaStepWin & 2) === 0 ? 0 : 1));
  }

  for (let color = 0; color < COLOR_COUNT; color += 1) {
    const sign = color === 0 ? 1 : -1;
    const enemy = color ^ 1;
    for (let kind = 0; kind < MON_KIND_COUNT; kind += 1) {
      const index = i32(position.monLocations, monId(kind, color));
      if (index < 0) continue;
      const cell = u16(position.cells, index);
      if (cellOccupancy(cell) !== OCC_MON) continue;
      const cooldown = cellCooldown(cell);
      if (cooldown !== 0) {
        value -=
          sign *
          ((kind === KIND_DRAINER ? weights.faintDrainer : weights.faintMon) +
            weights.faintCooldownStep * cooldown);
      }
      const square = u8(position.squares, index);
      if (cooldown === 0 && square < SQ_MON_BASE) {
        value += sign * weights.activeMon;
      }

      const carriedMana = cellMana(cell);
      if (carriedMana !== 0) {
        const points = manaScoreValue(carriedMana, color);
        const poolDistance = u8(POOL_DISTANCE, index);
        value +=
          sign *
          (i32(tables.carrierCloseToPool, poolDistance) +
            points * weights.carrierPointBonus);
        if (carriedMana === MANA_SUPER) value += sign * weights.supermanaCarrier;
        if (color === position.active && poolDistance <= remaining) {
          value += sign * weights.carrierScoresThisTurn * points;
        } else if (color !== position.active && poolDistance <= MONS_MOVES_PER_TURN) {
          value += sign * weights.carrierScoresNextTurn * points;
          if (windowTracking) {
            if (color === 0) {
              if (points > carrierNextPointsWhite) carrierNextPointsWhite = points;
            } else if (points > carrierNextPointsBlack) {
              carrierNextPointsBlack = points;
            }
          }
        }
        const ownScore = color === 0 ? position.whiteScore : position.blackScore;
        if (points >= TARGET_SCORE - ownScore) {
          value += sign * weights.winningCarrier;
        }
      } else if (cellConsumable(cell) === CONS_BOMB) {
        value += sign * weights.bomb;
      }

      if (cooldown !== 0) continue;

      if (kind === KIND_DRAINER) {
        let plan = 0;
        const minMana = color === 0 ? nearestManaWhite : nearestManaBlack;
        const pickupScoreDistance =
          color === 0 ? shortestPickupScoreWhite : shortestPickupScoreBlack;
        const tripBudget = color === position.active ? remaining : MONS_MOVES_PER_TURN;
        const twoPointPickup = color === 0 ? twoPointPickupWhite : twoPointPickupBlack;
        const tripDistance =
          carriedMana !== 0
            ? u8(POOL_DISTANCE, index)
            : cellConsumable(cell) !== 0
              ? UNREACHABLE_DISTANCE
              : pickupScoreDistance;
        // Weigh the best two-point trip against the shortest one and keep whichever is worth
        // more; at the neutral scale both are priced by the same table, so the shorter wins.
        const twoPointTrip =
          carriedMana === 0 &&
          cellConsumable(cell) === 0 &&
          twoPointPickup < UNREACHABLE_DISTANCE &&
          twoPointPickup !== tripDistance &&
          tripValue(tables, tables.drainerTripTwoPoint, twoPointPickup, tripBudget) >
            tripValue(tables, tables.drainerTrip, tripDistance, tripBudget);
        const tripTable = twoPointTrip
          ? tables.drainerTripTwoPoint
          : tables.drainerTrip;
        const tripSteps = twoPointTrip ? twoPointPickup : tripDistance;
        plan -= i32(tables.tripStep, tripSteps);
        const tripExcess = tripSteps > tripBudget ? tripSteps - tripBudget : 0;
        const tripTurns =
          1 + (((tripExcess + MONS_MOVES_PER_TURN - 1) / MONS_MOVES_PER_TURN) | 0);
        if (color === 0) {
          if (tripTurns < tripTurnsWhite) tripTurnsWhite = tripTurns;
        } else if (tripTurns < tripTurnsBlack) {
          tripTurnsBlack = tripTurns;
        }
        plan += i32(tripTable, tripExcess);
        plan += i32(tables.drainerCloseToMana, minMana);
        plan += i32(tables.drainerCloseToOwnPool, ownPoolDistance(index, color));
        plan += i32(
          tables.drainerCloseToSupermana,
          chebyshev(index, SUPERMANA_BASE_INDEX),
        );
        if (
          carriedMana === 0 &&
          cellConsumable(cell) === 0 &&
          color === position.active &&
          pickupScoreDistance <= remaining
        ) {
          plan += weights.drainerPickupScoresThisTurn;
        }
        value += sign * plan;
        const guarded = awakeAngelGuards(position, color, index);
        const threatSteps = estimatedAttackSteps(position, index, color, guarded);
        const carrying = carriedMana !== 0;
        const bucket =
          color !== position.active || remaining === 0
            ? THREAT_BUCKET_EXPOSED
            : remaining >= THREAT_SPARE_MOVES
              ? THREAT_BUCKET_SPARE
              : THREAT_BUCKET_FEW;
        const row = (carrying ? THREAT_BUCKETS : 0) + bucket;
        if (threatSteps === 0) {
          value -= sign * i32(tables.threatImmediate, row);
        } else if (threatSteps <= MONS_MOVES_PER_TURN) {
          value -=
            sign * (tables.threatWalk[row * THREAT_WALK_STRIDE + threatSteps] ?? 0);
        }
        if (guarded) {
          value += sign * weights.angelGuardingDrainer;
        }
      } else if (kind === KIND_ANGEL) {
        const own = color === 0 ? drainerWhite : drainerBlack;
        if (own >= 0) {
          value += sign * i32(tables.angelCloseToDrainer, chebyshev(index, own));
        }
      } else if (kind === KIND_SPIRIT) {
        let nearestEnemy = UNREACHABLE_DISTANCE;
        for (let other = 0; other < MON_KIND_COUNT; other += 1) {
          const enemyIndex = i32(position.monLocations, monId(other, enemy));
          if (enemyIndex < 0) continue;
          const enemyCell = u16(position.cells, enemyIndex);
          if (cellCooldown(enemyCell) !== 0) continue;
          const distance = chebyshev(index, enemyIndex);
          if (distance < nearestEnemy) nearestEnemy = distance;
        }
        value += sign * i32(tables.spiritCloseToEnemy, nearestEnemy);
        if (square >= SQ_MON_BASE) value -= sign * weights.spiritOnOwnBase;
      } else {
        value += sign * i32(tables.monCloseToCenter, centerDistance(index));
        const enemyDrainer = enemy === 0 ? drainerWhite : drainerBlack;
        const enemyDrainerReady = enemy === 0 ? drainerReadyWhite : drainerReadyBlack;
        if (enemyDrainer >= 0 && enemyDrainerReady) {
          value +=
            sign *
            i32(tables.attackerCloseToEnemyDrainer, chebyshev(index, enemyDrainer));
        }
      }
    }
  }

  // The lead in tempo decides a race that is nearly over; earlier it is one plan among many.
  if (
    weights.raceHalfTurn !== 0 &&
    (needWhite < needBlack ? needWhite : needBlack) <= RACE_LATE_NEED
  ) {
    // A side that has already spent its free mana step this turn needs one more turn to
    // deliver by that channel.
    const stepUsed = position.manaMoves > 0 ? 1 : 0;
    const stepWhite = manaStepTurnsWhite + (position.active === 0 ? stepUsed : 0);
    const stepBlack = manaStepTurnsBlack + (position.active === 1 ? stepUsed : 0);
    const tauWhite = tripTurnsWhite < stepWhite ? tripTurnsWhite : stepWhite;
    const tauBlack = tripTurnsBlack < stepBlack ? tripTurnsBlack : stepBlack;
    // Half-turns: the side to move starts its count now, the other side one half-turn later.
    const half =
      tauBlack * 2 +
      (position.active === 1 ? 0 : 1) -
      (tauWhite * 2 + (position.active === 0 ? 0 : 1));
    const clamped =
      half > RACE_SPAN ? RACE_SPAN : half < -RACE_SPAN ? -RACE_SPAN : half;
    value += i32(tables.race, clamped + RACE_SPAN);
  }

  if (windowTracking) {
    const inactive = position.active ^ 1;
    const inactiveSign = inactive === 0 ? 1 : -1;
    const pickupPoints = inactive === 0 ? pickupBestPointsWhite : pickupBestPointsBlack;
    const carrierPoints =
      inactive === 0 ? carrierNextPointsWhite : carrierNextPointsBlack;
    const manaMoveCount = inactive === 0 ? manaMoveCountWhite : manaMoveCountBlack;
    const manaMoveWindow = manaMoveCount > 0;
    const pickupBestAdjacentOwn =
      inactive === 0 ? pickupBestAdjacentOwnWhite : pickupBestAdjacentOwnBlack;
    const spareManaMove =
      manaMoveCount >= 2 || (manaMoveCount === 1 && !pickupBestAdjacentOwn);
    let window = pickupPoints;
    if (window > 0 && spareManaMove) window += 1;
    let carrierWindow = carrierPoints;
    if (carrierWindow > 0 && manaMoveWindow) carrierWindow += 1;
    if (carrierWindow > window) window = carrierWindow;
    if (manaMoveWindow && window < 1) window = 1;
    const inactiveScore = inactive === 0 ? position.whiteScore : position.blackScore;
    if (window > 0 && inactiveScore + window >= TARGET_SCORE) {
      value += inactiveSign * winsNextTurnThreat;
    }
  }

  return value;
}
