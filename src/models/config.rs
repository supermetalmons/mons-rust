use crate::*;
use std::collections::{HashMap, HashSet};

pub struct Config;

impl Config {
    pub const BOARD_SIZE: i32 = 11;
    pub const TARGET_SCORE: i32 = 5;

    pub const MONS_MOVES_PER_TURN: i32 = 5;
    pub const MANA_MOVES_PER_TURN: i32 = 1;
    pub const ACTIONS_PER_TURN: i32 = 1;

    pub fn squares() -> std::collections::HashMap<Location, Square> {
        use Square::*;
        let mut squares = std::collections::HashMap::new();
        squares.insert(Location::new(0, 0), ManaPool { color: Color::Black });
        squares.insert(Location::new(0, 10), ManaPool { color: Color::Black });
        squares.insert(Location::new(10, 0), ManaPool { color: Color::White });
        squares.insert(Location::new(10, 10), ManaPool { color: Color::White });

        squares.insert(Location::new(0, 3), MonBase { kind: MonKind::Mystic, color: Color::Black });
        squares.insert(Location::new(0, 4), MonBase { kind: MonKind::Spirit, color: Color::Black });
        squares.insert(Location::new(0, 5), MonBase { kind: MonKind::Drainer, color: Color::Black });
        squares.insert(Location::new(0, 6), MonBase { kind: MonKind::Angel, color: Color::Black });
        squares.insert(Location::new(0, 7), MonBase { kind: MonKind::Demon, color: Color::Black });
        
        squares.insert(Location::new(10, 3), MonBase { kind: MonKind::Demon, color: Color::White });
        squares.insert(Location::new(10, 4), MonBase { kind: MonKind::Angel, color: Color::White });
        squares.insert(Location::new(10, 5), MonBase { kind: MonKind::Drainer, color: Color::White });
        squares.insert(Location::new(10, 6), MonBase { kind: MonKind::Spirit, color: Color::White });
        squares.insert(Location::new(10, 7), MonBase { kind: MonKind::Mystic, color: Color::White });

        squares.insert(Location::new(3, 4), ManaBase { color: Color::Black });
        squares.insert(Location::new(3, 6), ManaBase { color: Color::Black });
        squares.insert(Location::new(7, 4), ManaBase { color: Color::White });
        squares.insert(Location::new(7, 6), ManaBase { color: Color::White });
        squares.insert(Location::new(4, 3), ManaBase { color: Color::Black });
        squares.insert(Location::new(4, 5), ManaBase { color: Color::Black });
        squares.insert(Location::new(4, 7), ManaBase { color: Color::Black });
        squares.insert(Location::new(6, 3), ManaBase { color: Color::White });
        squares.insert(Location::new(6, 5), ManaBase { color: Color::White });
        squares.insert(Location::new(6, 7), ManaBase { color: Color::White });

        squares.insert(Location::new(5, 0), ConsumableBase);
        squares.insert(Location::new(5, 10), ConsumableBase);
        squares.insert(Location::new(5, 5), SupermanaBase);

        squares
    }

    pub fn initial_items() -> HashMap<Location, Item> {
        Self::squares().iter().filter_map(|(location, square)| {
            match square {
                Square::MonBase { kind, color } => Some((
                    *location,
                    Item::Mon {
                        mon: Mon::new(*kind, *color, 0),
                    },
                )),
                Square::ManaBase { color } => Some((
                    *location,
                    Item::Mana {
                        mana: Mana::Regular(*color),
                    },
                )),
                Square::SupermanaBase => Some((
                    *location,
                    Item::Mana {
                        mana: Mana::Supermana,
                    },
                )),
                Square::ConsumableBase => Some((
                    *location,
                    Item::Consumable {
                        consumable: Consumable::BombOrPotion,
                    },
                )),
                _ => None,
            }
        }).collect()
    }

    pub const BOARD_CENTER_INDEX: i32 = Self::BOARD_SIZE / 2;
    pub const MAX_LOCATION_INDEX: i32 = Self::BOARD_SIZE - 1;

    pub fn mons_bases() -> HashSet<Location> {
        Self::squares().iter().filter_map(|(location, square)| {
            match square {
                Square::MonBase { .. } => Some(*location),
                _ => None,
            }
        }).collect()
    }
}
