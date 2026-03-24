use crate::*;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::{BuildHasherDefault, Hasher};

const EXACT_ANALYSIS_CACHE_MAX_ENTRIES: usize = 512;
const EXACT_ATTACK_REACH_CACHE_MAX_ENTRIES: usize = 8192;
const EXACT_CARRIER_STEPS_CACHE_MAX_ENTRIES: usize = 8192;
#[cfg(any(target_arch = "wasm32", test))]
const EXACT_DRAINER_TO_MANA_CACHE_MAX_ENTRIES: usize = 8192;
#[cfg(any(target_arch = "wasm32", test))]
const EXACT_FOLLOWUP_SUMMARY_CACHE_MAX_ENTRIES: usize = 4096;
const EXACT_PICKUP_PATH_CACHE_MAX_ENTRIES: usize = 8192;
const EXACT_SPIRIT_REACH_CACHE_MAX_ENTRIES: usize = 4096;
#[cfg(any(target_arch = "wasm32", test))]
const EXACT_SPIRIT_SUMMARY_CACHE_MAX_ENTRIES: usize = 2048;
const EXACT_WALK_THREAT_CACHE_MAX_ENTRIES: usize = 8192;
#[cfg(any(target_arch = "wasm32", test))]
const EXACT_SECURE_MANA_CACHE_MAX_ENTRIES: usize = 4096;
const EXACT_SPIRIT_UTILITY_CAP: i32 = 6;
const EXACT_BFS_CAPACITY: usize = 128;
const EXACT_LOCATION_STATE_CAPACITY: usize =
    (Config::BOARD_SIZE as usize) * (Config::BOARD_SIZE as usize);
const EXACT_PAYLOAD_VARIANTS: usize = 5;
const EXACT_PAYLOAD_STATE_CAPACITY: usize = EXACT_LOCATION_STATE_CAPACITY * EXACT_PAYLOAD_VARIANTS;
#[cfg(any(target_arch = "wasm32", test))]
const EXACT_SECURE_TOUCHED_ITEMS_CAPACITY: usize = 12;

#[derive(Default)]
struct ExactFastHasher(u64);

impl Hasher for ExactFastHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        let mut hash = if self.0 == 0 {
            0xcbf29ce484222325u64
        } else {
            self.0
        };
        for byte in bytes {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
        self.0 = hash;
    }

    fn write_u64(&mut self, value: u64) {
        self.write(&value.to_le_bytes());
    }
}

type ExactBuildHasher = BuildHasherDefault<ExactFastHasher>;
type ExactHashMap<K, V> = HashMap<K, V, ExactBuildHasher>;
type ExactHashSet<K> = HashSet<K, ExactBuildHasher>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ExactActorPayload {
    None,
    Mana(Mana),
    Bomb,
}

#[derive(Clone)]
struct ExactLocationSeen([bool; EXACT_LOCATION_STATE_CAPACITY]);

impl ExactLocationSeen {
    #[inline]
    fn new() -> Self {
        Self([false; EXACT_LOCATION_STATE_CAPACITY])
    }

    #[inline]
    fn contains(&self, location: Location) -> bool {
        self.0[location.index()]
    }

    #[inline]
    fn insert(&mut self, location: Location) -> bool {
        let slot = location.index();
        if self.0[slot] {
            false
        } else {
            self.0[slot] = true;
            true
        }
    }
}

#[derive(Clone)]
struct ExactPayloadSeen([bool; EXACT_PAYLOAD_STATE_CAPACITY]);

impl ExactPayloadSeen {
    #[inline]
    fn new() -> Self {
        Self([false; EXACT_PAYLOAD_STATE_CAPACITY])
    }

    #[inline]
    fn insert(&mut self, location: Location, payload: ExactActorPayload) -> bool {
        let slot = exact_payload_state_slot(location, payload);
        if self.0[slot] {
            false
        } else {
            self.0[slot] = true;
            true
        }
    }
}

#[inline]
fn exact_payload_state_slot(location: Location, payload: ExactActorPayload) -> usize {
    location.index() * EXACT_PAYLOAD_VARIANTS + exact_payload_variant_index(payload)
}

