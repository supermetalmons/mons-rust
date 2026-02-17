use crate::*;

#[derive(Debug, Clone, Copy)]
pub struct ScoringWeights {
    pub confirmed_score: i32,
    pub fainted_mon: i32,
    pub fainted_drainer: i32,
    pub fainted_cooldown_step: i32,
    pub drainer_at_risk: i32,
    pub mana_close_to_same_pool: i32,
    pub mon_with_mana_close_to_any_pool: i32,
    pub extra_for_supermana: i32,
    pub extra_for_opponents_mana: i32,
    pub drainer_close_to_mana: i32,
    pub drainer_holding_mana: i32,
    pub drainer_close_to_own_pool: i32,
    pub drainer_close_to_supermana: i32,
    pub mon_close_to_center: i32,
    pub spirit_close_to_enemy: i32,
    pub angel_guarding_drainer: i32,
    pub angel_close_to_friendly_drainer: i32,
    pub has_consumable: i32,
    pub active_mon: i32,
    pub regular_mana_to_owner_pool: i32,
    pub regular_mana_drainer_control: i32,
    pub supermana_drainer_control: i32,
    pub mana_carrier_at_risk: i32,
    pub mana_carrier_guarded: i32,
    pub mana_carrier_one_step_from_pool: i32,
    pub supermana_carrier_one_step_from_pool_extra: i32,
    pub immediate_winning_carrier: i32,
    pub drainer_best_mana_path: i32,
    pub drainer_pickup_score_this_turn: i32,
    pub mana_carrier_score_this_turn: i32,
    pub drainer_immediate_threat: i32,
}

pub const DEFAULT_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    confirmed_score: 1000,
    fainted_mon: -500,
    fainted_drainer: -800,
    fainted_cooldown_step: 0,
    drainer_at_risk: -350,
    mana_close_to_same_pool: 500,
    mon_with_mana_close_to_any_pool: 800,
    extra_for_supermana: 120,
    extra_for_opponents_mana: 100,
    drainer_close_to_mana: 300,
    drainer_holding_mana: 350,
    drainer_close_to_own_pool: 180,
    drainer_close_to_supermana: 120,
    mon_close_to_center: 210,
    spirit_close_to_enemy: 160,
    angel_guarding_drainer: 180,
    angel_close_to_friendly_drainer: 120,
    has_consumable: 110,
    active_mon: 50,
    regular_mana_to_owner_pool: 0,
    regular_mana_drainer_control: 0,
    supermana_drainer_control: 0,
    mana_carrier_at_risk: 0,
    mana_carrier_guarded: 0,
    mana_carrier_one_step_from_pool: 0,
    supermana_carrier_one_step_from_pool_extra: 0,
    immediate_winning_carrier: 0,
    drainer_best_mana_path: 0,
    drainer_pickup_score_this_turn: 0,
    mana_carrier_score_this_turn: 0,
    drainer_immediate_threat: 0,
};

pub const BALANCED_DISTANCE_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    confirmed_score: 1000,
    fainted_mon: -520,
    fainted_drainer: -900,
    fainted_cooldown_step: -80,
    drainer_at_risk: -420,
    mana_close_to_same_pool: 520,
    mon_with_mana_close_to_any_pool: 820,
    extra_for_supermana: 130,
    extra_for_opponents_mana: 120,
    drainer_close_to_mana: 330,
    drainer_holding_mana: 370,
    drainer_close_to_own_pool: 280,
    drainer_close_to_supermana: 180,
    mon_close_to_center: 180,
    spirit_close_to_enemy: 220,
    angel_guarding_drainer: 280,
    angel_close_to_friendly_drainer: 180,
    has_consumable: 105,
    active_mon: 45,
    regular_mana_to_owner_pool: 0,
    regular_mana_drainer_control: 0,
    supermana_drainer_control: 0,
    mana_carrier_at_risk: 0,
    mana_carrier_guarded: 0,
    mana_carrier_one_step_from_pool: 160,
    supermana_carrier_one_step_from_pool_extra: 80,
    immediate_winning_carrier: 0,
    drainer_best_mana_path: 0,
    drainer_pickup_score_this_turn: 0,
    mana_carrier_score_this_turn: 0,
    drainer_immediate_threat: 0,
};

