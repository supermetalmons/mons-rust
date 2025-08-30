use crate::models::scoring::evaluate_preferability;
use crate::*;

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct MonsGameModel {
    game: MonsGame,
}

#[wasm_bindgen]
impl MonsGameModel {
    pub fn new() -> MonsGameModel {
        Self {
            game: MonsGame::new(true),
        }
    }

    pub fn from_fen(fen: &str) -> Option<MonsGameModel> {
        if let Some(game) = MonsGame::from_fen(fen, true) {
            Some(Self { game: game })
        } else {
            return None;
        }
    }

    pub fn without_last_turn(&self) -> Option<MonsGameModel> {
        let mut takeback_fens = self.game.takeback_fens.clone();
        let mut verbose_tracking_entities = self.game.verbose_tracking_entities.clone();

        if verbose_tracking_entities.len() <= 1 {
            return None;
        }

        if !takeback_fens.is_empty() {
            takeback_fens.pop();
        }
        verbose_tracking_entities.pop();

        let fen = verbose_tracking_entities
            .last()
            .map(|e| e.fen.clone())
            .unwrap_or_else(|| self.game.fen());

        if let Some(mut new_game) = MonsGame::from_fen(fen.as_str(), true) {
            new_game.takeback_fens = takeback_fens;
            new_game.verbose_tracking_entities = verbose_tracking_entities;
            new_game.with_verbose_tracking = self.game.with_verbose_tracking;
            new_game.is_moves_verified = self.game.is_moves_verified;
            return Some(Self { game: new_game });
        }

        None
    }

    pub fn fen(&self) -> String {
        return self.game.fen();
    }

    pub fn smart_automove(&mut self) -> OutputModel {
        let start_color: Color = self.game.active_color;

        let mut best_game = self.game.clone();
        let mut best_game_output = Self::automove_game(&mut best_game);
        let mut best_game_preferability = evaluate_preferability(&best_game, start_color);

        for _ in 0..30 {
            let mut candidate_game = self.game.clone();
            let candidate_output = Self::automove_game(&mut candidate_game);
            let candidate_preferability = evaluate_preferability(&candidate_game, start_color);

            if candidate_preferability > best_game_preferability {
                best_game = candidate_game;
                best_game_output = candidate_output;
                best_game_preferability = candidate_preferability;
            }
        }

        self.game = best_game;
        return best_game_output;
    }

    pub fn automove(&mut self) -> OutputModel {
        return Self::automove_game(&mut self.game);
    }

    fn automove_game(game: &mut MonsGame) -> OutputModel {
        let mut inputs = Vec::new();
        let mut output = game.process_input(vec![], false, false);

        loop {
            match output {
                Output::InvalidInput => {
                    return OutputModel::new(Output::InvalidInput, "");
                }
                Output::LocationsToStartFrom(locations) => {
                    if locations.is_empty() {
                        return OutputModel::new(Output::InvalidInput, "");
                    }
                    let random_index = random_index(locations.len());
                    let location = locations[random_index];
                    inputs.push(Input::Location(location));
                    output = game.process_input(inputs.clone(), false, false);
                }
                Output::NextInputOptions(options) => {
                    if options.is_empty() {
                        return OutputModel::new(Output::InvalidInput, "");
                    }
                    let random_index = random_index(options.len());
                    let next_input = options[random_index].input.clone();
                    inputs.push(next_input);
                    output = game.process_input(inputs.clone(), false, false);
                }
                Output::Events(events) => {
                    let input_fen = Input::fen_from_array(&inputs);
                    return OutputModel::new(Output::Events(events), input_fen.as_str());
                }
            }
        }
    }

    pub fn process_input(
        &mut self,
        locations: Vec<Location>,
        modifier: Option<Modifier>,
    ) -> OutputModel {
        let mut inputs: Vec<Input> = locations.into_iter().map(Input::Location).collect();
        if let Some(modifier) = modifier {
            inputs.push(Input::Modifier(modifier));
        }
        let input_fen = Input::fen_from_array(&inputs);
        let output = self.game.process_input(inputs, false, false);
        return OutputModel::new(output, input_fen.as_str());
    }