#[inline]
fn exact_payload_variant_index(payload: ExactActorPayload) -> usize {
    match payload {
        ExactActorPayload::None => 0,
        ExactActorPayload::Mana(Mana::Regular(Color::White)) => 1,
        ExactActorPayload::Mana(Mana::Regular(Color::Black)) => 2,
        ExactActorPayload::Mana(Mana::Supermana) => 3,
        ExactActorPayload::Bomb => 4,
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ExactScorePathWindow {
    pub best_steps: Option<i32>,
    pub multi_pressure: i32,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ExactImmediateScoreWindow {
    pub best_score: i32,
    pub multi_pressure: i32,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ExactDrainerPickupPath {
    pub path_steps: i32,
    pub total_moves: i32,
    pub mana_value: i32,
    #[cfg(any(target_arch = "wasm32", test))]
    pub mana: Mana,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ExactSpiritSummary {
    pub utility: i32,
    pub same_turn_score: bool,
    #[cfg(any(target_arch = "wasm32", test))]
    pub same_turn_score_value: i32,
    pub same_turn_opponent_mana_score: bool,
    #[cfg(any(target_arch = "wasm32", test))]
    pub same_turn_opponent_mana_score_value: i32,
    pub supermana_progress: bool,
    pub opponent_mana_progress: bool,
    pub next_turn_setup_gain: i32,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ExactColorSummary {
    pub score_path_window: ExactScorePathWindow,
    pub immediate_window: ExactImmediateScoreWindow,
    pub best_drainer_pickup: Option<ExactDrainerPickupPath>,
    #[cfg(any(target_arch = "wasm32", test))]
    pub best_carrier_steps: Option<i32>,
    #[cfg(any(target_arch = "wasm32", test))]
    pub best_drainer_to_mana_steps: Option<i32>,
    pub spirit: ExactSpiritSummary,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ExactTurnSummary {
    pub can_attack_opponent_drainer: bool,
    pub safe_supermana_progress: bool,
    pub safe_supermana_progress_steps: Option<i32>,
    pub safe_opponent_mana_progress: bool,
    pub safe_opponent_mana_progress_steps: Option<i32>,
    pub spirit_assisted_supermana_progress: bool,
    pub spirit_assisted_opponent_mana_progress: bool,
    pub spirit_assisted_score: bool,
    pub spirit_assisted_score_value: i32,
    pub spirit_assisted_denial: bool,
    pub spirit_assisted_denial_value: i32,
    pub same_turn_score_window_value: i32,
    pub score_path_best_steps: Option<i32>,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ExactTurnTacticalProjection {
    pub safe_supermana_progress: bool,
    pub safe_supermana_progress_steps: Option<i32>,
    pub safe_opponent_mana_progress: bool,
    pub safe_opponent_mana_progress_steps: Option<i32>,
    pub spirit_assisted_score: bool,
    pub spirit_assisted_score_value: i32,
    pub spirit_assisted_denial: bool,
    pub spirit_assisted_denial_value: i32,
    pub same_turn_score_window_value: i32,
}

#[cfg(any(target_arch = "wasm32", test))]
pub(crate) const EXACT_TURN_TACTICAL_NEED_SUPERMANA_PROGRESS: u8 = 1 << 0;
#[cfg(any(target_arch = "wasm32", test))]
pub(crate) const EXACT_TURN_TACTICAL_NEED_OPPONENT_MANA_PROGRESS: u8 = 1 << 1;
#[cfg(any(target_arch = "wasm32", test))]
pub(crate) const EXACT_TURN_TACTICAL_NEED_SPIRIT_SCORE: u8 = 1 << 2;
#[cfg(any(target_arch = "wasm32", test))]
pub(crate) const EXACT_TURN_TACTICAL_NEED_SPIRIT_DENIAL: u8 = 1 << 3;
#[cfg(any(target_arch = "wasm32", test))]
pub(crate) const EXACT_TURN_TACTICAL_NEED_SCORE_WINDOW: u8 = 1 << 4;
#[cfg(any(target_arch = "wasm32", test))]
const EXACT_TACTICAL_SPIRIT_NEED_SCORE: u8 = 1 << 0;
#[cfg(any(target_arch = "wasm32", test))]
const EXACT_TACTICAL_SPIRIT_NEED_DENIAL: u8 = 1 << 1;
#[cfg(any(target_arch = "wasm32", test))]
const EXACT_TACTICAL_SPIRIT_NEED_PROGRESS: u8 = 1 << 2;

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ExactOpportunityBudget {
    pub remaining_mon_moves: i32,
    pub can_use_action: bool,
    pub can_move_mana: bool,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ExactOpportunityDelta {
    pub same_turn_score_window_value: i32,
    pub spirit_gain: i32,
    pub opponent_window_deny_gain: i32,
    pub drainer_attack_available: bool,
    pub drainer_safety: i32,
    pub safe_supermana_progress_steps: Option<i32>,
    pub safe_opponent_mana_progress_steps: Option<i32>,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ExactOpportunityContext {
    pub budget: ExactOpportunityBudget,
    #[allow(dead_code)]
    pub turn: ExactTurnSummary,
    pub delta: ExactOpportunityDelta,
    pub opponent_can_win_immediately: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExactColorSummaryMode {
    #[cfg(any(target_arch = "wasm32", test))]
    ActiveTactical,
    PassiveStrategic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ExactPickupFilter {
    Any,
    #[cfg(any(target_arch = "wasm32", test))]
    Wanted(Mana),
}

impl ExactPickupFilter {
    #[inline]
    fn matches(self, _mana: Mana) -> bool {
        match self {
            ExactPickupFilter::Any => true,
            #[cfg(any(target_arch = "wasm32", test))]
            ExactPickupFilter::Wanted(wanted) => _mana == wanted,
        }
    }
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug, Clone, Copy, Default)]
struct ExactFollowupSummary {
    best_score_steps: Option<i32>,
    opponent_best_score_steps: Option<i32>,
    immediate_score: i32,
    immediate_opponent_mana_score: i32,
    secure_supermana: bool,
    secure_opponent_mana: bool,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ExactStateAnalysis {
    pub white: ExactColorSummary,
    pub black: ExactColorSummary,
}

#[cfg(any(target_arch = "wasm32", test))]
impl ExactStateAnalysis {
    #[inline]
    pub(crate) fn color_summary(self, color: Color) -> ExactColorSummary {
        if color == Color::White {
            self.white
        } else {
            self.black
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ExactStrategicAnalysis {
    pub white: ExactColorSummary,
    pub black: ExactColorSummary,
}

impl ExactStrategicAnalysis {
    #[inline]
    pub(crate) fn color_summary(self, color: Color) -> ExactColorSummary {
        if color == Color::White {
            self.white
        } else {
            self.black
        }
    }
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Default)]
pub(crate) struct ExactStateAnalysisCache {
    entries: ExactHashMap<u64, ExactStateAnalysis>,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Default)]
struct ExactTurnSummaryCache {
    entries: ExactHashMap<u64, ExactTurnSummary>,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ExactTurnTacticalProjectionKey {
    state_hash: u64,
    color: Color,
    remaining_mon_moves: i32,
    can_use_action: bool,
    flags: u8,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Default)]
struct ExactTurnTacticalProjectionCache {
    entries: ExactHashMap<ExactTurnTacticalProjectionKey, ExactTurnTacticalProjection>,
}

#[derive(Default)]
pub(crate) struct ExactStrategicAnalysisCache {
    entries: ExactHashMap<u64, ExactStrategicAnalysis>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ExactAttackQueryKey {
    board_hash: u64,
    attacker_color: Color,
    target_color: Color,
    target: Location,
    remaining_moves: i32,
    can_use_action: bool,
}

#[derive(Default)]
struct ExactAttackReachCache {
    entries: ExactHashMap<ExactAttackQueryKey, bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ExactCarrierStepsQueryKey {
    board_hash: u64,
    start: Location,
    mana: Mana,
}

#[derive(Default)]
struct ExactCarrierStepsCache {
    entries: ExactHashMap<ExactCarrierStepsQueryKey, Option<i32>>,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ExactDrainerToManaQueryKey {
    board_hash: u64,
    color: Color,
    start: Location,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Default)]
struct ExactDrainerToManaCache {
    entries: ExactHashMap<ExactDrainerToManaQueryKey, Option<i32>>,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ExactFollowupSummaryKey {
    board_hash: u64,
    color: Color,
    remaining_moves: i32,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Default)]
struct ExactFollowupSummaryCache {
    entries: ExactHashMap<ExactFollowupSummaryKey, ExactFollowupSummary>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ExactPickupPathQueryKey {
    board_hash: u64,
    color: Color,
    start: Location,
    max_steps: Option<i32>,
    filter: ExactPickupFilter,
}

#[derive(Default)]
struct ExactPickupPathCache {
    entries: ExactHashMap<ExactPickupPathQueryKey, Option<ExactDrainerPickupPath>>,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ExactSpiritSummaryKey {
    board_hash: u64,
    color: Color,
    remaining_mon_moves: i32,
    can_use_action: bool,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Default)]
struct ExactSpiritSummaryCache {
    entries: ExactHashMap<ExactSpiritSummaryKey, ExactSpiritSummary>,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ExactTacticalSpiritSummaryKey {
    board_hash: u64,
    color: Color,
    remaining_mon_moves: i32,
    can_use_action: bool,
    fields: u8,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Default)]
struct ExactSpiritTacticalSummaryCache {
    entries: ExactHashMap<ExactTacticalSpiritSummaryKey, ExactSpiritSummary>,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ExactTacticalSpiritAfterWindowKey {
    board_hash: u64,
    remaining_mon_moves: i32,
    need_score: bool,
    need_denial: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ExactSpiritReachQueryKey {
    board_hash: u64,
    start: Location,
    color: Color,
    remaining_mon_moves: i32,
}

#[derive(Default)]
struct ExactSpiritReachCache {
    entries: ExactHashMap<ExactSpiritReachQueryKey, Vec<(Location, i32)>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ExactWalkThreatQueryKey {
    board_hash: u64,
    color: Color,
    location: Location,
    angel_nearby: bool,
}

#[derive(Default)]
struct ExactWalkThreatCache {
    entries: ExactHashMap<ExactWalkThreatQueryKey, bool>,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ExactSecureManaStateKey {
    board_hash: u64,
    active_color: Color,
    mons_moves_count: i32,
    white_regular_mana_count: u8,
    black_regular_mana_count: u8,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ExactSecureManaQueryKey {
    state: ExactSecureManaStateKey,
    color: Color,
    wanted: Mana,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Default)]
struct ExactSecureManaCache {
    entries: ExactHashMap<ExactSecureManaQueryKey, Option<i32>>,
    visiting: ExactHashSet<ExactSecureManaQueryKey>,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ExactQueryDiagnostics {
    #[cfg(any(target_arch = "wasm32", test))]
    pub exact_turn_summary_builds: u32,
    #[cfg(any(target_arch = "wasm32", test))]
    pub active_tactical_summary_builds: u32,
    pub passive_strategic_summary_builds: u32,
    #[cfg(any(target_arch = "wasm32", test))]
    pub exact_spirit_summary_calls: u32,
    #[cfg(any(target_arch = "wasm32", test))]
    pub exact_spirit_summary_cache_hits: u32,
    #[cfg(any(target_arch = "wasm32", test))]
    pub tactical_spirit_summary_calls: u32,
    #[cfg(any(target_arch = "wasm32", test))]
    pub tactical_spirit_summary_cache_hits: u32,
    pub passive_spirit_summary_calls: u32,
    #[cfg(any(target_arch = "wasm32", test))]
    pub exact_followup_summary_calls: u32,
    #[cfg(any(target_arch = "wasm32", test))]
    pub exact_followup_summary_cache_hits: u32,
    #[cfg(any(target_arch = "wasm32", test))]
    pub exact_secure_mana_calls: u32,
    #[cfg(any(target_arch = "wasm32", test))]
    pub exact_secure_mana_cache_hits: u32,
    pub pickup_path_calls: u32,
    pub pickup_path_cache_hits: u32,
    pub pickup_path_cache_misses: u32,
}

thread_local! {
    #[cfg(any(target_arch = "wasm32", test))]
    static EXACT_STATE_ANALYSIS_CACHE: RefCell<ExactStateAnalysisCache> =
        RefCell::new(ExactStateAnalysisCache::default());
    #[cfg(any(target_arch = "wasm32", test))]
    static EXACT_TURN_SUMMARY_CACHE: RefCell<ExactTurnSummaryCache> =
        RefCell::new(ExactTurnSummaryCache::default());
    #[cfg(any(target_arch = "wasm32", test))]
    static EXACT_TURN_TACTICAL_PROJECTION_CACHE: RefCell<ExactTurnTacticalProjectionCache> =
        RefCell::new(ExactTurnTacticalProjectionCache::default());
    static EXACT_STRATEGIC_ANALYSIS_CACHE: RefCell<ExactStrategicAnalysisCache> =
        RefCell::new(ExactStrategicAnalysisCache::default());
    static EXACT_ATTACK_REACH_CACHE: RefCell<ExactAttackReachCache> =
        RefCell::new(ExactAttackReachCache::default());
    static EXACT_CARRIER_STEPS_CACHE: RefCell<ExactCarrierStepsCache> =
        RefCell::new(ExactCarrierStepsCache::default());
    #[cfg(any(target_arch = "wasm32", test))]
    static EXACT_DRAINER_TO_MANA_CACHE: RefCell<ExactDrainerToManaCache> =
        RefCell::new(ExactDrainerToManaCache::default());
    #[cfg(any(target_arch = "wasm32", test))]
    static EXACT_FOLLOWUP_SUMMARY_CACHE: RefCell<ExactFollowupSummaryCache> =
        RefCell::new(ExactFollowupSummaryCache::default());
    static EXACT_PICKUP_PATH_CACHE: RefCell<ExactPickupPathCache> =
        RefCell::new(ExactPickupPathCache::default());
    static EXACT_SPIRIT_REACH_CACHE: RefCell<ExactSpiritReachCache> =
        RefCell::new(ExactSpiritReachCache::default());
    #[cfg(any(target_arch = "wasm32", test))]
    static EXACT_SPIRIT_SUMMARY_CACHE: RefCell<ExactSpiritSummaryCache> =
        RefCell::new(ExactSpiritSummaryCache::default());
    #[cfg(any(target_arch = "wasm32", test))]
    static EXACT_SPIRIT_TACTICAL_SUMMARY_CACHE: RefCell<ExactSpiritTacticalSummaryCache> =
        RefCell::new(ExactSpiritTacticalSummaryCache::default());
    static EXACT_WALK_THREAT_CACHE: RefCell<ExactWalkThreatCache> =
        RefCell::new(ExactWalkThreatCache::default());
    #[cfg(any(target_arch = "wasm32", test))]
    static EXACT_SECURE_MANA_CACHE: RefCell<ExactSecureManaCache> =
        RefCell::new(ExactSecureManaCache::default());
    #[cfg(test)]
    static EXACT_QUERY_DIAGNOSTICS: RefCell<ExactQueryDiagnostics> =
        RefCell::new(ExactQueryDiagnostics::default());
}

#[cfg(test)]
#[inline]
fn update_exact_query_diagnostics(update: impl FnOnce(&mut ExactQueryDiagnostics)) {
    EXACT_QUERY_DIAGNOSTICS.with(|diagnostics| update(&mut diagnostics.borrow_mut()));
}

#[cfg(not(test))]
#[inline]
fn update_exact_query_diagnostics(_: impl FnOnce(&mut ExactQueryDiagnostics)) {}

#[cfg(test)]
#[inline]
pub(crate) fn clear_exact_query_diagnostics() {
    EXACT_QUERY_DIAGNOSTICS.with(|diagnostics| {
        *diagnostics.borrow_mut() = ExactQueryDiagnostics::default();
    });
}

#[cfg(test)]
#[inline]
pub(crate) fn exact_query_diagnostics_snapshot() -> ExactQueryDiagnostics {
    EXACT_QUERY_DIAGNOSTICS.with(|diagnostics| *diagnostics.borrow())
}

#[cfg(any(target_arch = "wasm32", test))]
#[inline]
pub(crate) fn clear_exact_state_analysis_cache() {
    EXACT_STATE_ANALYSIS_CACHE.with(|cache| cache.borrow_mut().entries.clear());
    EXACT_TURN_SUMMARY_CACHE.with(|cache| cache.borrow_mut().entries.clear());
    EXACT_TURN_TACTICAL_PROJECTION_CACHE.with(|cache| cache.borrow_mut().entries.clear());
    EXACT_STRATEGIC_ANALYSIS_CACHE.with(|cache| cache.borrow_mut().entries.clear());
    EXACT_ATTACK_REACH_CACHE.with(|cache| cache.borrow_mut().entries.clear());
    EXACT_CARRIER_STEPS_CACHE.with(|cache| cache.borrow_mut().entries.clear());
    EXACT_DRAINER_TO_MANA_CACHE.with(|cache| cache.borrow_mut().entries.clear());
    EXACT_FOLLOWUP_SUMMARY_CACHE.with(|cache| cache.borrow_mut().entries.clear());
    EXACT_PICKUP_PATH_CACHE.with(|cache| cache.borrow_mut().entries.clear());
    EXACT_SPIRIT_REACH_CACHE.with(|cache| cache.borrow_mut().entries.clear());
    EXACT_SPIRIT_SUMMARY_CACHE.with(|cache| cache.borrow_mut().entries.clear());
    EXACT_SPIRIT_TACTICAL_SUMMARY_CACHE.with(|cache| cache.borrow_mut().entries.clear());
    EXACT_WALK_THREAT_CACHE.with(|cache| cache.borrow_mut().entries.clear());
    EXACT_SECURE_MANA_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        cache.entries.clear();
        cache.visiting.clear();
    });
}

#[cfg(any(target_arch = "wasm32", test))]
pub(crate) fn exact_state_analysis(game: &MonsGame) -> ExactStateAnalysis {
    let key = exact_search_state_hash(game);
    exact_state_analysis_with_search_hash(game, key)
}

#[cfg(any(target_arch = "wasm32", test))]
pub(crate) fn exact_state_analysis_with_search_hash(
    game: &MonsGame,
    key: u64,
) -> ExactStateAnalysis {
    EXACT_STATE_ANALYSIS_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Some(cached) = cache.entries.get(&key).copied() {
            return cached;
        }
        let built = build_exact_state_analysis(game);
        if cache.entries.len() >= EXACT_ANALYSIS_CACHE_MAX_ENTRIES
            && !cache.entries.contains_key(&key)
        {
            cache.entries.clear();
        }
        cache.entries.insert(key, built);
        built
    })
}

pub(crate) fn exact_strategic_analysis(game: &MonsGame) -> ExactStrategicAnalysis {
    let key = exact_search_state_hash(game);
    exact_strategic_analysis_with_search_hash(game, key)
}

pub(crate) fn exact_strategic_analysis_with_search_hash(
    game: &MonsGame,
    key: u64,
) -> ExactStrategicAnalysis {
    EXACT_STRATEGIC_ANALYSIS_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Some(cached) = cache.entries.get(&key).copied() {
            return cached;
        }
        let built = build_exact_strategic_analysis(game);
        if cache.entries.len() >= EXACT_ANALYSIS_CACHE_MAX_ENTRIES
            && !cache.entries.contains_key(&key)
        {
            cache.entries.clear();
        }
        cache.entries.insert(key, built);
        built
    })
}

#[cfg(any(target_arch = "wasm32", test))]
#[inline]
pub(crate) fn exact_turn_summary(game: &MonsGame, color: Color) -> ExactTurnSummary {
    let key = exact_search_state_hash(game);
    exact_turn_summary_with_search_hash(game, color, key)
}

#[cfg(any(target_arch = "wasm32", test))]
#[inline]
pub(crate) fn exact_turn_summary_with_search_hash(
    game: &MonsGame,
    color: Color,
    key: u64,
) -> ExactTurnSummary {
    if game.active_color != color {
        ExactTurnSummary {
            ..ExactTurnSummary::default()
        }
    } else {
        EXACT_TURN_SUMMARY_CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();
            if let Some(cached) = cache.entries.get(&key).copied() {
                return cached;
            }
            let built = build_exact_turn_summary(game);
            if cache.entries.len() >= EXACT_ANALYSIS_CACHE_MAX_ENTRIES
                && !cache.entries.contains_key(&key)
            {
                cache.entries.clear();
            }
            cache.entries.insert(key, built);
            built
        })
    }
}

#[cfg(any(target_arch = "wasm32", test))]
#[inline]
pub(crate) fn exact_turn_tactical_projection(
    game: &MonsGame,
    color: Color,
    flags: u8,
) -> ExactTurnTacticalProjection {
    exact_turn_tactical_projection_with_search_hash(
        game,
        color,
        exact_search_state_hash(game),
        flags,
    )
}

#[cfg(any(target_arch = "wasm32", test))]
#[inline]
pub(crate) fn exact_turn_tactical_projection_with_search_hash(
    game: &MonsGame,
    color: Color,
    key: u64,
    flags: u8,
) -> ExactTurnTacticalProjection {
    if flags == 0 || game.active_color != color {
        return ExactTurnTacticalProjection::default();
    }

    let remaining_mon_moves = (Config::MONS_MOVES_PER_TURN - game.mons_moves_count).max(0);
    let can_use_action = game.player_can_use_action();
    let cache_key = ExactTurnTacticalProjectionKey {
        state_hash: key,
        color,
        remaining_mon_moves,
        can_use_action,
        flags,
    };
    EXACT_TURN_TACTICAL_PROJECTION_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Some(cached) = cache.entries.get(&cache_key).copied() {
            return cached;
        }
        let built = build_exact_turn_tactical_projection(game, flags);
        if cache.entries.len() >= EXACT_ANALYSIS_CACHE_MAX_ENTRIES
            && !cache.entries.contains_key(&cache_key)
        {
            cache.entries.clear();
        }
        cache.entries.insert(cache_key, built);
        built
    })
}

#[cfg(any(target_arch = "wasm32", test))]
pub(crate) fn can_attack_opponent_drainer_this_turn(game: &MonsGame, color: Color) -> bool {
    exact_turn_summary(game, color).can_attack_opponent_drainer
}

#[cfg(any(target_arch = "wasm32", test))]
pub(crate) fn exact_opportunity_context(game: &MonsGame, color: Color) -> ExactOpportunityContext {
    let key = exact_search_state_hash(game);
    exact_opportunity_context_with_search_hash(game, color, key)
}

#[cfg(any(target_arch = "wasm32", test))]
pub(crate) fn exact_opportunity_context_with_search_hash(
    game: &MonsGame,
    color: Color,
    key: u64,
) -> ExactOpportunityContext {
    if game.active_color != color {
        return ExactOpportunityContext::default();
    }

    let budget = ExactOpportunityBudget {
        remaining_mon_moves: (Config::MONS_MOVES_PER_TURN - game.mons_moves_count).max(0),
        can_use_action: game.player_can_use_action(),
        can_move_mana: game.player_can_move_mana(),
    };
    let turn = exact_turn_summary_with_search_hash(game, color, key);
    let drainer_safety = exact_own_drainer_safety_score(&game.board, color);
    let opponent = color.other();
    let opponent_score = if opponent == Color::White {
        game.white_score
    } else {
        game.black_score
    };
    let opponent_needed = Config::TARGET_SCORE.saturating_sub(opponent_score);
    let opponent_immediate = exact_strategic_analysis_with_search_hash(game, key)
        .color_summary(opponent)
        .immediate_window
        .best_score;
    let opponent_can_win_immediately = opponent_needed > 0 && opponent_immediate >= opponent_needed;
    let opponent_window_deny_gain = if opponent_needed > 0 && turn.same_turn_score_window_value > 0
    {
        turn.same_turn_score_window_value.min(opponent_needed)
    } else {
        0
    };

    ExactOpportunityContext {
        budget,
        turn,
        delta: ExactOpportunityDelta {
            same_turn_score_window_value: turn.same_turn_score_window_value,
            spirit_gain: turn
                .spirit_assisted_score_value
                .max(turn.spirit_assisted_denial_value),
            opponent_window_deny_gain,
            drainer_attack_available: turn.can_attack_opponent_drainer,
            drainer_safety,
            safe_supermana_progress_steps: turn.safe_supermana_progress_steps,
            safe_opponent_mana_progress_steps: turn.safe_opponent_mana_progress_steps,
        },
        opponent_can_win_immediately,
    }
}

#[cfg(any(target_arch = "wasm32", test))]
fn exact_own_drainer_safety_score(board: &Board, color: Color) -> i32 {
    let Some(drainer_location) = find_awake_drainer(board, color) else {
        return 0;
    };
    let angel_nearby = board
        .find_awake_angel(color)
        .map_or(false, |angel| angel.distance(&drainer_location) == 1);
    let immediate = is_drainer_under_immediate_threat(board, color, drainer_location, angel_nearby);
    let walk = is_drainer_under_walk_threat(board, color, drainer_location, angel_nearby);
    let exact_safe = is_drainer_exactly_safe_next_turn_on_board(board, color, drainer_location);

    if exact_safe && !immediate && !walk {
        2
    } else if exact_safe {
        1
    } else if immediate || walk {
        -2
    } else {
        -1
    }
}

pub(crate) fn can_attack_target_on_board(
    board: &Board,
    attacker_color: Color,
    target_color: Color,
    target: Location,
    remaining_moves: i32,
    can_use_action: bool,
) -> bool {
    if remaining_moves < 0 || !can_use_action || board.item(target).is_none() {
        return false;
    }

    let key = ExactAttackQueryKey {
        board_hash: exact_board_hash(board),
        attacker_color,
        target_color,
        target,
        remaining_moves,
        can_use_action,
    };
    if let Some(cached) =
        EXACT_ATTACK_REACH_CACHE.with(|cache| cache.borrow().entries.get(&key).copied())
    {
        return cached;
    }

    let result = can_attack_target_on_board_uncached(
        board,
        attacker_color,
        target_color,
        target,
        remaining_moves,
        can_use_action,
    );
    EXACT_ATTACK_REACH_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if cache.entries.len() >= EXACT_ATTACK_REACH_CACHE_MAX_ENTRIES
            && !cache.entries.contains_key(&key)
        {
            cache.entries.clear();
        }
        cache.entries.insert(key, result);
    });
    result
}

fn can_attack_target_on_board_uncached(
    board: &Board,
    attacker_color: Color,
    target_color: Color,
    target: Location,
    remaining_moves: i32,
    _can_use_action: bool,
) -> bool {
    let target_guarded = exact_is_location_guarded_by_angel(board, target_color, target);

    for (start, item) in board.occupied() {
        let mon = match item {
            Item::Mon { mon }
            | Item::MonWithMana { mon, .. }
            | Item::MonWithConsumable { mon, .. } => mon,
            Item::Mana { .. } | Item::Consumable { .. } => continue,
        };
        if mon.color != attacker_color || mon.is_fainted() {
            continue;
        }
        let allow_pick_bomb = !matches!(item, Item::MonWithMana { .. });
        let start_payload = match item {
            Item::MonWithConsumable {
                consumable: Consumable::Bomb,
                ..
            } => ExactActorPayload::Bomb,
            _ => ExactActorPayload::None,
        };
        let mut queue = VecDeque::with_capacity(EXACT_BFS_CAPACITY);
        let mut seen = ExactPayloadSeen::new();
        queue.push_back((start, start_payload, 0));
        seen.insert(start, start_payload);

        while let Some((location, payload, steps)) = queue.pop_front() {
            if steps > remaining_moves {
                continue;
            }
            if payload == ExactActorPayload::Bomb
                && board.item(target).is_some()
                && location.distance(&target) <= 3
            {
                return true;
            }
            if !matches!(board.square(location), Square::MonBase { .. }) && !target_guarded {
                if mon.kind == MonKind::Mystic
                    && (location.i - target.i).abs() == 2
                    && (location.j - target.j).abs() == 2
                {
                    return true;
                }
                if mon.kind == MonKind::Demon && demon_has_line_attack(board, location, target) {
                    return true;
                }
            }
            if steps == remaining_moves {
                continue;
            }
            for &next in location.nearby_locations_ref() {
                if let Some(next_payload) = actor_payload_after_move(
                    board,
                    mon.kind,
                    mon.color,
                    payload,
                    next,
                    allow_pick_bomb,
                ) {
                    if seen.insert(next, next_payload) {
                        queue.push_back((next, next_payload, steps + 1));
                    }
                }
            }
        }
    }
    false
}

fn exact_board_hash(board: &Board) -> u64 {
    let mut state = 0x6a09e667f3bcc909u64;
    for (idx, item) in board.items.iter().enumerate() {
        let Some(item) = item else { continue };
        let entry = ((idx as u64)
            .wrapping_add(1)
            .wrapping_mul(0x9e3779b185ebca87))
            ^ exact_hash_item(*item);
        state ^= exact_mix_u64(entry);
        state = state.rotate_left(17).wrapping_mul(0x94d049bb133111eb);
    }
    exact_mix_u64(state)
}

fn exact_search_state_hash(game: &MonsGame) -> u64 {
    let mut state = 0x6a09e667f3bcc909u64;
    for (idx, item) in game.board.items.iter().enumerate() {
        let Some(item) = item else { continue };
        let entry = ((idx as u64)
            .wrapping_add(1)
            .wrapping_mul(0x9e3779b185ebca87))
            ^ exact_search_hash_item(*item);
        state ^= exact_search_mix_u64(entry);
        state = state.rotate_left(17).wrapping_mul(0x94d049bb133111eb);
    }

    state ^= exact_search_mix_u64(game.white_score as i64 as u64 ^ 0x11);
    state ^= exact_search_mix_u64(game.black_score as i64 as u64 ^ 0x23);
    state ^= exact_search_mix_u64(exact_search_hash_color(game.active_color) ^ 0x35);
    state ^= exact_search_mix_u64(game.actions_used_count as i64 as u64 ^ 0x47);
    state ^= exact_search_mix_u64(game.mana_moves_count as i64 as u64 ^ 0x59);
    state ^= exact_search_mix_u64(game.mons_moves_count as i64 as u64 ^ 0x6b);
    state ^= exact_search_mix_u64(game.white_potions_count as i64 as u64 ^ 0x7d);
    state ^= exact_search_mix_u64(game.black_potions_count as i64 as u64 ^ 0x8f);
    state ^= exact_search_mix_u64(game.turn_number as i64 as u64 ^ 0xa1);
    exact_search_mix_u64(state)
}

#[inline]
fn exact_walk_destination_plausible(board: &Board, actor: Location, destination: Location) -> bool {
    let Some(actor_mon) = board.item(actor).and_then(|item| item.mon()).copied() else {
        return false;
    };
    match board.item(destination) {
        Some(Item::Mon { .. })
        | Some(Item::MonWithMana { .. })
        | Some(Item::MonWithConsumable { .. }) => false,
        Some(Item::Mana { .. }) | Some(Item::Consumable { .. }) | None => {
            match board.square(destination) {
                Square::Regular
                | Square::ConsumableBase
                | Square::ManaBase { .. }
                | Square::ManaPool { .. } => true,
                Square::SupermanaBase => actor_mon.kind == MonKind::Drainer,
                Square::MonBase { kind, color } => {
                    actor_mon.kind == kind && actor_mon.color == color
                }
            }
        }
    }
}

#[inline]
fn exact_search_hash_item(item: Item) -> u64 {
    match item {
        Item::Mon { mon } => 0x100 | exact_search_hash_mon(mon),
        Item::Mana { mana } => 0x200 | exact_search_hash_mana(mana),
        Item::MonWithMana { mon, mana } => {
            0x300 | exact_search_hash_mon(mon) | (exact_search_hash_mana(mana) << 16)
        }
        Item::MonWithConsumable { mon, consumable } => {
            0x400 | exact_search_hash_mon(mon) | (exact_search_hash_consumable(consumable) << 16)
        }
        Item::Consumable { consumable } => 0x500 | exact_search_hash_consumable(consumable),
    }
}

#[inline]
fn exact_search_hash_mon(mon: Mon) -> u64 {
    exact_search_hash_mon_kind(mon.kind)
        | (exact_search_hash_color(mon.color) << 4)
        | (((mon.cooldown as i64 as u64) & 0xff) << 8)
}

#[inline]
fn exact_search_hash_mon_kind(kind: MonKind) -> u64 {
    match kind {
        MonKind::Demon => 1,
        MonKind::Drainer => 2,
        MonKind::Angel => 3,
        MonKind::Spirit => 4,
        MonKind::Mystic => 5,
    }
}

#[inline]
fn exact_search_hash_color(color: Color) -> u64 {
    match color {
        Color::White => 1,
        Color::Black => 2,
    }
}

#[inline]
fn exact_search_hash_mana(mana: Mana) -> u64 {
    match mana {
        Mana::Regular(color) => 0x10 | exact_search_hash_color(color),
        Mana::Supermana => 0x20,
    }
}

#[inline]
fn exact_search_hash_consumable(consumable: Consumable) -> u64 {
    match consumable {
        Consumable::Potion => 1,
        Consumable::Bomb => 2,
        Consumable::BombOrPotion => 3,
    }
}

#[inline]
fn exact_search_mix_u64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e3779b97f4a7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
    value ^ (value >> 31)
}

#[cfg(any(target_arch = "wasm32", test))]
#[inline]
fn exact_secure_board_entry_hash(index: usize, item: Item) -> u64 {
    let entry = ((index as u64)
        .wrapping_add(1)
        .wrapping_mul(0x94d049bb133111eb))
        ^ exact_search_hash_item(item).wrapping_mul(0x9e3779b185ebca87);
    exact_search_mix_u64(entry)
}

#[cfg(any(target_arch = "wasm32", test))]
fn exact_secure_board_hash(board: &Board) -> u64 {
    exact_secure_board_state(board).0
}

#[cfg(any(target_arch = "wasm32", test))]
fn exact_secure_board_state(board: &Board) -> (u64, u8, u8) {
    let mut state = 0xa0761d6478bd642fu64;
    let mut white_regular = 0u8;
    let mut black_regular = 0u8;
    for (index, item) in board.items.iter().enumerate() {
        let Some(item) = item else { continue };
        state ^= exact_secure_board_entry_hash(index, *item);
        if let Item::Mana {
            mana: Mana::Regular(color),
        } = item
        {
            match color {
                Color::White => white_regular = white_regular.saturating_add(1),
                Color::Black => black_regular = black_regular.saturating_add(1),
            }
        }
    }
    (state, white_regular, black_regular)
}

#[cfg(any(target_arch = "wasm32", test))]
#[inline]
fn exact_adjust_regular_mana_counts(white: &mut u8, black: &mut u8, mana: Mana, delta: i8) {
    let count = match mana {
        Mana::Regular(Color::White) => white,
        Mana::Regular(Color::Black) => black,
        Mana::Supermana => return,
    };

    if delta < 0 {
        *count = count.saturating_sub((-delta) as u8);
    } else if delta > 0 {
        *count = count.saturating_add(delta as u8);
    }
}

#[cfg(any(target_arch = "wasm32", test))]
#[inline]
fn exact_secure_mana_state_key(game: &MonsGame) -> ExactSecureManaStateKey {
    exact_secure_mana_state_key_from_board(&game.board, game.active_color, game.mons_moves_count)
}

#[cfg(any(target_arch = "wasm32", test))]
#[inline]
fn exact_secure_mana_state_key_from_board(
    board: &Board,
    active_color: Color,
    mons_moves_count: i32,
) -> ExactSecureManaStateKey {
    let (board_hash, white_regular_mana_count, black_regular_mana_count) =
        exact_secure_board_state(board);
    ExactSecureManaStateKey {
        board_hash,
        active_color,
        mons_moves_count,
        white_regular_mana_count,
        black_regular_mana_count,
    }
}

#[inline]
fn exact_hash_item(item: Item) -> u64 {
    match item {
        Item::Mon { mon } => 0x100 | exact_hash_mon(mon),
        Item::Mana { mana } => 0x200 | exact_hash_mana(mana),
        Item::MonWithMana { mon, mana } => {
            0x300 | exact_hash_mon(mon) | (exact_hash_mana(mana) << 16)
        }
        Item::MonWithConsumable { mon, consumable } => {
            0x400 | exact_hash_mon(mon) | (exact_hash_consumable(consumable) << 16)
        }
        Item::Consumable { consumable } => 0x500 | exact_hash_consumable(consumable),
    }
}

#[inline]
fn exact_hash_mon(mon: Mon) -> u64 {
    exact_hash_mon_kind(mon.kind)
        | (exact_hash_color(mon.color) << 4)
        | (((mon.cooldown as i64 as u64) & 0xff) << 8)
}

#[inline]
fn exact_hash_mon_kind(kind: MonKind) -> u64 {
    match kind {
        MonKind::Demon => 1,
        MonKind::Drainer => 2,
        MonKind::Angel => 3,
        MonKind::Spirit => 4,
        MonKind::Mystic => 5,
    }
}

#[inline]
fn exact_hash_color(color: Color) -> u64 {
    match color {
        Color::White => 1,
        Color::Black => 2,
    }
}

#[inline]
fn exact_hash_mana(mana: Mana) -> u64 {
    match mana {
        Mana::Regular(color) => 1 | (exact_hash_color(color) << 4),
        Mana::Supermana => 2,
    }
}

#[inline]
fn exact_hash_consumable(consumable: Consumable) -> u64 {
    match consumable {
        Consumable::Bomb => 1,
        Consumable::Potion => 2,
        Consumable::BombOrPotion => 3,
    }
}

#[inline]
fn exact_mix_u64(value: u64) -> u64 {
    let mut mixed = value;
    mixed ^= mixed >> 30;
    mixed = mixed.wrapping_mul(0xbf58476d1ce4e5b9);
    mixed ^= mixed >> 27;
    mixed = mixed.wrapping_mul(0x94d049bb133111eb);
    mixed ^= mixed >> 31;
    mixed
}

pub(crate) fn drainer_immediate_threats(
    board: &Board,
    color: Color,
    location: Location,
) -> (i32, i32) {
    let mut action_threats = 0;
    let mut bomb_threats = 0;
    for (threat_location, item) in board.occupied() {
        let mon = match item {
            Item::Mon { mon }
            | Item::MonWithMana { mon, .. }
            | Item::MonWithConsumable { mon, .. } => mon,
            Item::Mana { .. } | Item::Consumable { .. } => continue,
        };
        if mon.color == color || mon.is_fainted() {
            continue;
        }
        let on_own_base = matches!(board.square(threat_location), Square::MonBase { .. });
        if !on_own_base {
            if mon.kind == MonKind::Mystic
                && (threat_location.i - location.i).abs() == 2
                && (threat_location.j - location.j).abs() == 2
            {
                action_threats += 1;
            } else if mon.kind == MonKind::Demon
                && demon_has_line_attack(board, threat_location, location)
            {
                action_threats += 1;
            }
        }
        if matches!(
            item,
            Item::MonWithConsumable {
                consumable: Consumable::Bomb,
                ..
            }
        ) && !on_own_base
            && threat_location.distance(&location) <= 3
        {
            bomb_threats += 1;
        }
    }
    (action_threats, bomb_threats)
}

pub(crate) fn is_drainer_under_immediate_threat(
    board: &Board,
    color: Color,
    location: Location,
    angel_nearby: bool,
) -> bool {
    let (action_threats, bomb_threats) = drainer_immediate_threats(board, color, location);
    if angel_nearby {
        bomb_threats > 0
    } else {
        action_threats + bomb_threats > 0
    }
}

pub(crate) fn is_drainer_under_walk_threat(
    board: &Board,
    color: Color,
    location: Location,
    angel_nearby: bool,
) -> bool {
    let key = ExactWalkThreatQueryKey {
        board_hash: exact_board_hash(board),
        color,
        location,
        angel_nearby,
    };
    if let Some(cached) =
        EXACT_WALK_THREAT_CACHE.with(|cache| cache.borrow().entries.get(&key).copied())
    {
        return cached;
    }

    let result = is_drainer_under_walk_threat_uncached(board, color, location, angel_nearby);
    EXACT_WALK_THREAT_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if cache.entries.len() >= EXACT_WALK_THREAT_CACHE_MAX_ENTRIES
            && !cache.entries.contains_key(&key)
        {
            cache.entries.clear();
        }
        cache.entries.insert(key, result);
    });
    result
}

fn is_drainer_under_walk_threat_uncached(
    board: &Board,
    color: Color,
    location: Location,
    angel_nearby: bool,
) -> bool {
    if angel_nearby {
        return board.occupied().any(|(threat_location, item)| {
            matches!(
                item,
                Item::MonWithConsumable {
                    mon,
                    consumable: Consumable::Bomb,
                } if mon.color != color
                    && !mon.is_fainted()
                    && !matches!(board.square(threat_location), Square::MonBase { .. })
                    && threat_location.distance(&location) <= 4
            )
        });
    }

    let valid = Location::valid_range();
    for (threat_location, item) in board.occupied() {
        let mon = match item {
            Item::Mon { mon }
            | Item::MonWithMana { mon, .. }
            | Item::MonWithConsumable { mon, .. } => mon,
            Item::Mana { .. } | Item::Consumable { .. } => continue,
        };
        if mon.color == color || mon.is_fainted() {
            continue;
        }
        if matches!(board.square(threat_location), Square::MonBase { .. }) {
            continue;
        }
        if mon.kind == MonKind::Mystic || mon.kind == MonKind::Demon {
            for dx in -1i32..=1 {
                for dy in -1i32..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let ni = threat_location.i + dx;
                    let nj = threat_location.j + dy;
                    if !valid.contains(&ni) || !valid.contains(&nj) {
                        continue;
                    }
                    let neighbor = Location::new(ni, nj);
                    if board.item(neighbor).is_some() {
                        continue;
                    }
                    if matches!(
                        board.square(neighbor),
                        Square::MonBase { .. } | Square::SupermanaBase
                    ) {
                        continue;
                    }
                    if mon.kind == MonKind::Mystic
                        && (neighbor.i - location.i).abs() == 2
                        && (neighbor.j - location.j).abs() == 2
                    {
                        return true;
                    }
                    if mon.kind == MonKind::Demon
                        && demon_has_line_attack(board, neighbor, location)
                    {
                        return true;
                    }
                }
            }
        }
        if matches!(
            item,
            Item::MonWithConsumable {
                consumable: Consumable::Bomb,
                ..
            }
        ) && threat_location.distance(&location) <= 4
        {
            return true;
        }
    }
    false
}

#[cfg(any(target_arch = "wasm32", test))]
pub(crate) fn is_drainer_exactly_safe_next_turn_on_board(
    board: &Board,
    color: Color,
    location: Location,
) -> bool {
    let angel_nearby = exact_is_location_guarded_by_angel(board, color, location);
    !can_attack_target_on_board(
        board,
        color.other(),
        color,
        location,
        Config::MONS_MOVES_PER_TURN,
        true,
    ) && !is_drainer_under_walk_threat(board, color, location, angel_nearby)
}

fn exact_is_location_guarded_by_angel(board: &Board, color: Color, location: Location) -> bool {
    board
        .find_awake_angel(color)
        .map_or(false, |angel_location| {
            angel_location.distance(&location) == 1
        })
}

#[cfg(any(target_arch = "wasm32", test))]
fn build_exact_state_analysis(game: &MonsGame) -> ExactStateAnalysis {
    let active_color = game.active_color;
    let active_summary =
        build_color_summary(game, active_color, ExactColorSummaryMode::ActiveTactical);
    let passive_summary = build_color_summary(
        game,
        active_color.other(),
        ExactColorSummaryMode::PassiveStrategic,
    );
    if active_color == Color::White {
        ExactStateAnalysis {
            white: active_summary,
            black: passive_summary,
        }
    } else {
        ExactStateAnalysis {
            white: passive_summary,
            black: active_summary,
        }
    }
}

fn build_exact_strategic_analysis(game: &MonsGame) -> ExactStrategicAnalysis {
    ExactStrategicAnalysis {
        white: build_color_summary(game, Color::White, ExactColorSummaryMode::PassiveStrategic),
        black: build_color_summary(game, Color::Black, ExactColorSummaryMode::PassiveStrategic),
    }
}

fn build_color_summary(
    game: &MonsGame,
    color: Color,
    mode: ExactColorSummaryMode,
) -> ExactColorSummary {
    update_exact_query_diagnostics(|diagnostics| match mode {
        #[cfg(any(target_arch = "wasm32", test))]
        ExactColorSummaryMode::ActiveTactical => diagnostics.active_tactical_summary_builds += 1,
        ExactColorSummaryMode::PassiveStrategic => {
            diagnostics.passive_strategic_summary_builds += 1
        }
    });

    let (full_turn_moves, can_use_action) = if game.active_color == color {
        (
            (Config::MONS_MOVES_PER_TURN - game.mons_moves_count).max(0),
            game.player_can_use_action(),
        )
    } else {
        (Config::MONS_MOVES_PER_TURN, true)
    };

    let board_hash = exact_board_hash(&game.board);
    let mut carrier_steps = Vec::new();
    let mut best_carrier_steps = None;
    for (location, item) in game.board.occupied() {
        let Item::MonWithMana { mon, mana } = item else {
            continue;
        };
        if mon.color != color || mon.is_fainted() {
            continue;
        }
        if let Some(steps) =
            exact_carrier_steps_to_any_pool_with_hash(&game.board, location, *mana, board_hash)
        {
            best_carrier_steps =
                Some(best_carrier_steps.map_or(steps, |best: i32| best.min(steps)));
            carrier_steps.push(steps);
        }
    }

    let best_drainer_pickup = find_awake_drainer(&game.board, color).and_then(|location| {
        exact_best_drainer_pickup_path_filtered_with_hash(
            &game.board,
            color,
            location,
            None,
            ExactPickupFilter::Any,
            board_hash,
        )
    });
    #[cfg(any(target_arch = "wasm32", test))]
    let best_drainer_to_mana_steps = find_awake_drainer(&game.board, color)
        .and_then(|location| exact_drainer_to_any_mana_steps(&game.board, color, location));

    if let Some(path) = best_drainer_pickup {
        carrier_steps.push(path.total_moves);
    }
    carrier_steps.sort_unstable();
    carrier_steps.dedup();

    let score_path_window = ExactScorePathWindow {
        best_steps: carrier_steps.first().copied(),
        multi_pressure: exact_multi_pressure_from_steps(carrier_steps.as_slice()),
    };

    let mut immediate_scores = Vec::new();
    for (location, item) in game.board.occupied() {
        let Item::MonWithMana { mon, mana } = item else {
            continue;
        };
        if mon.color != color || mon.is_fainted() {
            continue;
        }
        if let Some(steps) =
            exact_carrier_steps_to_any_pool_with_hash(&game.board, location, *mana, board_hash)
        {
            if steps <= full_turn_moves {
                immediate_scores.push(mana.score(color));
            }
        }
    }
    if let Some(path) = best_drainer_pickup {
        if path.total_moves <= full_turn_moves {
            immediate_scores.push(path.mana_value);
        }
    }
    let spirit = match mode {
        #[cfg(any(target_arch = "wasm32", test))]
        ExactColorSummaryMode::ActiveTactical => {
            let spirit = exact_spirit_summary(&game.board, color, full_turn_moves, can_use_action);
            if spirit.same_turn_score {
                immediate_scores.push(spirit.same_turn_score_value.max(1));
            }
            if spirit.same_turn_opponent_mana_score {
                immediate_scores.push(spirit.same_turn_opponent_mana_score_value.max(1));
            }
            spirit
        }
        ExactColorSummaryMode::PassiveStrategic => {
            exact_passive_spirit_summary(&game.board, color, full_turn_moves, can_use_action)
        }
    };
    immediate_scores.sort_unstable_by(|a, b| b.cmp(a));
    let immediate_window = ExactImmediateScoreWindow {
        best_score: immediate_scores.first().copied().unwrap_or(0),
        multi_pressure: exact_multi_pressure_from_scores(immediate_scores.as_slice()),
    };

    ExactColorSummary {
        score_path_window,
        immediate_window,
        best_drainer_pickup,
        #[cfg(any(target_arch = "wasm32", test))]
        best_carrier_steps,
        #[cfg(any(target_arch = "wasm32", test))]
        best_drainer_to_mana_steps,
        spirit,
    }
}

#[cfg(any(target_arch = "wasm32", test))]
fn build_exact_turn_summary(game: &MonsGame) -> ExactTurnSummary {
    update_exact_query_diagnostics(|diagnostics| diagnostics.exact_turn_summary_builds += 1);

    let color = game.active_color;
    let remaining_moves = (Config::MONS_MOVES_PER_TURN - game.mons_moves_count).max(0);
    let can_use_action = game.player_can_use_action();
    let tactical_spirit = exact_tactical_spirit_summary(
        &game.board,
        color,
        remaining_moves,
        can_use_action,
        EXACT_TACTICAL_SPIRIT_NEED_SCORE
            | EXACT_TACTICAL_SPIRIT_NEED_DENIAL
            | EXACT_TACTICAL_SPIRIT_NEED_PROGRESS,
    );
    let safe_supermana_progress_steps =
        exact_secure_specific_mana_steps_this_turn(game, color, Mana::Supermana);
    let safe_opponent_mana_progress_steps =
        exact_secure_specific_mana_steps_this_turn(game, color, Mana::Regular(color.other()));
    let same_turn_score_window_value =
        exact_best_immediate_score_on_board(&game.board, color, remaining_moves)
            .max(tactical_spirit.same_turn_score_value)
            .max(tactical_spirit.same_turn_opponent_mana_score_value);

    ExactTurnSummary {
        can_attack_opponent_drainer: can_attack_opponent_drainer_exact(game, color),
        safe_supermana_progress: safe_supermana_progress_steps.is_some(),
        safe_supermana_progress_steps,
        safe_opponent_mana_progress: safe_opponent_mana_progress_steps.is_some()
            || tactical_spirit.same_turn_opponent_mana_score,
        safe_opponent_mana_progress_steps,
        spirit_assisted_supermana_progress: tactical_spirit.supermana_progress,
        spirit_assisted_opponent_mana_progress: tactical_spirit.opponent_mana_progress,
        spirit_assisted_score: tactical_spirit.same_turn_score,
        spirit_assisted_score_value: tactical_spirit.same_turn_score_value,
        spirit_assisted_denial: tactical_spirit.same_turn_opponent_mana_score,
        spirit_assisted_denial_value: tactical_spirit.same_turn_opponent_mana_score_value,
        same_turn_score_window_value,
        score_path_best_steps: exact_best_score_steps_on_board(&game.board, color),
    }
}

#[cfg(any(target_arch = "wasm32", test))]
fn build_exact_turn_tactical_projection(game: &MonsGame, flags: u8) -> ExactTurnTacticalProjection {
    let color = game.active_color;
    let remaining_moves = (Config::MONS_MOVES_PER_TURN - game.mons_moves_count).max(0);
    let can_use_action = game.player_can_use_action();
    let need_supermana = flags & EXACT_TURN_TACTICAL_NEED_SUPERMANA_PROGRESS != 0;
    let need_opponent_mana = flags & EXACT_TURN_TACTICAL_NEED_OPPONENT_MANA_PROGRESS != 0;
    let need_spirit_score = flags & EXACT_TURN_TACTICAL_NEED_SPIRIT_SCORE != 0;
    let need_spirit_denial = flags & EXACT_TURN_TACTICAL_NEED_SPIRIT_DENIAL != 0;
    let need_score_window = flags & EXACT_TURN_TACTICAL_NEED_SCORE_WINDOW != 0;
    let include_score_window_denial =
        need_score_window && (need_opponent_mana || need_spirit_denial);
    let mut tactical_spirit_fields = 0;
    if need_spirit_score || need_score_window {
        tactical_spirit_fields |= EXACT_TACTICAL_SPIRIT_NEED_SCORE;
    }
    if need_spirit_denial || include_score_window_denial {
        tactical_spirit_fields |= EXACT_TACTICAL_SPIRIT_NEED_DENIAL;
    }
    let tactical_spirit = if tactical_spirit_fields != 0 {
        exact_tactical_spirit_summary(
            &game.board,
            color,
            remaining_moves,
            can_use_action,
            tactical_spirit_fields,
        )
    } else {
        ExactSpiritSummary::default()
    };
    let safe_supermana_progress_steps = if need_supermana {
        exact_secure_specific_mana_steps_this_turn(game, color, Mana::Supermana)
    } else {
        None
    };
    let safe_opponent_mana_progress_steps = if need_opponent_mana {
        exact_secure_specific_mana_steps_this_turn(game, color, Mana::Regular(color.other()))
    } else {
        None
    };
    let same_turn_score_window_value = if need_score_window {
        exact_best_immediate_score_on_board(&game.board, color, remaining_moves)
            .max(tactical_spirit.same_turn_score_value)
            .max(if include_score_window_denial {
                tactical_spirit.same_turn_opponent_mana_score_value
            } else {
                0
            })
    } else {
        0
    };

    ExactTurnTacticalProjection {
        safe_supermana_progress: safe_supermana_progress_steps.is_some(),
        safe_supermana_progress_steps,
        safe_opponent_mana_progress: safe_opponent_mana_progress_steps.is_some()
            || tactical_spirit.same_turn_opponent_mana_score,
        safe_opponent_mana_progress_steps,
        spirit_assisted_score: tactical_spirit.same_turn_score,
        spirit_assisted_score_value: tactical_spirit.same_turn_score_value,
        spirit_assisted_denial: tactical_spirit.same_turn_opponent_mana_score,
        spirit_assisted_denial_value: tactical_spirit.same_turn_opponent_mana_score_value,
        same_turn_score_window_value,
    }
}

fn exact_multi_pressure_from_steps(steps: &[i32]) -> i32 {
    let mut pressure = 0;
    if let Some(step) = steps.get(1) {
        pressure += 70 / (*step).max(1);
    }
    if let Some(step) = steps.get(2) {
        pressure += 40 / (*step).max(1);
    }
    pressure
}

fn exact_multi_pressure_from_scores(scores: &[i32]) -> i32 {
    let second = scores.get(1).copied().unwrap_or(0);
    let third = scores.get(2).copied().unwrap_or(0);
    second * 70 + third * 35
}

#[derive(Debug, Clone, Copy)]
struct ExactStateResult {
    steps: i32,
}

fn exact_shortest_payload_state<F>(
    board: &Board,
    start: Location,
    mon_kind: MonKind,
    color: Color,
    start_payload: ExactActorPayload,
    allow_pick_bomb: bool,
    max_steps: Option<i32>,
    mut goal: F,
) -> Option<ExactStateResult>
where
    F: FnMut(Location, ExactActorPayload) -> bool,
{
    let mut queue = VecDeque::with_capacity(EXACT_BFS_CAPACITY);
    let mut seen = ExactPayloadSeen::new();
    queue.push_back((start, start_payload, 0));
    seen.insert(start, start_payload);

    while let Some((location, payload, steps)) = queue.pop_front() {
        if goal(location, payload) {
            return Some(ExactStateResult { steps });
        }
        if max_steps.map_or(false, |limit| steps >= limit) {
            continue;
        }
        for &next in location.nearby_locations_ref() {
            if let Some(next_payload) =
                actor_payload_after_move(board, mon_kind, color, payload, next, allow_pick_bomb)
            {
                if seen.insert(next, next_payload) {
                    queue.push_back((next, next_payload, steps + 1));
                }
            }
        }
    }

    None
}

fn actor_payload_after_move(
    board: &Board,
    mon_kind: MonKind,
    color: Color,
    payload: ExactActorPayload,
    destination: Location,
    allow_pick_bomb: bool,
) -> Option<ExactActorPayload> {
    let item = board.items[destination.index()];
    match payload {
        ExactActorPayload::None => match item {
            Some(Item::Mon { .. })
            | Some(Item::MonWithMana { .. })
            | Some(Item::MonWithConsumable { .. }) => None,
            Some(Item::Mana { mana }) => {
                if mon_kind == MonKind::Drainer {
                    Some(ExactActorPayload::Mana(mana))
                } else {
                    None
                }
            }
            Some(Item::Consumable {
                consumable: Consumable::BombOrPotion,
            }) => {
                if allow_pick_bomb {
                    Some(ExactActorPayload::Bomb)
                } else {
                    Some(ExactActorPayload::None)
                }
            }
            Some(Item::Consumable { .. }) => None,
            None => {
                let square = Config::square_at(destination);
                if square_allows_empty_mon(square, mon_kind, color) {
                    Some(ExactActorPayload::None)
                } else {
                    None
                }
            }
        },
        ExactActorPayload::Mana(_) => match item {
            Some(Item::Mon { .. })
            | Some(Item::MonWithMana { .. })
            | Some(Item::MonWithConsumable { .. }) => None,
            Some(Item::Mana { mana }) => Some(ExactActorPayload::Mana(mana)),
            Some(Item::Consumable {
                consumable: Consumable::BombOrPotion,
            }) => Some(payload),
            Some(Item::Consumable { .. }) => None,
            None => {
                let square = Config::square_at(destination);
                if square_allows_mana_carrier(square, payload.mana().unwrap()) {
                    Some(payload)
                } else {
                    None
                }
            }
        },
        ExactActorPayload::Bomb => match item {
            Some(Item::Mon { .. })
            | Some(Item::Mana { .. })
            | Some(Item::MonWithMana { .. })
            | Some(Item::MonWithConsumable { .. }) => None,
            Some(Item::Consumable {
                consumable: Consumable::BombOrPotion,
            }) => Some(ExactActorPayload::Bomb),
            Some(Item::Consumable { .. }) => None,
            None => {
                let square = Config::square_at(destination);
                if matches!(
                    square,
                    Square::Regular
                        | Square::ConsumableBase
                        | Square::ManaBase { .. }
                        | Square::ManaPool { .. }
                ) {
                    Some(ExactActorPayload::Bomb)
                } else {
                    None
                }
            }
        },
    }
}

impl ExactActorPayload {
    fn mana(self) -> Option<Mana> {
        match self {
            ExactActorPayload::Mana(mana) => Some(mana),
            ExactActorPayload::None | ExactActorPayload::Bomb => None,
        }
    }
}

fn square_allows_empty_mon(square: Square, mon_kind: MonKind, color: Color) -> bool {
    match square {
        Square::Regular
        | Square::ConsumableBase
        | Square::ManaBase { .. }
        | Square::ManaPool { .. } => true,
        Square::SupermanaBase => mon_kind == MonKind::Drainer,
        Square::MonBase {
            kind: base_kind,
            color: base_color,
        } => base_kind == mon_kind && base_color == color,
    }
}

fn square_allows_mana_carrier(square: Square, mana: Mana) -> bool {
    match square {
        Square::Regular
        | Square::ConsumableBase
        | Square::ManaBase { .. }
        | Square::ManaPool { .. } => true,
        Square::SupermanaBase => mana == Mana::Supermana,
        Square::MonBase { .. } => false,
    }
}

#[inline]
fn exact_carrier_steps_to_any_pool_with_hash(
    board: &Board,
    start: Location,
    mana: Mana,
    board_hash: u64,
) -> Option<i32> {
    let key = ExactCarrierStepsQueryKey {
        board_hash,
        start,
        mana,
    };
    if let Some(cached) =
        EXACT_CARRIER_STEPS_CACHE.with(|cache| cache.borrow().entries.get(&key).copied())
    {
        return cached;
    }

    let result = exact_shortest_payload_state(
        board,
        start,
        MonKind::Drainer,
        Color::White,
        ExactActorPayload::Mana(mana),
        false,
        None,
        |location, payload| {
            matches!(payload, ExactActorPayload::Mana(_))
                && matches!(board.square(location), Square::ManaPool { .. })
        },
    )
    .map(|result| result.steps);

    EXACT_CARRIER_STEPS_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if cache.entries.len() >= EXACT_CARRIER_STEPS_CACHE_MAX_ENTRIES
            && !cache.entries.contains_key(&key)
        {
            cache.entries.clear();
        }
        cache.entries.insert(key, result);
    });
    result
}

fn exact_carrier_steps_to_any_pool(board: &Board, start: Location, mana: Mana) -> Option<i32> {
    exact_carrier_steps_to_any_pool_with_hash(board, start, mana, exact_board_hash(board))
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug, Clone, Copy, Default)]
struct ExactDrainerPickupWindow {
    any: Option<ExactDrainerPickupPath>,
    opponent: Option<ExactDrainerPickupPath>,
}

#[cfg(any(target_arch = "wasm32", test))]
#[inline]
fn exact_pickup_path_beats(
    candidate: ExactDrainerPickupPath,
    current: Option<ExactDrainerPickupPath>,
) -> bool {
    match current {
        None => true,
        Some(current) => {
            let candidate_metric = candidate.path_steps * 3 - candidate.mana_value;
            let current_metric = current.path_steps * 3 - current.mana_value;
            candidate_metric < current_metric
                || (candidate_metric == current_metric && candidate.mana_value > current.mana_value)
        }
    }
}

#[cfg(any(target_arch = "wasm32", test))]
fn exact_drainer_pickup_window_uncached(
    board: &Board,
    color: Color,
    start: Location,
    max_steps: Option<i32>,
    opponent_mana: Mana,
) -> ExactDrainerPickupWindow {
    update_exact_query_diagnostics(|diagnostics| diagnostics.pickup_path_calls += 1);

    let mut queue = VecDeque::with_capacity(EXACT_BFS_CAPACITY);
    let mut seen = ExactPayloadSeen::new();
    queue.push_back((start, ExactActorPayload::None, 0));
    seen.insert(start, ExactActorPayload::None);
    let mut best = ExactDrainerPickupWindow::default();

    while let Some((location, payload, steps)) = queue.pop_front() {
        if max_steps.map_or(false, |limit| steps > limit) {
            continue;
        }
        if let ExactActorPayload::Mana(mana) = payload {
            if matches!(board.square(location), Square::ManaPool { .. }) {
                let candidate = ExactDrainerPickupPath {
                    path_steps: steps.saturating_sub(1),
                    total_moves: steps,
                    mana_value: mana.score(color),
                    mana,
                };
                if exact_pickup_path_beats(candidate, best.any) {
                    best.any = Some(candidate);
                }
                if mana == opponent_mana && exact_pickup_path_beats(candidate, best.opponent) {
                    best.opponent = Some(candidate);
                }
            }
        }

        for &next in location.nearby_locations_ref() {
            if let Some(next_payload) =
                actor_payload_after_move(board, MonKind::Drainer, color, payload, next, false)
            {
                if seen.insert(next, next_payload) {
                    queue.push_back((next, next_payload, steps + 1));
                }
            }
        }
    }

    best
}

#[inline]
fn exact_best_drainer_pickup_path_filtered_with_hash(
    board: &Board,
    color: Color,
    start: Location,
    max_steps: Option<i32>,
    mana_filter: ExactPickupFilter,
    board_hash: u64,
) -> Option<ExactDrainerPickupPath> {
    update_exact_query_diagnostics(|diagnostics| diagnostics.pickup_path_calls += 1);
    let key = ExactPickupPathQueryKey {
        board_hash,
        color,
        start,
        max_steps,
        filter: mana_filter,
    };
    if let Some(cached) =
        EXACT_PICKUP_PATH_CACHE.with(|cache| cache.borrow().entries.get(&key).copied())
    {
        update_exact_query_diagnostics(|diagnostics| diagnostics.pickup_path_cache_hits += 1);
        return cached;
    }
    update_exact_query_diagnostics(|diagnostics| diagnostics.pickup_path_cache_misses += 1);

    let mut queue = VecDeque::with_capacity(EXACT_BFS_CAPACITY);
    let mut seen = ExactPayloadSeen::new();
    let start_state = (start, ExactActorPayload::None, 0);
    queue.push_back(start_state);
    seen.insert(start, ExactActorPayload::None);
    let mut best: Option<ExactDrainerPickupPath> = None;

    while let Some((location, payload, steps)) = queue.pop_front() {
        if max_steps.map_or(false, |limit| steps > limit) {
            continue;
        }
        if let ExactActorPayload::Mana(mana) = payload {
            if mana_filter.matches(mana)
                && matches!(board.square(location), Square::ManaPool { .. })
            {
                let candidate = ExactDrainerPickupPath {
                    path_steps: steps.saturating_sub(1),
                    total_moves: steps,
                    mana_value: mana.score(color),
                    #[cfg(any(target_arch = "wasm32", test))]
                    mana,
                };
                if exact_pickup_path_beats(candidate, best) {
                    best = Some(candidate);
                }
            }
        }

        for &next in location.nearby_locations_ref() {
            if let Some(next_payload) =
                actor_payload_after_move(board, MonKind::Drainer, color, payload, next, false)
            {
                if seen.insert(next, next_payload) {
                    queue.push_back((next, next_payload, steps + 1));
                }
            }
        }
    }

    EXACT_PICKUP_PATH_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if cache.entries.len() >= EXACT_PICKUP_PATH_CACHE_MAX_ENTRIES
            && !cache.entries.contains_key(&key)
        {
            cache.entries.clear();
        }
        cache.entries.insert(key, best);
    });
    best
}

