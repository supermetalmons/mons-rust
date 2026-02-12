use crate::*;

const CONFIRMED_SCORE: i32 = 1_000;
const FAINTED_MON: i32 = -500;
const FAINTED_DRAINER: i32 = -800;
const DRAINER_AT_RISK: i32 = -350;
const MANA_CLOSE_TO_SAME_POOL: i32 = 500;
const MON_WITH_MANA_CLOSE_TO_ANY_POOL: i32 = 800;
const EXTRA_FOR_SUPERMANA: i32 = 120;
const EXTRA_FOR_OPPONENTS_MANA: i32 = 100;
const DRAINER_CLOSE_TO_MANA: i32 = 300;
const DRAINER_HOLDING_MANA: i32 = 350;
const HAS_CONSUMABLE: i32 = 110;
const ACTIVE_MON: i32 = 50;

pub fn evaluate_preferability(game: &MonsGame, color: Color) -> i32 {
    let mut score = match color {
        Color::White => {
            (game.white_score - game.black_score) * CONFIRMED_SCORE
                + (game.white_potions_count - game.black_potions_count) * HAS_CONSUMABLE
        }
        Color::Black => {
            (game.black_score - game.white_score) * CONFIRMED_SCORE
                + (game.black_potions_count - game.white_potions_count) * HAS_CONSUMABLE
        }
    };

    score *= CONFIRMED_SCORE;

    for (&location, item) in &game.board.items {
        match item {
            Item::Mon { mon } => {
                let side_multiplier = mon_multiplier(color, mon.color);
                score += evaluate_mon_state(&game.board, mon, location, color);
                if !matches!(game.board.square(location), Square::MonBase { .. }) {
                    score += side_multiplier * ACTIVE_MON;
                }
            }
            Item::MonWithConsumable { mon, .. } => {
                let side_multiplier = mon_multiplier(color, mon.color);
                score += side_multiplier * HAS_CONSUMABLE;
                score += evaluate_mon_state(&game.board, mon, location, color);
            }
            Item::Mana { .. } => {
                score += MANA_CLOSE_TO_SAME_POOL
                    / distance(location, Destination::ClosestPool(color)).max(1);
            }
            Item::MonWithMana { mon, mana } => {
                let side_multiplier = mon_multiplier(color, mon.color);
                let mana_extra = match mana {
                    Mana::Regular(mana_color) => {
                        if *mana_color == color {
                            0
                        } else {
                            EXTRA_FOR_OPPONENTS_MANA
                        }
                    }
                    Mana::Supermana => EXTRA_FOR_SUPERMANA,
                };

                score += side_multiplier * DRAINER_HOLDING_MANA;
                score += side_multiplier * (MON_WITH_MANA_CLOSE_TO_ANY_POOL + mana_extra)
                    / distance(location, Destination::AnyClosestPool).max(1);
            }
            Item::Consumable { .. } => {}
        }
    }

    score
}

enum Destination {
    AnyClosestPool,
    ClosestPool(Color),
}

fn mon_multiplier(perspective: Color, mon_color: Color) -> i32 {
    if mon_color == perspective {
        1
    } else {
        -1
    }
}

fn evaluate_mon_state(board: &Board, mon: &Mon, location: Location, perspective: Color) -> i32 {
    let side_multiplier = mon_multiplier(perspective, mon.color);

    if mon.is_fainted() {
        return side_multiplier
            * if mon.kind == MonKind::Drainer {
                FAINTED_DRAINER
            } else {
                FAINTED_MON
            };
    }

    if mon.kind == MonKind::Drainer {
        let (danger, min_mana, angel_nearby) = drainer_distances(board, mon.color, location);
        let mut drainer_score = side_multiplier * DRAINER_CLOSE_TO_MANA / min_mana.max(1);
        if !angel_nearby {
            drainer_score += side_multiplier * DRAINER_AT_RISK / danger.max(1);
        }
        return drainer_score;
    }

    0
}

