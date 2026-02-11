use crate::*;

const CONFIRMED_SCORE: i32 = 1_500_000;
const POTION_ADVANTAGE: i32 = 25_000;
const FAINTED_MON: i32 = -120_000;
const FAINTED_DRAINER: i32 = -950_000;
const DRAINER_NEAR_MANA: i32 = 140_000;
const DRAINER_AT_RISK: i32 = -420_000;
const DRAINER_PROTECTED_BY_ANGEL: i32 = 110_000;
const CARRIER_PROGRESS: i32 = 420_000;
const DRAINER_CARRIER_BONUS: i32 = 160_000;
const CARRIER_AT_RISK: i32 = -260_000;
const FREE_MANA_ROUTE: i32 = 180_000;
const SUPERMANA_ROUTE_BONUS: i32 = 80_000;
const HAS_CONSUMABLE: i32 = 30_000;
const REGULAR_MANA_TO_OWN_POOL: i32 = 260_000;
const REGULAR_MANA_TO_OPPONENT_POOL: i32 = 180_000;
const REGULAR_MANA_CONTEST: i32 = 90_000;
const SPIRIT_MANA_PRESSURE: i32 = 110_000;
const SPIRIT_IMMEDIATE_SCORE_PRESSURE: i32 = 230_000;
const SPIRIT_POTION_CHAIN_PRESSURE: i32 = 140_000;

pub fn evaluate_preferability(game: &MonsGame, color: Color) -> i32 {
    let mut score = match color {
        Color::White => {
            (game.white_score - game.black_score) * CONFIRMED_SCORE
                + (game.white_potions_count - game.black_potions_count) * POTION_ADVANTAGE
        }
        Color::Black => {
            (game.black_score - game.white_score) * CONFIRMED_SCORE
                + (game.black_potions_count - game.white_potions_count) * POTION_ADVANTAGE
        }
    };

    let drainer_locations = DrainerLocations::from_board(&game.board);

    for (&location, item) in &game.board.items {
        match item {
            Item::Mon { mon } => {
                score += evaluate_mon_state(&game.board, mon, location, color, false);
            }
            Item::MonWithConsumable { mon, .. } => {
                let my_mon_multiplier = mon_multiplier(color, mon.color);
                score += my_mon_multiplier * HAS_CONSUMABLE;
                score += evaluate_mon_state(&game.board, mon, location, color, true);
            }
            Item::Mana { mana } => {
                score += evaluate_free_mana(*mana, location, color, &drainer_locations);
            }
            Item::MonWithMana { mon, mana } => {
                score += evaluate_carrier_state(&game.board, mon, *mana, location, color);
            }
            Item::Consumable { .. } => {}
        }
    }

    score += evaluate_spirit_scoring_pressure(game, color);

    score
}

enum Destination {
    AnyClosestPool,
    ClosestPool(Color),
}

struct DrainerLocations {
    white: Option<Location>,
    black: Option<Location>,
}

impl DrainerLocations {
    fn from_board(board: &Board) -> Self {
        let mut white = None;
        let mut black = None;

        for (&location, item) in &board.items {
            if let Some(mon) = item.mon() {
                if mon.kind == MonKind::Drainer && !mon.is_fainted() {
                    if mon.color == Color::White {
                        white = Some(location);
                    } else {
                        black = Some(location);
                    }
                }
            }
        }

        Self { white, black }
    }

    fn min_distance_to(&self, color: Color, location: Location) -> i32 {
        let drainer = if color == Color::White {
            self.white
        } else {
            self.black
        };

        drainer.map_or(Config::BOARD_SIZE, |drainer_location| {
            drainer_location.distance(&location) + 1
        })
    }
}

fn mon_multiplier(perspective: Color, mon_color: Color) -> i32 {
    if mon_color == perspective {
        1
    } else {
        -1
    }
}

