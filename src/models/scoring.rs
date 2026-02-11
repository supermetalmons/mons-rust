use crate::*;

pub fn evaluate_preferability(game: &MonsGame, color: Color) -> i32 {
    struct Multiplier;
    impl Multiplier {
        const CONFIRMED_SCORE: i32 = 1000;
        const FAINTED_MON: i32 = -500;
        const FAINTED_DRAINER: i32 = -800;
        const DRAINER_AT_RISK: i32 = -350;
        const MANA_CLOSE_TO_SAME_POOL: i32 = 500;
        const MON_WITH_MANA_CLOSE_TO_ANY_POOL: i32 = 800;
        const EXTRA_FOR_SUPERMANA: i32 = 120;
        const EXTRA_FOR_OPPONENTS_MANA: i32 = 100;
        const DRAINER_CLOSE_TO_MANA: i32 = 300;
        const DRAINER_HOLDING_MANA: i32 = 350;
        const MON_CLOSE_TO_CENTER: i32 = 210;
        const HAS_CONSUMABLE: i32 = 110;
        const ACTIVE_MON: i32 = 50;
    }

    let mons_bases = Config::mons_bases_ref();

    let mut score = match color {
        Color::White => {
            (game.white_score - game.black_score) * Multiplier::CONFIRMED_SCORE
                + (game.white_potions_count - game.black_potions_count) * Multiplier::HAS_CONSUMABLE
        }
        Color::Black => {
            (game.black_score - game.white_score) * Multiplier::CONFIRMED_SCORE
                + (game.black_potions_count - game.white_potions_count) * Multiplier::HAS_CONSUMABLE
        }
    };

    score *= Multiplier::CONFIRMED_SCORE;

    for (&location, item) in &game.board.items {
        match item {
            Item::Mon { mon } => {
                let my_mon_multiplier = if mon.color == color { 1 } else { -1 };
                let is_drainer = mon.kind == MonKind::Drainer;

                if mon.is_fainted() {
                    score += my_mon_multiplier
                        * (if is_drainer {
                            Multiplier::FAINTED_DRAINER
                        } else {
                            Multiplier::FAINTED_MON
                        });
                } else if is_drainer {
                    let (danger, min_mana, angel_nearby) =
                        drainer_distances(&game.board, mon.color, location);
                    score += my_mon_multiplier * Multiplier::DRAINER_CLOSE_TO_MANA / min_mana;
                    if !angel_nearby {
                        score += my_mon_multiplier * Multiplier::DRAINER_AT_RISK / danger;
                    }
                } else if mon.kind != MonKind::Angel {
                    score += my_mon_multiplier * Multiplier::MON_CLOSE_TO_CENTER
                        / distance(location, Destination::Center);
                }

                if !mons_bases.contains(&location) {
                    score += my_mon_multiplier * Multiplier::ACTIVE_MON;
                }
            }
            Item::MonWithConsumable { mon, .. } => {
                let my_mon_multiplier = if mon.color == color { 1 } else { -1 };
                let is_drainer = mon.kind == MonKind::Drainer;
                score += my_mon_multiplier * Multiplier::HAS_CONSUMABLE;

                if is_drainer {
                    let (danger, min_mana, angel_nearby) =
                        drainer_distances(&game.board, mon.color, location);
                    score += my_mon_multiplier * Multiplier::DRAINER_CLOSE_TO_MANA / min_mana;
                    if !angel_nearby {
                        score += my_mon_multiplier * Multiplier::DRAINER_AT_RISK / danger;
                    }
                } else if mon.kind != MonKind::Angel {
                    score += my_mon_multiplier * Multiplier::MON_CLOSE_TO_CENTER
                        / distance(location, Destination::Center);
                }
            }
            Item::Mana { .. } => {
                score += Multiplier::MANA_CLOSE_TO_SAME_POOL
                    / distance(location, Destination::ClosestPool(color));
            }
            Item::MonWithMana { mon, mana } => {
                let my_mon_multiplier = if mon.color == color { 1 } else { -1 };
                let mana_extra = match mana {
                    Mana::Regular(mana_color) => {
                        if *mana_color == color {
                            0
                        } else {
                            Multiplier::EXTRA_FOR_OPPONENTS_MANA
                        }
                    }
                    Mana::Supermana => Multiplier::EXTRA_FOR_SUPERMANA,
                };

                score += my_mon_multiplier * Multiplier::DRAINER_HOLDING_MANA;
                score += my_mon_multiplier
                    * (Multiplier::MON_WITH_MANA_CLOSE_TO_ANY_POOL + mana_extra)
                    / distance(location, Destination::AnyClosestPool);
            }
            Item::Consumable { .. } => {}
        }
    }

    score
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

fn distance(location: Location, destination: Destination) -> i32 {
    let distance = match destination {
        Destination::Center => (Config::BOARD_CENTER_INDEX as i32 - location.i as i32).abs(),
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