#[cfg(test)]
pub const MANA_RACE_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    regular_mana_to_owner_pool: 260,
    regular_mana_drainer_control: 26,
    supermana_drainer_control: 44,
    mana_carrier_at_risk: -260,
    mana_carrier_guarded: 140,
    drainer_close_to_own_pool: 300,
    drainer_close_to_supermana: 220,
    drainer_holding_mana: 380,
    spirit_close_to_enemy: 210,
    angel_guarding_drainer: 290,
    angel_close_to_friendly_drainer: 190,
    ..BALANCED_DISTANCE_SCORING_WEIGHTS
};

pub const MANA_RACE_LITE_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    regular_mana_to_owner_pool: 150,
    regular_mana_drainer_control: 15,
    supermana_drainer_control: 26,
    mana_carrier_at_risk: -150,
    mana_carrier_guarded: 70,
    drainer_close_to_own_pool: 290,
    drainer_close_to_supermana: 200,
    angel_guarding_drainer: 290,
    mana_close_to_same_pool: 420,
    fainted_cooldown_step: -70,
    mana_carrier_one_step_from_pool: 220,
    supermana_carrier_one_step_from_pool_extra: 120,
    immediate_winning_carrier: 0,
    ..BALANCED_DISTANCE_SCORING_WEIGHTS
};

pub const MANA_RACE_LITE_TACTICAL_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    regular_mana_to_owner_pool: 170,
    regular_mana_drainer_control: 18,
    supermana_drainer_control: 30,
    fainted_cooldown_step: -110,
    mana_carrier_at_risk: -220,
    mana_carrier_guarded: 120,
    mana_carrier_one_step_from_pool: 270,
    supermana_carrier_one_step_from_pool_extra: 170,
    ..MANA_RACE_LITE_SCORING_WEIGHTS
};

#[cfg(test)]
pub const FINISHER_BALANCED_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    mana_carrier_one_step_from_pool: 260,
    supermana_carrier_one_step_from_pool_extra: 140,
    immediate_winning_carrier: 850,
    ..BALANCED_DISTANCE_SCORING_WEIGHTS
};

#[cfg(test)]
pub const FINISHER_MANA_RACE_LITE_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    mana_carrier_one_step_from_pool: 300,
    supermana_carrier_one_step_from_pool_extra: 180,
    immediate_winning_carrier: 980,
    ..MANA_RACE_LITE_SCORING_WEIGHTS
};

#[cfg(test)]
pub const FINISHER_MANA_RACE_LITE_AGGRESSIVE_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    mana_carrier_at_risk: -190,
    mana_carrier_guarded: 90,
    mana_carrier_one_step_from_pool: 340,
    supermana_carrier_one_step_from_pool_extra: 220,
    immediate_winning_carrier: 1200,
    ..MANA_RACE_LITE_SCORING_WEIGHTS
};

pub const FINISHER_BALANCED_SOFT_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    mana_carrier_one_step_from_pool: 220,
    supermana_carrier_one_step_from_pool_extra: 110,
    immediate_winning_carrier: 360,
    ..BALANCED_DISTANCE_SCORING_WEIGHTS
};

pub const FINISHER_BALANCED_SOFT_AGGRESSIVE_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    mana_carrier_one_step_from_pool: 250,
    supermana_carrier_one_step_from_pool_extra: 130,
    immediate_winning_carrier: 540,
    ..BALANCED_DISTANCE_SCORING_WEIGHTS
};

pub const FINISHER_MANA_RACE_LITE_SOFT_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    mana_carrier_one_step_from_pool: 250,
    supermana_carrier_one_step_from_pool_extra: 140,
    immediate_winning_carrier: 420,
    ..MANA_RACE_LITE_SCORING_WEIGHTS
};

pub const FINISHER_MANA_RACE_LITE_SOFT_AGGRESSIVE_SCORING_WEIGHTS: ScoringWeights =
    ScoringWeights {
        mana_carrier_one_step_from_pool: 280,
        supermana_carrier_one_step_from_pool_extra: 165,
        immediate_winning_carrier: 620,
        ..MANA_RACE_LITE_SCORING_WEIGHTS
    };