fn evaluate_mon_state(
    board: &Board,
    mon: &Mon,
    location: Location,
    perspective: Color,
    has_consumable: bool,
) -> i32 {
    let my_mon_multiplier = mon_multiplier(perspective, mon.color);

    if mon.is_fainted() {
        return my_mon_multiplier
            * if mon.kind == MonKind::Drainer {
                FAINTED_DRAINER
            } else {
                FAINTED_MON
            };
    }

    if mon.kind == MonKind::Drainer {
        let (danger, min_mana, angel_nearby) = drainer_distances(board, mon.color, location);
        let mut score = 0;
        score += my_mon_multiplier * DRAINER_NEAR_MANA / min_mana.max(1);
        if angel_nearby {
            score += my_mon_multiplier * DRAINER_PROTECTED_BY_ANGEL;
        } else {
            score += my_mon_multiplier * DRAINER_AT_RISK / danger.max(1);
        }

        // Slightly penalize exposed drainers carrying a bomb/potion in place of safer positioning.
        if has_consumable && !angel_nearby {
            score += my_mon_multiplier * (DRAINER_AT_RISK / 2) / danger.max(1);
        }

        // Keep drainer positioning tied to how quickly carried mana can be converted to points.
        let route_to_pool = distance(location, Destination::AnyClosestPool);
        let drainer_route_bias = FREE_MANA_ROUTE / (route_to_pool.max(1) * 6);
        return score + my_mon_multiplier * drainer_route_bias;
    }

    0
}

fn evaluate_carrier_state(
    board: &Board,
    mon: &Mon,
    mana: Mana,
    location: Location,
    perspective: Color,
) -> i32 {
    let my_mon_multiplier = mon_multiplier(perspective, mon.color);
    let carried_score_value = mana.score(mon.color);
    let distance_to_pool = distance(location, Destination::AnyClosestPool).max(1);

    let mut score = my_mon_multiplier * (CARRIER_PROGRESS * carried_score_value) / distance_to_pool;
    if mon.kind == MonKind::Drainer {
        score += my_mon_multiplier * DRAINER_CARRIER_BONUS;
        let (danger, _, angel_nearby) = drainer_distances(board, mon.color, location);
        if angel_nearby {
            score += my_mon_multiplier * DRAINER_PROTECTED_BY_ANGEL;
        } else {
            score += my_mon_multiplier * CARRIER_AT_RISK / danger.max(1);
        }
    } else {
        let danger = general_danger_distance(board, mon.color, location);
        score += my_mon_multiplier * (CARRIER_AT_RISK / 2) / danger.max(1);
    }

    score
}

fn evaluate_free_mana(
    mana: Mana,
    location: Location,
    perspective: Color,
    drainer_locations: &DrainerLocations,
) -> i32 {
    match mana {
        Mana::Regular(mana_color) => {
            let owner_multiplier = mon_multiplier(perspective, mana_color);
            let own_pool_route = distance(location, Destination::ClosestPool(mana_color)).max(1);
            let opponent_pool_route =
                distance(location, Destination::ClosestPool(mana_color.other())).max(1);

            let owner_drainer_route =
                drainer_locations.min_distance_to(mana_color, location) + own_pool_route;
            let opponent_drainer_route = drainer_locations
                .min_distance_to(mana_color.other(), location)
                + opponent_pool_route;

            owner_multiplier * REGULAR_MANA_TO_OWN_POOL / own_pool_route
                - owner_multiplier * REGULAR_MANA_TO_OPPONENT_POOL / opponent_pool_route
                + owner_multiplier * REGULAR_MANA_CONTEST / owner_drainer_route.max(1)
                - owner_multiplier * (REGULAR_MANA_CONTEST * 2) / opponent_drainer_route.max(1)
        }
        Mana::Supermana => {
            let my_color = perspective;
            let opponent_color = perspective.other();

            let route_to_pool = distance(location, Destination::AnyClosestPool);
            let my_route = drainer_locations.min_distance_to(my_color, location) + route_to_pool;
            let opponent_route =
                drainer_locations.min_distance_to(opponent_color, location) + route_to_pool;

            let mut score =
                FREE_MANA_ROUTE * 2 / my_route.max(1) - FREE_MANA_ROUTE * 2 / opponent_route.max(1);
            score += SUPERMANA_ROUTE_BONUS / my_route.max(1);
            score -= SUPERMANA_ROUTE_BONUS / opponent_route.max(1);
            score
        }
    }
}