fn exact_best_drainer_pickup_path_filtered(
    board: &Board,
    color: Color,
    start: Location,
    max_steps: Option<i32>,
    mana_filter: ExactPickupFilter,
) -> Option<ExactDrainerPickupPath> {
    exact_best_drainer_pickup_path_filtered_with_hash(
        board,
        color,
        start,
        max_steps,
        mana_filter,
        exact_board_hash(board),
    )
}

fn find_awake_drainer(board: &Board, color: Color) -> Option<Location> {
    board.occupied().find_map(|(location, item)| {
        let mon = item.mon()?;
        (mon.color == color && mon.kind == MonKind::Drainer && !mon.is_fainted())
            .then_some(location)
    })
}

#[cfg(any(target_arch = "wasm32", test))]
fn exact_drainer_to_any_mana_steps(board: &Board, color: Color, start: Location) -> Option<i32> {
    let key = ExactDrainerToManaQueryKey {
        board_hash: exact_board_hash(board),
        color,
        start,
    };
    if let Some(cached) =
        EXACT_DRAINER_TO_MANA_CACHE.with(|cache| cache.borrow().entries.get(&key).copied())
    {
        return cached;
    }

    let result = exact_shortest_payload_state(
        board,
        start,
        MonKind::Drainer,
        color,
        ExactActorPayload::None,
        false,
        None,
        |_, payload| matches!(payload, ExactActorPayload::Mana(_)),
    )
    .map(|result| result.steps);

    EXACT_DRAINER_TO_MANA_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if cache.entries.len() >= EXACT_DRAINER_TO_MANA_CACHE_MAX_ENTRIES
            && !cache.entries.contains_key(&key)
        {
            cache.entries.clear();
        }
        cache.entries.insert(key, result);
    });
    result
}