pub const MANA_RACE_LITE_D2_TUNED_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    regular_mana_to_owner_pool: 170,
    regular_mana_drainer_control: 18,
    mana_close_to_same_pool: 380,
    drainer_close_to_own_pool: 320,
    mana_carrier_at_risk: -210,
    mana_carrier_guarded: 95,
    mana_carrier_one_step_from_pool: 260,
    supermana_carrier_one_step_from_pool_extra: 150,
    immediate_winning_carrier: 300,
    ..MANA_RACE_LITE_SCORING_WEIGHTS
};

#[cfg(test)]
pub const MANA_RACE_LITE_D2_TUNED_AGGRESSIVE_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    regular_mana_to_owner_pool: 190,
    regular_mana_drainer_control: 22,
    mana_close_to_same_pool: 360,
    drainer_close_to_own_pool: 340,
    mana_carrier_at_risk: -250,
    mana_carrier_guarded: 110,
    mana_carrier_one_step_from_pool: 300,
    supermana_carrier_one_step_from_pool_extra: 180,
    immediate_winning_carrier: 420,
    ..MANA_RACE_LITE_SCORING_WEIGHTS
};

#[cfg(test)]
pub const MANA_RACE_NEUTRAL_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    regular_mana_to_owner_pool: 220,
    regular_mana_drainer_control: 18,
    supermana_drainer_control: 34,
    mana_carrier_at_risk: -180,
    mana_carrier_guarded: 90,
    mana_close_to_same_pool: 300,
    drainer_close_to_own_pool: 300,
    drainer_close_to_supermana: 210,
    fainted_cooldown_step: -90,
    mana_carrier_one_step_from_pool: 200,
    supermana_carrier_one_step_from_pool_extra: 100,
    ..BALANCED_DISTANCE_SCORING_WEIGHTS
};

pub const TACTICAL_BALANCED_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    fainted_cooldown_step: -120,
    spirit_close_to_enemy: 230,
    angel_guarding_drainer: 300,
    mana_carrier_at_risk: -200,
    mana_carrier_guarded: 110,
    mana_carrier_one_step_from_pool: 240,
    supermana_carrier_one_step_from_pool_extra: 150,
    ..BALANCED_DISTANCE_SCORING_WEIGHTS
};

pub const TACTICAL_MANA_RACE_LITE_SCORING_WEIGHTS: ScoringWeights =
    MANA_RACE_LITE_TACTICAL_SCORING_WEIGHTS;

pub const TACTICAL_BALANCED_AGGRESSIVE_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    fainted_cooldown_step: -160,
    mana_carrier_at_risk: -260,
    mana_carrier_guarded: 140,
    mana_carrier_one_step_from_pool: 320,
    supermana_carrier_one_step_from_pool_extra: 220,
    spirit_close_to_enemy: 250,
    angel_guarding_drainer: 320,
    ..TACTICAL_BALANCED_SCORING_WEIGHTS
};

pub const TACTICAL_MANA_RACE_LITE_AGGRESSIVE_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    fainted_cooldown_step: -150,
    mana_carrier_at_risk: -290,
    mana_carrier_guarded: 160,
    mana_carrier_one_step_from_pool: 360,
    supermana_carrier_one_step_from_pool_extra: 240,
    regular_mana_to_owner_pool: 200,
    regular_mana_drainer_control: 24,
    supermana_drainer_control: 36,
    ..TACTICAL_MANA_RACE_LITE_SCORING_WEIGHTS
};

pub const RUNTIME_FAST_DRAINER_CONTEXT_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    drainer_best_mana_path: 220,
    drainer_pickup_score_this_turn: 180,
    mana_carrier_score_this_turn: 260,
    drainer_immediate_threat: -190,
    drainer_close_to_mana: 360,
    drainer_holding_mana: 430,
    mana_carrier_at_risk: -250,
    mana_carrier_guarded: 130,
    mana_carrier_one_step_from_pool: 300,
    supermana_carrier_one_step_from_pool_extra: 190,
    immediate_winning_carrier: 520,
    ..MANA_RACE_LITE_D2_TUNED_SCORING_WEIGHTS
};

