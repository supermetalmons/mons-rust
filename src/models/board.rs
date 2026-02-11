use crate::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Board {
    pub items: std::collections::HashMap<Location, Item>,
}

impl Board {
    pub fn new() -> Self {
        Self {
            items: Config::initial_items(),
        }
    }

    pub fn new_with_items(items: std::collections::HashMap<Location, Item>) -> Self {
        Self { items }
    }

    pub fn remove_item(&mut self, location: Location) {
        self.items.remove(&location);
    }

    pub fn put(&mut self, item: Item, location: Location) {
        self.items.insert(location, item);
    }

    pub fn item(&self, location: Location) -> Option<&Item> {
        self.items.get(&location)
    }

    pub fn square(&self, location: Location) -> Square {
        *Config::squares_ref()
            .get(&location)
            .unwrap_or(&Square::Regular)
    }

    pub fn all_mons_bases(&self) -> Vec<Location> {
        Config::squares_ref()
            .iter()
            .filter_map(|(location, square)| {
                if let Square::MonBase { .. } = square {
                    Some(*location)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn supermana_base(&self) -> Location {
        *Config::squares_ref()
            .iter()
            .find(|(_, square)| matches!(square, Square::SupermanaBase))
            .expect("Expected at least one supermana base")
            .0
    }

    pub fn all_mons_locations(&self, color: Color) -> Vec<Location> {
        self.items
            .iter()
            .filter_map(|(location, item)| {
                if let Some(mon) = item.mon() {
                    if mon.color == color {
                        Some(*location)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn all_free_regular_mana_locations(&self, color: Color) -> Vec<Location> {
        self.items
            .iter()
            .filter_map(|(location, item)| match item {
                Item::Mana { mana } => match mana {
                    Mana::Regular(mana_color) if *mana_color == color => Some(*location),
                    _ => None,
                },
                _ => None,
            })
            .collect()
    }

    pub fn base(&self, mon: Mon) -> Location {
        *Config::squares_ref()
            .iter()
            .find(|(_, square)| matches!(square, Square::MonBase { kind, color } if kind == &mon.kind && color == &mon.color))
            .expect("Expected at least one base for the given mon")
            .0
    }

    pub fn fainted_mons_locations(&self, color: Color) -> Vec<Location> {
        self.items
            .iter()
            .filter_map(|(location, item)| match item {
                Item::Mon { mon } if mon.color == color && mon.is_fainted() => Some(*location),
                _ => None,
            })
            .collect()
    }

    pub fn find_mana(&self, color: Color) -> Option<Location> {
        self.items.iter().find_map(|(location, item)| match item {
            Item::Mana { mana } => match mana {
                Mana::Regular(mana_color) if *mana_color == color => Some(*location),
                _ => None,
            },
            _ => None,
        })
    }

    pub fn find_awake_angel(&self, color: Color) -> Option<Location> {
        self.items.iter().find_map(|(location, item)| {
            if let Some(mon) = item.mon() {
                if mon.color == color && mon.kind == MonKind::Angel && !mon.is_fainted() {
                    Some(*location)
                } else {
                    None
                }
            } else {
                None
            }
        })
    }
}