#[cfg(any(target_arch = "wasm32", test))]
fn exact_secure_specific_mana_steps_this_turn(
    game: &MonsGame,
    color: Color,
    wanted: Mana,
) -> Option<i32> {
    let remaining_moves = if game.active_color == color {
        (Config::MONS_MOVES_PER_TURN - game.mons_moves_count).max(0)
    } else {
        Config::MONS_MOVES_PER_TURN
    };
    exact_secure_specific_mana_steps_on_board(&game.board, color, wanted, remaining_moves)
}

#[cfg(any(target_arch = "wasm32", test))]
fn can_secure_specific_mana_on_board(
    board: &Board,
    color: Color,
    wanted: Mana,
    remaining_moves: i32,
) -> bool {
    exact_secure_specific_mana_steps_on_board(board, color, wanted, remaining_moves).is_some()
}

#[cfg(any(target_arch = "wasm32", test))]
pub(crate) fn exact_secure_specific_mana_steps_on_board(
    board: &Board,
    color: Color,
    wanted: Mana,
    remaining_moves: i32,
) -> Option<i32> {
    if remaining_moves < 0 {
        return None;
    }

    let mons_moves_count =
        (Config::MONS_MOVES_PER_TURN - remaining_moves).clamp(0, Config::MONS_MOVES_PER_TURN);
    let game = MonsGame::new_simulation_state(
        board.clone(),
        0,
        0,
        color,
        Config::ACTIONS_PER_TURN,
        0,
        mons_moves_count,
        0,
        0,
        2,
    );
    // Non-terminal same-turn states still have the mana move available; exhausting it here
    // would make the synthetic game auto-end after one mon move and miss multi-step drainer paths.
    let state = exact_secure_mana_state_key_from_board(board, color, mons_moves_count);
    exact_secure_specific_mana_steps_in_game_with_key(&game, color, wanted, state)
}

#[cfg(any(target_arch = "wasm32", test))]
fn exact_secure_specific_mana_steps_in_game_with_key(
    game: &MonsGame,
    color: Color,
    wanted: Mana,
    state: ExactSecureManaStateKey,
) -> Option<i32> {
    update_exact_query_diagnostics(|diagnostics| diagnostics.exact_secure_mana_calls += 1);
    let mut game = game.clone_for_simulation();
    EXACT_SECURE_MANA_CACHE.with(|cache| {
        exact_secure_specific_mana_steps_in_game_with_key_mut(
            &mut game,
            color,
            wanted,
            state,
            &mut cache.borrow_mut(),
        )
    })
}

#[cfg(any(target_arch = "wasm32", test))]
fn exact_secure_specific_mana_steps_in_game_with_key_mut(
    game: &mut MonsGame,
    color: Color,
    wanted: Mana,
    state: ExactSecureManaStateKey,
    cache: &mut ExactSecureManaCache,
) -> Option<i32> {
    let drainer_location = find_awake_drainer(&game.board, color)?;
    exact_secure_specific_mana_steps_in_game_with_key_at_mut(
        game,
        color,
        drainer_location,
        wanted,
        state,
        cache,
    )
}

#[cfg(any(target_arch = "wasm32", test))]
fn exact_secure_specific_mana_steps_in_game_with_key_at_mut(
    game: &mut MonsGame,
    color: Color,
    drainer_location: Location,
    wanted: Mana,
    state: ExactSecureManaStateKey,
    cache: &mut ExactSecureManaCache,
) -> Option<i32> {
    let key = ExactSecureManaQueryKey {
        state,
        color,
        wanted,
    };
    if let Some(cached) = cache.entries.get(&key).copied() {
        update_exact_query_diagnostics(|diagnostics| diagnostics.exact_secure_mana_cache_hits += 1);
        return cached;
    }

    if !cache.visiting.insert(key) {
        return None;
    }

    let result = exact_secure_specific_mana_steps_in_game_uncached_at_mut(
        game,
        color,
        drainer_location,
        wanted,
        state,
        cache,
    );
    cache.visiting.remove(&key);
    if cache.entries.len() >= EXACT_SECURE_MANA_CACHE_MAX_ENTRIES
        && !cache.entries.contains_key(&key)
    {
        cache.entries.clear();
        cache.visiting.clear();
    }
    cache.entries.insert(key, result);
    result
}

#[cfg(any(target_arch = "wasm32", test))]
fn exact_secure_specific_mana_steps_in_game_uncached_at_mut(
    game: &mut MonsGame,
    color: Color,
    drainer_location: Location,
    wanted: Mana,
    state_key: ExactSecureManaStateKey,
    cache: &mut ExactSecureManaCache,
) -> Option<i32> {
    if matches!(
        game.board.item(drainer_location),
        Some(Item::MonWithMana { mana, .. }) if *mana == wanted
    ) {
        if is_drainer_exactly_safe_next_turn_on_board(&game.board, color, drainer_location) {
            return Some(0);
        }
    }

    if game.active_color != color || !game.player_can_move_mon() {
        return None;
    }

    let mut best = None;
    for &next in drainer_location.nearby_locations_ref() {
        let Some(transition) =
            exact_apply_secure_drainer_walk_in_place(game, state_key, drainer_location, next)
        else {
            continue;
        };
        let candidate = if transition.scored_mana == Some(wanted) {
            Some(1)
        } else {
            exact_secure_specific_mana_steps_in_game_with_key_at_mut(
                game,
                color,
                next,
                wanted,
                transition.after_key,
                cache,
            )
            .map(|next_steps| next_steps.saturating_add(1))
        };
        exact_undo_secure_drainer_walk(game, transition.undo);
        if let Some(candidate) = candidate {
            best = Some(best.map_or(candidate, |current: i32| current.min(candidate)));
            if candidate == 1 {
                break;
            }
        }
    }

    best
}

#[cfg(any(target_arch = "wasm32", test))]
pub(crate) fn exact_secure_specific_mana_path_from(
    game: &MonsGame,
    color: Color,
    start: Location,
    wanted: Mana,
) -> Option<Vec<Location>> {
    let mut visiting =
        ExactHashSet::with_capacity_and_hasher(EXACT_BFS_CAPACITY, ExactBuildHasher::default());
    exact_secure_specific_mana_path_from_uncached(
        game,
        color,
        start,
        wanted,
        exact_secure_mana_state_key(game),
        &mut visiting,
    )
}

#[cfg(any(target_arch = "wasm32", test))]
fn exact_secure_specific_mana_path_from_uncached(
    game: &MonsGame,
    color: Color,
    start: Location,
    wanted: Mana,
    state_key: ExactSecureManaStateKey,
    visiting: &mut ExactHashSet<ExactSecureManaStateKey>,
) -> Option<Vec<Location>> {
    if !visiting.insert(state_key) {
        return None;
    }

    let result = if matches!(
        game.board.item(start),
        Some(Item::MonWithMana { mana, .. }) if *mana == wanted
    ) && is_drainer_exactly_safe_next_turn_on_board(&game.board, color, start)
    {
        Some(Vec::new())
    } else if game.active_color != color || !game.player_can_move_mon() {
        None
    } else {
        let mut best_path: Option<Vec<Location>> = None;

        for &next in start.nearby_locations_ref() {
            let Some(transition) = exact_apply_secure_drainer_walk(game, state_key, start, next)
            else {
                continue;
            };

            let candidate_path = if transition.scored_mana == Some(wanted) {
                Some(vec![next])
            } else if exact_secure_specific_mana_steps_in_game_with_key(
                &transition.after,
                color,
                wanted,
                transition.after_key,
            )
            .is_some()
            {
                let Some(next_start) = find_awake_drainer(&transition.after.board, color) else {
                    continue;
                };
                let Some(mut suffix) = exact_secure_specific_mana_path_from_uncached(
                    &transition.after,
                    color,
                    next_start,
                    wanted,
                    transition.after_key,
                    visiting,
                ) else {
                    continue;
                };
                let mut path = Vec::with_capacity(suffix.len() + 1);
                path.push(next);
                path.append(&mut suffix);
                Some(path)
            } else {
                None
            };

            let Some(candidate_path) = candidate_path else {
                continue;
            };
            let replace = match &best_path {
                None => true,
                Some(current) => candidate_path.len() < current.len(),
            };
            if replace {
                best_path = Some(candidate_path);
            }
        }

        best_path
    };

    visiting.remove(&state_key);
    result
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug, Clone)]
struct ExactSecureDrainerWalkTransition {
    after: MonsGame,
    after_key: ExactSecureManaStateKey,
    scored_mana: Option<Mana>,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug, Clone, Copy)]
struct ExactSecureGameSnapshot {
    white_score: i32,
    black_score: i32,
    active_color: Color,
    actions_used_count: i32,
    mana_moves_count: i32,
    mons_moves_count: i32,
    white_potions_count: i32,
    black_potions_count: i32,
    turn_number: i32,
}

#[cfg(any(target_arch = "wasm32", test))]
impl ExactSecureGameSnapshot {
    #[inline]
    fn capture(game: &MonsGame) -> Self {
        Self {
            white_score: game.white_score,
            black_score: game.black_score,
            active_color: game.active_color,
            actions_used_count: game.actions_used_count,
            mana_moves_count: game.mana_moves_count,
            mons_moves_count: game.mons_moves_count,
            white_potions_count: game.white_potions_count,
            black_potions_count: game.black_potions_count,
            turn_number: game.turn_number,
        }
    }
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug, Clone, Copy)]
struct ExactSecureTouchedItem {
    location: Location,
    before: Option<Item>,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug, Clone, Copy)]
struct ExactSecureTouchedItems {
    items: [Option<ExactSecureTouchedItem>; EXACT_SECURE_TOUCHED_ITEMS_CAPACITY],
    len: usize,
    seen_mask: u128,
}

#[cfg(any(target_arch = "wasm32", test))]
impl ExactSecureTouchedItems {
    #[inline]
    fn new() -> Self {
        Self {
            items: [None; EXACT_SECURE_TOUCHED_ITEMS_CAPACITY],
            len: 0,
            seen_mask: 0,
        }
    }

    #[inline]
    fn push_once(&mut self, board: &Board, location: Location) {
        let seen_bit = 1u128 << location.index();
        if self.seen_mask & seen_bit != 0 {
            return;
        }

        assert!(self.len < EXACT_SECURE_TOUCHED_ITEMS_CAPACITY);
        self.items[self.len] = Some(ExactSecureTouchedItem {
            location,
            before: board.item(location).copied(),
        });
        self.len += 1;
        self.seen_mask |= seen_bit;
    }
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug, Clone, Copy)]
struct ExactSecureDrainerWalkUndo {
    snapshot: ExactSecureGameSnapshot,
    touched_items: ExactSecureTouchedItems,
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug, Clone, Copy)]
struct ExactSecureDrainerWalkMutation {
    after_key: ExactSecureManaStateKey,
    scored_mana: Option<Mana>,
    undo: ExactSecureDrainerWalkUndo,
}

#[cfg(any(target_arch = "wasm32", test))]
#[inline]
fn exact_secure_board_hash_after_touched_items(
    before_hash: u64,
    board: &Board,
    touched_items: ExactSecureTouchedItems,
) -> u64 {
    let mut after_hash = before_hash;
    for idx in 0..touched_items.len {
        let entry = touched_items.items[idx].unwrap();
        let index = entry.location.index();
        if let Some(item) = entry.before {
            after_hash ^= exact_secure_board_entry_hash(index, item);
        }
        if let Some(item) = board.items[index] {
            after_hash ^= exact_secure_board_entry_hash(index, item);
        }
    }
    after_hash
}

#[cfg(any(target_arch = "wasm32", test))]
#[inline]
fn exact_undo_secure_drainer_walk(game: &mut MonsGame, undo: ExactSecureDrainerWalkUndo) {
    game.white_score = undo.snapshot.white_score;
    game.black_score = undo.snapshot.black_score;
    game.active_color = undo.snapshot.active_color;
    game.actions_used_count = undo.snapshot.actions_used_count;
    game.mana_moves_count = undo.snapshot.mana_moves_count;
    game.mons_moves_count = undo.snapshot.mons_moves_count;
    game.white_potions_count = undo.snapshot.white_potions_count;
    game.black_potions_count = undo.snapshot.black_potions_count;
    game.turn_number = undo.snapshot.turn_number;

    for idx in 0..undo.touched_items.len {
        let entry = undo.touched_items.items[idx].unwrap();
        match entry.before {
            Some(item) => game.board.put(item, entry.location),
            None => game.board.remove_item(entry.location),
        }
    }
}