pub const RUNTIME_RUSH_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    drainer_at_risk: -300,
    mana_close_to_same_pool: 600,
    drainer_close_to_mana: 420,
    drainer_close_to_own_pool: 360,
    drainer_close_to_supermana: 220,
    mon_close_to_center: 140,
    spirit_close_to_enemy: 180,
    angel_guarding_drainer: 200,
    angel_close_to_friendly_drainer: 120,
    ..BALANCED_DISTANCE_SCORING_WEIGHTS
};

#[cfg(test)]
pub const RUNTIME_FAST_WINLOSS_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    regular_mana_to_owner_pool: 176,
    regular_mana_drainer_control: 19,
    supermana_drainer_control: 28,
    drainer_close_to_own_pool: 325,
    mana_carrier_at_risk: -225,
    mana_carrier_guarded: 105,
    mana_carrier_one_step_from_pool: 285,
    supermana_carrier_one_step_from_pool_extra: 170,
    immediate_winning_carrier: 380,
    drainer_best_mana_path: 90,
    drainer_pickup_score_this_turn: 70,
    mana_carrier_score_this_turn: 140,
    drainer_immediate_threat: -95,
    ..MANA_RACE_LITE_D2_TUNED_SCORING_WEIGHTS
};

#[cfg(test)]
pub const RUNTIME_NORMAL_WINLOSS_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    confirmed_score: 900,
    fainted_cooldown_step: -130,
    spirit_close_to_enemy: 235,
    angel_guarding_drainer: 310,
    regular_mana_to_owner_pool: 70,
    regular_mana_drainer_control: 8,
    supermana_drainer_control: 16,
    mana_carrier_at_risk: -230,
    mana_carrier_guarded: 125,
    mana_carrier_one_step_from_pool: 300,
    supermana_carrier_one_step_from_pool_extra: 185,
    immediate_winning_carrier: 520,
    drainer_best_mana_path: 70,
    drainer_pickup_score_this_turn: 55,
    mana_carrier_score_this_turn: 150,
    drainer_immediate_threat: -100,
    ..TACTICAL_BALANCED_SCORING_WEIGHTS
};

#[cfg(test)]
pub const RUNTIME_FAST_DRAINER_PRIORITY_SCORING_WEIGHTS: ScoringWeights = ScoringWeights {
    drainer_at_risk: -520,
    drainer_close_to_mana: 370,
    drainer_holding_mana: 470,
    drainer_close_to_own_pool: 350,
    regular_mana_drainer_control: 24,
    mana_carrier_at_risk: -240,
    mana_carrier_guarded: 120,
    mana_carrier_one_step_from_pool: 310,
    supermana_carrier_one_step_from_pool_extra: 190,
    immediate_winning_carrier: 460,
    angel_guarding_drainer: 340,
    ..MANA_RACE_LITE_D2_TUNED_SCORING_WEIGHTS
};

const SPIRIT_ON_OWN_BASE_PENALTY: i32 = 180;

pub fn evaluate_preferability(game: &MonsGame, color: Color) -> i32 {
    evaluate_preferability_with_weights(game, color, &DEFAULT_SCORING_WEIGHTS)
}