fn evaluate_spirit_scoring_pressure(game: &MonsGame, perspective: Color) -> i32 {
    let spirit_locations = SpiritLocations::from_board(&game.board);
    let mut total = 0;

    for color in [Color::White, Color::Black] {
        let Some(spirit_location) = spirit_locations.location(color) else {
            continue;
        };

        let side_multiplier = mon_multiplier(perspective, color);
        let actions_budget = spirit_action_budget(game, color);
        let move_budget = spirit_move_budget(game, color);
        if actions_budget <= 0 {
            continue;
        }

        let base_setup_penalty =
            if matches!(game.board.square(spirit_location), Square::MonBase { .. }) {
                1
            } else {
                0
            };

        for (&target_location, item) in &game.board.items {
            let target_mana = match item {
                Item::Mana { mana } | Item::MonWithMana { mana, .. } => Some(*mana),
                _ => None,
            };
            let Some(mana) = target_mana else {
                continue;
            };

            let pool_distance = distance(target_location, Destination::AnyClosestPool).max(1);
            let spirit_setup_distance =
                (spirit_location.distance(&target_location) - 2).abs() + base_setup_penalty;
            if spirit_setup_distance > move_budget + 1 {
                continue;
            }
            let score_value = mana.score(color).max(1);
            let base_denominator = (pool_distance + spirit_setup_distance + 1).max(1);
            let can_setup_this_turn = move_budget >= spirit_setup_distance;

            let mut pressure = SPIRIT_MANA_PRESSURE * score_value / base_denominator;

            // If spirit can target now and has enough action budget this turn, this is near-forced score pressure.
            if spirit_setup_distance == 0 && actions_budget >= pool_distance && can_setup_this_turn
            {
                pressure += SPIRIT_IMMEDIATE_SCORE_PRESSURE * score_value / pool_distance.max(1);
            }

            // Potions let spirit chain extra pushes in the same turn; this is especially dangerous near pools.
            if actions_budget >= 2
                && pool_distance <= 2
                && spirit_setup_distance <= 1
                && can_setup_this_turn
            {
                let extra_actions = (actions_budget - 1).min(2);
                pressure +=
                    SPIRIT_POTION_CHAIN_PRESSURE * extra_actions * score_value / base_denominator;
            }

            if !can_setup_this_turn {
                pressure /= 4;
            }

            total += side_multiplier * pressure;
        }
    }

    total
}

struct SpiritLocations {
    white: Option<Location>,
    black: Option<Location>,
}

impl SpiritLocations {
    fn from_board(board: &Board) -> Self {
        let mut white = None;
        let mut black = None;

        for (&location, item) in &board.items {
            if let Item::Mon { mon } = item {
                if mon.kind == MonKind::Spirit && !mon.is_fainted() {
                    if mon.color == Color::White {
                        white = Some(location);
                    } else {
                        black = Some(location);
                    }
                }
            }
        }

        Self { white, black }
    }

    fn location(&self, color: Color) -> Option<Location> {
        match color {
            Color::White => self.white,
            Color::Black => self.black,
        }
    }
}

fn spirit_action_budget(game: &MonsGame, color: Color) -> i32 {
    let potions = color_potions_count(game, color).max(0);
    let base_actions = if game.active_color == color {
        if game.is_first_turn() {
            0
        } else if game.actions_used_count < Config::ACTIONS_PER_TURN {
            1
        } else {
            0
        }
    } else {
        1
    };

    (base_actions + potions).clamp(0, 4)
}

fn spirit_move_budget(game: &MonsGame, color: Color) -> i32 {
    if game.active_color == color {
        (Config::MONS_MOVES_PER_TURN - game.mons_moves_count).max(0)
    } else {
        Config::MONS_MOVES_PER_TURN
    }
}