#[cfg(any(target_arch = "wasm32", test))]
fn exact_apply_secure_drainer_walk_in_place(
    game: &mut MonsGame,
    state_key: ExactSecureManaStateKey,
    from: Location,
    to: Location,
) -> Option<ExactSecureDrainerWalkMutation> {
    if !exact_walk_destination_plausible(&game.board, from, to) {
        return None;
    }

    let start_item = game.board.item(from).copied()?;
    let start_mon = start_item.mon().copied()?;
    if start_mon.kind != MonKind::Drainer || start_mon.is_fainted() {
        return None;
    }

    let snapshot = ExactSecureGameSnapshot::capture(game);
    let target_item = game.board.item(to).copied();
    let mut touched_items = ExactSecureTouchedItems::new();
    let mut white_regular_mana_count = state_key.white_regular_mana_count;
    let mut black_regular_mana_count = state_key.black_regular_mana_count;
    touched_items.push_once(&game.board, from);
    touched_items.push_once(&game.board, to);

    game.mons_moves_count += 1;
    game.board.remove_item(from);
    game.board.put(start_item, to);

    match target_item {
        Some(Item::Mon { .. })
        | Some(Item::MonWithMana { .. })
        | Some(Item::MonWithConsumable { .. }) => return None,
        Some(Item::Mana { mana }) => {
            exact_adjust_regular_mana_counts(
                &mut white_regular_mana_count,
                &mut black_regular_mana_count,
                mana,
                -1,
            );
            if let Some(start_mana) = start_item.mana() {
                exact_adjust_regular_mana_counts(
                    &mut white_regular_mana_count,
                    &mut black_regular_mana_count,
                    *start_mana,
                    1,
                );
                game.board.put(Item::Mana { mana: *start_mana }, from);
            }
            game.board.put(
                Item::MonWithMana {
                    mon: start_mon,
                    mana,
                },
                to,
            );
        }
        Some(Item::Consumable { consumable }) => match consumable {
            Consumable::Bomb | Consumable::Potion => return None,
            Consumable::BombOrPotion => {
                if start_item.consumable().is_some() || start_item.mana().is_some() {
                    if start_mon.color == Color::White {
                        game.white_potions_count += 1;
                    } else {
                        game.black_potions_count += 1;
                    }
                    game.board.put(start_item, to);
                } else {
                    return None;
                }
            }
        },
        None => {}
    }

    let scored_mana = match game.board.square(to) {
        Square::ManaPool { .. } => start_item.mana().copied(),
        Square::Regular
        | Square::ConsumableBase
        | Square::ManaBase { .. }
        | Square::SupermanaBase
        | Square::MonBase { .. } => None,
    };
    if let Some(mana) = scored_mana {
        let score = mana.score(game.active_color);
        if game.active_color == Color::White {
            game.white_score += score;
        } else {
            game.black_score += score;
        }
        match game.board.item(to).copied() {
            Some(Item::Mon { mon })
            | Some(Item::MonWithMana { mon, .. })
            | Some(Item::MonWithConsumable { mon, .. }) => {
                game.board.put(Item::Mon { mon }, to);
            }
            Some(Item::Mana { .. }) | Some(Item::Consumable { .. }) | None => {
                game.board.remove_item(to);
            }
        }
    }

    let first_turn = game.turn_number == 1;
    let player_can_move_mon = game.mons_moves_count < Config::MONS_MOVES_PER_TURN;
    let player_can_move_mana = !first_turn && game.mana_moves_count < Config::MANA_MOVES_PER_TURN;
    let active_regular_mana_count = if game.active_color == Color::White {
        white_regular_mana_count
    } else {
        black_regular_mana_count
    };
    let should_end_turn = game.white_score < Config::TARGET_SCORE
        && game.black_score < Config::TARGET_SCORE
        && (first_turn && !player_can_move_mon
            || !first_turn && !player_can_move_mana
            || !first_turn && !player_can_move_mon && active_regular_mana_count == 0);
    if should_end_turn {
        let next_active_color = game.active_color.other();
        game.active_color = next_active_color;
        game.turn_number += 1;
        game.actions_used_count = 0;
        game.mana_moves_count = 0;
        game.mons_moves_count = 0;

        for index in 0..game.board.items.len() {
            let Some(Item::Mon { mon }) = game.board.items[index] else {
                continue;
            };
            if mon.color != next_active_color || !mon.is_fainted() {
                continue;
            }
            let mon_location = Location::from_index(index);
            touched_items.push_once(&game.board, mon_location);
            let mut mon = mon;
            mon.decrease_cooldown();
            game.board.items[index] = Some(Item::Mon { mon });
        }
    }

    let after_key = ExactSecureManaStateKey {
        board_hash: exact_secure_board_hash_after_touched_items(
            state_key.board_hash,
            &game.board,
            touched_items,
        ),
        active_color: game.active_color,
        mons_moves_count: game.mons_moves_count,
        white_regular_mana_count,
        black_regular_mana_count,
    };
    Some(ExactSecureDrainerWalkMutation {
        after_key,
        scored_mana,
        undo: ExactSecureDrainerWalkUndo {
            snapshot,
            touched_items,
        },
    })
}

#[cfg(any(target_arch = "wasm32", test))]
fn exact_apply_secure_drainer_walk(
    game: &MonsGame,
    state_key: ExactSecureManaStateKey,
    from: Location,
    to: Location,
) -> Option<ExactSecureDrainerWalkTransition> {
    let mut after = game.clone_for_simulation();
    let mutation = exact_apply_secure_drainer_walk_in_place(&mut after, state_key, from, to)?;
    Some(ExactSecureDrainerWalkTransition {
        after,
        after_key: mutation.after_key,
        scored_mana: mutation.scored_mana,
    })
}

#[cfg(any(target_arch = "wasm32", test))]
fn can_attack_opponent_drainer_exact(game: &MonsGame, color: Color) -> bool {
    let Some(target) = find_awake_drainer(&game.board, color.other()) else {
        return false;
    };
    can_attack_target_on_board(
        &game.board,
        color,
        color.other(),
        target,
        if game.active_color == color {
            (Config::MONS_MOVES_PER_TURN - game.mons_moves_count).max(0)
        } else {
            Config::MONS_MOVES_PER_TURN
        },
        if game.active_color == color {
            game.player_can_use_action()
        } else {
            true
        },
    )
}

fn demon_has_line_attack(board: &Board, from: Location, target: Location) -> bool {
    let di = (from.i - target.i).abs();
    let dj = (from.j - target.j).abs();
    if !((di == 2 && dj == 0) || (di == 0 && dj == 2)) {
        return false;
    }
    let middle = from.location_between(&target);
    board.item(middle).is_none()
        && !matches!(
            board.square(middle),
            Square::SupermanaBase | Square::MonBase { .. }
        )
}

#[cfg(any(target_arch = "wasm32", test))]
fn exact_spirit_summary(
    board: &Board,
    color: Color,
    remaining_mon_moves: i32,
    can_use_action: bool,
) -> ExactSpiritSummary {
    update_exact_query_diagnostics(|diagnostics| diagnostics.exact_spirit_summary_calls += 1);
    if remaining_mon_moves < 0 {
        return ExactSpiritSummary::default();
    }
    let key = ExactSpiritSummaryKey {
        board_hash: exact_board_hash(board),
        color,
        remaining_mon_moves,
        can_use_action,
    };
    if let Some(cached) =
        EXACT_SPIRIT_SUMMARY_CACHE.with(|cache| cache.borrow().entries.get(&key).copied())
    {
        update_exact_query_diagnostics(|diagnostics| {
            diagnostics.exact_spirit_summary_cache_hits += 1
        });
        return cached;
    }

    let summary = exact_spirit_summary_uncached(board, color, remaining_mon_moves, can_use_action);
    EXACT_SPIRIT_SUMMARY_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if cache.entries.len() >= EXACT_SPIRIT_SUMMARY_CACHE_MAX_ENTRIES
            && !cache.entries.contains_key(&key)
        {
            cache.entries.clear();
        }
        cache.entries.insert(key, summary);
    });
    summary
}

#[cfg(any(target_arch = "wasm32", test))]
fn exact_tactical_spirit_summary(
    board: &Board,
    color: Color,
    remaining_mon_moves: i32,
    can_use_action: bool,
    fields: u8,
) -> ExactSpiritSummary {
    update_exact_query_diagnostics(|diagnostics| diagnostics.tactical_spirit_summary_calls += 1);
    if remaining_mon_moves < 0 || fields == 0 {
        return ExactSpiritSummary::default();
    }
    let key = ExactTacticalSpiritSummaryKey {
        board_hash: exact_board_hash(board),
        color,
        remaining_mon_moves,
        can_use_action,
        fields,
    };
    if let Some(cached) =
        EXACT_SPIRIT_TACTICAL_SUMMARY_CACHE.with(|cache| cache.borrow().entries.get(&key).copied())
    {
        update_exact_query_diagnostics(|diagnostics| {
            diagnostics.tactical_spirit_summary_cache_hits += 1;
        });
        return cached;
    }

    let summary = exact_tactical_spirit_summary_uncached(
        board,
        color,
        remaining_mon_moves,
        can_use_action,
        fields,
        key.board_hash,
    );
    EXACT_SPIRIT_TACTICAL_SUMMARY_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if cache.entries.len() >= EXACT_SPIRIT_SUMMARY_CACHE_MAX_ENTRIES
            && !cache.entries.contains_key(&key)
        {
            cache.entries.clear();
        }
        cache.entries.insert(key, summary);
    });
    summary
}

fn exact_passive_spirit_summary(
    board: &Board,
    color: Color,
    remaining_mon_moves: i32,
    can_use_action: bool,
) -> ExactSpiritSummary {
    update_exact_query_diagnostics(|diagnostics| diagnostics.passive_spirit_summary_calls += 1);
    if remaining_mon_moves < 0 || !can_use_action {
        return ExactSpiritSummary::default();
    }

    let mut best = ExactSpiritSummary::default();

    for (location, item) in board.occupied() {
        let Some(mon) = item.mon() else {
            continue;
        };
        if mon.color != color || mon.kind != MonKind::Spirit || mon.is_fainted() {
            continue;
        }

        for (spirit_pos, _) in
            reachable_spirit_positions(board, location, color, remaining_mon_moves)
        {
            if matches!(board.square(spirit_pos), Square::MonBase { .. }) {
                continue;
            }

            let mut reachable_targets = 0;
            let mut setup_gain = 0;
            let mut supermana_progress = false;
            let mut opponent_mana_progress = false;

            for &target in spirit_pos.reachable_by_spirit_action_ref() {
                let Some(target_item) = board.item(target).copied() else {
                    continue;
                };
                if !spirit_target_allowed(target_item) {
                    continue;
                }
                if !target
                    .nearby_locations_ref()
                    .iter()
                    .copied()
                    .any(|destination| {
                        spirit_destination_allowed(board, target, target_item, destination)
                    })
                {
                    continue;
                }

                reachable_targets += 1;
                match target_item {
                    Item::Mana {
                        mana: Mana::Supermana,
                    } => {
                        supermana_progress = true;
                        setup_gain = setup_gain.max(2);
                    }
                    Item::Mana {
                        mana: Mana::Regular(mana_color),
                    } if mana_color == color.other() => {
                        opponent_mana_progress = true;
                        setup_gain = setup_gain.max(2);
                    }
                    Item::Mon { mon }
                    | Item::MonWithMana { mon, .. }
                    | Item::MonWithConsumable { mon, .. } => {
                        if mon.color == color && mon.kind == MonKind::Drainer && !mon.is_fainted() {
                            setup_gain = setup_gain.max(2);
                        } else if mon.color != color && !mon.is_fainted() {
                            setup_gain = setup_gain.max(1);
                        }
                    }
                    Item::Mana { .. } | Item::Consumable { .. } => {}
                }
            }

            if supermana_progress {
                best.supermana_progress = true;
            }
            if opponent_mana_progress {
                best.opponent_mana_progress = true;
            }

            let utility = reachable_targets
                .min(EXACT_SPIRIT_UTILITY_CAP)
                .max((1 + setup_gain).min(EXACT_SPIRIT_UTILITY_CAP));
            if utility > best.utility {
                best.utility = utility;
                best.next_turn_setup_gain = setup_gain;
            } else if utility == best.utility {
                best.next_turn_setup_gain = best.next_turn_setup_gain.max(setup_gain);
            }
        }
    }

    best
}

#[cfg(any(target_arch = "wasm32", test))]
fn exact_tactical_spirit_summary_uncached(
    board: &Board,
    color: Color,
    remaining_mon_moves: i32,
    can_use_action: bool,
    fields: u8,
    board_hash: u64,
) -> ExactSpiritSummary {
    if !can_use_action {
        return ExactSpiritSummary::default();
    }

    let need_score = fields & EXACT_TACTICAL_SPIRIT_NEED_SCORE != 0;
    let need_denial = fields & EXACT_TACTICAL_SPIRIT_NEED_DENIAL != 0;
    let need_progress = fields & EXACT_TACTICAL_SPIRIT_NEED_PROGRESS != 0;
    let before_window = exact_best_immediate_tactical_window_on_board_with_hash(
        board,
        color,
        remaining_mon_moves,
        need_score,
        need_denial,
        board_hash,
    );
    let before_same_turn_score = before_window.best_score;
    let before_same_turn_opponent_score = before_window.best_opponent_mana_score;
    let max_same_turn_score = if need_score {
        Mana::Supermana.score(color)
    } else {
        0
    };
    let max_same_turn_opponent_score = if need_denial {
        Mana::Regular(color.other()).score(color)
    } else {
        0
    };
    let mut best = ExactSpiritSummary::default();
    let mut after_window_cache: ExactHashMap<
        ExactTacticalSpiritAfterWindowKey,
        ExactImmediateTacticalWindow,
    > = ExactHashMap::default();

    for (location, item) in board.occupied() {
        let Some(mon) = item.mon() else {
            continue;
        };
        if mon.color != color || mon.kind != MonKind::Spirit || mon.is_fainted() {
            continue;
        }

        for (spirit_pos, spirit_steps) in
            reachable_spirit_positions(board, location, color, remaining_mon_moves)
        {
            if matches!(board.square(spirit_pos), Square::MonBase { .. }) {
                continue;
            }
            let action_board_storage = (spirit_pos != location).then(|| {
                let mut moved = board.clone();
                moved.remove_item(location);
                moved.put(*item, spirit_pos);
                moved
            });
            let action_board = action_board_storage.as_ref().unwrap_or(board);
            let remaining_after_action = remaining_mon_moves.saturating_sub(spirit_steps);

            for &target in spirit_pos.reachable_by_spirit_action_ref() {
                let Some(target_item) = action_board.item(target).copied() else {
                    continue;
                };
                if !spirit_target_allowed(target_item) {
                    continue;
                }
                for &dest in target.nearby_locations_ref() {
                    if !spirit_destination_allowed(action_board, target, target_item, dest) {
                        continue;
                    }
                    let (after_board, score_delta, opponent_mana_score_delta) =
                        apply_spirit_move_preview(action_board, target, target_item, dest, color);
                    let need_after_score =
                        need_score && best.same_turn_score_value < max_same_turn_score;
                    let need_after_denial = need_denial
                        && best.same_turn_opponent_mana_score_value < max_same_turn_opponent_score;
                    let after_window = if need_after_score || need_after_denial {
                        let after_board_hash = exact_board_hash(&after_board);
                        let key = ExactTacticalSpiritAfterWindowKey {
                            board_hash: after_board_hash,
                            remaining_mon_moves: remaining_after_action,
                            need_score: need_after_score,
                            need_denial: need_after_denial,
                        };
                        if let Some(cached) = after_window_cache.get(&key).copied() {
                            cached
                        } else {
                            let window = exact_best_immediate_tactical_window_on_board_with_hash(
                                &after_board,
                                color,
                                remaining_after_action,
                                need_after_score,
                                need_after_denial,
                                after_board_hash,
                            );
                            after_window_cache.insert(key, window);
                            window
                        }
                    } else {
                        ExactImmediateTacticalWindow::default()
                    };
                    let after_same_turn_score = if need_after_score {
                        score_delta.max(after_window.best_score)
                    } else {
                        best.same_turn_score_value
                    };
                    let after_same_turn_opponent_score = if need_after_denial {
                        opponent_mana_score_delta.max(after_window.best_opponent_mana_score)
                    } else {
                        best.same_turn_opponent_mana_score_value
                    };

                    if need_score
                        && best.same_turn_score_value < max_same_turn_score
                        && (score_delta > 0 || after_same_turn_score > before_same_turn_score)
                    {
                        best.same_turn_score = true;
                        best.same_turn_score_value =
                            best.same_turn_score_value.max(after_same_turn_score);
                    }
                    if need_denial
                        && best.same_turn_opponent_mana_score_value < max_same_turn_opponent_score
                        && (opponent_mana_score_delta > 0
                            || after_same_turn_opponent_score > before_same_turn_opponent_score)
                    {
                        best.same_turn_opponent_mana_score = true;
                        best.same_turn_opponent_mana_score_value = best
                            .same_turn_opponent_mana_score_value
                            .max(after_same_turn_opponent_score);
                    }
                    if need_progress
                        && !best.supermana_progress
                        && ((matches!(
                            target_item,
                            Item::Mana {
                                mana: Mana::Supermana,
                            }
                        ) && score_delta > 0)
                            || can_secure_specific_mana_on_board(
                                &after_board,
                                color,
                                Mana::Supermana,
                                remaining_after_action,
                            ))
                    {
                        best.supermana_progress = true;
                    }
                    if need_progress
                        && !best.opponent_mana_progress
                        && (opponent_mana_score_delta > 0
                            || can_secure_specific_mana_on_board(
                                &after_board,
                                color,
                                Mana::Regular(color.other()),
                                remaining_after_action,
                            ))
                    {
                        best.opponent_mana_progress = true;
                    }
                    if (!need_score || best.same_turn_score_value >= max_same_turn_score)
                        && (!need_denial
                            || best.same_turn_opponent_mana_score_value
                                >= max_same_turn_opponent_score)
                        && (!need_progress
                            || (best.supermana_progress && best.opponent_mana_progress))
                    {
                        return best;
                    }
                }
            }
        }
    }

    best
}

#[cfg(any(target_arch = "wasm32", test))]
fn exact_spirit_summary_uncached(
    board: &Board,
    color: Color,
    remaining_mon_moves: i32,
    can_use_action: bool,
) -> ExactSpiritSummary {
    if !can_use_action {
        return ExactSpiritSummary::default();
    }
    let before_summary = exact_followup_summary(board, color, remaining_mon_moves);
    let before_best_steps = before_summary.best_score_steps;
    let opponent_before = before_summary.opponent_best_score_steps;
    let before_same_turn_score = before_summary.immediate_score;
    let before_same_turn_opponent_score = before_summary.immediate_opponent_mana_score;
    let mut best = ExactSpiritSummary::default();

    for (location, item) in board.occupied() {
        let Some(mon) = item.mon() else {
            continue;
        };
        if mon.color != color || mon.kind != MonKind::Spirit || mon.is_fainted() {
            continue;
        }

        for (spirit_pos, spirit_steps) in
            reachable_spirit_positions(board, location, color, remaining_mon_moves)
        {
            if matches!(board.square(spirit_pos), Square::MonBase { .. }) {
                continue;
            }
            let action_board_storage = (spirit_pos != location).then(|| {
                let mut moved = board.clone();
                moved.remove_item(location);
                moved.put(*item, spirit_pos);
                moved
            });
            let action_board = action_board_storage.as_ref().unwrap_or(board);
            let remaining_after_action = remaining_mon_moves.saturating_sub(spirit_steps);
            for &target in spirit_pos.reachable_by_spirit_action_ref() {
                let Some(target_item) = action_board.item(target).copied() else {
                    continue;
                };
                if !spirit_target_allowed(target_item) {
                    continue;
                }
                for &dest in target.nearby_locations_ref() {
                    if !spirit_destination_allowed(&action_board, target, target_item, dest) {
                        continue;
                    }
                    let (after_board, score_delta, opponent_mana_score_delta) =
                        apply_spirit_move_preview(&action_board, target, target_item, dest, color);
                    let after_summary =
                        exact_followup_summary(&after_board, color, remaining_after_action);
                    let after_best_steps = after_summary.best_score_steps;
                    let after_opponent_steps = after_summary.opponent_best_score_steps;
                    let after_same_turn_score = score_delta.max(after_summary.immediate_score);
                    let after_same_turn_opponent_score =
                        opponent_mana_score_delta.max(after_summary.immediate_opponent_mana_score);
                    let supermana_progress_enabled = after_summary.secure_supermana
                        || matches!(
                            target_item,
                            Item::Mana {
                                mana: Mana::Supermana,
                            }
                        ) && score_delta > 0;
                    let opponent_progress_enabled =
                        after_summary.secure_opponent_mana || opponent_mana_score_delta > 0;
                    let own_gain = best_step_improvement(before_best_steps, after_best_steps);
                    let deny_gain = best_step_worsening(opponent_before, after_opponent_steps);
                    let same_turn_score_enabled =
                        score_delta > 0 || after_same_turn_score > before_same_turn_score;
                    let same_turn_opponent_score_enabled = opponent_mana_score_delta > 0
                        || after_same_turn_opponent_score > before_same_turn_opponent_score;
                    let score_gain = if same_turn_score_enabled {
                        after_same_turn_score
                            .saturating_sub(before_same_turn_score)
                            .max(score_delta)
                    } else {
                        0
                    };
                    let opponent_score_gain = if same_turn_opponent_score_enabled {
                        after_same_turn_opponent_score
                            .saturating_sub(before_same_turn_opponent_score)
                            .max(opponent_mana_score_delta)
                    } else {
                        0
                    };
                    let setup_gain = own_gain.saturating_add(deny_gain);
                    let utility =
                        exact_spirit_utility_score(score_gain, opponent_score_gain, setup_gain);

                    if same_turn_score_enabled {
                        best.same_turn_score = true;
                        best.same_turn_score_value =
                            best.same_turn_score_value.max(after_same_turn_score);
                    }
                    if supermana_progress_enabled {
                        best.supermana_progress = true;
                    }
                    if same_turn_opponent_score_enabled {
                        best.same_turn_opponent_mana_score = true;
                        best.same_turn_opponent_mana_score_value = best
                            .same_turn_opponent_mana_score_value
                            .max(after_same_turn_opponent_score);
                    }
                    if opponent_progress_enabled {
                        best.opponent_mana_progress = true;
                    }

                    if utility > best.utility {
                        best.utility = utility;
                        best.next_turn_setup_gain = setup_gain;
                    } else if utility == best.utility {
                        best.next_turn_setup_gain = best.next_turn_setup_gain.max(setup_gain);
                    }
                }
            }
        }
    }

    best
}

#[cfg(any(target_arch = "wasm32", test))]
fn exact_followup_summary(
    board: &Board,
    color: Color,
    remaining_moves: i32,
) -> ExactFollowupSummary {
    update_exact_query_diagnostics(|diagnostics| diagnostics.exact_followup_summary_calls += 1);
    if remaining_moves < 0 {
        return ExactFollowupSummary::default();
    }

    let board_hash = exact_board_hash(board);
    let key = ExactFollowupSummaryKey {
        board_hash,
        color,
        remaining_moves,
    };
    if let Some(cached) =
        EXACT_FOLLOWUP_SUMMARY_CACHE.with(|cache| cache.borrow().entries.get(&key).copied())
    {
        update_exact_query_diagnostics(|diagnostics| {
            diagnostics.exact_followup_summary_cache_hits += 1
        });
        return cached;
    }

    let summary = ExactFollowupSummary {
        best_score_steps: exact_best_score_steps_on_board_with_hash(board, color, board_hash),
        opponent_best_score_steps: exact_best_score_steps_on_board_with_hash(
            board,
            color.other(),
            board_hash,
        ),
        immediate_score: exact_best_immediate_score_on_board_with_hash(
            board,
            color,
            remaining_moves,
            board_hash,
        ),
        immediate_opponent_mana_score: exact_best_immediate_opponent_mana_score_on_board_with_hash(
            board,
            color,
            remaining_moves,
            board_hash,
        ),
        secure_supermana: can_secure_specific_mana_on_board(
            board,
            color,
            Mana::Supermana,
            remaining_moves,
        ),
        secure_opponent_mana: can_secure_specific_mana_on_board(
            board,
            color,
            Mana::Regular(color.other()),
            remaining_moves,
        ),
    };

    EXACT_FOLLOWUP_SUMMARY_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if cache.entries.len() >= EXACT_FOLLOWUP_SUMMARY_CACHE_MAX_ENTRIES
            && !cache.entries.contains_key(&key)
        {
            cache.entries.clear();
        }
        cache.entries.insert(key, summary);
    });
    summary
}