    pub fn can_takeback(&self, color: Color) -> bool {
        return self.game.can_takeback(color);
    }

    pub fn takeback(&mut self) -> OutputModel {
        let inputs: Vec<Input> = vec![Input::Takeback];
        let input_fen = Input::fen_from_array(&inputs);
        let output = self.game.process_input(inputs, false, false);
        return OutputModel::new(output, input_fen.as_str());
    }

    pub fn process_input_fen(&mut self, input_fen: &str) -> OutputModel {
        let inputs = Input::array_from_fen(input_fen);
        let output = self.game.process_input(inputs, false, false);
        return OutputModel::new(output, input_fen);
    }

    pub fn remove_item(&mut self, location: Location) {
        self.game.board.remove_item(location);
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
        if let Some(other_game) = MonsGame::from_fen(other_fen, false) {
            return self.game.is_later_than(&other_game);
        } else {
            return true;
        }
    }

    pub fn is_moves_verified(&self) -> bool {
        return self.game.is_moves_verified;
    }

    pub fn verify_moves(&mut self, flat_moves_string_w: &str, flat_moves_string_b: &str) -> bool {
        let moves_w: Vec<&str> = if flat_moves_string_w.is_empty() {
            Vec::new()
        } else {
            flat_moves_string_w.split("-").collect()
        };
        let moves_b: Vec<&str> = if flat_moves_string_b.is_empty() {
            Vec::new()
        } else {
            flat_moves_string_b.split("-").collect()
        };

        let mut fresh_verification_game = MonsGame::new(true);

        let mut w_index = 0;
        let mut b_index = 0;

        while w_index < moves_w.len() || b_index < moves_b.len() {
            if fresh_verification_game.active_color == Color::White {
                if w_index >= moves_w.len() {
                    return false;
                }
                let inputs = Input::array_from_fen(moves_w[w_index]);
                _ = fresh_verification_game.process_input(inputs, false, false);
                w_index += 1;
            } else {
                if b_index >= moves_b.len() {
                    return false;
                }
                let inputs = Input::array_from_fen(moves_b[b_index]);
                _ = fresh_verification_game.process_input(inputs, false, false);
                b_index += 1;
            }
        }

        if fresh_verification_game.fen() == self.game.fen() {
            self.game.takeback_fens = fresh_verification_game.takeback_fens;
            self.game.verbose_tracking_entities = fresh_verification_game.verbose_tracking_entities;
            self.game.is_moves_verified = true;
            return true;
        } else {
            return false;
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

    pub fn turn_number(&self) -> i32 {
        return self.game.turn_number;
    }

    pub fn inactive_player_items_counters(&self) -> Vec<i32> {
        let player_potions_count = match self.game.active_color.other() {
            Color::White => self.game.white_potions_count,
            Color::Black => self.game.black_potions_count,
        };
        return [0, 0, 0, player_potions_count].to_vec();
    }

    pub fn available_move_kinds(&self) -> Vec<i32> {
        let map = self.game.available_move_kinds();
        return [
            map[&AvailableMoveKind::MonMove],
            map[&AvailableMoveKind::ManaMove],
            map[&AvailableMoveKind::Action],
            map[&AvailableMoveKind::Potion],
        ]
        .to_vec();
    }

    pub fn locations_with_content(&self) -> Vec<Location> {
        let mut locations = self
            .game
            .board
            .items
            .keys()
            .cloned()
            .collect::<Vec<Location>>();
        let mons_bases = self.game.board.all_mons_bases();
        locations.extend(mons_bases);
        locations.sort();
        locations.dedup();
        return locations;
    }

    pub fn verbose_tracking_entities(&self) -> Vec<VerboseTrackingEntityModel> {
        self.game
            .verbose_tracking_entities
            .iter()
            .map(|e| VerboseTrackingEntityModel::new(e))
            .collect()
    }
}

fn random_index(len: usize) -> usize {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    rng.gen_range(0..len)
}

#[wasm_bindgen]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum OutputModelKind {
    InvalidInput,
    LocationsToStartFrom,
    NextInputOptions,
    Events,
}

#[wasm_bindgen]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct OutputModel {
    pub kind: OutputModelKind,
    locations: Vec<Location>,
    next_inputs: Vec<NextInputModel>,
    events: Vec<EventModel>,
    input_fen: String,
}