pub fn evaluate_preferability_with_weights(
    game: &MonsGame,
    color: Color,
    weights: &ScoringWeights,
) -> i32 {
    let supermana_base = game.board.supermana_base();
    let remaining_mon_moves_for_active =
        (Config::MONS_MOVES_PER_TURN - game.mons_moves_count).max(0);

    let mons_bases = Config::mons_bases_ref();

    let mut score = match color {
        Color::White => {
            (game.white_score - game.black_score) * weights.confirmed_score
                + (game.white_potions_count - game.black_potions_count) * weights.has_consumable
        }
        Color::Black => {
            (game.black_score - game.white_score) * weights.confirmed_score
                + (game.black_potions_count - game.white_potions_count) * weights.has_consumable
        }
    };

    score *= weights.confirmed_score;

    for (&location, item) in &game.board.items {
        match item {
            Item::Mon { mon } => {
                let my_mon_multiplier = if mon.color == color { 1 } else { -1 };
                let is_drainer = mon.kind == MonKind::Drainer;

                if mon.is_fainted() {
                    score += my_mon_multiplier
                        * (if is_drainer {
                            weights.fainted_drainer
                        } else {
                            weights.fainted_mon
                        });
                    score += my_mon_multiplier * weights.fainted_cooldown_step * mon.cooldown;
                } else if is_drainer {
                    let (danger, min_mana, angel_nearby) =
                        drainer_distances(&game.board, mon.color, location);
                    score += my_mon_multiplier * weights.drainer_close_to_mana / min_mana;
                    score += my_mon_multiplier * weights.drainer_close_to_own_pool
                        / distance(location, Destination::ClosestPool(mon.color));
                    score += my_mon_multiplier * weights.drainer_close_to_supermana
                        / distance_to_location(location, supermana_base);
                    if !angel_nearby {
                        score += my_mon_multiplier * weights.drainer_at_risk / danger;
                    } else {
                        score += my_mon_multiplier * weights.angel_guarding_drainer;
                    }

                    if let Some((path_steps, mana_value)) =
                        best_drainer_pickup_path(&game.board, mon.color, location)
                    {
                        score += my_mon_multiplier * weights.drainer_best_mana_path * mana_value
                            / (path_steps + 1);
                        if mon.color == game.active_color
                            && path_steps <= remaining_mon_moves_for_active
                        {
                            score += my_mon_multiplier
                                * weights.drainer_pickup_score_this_turn
                                * mana_value;
                        }
                    }

                    let (action_threats, bomb_threats) =
                        drainer_immediate_threats(&game.board, mon.color, location);
                    let immediate_threats = if angel_nearby {
                        bomb_threats
                    } else {
                        action_threats + bomb_threats
                    };
                    if immediate_threats > 0 {
                        score += my_mon_multiplier
                            * weights.drainer_immediate_threat
                            * immediate_threats;
                    }
                } else if mon.kind == MonKind::Spirit {
                    let enemy_distance =
                        nearest_enemy_mon_distance(&game.board, mon.color, location);
                    score += my_mon_multiplier * weights.spirit_close_to_enemy / enemy_distance;
                    score -= my_mon_multiplier
                        * spirit_on_own_base_penalty(&game.board, *mon, location);
                } else if mon.kind == MonKind::Angel {
                    let friendly_drainer_distance =
                        nearest_friendly_drainer_distance(&game.board, mon.color, location);
                    score += my_mon_multiplier * weights.angel_close_to_friendly_drainer
                        / friendly_drainer_distance;
                } else if mon.kind != MonKind::Angel {
                    score += my_mon_multiplier * weights.mon_close_to_center
                        / distance(location, Destination::Center);
                }

                if !mons_bases.contains(&location) {
                    score += my_mon_multiplier * weights.active_mon;
                }
            }
            Item::MonWithConsumable { mon, .. } => {
                let my_mon_multiplier = if mon.color == color { 1 } else { -1 };
                let is_drainer = mon.kind == MonKind::Drainer;
                score += my_mon_multiplier * weights.has_consumable;

                if is_drainer {
                    let (danger, min_mana, angel_nearby) =
                        drainer_distances(&game.board, mon.color, location);
                    score += my_mon_multiplier * weights.drainer_close_to_mana / min_mana;
                    score += my_mon_multiplier * weights.drainer_close_to_own_pool
                        / distance(location, Destination::ClosestPool(mon.color));
                    score += my_mon_multiplier * weights.drainer_close_to_supermana
                        / distance_to_location(location, supermana_base);
                    if !angel_nearby {
                        score += my_mon_multiplier * weights.drainer_at_risk / danger;
                    } else {
                        score += my_mon_multiplier * weights.angel_guarding_drainer;
                    }

                    let (action_threats, bomb_threats) =
                        drainer_immediate_threats(&game.board, mon.color, location);
                    let immediate_threats = if angel_nearby {
                        bomb_threats
                    } else {
                        action_threats + bomb_threats
                    };
                    if immediate_threats > 0 {
                        score += my_mon_multiplier
                            * weights.drainer_immediate_threat
                            * immediate_threats;
                    }
                } else if mon.kind == MonKind::Spirit {
                    let enemy_distance =
                        nearest_enemy_mon_distance(&game.board, mon.color, location);
                    score += my_mon_multiplier * weights.spirit_close_to_enemy / enemy_distance;
                    score -= my_mon_multiplier
                        * spirit_on_own_base_penalty(&game.board, *mon, location);
                } else if mon.kind == MonKind::Angel {
                    let friendly_drainer_distance =
                        nearest_friendly_drainer_distance(&game.board, mon.color, location);
                    score += my_mon_multiplier * weights.angel_close_to_friendly_drainer
                        / friendly_drainer_distance;
                } else if mon.kind != MonKind::Angel {
                    score += my_mon_multiplier * weights.mon_close_to_center
                        / distance(location, Destination::Center);
                }
            }
            Item::Mana { mana } => {
                score += weights.mana_close_to_same_pool
                    / distance(location, Destination::ClosestPool(color));
                let mana_bonus = match mana {
                    Mana::Regular(mana_color) => {
                        let owner_multiplier = if *mana_color == color { 1 } else { -1 };
                        let owner_pool_distance =
                            distance(location, Destination::ClosestPool(*mana_color));
                        let owner_drainer_distance =
                            nearest_friendly_drainer_distance(&game.board, *mana_color, location);
                        let enemy_drainer_distance = nearest_friendly_drainer_distance(
                            &game.board,
                            mana_color.other(),
                            location,
                        );
                        let drainer_control =
                            (enemy_drainer_distance - owner_drainer_distance).clamp(-4, 4);
                        owner_multiplier
                            * (weights.regular_mana_to_owner_pool / owner_pool_distance
                                + weights.regular_mana_drainer_control * drainer_control)
                    }
                    Mana::Supermana => {
                        let my_drainer_distance =
                            nearest_friendly_drainer_distance(&game.board, color, location);
                        let enemy_drainer_distance =
                            nearest_friendly_drainer_distance(&game.board, color.other(), location);
                        let drainer_control =
                            (enemy_drainer_distance - my_drainer_distance).clamp(-4, 4);
                        weights.supermana_drainer_control * drainer_control
                    }
                };
                score += mana_bonus;
            }
            Item::MonWithMana { mon, mana } => {
                let my_mon_multiplier = if mon.color == color { 1 } else { -1 };
                let nearest_pool_distance = distance(location, Destination::AnyClosestPool);
                let mana_extra = match mana {
                    Mana::Regular(mana_color) => {
                        if *mana_color == color {
                            0
                        } else {
                            weights.extra_for_opponents_mana
                        }
                    }
                    Mana::Supermana => weights.extra_for_supermana,
                };

                score += my_mon_multiplier * weights.drainer_holding_mana;
                score += my_mon_multiplier * (weights.mon_with_mana_close_to_any_pool + mana_extra)
                    / nearest_pool_distance;

                if nearest_pool_distance <= 2 {
                    let immediate_bonus = match mana {
                        Mana::Supermana => {
                            weights.mana_carrier_one_step_from_pool
                                + weights.supermana_carrier_one_step_from_pool_extra
                        }
                        Mana::Regular(_) => weights.mana_carrier_one_step_from_pool,
                    };
                    score += my_mon_multiplier * immediate_bonus;

                    let carrier_score = if mon.color == Color::White {
                        game.white_score
                    } else {
                        game.black_score
                    };
                    let score_if_scored_now = carrier_score + mana.score(mon.color);
                    if score_if_scored_now >= Config::TARGET_SCORE {
                        score += my_mon_multiplier * weights.immediate_winning_carrier;
                    }
                }

                let (danger, _, angel_nearby) = drainer_distances(&game.board, mon.color, location);
                score += my_mon_multiplier * weights.mana_carrier_at_risk / danger;
                if angel_nearby {
                    score += my_mon_multiplier * weights.mana_carrier_guarded;
                }

                if mon.color == game.active_color {
                    let pool_steps = nearest_pool_distance - 1;
                    if pool_steps <= remaining_mon_moves_for_active {
                        score += my_mon_multiplier * weights.mana_carrier_score_this_turn;
                    }
                }

                if mon.kind == MonKind::Drainer {
                    score += my_mon_multiplier * weights.drainer_close_to_own_pool
                        / distance(location, Destination::ClosestPool(mon.color));

                    let (action_threats, bomb_threats) =
                        drainer_immediate_threats(&game.board, mon.color, location);
                    let immediate_threats = if angel_nearby {
                        bomb_threats
                    } else {
                        action_threats + bomb_threats
                    };
                    if immediate_threats > 0 {
                        score += my_mon_multiplier
                            * weights.drainer_immediate_threat
                            * immediate_threats;
                    }
                } else if mon.kind == MonKind::Spirit {
                    score -= my_mon_multiplier
                        * spirit_on_own_base_penalty(&game.board, *mon, location);
                }
            }
            Item::Consumable { .. } => {}
        }
    }

    score
}