fn reachable_spirit_positions(
    board: &Board,
    start: Location,
    color: Color,
    remaining_mon_moves: i32,
) -> Vec<(Location, i32)> {
    if remaining_mon_moves < 0 {
        return Vec::new();
    }

    let key = ExactSpiritReachQueryKey {
        board_hash: exact_board_hash(board),
        start,
        color,
        remaining_mon_moves,
    };
    if let Some(cached) =
        EXACT_SPIRIT_REACH_CACHE.with(|cache| cache.borrow().entries.get(&key).cloned())
    {
        return cached;
    }

    let mut queue = VecDeque::with_capacity(EXACT_BFS_CAPACITY);
    let mut seen = ExactLocationSeen::new();
    queue.push_back((start, 0));
    seen.insert(start);
    let mut positions = Vec::new();

    while let Some((location, steps)) = queue.pop_front() {
        positions.push((location, steps));
        if steps >= remaining_mon_moves {
            continue;
        }
        for &next in location.nearby_locations_ref() {
            if seen.contains(next) {
                continue;
            }
            let item = board.item(next);
            let square = board.square(next);
            let passable = match item {
                Some(Item::Mon { .. })
                | Some(Item::MonWithMana { .. })
                | Some(Item::MonWithConsumable { .. })
                | Some(Item::Mana { .. }) => false,
                Some(Item::Consumable {
                    consumable: Consumable::BombOrPotion,
                }) => true,
                Some(Item::Consumable { .. }) => false,
                None => match square {
                    Square::Regular
                    | Square::ConsumableBase
                    | Square::ManaBase { .. }
                    | Square::ManaPool { .. } => true,
                    Square::MonBase {
                        kind: MonKind::Spirit,
                        color: base_color,
                    } => base_color == color,
                    Square::SupermanaBase | Square::MonBase { .. } => false,
                },
            };
            if passable {
                seen.insert(next);
                queue.push_back((next, steps + 1));
            }
        }
    }

    EXACT_SPIRIT_REACH_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if cache.entries.len() >= EXACT_SPIRIT_REACH_CACHE_MAX_ENTRIES
            && !cache.entries.contains_key(&key)
        {
            cache.entries.clear();
        }
        cache.entries.insert(key, positions.clone());
    });
    positions
}

fn spirit_target_allowed(item: Item) -> bool {
    match item {
        Item::Mon { mon } | Item::MonWithMana { mon, .. } | Item::MonWithConsumable { mon, .. } => {
            !mon.is_fainted()
        }
        Item::Mana { .. } | Item::Consumable { .. } => true,
    }
}

fn spirit_destination_allowed(
    board: &Board,
    _target_location: Location,
    target_item: Item,
    destination: Location,
) -> bool {
    let destination_item = board.item(destination).copied();
    let destination_square = board.square(destination);
    let target_mon = target_item.mon().copied();
    let target_mana = target_item.mana().copied();

    let valid_destination = match destination_item {
        Some(Item::Mon {
            mon: destination_mon,
        }) => match target_item {
            Item::Mon { .. } | Item::MonWithMana { .. } | Item::MonWithConsumable { .. } => false,
            Item::Mana { .. } => {
                destination_mon.kind == MonKind::Drainer && !destination_mon.is_fainted()
            }
            Item::Consumable {
                consumable: Consumable::BombOrPotion,
            } => true,
            Item::Consumable { .. } => false,
        },
        Some(Item::Mana { .. }) => {
            matches!(target_mon, Some(mon) if mon.kind == MonKind::Drainer && !mon.is_fainted())
        }
        Some(Item::MonWithMana { .. }) | Some(Item::MonWithConsumable { .. }) => {
            matches!(
                target_item,
                Item::Consumable {
                    consumable: Consumable::BombOrPotion,
                }
            )
        }
        Some(Item::Consumable {
            consumable: Consumable::BombOrPotion,
        }) => matches!(
            target_item,
            Item::Mon { .. } | Item::MonWithMana { .. } | Item::MonWithConsumable { .. }
        ),
        Some(Item::Consumable { .. }) => false,
        None => true,
    };

    if !valid_destination {
        return false;
    }

    match destination_square {
        Square::Regular
        | Square::ConsumableBase
        | Square::ManaBase { .. }
        | Square::ManaPool { .. } => true,
        Square::SupermanaBase => {
            target_mana == Some(Mana::Supermana)
                || (target_mana.is_none()
                    && matches!(target_mon.map(|mon| mon.kind), Some(MonKind::Drainer)))
        }
        Square::MonBase { kind, color } => {
            matches!(target_mon, Some(mon) if mon.kind == kind && mon.color == color)
                && target_mana.is_none()
                && target_item.consumable().is_none()
        }
    }
}

#[cfg(any(target_arch = "wasm32", test))]
fn apply_spirit_move_preview(
    board: &Board,
    from: Location,
    target_item: Item,
    to: Location,
    perspective: Color,
) -> (Board, i32, i32) {
    let mut board = board.clone();
    let destination_item = board.item(to).copied();
    let destination_square = board.square(to);
    board.remove_item(from);

    let mut placed_item = target_item;
    let mut score_delta = 0;
    let mut opponent_mana_score_delta = 0;

    match (target_item, destination_item) {
        (Item::Mon { mon }, Some(Item::Mana { mana })) => {
            placed_item = Item::MonWithMana { mon, mana };
        }
        (Item::Mana { mana }, Some(Item::Mon { mon })) => {
            placed_item = Item::MonWithMana { mon, mana };
        }
        (Item::MonWithMana { mon, mana: old }, Some(Item::Mana { mana: new })) => {
            board.put(Item::Mana { mana: old }, from);
            placed_item = Item::MonWithMana { mon, mana: new };
        }
        (Item::Consumable { .. }, Some(Item::Mon { mon })) => {
            placed_item = Item::Mon { mon };
        }
        (Item::Mon { mon }, Some(Item::Consumable { .. })) => {
            placed_item = Item::Mon { mon };
        }
        (Item::MonWithMana { mon, mana }, Some(Item::Consumable { .. })) => {
            placed_item = Item::MonWithMana { mon, mana };
        }
        (Item::MonWithConsumable { mon, .. }, Some(Item::Consumable { .. })) => {
            placed_item = Item::MonWithConsumable {
                mon,
                consumable: Consumable::Bomb,
            };
        }
        _ => {}
    }

    match destination_square {
        Square::ManaPool { .. } => {
            if let Some(mana) = placed_item.mana().copied() {
                score_delta = mana.score(perspective);
                if mana == Mana::Regular(perspective.other()) {
                    opponent_mana_score_delta = score_delta;
                }
                if let Some(mon) = placed_item.mon().copied() {
                    placed_item = Item::Mon { mon };
                } else {
                    board.remove_item(to);
                    return (board, score_delta, opponent_mana_score_delta);
                }
            }
        }
        _ => {}
    }

    board.put(placed_item, to);
    (board, score_delta, opponent_mana_score_delta)
}

#[cfg(any(target_arch = "wasm32", test))]
fn best_step_improvement(before: Option<i32>, after: Option<i32>) -> i32 {
    match (before, after) {
        (Some(before), Some(after)) if after < before => before - after,
        (None, Some(_)) => 2,
        _ => 0,
    }
}

#[cfg(any(target_arch = "wasm32", test))]
fn best_step_worsening(before: Option<i32>, after: Option<i32>) -> i32 {
    match (before, after) {
        (Some(before), Some(after)) if after > before => after - before,
        (Some(_), None) => 2,
        _ => 0,
    }
}

#[cfg(any(target_arch = "wasm32", test))]
fn exact_spirit_utility_score(score_delta: i32, opponent_score_delta: i32, setup_gain: i32) -> i32 {
    let score_bonus = if opponent_score_delta > 0 {
        5 + opponent_score_delta
    } else if score_delta > 0 {
        4 + score_delta
    } else {
        0
    };
    score_bonus.max((1 + setup_gain).min(EXACT_SPIRIT_UTILITY_CAP))
}

#[cfg(any(target_arch = "wasm32", test))]
pub(crate) fn exact_best_score_steps_on_board(board: &Board, color: Color) -> Option<i32> {
    exact_best_score_steps_on_board_with_hash(board, color, exact_board_hash(board))
}

#[cfg(any(target_arch = "wasm32", test))]
fn exact_best_score_steps_on_board_with_hash(
    board: &Board,
    color: Color,
    board_hash: u64,
) -> Option<i32> {
    let mut best = None;
    for (location, item) in board.occupied() {
        match item {
            Item::MonWithMana { mon, mana } if mon.color == color && !mon.is_fainted() => {
                if let Some(steps) =
                    exact_carrier_steps_to_any_pool_with_hash(board, location, *mana, board_hash)
                {
                    best = Some(best.map_or(steps, |current: i32| current.min(steps)));
                }
            }
            Item::Mon { mon } | Item::MonWithConsumable { mon, .. }
                if mon.color == color && mon.kind == MonKind::Drainer && !mon.is_fainted() =>
            {
                if let Some(path) = exact_best_drainer_pickup_path_filtered_with_hash(
                    board,
                    color,
                    location,
                    None,
                    ExactPickupFilter::Any,
                    board_hash,
                ) {
                    best = Some(best.map_or(path.total_moves, |current: i32| {
                        current.min(path.total_moves)
                    }));
                }
            }
            _ => {}
        }
    }
    best
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug, Clone, Copy, Default)]
struct ExactImmediateTacticalWindow {
    best_score: i32,
    best_opponent_mana_score: i32,
}

#[cfg(any(target_arch = "wasm32", test))]
fn exact_best_immediate_tactical_window_on_board_with_hash(
    board: &Board,
    color: Color,
    move_budget: i32,
    need_score: bool,
    need_denial: bool,
    board_hash: u64,
) -> ExactImmediateTacticalWindow {
    if move_budget < 0 || (!need_score && !need_denial) {
        return ExactImmediateTacticalWindow::default();
    }

    let opponent_mana = Mana::Regular(color.other());
    let max_score = if need_score {
        Mana::Supermana.score(color)
    } else {
        0
    };
    let max_opponent_mana_score = if need_denial {
        opponent_mana.score(color)
    } else {
        0
    };
    let mut best = ExactImmediateTacticalWindow::default();
    for (location, item) in board.occupied() {
        match item {
            Item::MonWithMana { mon, mana } if mon.color == color && !mon.is_fainted() => {
                if exact_carrier_steps_to_any_pool_with_hash(board, location, *mana, board_hash)
                    .map_or(false, |steps| steps <= move_budget)
                {
                    let mana_value = mana.score(color);
                    if need_score {
                        best.best_score = best.best_score.max(mana_value);
                    }
                    if need_denial && *mana == opponent_mana {
                        best.best_opponent_mana_score =
                            best.best_opponent_mana_score.max(mana_value);
                    }
                }
            }
            Item::Mon { mon } | Item::MonWithConsumable { mon, .. }
                if mon.color == color && mon.kind == MonKind::Drainer && !mon.is_fainted() =>
            {
                if need_score && need_denial {
                    let pickup = exact_drainer_pickup_window_uncached(
                        board,
                        color,
                        location,
                        Some(move_budget),
                        opponent_mana,
                    );
                    if let Some(path) = pickup.any {
                        best.best_score = best.best_score.max(path.mana_value);
                    }
                    if let Some(path) = pickup.opponent {
                        best.best_opponent_mana_score =
                            best.best_opponent_mana_score.max(path.mana_value);
                    }
                } else if need_score {
                    if let Some(path) = exact_best_drainer_pickup_path_filtered_with_hash(
                        board,
                        color,
                        location,
                        Some(move_budget),
                        ExactPickupFilter::Any,
                        board_hash,
                    ) {
                        best.best_score = best.best_score.max(path.mana_value);
                    }
                } else if need_denial {
                    if let Some(path) = exact_best_drainer_pickup_path_filtered_with_hash(
                        board,
                        color,
                        location,
                        Some(move_budget),
                        ExactPickupFilter::Wanted(opponent_mana),
                        board_hash,
                    ) {
                        best.best_opponent_mana_score =
                            best.best_opponent_mana_score.max(path.mana_value);
                    }
                }
            }
            _ => {}
        }

        let score_done = !need_score || best.best_score >= max_score;
        let denial_done = !need_denial || best.best_opponent_mana_score >= max_opponent_mana_score;
        if score_done && denial_done {
            return best;
        }
    }
    best
}

#[cfg(any(target_arch = "wasm32", test))]
fn exact_best_immediate_score_on_board(board: &Board, color: Color, move_budget: i32) -> i32 {
    exact_best_immediate_score_on_board_with_hash(
        board,
        color,
        move_budget,
        exact_board_hash(board),
    )
}

#[cfg(any(target_arch = "wasm32", test))]
fn exact_best_immediate_score_on_board_with_hash(
    board: &Board,
    color: Color,
    move_budget: i32,
    board_hash: u64,
) -> i32 {
    if move_budget < 0 {
        return 0;
    }

    let max_score = Mana::Supermana.score(color);
    let mut best = 0;
    for (location, item) in board.occupied() {
        match item {
            Item::MonWithMana { mon, mana } if mon.color == color && !mon.is_fainted() => {
                if exact_carrier_steps_to_any_pool_with_hash(board, location, *mana, board_hash)
                    .map_or(false, |steps| steps <= move_budget)
                {
                    best = best.max(mana.score(color));
                }
            }
            Item::Mon { mon } | Item::MonWithConsumable { mon, .. }
                if mon.color == color && mon.kind == MonKind::Drainer && !mon.is_fainted() =>
            {
                if let Some(path) = exact_best_drainer_pickup_path_filtered_with_hash(
                    board,
                    color,
                    location,
                    Some(move_budget),
                    ExactPickupFilter::Any,
                    board_hash,
                ) {
                    best = best.max(path.mana_value);
                }
            }
            _ => {}
        }

        if best >= max_score {
            return best;
        }
    }
    best
}

#[cfg(any(target_arch = "wasm32", test))]
fn exact_best_immediate_opponent_mana_score_on_board(
    board: &Board,
    color: Color,
    move_budget: i32,
) -> i32 {
    exact_best_immediate_opponent_mana_score_on_board_with_hash(
        board,
        color,
        move_budget,
        exact_board_hash(board),
    )
}