#[wasm_bindgen]
impl OutputModel {
    pub fn locations(&self) -> Vec<Location> {
        self.locations.clone()
    }

    pub fn next_inputs(&self) -> Vec<NextInputModel> {
        self.next_inputs.clone()
    }

    pub fn events(&self) -> Vec<EventModel> {
        self.events.clone()
    }

    pub fn input_fen(&self) -> String {
        self.input_fen.clone()
    }
}

impl OutputModel {
    fn new(output: Output, input_fen: &str) -> Self {
        match output {
            Output::InvalidInput => Self {
                kind: OutputModelKind::InvalidInput,
                locations: vec![],
                next_inputs: vec![],
                events: vec![],
                input_fen: input_fen.to_string(),
            },
            Output::LocationsToStartFrom(locations) => Self {
                kind: OutputModelKind::LocationsToStartFrom,
                locations,
                next_inputs: vec![],
                events: vec![],
                input_fen: input_fen.to_string(),
            },
            Output::NextInputOptions(next_inputs) => Self {
                kind: OutputModelKind::NextInputOptions,
                locations: vec![],
                next_inputs: next_inputs
                    .into_iter()
                    .map(|input| NextInputModel::new(&input))
                    .collect(),
                events: vec![],
                input_fen: input_fen.to_string(),
            },
            Output::Events(events) => Self {
                kind: OutputModelKind::Events,
                locations: vec![],
                next_inputs: vec![],
                events: events
                    .into_iter()
                    .map(|event| EventModel::new(&event))
                    .collect(),
                input_fen: input_fen.to_string(),
            },
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct SquareModel {
    pub kind: SquareModelKind,
    pub color: Option<Color>,
    pub mon_kind: Option<MonKind>,
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
            Square::Regular => SquareModel {
                kind: SquareModelKind::Regular,
                color: None,
                mon_kind: None,
            },
            Square::ConsumableBase => SquareModel {
                kind: SquareModelKind::ConsumableBase,
                color: None,
                mon_kind: None,
            },
            Square::SupermanaBase => SquareModel {
                kind: SquareModelKind::SupermanaBase,
                color: None,
                mon_kind: None,
            },
            Square::ManaBase { color } => SquareModel {
                kind: SquareModelKind::ManaBase,
                color: Some(*color),
                mon_kind: None,
            },
            Square::ManaPool { color } => SquareModel {
                kind: SquareModelKind::ManaPool,
                color: Some(*color),
                mon_kind: None,
            },
            Square::MonBase { kind, color } => SquareModel {
                kind: SquareModelKind::MonBase,
                color: Some(*color),
                mon_kind: Some(*kind),
            },
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
    pub kind: ItemModelKind,
    pub mon: Option<Mon>,
    pub mana: Option<ManaModel>,
    pub consumable: Option<Consumable>,
}

impl ItemModel {
    fn new(item: &Item) -> Self {
        let (kind, mon, mana, consumable) = match item {
            Item::Mon { mon } => (ItemModelKind::Mon, Some(*mon), None, None),
            Item::Mana { mana } => (ItemModelKind::Mana, None, Some(ManaModel::new(mana)), None),
            Item::MonWithMana { mon, mana } => (
                ItemModelKind::MonWithMana,
                Some(*mon),
                Some(ManaModel::new(mana)),
                None,
            ),
            Item::MonWithConsumable { mon, consumable } => (
                ItemModelKind::MonWithConsumable,
                Some(*mon),
                None,
                Some(*consumable),
            ),
            Item::Consumable { consumable } => {
                (ItemModelKind::Consumable, None, None, Some(*consumable))
            }
        };
        Self {
            kind,
            mon,
            mana,
            consumable,
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ManaModel {
    pub kind: ManaKind,
    pub color: Color,
}

impl ManaModel {
    fn new(item: &Mana) -> Self {
        match item {
            Mana::Regular(color) => ManaModel {
                kind: ManaKind::Regular,
                color: *color,
            },
            Mana::Supermana => ManaModel {
                kind: ManaKind::Supermana,
                color: Color::White,
            },
        }
    }
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
    pub location: Option<Location>,
    pub modifier: Option<Modifier>,
    pub kind: NextInputKind,
    pub actor_mon_item: Option<ItemModel>,
}

impl NextInputModel {
    fn new(input: &NextInput) -> Self {
        Self {
            location: match input.input {
                Input::Location(loc) => Some(loc),
                _ => None,
            },
            modifier: match input.input {
                Input::Modifier(modifier) => Some(modifier),
                _ => None,
            },
            kind: input.kind,
            actor_mon_item: if input.actor_mon_item.is_some() {
                Some(ItemModel::new(&input.actor_mon_item.unwrap()))
            } else {
                None
            },
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum EventModelKind {
    MonMove,
    ManaMove,
    ManaScored,
    MysticAction,
    DemonAction,
    DemonAdditionalStep,
    SpiritTargetMove,
    PickupBomb,
    PickupPotion,
    PickupMana,
    MonFainted,
    ManaDropped,
    SupermanaBackToBase,
    BombAttack,
    MonAwake,
    BombExplosion,
    NextTurn,
    GameOver,
    Takeback,
    UsePotion,
}

#[wasm_bindgen]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct EventModel {
    pub kind: EventModelKind,
    pub item: Option<ItemModel>,
    pub mon: Option<Mon>,
    pub mana: Option<ManaModel>,
    pub loc1: Option<Location>,
    pub loc2: Option<Location>,
    pub color: Option<Color>,
}

impl EventModel {
    fn new(event: &Event) -> Self {
        match event {
            Event::MonMove { item, from, to } => EventModel {
                kind: EventModelKind::MonMove,
                item: Some(ItemModel::new(item)),
                mon: None,
                mana: None,
                loc1: Some(*from),
                loc2: Some(*to),
                color: None,
            },
            Event::ManaMove { mana, from, to } => EventModel {
                kind: EventModelKind::ManaMove,
                item: None,
                mon: None,
                mana: Some(ManaModel::new(mana)),
                loc1: Some(*from),
                loc2: Some(*to),
                color: None,
            },
            Event::ManaScored { mana, at } => EventModel {
                kind: EventModelKind::ManaScored,
                item: None,
                mon: None,
                mana: Some(ManaModel::new(mana)),
                loc1: Some(*at),
                loc2: None,
                color: None,
            },
            Event::MysticAction { mystic, from, to } => EventModel {
                kind: EventModelKind::MysticAction,
                item: None,
                mon: Some(mystic.clone()),
                mana: None,
                loc1: Some(*from),
                loc2: Some(*to),
                color: None,
            },
            Event::DemonAction { demon, from, to } => EventModel {
                kind: EventModelKind::DemonAction,
                item: None,
                mon: Some(demon.clone()),
                mana: None,
                loc1: Some(*from),
                loc2: Some(*to),
                color: None,
            },
            Event::DemonAdditionalStep { demon, from, to } => EventModel {
                kind: EventModelKind::DemonAdditionalStep,
                item: None,
                mon: Some(demon.clone()),
                mana: None,
                loc1: Some(*from),
                loc2: Some(*to),
                color: None,
            },
            Event::SpiritTargetMove { item, from, to } => EventModel {
                kind: EventModelKind::SpiritTargetMove,
                item: Some(ItemModel::new(item)),
                mon: None,
                mana: None,
                loc1: Some(*from),
                loc2: Some(*to),
                color: None,
            },
            Event::PickupBomb { by, at } => EventModel {
                kind: EventModelKind::PickupBomb,
                item: None,
                mon: Some(by.clone()),
                mana: None,
                loc1: Some(*at),
                loc2: None,
                color: None,
            },
            Event::PickupPotion { by, at } => EventModel {
                kind: EventModelKind::PickupPotion,
                item: Some(ItemModel::new(by)),
                mon: None,
                mana: None,
                loc1: Some(*at),
                loc2: None,
                color: None,
            },
            Event::PickupMana { mana, by, at } => EventModel {
                kind: EventModelKind::PickupMana,
                item: None,
                mon: Some(by.clone()),
                mana: Some(ManaModel::new(mana)),
                loc1: Some(*at),
                loc2: None,
                color: None,
            },
            Event::MonFainted { mon, from, to } => EventModel {
                kind: EventModelKind::MonFainted,
                item: None,
                mon: Some(mon.clone()),
                mana: None,
                loc1: Some(*from),
                loc2: Some(*to),
                color: None,
            },
            Event::ManaDropped { mana, at } => EventModel {
                kind: EventModelKind::ManaDropped,
                item: None,
                mon: None,
                mana: Some(ManaModel::new(mana)),
                loc1: Some(*at),
                loc2: None,
                color: None,
            },
            Event::SupermanaBackToBase { from, to } => EventModel {
                kind: EventModelKind::SupermanaBackToBase,
                item: None,
                mon: None,
                mana: None,
                loc1: Some(*from),
                loc2: Some(*to),
                color: None,
            },
            Event::BombAttack { by, from, to } => EventModel {
                kind: EventModelKind::BombAttack,
                item: None,
                mon: Some(by.clone()),
                mana: None,
                loc1: Some(*from),
                loc2: Some(*to),
                color: None,
            },
            Event::MonAwake { mon, at } => EventModel {
                kind: EventModelKind::MonAwake,
                item: None,
                mon: Some(mon.clone()),
                mana: None,
                loc1: Some(*at),
                loc2: None,
                color: None,
            },
            Event::BombExplosion { at } => EventModel {
                kind: EventModelKind::BombExplosion,
                item: None,
                mon: None,
                mana: None,
                loc1: Some(*at),
                loc2: None,
                color: None,
            },
            Event::NextTurn { color } => EventModel {
                kind: EventModelKind::NextTurn,
                item: None,
                mon: None,
                mana: None,
                loc1: None,
                loc2: None,
                color: Some(*color),
            },
            Event::GameOver { winner } => EventModel {
                kind: EventModelKind::GameOver,
                item: None,
                mon: None,
                mana: None,
                loc1: None,
                loc2: None,
                color: Some(*winner),
            },
            Event::Takeback => EventModel {
                kind: EventModelKind::Takeback,
                item: None,
                mon: None,
                mana: None,
                loc1: None,
                loc2: None,
                color: None,
            },
            Event::UsePotion { at } => EventModel {
                kind: EventModelKind::UsePotion,
                item: None,
                mon: None,
                mana: None,
                loc1: Some(*at),
                loc2: None,
                color: None,
            },
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct VerboseTrackingEntityModel {
    fen: String,
    events: Vec<Event>,
}

impl VerboseTrackingEntityModel {
    fn new(entity: &VerboseTrackingEntity) -> Self {
        Self {
            fen: entity.fen.clone(),
            events: entity.events.clone(),
        }
    }
}

#[wasm_bindgen]
impl VerboseTrackingEntityModel {
    pub fn fen(&self) -> String {
        self.fen.clone()
    }
    pub fn events(&self) -> Vec<EventModel> {
        self.events.iter().map(|e| EventModel::new(e)).collect()
    }
}