fn spirit_on_own_base_penalty(board: &Board, mon: Mon, location: Location) -> i32 {
    if mon.kind == MonKind::Spirit && !mon.is_fainted() && location == board.base(mon) {
        SPIRIT_ON_OWN_BASE_PENALTY
    } else {
        0
    }
}

enum Destination {
    Center,
    AnyClosestPool,
    ClosestPool(Color),
}

fn drainer_distances(board: &Board, color: Color, location: Location) -> (i32, i32, bool) {
    let mut min_mana = Config::BOARD_SIZE as i32;
    let mut min_danger = Config::BOARD_SIZE as i32;
    let mut angel_nearby = false;

    for (&item_location, item) in &board.items {
        match item {
            Item::Mana { .. } => {
                let delta = item_location.distance(&location) as i32;
                if delta < min_mana {
                    min_mana = delta;
                }
            }
            Item::Mon { mon } | Item::MonWithConsumable { mon, .. } => {
                if mon.color != color
                    && !mon.is_fainted()
                    && (mon.kind == MonKind::Mystic
                        || mon.kind == MonKind::Demon
                        || matches!(item, Item::MonWithConsumable { .. }))
                {
                    let delta = item_location.distance(&location) as i32;
                    if delta < min_danger {
                        min_danger = delta;
                    }
                } else if mon.color == color
                    && !mon.is_fainted()
                    && mon.kind == MonKind::Angel
                    && item_location.distance(&location) == 1
                {
                    angel_nearby = true;
                }
            }
            Item::Consumable { .. } => {
                let delta = item_location.distance(&location) as i32;
                if delta < min_danger {
                    min_danger = delta;
                }
            }
            Item::MonWithMana { .. } => {}
        }
    }

    (min_danger, min_mana, angel_nearby)
}

