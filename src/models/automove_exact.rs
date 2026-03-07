use crate::*;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::Instant;

const EXACT_ANALYSIS_CACHE_MAX_ENTRIES: usize = 512;
const EXACT_TURN_QUERY_CACHE_MAX_ENTRIES: usize = 1024;
const EXACT_ATTACK_REACH_CACHE_MAX_ENTRIES: usize = 8192;
const EXACT_CARRIER_STEPS_CACHE_MAX_ENTRIES: usize = 8192;
const EXACT_DRAINER_TO_MANA_CACHE_MAX_ENTRIES: usize = 8192;
const EXACT_FOLLOWUP_SUMMARY_CACHE_MAX_ENTRIES: usize = 4096;
const EXACT_PICKUP_PATH_CACHE_MAX_ENTRIES: usize = 8192;
const EXACT_SPIRIT_REACH_CACHE_MAX_ENTRIES: usize = 4096;
const EXACT_SPIRIT_SUMMARY_CACHE_MAX_ENTRIES: usize = 2048;
const EXACT_WALK_THREAT_CACHE_MAX_ENTRIES: usize = 8192;
const EXACT_SECURE_MANA_CACHE_MAX_ENTRIES: usize = 4096;
const EXACT_SPIRIT_UTILITY_CAP: i32 = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ExactActorPayload {
    None,
    Mana(Mana),
    Bomb,
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
    pub mana: Mana,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ExactSpiritSummary {
    pub utility: i32,
    pub same_turn_score: bool,
    pub same_turn_score_value: i32,
    pub same_turn_opponent_mana_score: bool,
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
    pub best_carrier_steps: Option<i32>,
    pub best_drainer_to_mana_steps: Option<i32>,
    pub spirit: ExactSpiritSummary,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ExactSecureManaQuery {
    pub steps: Option<i32>,
    pub safe_landing: Option<Location>,
    pub scores_this_turn: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ExactTurnQueryBundle {
    pub color: Option<Color>,
    pub can_attack_opponent_drainer: bool,
    pub supermana: ExactSecureManaQuery,
    pub opponent_mana: ExactSecureManaQuery,
    pub spirit: ExactSpiritSummary,
}

impl ExactTurnQueryBundle {
    #[inline]
    pub(crate) fn summary(self) -> ExactTurnSummary {
        ExactTurnSummary {
            color: self.color,
            can_attack_opponent_drainer: self.can_attack_opponent_drainer,
            safe_supermana_progress: self.supermana.steps.is_some(),
            safe_supermana_progress_steps: self.supermana.steps,
            safe_supermana_landing: self.supermana.safe_landing,
            safe_opponent_mana_progress: self.opponent_mana.steps.is_some()
                || self.spirit.same_turn_opponent_mana_score,
            safe_opponent_mana_progress_steps: self.opponent_mana.steps,
            safe_opponent_mana_landing: self.opponent_mana.safe_landing,
            spirit_assisted_supermana_progress: self.spirit.supermana_progress,
            spirit_assisted_opponent_mana_progress: self.spirit.opponent_mana_progress,
            spirit_assisted_score: self.spirit.same_turn_score,
            spirit_assisted_denial: self.spirit.same_turn_opponent_mana_score,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ExactTurnSummary {
    #[allow(dead_code)]
    pub color: Option<Color>,
    pub can_attack_opponent_drainer: bool,
    pub safe_supermana_progress: bool,
    pub safe_supermana_progress_steps: Option<i32>,
    pub safe_supermana_landing: Option<Location>,
    pub safe_opponent_mana_progress: bool,
    pub safe_opponent_mana_progress_steps: Option<i32>,
    pub safe_opponent_mana_landing: Option<Location>,
    pub spirit_assisted_supermana_progress: bool,
    pub spirit_assisted_opponent_mana_progress: bool,
    pub spirit_assisted_score: bool,
    pub spirit_assisted_denial: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ExactQueryFamily {
    AttackReach,
    DrainerPickup,
    SecureMana,
    SafeLanding,
    SpiritAssist,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ExactQueryFamilyDiagnostics {
    pub queries: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub elapsed_ns: u64,
}

impl ExactQueryFamilyDiagnostics {
    fn record(&mut self, cache_hit: Option<bool>, elapsed_ns: u64) {
        self.queries += 1;
        if let Some(cache_hit) = cache_hit {
            if cache_hit {
                self.cache_hits += 1;
            } else {
                self.cache_misses += 1;
            }
        }
        self.elapsed_ns = self.elapsed_ns.saturating_add(elapsed_ns);
    }

    fn merge(&mut self, other: Self) {
        self.queries += other.queries;
        self.cache_hits += other.cache_hits;
        self.cache_misses += other.cache_misses;
        self.elapsed_ns = self.elapsed_ns.saturating_add(other.elapsed_ns);
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ExactQueryDiagnosticsSnapshot {
    pub attack_reach: ExactQueryFamilyDiagnostics,
    pub drainer_pickup: ExactQueryFamilyDiagnostics,
    pub secure_mana: ExactQueryFamilyDiagnostics,
    pub safe_landing: ExactQueryFamilyDiagnostics,
    pub spirit_assist: ExactQueryFamilyDiagnostics,
}

impl ExactQueryDiagnosticsSnapshot {
    fn family_mut(&mut self, family: ExactQueryFamily) -> &mut ExactQueryFamilyDiagnostics {
        match family {
            ExactQueryFamily::AttackReach => &mut self.attack_reach,
            ExactQueryFamily::DrainerPickup => &mut self.drainer_pickup,
            ExactQueryFamily::SecureMana => &mut self.secure_mana,
            ExactQueryFamily::SafeLanding => &mut self.safe_landing,
            ExactQueryFamily::SpiritAssist => &mut self.spirit_assist,
        }
    }

    pub(crate) fn merge(&mut self, other: Self) {
        self.attack_reach.merge(other.attack_reach);
        self.drainer_pickup.merge(other.drainer_pickup);
        self.secure_mana.merge(other.secure_mana);
        self.safe_landing.merge(other.safe_landing);
        self.spirit_assist.merge(other.spirit_assist);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ExactPickupFilter {
    Any,
    Wanted(Mana),
}

impl ExactPickupFilter {
    #[inline]
    fn matches(self, mana: Mana) -> bool {
        match self {
            ExactPickupFilter::Any => true,
            ExactPickupFilter::Wanted(wanted) => mana == wanted,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct ExactFollowupSummary {
    best_score_steps: Option<i32>,
    opponent_best_score_steps: Option<i32>,
    immediate_score: i32,
    immediate_opponent_mana_score: i32,
    secure_supermana: bool,
    secure_opponent_mana: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ExactStateAnalysis {
    pub white: ExactColorSummary,
    pub black: ExactColorSummary,
    #[allow(dead_code)]
    pub active_turn_queries: ExactTurnQueryBundle,
    #[allow(dead_code)]
    pub active_turn: ExactTurnSummary,
}

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

pub(crate) struct ExactStateAnalysisCache {
    entries: HashMap<u64, ExactStateAnalysis>,
}

struct ExactTurnQueryCache {
    entries: HashMap<u64, ExactTurnQueryBundle>,
}

impl Default for ExactStateAnalysisCache {
    fn default() -> Self {
        Self {
            entries: HashMap::with_capacity(EXACT_ANALYSIS_CACHE_MAX_ENTRIES),
        }
    }
}

impl Default for ExactTurnQueryCache {
    fn default() -> Self {
        Self {
            entries: HashMap::with_capacity(EXACT_TURN_QUERY_CACHE_MAX_ENTRIES),
        }
    }
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

struct ExactAttackReachCache {
    entries: HashMap<ExactAttackQueryKey, bool>,
}

impl Default for ExactAttackReachCache {
    fn default() -> Self {
        Self {
            entries: HashMap::with_capacity(EXACT_ATTACK_REACH_CACHE_MAX_ENTRIES),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ExactCarrierStepsQueryKey {
    board_hash: u64,
    start: Location,
    mana: Mana,
}

struct ExactCarrierStepsCache {
    entries: HashMap<ExactCarrierStepsQueryKey, Option<i32>>,
}

impl Default for ExactCarrierStepsCache {
    fn default() -> Self {
        Self {
            entries: HashMap::with_capacity(EXACT_CARRIER_STEPS_CACHE_MAX_ENTRIES),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ExactDrainerToManaQueryKey {
    board_hash: u64,
    color: Color,
    start: Location,
}

struct ExactDrainerToManaCache {
    entries: HashMap<ExactDrainerToManaQueryKey, Option<i32>>,
}

impl Default for ExactDrainerToManaCache {
    fn default() -> Self {
        Self {
            entries: HashMap::with_capacity(EXACT_DRAINER_TO_MANA_CACHE_MAX_ENTRIES),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ExactFollowupSummaryKey {
    board_hash: u64,
    color: Color,
    remaining_moves: i32,
}

struct ExactFollowupSummaryCache {
    entries: HashMap<ExactFollowupSummaryKey, ExactFollowupSummary>,
}

impl Default for ExactFollowupSummaryCache {
    fn default() -> Self {
        Self {
            entries: HashMap::with_capacity(EXACT_FOLLOWUP_SUMMARY_CACHE_MAX_ENTRIES),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ExactPickupPathQueryKey {
    board_hash: u64,
    color: Color,
    start: Location,
    max_steps: Option<i32>,
    filter: ExactPickupFilter,
}

struct ExactPickupPathCache {
    entries: HashMap<ExactPickupPathQueryKey, Option<ExactDrainerPickupPath>>,
}

impl Default for ExactPickupPathCache {
    fn default() -> Self {
        Self {
            entries: HashMap::with_capacity(EXACT_PICKUP_PATH_CACHE_MAX_ENTRIES),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ExactSpiritSummaryKey {
    board_hash: u64,
    color: Color,
    remaining_mon_moves: i32,
    can_use_action: bool,
}

struct ExactSpiritSummaryCache {
    entries: HashMap<ExactSpiritSummaryKey, ExactSpiritSummary>,
}

impl Default for ExactSpiritSummaryCache {
    fn default() -> Self {
        Self {
            entries: HashMap::with_capacity(EXACT_SPIRIT_SUMMARY_CACHE_MAX_ENTRIES),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ExactSpiritReachQueryKey {
    board_hash: u64,
    start: Location,
    color: Color,
    remaining_mon_moves: i32,
}

struct ExactSpiritReachCache {
    entries: HashMap<ExactSpiritReachQueryKey, Vec<(Location, i32)>>,
}

impl Default for ExactSpiritReachCache {
    fn default() -> Self {
        Self {
            entries: HashMap::with_capacity(EXACT_SPIRIT_REACH_CACHE_MAX_ENTRIES),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ExactWalkThreatQueryKey {
    board_hash: u64,
    color: Color,
    location: Location,
    angel_nearby: bool,
}

struct ExactWalkThreatCache {
    entries: HashMap<ExactWalkThreatQueryKey, bool>,
}

impl Default for ExactWalkThreatCache {
    fn default() -> Self {
        Self {
            entries: HashMap::with_capacity(EXACT_WALK_THREAT_CACHE_MAX_ENTRIES),
        }
    }
}

struct ExactSecureManaCache {
    entries: HashMap<ExactSecureManaQueryKey, ExactSecureManaQuery>,
}

impl Default for ExactSecureManaCache {
    fn default() -> Self {
        Self {
            entries: HashMap::with_capacity(EXACT_SECURE_MANA_CACHE_MAX_ENTRIES),
        }
    }
}

#[derive(Default)]
struct ExactQueryDiagnostics {
    snapshot: ExactQueryDiagnosticsSnapshot,
    enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ExactSecureManaQueryKey {
    board_hash: u64,
    color: Color,
    wanted: Mana,
    remaining_moves: i32,
}

thread_local! {
    static EXACT_STATE_ANALYSIS_CACHE: RefCell<ExactStateAnalysisCache> =
        RefCell::new(ExactStateAnalysisCache::default());
    static EXACT_TURN_QUERY_CACHE: RefCell<ExactTurnQueryCache> =
        RefCell::new(ExactTurnQueryCache::default());
    static EXACT_ATTACK_REACH_CACHE: RefCell<ExactAttackReachCache> =
        RefCell::new(ExactAttackReachCache::default());
    static EXACT_CARRIER_STEPS_CACHE: RefCell<ExactCarrierStepsCache> =
        RefCell::new(ExactCarrierStepsCache::default());
    static EXACT_DRAINER_TO_MANA_CACHE: RefCell<ExactDrainerToManaCache> =
        RefCell::new(ExactDrainerToManaCache::default());
    static EXACT_FOLLOWUP_SUMMARY_CACHE: RefCell<ExactFollowupSummaryCache> =
        RefCell::new(ExactFollowupSummaryCache::default());
    static EXACT_PICKUP_PATH_CACHE: RefCell<ExactPickupPathCache> =
        RefCell::new(ExactPickupPathCache::default());
    static EXACT_SPIRIT_REACH_CACHE: RefCell<ExactSpiritReachCache> =
        RefCell::new(ExactSpiritReachCache::default());
    static EXACT_SPIRIT_SUMMARY_CACHE: RefCell<ExactSpiritSummaryCache> =
        RefCell::new(ExactSpiritSummaryCache::default());
    static EXACT_WALK_THREAT_CACHE: RefCell<ExactWalkThreatCache> =
        RefCell::new(ExactWalkThreatCache::default());
    static EXACT_SECURE_MANA_CACHE: RefCell<ExactSecureManaCache> =
        RefCell::new(ExactSecureManaCache::default());
    static EXACT_QUERY_DIAGNOSTICS: RefCell<ExactQueryDiagnostics> =
        RefCell::new(ExactQueryDiagnostics::default());
}

#[inline]
pub(crate) fn clear_exact_state_analysis_cache() {
    EXACT_STATE_ANALYSIS_CACHE.with(|cache| cache.borrow_mut().entries.clear());
    EXACT_TURN_QUERY_CACHE.with(|cache| cache.borrow_mut().entries.clear());
    EXACT_ATTACK_REACH_CACHE.with(|cache| cache.borrow_mut().entries.clear());
    EXACT_CARRIER_STEPS_CACHE.with(|cache| cache.borrow_mut().entries.clear());
    EXACT_DRAINER_TO_MANA_CACHE.with(|cache| cache.borrow_mut().entries.clear());
    EXACT_FOLLOWUP_SUMMARY_CACHE.with(|cache| cache.borrow_mut().entries.clear());
    EXACT_PICKUP_PATH_CACHE.with(|cache| cache.borrow_mut().entries.clear());
    EXACT_SPIRIT_REACH_CACHE.with(|cache| cache.borrow_mut().entries.clear());
    EXACT_SPIRIT_SUMMARY_CACHE.with(|cache| cache.borrow_mut().entries.clear());
    EXACT_WALK_THREAT_CACHE.with(|cache| cache.borrow_mut().entries.clear());
    EXACT_SECURE_MANA_CACHE.with(|cache| cache.borrow_mut().entries.clear());
}

#[inline]
pub(crate) fn reset_exact_query_diagnostics() {
    EXACT_QUERY_DIAGNOSTICS.with(|diagnostics| {
        diagnostics.borrow_mut().snapshot = ExactQueryDiagnosticsSnapshot::default();
    });
}

#[inline]
pub(crate) fn set_exact_query_diagnostics_enabled(enabled: bool) {
    EXACT_QUERY_DIAGNOSTICS.with(|diagnostics| diagnostics.borrow_mut().enabled = enabled);
}

#[inline]
pub(crate) fn exact_query_diagnostics_snapshot() -> ExactQueryDiagnosticsSnapshot {
    EXACT_QUERY_DIAGNOSTICS.with(|diagnostics| diagnostics.borrow().snapshot)
}

#[inline]
fn record_exact_query_diagnostics(
    family: ExactQueryFamily,
    cache_hit: Option<bool>,
    elapsed_ns: u64,
) {
    EXACT_QUERY_DIAGNOSTICS.with(|diagnostics| {
        let mut diagnostics = diagnostics.borrow_mut();
        if !diagnostics.enabled {
            return;
        }
        diagnostics
            .snapshot
            .family_mut(family)
            .record(cache_hit, elapsed_ns);
    });
}

pub(crate) fn exact_state_analysis(game: &MonsGame) -> ExactStateAnalysis {
    let key = MonsGameModel::search_state_hash(game);
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

#[inline]
pub(crate) fn exact_turn_summary(game: &MonsGame, color: Color) -> ExactTurnSummary {
    if game.active_color == color {
        cached_active_turn_query_bundle(game).summary()
    } else {
        ExactTurnSummary {
            color: Some(color),
            ..ExactTurnSummary::default()
        }
    }
}

#[inline]
pub(crate) fn exact_turn_query_bundle(game: &MonsGame, color: Color) -> ExactTurnQueryBundle {
    if game.active_color == color {
        cached_active_turn_query_bundle(game)
    } else {
        ExactTurnQueryBundle {
            color: Some(color),
            ..ExactTurnQueryBundle::default()
        }
    }
}

fn cached_active_turn_query_bundle(game: &MonsGame) -> ExactTurnQueryBundle {
    let key = MonsGameModel::search_state_hash(game);
    if let Some(cached) =
        EXACT_TURN_QUERY_CACHE.with(|cache| cache.borrow().entries.get(&key).copied())
    {
        return cached;
    }

    let built = build_turn_query_bundle(game);
    EXACT_TURN_QUERY_CACHE.with(|turn_cache| {
        let mut turn_cache = turn_cache.borrow_mut();
        if turn_cache.entries.len() >= EXACT_TURN_QUERY_CACHE_MAX_ENTRIES
            && !turn_cache.entries.contains_key(&key)
        {
            turn_cache.entries.clear();
        }
        turn_cache.entries.insert(key, built);
    });
    built
}

pub(crate) fn can_attack_opponent_drainer_this_turn(game: &MonsGame, color: Color) -> bool {
    exact_turn_summary(game, color).can_attack_opponent_drainer
}

pub(crate) fn can_attack_target_on_board(
    board: &Board,
    attacker_color: Color,
    target_color: Color,
    target: Location,
    remaining_moves: i32,
    can_use_action: bool,
) -> bool {
    let started = Instant::now();
    if remaining_moves < 0 || !can_use_action || board.item(target).is_none() {
        record_exact_query_diagnostics(
            ExactQueryFamily::AttackReach,
            Some(false),
            started.elapsed().as_nanos() as u64,
        );
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
        record_exact_query_diagnostics(
            ExactQueryFamily::AttackReach,
            Some(true),
            started.elapsed().as_nanos() as u64,
        );
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
    record_exact_query_diagnostics(
        ExactQueryFamily::AttackReach,
        Some(false),
        started.elapsed().as_nanos() as u64,
    );
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
    let target_guarded = MonsGameModel::is_location_guarded_by_angel(board, target_color, target);

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
        let mut queue = VecDeque::new();
        let mut seen = HashSet::new();
        queue.push_back((start, start_payload, 0));
        seen.insert((start, start_payload));

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
                    if seen.insert((next, next_payload)) {
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

pub(crate) fn is_drainer_exactly_safe_next_turn_on_board(
    board: &Board,
    color: Color,
    location: Location,
) -> bool {
    let started = Instant::now();
    let angel_nearby = MonsGameModel::is_location_guarded_by_angel(board, color, location);
    let result = !can_attack_target_on_board(
        board,
        color.other(),
        color,
        location,
        Config::MONS_MOVES_PER_TURN,
        true,
    ) && !is_drainer_under_walk_threat(board, color, location, angel_nearby);
    record_exact_query_diagnostics(
        ExactQueryFamily::SafeLanding,
        None,
        started.elapsed().as_nanos() as u64,
    );
    result
}

fn build_exact_state_analysis(game: &MonsGame) -> ExactStateAnalysis {
    let white = build_color_summary(game, Color::White);
    let black = build_color_summary(game, Color::Black);
    let active_turn_queries = ExactTurnQueryBundle {
        color: Some(game.active_color),
        ..ExactTurnQueryBundle::default()
    };
    let active_turn = ExactTurnSummary {
        color: Some(game.active_color),
        ..ExactTurnSummary::default()
    };
    ExactStateAnalysis {
        white,
        black,
        active_turn_queries,
        active_turn,
    }
}

fn build_color_summary(game: &MonsGame, color: Color) -> ExactColorSummary {
    let full_turn_moves = if game.active_color == color {
        (Config::MONS_MOVES_PER_TURN - game.mons_moves_count).max(0)
    } else {
        Config::MONS_MOVES_PER_TURN
    };
    let can_use_action = if game.active_color == color {
        game.player_can_use_action()
    } else {
        true
    };

    let mut carrier_steps = Vec::new();
    let mut best_carrier_steps = None;
    for (location, item) in game.board.occupied() {
        let Item::MonWithMana { mon, mana } = item else {
            continue;
        };
        if mon.color != color || mon.is_fainted() {
            continue;
        }
        if let Some(steps) = exact_carrier_steps_to_any_pool(&game.board, location, *mana) {
            best_carrier_steps =
                Some(best_carrier_steps.map_or(steps, |best: i32| best.min(steps)));
            carrier_steps.push(steps);
        }
    }

    let best_drainer_pickup = find_awake_drainer(&game.board, color)
        .and_then(|location| exact_best_drainer_pickup_path(&game.board, color, location));
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
        if let Some(steps) = exact_carrier_steps_to_any_pool(&game.board, location, *mana) {
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
    let spirit = if game.active_color == color {
        exact_spirit_summary(&game.board, color, full_turn_moves, can_use_action)
    } else {
        approximate_nonactive_spirit_summary(&game.board, color)
    };
    if spirit.same_turn_score {
        immediate_scores.push(spirit.same_turn_score_value.max(1));
    }
    if spirit.same_turn_opponent_mana_score {
        immediate_scores.push(spirit.same_turn_opponent_mana_score_value.max(1));
    }
    immediate_scores.sort_unstable_by(|a, b| b.cmp(a));
    let immediate_window = ExactImmediateScoreWindow {
        best_score: immediate_scores.first().copied().unwrap_or(0),
        multi_pressure: exact_multi_pressure_from_scores(immediate_scores.as_slice()),
    };

    ExactColorSummary {
        score_path_window,
        immediate_window,
        best_drainer_pickup,
        best_carrier_steps,
        best_drainer_to_mana_steps,
        spirit,
    }
}

fn approximate_nonactive_spirit_summary(board: &Board, color: Color) -> ExactSpiritSummary {
    let mut best = ExactSpiritSummary::default();

    for (location, item) in board.occupied() {
        let Some(mon) = item.mon() else {
            continue;
        };
        if mon.color != color || mon.kind != MonKind::Spirit || mon.is_fainted() {
            continue;
        }

        let mut actionable_targets = 0i32;
        let mut pressure = 0i32;
        for &target in location.reachable_by_spirit_action_ref() {
            let Some(target_item) = board.item(target).copied() else {
                continue;
            };
            if !spirit_target_allowed(target_item) {
                continue;
            }

            let has_destination = target
                .nearby_locations_ref()
                .iter()
                .copied()
                .any(|dest| spirit_destination_allowed(board, target, target_item, dest));
            if !has_destination {
                continue;
            }

            actionable_targets += 1;
            pressure = pressure.max(match target_item {
                Item::Mana {
                    mana: Mana::Regular(owner),
                } if owner == color.other() => 3,
                Item::Mana {
                    mana: Mana::Supermana,
                } => 2,
                Item::MonWithMana { mon, mana } if mon.color == color => {
                    if mana == Mana::Regular(color.other()) {
                        3
                    } else {
                        2
                    }
                }
                Item::Mon { mon }
                | Item::MonWithMana { mon, .. }
                | Item::MonWithConsumable { mon, .. }
                    if mon.color != color =>
                {
                    2
                }
                _ => 1,
            });
        }

        let utility = actionable_targets.min(EXACT_SPIRIT_UTILITY_CAP).max(pressure);
        if utility > best.utility {
            best.utility = utility;
            best.next_turn_setup_gain = pressure.min(4);
        } else if utility == best.utility {
            best.next_turn_setup_gain = best.next_turn_setup_gain.max(pressure.min(4));
        }
    }

    best
}

fn build_turn_query_bundle(game: &MonsGame) -> ExactTurnQueryBundle {
    let color = game.active_color;
    let remaining_mon_moves = (Config::MONS_MOVES_PER_TURN - game.mons_moves_count).max(0);
    let can_use_action = game.player_can_use_action();
    let spirit = exact_spirit_summary(&game.board, color, remaining_mon_moves, can_use_action);
    ExactTurnQueryBundle {
        color: Some(color),
        can_attack_opponent_drainer: can_attack_opponent_drainer_exact(game, color),
        supermana: exact_secure_specific_mana_query_this_turn(game, color, Mana::Supermana),
        opponent_mana: exact_secure_specific_mana_query_this_turn(
            game,
            color,
            Mana::Regular(color.other()),
        ),
        spirit,
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
    let mut queue = VecDeque::with_capacity(64);
    let mut seen = HashSet::with_capacity(128);
    queue.push_back((start, start_payload, 0));
    seen.insert((start, start_payload));

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
                if seen.insert((next, next_payload)) {
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
    let item = board.item(destination).copied();
    let square = board.square(destination);
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

fn exact_carrier_steps_to_any_pool(board: &Board, start: Location, mana: Mana) -> Option<i32> {
    let key = ExactCarrierStepsQueryKey {
        board_hash: exact_board_hash(board),
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

fn exact_best_drainer_pickup_path(
    board: &Board,
    color: Color,
    start: Location,
) -> Option<ExactDrainerPickupPath> {
    exact_best_drainer_pickup_path_filtered(board, color, start, None, ExactPickupFilter::Any)
}

fn exact_best_drainer_pickup_path_filtered(
    board: &Board,
    color: Color,
    start: Location,
    max_steps: Option<i32>,
    mana_filter: ExactPickupFilter,
) -> Option<ExactDrainerPickupPath> {
    let started = Instant::now();
    let key = ExactPickupPathQueryKey {
        board_hash: exact_board_hash(board),
        color,
        start,
        max_steps,
        filter: mana_filter,
    };
    if let Some(cached) =
        EXACT_PICKUP_PATH_CACHE.with(|cache| cache.borrow().entries.get(&key).copied())
    {
        record_exact_query_diagnostics(
            ExactQueryFamily::DrainerPickup,
            Some(true),
            started.elapsed().as_nanos() as u64,
        );
        return cached;
    }

    let mut queue = VecDeque::with_capacity(64);
    let mut seen = HashSet::with_capacity(128);
    let start_state = (start, ExactActorPayload::None, 0);
    queue.push_back(start_state);
    seen.insert((start, ExactActorPayload::None));
    let mut best: Option<ExactDrainerPickupPath> = None;

    while let Some((location, payload, steps)) = queue.pop_front() {
        if max_steps.map_or(false, |limit| steps > limit) {
            continue;
        }
        if let ExactActorPayload::Mana(mana) = payload {
            if mana_filter.matches(mana)
                && matches!(board.square(location), Square::ManaPool { .. })
            {
                let total_moves: i32 = steps;
                let candidate = ExactDrainerPickupPath {
                    path_steps: total_moves.saturating_sub(1),
                    total_moves,
                    mana_value: mana.score(color),
                    mana,
                };
                let replace = match best {
                    None => true,
                    Some(current) => {
                        let candidate_metric = candidate.path_steps * 3 - candidate.mana_value;
                        let current_metric = current.path_steps * 3 - current.mana_value;
                        candidate_metric < current_metric
                            || (candidate_metric == current_metric
                                && candidate.mana_value > current.mana_value)
                    }
                };
                if replace {
                    best = Some(candidate);
                }
            }
        }

        for &next in location.nearby_locations_ref() {
            if let Some(next_payload) =
                actor_payload_after_move(board, MonKind::Drainer, color, payload, next, false)
            {
                if seen.insert((next, next_payload)) {
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
    record_exact_query_diagnostics(
        ExactQueryFamily::DrainerPickup,
        Some(false),
        started.elapsed().as_nanos() as u64,
    );
    best
}

fn find_awake_drainer(board: &Board, color: Color) -> Option<Location> {
    board.occupied().find_map(|(location, item)| {
        let mon = item.mon()?;
        (mon.color == color && mon.kind == MonKind::Drainer && !mon.is_fainted())
            .then_some(location)
    })
}

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

#[derive(Debug, Clone)]
struct ExactSecureState {
    board: Board,
    drainer_location: Location,
    steps: i32,
    previous_state: Option<usize>,
    via_location: Option<Location>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ExactSecureStateKey {
    board_hash: u64,
    drainer_location: Location,
}

#[derive(Debug, Clone, Copy)]
struct ExactSecureGoal {
    steps: i32,
    state_index: usize,
    tail_location: Option<Location>,
    safe_landing: Option<Location>,
    scores_this_turn: bool,
}

#[derive(Debug, Clone)]
struct ExactSecureSearchOutcome {
    query: ExactSecureManaQuery,
    path: Option<Vec<Location>>,
}

#[derive(Debug, Clone)]
struct ExactDrainerMovePreview {
    board: Board,
    drainer_location: Location,
    scored_mana: Option<Mana>,
}

fn exact_secure_specific_mana_query_this_turn(
    game: &MonsGame,
    color: Color,
    wanted: Mana,
) -> ExactSecureManaQuery {
    let remaining_moves = if game.active_color == color {
        (Config::MONS_MOVES_PER_TURN - game.mons_moves_count).max(0)
    } else {
        Config::MONS_MOVES_PER_TURN
    };
    exact_secure_specific_mana_query_on_board(&game.board, color, wanted, remaining_moves)
}

fn can_secure_specific_mana_on_board(
    board: &Board,
    color: Color,
    wanted: Mana,
    remaining_moves: i32,
) -> bool {
    exact_secure_specific_mana_query_on_board(board, color, wanted, remaining_moves)
        .steps
        .is_some()
}

fn exact_secure_specific_mana_steps_on_board(
    board: &Board,
    color: Color,
    wanted: Mana,
    remaining_moves: i32,
) -> Option<i32> {
    exact_secure_specific_mana_query_on_board(board, color, wanted, remaining_moves).steps
}

fn exact_secure_specific_mana_query_on_board(
    board: &Board,
    color: Color,
    wanted: Mana,
    remaining_moves: i32,
) -> ExactSecureManaQuery {
    if remaining_moves < 0 {
        return ExactSecureManaQuery::default();
    }

    let started = Instant::now();
    let key = ExactSecureManaQueryKey {
        board_hash: exact_board_hash(board),
        color,
        wanted,
        remaining_moves,
    };
    if let Some(cached) =
        EXACT_SECURE_MANA_CACHE.with(|cache| cache.borrow().entries.get(&key).copied())
    {
        record_exact_query_diagnostics(
            ExactQueryFamily::SecureMana,
            Some(true),
            started.elapsed().as_nanos() as u64,
        );
        return cached;
    }

    let result = exact_secure_specific_mana_search(board, color, wanted, remaining_moves, false)
        .query;
    EXACT_SECURE_MANA_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if cache.entries.len() >= EXACT_SECURE_MANA_CACHE_MAX_ENTRIES
            && !cache.entries.contains_key(&key)
        {
            cache.entries.clear();
        }
        cache.entries.insert(key, result);
    });
    record_exact_query_diagnostics(
        ExactQueryFamily::SecureMana,
        Some(false),
        started.elapsed().as_nanos() as u64,
    );
    result
}

fn exact_secure_specific_mana_search(
    board: &Board,
    color: Color,
    wanted: Mana,
    remaining_moves: i32,
    want_path: bool,
) -> ExactSecureSearchOutcome {
    if remaining_moves < 0 {
        return ExactSecureSearchOutcome {
            query: ExactSecureManaQuery::default(),
            path: None,
        };
    }

    let Some(drainer_location) = find_awake_drainer(board, color) else {
        return ExactSecureSearchOutcome {
            query: ExactSecureManaQuery::default(),
            path: None,
        };
    };

    let initial_state = ExactSecureState {
        board: board.clone(),
        drainer_location,
        steps: 0,
        previous_state: None,
        via_location: None,
    };
    let mut states = Vec::with_capacity(64);
    states.push(initial_state);
    let mut queue = VecDeque::with_capacity(64);
    queue.push_back(0usize);
    let mut seen = HashSet::with_capacity(128);
    seen.insert(ExactSecureStateKey {
        board_hash: exact_board_hash(board),
        drainer_location,
    });
    let mut best_goal: Option<ExactSecureGoal> = None;

    while let Some(state_index) = queue.pop_front() {
        let state_steps = states[state_index].steps;
        let state_drainer_location = states[state_index].drainer_location;
        if best_goal.map_or(false, |goal| state_steps > goal.steps) {
            break;
        }

        if exact_state_has_safe_carrier(
            &states[state_index].board,
            color,
            state_drainer_location,
            wanted,
        ) {
            let candidate = ExactSecureGoal {
                steps: state_steps,
                state_index,
                tail_location: None,
                safe_landing: Some(state_drainer_location),
                scores_this_turn: false,
            };
            if exact_secure_goal_better(candidate, best_goal) {
                best_goal = Some(candidate);
            }
        }

        if state_steps >= remaining_moves
            || best_goal.map_or(false, |goal| state_steps.saturating_add(1) > goal.steps)
        {
            continue;
        }

        for &next in state_drainer_location.nearby_locations_ref() {
            let Some(preview) = apply_exact_drainer_move_preview(
                &states[state_index].board,
                color,
                state_drainer_location,
                next,
            ) else {
                continue;
            };

            let next_steps = state_steps.saturating_add(1);
            if preview.scored_mana == Some(wanted) {
                let candidate = ExactSecureGoal {
                    steps: next_steps,
                    state_index,
                    tail_location: Some(next),
                    safe_landing: None,
                    scores_this_turn: true,
                };
                if exact_secure_goal_better(candidate, best_goal) {
                    best_goal = Some(candidate);
                }
                continue;
            }

            let key = ExactSecureStateKey {
                board_hash: exact_board_hash(&preview.board),
                drainer_location: preview.drainer_location,
            };
            if !seen.insert(key) {
                continue;
            }

            states.push(ExactSecureState {
                board: preview.board,
                drainer_location: preview.drainer_location,
                steps: next_steps,
                previous_state: Some(state_index),
                via_location: Some(next),
            });
            queue.push_back(states.len() - 1);
        }
    }

    let Some(goal) = best_goal else {
        return ExactSecureSearchOutcome {
            query: ExactSecureManaQuery::default(),
            path: None,
        };
    };

    ExactSecureSearchOutcome {
        query: ExactSecureManaQuery {
            steps: Some(goal.steps),
            safe_landing: goal.safe_landing,
            scores_this_turn: goal.scores_this_turn,
        },
        path: want_path.then(|| reconstruct_exact_secure_path(states.as_slice(), goal)),
    }
}

fn exact_secure_goal_better(
    candidate: ExactSecureGoal,
    current: Option<ExactSecureGoal>,
) -> bool {
    current.map_or(true, |current| {
        candidate.steps < current.steps
            || (candidate.steps == current.steps
                && candidate.scores_this_turn
                && !current.scores_this_turn)
    })
}

fn reconstruct_exact_secure_path(
    states: &[ExactSecureState],
    goal: ExactSecureGoal,
) -> Vec<Location> {
    let mut path = Vec::with_capacity(goal.steps.max(0) as usize);
    let mut cursor = Some(goal.state_index);
    while let Some(index) = cursor {
        let state = &states[index];
        if let Some(location) = state.via_location {
            path.push(location);
        }
        cursor = state.previous_state;
    }
    path.reverse();
    if let Some(tail_location) = goal.tail_location {
        path.push(tail_location);
    }
    path
}

fn exact_state_has_safe_carrier(
    board: &Board,
    color: Color,
    drainer_location: Location,
    wanted: Mana,
) -> bool {
    matches!(
        board.item(drainer_location),
        Some(Item::MonWithMana { mana, .. }) if *mana == wanted
    ) && is_drainer_exactly_safe_next_turn_on_board(board, color, drainer_location)
}

fn apply_exact_drainer_move_preview(
    board: &Board,
    color: Color,
    from: Location,
    to: Location,
) -> Option<ExactDrainerMovePreview> {
    let start_item = board.item(from).copied()?;
    let start_mon = match start_item {
        Item::Mon { mon } | Item::MonWithMana { mon, .. } => mon,
        Item::Mana { .. } | Item::MonWithConsumable { .. } | Item::Consumable { .. } => {
            return None
        }
    };
    if start_mon.color != color || start_mon.kind != MonKind::Drainer || start_mon.is_fainted() {
        return None;
    }

    let destination_item = board.item(to).copied();
    let destination_square = board.square(to);
    let legal = match start_item {
        Item::Mon { .. } => match destination_item {
            Some(Item::Mon { .. })
            | Some(Item::MonWithMana { .. })
            | Some(Item::MonWithConsumable { .. }) => false,
            Some(Item::Mana { .. }) => true,
            Some(Item::Consumable {
                consumable: Consumable::BombOrPotion,
            }) => true,
            Some(Item::Consumable { .. }) => false,
            None => square_allows_empty_mon(destination_square, MonKind::Drainer, color),
        },
        Item::MonWithMana { mana, .. } => match destination_item {
            Some(Item::Mon { .. })
            | Some(Item::MonWithMana { .. })
            | Some(Item::MonWithConsumable { .. }) => false,
            Some(Item::Mana { .. }) => true,
            Some(Item::Consumable {
                consumable: Consumable::BombOrPotion,
            }) => true,
            Some(Item::Consumable { .. }) => false,
            None => square_allows_mana_carrier(destination_square, mana),
        },
        Item::Mana { .. } | Item::MonWithConsumable { .. } | Item::Consumable { .. } => false,
    };
    if !legal {
        return None;
    }

    let mut board = board.clone();
    board.remove_item(from);

    let mut placed_item = start_item;
    match (start_item, destination_item) {
        (Item::Mon { mon }, Some(Item::Mana { mana })) => {
            placed_item = Item::MonWithMana { mon, mana };
        }
        (Item::MonWithMana { mon, mana: old }, Some(Item::Mana { mana: new })) => {
            board.put(Item::Mana { mana: old }, from);
            placed_item = Item::MonWithMana { mon, mana: new };
        }
        (Item::Mon { mon }, Some(Item::Consumable { .. })) => {
            placed_item = Item::Mon { mon };
        }
        (Item::MonWithMana { mon, mana }, Some(Item::Consumable { .. })) => {
            placed_item = Item::MonWithMana { mon, mana };
        }
        _ => {}
    }

    let mut scored_mana = None;
    if matches!(destination_square, Square::ManaPool { .. }) {
        if let Some(mana) = placed_item.mana().copied() {
            scored_mana = Some(mana);
            if let Some(mon) = placed_item.mon().copied() {
                placed_item = Item::Mon { mon };
            } else {
                return None;
            }
        }
    }

    board.put(placed_item, to);
    Some(ExactDrainerMovePreview {
        board,
        drainer_location: to,
        scored_mana,
    })
}

pub(crate) fn exact_secure_specific_mana_path_from(
    game: &MonsGame,
    color: Color,
    start: Location,
    wanted: Mana,
) -> Option<Vec<Location>> {
    if find_awake_drainer(&game.board, color) != Some(start) {
        return None;
    }
    let remaining_moves = if game.active_color == color {
        (Config::MONS_MOVES_PER_TURN - game.mons_moves_count).max(0)
    } else {
        Config::MONS_MOVES_PER_TURN
    };
    exact_secure_specific_mana_search(&game.board, color, wanted, remaining_moves, true).path
}

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

fn exact_spirit_summary(
    board: &Board,
    color: Color,
    remaining_mon_moves: i32,
    can_use_action: bool,
) -> ExactSpiritSummary {
    let started = Instant::now();
    if remaining_mon_moves < 0 {
        record_exact_query_diagnostics(
            ExactQueryFamily::SpiritAssist,
            Some(false),
            started.elapsed().as_nanos() as u64,
        );
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
        record_exact_query_diagnostics(
            ExactQueryFamily::SpiritAssist,
            Some(true),
            started.elapsed().as_nanos() as u64,
        );
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
    record_exact_query_diagnostics(
        ExactQueryFamily::SpiritAssist,
        Some(false),
        started.elapsed().as_nanos() as u64,
    );
    summary
}

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

fn exact_followup_summary(
    board: &Board,
    color: Color,
    remaining_moves: i32,
) -> ExactFollowupSummary {
    if remaining_moves < 0 {
        return ExactFollowupSummary::default();
    }

    let key = ExactFollowupSummaryKey {
        board_hash: exact_board_hash(board),
        color,
        remaining_moves,
    };
    if let Some(cached) =
        EXACT_FOLLOWUP_SUMMARY_CACHE.with(|cache| cache.borrow().entries.get(&key).copied())
    {
        return cached;
    }

    let summary = ExactFollowupSummary {
        best_score_steps: exact_best_score_steps_on_board(board, color),
        opponent_best_score_steps: exact_best_score_steps_on_board(board, color.other()),
        immediate_score: exact_best_immediate_score_on_board(board, color, remaining_moves),
        immediate_opponent_mana_score: exact_best_immediate_opponent_mana_score_on_board(
            board,
            color,
            remaining_moves,
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

    let mut queue = VecDeque::with_capacity(64);
    let mut seen = HashSet::with_capacity(128);
    queue.push_back((start, 0));
    seen.insert(start);
    let mut positions = Vec::new();

    while let Some((location, steps)) = queue.pop_front() {
        positions.push((location, steps));
        if steps >= remaining_mon_moves {
            continue;
        }
        for &next in location.nearby_locations_ref() {
            if seen.contains(&next) {
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

fn best_step_improvement(before: Option<i32>, after: Option<i32>) -> i32 {
    match (before, after) {
        (Some(before), Some(after)) if after < before => before - after,
        (None, Some(_)) => 2,
        _ => 0,
    }
}

fn best_step_worsening(before: Option<i32>, after: Option<i32>) -> i32 {
    match (before, after) {
        (Some(before), Some(after)) if after > before => after - before,
        (Some(_), None) => 2,
        _ => 0,
    }
}

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

fn exact_best_score_steps_on_board(board: &Board, color: Color) -> Option<i32> {
    let mut best = None;
    for (location, item) in board.occupied() {
        match item {
            Item::MonWithMana { mon, mana } if mon.color == color && !mon.is_fainted() => {
                if let Some(steps) = exact_carrier_steps_to_any_pool(board, location, *mana) {
                    best = Some(best.map_or(steps, |current: i32| current.min(steps)));
                }
            }
            Item::Mon { mon } | Item::MonWithConsumable { mon, .. }
                if mon.color == color && mon.kind == MonKind::Drainer && !mon.is_fainted() =>
            {
                if let Some(path) = exact_best_drainer_pickup_path(board, color, location) {
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

fn exact_best_immediate_score_on_board(board: &Board, color: Color, move_budget: i32) -> i32 {
    if move_budget < 0 {
        return 0;
    }

    let mut best = 0;
    for (location, item) in board.occupied() {
        match item {
            Item::MonWithMana { mon, mana } if mon.color == color && !mon.is_fainted() => {
                if exact_carrier_steps_to_any_pool(board, location, *mana)
                    .map_or(false, |steps| steps <= move_budget)
                {
                    best = best.max(mana.score(color));
                }
            }
            Item::Mon { mon } | Item::MonWithConsumable { mon, .. }
                if mon.color == color && mon.kind == MonKind::Drainer && !mon.is_fainted() =>
            {
                if let Some(path) = exact_best_drainer_pickup_path_filtered(
                    board,
                    color,
                    location,
                    Some(move_budget),
                    ExactPickupFilter::Any,
                ) {
                    best = best.max(path.mana_value);
                }
            }
            _ => {}
        }
    }
    best
}

fn exact_best_immediate_opponent_mana_score_on_board(
    board: &Board,
    color: Color,
    move_budget: i32,
) -> i32 {
    if move_budget < 0 {
        return 0;
    }

    let mut best = 0;
    let opponent_mana = Mana::Regular(color.other());
    for (location, item) in board.occupied() {
        match item {
            Item::MonWithMana { mon, mana }
                if mon.color == color && !mon.is_fainted() && *mana == opponent_mana =>
            {
                if exact_carrier_steps_to_any_pool(board, location, *mana)
                    .map_or(false, |steps| steps <= move_budget)
                {
                    best = best.max(mana.score(color));
                }
            }
            Item::Mon { mon } | Item::MonWithConsumable { mon, .. }
                if mon.color == color && mon.kind == MonKind::Drainer && !mon.is_fainted() =>
            {
                if let Some(path) = exact_best_drainer_pickup_path_filtered(
                    board,
                    color,
                    location,
                    Some(move_budget),
                    ExactPickupFilter::Wanted(opponent_mana),
                ) {
                    best = best.max(path.mana_value);
                }
            }
            _ => {}
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
            MonsGameModel::is_location_guarded_by_angel(&board, Color::White, Location::new(6, 5));
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

        assert_eq!(exact_best_immediate_score_on_board(&game.board, Color::White, 2), 0);

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
        assert_eq!(after_summary.immediate_score, Mana::Supermana.score(Color::White));

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

        assert_eq!(exact_best_immediate_score_on_board(&game.board, Color::White, 2), 0);

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
        assert_eq!(after_summary.immediate_score, Mana::Supermana.score(Color::White));

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
