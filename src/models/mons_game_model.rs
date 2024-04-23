use crate::*;

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct MonsGameModel {
    game: MonsGame,
}

#[wasm_bindgen]
impl MonsGameModel {
    pub fn from_fen(fen: &str) -> Option<MonsGameModel> {
        if let Some(game) = MonsGame::from_fen(fen) {
            Some(Self {
                game: game,
            })
        } else {
            return None;
        }
    }

    pub fn fen(&self) -> String {
        return self.game.fen();
    }

    pub fn process_input(&mut self, locations: Vec<Location>, modifier: Option<Modifier>) -> OutputModel {
        let mut inputs: Vec<Input> = locations.into_iter().map(Input::Location).collect();
        if let Some(modifier) = modifier {
            inputs.push(Input::Modifier(modifier));
        }
        let input_fen =  Input::fen_from_array(&inputs);
        let output = self.game.process_input(inputs, false, false);
        return OutputModel::new(output, input_fen.as_str());
    }

    pub fn process_input_fen(&mut self, input_fen: &str) -> OutputModel {
        let inputs = Input::array_from_fen(input_fen);
        let output = self.game.process_input(inputs, false, false);
        return OutputModel::new(output, input_fen);
    }

    pub fn item(&self, at: Location) -> Option<ItemModel> {
        if let Some(item) = self.game.board.item(at) {
            return Some(ItemModel::new(item));
        } else {
            return None;
        }        
    }

    pub fn square(&self, at: Location) -> SquareModel {
        let square = self.game.board.square(at);
        return SquareModel::new(&square);
    }

    pub fn is_later_than(&self, other_fen: &str) -> bool {
        if let Some(other_game) = MonsGame::from_fen(other_fen) {
            return self.game.is_later_than(&other_game);
        } else {
            return true;
        }
    }

    pub fn active_color(&self) -> Color {
        return self.game.active_color;
    }

    pub fn winner_color(&self) -> Option<Color> {
        return self.game.winner_color();
    }

    pub fn black_score(&self) -> i32 {
        return self.game.black_score;
    }

    pub fn white_score(&self) -> i32 {
        return self.game.white_score;
    }

    pub fn available_move_kinds(&self) -> Vec<i32> {
        let map = self.game.available_move_kinds();
        return [map[&AvailableMoveKind::MonMove], map[&AvailableMoveKind::ManaMove], map[&AvailableMoveKind::Action], map[&AvailableMoveKind::Potion]].to_vec();
    }

    pub fn locations_with_content(&self) -> Vec<Location> {
        let mut locations = self.game.board.items.keys().cloned().collect::<Vec<Location>>();
        let mons_bases = self.game.board.all_mons_bases();
        locations.extend(mons_bases);
        locations.sort();
        locations.dedup();
        return locations;
    }
}

#[wasm_bindgen]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct OutputModel {
    // TODO: implement
    // TODO: return input_fen here as well to pass it to a peer
}

impl OutputModel {
    fn new(output: Output, input_fen: &str) -> Self {
        // TODO: implement
        Self {
            // TODO: fields to be initialized based on the provided item
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct SquareModel {
    kind: SquareModelKind,
    color: Option<Color>,
    mon_kind: Option<MonKind>,
}

#[wasm_bindgen]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SquareModelKind {
    Regular,
    ConsumableBase,
    SupermanaBase,
    ManaBase,
    ManaPool,
    MonBase,
}

impl SquareModel {
    fn new(item: &Square) -> Self {
        match item {
            Square::Regular => SquareModel { kind: SquareModelKind::Regular, color: None, mon_kind: None },
            Square::ConsumableBase => SquareModel { kind: SquareModelKind::ConsumableBase, color: None, mon_kind: None },
            Square::SupermanaBase => SquareModel { kind: SquareModelKind::SupermanaBase, color: None, mon_kind: None },
            Square::ManaBase { color } => SquareModel { kind: SquareModelKind::ManaBase, color: Some(*color), mon_kind: None },
            Square::ManaPool { color } => SquareModel { kind: SquareModelKind::ManaPool, color: Some(*color), mon_kind: None },
            Square::MonBase { kind, color } => SquareModel { kind: SquareModelKind::MonBase, color: Some(*color), mon_kind: Some(*kind) },
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ItemModelKind {
    Mon,
    Mana,
    MonWithMana,
    MonWithConsumable,
    Consumable,
}

#[wasm_bindgen]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ItemModel {
    kind: ItemModelKind,
    mon: Option<Mon>,
    mana: Option<Mana>,
    consumable: Option<Consumable>,
}

impl ItemModel {
    fn new(item: &Item) -> Self {
        let (kind, mon, mana, consumable) = match item {
            Item::Mon { mon } => (ItemModelKind::Mon, Some(*mon), None, None),
            Item::Mana { mana } => (ItemModelKind::Mana, None, Some(*mana), None),
            Item::MonWithMana { mon, mana } => (ItemModelKind::MonWithMana, Some(*mon), Some(*mana), None),
            Item::MonWithConsumable { mon, consumable } => (ItemModelKind::MonWithConsumable, Some(*mon), None, Some(*consumable)),
            Item::Consumable { consumable } => (ItemModelKind::Consumable, None, None, Some(*consumable)),
        };
        Self { kind, mon, mana, consumable }
    }
}

#[wasm_bindgen]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ManaModel {
    pub kind: ManaKind,
    pub color: Color,
}

#[wasm_bindgen]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ManaKind {
    Regular,
    Supermana,
}

#[wasm_bindgen]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct NextInputModel {
    // TODO: implement
}

#[wasm_bindgen]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct EventModel {
    // TODO: implement
}
