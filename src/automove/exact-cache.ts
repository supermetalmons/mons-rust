import { Color } from "../engine/domain.js";
import { type Location } from "../engine/geometry.js";
import {
  type ExactDrainerPickupPath,
  type ExactSpiritSummary,
  ExactStrategicAnalysis,
  type ExactTurnSummary,
  type ExactTurnTacticalProjection,
} from "./exact-types.js";
import { type AutomoveExecutionContext } from "./execution-context.js";
import { Hash64Table } from "./hash64.js";

const EXACT_ANALYSIS_CACHE_MAX_ENTRIES = 512;
const EXACT_ATTACK_REACH_CACHE_MAX_ENTRIES = 8_192;
const EXACT_CARRIER_STEPS_CACHE_MAX_ENTRIES = 8_192;
const EXACT_DRAINER_SAFETY_CACHE_MAX_ENTRIES = 8_192;
const EXACT_DRAINER_TO_MANA_CACHE_MAX_ENTRIES = 8_192;
const EXACT_PICKUP_PATH_CACHE_MAX_ENTRIES = 8_192;
const EXACT_WALK_THREAT_CACHE_MAX_ENTRIES = 8_192;
export const EXACT_SECURE_MANA_CACHE_MAX_ENTRIES = 4_096;
const EXACT_SPIRIT_REACH_CACHE_MAX_ENTRIES = 4_096;
export const EXACT_SPIRIT_SUMMARY_CACHE_MAX_ENTRIES = 2_048;

const EXACT_SESSION_CACHES = Symbol("exact-session-caches");

class ExactSessionCaches {
  public readonly attackReach = new Hash64Table<boolean>(
    EXACT_ATTACK_REACH_CACHE_MAX_ENTRIES,
  );
  public readonly walkThreat = new Hash64Table<boolean>(
    EXACT_WALK_THREAT_CACHE_MAX_ENTRIES,
  );
  public readonly drainerSafety = new Hash64Table<number>(
    EXACT_DRAINER_SAFETY_CACHE_MAX_ENTRIES,
  );
  public readonly carrierSteps = new Hash64Table<number | undefined>(
    EXACT_CARRIER_STEPS_CACHE_MAX_ENTRIES,
  );
  public readonly drainerToMana = new Hash64Table<number | undefined>(
    EXACT_DRAINER_TO_MANA_CACHE_MAX_ENTRIES,
  );
  public readonly pickupPath = new Hash64Table<
    ExactDrainerPickupPath | undefined
  >(EXACT_PICKUP_PATH_CACHE_MAX_ENTRIES);
  public readonly strategicAnalysis = new Hash64Table<ExactStrategicAnalysis>(
    EXACT_ANALYSIS_CACHE_MAX_ENTRIES,
  );
  public readonly turnSummary = new Hash64Table<ExactTurnSummary>(
    EXACT_ANALYSIS_CACHE_MAX_ENTRIES,
  );
  public readonly turnTacticalProjection =
    new Hash64Table<ExactTurnTacticalProjection>(
      EXACT_ANALYSIS_CACHE_MAX_ENTRIES,
    );
  public readonly secureMana = new Hash64Table<number | undefined>(
    EXACT_SECURE_MANA_CACHE_MAX_ENTRIES,
  );
  public readonly spiritReach = new Hash64Table<
    readonly (readonly [Location, number])[]
  >(EXACT_SPIRIT_REACH_CACHE_MAX_ENTRIES);
  public readonly spiritTacticalSummary = new Hash64Table<ExactSpiritSummary>(
    EXACT_SPIRIT_SUMMARY_CACHE_MAX_ENTRIES,
  );

  public readonly capacity =
    EXACT_ATTACK_REACH_CACHE_MAX_ENTRIES +
    EXACT_WALK_THREAT_CACHE_MAX_ENTRIES +
    EXACT_DRAINER_SAFETY_CACHE_MAX_ENTRIES +
    EXACT_CARRIER_STEPS_CACHE_MAX_ENTRIES +
    EXACT_DRAINER_TO_MANA_CACHE_MAX_ENTRIES +
    EXACT_PICKUP_PATH_CACHE_MAX_ENTRIES +
    EXACT_ANALYSIS_CACHE_MAX_ENTRIES * 3 +
    EXACT_SECURE_MANA_CACHE_MAX_ENTRIES +
    EXACT_SPIRIT_REACH_CACHE_MAX_ENTRIES +
    EXACT_SPIRIT_SUMMARY_CACHE_MAX_ENTRIES;

  public get size(): number {
    return (
      this.attackReach.size +
      this.walkThreat.size +
      this.drainerSafety.size +
      this.carrierSteps.size +
      this.drainerToMana.size +
      this.pickupPath.size +
      this.strategicAnalysis.size +
      this.turnSummary.size +
      this.turnTacticalProjection.size +
      this.secureMana.size +
      this.spiritReach.size +
      this.spiritTacticalSummary.size
    );
  }

  public clear(): void {
    this.attackReach.clear();
    this.walkThreat.clear();
    this.drainerSafety.clear();
    this.carrierSteps.clear();
    this.drainerToMana.clear();
    this.pickupPath.clear();
    this.strategicAnalysis.clear();
    this.turnSummary.clear();
    this.turnTacticalProjection.clear();
    this.secureMana.clear();
    this.spiritReach.clear();
    this.spiritTacticalSummary.clear();
  }
}

export function exactCaches(
  context: AutomoveExecutionContext,
): ExactSessionCaches {
  return context.caches.session.getOrCreate(
    EXACT_SESSION_CACHES,
    () => new ExactSessionCaches(),
  );
}

export function colorKey(color: Color): number {
  return color === Color.White ? 0 : 1;
}

/** Pack up to six small signed fields exactly; callers bypass caching on overflow. */
export function exactCacheTag(
  ...values: readonly number[]
): number | undefined {
  if (values.length > 6) return undefined;
  let tag = 0;
  let multiplier = 1;
  for (const raw of values) {
    if (!Number.isInteger(raw)) return undefined;
    const value = raw;
    if (value < -1 || value > 254) return undefined;
    tag += (value + 1) * multiplier;
    multiplier *= 256;
  }
  return Number.isSafeInteger(tag) ? tag : undefined;
}

export function clearExactStateAnalysisCache(
  context: AutomoveExecutionContext,
): void {
  exactCaches(context).clear();
}