fn best_drainer_pickup_path(board: &Board, color: Color, from: Location) -> Option<(i32, i32)> {
    let mut best: Option<(i32, i32)> = None;
    for (&mana_location, item) in &board.items {
        let Item::Mana { mana } = item else {
            continue;
        };

        let pickup_steps = from.distance(&mana_location) as i32;
        let score_steps = distance(mana_location, Destination::AnyClosestPool) - 1;
        let total_steps = pickup_steps + score_steps;
        let mana_value = mana.score(color);

        let replace = match best {
            None => true,
            Some((best_steps, best_mana_value)) => {
                let total_metric = total_steps * 3 - mana_value;
                let best_metric = best_steps * 3 - best_mana_value;
                total_metric < best_metric
                    || (total_metric == best_metric && mana_value > best_mana_value)
            }
        };
        if replace {
            best = Some((total_steps, mana_value));
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spirit_on_own_base_penalty_applies_for_awake_spirit_on_base() {
        let spirit = Mon::new(MonKind::Spirit, Color::White, 0);
        let board = Board::new();
        let base = board.base(spirit);
        assert_eq!(
            spirit_on_own_base_penalty(&board, spirit, base),
            SPIRIT_ON_OWN_BASE_PENALTY
        );
    }

    #[test]
    fn spirit_on_own_base_penalty_is_zero_off_base_or_fainted() {
        let board = Board::new();

        let awake_spirit = Mon::new(MonKind::Spirit, Color::White, 0);
        let awake_base = board.base(awake_spirit);
        let off_base = if awake_base.i == 0 {
            Location::new(awake_base.i + 1, awake_base.j)
        } else {
            Location::new(awake_base.i - 1, awake_base.j)
        };
        assert_eq!(spirit_on_own_base_penalty(&board, awake_spirit, off_base), 0);

        let fainted_spirit = Mon::new(MonKind::Spirit, Color::White, 1);
        let fainted_base = board.base(fainted_spirit);
        assert_eq!(
            spirit_on_own_base_penalty(&board, fainted_spirit, fainted_base),
            0
        );
    }
}

fn drainer_immediate_threats(board: &Board, color: Color, location: Location) -> (i32, i32) {
    let mut action_threats = 0;
    let mut bomb_threats = 0;

    for (&threat_location, item) in &board.items {
        match item {
            Item::Mon { mon } => {
                if mon.color == color || mon.is_fainted() {
                    continue;
                }
                if mon.kind == MonKind::Mystic
                    && (threat_location.i - location.i).abs() == 2
                    && (threat_location.j - location.j).abs() == 2
                {
                    action_threats += 1;
                } else if mon.kind == MonKind::Demon {
                    let di = (threat_location.i - location.i).abs();
                    let dj = (threat_location.j - location.j).abs();
                    if (di == 2 && dj == 0) || (di == 0 && dj == 2) {
                        let middle = threat_location.location_between(&location);
                        if board.item(middle).is_none()
                            && !matches!(
                                board.square(middle),
                                Square::SupermanaBase | Square::MonBase { .. }
                            )
                        {
                            action_threats += 1;
                        }
                    }
                }
            }
            Item::MonWithConsumable { mon, consumable } => {
                if mon.color == color || mon.is_fainted() {
                    continue;
                }
                if *consumable == Consumable::Bomb && threat_location.distance(&location) <= 3 {
                    bomb_threats += 1;
                }
            }
            Item::Mana { .. } | Item::MonWithMana { .. } | Item::Consumable { .. } => {}
        }
    }

    (action_threats, bomb_threats)
}

fn nearest_enemy_mon_distance(board: &Board, color: Color, location: Location) -> i32 {
    let mut best = Config::BOARD_SIZE as i32;
    for (&item_location, item) in &board.items {
        if let Some(mon) = item.mon() {
            if mon.color != color && !mon.is_fainted() {
                let delta = item_location.distance(&location) as i32;
                if delta < best {
                    best = delta;
                }
            }
        }
    }
    best.max(1)
}

fn nearest_friendly_drainer_distance(board: &Board, color: Color, location: Location) -> i32 {
    let mut best = Config::BOARD_SIZE as i32;
    for (&item_location, item) in &board.items {
        if let Some(mon) = item.mon() {
            if mon.color == color && mon.kind == MonKind::Drainer && !mon.is_fainted() {
                let delta = item_location.distance(&location) as i32;
                if delta < best {
                    best = delta;
                }
            }
        }
    }
    best.max(1)
}

fn distance_to_location(location: Location, destination: Location) -> i32 {
    location.distance(&destination) as i32 + 1
}

fn distance(location: Location, destination: Destination) -> i32 {
    let distance = match destination {
        Destination::Center => {
            // Once within 1 step from center, extra centralization is not rewarded further.
            (Config::BOARD_CENTER_INDEX as i32 - location.i as i32)
                .abs()
                .max(1)
        }
        Destination::AnyClosestPool => {
            let max_index = Config::MAX_LOCATION_INDEX as i32;
            let i = location.i as i32;
            let j = location.j as i32;
            i32::max(
                i32::min(i, (max_index - i).abs()),
                i32::min(j, (max_index - j).abs()),
            )
        }
        Destination::ClosestPool(color) => {
            let pool_row = if color == Color::White {
                Config::MAX_LOCATION_INDEX as i32
            } else {
                0
            };
            let i = location.i as i32;
            let j = location.j as i32;
            i32::max(
                (pool_row - i).abs(),
                i32::min(j, (Config::MAX_LOCATION_INDEX as i32 - j).abs()),
            )
        }
    };
    distance + 1
}
