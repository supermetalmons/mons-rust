use crate::*;
use crate::models::location::BOARD_CELLS;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

static SQUARES_MAP: LazyLock<HashMap<Location, Square>> = LazyLock::new(Config::build_squares);
static SQUARES_ARRAY: LazyLock<[Square; BOARD_CELLS]> = LazyLock::new(|| {
    let mut arr = [Square::Regular; BOARD_CELLS];
    for (&loc, &sq) in SQUARES_MAP.iter() {
        arr[loc.index()] = sq;
    }
    arr
});
static MONS_BASES_SET: LazyLock<HashSet<Location>> = LazyLock::new(|| {
    Config::MONS_BASE_LOCATIONS.iter().copied().collect()
});
static IS_MON_BASE: LazyLock<[bool; BOARD_CELLS]> = LazyLock::new(|| {
    let mut arr = [false; BOARD_CELLS];
    for loc in &Config::MONS_BASE_LOCATIONS {
        arr[loc.index()] = true;
    }
    arr
});

pub struct Config;

impl Config {
    pub const BOARD_SIZE: i32 = 11;
    pub const TARGET_SCORE: i32 = 5;

    pub const MONS_MOVES_PER_TURN: i32 = 5;
    pub const MANA_MOVES_PER_TURN: i32 = 1;
    pub const ACTIONS_PER_TURN: i32 = 1;

    fn build_squares() -> HashMap<Location, Square> {
        use Square::*;
        let mut squares = HashMap::new();
        squares.insert(
            Location::new(0, 0),
            ManaPool {
                color: Color::Black,
            },
        );
        squares.insert(
            Location::new(0, 10),
            ManaPool {
                color: Color::Black,
            },
        );
        squares.insert(
            Location::new(10, 0),
            ManaPool {
                color: Color::White,
            },
        );
        squares.insert(
            Location::new(10, 10),
            ManaPool {
                color: Color::White,
            },
        );

        squares.insert(
            Location::new(0, 3),
            MonBase {
                kind: MonKind::Mystic,
                color: Color::Black,
            },
        );
        squares.insert(
            Location::new(0, 4),
            MonBase {
                kind: MonKind::Spirit,
                color: Color::Black,
            },
        );
        squares.insert(
            Location::new(0, 5),
            MonBase {
                kind: MonKind::Drainer,
                color: Color::Black,
            },
        );
        squares.insert(
            Location::new(0, 6),
            MonBase {
                kind: MonKind::Angel,
                color: Color::Black,
            },
        );
        squares.insert(
            Location::new(0, 7),
            MonBase {
                kind: MonKind::Demon,
                color: Color::Black,
            },
        );

        squares.insert(
            Location::new(10, 3),
            MonBase {
                kind: MonKind::Demon,
                color: Color::White,
            },
        );
        squares.insert(
            Location::new(10, 4),
            MonBase {
                kind: MonKind::Angel,
                color: Color::White,
            },
        );
        squares.insert(
            Location::new(10, 5),
            MonBase {
                kind: MonKind::Drainer,
                color: Color::White,
            },
        );
        squares.insert(
            Location::new(10, 6),
            MonBase {
                kind: MonKind::Spirit,
                color: Color::White,
            },
        );
        squares.insert(
            Location::new(10, 7),
            MonBase {
                kind: MonKind::Mystic,
                color: Color::White,
            },
        );

        squares.insert(
            Location::new(3, 4),
            ManaBase {
                color: Color::Black,
            },
        );
        squares.insert(
            Location::new(3, 6),
            ManaBase {
                color: Color::Black,
            },
        );
        squares.insert(
            Location::new(7, 4),
            ManaBase {
                color: Color::White,
            },
        );
        squares.insert(
            Location::new(7, 6),
            ManaBase {
                color: Color::White,
            },
        );
        squares.insert(
            Location::new(4, 3),
            ManaBase {
                color: Color::Black,
            },
        );
        squares.insert(
            Location::new(4, 5),
            ManaBase {
                color: Color::Black,
            },
        );
        squares.insert(
            Location::new(4, 7),
            ManaBase {
                color: Color::Black,
            },
        );
        squares.insert(
            Location::new(6, 3),
            ManaBase {
                color: Color::White,
            },
        );
        squares.insert(
            Location::new(6, 5),
            ManaBase {
                color: Color::White,
            },
        );
        squares.insert(
            Location::new(6, 7),
            ManaBase {
                color: Color::White,
            },
        );

        squares.insert(Location::new(5, 0), ConsumableBase);
        squares.insert(Location::new(5, 10), ConsumableBase);
        squares.insert(Location::new(5, 5), SupermanaBase);

        squares
    }

    pub fn squares_ref() -> &'static HashMap<Location, Square> {
        &SQUARES_MAP
    }

    pub fn squares() -> HashMap<Location, Square> {
        Self::squares_ref().clone()
    }

    #[inline]
    pub fn square_at(location: Location) -> Square {
        SQUARES_ARRAY[location.index()]
    }

    pub fn squares_array() -> &'static [Square; BOARD_CELLS] {
        &SQUARES_ARRAY
    }

    pub fn initial_items() -> HashMap<Location, Item> {
        Self::squares_ref()
            .iter()
            .filter_map(|(location, square)| match square {
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
            })
            .collect()
    }

    pub fn initial_items_array() -> [Option<Item>; BOARD_CELLS] {
        let mut arr: [Option<Item>; BOARD_CELLS] = [None; BOARD_CELLS];
        for (&location, square) in SQUARES_MAP.iter() {
            let item = match square {
                Square::MonBase { kind, color } => Some(Item::Mon {
                    mon: Mon::new(*kind, *color, 0),
                }),
                Square::ManaBase { color } => Some(Item::Mana {
                    mana: Mana::Regular(*color),
                }),
                Square::SupermanaBase => Some(Item::Mana {
                    mana: Mana::Supermana,
                }),
                Square::ConsumableBase => Some(Item::Consumable {
                    consumable: Consumable::BombOrPotion,
                }),
                _ => None,
            };
            if let Some(item) = item {
                arr[location.index()] = Some(item);
            }
        }
        arr
    }

    pub const BOARD_CENTER_INDEX: i32 = Self::BOARD_SIZE / 2;
    pub const MAX_LOCATION_INDEX: i32 = Self::BOARD_SIZE - 1;

    pub const SUPERMANA_BASE: Location = Location { i: 5, j: 5 };

    pub const MONS_BASE_LOCATIONS: [Location; 10] = [
        Location { i: 0, j: 3 },
        Location { i: 0, j: 4 },
        Location { i: 0, j: 5 },
        Location { i: 0, j: 6 },
        Location { i: 0, j: 7 },
        Location { i: 10, j: 3 },
        Location { i: 10, j: 4 },
        Location { i: 10, j: 5 },
        Location { i: 10, j: 6 },
        Location { i: 10, j: 7 },
    ];

    pub fn mon_base(kind: MonKind, color: Color) -> Location {
        for &loc in &Self::MONS_BASE_LOCATIONS {
            if let Square::MonBase { kind: k, color: c } = Self::square_at(loc) {
                if k == kind && c == color {
                    return loc;
                }
            }
        }
        panic!("Expected at least one base for the given mon");
    }

    pub fn is_mon_base(location: Location) -> bool {
        IS_MON_BASE[location.index()]
    }

    pub fn mons_bases() -> HashSet<Location> {
        MONS_BASES_SET.clone()
    }

    pub fn mons_bases_ref() -> &'static HashSet<Location> {
        &MONS_BASES_SET
    }
}
