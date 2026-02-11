use crate::*;

#[wasm_bindgen]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
pub struct Location {
    pub i: i32,
    pub j: i32,
}

#[wasm_bindgen]
impl Location {
    #[wasm_bindgen(constructor)]
    pub fn new(i: i32, j: i32) -> Self {
        Self { i, j }
    }
}

impl Location {
    pub fn valid_range() -> std::ops::Range<i32> {
        0..Config::BOARD_SIZE
    }

    pub fn nearby_locations(&self) -> Vec<Location> {
        self.nearby_locations_with_distance(1)
    }

    pub fn reachable_by_bomb(&self) -> Vec<Location> {
        self.nearby_locations_with_distance(3)
    }

    pub fn reachable_by_mystic_action(&self) -> Vec<Location> {
        let deltas = [(-2, -2), (2, 2), (-2, 2), (2, -2)];
        deltas
            .iter()
            .filter_map(|&(dx, dy)| {
                let (new_i, new_j) = (self.i + dx, self.j + dy);
                if Self::valid_range().contains(&new_i) && Self::valid_range().contains(&new_j) {
                    Some(Location::new(new_i, new_j))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn reachable_by_demon_action(&self) -> Vec<Location> {
        let deltas = [(-2, 0), (2, 0), (0, 2), (0, -2)];
        deltas
            .iter()
            .filter_map(|&(dx, dy)| {
                let (new_i, new_j) = (self.i + dx, self.j + dy);
                if Self::valid_range().contains(&new_i) && Self::valid_range().contains(&new_j) {
                    Some(Location::new(new_i, new_j))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn reachable_by_spirit_action(&self) -> Vec<Location> {
        let mut locations = Vec::new();
        for x in -2i32..=2 {
            for y in -2i32..=2 {
                if x.abs().max(y.abs()) == 2
                    && Self::valid_range().contains(&(self.i + x))
                    && Self::valid_range().contains(&(self.j + y))
                {
                    locations.push(Location::new(self.i + x, self.j + y));
                }
            }
        }
        locations
    }

    pub fn location_between(&self, another: &Location) -> Location {
        Location::new((self.i + another.i) / 2, (self.j + another.j) / 2)
    }

    pub fn distance(&self, to: &Location) -> i32 {
        ((to.i - self.i).abs()).max((to.j - self.j).abs())
    }

    fn nearby_locations_with_distance(&self, distance: i32) -> Vec<Location> {
        let mut locations = Vec::new();
        for x in (self.i - distance)..=(self.i + distance) {
            for y in (self.j - distance)..=(self.j + distance) {
                if Self::valid_range().contains(&x)
                    && Self::valid_range().contains(&y)
                    && (x != self.i || y != self.j)
                {
                    locations.push(Location::new(x, y));
                }
            }
        }
        locations
    }
}