#[cfg(any(target_arch = "wasm32", test))]
fn exact_best_immediate_opponent_mana_score_on_board_with_hash(
    board: &Board,
    color: Color,
    move_budget: i32,
    board_hash: u64,
) -> i32 {
    if move_budget < 0 {
        return 0;
    }

    let mut best = 0;
    let opponent_mana = Mana::Regular(color.other());
    let max_opponent_score = opponent_mana.score(color);
    for (location, item) in board.occupied() {
        match item {
            Item::MonWithMana { mon, mana }
                if mon.color == color && !mon.is_fainted() && *mana == opponent_mana =>
            {
                if exact_carrier_steps_to_any_pool_with_hash(board, location, *mana, board_hash)
                    .map_or(false, |steps| steps <= move_budget)
                {
                    best = best.max(mana.score(color));
                }
            }
            Item::Mon { mon } | Item::MonWithConsumable { mon, .. }
                if mon.color == color && mon.kind == MonKind::Drainer && !mon.is_fainted() =>
            {
                if let Some(path) = exact_best_drainer_pickup_path_filtered_with_hash(
                    board,
                    color,
                    location,
                    Some(move_budget),
                    ExactPickupFilter::Wanted(opponent_mana),
                    board_hash,
                ) {
                    best = best.max(path.mana_value);
                }
            }
            _ => {}
        }

        if best >= max_opponent_score {
            return best;
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game_with_items(items: Vec<(Location, Item)>, active_color: Color) -> MonsGame {
        let mut game = MonsGame::new(false);
        game.board = Board::new_with_items(items.into_iter().collect());
        game.active_color = active_color;
        game.actions_used_count = 0;
        game.mana_moves_count = 0;
        game.mons_moves_count = 0;
        game.turn_number = 2;
        game.white_score = 0;
        game.black_score = 0;
        game.white_potions_count = 0;
        game.black_potions_count = 0;
        game
    }

    fn assert_secure_drainer_walk_matches_process_input(
        game: &MonsGame,
        from: Location,
        to: Location,
    ) {
        let helper =
            exact_apply_secure_drainer_walk(game, exact_secure_mana_state_key(game), from, to);
        let mut slow = game.clone_for_simulation();
        let inputs = [Input::Location(from), Input::Location(to)];
        match slow.process_input_slice(inputs.as_slice(), false, false) {
            Output::Events(events) => {
                let helper = helper.expect("helper should match legal drainer walk");
                let scored_mana = events.iter().find_map(|event| match event {
                    Event::ManaScored { mana, .. } => Some(*mana),
                    Event::MonMove { .. }
                    | Event::ManaMove { .. }
                    | Event::MysticAction { .. }
                    | Event::DemonAction { .. }
                    | Event::DemonAdditionalStep { .. }
                    | Event::SpiritTargetMove { .. }
                    | Event::PickupBomb { .. }
                    | Event::PickupPotion { .. }
                    | Event::PickupMana { .. }
                    | Event::MonFainted { .. }
                    | Event::ManaDropped { .. }
                    | Event::SupermanaBackToBase { .. }
                    | Event::BombAttack { .. }
                    | Event::BombExplosion { .. }
                    | Event::MonAwake { .. }
                    | Event::GameOver { .. }
                    | Event::NextTurn { .. }
                    | Event::Takeback
                    | Event::UsePotion { .. } => None,
                });
                assert_eq!(helper.scored_mana, scored_mana);
                assert_eq!(
                    MonsGameModel::search_state_hash(&helper.after),
                    MonsGameModel::search_state_hash(&slow)
                );
                assert_eq!(
                    helper.after_key.board_hash,
                    exact_secure_board_hash(&helper.after.board)
                );
                assert_eq!(helper.after_key.active_color, helper.after.active_color);
                assert_eq!(
                    helper.after_key.mons_moves_count,
                    helper.after.mons_moves_count
                );
            }
            Output::InvalidInput
            | Output::LocationsToStartFrom(_)
            | Output::NextInputOptions(_) => {
                assert!(helper.is_none());
            }
        }
    }

    #[test]
    fn exact_pickup_path_finds_same_turn_supermana_score() {
        let game = game_with_items(
            vec![
                (
                    Location::new(8, 1),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(9, 1),
                    Item::Mana {
                        mana: Mana::Regular(Color::White),
                    },
                ),
            ],
            Color::White,
        );
        let summary = exact_state_analysis(&game).white;
        let best = summary.best_drainer_pickup.expect("pickup path");
        assert_eq!(best.mana, Mana::Regular(Color::White));
        assert_eq!(best.mana_value, 1);
        assert!(best.total_moves <= Config::MONS_MOVES_PER_TURN);
    }

    #[test]
    fn exact_turn_summary_detects_safe_supermana_progress() {
        let game = game_with_items(
            vec![
                (
                    Location::new(6, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(5, 5),
                    Item::Mana {
                        mana: Mana::Supermana,
                    },
                ),
                (
                    Location::new(0, 10),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
        );
        let turn = exact_turn_summary(&game, Color::White);
        assert!(turn.safe_supermana_progress);
        assert_eq!(turn.safe_supermana_progress_steps, Some(1));
    }

    #[test]
    fn exact_turn_summary_detects_two_step_safe_supermana_progress() {
        let game = game_with_items(
            vec![
                (
                    Location::new(7, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(5, 5),
                    Item::Mana {
                        mana: Mana::Supermana,
                    },
                ),
                (
                    Location::new(0, 10),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
        );
        let turn = exact_turn_summary(&game, Color::White);
        assert!(turn.safe_supermana_progress);
        assert_eq!(turn.safe_supermana_progress_steps, Some(2));
    }

    #[test]
    fn exact_secure_specific_mana_path_reconstructs_safe_supermana_pickup() {
        let game = game_with_items(
            vec![
                (
                    Location::new(6, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(5, 5),
                    Item::Mana {
                        mana: Mana::Supermana,
                    },
                ),
                (
                    Location::new(0, 10),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
        );

        assert_eq!(
            exact_secure_specific_mana_path_from(
                &game,
                Color::White,
                Location::new(6, 5),
                Mana::Supermana,
            ),
            Some(vec![Location::new(5, 5)])
        );
    }

    #[test]
    fn exact_secure_drainer_walk_matches_process_input_for_pickup() {
        let game = game_with_items(
            vec![
                (
                    Location::new(6, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(5, 5),
                    Item::Mana {
                        mana: Mana::Supermana,
                    },
                ),
                (
                    Location::new(0, 10),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
        );

        assert_secure_drainer_walk_matches_process_input(
            &game,
            Location::new(6, 5),
            Location::new(5, 5),
        );
    }

    #[test]
    fn exact_secure_drainer_walk_matches_process_input_for_score() {
        let game = game_with_items(
            vec![
                (
                    Location::new(9, 1),
                    Item::MonWithMana {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                        mana: Mana::Supermana,
                    },
                ),
                (
                    Location::new(0, 10),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
        );

        assert_secure_drainer_walk_matches_process_input(
            &game,
            Location::new(9, 1),
            Location::new(10, 0),
        );
    }

    #[test]
    fn exact_secure_drainer_walk_matches_process_input_for_invalid_consumable_pickup() {
        let game = game_with_items(
            vec![
                (
                    Location::new(6, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(5, 5),
                    Item::Consumable {
                        consumable: Consumable::BombOrPotion,
                    },
                ),
                (
                    Location::new(0, 10),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
        );

        assert_secure_drainer_walk_matches_process_input(
            &game,
            Location::new(6, 5),
            Location::new(5, 5),
        );
    }

    #[test]
    fn exact_secure_drainer_walk_tracks_next_turn_state_key() {
        let mut game = game_with_items(
            vec![
                (
                    Location::new(9, 1),
                    Item::MonWithMana {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                        mana: Mana::Regular(Color::White),
                    },
                ),
                (
                    Location::new(0, 10),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
                (
                    Location::new(4, 4),
                    Item::Mon {
                        mon: Mon::new(MonKind::Mystic, Color::Black, 1),
                    },
                ),
            ],
            Color::White,
        );
        game.mons_moves_count = Config::MONS_MOVES_PER_TURN - 1;

        let transition = exact_apply_secure_drainer_walk(
            &game,
            exact_secure_mana_state_key(&game),
            Location::new(9, 1),
            Location::new(10, 0),
        )
        .expect("score move should be legal");

        assert_eq!(transition.scored_mana, Some(Mana::Regular(Color::White)));
        assert_eq!(transition.after.active_color, Color::Black);
        assert_eq!(transition.after.mons_moves_count, 0);
        assert_eq!(
            transition.after_key.board_hash,
            exact_secure_board_hash(&transition.after.board)
        );
        assert_eq!(
            transition.after_key.active_color,
            transition.after.active_color
        );
        assert_eq!(
            transition.after_key.mons_moves_count,
            transition.after.mons_moves_count
        );
    }

    #[test]
    fn exact_attack_cache_preserves_repeated_mystic_reach_result() {
        clear_exact_state_analysis_cache();
        let board = game_with_items(
            vec![
                (
                    Location::new(6, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(2, 7),
                    Item::Mon {
                        mon: Mon::new(MonKind::Mystic, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
        )
        .board;
        let first = can_attack_target_on_board(
            &board,
            Color::Black,
            Color::White,
            Location::new(6, 5),
            2,
            true,
        );
        let second = can_attack_target_on_board(
            &board,
            Color::Black,
            Color::White,
            Location::new(6, 5),
            2,
            true,
        );
        clear_exact_state_analysis_cache();
        let third = can_attack_target_on_board(
            &board,
            Color::Black,
            Color::White,
            Location::new(6, 5),
            2,
            true,
        );

        assert!(first);
        assert_eq!(first, second);
        assert_eq!(first, third);
    }

    #[test]
    fn exact_carrier_steps_cache_preserves_repeated_result() {
        clear_exact_state_analysis_cache();
        let board = game_with_items(
            vec![
                (
                    Location::new(8, 5),
                    Item::MonWithMana {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                        mana: Mana::Supermana,
                    },
                ),
                (
                    Location::new(0, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
        )
        .board;

        let first = exact_carrier_steps_to_any_pool(&board, Location::new(8, 5), Mana::Supermana);
        let second = exact_carrier_steps_to_any_pool(&board, Location::new(8, 5), Mana::Supermana);
        clear_exact_state_analysis_cache();
        let third = exact_carrier_steps_to_any_pool(&board, Location::new(8, 5), Mana::Supermana);

        assert_eq!(first, second);
        assert_eq!(first, third);
    }

    #[test]
    fn exact_drainer_to_mana_cache_preserves_repeated_result() {
        clear_exact_state_analysis_cache();
        let board = game_with_items(
            vec![
                (
                    Location::new(8, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(7, 5),
                    Item::Mana {
                        mana: Mana::Supermana,
                    },
                ),
                (
                    Location::new(0, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
        )
        .board;

        let first = exact_drainer_to_any_mana_steps(&board, Color::White, Location::new(8, 5));
        let second = exact_drainer_to_any_mana_steps(&board, Color::White, Location::new(8, 5));
        clear_exact_state_analysis_cache();
        let third = exact_drainer_to_any_mana_steps(&board, Color::White, Location::new(8, 5));

        assert_eq!(first, second);
        assert_eq!(first, third);
    }

    #[test]
    fn exact_walk_threat_cache_preserves_repeated_bomb_walk_threat_result() {
        clear_exact_state_analysis_cache();
        let board = game_with_items(
            vec![
                (
                    Location::new(6, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(2, 5),
                    Item::MonWithConsumable {
                        mon: Mon::new(MonKind::Demon, Color::Black, 0),
                        consumable: Consumable::Bomb,
                    },
                ),
            ],
            Color::White,
        )
        .board;
        let first = is_drainer_under_walk_threat(&board, Color::White, Location::new(6, 5), false);
        let second = is_drainer_under_walk_threat(&board, Color::White, Location::new(6, 5), false);
        clear_exact_state_analysis_cache();
        let third = is_drainer_under_walk_threat(&board, Color::White, Location::new(6, 5), false);

        assert!(first);
        assert_eq!(first, second);
        assert_eq!(first, third);
    }

    #[test]
    fn exact_drainer_safety_helper_matches_cached_components() {
        clear_exact_state_analysis_cache();
        let board = game_with_items(
            vec![
                (
                    Location::new(6, 5),
                    Item::MonWithMana {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                        mana: Mana::Supermana,
                    },
                ),
                (
                    Location::new(0, 10),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
        )
        .board;
        let angel_nearby =
            exact_is_location_guarded_by_angel(&board, Color::White, Location::new(6, 5));
        let expected = !can_attack_target_on_board(
            &board,
            Color::Black,
            Color::White,
            Location::new(6, 5),
            Config::MONS_MOVES_PER_TURN,
            true,
        ) && !is_drainer_under_walk_threat(
            &board,
            Color::White,
            Location::new(6, 5),
            angel_nearby,
        );

        assert_eq!(
            is_drainer_exactly_safe_next_turn_on_board(&board, Color::White, Location::new(6, 5)),
            expected
        );
    }

    #[test]
    fn exact_search_state_hash_matches_model_search_state_hash() {
        let mut game = game_with_items(
            vec![
                (
                    Location::new(4, 4),
                    Item::MonWithMana {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                        mana: Mana::Supermana,
                    },
                ),
                (
                    Location::new(7, 6),
                    Item::MonWithConsumable {
                        mon: Mon::new(MonKind::Demon, Color::Black, 1),
                        consumable: Consumable::Bomb,
                    },
                ),
                (
                    Location::new(6, 5),
                    Item::Consumable {
                        consumable: Consumable::Potion,
                    },
                ),
            ],
            Color::Black,
        );
        game.white_score = 1;
        game.black_score = 2;
        game.turn_number = 7;
        game.actions_used_count = 1;
        game.mana_moves_count = 1;
        game.mons_moves_count = 2;
        game.white_potions_count = 1;
        game.black_potions_count = 0;

        assert_eq!(
            exact_search_state_hash(&game),
            MonsGameModel::search_state_hash(&game)
        );
    }

    #[test]
    fn exact_angel_guard_helper_matches_model_helper() {
        let board = game_with_items(
            vec![
                (
                    Location::new(5, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Angel, Color::White, 0),
                    },
                ),
                (
                    Location::new(1, 1),
                    Item::Mon {
                        mon: Mon::new(MonKind::Angel, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
        )
        .board;

        assert_eq!(
            exact_is_location_guarded_by_angel(&board, Color::White, Location::new(6, 5)),
            MonsGameModel::is_location_guarded_by_angel(&board, Color::White, Location::new(6, 5))
        );
        assert_eq!(
            exact_is_location_guarded_by_angel(&board, Color::White, Location::new(7, 5)),
            MonsGameModel::is_location_guarded_by_angel(&board, Color::White, Location::new(7, 5))
        );
        assert_eq!(
            exact_is_location_guarded_by_angel(&board, Color::Black, Location::new(1, 2)),
            MonsGameModel::is_location_guarded_by_angel(&board, Color::Black, Location::new(1, 2))
        );
    }

    #[test]
    fn exact_followup_summary_matches_component_queries() {
        clear_exact_state_analysis_cache();
        let mut game = game_with_items(
            vec![
                (
                    Location::new(5, 1),
                    Item::Mon {
                        mon: Mon::new(MonKind::Spirit, Color::White, 0),
                    },
                ),
                (
                    Location::new(8, 2),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(7, 1),
                    Item::Mana {
                        mana: Mana::Supermana,
                    },
                ),
                (
                    Location::new(0, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
        );
        game.mons_moves_count = Config::MONS_MOVES_PER_TURN - 1;

        let summary = exact_followup_summary(&game.board, Color::White, 1);
        assert_eq!(
            summary.best_score_steps,
            exact_best_score_steps_on_board(&game.board, Color::White)
        );
        assert_eq!(
            summary.opponent_best_score_steps,
            exact_best_score_steps_on_board(&game.board, Color::Black)
        );
        assert_eq!(
            summary.immediate_score,
            exact_best_immediate_score_on_board(&game.board, Color::White, 1)
        );
        assert_eq!(
            summary.immediate_opponent_mana_score,
            exact_best_immediate_opponent_mana_score_on_board(&game.board, Color::White, 1)
        );
        assert_eq!(
            summary.secure_supermana,
            can_secure_specific_mana_on_board(&game.board, Color::White, Mana::Supermana, 1)
        );
        assert_eq!(
            summary.secure_opponent_mana,
            can_secure_specific_mana_on_board(
                &game.board,
                Color::White,
                Mana::Regular(Color::Black),
                1,
            )
        );
    }

    #[test]
    fn exact_followup_summary_cache_preserves_repeated_result() {
        clear_exact_state_analysis_cache();
        let board = game_with_items(
            vec![
                (
                    Location::new(9, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(8, 5),
                    Item::Mana {
                        mana: Mana::Regular(Color::Black),
                    },
                ),
                (
                    Location::new(0, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
        )
        .board;

        let first = exact_followup_summary(&board, Color::White, 2);
        let second = exact_followup_summary(&board, Color::White, 2);
        clear_exact_state_analysis_cache();
        let third = exact_followup_summary(&board, Color::White, 2);

        assert_eq!(first.best_score_steps, second.best_score_steps);
        assert_eq!(
            first.opponent_best_score_steps,
            second.opponent_best_score_steps
        );
        assert_eq!(first.immediate_score, second.immediate_score);
        assert_eq!(
            first.immediate_opponent_mana_score,
            second.immediate_opponent_mana_score
        );
        assert_eq!(first.secure_supermana, second.secure_supermana);
        assert_eq!(first.secure_opponent_mana, second.secure_opponent_mana);
        assert_eq!(first.best_score_steps, third.best_score_steps);
        assert_eq!(first.immediate_score, third.immediate_score);
        assert_eq!(first.secure_opponent_mana, third.secure_opponent_mana);
    }

    #[test]
    fn exact_pickup_path_cache_preserves_repeated_filtered_result() {
        clear_exact_state_analysis_cache();
        let board = game_with_items(
            vec![
                (
                    Location::new(8, 4),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(7, 4),
                    Item::Mana {
                        mana: Mana::Regular(Color::Black),
                    },
                ),
                (
                    Location::new(0, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
        )
        .board;

        let filter = ExactPickupFilter::Wanted(Mana::Regular(Color::Black));
        let first = exact_best_drainer_pickup_path_filtered(
            &board,
            Color::White,
            Location::new(8, 4),
            Some(2),
            filter,
        );
        let second = exact_best_drainer_pickup_path_filtered(
            &board,
            Color::White,
            Location::new(8, 4),
            Some(2),
            filter,
        );
        clear_exact_state_analysis_cache();
        let third = exact_best_drainer_pickup_path_filtered(
            &board,
            Color::White,
            Location::new(8, 4),
            Some(2),
            filter,
        );

        assert_eq!(
            first.map(|path| path.total_moves),
            second.map(|path| path.total_moves)
        );
        assert_eq!(first.map(|path| path.mana), second.map(|path| path.mana));
        assert_eq!(
            first.map(|path| path.total_moves),
            third.map(|path| path.total_moves)
        );
        assert_eq!(
            first.map(|path| path.mana_value),
            third.map(|path| path.mana_value)
        );
    }

    #[test]
    fn exact_secure_mana_cache_preserves_repeated_supermana_result() {
        clear_exact_state_analysis_cache();
        let game = game_with_items(
            vec![
                (
                    Location::new(6, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(5, 5),
                    Item::Mana {
                        mana: Mana::Supermana,
                    },
                ),
                (
                    Location::new(0, 10),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
        );
        let first = exact_secure_specific_mana_steps_on_board(
            &game.board,
            Color::White,
            Mana::Supermana,
            5,
        );
        let second = exact_secure_specific_mana_steps_on_board(
            &game.board,
            Color::White,
            Mana::Supermana,
            5,
        );
        clear_exact_state_analysis_cache();
        let third = exact_secure_specific_mana_steps_on_board(
            &game.board,
            Color::White,
            Mana::Supermana,
            5,
        );

        assert_eq!(first, second);
        assert_eq!(first, third);
    }

    #[test]
    fn exact_secure_mana_steps_find_shortest_supermana_score_path() {
        clear_exact_state_analysis_cache();
        let board = game_with_items(
            vec![
                (
                    Location::new(8, 1),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(9, 1),
                    Item::Mana {
                        mana: Mana::Supermana,
                    },
                ),
                (
                    Location::new(0, 10),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
        )
        .board;

        assert_eq!(
            exact_secure_specific_mana_steps_on_board(&board, Color::White, Mana::Supermana, 5),
            Some(1)
        );
    }

    #[test]
    fn exact_secure_mana_steps_allow_last_move_supermana_pickup() {
        clear_exact_state_analysis_cache();
        let board = game_with_items(
            vec![
                (
                    Location::new(6, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(5, 5),
                    Item::Mana {
                        mana: Mana::Supermana,
                    },
                ),
                (
                    Location::new(0, 10),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
        )
        .board;

        assert_eq!(
            exact_secure_specific_mana_steps_on_board(&board, Color::White, Mana::Supermana, 1),
            Some(1)
        );
    }

    #[test]
    fn exact_turn_summary_detects_safe_opponent_mana_progress_steps() {
        let game = game_with_items(
            vec![
                (
                    Location::new(6, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(5, 4),
                    Item::Mana {
                        mana: Mana::Regular(Color::Black),
                    },
                ),
                (
                    Location::new(0, 10),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
        );
        let turn = exact_turn_summary(&game, Color::White);
        assert!(turn.safe_opponent_mana_progress);
        assert_eq!(turn.safe_opponent_mana_progress_steps, Some(1));
    }

    #[test]
    fn exact_turn_summary_detects_two_step_safe_opponent_mana_progress() {
        let game = game_with_items(
            vec![
                (
                    Location::new(7, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(5, 4),
                    Item::Mana {
                        mana: Mana::Regular(Color::Black),
                    },
                ),
                (
                    Location::new(0, 10),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
        );
        let turn = exact_turn_summary(&game, Color::White);
        assert!(turn.safe_opponent_mana_progress);
        assert_eq!(turn.safe_opponent_mana_progress_steps, Some(2));
    }

    #[test]
    fn exact_secure_specific_mana_path_reconstructs_safe_opponent_mana_pickup() {
        let game = game_with_items(
            vec![
                (
                    Location::new(6, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(5, 4),
                    Item::Mana {
                        mana: Mana::Regular(Color::Black),
                    },
                ),
                (
                    Location::new(0, 10),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
        );

        assert_eq!(
            exact_secure_specific_mana_path_from(
                &game,
                Color::White,
                Location::new(6, 5),
                Mana::Regular(Color::Black),
            ),
            Some(vec![Location::new(5, 4)])
        );
    }

    #[test]
    fn exact_turn_summary_rejects_exact_vulnerable_supermana_progress() {
        let mut game = game_with_items(
            vec![
                (
                    Location::new(9, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(8, 5),
                    Item::Mana {
                        mana: Mana::Supermana,
                    },
                ),
                (
                    Location::new(4, 7),
                    Item::Mon {
                        mon: Mon::new(MonKind::Mystic, Color::Black, 0),
                    },
                ),
                (
                    Location::new(0, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
        );
        game.mons_moves_count = Config::MONS_MOVES_PER_TURN - 1;

        assert!(!exact_turn_summary(&game, Color::White).safe_supermana_progress);
    }

    #[test]
    fn exact_turn_summary_rejects_exact_vulnerable_opponent_mana_progress() {
        let mut game = game_with_items(
            vec![
                (
                    Location::new(9, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(8, 5),
                    Item::Mana {
                        mana: Mana::Regular(Color::Black),
                    },
                ),
                (
                    Location::new(4, 7),
                    Item::Mon {
                        mon: Mon::new(MonKind::Mystic, Color::Black, 0),
                    },
                ),
                (
                    Location::new(0, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
        );
        game.mons_moves_count = Config::MONS_MOVES_PER_TURN - 1;

        assert!(!exact_turn_summary(&game, Color::White).safe_opponent_mana_progress);
    }

    #[test]
    fn exact_turn_summary_rejects_spirit_assisted_supermana_progress_without_safe_followup() {
        let mut game = game_with_items(
            vec![
                (
                    Location::new(5, 1),
                    Item::Mon {
                        mon: Mon::new(MonKind::Spirit, Color::White, 0),
                    },
                ),
                (
                    Location::new(8, 2),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(7, 1),
                    Item::Mana {
                        mana: Mana::Supermana,
                    },
                ),
                (
                    Location::new(5, 3),
                    Item::Mon {
                        mon: Mon::new(MonKind::Mystic, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
        );
        game.mons_moves_count = Config::MONS_MOVES_PER_TURN - 1;

        let turn = exact_turn_summary(&game, Color::White);
        assert!(!turn.safe_supermana_progress);
        assert!(!turn.spirit_assisted_supermana_progress);
        assert!(!turn.spirit_assisted_score);
    }

    #[test]
    fn exact_turn_summary_rejects_spirit_assisted_opponent_mana_progress_without_safe_followup() {
        let mut game = game_with_items(
            vec![
                (
                    Location::new(5, 1),
                    Item::Mon {
                        mon: Mon::new(MonKind::Spirit, Color::White, 0),
                    },
                ),
                (
                    Location::new(8, 2),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(7, 1),
                    Item::Mana {
                        mana: Mana::Regular(Color::Black),
                    },
                ),
                (
                    Location::new(5, 3),
                    Item::Mon {
                        mon: Mon::new(MonKind::Mystic, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
        );
        game.mons_moves_count = Config::MONS_MOVES_PER_TURN - 1;

        let turn = exact_turn_summary(&game, Color::White);
        assert!(!turn.safe_opponent_mana_progress);
        assert!(!turn.spirit_assisted_opponent_mana_progress);
        assert!(!turn.spirit_assisted_denial);
    }

    #[test]
    fn exact_turn_summary_detects_two_step_spirit_assisted_supermana_progress() {
        let mut game = game_with_items(
            vec![
                (
                    Location::new(5, 1),
                    Item::Mon {
                        mon: Mon::new(MonKind::Spirit, Color::White, 0),
                    },
                ),
                (
                    Location::new(10, 4),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(7, 1),
                    Item::Mana {
                        mana: Mana::Supermana,
                    },
                ),
                (
                    Location::new(0, 10),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
        );
        game.mons_moves_count = Config::MONS_MOVES_PER_TURN - 2;

        let (after_board, score_delta, opponent_score_delta) = apply_spirit_move_preview(
            &game.board,
            Location::new(7, 1),
            Item::Mana {
                mana: Mana::Supermana,
            },
            Location::new(8, 2),
            Color::White,
        );
        let after_summary = exact_followup_summary(&after_board, Color::White, 2);
        assert_eq!(score_delta, 0);
        assert_eq!(opponent_score_delta, 0);
        assert!(after_summary.secure_supermana);

        let turn = exact_turn_summary(&game, Color::White);
        assert!(!turn.safe_supermana_progress);
        assert!(turn.spirit_assisted_supermana_progress);
        assert!(!turn.spirit_assisted_score);
    }

    #[test]
    fn exact_turn_summary_detects_two_step_spirit_assisted_opponent_mana_progress() {
        let mut game = game_with_items(
            vec![
                (
                    Location::new(5, 1),
                    Item::Mon {
                        mon: Mon::new(MonKind::Spirit, Color::White, 0),
                    },
                ),
                (
                    Location::new(10, 4),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(7, 1),
                    Item::Mana {
                        mana: Mana::Regular(Color::Black),
                    },
                ),
                (
                    Location::new(0, 10),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
        );
        game.mons_moves_count = Config::MONS_MOVES_PER_TURN - 2;

        let (after_board, score_delta, opponent_score_delta) = apply_spirit_move_preview(
            &game.board,
            Location::new(7, 1),
            Item::Mana {
                mana: Mana::Regular(Color::Black),
            },
            Location::new(8, 2),
            Color::White,
        );
        let after_summary = exact_followup_summary(&after_board, Color::White, 2);
        assert_eq!(score_delta, 0);
        assert_eq!(opponent_score_delta, 0);
        assert!(after_summary.secure_opponent_mana);

        let turn = exact_turn_summary(&game, Color::White);
        assert!(!turn.safe_opponent_mana_progress);
        assert!(turn.spirit_assisted_opponent_mana_progress);
        assert!(!turn.spirit_assisted_denial);
    }

    #[test]
    fn exact_turn_summary_detects_two_step_spirit_assisted_supermana_score() {
        let mut game = game_with_items(
            vec![
                (
                    Location::new(5, 1),
                    Item::Mon {
                        mon: Mon::new(MonKind::Spirit, Color::White, 0),
                    },
                ),
                (
                    Location::new(8, 1),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(7, 1),
                    Item::Mana {
                        mana: Mana::Supermana,
                    },
                ),
                (
                    Location::new(0, 10),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
        );
        game.mons_moves_count = Config::MONS_MOVES_PER_TURN - 2;

        assert_eq!(
            exact_best_immediate_score_on_board(&game.board, Color::White, 2),
            0
        );

        let (after_board, score_delta, opponent_score_delta) = apply_spirit_move_preview(
            &game.board,
            Location::new(7, 1),
            Item::Mana {
                mana: Mana::Supermana,
            },
            Location::new(8, 1),
            Color::White,
        );
        let after_summary = exact_followup_summary(&after_board, Color::White, 2);
        assert_eq!(score_delta, 0);
        assert_eq!(opponent_score_delta, 0);
        assert_eq!(
            after_summary.immediate_score,
            Mana::Supermana.score(Color::White)
        );

        let turn = exact_turn_summary(&game, Color::White);
        assert!(turn.spirit_assisted_score);
        assert!(!turn.spirit_assisted_denial);
    }

    #[test]
    fn exact_turn_summary_detects_two_step_spirit_assisted_opponent_mana_score() {
        let mut game = game_with_items(
            vec![
                (
                    Location::new(5, 1),
                    Item::Mon {
                        mon: Mon::new(MonKind::Spirit, Color::White, 0),
                    },
                ),
                (
                    Location::new(8, 1),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(7, 1),
                    Item::Mana {
                        mana: Mana::Regular(Color::Black),
                    },
                ),
                (
                    Location::new(0, 10),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
        );
        game.mons_moves_count = Config::MONS_MOVES_PER_TURN - 2;

        assert_eq!(
            exact_best_immediate_opponent_mana_score_on_board(&game.board, Color::White, 2),
            0
        );

        let (after_board, score_delta, opponent_score_delta) = apply_spirit_move_preview(
            &game.board,
            Location::new(7, 1),
            Item::Mana {
                mana: Mana::Regular(Color::Black),
            },
            Location::new(8, 1),
            Color::White,
        );
        let after_summary = exact_followup_summary(&after_board, Color::White, 2);
        assert_eq!(score_delta, 0);
        assert_eq!(opponent_score_delta, 0);
        assert_eq!(
            after_summary.immediate_opponent_mana_score,
            Mana::Regular(Color::Black).score(Color::White)
        );

        let turn = exact_turn_summary(&game, Color::White);
        assert!(turn.spirit_assisted_score);
        assert!(turn.spirit_assisted_denial);
    }

    #[test]
    fn exact_turn_summary_detects_move_then_spirit_assisted_supermana_score() {
        let mut game = game_with_items(
            vec![
                (
                    Location::new(5, 1),
                    Item::Mon {
                        mon: Mon::new(MonKind::Spirit, Color::White, 0),
                    },
                ),
                (
                    Location::new(9, 0),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(8, 1),
                    Item::Mana {
                        mana: Mana::Supermana,
                    },
                ),
                (
                    Location::new(0, 10),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
        );
        game.mons_moves_count = Config::MONS_MOVES_PER_TURN - 2;

        assert_eq!(
            exact_best_immediate_score_on_board(&game.board, Color::White, 2),
            0
        );

        let mut action_board = game.board.clone();
        action_board.remove_item(Location::new(5, 1));
        action_board.put(
            Item::Mon {
                mon: Mon::new(MonKind::Spirit, Color::White, 0),
            },
            Location::new(6, 1),
        );

        let (after_board, score_delta, opponent_score_delta) = apply_spirit_move_preview(
            &action_board,
            Location::new(8, 1),
            Item::Mana {
                mana: Mana::Supermana,
            },
            Location::new(9, 0),
            Color::White,
        );
        let after_summary = exact_followup_summary(&after_board, Color::White, 1);
        assert_eq!(score_delta, 0);
        assert_eq!(opponent_score_delta, 0);
        assert_eq!(
            after_summary.immediate_score,
            Mana::Supermana.score(Color::White)
        );

        let spirit = exact_state_analysis(&game).white.spirit;
        assert!(spirit.same_turn_score);

        let turn = exact_turn_summary(&game, Color::White);
        assert!(turn.spirit_assisted_score);
        assert!(!turn.spirit_assisted_denial);
    }

    #[test]
    fn exact_turn_summary_detects_move_then_spirit_assisted_opponent_mana_score() {
        let mut game = game_with_items(
            vec![
                (
                    Location::new(5, 1),
                    Item::Mon {
                        mon: Mon::new(MonKind::Spirit, Color::White, 0),
                    },
                ),
                (
                    Location::new(9, 0),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(8, 1),
                    Item::Mana {
                        mana: Mana::Regular(Color::Black),
                    },
                ),
                (
                    Location::new(0, 10),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
        );
        game.mons_moves_count = Config::MONS_MOVES_PER_TURN - 2;

        assert_eq!(
            exact_best_immediate_opponent_mana_score_on_board(&game.board, Color::White, 2),
            0
        );

        let mut action_board = game.board.clone();
        action_board.remove_item(Location::new(5, 1));
        action_board.put(
            Item::Mon {
                mon: Mon::new(MonKind::Spirit, Color::White, 0),
            },
            Location::new(6, 1),
        );

        let (after_board, score_delta, opponent_score_delta) = apply_spirit_move_preview(
            &action_board,
            Location::new(8, 1),
            Item::Mana {
                mana: Mana::Regular(Color::Black),
            },
            Location::new(9, 0),
            Color::White,
        );
        let after_summary = exact_followup_summary(&after_board, Color::White, 1);
        assert_eq!(score_delta, 0);
        assert_eq!(opponent_score_delta, 0);
        assert_eq!(
            after_summary.immediate_opponent_mana_score,
            Mana::Regular(Color::Black).score(Color::White)
        );

        let spirit = exact_state_analysis(&game).white.spirit;
        assert!(spirit.same_turn_score);
        assert!(spirit.same_turn_opponent_mana_score);

        let turn = exact_turn_summary(&game, Color::White);
        assert!(turn.spirit_assisted_score);
        assert!(turn.spirit_assisted_denial);
    }

    #[test]
    fn exact_spirit_summary_detects_same_turn_opponent_mana_score() {
        let game = game_with_items(
            vec![
                (
                    Location::new(7, 1),
                    Item::Mon {
                        mon: Mon::new(MonKind::Spirit, Color::White, 0),
                    },
                ),
                (
                    Location::new(9, 1),
                    Item::Mana {
                        mana: Mana::Regular(Color::Black),
                    },
                ),
            ],
            Color::White,
        );
        let spirit = exact_state_analysis(&game).white.spirit;
        assert!(spirit.same_turn_score);
        assert!(spirit.same_turn_opponent_mana_score);
        assert_eq!(spirit.same_turn_opponent_mana_score_value, 2);
    }

    #[test]
    fn exact_spirit_summary_detects_same_turn_setup_into_drainer_score() {
        let game = game_with_items(
            vec![
                (
                    Location::new(7, 1),
                    Item::Mon {
                        mon: Mon::new(MonKind::Spirit, Color::White, 0),
                    },
                ),
                (
                    Location::new(9, 0),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(9, 1),
                    Item::Mana {
                        mana: Mana::Regular(Color::Black),
                    },
                ),
            ],
            Color::White,
        );
        let spirit = exact_state_analysis(&game).white.spirit;
        assert!(spirit.same_turn_score);
        assert!(spirit.same_turn_opponent_mana_score);
    }

    #[test]
    fn exact_spirit_summary_detects_bridge_move_into_drainer_score() {
        let game = game_with_items(
            vec![
                (
                    Location::new(4, 0),
                    Item::Mon {
                        mon: Mon::new(MonKind::Spirit, Color::White, 0),
                    },
                ),
                (
                    Location::new(4, 1),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(5, 0),
                    Item::Mana {
                        mana: Mana::Regular(Color::Black),
                    },
                ),
            ],
            Color::White,
        );

        assert_eq!(
            exact_best_immediate_score_on_board(
                &game.board,
                Color::White,
                Config::MONS_MOVES_PER_TURN,
            ),
            0
        );

        clear_exact_state_analysis_cache();
        assert!(exact_turn_summary(&game, Color::White).spirit_assisted_score);
        let spirit =
            exact_spirit_summary(&game.board, Color::White, Config::MONS_MOVES_PER_TURN, true);
        assert!(spirit.same_turn_score);
        assert_eq!(spirit.same_turn_score_value, 2);
        assert!(spirit.same_turn_opponent_mana_score);
        assert_eq!(spirit.same_turn_opponent_mana_score_value, 2);
    }

    #[test]
    fn exact_spirit_summary_cache_preserves_repeated_result() {
        clear_exact_state_analysis_cache();
        let board = game_with_items(
            vec![
                (
                    Location::new(7, 1),
                    Item::Mon {
                        mon: Mon::new(MonKind::Spirit, Color::White, 0),
                    },
                ),
                (
                    Location::new(9, 0),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(9, 1),
                    Item::Mana {
                        mana: Mana::Regular(Color::Black),
                    },
                ),
            ],
            Color::White,
        )
        .board;

        let first = exact_spirit_summary(&board, Color::White, Config::MONS_MOVES_PER_TURN, true);
        let second = exact_spirit_summary(&board, Color::White, Config::MONS_MOVES_PER_TURN, true);
        clear_exact_state_analysis_cache();
        let third = exact_spirit_summary(&board, Color::White, Config::MONS_MOVES_PER_TURN, true);

        assert_eq!(first.utility, second.utility);
        assert_eq!(first.same_turn_score, second.same_turn_score);
        assert_eq!(
            first.same_turn_opponent_mana_score,
            second.same_turn_opponent_mana_score
        );
        assert_eq!(first.next_turn_setup_gain, second.next_turn_setup_gain);
        assert_eq!(first.utility, third.utility);
        assert_eq!(first.same_turn_score_value, third.same_turn_score_value);
        assert_eq!(
            first.same_turn_opponent_mana_score_value,
            third.same_turn_opponent_mana_score_value
        );
    }

    #[test]
    fn exact_spirit_reach_cache_preserves_repeated_positions() {
        clear_exact_state_analysis_cache();
        let board = game_with_items(
            vec![
                (
                    Location::new(7, 1),
                    Item::Mon {
                        mon: Mon::new(MonKind::Spirit, Color::White, 0),
                    },
                ),
                (
                    Location::new(6, 1),
                    Item::Consumable {
                        consumable: Consumable::BombOrPotion,
                    },
                ),
                (
                    Location::new(0, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
        )
        .board;

        let first = reachable_spirit_positions(&board, Location::new(7, 1), Color::White, 3);
        let second = reachable_spirit_positions(&board, Location::new(7, 1), Color::White, 3);
        clear_exact_state_analysis_cache();
        let third = reachable_spirit_positions(&board, Location::new(7, 1), Color::White, 3);

        assert_eq!(first, second);
        assert_eq!(first, third);
    }

    #[test]
    fn exact_turn_summary_uses_spirit_denial_for_safe_opponent_progress() {
        clear_exact_state_analysis_cache();
        let game = game_with_items(
            vec![
                (
                    Location::new(7, 1),
                    Item::Mon {
                        mon: Mon::new(MonKind::Spirit, Color::White, 0),
                    },
                ),
                (
                    Location::new(9, 1),
                    Item::Mana {
                        mana: Mana::Regular(Color::Black),
                    },
                ),
            ],
            Color::White,
        );

        let turn = exact_turn_summary(&game, Color::White);
        assert!(turn.safe_opponent_mana_progress);
        assert!(turn.spirit_assisted_denial);
    }

    #[test]
    fn exact_turn_tactical_projection_matches_supermana_turn_summary_fields() {
        clear_exact_state_analysis_cache();
        let game = game_with_items(
            vec![
                (
                    Location::new(6, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::White, 0),
                    },
                ),
                (
                    Location::new(5, 5),
                    Item::Mana {
                        mana: Mana::Supermana,
                    },
                ),
                (
                    Location::new(0, 10),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
        );

        let turn = exact_turn_summary(&game, Color::White);
        let projection = exact_turn_tactical_projection(
            &game,
            Color::White,
            EXACT_TURN_TACTICAL_NEED_SUPERMANA_PROGRESS | EXACT_TURN_TACTICAL_NEED_SCORE_WINDOW,
        );

        assert_eq!(
            projection.safe_supermana_progress,
            turn.safe_supermana_progress
        );
        assert_eq!(
            projection.safe_supermana_progress_steps,
            turn.safe_supermana_progress_steps
        );
        assert_eq!(
            projection.same_turn_score_window_value,
            turn.same_turn_score_window_value
        );
        assert!(!projection.safe_opponent_mana_progress);
        assert_eq!(projection.safe_opponent_mana_progress_steps, None);
    }

    #[test]
    fn exact_turn_tactical_projection_skips_spirit_progress_queries_for_score_only_projection() {
        clear_exact_state_analysis_cache();

        let game = game_with_items(
            vec![
                (
                    Location::new(7, 1),
                    Item::Mon {
                        mon: Mon::new(MonKind::Spirit, Color::White, 0),
                    },
                ),
                (
                    Location::new(9, 1),
                    Item::Mana {
                        mana: Mana::Regular(Color::Black),
                    },
                ),
            ],
            Color::White,
        );
        let turn = exact_turn_summary(&game, Color::White);
        let remaining_moves = (Config::MONS_MOVES_PER_TURN - game.mons_moves_count).max(0);
        let expected_score_window =
            exact_best_immediate_score_on_board(&game.board, Color::White, remaining_moves)
                .max(turn.spirit_assisted_score_value);
        clear_exact_query_diagnostics();

        let projection = exact_turn_tactical_projection(
            &game,
            Color::White,
            EXACT_TURN_TACTICAL_NEED_SPIRIT_SCORE | EXACT_TURN_TACTICAL_NEED_SCORE_WINDOW,
        );
        let diagnostics = exact_query_diagnostics_snapshot();

        assert!(diagnostics.tactical_spirit_summary_calls > 0);
        assert_eq!(
            diagnostics.exact_secure_mana_calls, 0,
            "score-only tactical projection should not pay spirit progress secure-mana queries"
        );
        assert_eq!(projection.spirit_assisted_score, turn.spirit_assisted_score);
        assert_eq!(
            projection.spirit_assisted_score_value,
            turn.spirit_assisted_score_value
        );
        assert!(!projection.spirit_assisted_denial);
        assert_eq!(projection.spirit_assisted_denial_value, 0);
        assert_eq!(
            projection.same_turn_score_window_value,
            expected_score_window
        );
    }

    #[test]
    fn exact_turn_tactical_projection_matches_denial_only_spirit_fields() {
        clear_exact_state_analysis_cache();
        let game = game_with_items(
            vec![
                (
                    Location::new(7, 1),
                    Item::Mon {
                        mon: Mon::new(MonKind::Spirit, Color::White, 0),
                    },
                ),
                (
                    Location::new(9, 1),
                    Item::Mana {
                        mana: Mana::Regular(Color::Black),
                    },
                ),
            ],
            Color::White,
        );

        let turn = exact_turn_summary(&game, Color::White);
        let projection = exact_turn_tactical_projection(
            &game,
            Color::White,
            EXACT_TURN_TACTICAL_NEED_OPPONENT_MANA_PROGRESS
                | EXACT_TURN_TACTICAL_NEED_SPIRIT_DENIAL,
        );

        assert_eq!(
            projection.safe_opponent_mana_progress,
            turn.safe_opponent_mana_progress
        );
        assert_eq!(
            projection.safe_opponent_mana_progress_steps,
            turn.safe_opponent_mana_progress_steps
        );
        assert_eq!(
            projection.spirit_assisted_denial,
            turn.spirit_assisted_denial
        );
        assert_eq!(
            projection.spirit_assisted_denial_value,
            turn.spirit_assisted_denial_value
        );
        assert!(!projection.spirit_assisted_score);
        assert_eq!(projection.spirit_assisted_score_value, 0);
        assert_eq!(projection.same_turn_score_window_value, 0);
    }

    #[test]
    fn exact_turn_tactical_projection_matches_spirit_turn_summary_fields() {
        clear_exact_state_analysis_cache();
        let game = game_with_items(
            vec![
                (
                    Location::new(7, 1),
                    Item::Mon {
                        mon: Mon::new(MonKind::Spirit, Color::White, 0),
                    },
                ),
                (
                    Location::new(9, 1),
                    Item::Mana {
                        mana: Mana::Regular(Color::Black),
                    },
                ),
            ],
            Color::White,
        );

        let turn = exact_turn_summary(&game, Color::White);
        let projection = exact_turn_tactical_projection(
            &game,
            Color::White,
            EXACT_TURN_TACTICAL_NEED_OPPONENT_MANA_PROGRESS
                | EXACT_TURN_TACTICAL_NEED_SPIRIT_SCORE
                | EXACT_TURN_TACTICAL_NEED_SPIRIT_DENIAL
                | EXACT_TURN_TACTICAL_NEED_SCORE_WINDOW,
        );

        assert_eq!(
            projection.safe_opponent_mana_progress,
            turn.safe_opponent_mana_progress
        );
        assert_eq!(
            projection.safe_opponent_mana_progress_steps,
            turn.safe_opponent_mana_progress_steps
        );
        assert_eq!(
            projection.spirit_assisted_denial,
            turn.spirit_assisted_denial
        );
        assert_eq!(
            projection.spirit_assisted_denial_value,
            turn.spirit_assisted_denial_value
        );
        assert_eq!(projection.spirit_assisted_score, turn.spirit_assisted_score);
        assert_eq!(
            projection.spirit_assisted_score_value,
            turn.spirit_assisted_score_value
        );
        assert_eq!(
            projection.same_turn_score_window_value,
            turn.same_turn_score_window_value
        );
    }

    #[test]
    fn exact_state_analysis_uses_full_spirit_only_for_active_color_on_opening_black_turn() {
        clear_exact_state_analysis_cache();
        clear_exact_query_diagnostics();

        let mut game = MonsGame::new(false);
        for step in [
            "l10,3;l9,2",
            "l9,2;l8,1",
            "l8,1;l7,0",
            "l7,0;l6,0",
            "l6,0;l5,0;mp",
        ] {
            assert!(matches!(
                game.process_input(Input::array_from_fen(step), false, false),
                Output::Events(_)
            ));
        }
        assert_eq!(game.active_color, Color::Black);
        assert_eq!(game.turn_number, 2);

        let _ = exact_state_analysis(&game);
        let diagnostics = exact_query_diagnostics_snapshot();
        assert_eq!(diagnostics.active_tactical_summary_builds, 1);
        assert_eq!(diagnostics.passive_strategic_summary_builds, 1);
        assert_eq!(diagnostics.exact_spirit_summary_calls, 1);
        assert_eq!(diagnostics.passive_spirit_summary_calls, 1);
        assert!(
            diagnostics.exact_followup_summary_calls > 0,
            "active tactical summary should still use full exact followup analysis"
        );
    }

    #[test]
    fn exact_turn_summary_avoids_full_followup_on_opening_black_turn() {
        clear_exact_state_analysis_cache();
        clear_exact_query_diagnostics();

        let mut game = MonsGame::new(false);
        for step in [
            "l10,3;l9,2",
            "l9,2;l8,1",
            "l8,1;l7,0",
            "l7,0;l6,0",
            "l6,0;l5,0;mp",
        ] {
            assert!(matches!(
                game.process_input(Input::array_from_fen(step), false, false),
                Output::Events(_)
            ));
        }
        assert_eq!(game.active_color, Color::Black);
        assert_eq!(game.turn_number, 2);

        let _ = exact_turn_summary(&game, Color::Black);
        let diagnostics = exact_query_diagnostics_snapshot();
        assert_eq!(diagnostics.exact_turn_summary_builds, 1);
        assert_eq!(diagnostics.exact_spirit_summary_calls, 0);
        assert!(diagnostics.tactical_spirit_summary_calls > 0);
        assert_eq!(diagnostics.exact_followup_summary_calls, 0);
        assert_eq!(diagnostics.passive_spirit_summary_calls, 0);
    }

    #[test]
    fn exact_turn_summary_detects_same_turn_drainer_attack() {
        let game = game_with_items(
            vec![
                (
                    Location::new(5, 5),
                    Item::Mon {
                        mon: Mon::new(MonKind::Mystic, Color::White, 0),
                    },
                ),
                (
                    Location::new(7, 7),
                    Item::Mon {
                        mon: Mon::new(MonKind::Drainer, Color::Black, 0),
                    },
                ),
            ],
            Color::White,
        );
        assert!(exact_turn_summary(&game, Color::White).can_attack_opponent_drainer);
    }
}