fn color_potions_count(game: &MonsGame, color: Color) -> i32 {
    if color == Color::White {
        game.white_potions_count
    } else {
        game.black_potions_count
    }
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
                        || mon.kind == MonKind::Spirit
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

fn distance(location: Location, destination: Destination) -> i32 {
    let distance = match destination {
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

fn general_danger_distance(board: &Board, color: Color, location: Location) -> i32 {
    let mut min_danger = Config::BOARD_SIZE as i32;

    for (&item_location, item) in &board.items {
        match item {
            Item::Mon { mon } | Item::MonWithConsumable { mon, .. } => {
                if mon.color != color
                    && !mon.is_fainted()
                    && (mon.kind == MonKind::Mystic
                        || mon.kind == MonKind::Demon
                        || mon.kind == MonKind::Spirit
                        || matches!(item, Item::MonWithConsumable { .. }))
                {
                    min_danger = min_danger.min(item_location.distance(&location) as i32);
                }
            }
            Item::Consumable { .. } => {
                min_danger = min_danger.min(item_location.distance(&location) as i32);
            }
            Item::Mana { .. } | Item::MonWithMana { .. } => {}
        }
    }

    min_danger.max(1)
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

    #[test]
    fn spirit_near_pool_mana_increases_scoring_pressure() {
        let mut spirit_can_score = empty_game();
        spirit_can_score.turn_number = 2;
        spirit_can_score.active_color = Color::White;
        spirit_can_score.board.put(
            Item::Mon {
                mon: Mon::new(MonKind::Spirit, Color::White, 0),
            },
            Location::new(7, 0),
        );
        spirit_can_score.board.put(
            Item::Mana {
                mana: Mana::Regular(Color::Black),
            },
            Location::new(9, 0),
        );

        let mut spirit_far = spirit_can_score.clone();
        spirit_far.board.remove_item(Location::new(7, 0));
        spirit_far.board.put(
            Item::Mon {
                mon: Mon::new(MonKind::Spirit, Color::White, 0),
            },
            Location::new(0, 5),
        );

        assert!(
            evaluate_preferability(&spirit_can_score, Color::White)
                > evaluate_preferability(&spirit_far, Color::White)
        );
    }

    #[test]
    fn enemy_spirit_with_potion_is_more_dangerous_for_near_pool_mana() {
        let mut enemy_without_potion = empty_game();
        enemy_without_potion.turn_number = 2;
        enemy_without_potion.active_color = Color::Black;
        enemy_without_potion.board.put(
            Item::Mon {
                mon: Mon::new(MonKind::Spirit, Color::Black, 0),
            },
            Location::new(4, 0),
        );
        enemy_without_potion.board.put(
            Item::Mana {
                mana: Mana::Regular(Color::White),
            },
            Location::new(2, 0),
        );

        let mut enemy_with_potion = enemy_without_potion.clone();
        enemy_with_potion.black_potions_count = 1;

        assert!(
            evaluate_preferability(&enemy_with_potion, Color::White)
                < evaluate_preferability(&enemy_without_potion, Color::White)
        );
    }

    #[test]
    fn spirit_on_base_has_less_immediate_pressure_than_off_base() {
        let mut spirit_on_base = empty_game();
        spirit_on_base.turn_number = 2;
        spirit_on_base.active_color = Color::White;
        spirit_on_base.board.put(
            Item::Mon {
                mon: Mon::new(MonKind::Spirit, Color::White, 0),
            },
            Location::new(10, 6),
        );
        spirit_on_base.board.put(
            Item::Mana {
                mana: Mana::Regular(Color::Black),
            },
            Location::new(9, 0),
        );

        let mut spirit_off_base = spirit_on_base.clone();
        spirit_off_base.board.remove_item(Location::new(10, 6));
        spirit_off_base.board.put(
            Item::Mon {
                mon: Mon::new(MonKind::Spirit, Color::White, 0),
            },
            Location::new(8, 1),
        );

        assert!(
            evaluate_preferability(&spirit_off_base, Color::White)
                > evaluate_preferability(&spirit_on_base, Color::White)
        );
    }

    #[test]
    fn exhausted_mon_moves_reduce_active_spirit_pressure() {
        let mut can_move = empty_game();
        can_move.turn_number = 2;
        can_move.active_color = Color::White;
        can_move.board.put(
            Item::Mon {
                mon: Mon::new(MonKind::Spirit, Color::White, 0),
            },
            Location::new(10, 6),
        );
        can_move.board.put(
            Item::Mana {
                mana: Mana::Regular(Color::Black),
            },
            Location::new(9, 0),
        );

        let mut no_moves_left = can_move.clone();
        no_moves_left.mons_moves_count = Config::MONS_MOVES_PER_TURN;

        assert!(
            evaluate_preferability(&can_move, Color::White)
                > evaluate_preferability(&no_moves_left, Color::White)
        );
    }
}