fn drainer_distances(board: &Board, color: Color, location: Location) -> (i32, i32, bool) {
    let mut min_mana = Config::BOARD_SIZE;
    let mut min_danger = Config::BOARD_SIZE;
    let mut angel_nearby = false;

    for (&item_location, item) in &board.items {
        match item {
            Item::Mana { .. } => {
                let delta = item_location.distance(&location);
                if delta < min_mana {
                    min_mana = delta;
                }
            }
            Item::Mon { mon } | Item::MonWithConsumable { mon, .. } => {
                if mon.color == color.other()
                    && !mon.is_fainted()
                    && (mon.kind == MonKind::Mystic
                        || mon.kind == MonKind::Demon
                        || matches!(item, Item::MonWithConsumable { .. }))
                {
                    let delta = item_location.distance(&location);
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
                let delta = item_location.distance(&location);
                if delta < min_danger {
                    min_danger = delta;
                }
            }
            Item::MonWithMana { .. } => {}
        }
    }

    (min_danger, min_mana, angel_nearby)
}

fn distance(location: Location, destination: Destination) -> i32 {
    let distance = match destination {
        Destination::AnyClosestPool => i32::max(
            i32::min(location.i, (Config::MAX_LOCATION_INDEX - location.i).abs()),
            i32::min(location.j, (Config::MAX_LOCATION_INDEX - location.j).abs()),
        ),
        Destination::ClosestPool(color) => {
            let pool_row = if color == Color::White {
                Config::MAX_LOCATION_INDEX
            } else {
                0
            };
            i32::max(
                (pool_row - location.i).abs(),
                i32::min(location.j, (Config::MAX_LOCATION_INDEX - location.j).abs()),
            )
        }
    };
    distance + 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn empty_game() -> MonsGame {
        MonsGame {
            board: Board::new_with_items(HashMap::new()),
            white_score: 0,
            black_score: 0,
            active_color: Color::White,
            actions_used_count: 0,
            mana_moves_count: 0,
            mons_moves_count: 0,
            white_potions_count: 0,
            black_potions_count: 0,
            turn_number: 1,
            takeback_fens: vec![],
            is_moves_verified: true,
            with_verbose_tracking: false,
            verbose_tracking_entities: vec![],
        }
    }

    #[test]
    fn non_drainer_position_no_longer_depends_on_center() {
        let mut center_game = empty_game();
        center_game.board.put(
            Item::Mon {
                mon: Mon::new(MonKind::Demon, Color::White, 0),
            },
            Location::new(5, 5),
        );

        let mut edge_game = empty_game();
        edge_game.board.put(
            Item::Mon {
                mon: Mon::new(MonKind::Demon, Color::White, 0),
            },
            Location::new(0, 0),
        );

        assert_eq!(
            evaluate_preferability(&center_game, Color::White),
            evaluate_preferability(&edge_game, Color::White)
        );
    }

    #[test]
    fn active_mon_off_base_is_preferred() {
        let mon = Mon::new(MonKind::Demon, Color::White, 0);
        let mut on_base = empty_game();
        let base_location = on_base.board.base(mon);
        on_base.board.put(Item::Mon { mon }, base_location);

        let mut off_base = empty_game();
        let off_base_location = if base_location != Location::new(5, 5) {
            Location::new(5, 5)
        } else {
            Location::new(5, 4)
        };
        off_base.board.put(Item::Mon { mon }, off_base_location);

        assert!(
            evaluate_preferability(&off_base, Color::White)
                > evaluate_preferability(&on_base, Color::White)
        );
    }

    #[test]
    fn carrier_closer_to_pool_is_preferred() {
        let mut close_game = empty_game();
        close_game.board.put(
            Item::MonWithMana {
                mon: Mon::new(MonKind::Drainer, Color::White, 0),
                mana: Mana::Regular(Color::White),
            },
            Location::new(9, 9),
        );

        let mut far_game = empty_game();
        far_game.board.put(
            Item::MonWithMana {
                mon: Mon::new(MonKind::Drainer, Color::White, 0),
                mana: Mana::Regular(Color::White),
            },
            Location::new(5, 5),
        );

        assert!(
            evaluate_preferability(&close_game, Color::White)
                > evaluate_preferability(&far_game, Color::White)
        );
    }

    #[test]
    fn regular_mana_closer_to_owners_pool_is_preferred() {
        let mut closer_to_owner_pool = empty_game();
        closer_to_owner_pool.board.put(
            Item::Mana {
                mana: Mana::Regular(Color::White),
            },
            Location::new(9, 0),
        );

        let mut closer_to_opponent_pool = empty_game();
        closer_to_opponent_pool.board.put(
            Item::Mana {
                mana: Mana::Regular(Color::White),
            },
            Location::new(1, 0),
        );

        assert!(
            evaluate_preferability(&closer_to_owner_pool, Color::White)
                > evaluate_preferability(&closer_to_opponent_pool, Color::White)
        );
    }

    #[test]
    fn pressuring_enemy_drainer_increases_preferability() {
        let mut safe_enemy_drainer = empty_game();
        safe_enemy_drainer.board.put(
            Item::Mon {
                mon: Mon::new(MonKind::Drainer, Color::Black, 0),
            },
            Location::new(5, 5),
        );

        let mut pressured_enemy_drainer = safe_enemy_drainer.clone();
        pressured_enemy_drainer.board.put(
            Item::Mon {
                mon: Mon::new(MonKind::Mystic, Color::White, 0),
            },
            Location::new(5, 6),
        );

        assert!(
            evaluate_preferability(&pressured_enemy_drainer, Color::White)
                > evaluate_preferability(&safe_enemy_drainer, Color::White)
        );
    }
}
