use crate::models::scoring::evaluate_preferability;
use crate::*;

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct MonsGameModel {
    game: MonsGame,
}

const DEFAULT_SMART_SEARCH_DEPTH: usize = 2;
const DEFAULT_SMART_MAX_VISITED_NODES: usize = 320;
const MIN_SMART_SEARCH_DEPTH: usize = 1;
const MAX_SMART_SEARCH_DEPTH: usize = 4;
const MIN_SMART_MAX_VISITED_NODES: usize = 32;
const MAX_SMART_MAX_VISITED_NODES: usize = 20_000;
const SMART_TERMINAL_SCORE: i32 = i32::MAX / 8;
const SMART_MAX_INPUT_CHAIN: usize = 8;

#[derive(Clone, Copy)]
struct SmartSearchConfig {
    depth: usize,
    max_visited_nodes: usize,
    root_enum_limit: usize,
    root_branch_limit: usize,
    node_enum_limit: usize,
    node_branch_limit: usize,
}

impl SmartSearchConfig {
    fn from_budget(depth: i32, max_visited_nodes: i32) -> Self {
        let depth =
            depth.clamp(MIN_SMART_SEARCH_DEPTH as i32, MAX_SMART_SEARCH_DEPTH as i32) as usize;
        let max_visited_nodes = max_visited_nodes.clamp(
            MIN_SMART_MAX_VISITED_NODES as i32,
            MAX_SMART_MAX_VISITED_NODES as i32,
        ) as usize;

        let root_branch_limit = (max_visited_nodes / 24).clamp(4, 28);
        let node_branch_limit = (max_visited_nodes / 40).clamp(4, 18);
        let root_enum_limit = (root_branch_limit * 5).clamp(root_branch_limit, 180);
        let node_enum_limit = (node_branch_limit * 3).clamp(node_branch_limit, 96);

        Self {
            depth,
            max_visited_nodes,
            root_enum_limit,
            root_branch_limit,
            node_enum_limit,
            node_branch_limit,
        }
    }
}

struct ScoredRootMove {
    inputs: Vec<Input>,
    game: MonsGame,
    heuristic: i32,
}

#[wasm_bindgen]
impl MonsGameModel {
    pub fn new() -> MonsGameModel {
        Self {
            game: MonsGame::new(true),
        }
    }

    #[wasm_bindgen(js_name = newForSimulation)]
    pub fn new_for_simulation() -> MonsGameModel {
        Self {
            game: MonsGame::new(false),
        }
    }

    pub fn from_fen(fen: &str) -> Option<MonsGameModel> {
        if let Some(game) = MonsGame::from_fen(fen, true) {
            Some(Self { game: game })
        } else {
            return None;
        }
    }

    #[wasm_bindgen(js_name = fromFenForSimulation)]
    pub fn from_fen_for_simulation(fen: &str) -> Option<MonsGameModel> {
        MonsGame::from_fen(fen, false).map(|game| Self { game })
    }

    pub fn without_last_turn(&self, takeback_fens: Vec<String>) -> Option<MonsGameModel> {
        let mut verbose_tracking_entities = self.game.verbose_tracking_entities.clone();

        if verbose_tracking_entities.len() <= 1 {
            return None;
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
        self.smart_automove_with_budget(
            DEFAULT_SMART_SEARCH_DEPTH as i32,
            DEFAULT_SMART_MAX_VISITED_NODES as i32,
        )
    }

    #[wasm_bindgen(js_name = smartAutomoveWithBudget)]
    pub fn smart_automove_with_budget(
        &mut self,
        depth: i32,
        max_visited_nodes: i32,
    ) -> OutputModel {
        let config = SmartSearchConfig::from_budget(depth, max_visited_nodes);
        if let Some(best_inputs) = Self::best_smart_inputs(&self.game, config) {
            let input_fen = Input::fen_from_array(&best_inputs);
            let output = self.game.process_input(best_inputs, false, false);
            OutputModel::new(output, input_fen.as_str())
        } else {
            Self::automove_game(&mut self.game)
        }
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

    #[wasm_bindgen(js_name = setVerboseTracking)]
    pub fn set_verbose_tracking(&mut self, enabled: bool) {
        self.game.set_verbose_tracking(enabled);
    }

    #[wasm_bindgen(js_name = clearTracking)]
    pub fn clear_tracking(&mut self) {
        self.game.clear_tracking();
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

        let with_verbose_tracking = self.game.with_verbose_tracking;
        let mut fresh_verification_game = MonsGame::new(with_verbose_tracking);

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
            if with_verbose_tracking {
                self.game.verbose_tracking_entities =
                    fresh_verification_game.verbose_tracking_entities;
            } else {
                self.game.verbose_tracking_entities.clear();
                self.game.verbose_tracking_entities.shrink_to_fit();
            }
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

    pub fn takeback_fens(&self) -> Vec<String> {
        self.game.takeback_fens.clone()
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

impl MonsGameModel {
    fn best_smart_inputs(game: &MonsGame, config: SmartSearchConfig) -> Option<Vec<Input>> {
        let perspective = game.active_color;
        let mut visited_nodes = 0usize;
        let mut alpha = i32::MIN;
        let beta = i32::MAX;

        let root_moves = Self::ranked_root_moves(game, perspective, config);
        if root_moves.is_empty() {
            return None;
        }

        let mut best_score = i32::MIN;
        let mut best_inputs: Option<Vec<Input>> = None;

        for candidate in root_moves {
            if visited_nodes >= config.max_visited_nodes {
                break;
            }

            visited_nodes += 1;
            let candidate_score = if config.depth > 1 {
                Self::search_score(
                    &candidate.game,
                    perspective,
                    config.depth - 1,
                    alpha,
                    beta,
                    &mut visited_nodes,
                    config,
                )
            } else {
                candidate.heuristic
            };

            if best_inputs.is_none() || candidate_score > best_score {
                best_score = candidate_score;
                best_inputs = Some(candidate.inputs);
            }

            if candidate_score > alpha {
                alpha = candidate_score;
            }
        }

        best_inputs
    }

    fn ranked_root_moves(
        game: &MonsGame,
        perspective: Color,
        config: SmartSearchConfig,
    ) -> Vec<ScoredRootMove> {
        let mut candidates = Vec::new();

        for inputs in Self::enumerate_legal_inputs(game, config.root_enum_limit) {
            if let Some(simulated_game) = Self::apply_inputs_for_search(game, &inputs) {
                let heuristic = Self::score_state(
                    &simulated_game,
                    perspective,
                    config.depth.saturating_sub(1),
                    config.depth,
                );
                candidates.push(ScoredRootMove {
                    inputs,
                    game: simulated_game,
                    heuristic,
                });
            }
        }

        candidates.sort_by(|a, b| b.heuristic.cmp(&a.heuristic));
        if candidates.len() > config.root_branch_limit {
            candidates.truncate(config.root_branch_limit);
        }
        candidates
    }

    fn search_score(
        game: &MonsGame,
        perspective: Color,
        depth: usize,
        mut alpha: i32,
        mut beta: i32,
        visited_nodes: &mut usize,
        config: SmartSearchConfig,
    ) -> i32 {
        if let Some(terminal_score) = Self::terminal_score(game, perspective, depth, config.depth) {
            return terminal_score;
        }
        if depth == 0 || *visited_nodes >= config.max_visited_nodes {
            return evaluate_preferability(game, perspective);
        }

        let maximizing = game.active_color == perspective;
        let mut children = Self::ranked_child_states(game, perspective, maximizing, config);
        if children.is_empty() {
            return evaluate_preferability(game, perspective);
        }

        if maximizing {
            let mut value = i32::MIN;
            for child in children.drain(..) {
                if *visited_nodes >= config.max_visited_nodes {
                    break;
                }

                *visited_nodes += 1;
                let score = Self::search_score(
                    &child,
                    perspective,
                    depth - 1,
                    alpha,
                    beta,
                    visited_nodes,
                    config,
                );
                value = value.max(score);
                alpha = alpha.max(value);
                if alpha >= beta {
                    break;
                }
            }
            if value == i32::MIN {
                evaluate_preferability(game, perspective)
            } else {
                value
            }
        } else {
            let mut value = i32::MAX;
            for child in children.drain(..) {
                if *visited_nodes >= config.max_visited_nodes {
                    break;
                }

                *visited_nodes += 1;
                let score = Self::search_score(
                    &child,
                    perspective,
                    depth - 1,
                    alpha,
                    beta,
                    visited_nodes,
                    config,
                );
                value = value.min(score);
                beta = beta.min(value);
                if beta <= alpha {
                    break;
                }
            }
            if value == i32::MAX {
                evaluate_preferability(game, perspective)
            } else {
                value
            }
        }
    }

    fn ranked_child_states(
        game: &MonsGame,
        perspective: Color,
        maximizing: bool,
        config: SmartSearchConfig,
    ) -> Vec<MonsGame> {
        let mut scored_states: Vec<(i32, MonsGame)> = Vec::new();
        for inputs in Self::enumerate_legal_inputs(game, config.node_enum_limit) {
            if let Some(simulated_game) = Self::apply_inputs_for_search(game, &inputs) {
                let heuristic = Self::score_state(&simulated_game, perspective, 0, config.depth);
                scored_states.push((heuristic, simulated_game));
            }
        }

        if maximizing {
            scored_states.sort_by(|a, b| b.0.cmp(&a.0));
        } else {
            scored_states.sort_by(|a, b| a.0.cmp(&b.0));
        }

        if scored_states.len() > config.node_branch_limit {
            scored_states.truncate(config.node_branch_limit);
        }

        scored_states.into_iter().map(|(_, game)| game).collect()
    }

    fn enumerate_legal_inputs(game: &MonsGame, max_moves: usize) -> Vec<Vec<Input>> {
        let mut all_inputs = Vec::new();
        let mut partial_inputs = Vec::new();
        let mut simulated_game = game.clone_for_simulation();
        Self::collect_legal_inputs(
            &mut simulated_game,
            &mut partial_inputs,
            &mut all_inputs,
            max_moves,
        );
        all_inputs
    }

    fn collect_legal_inputs(
        game: &mut MonsGame,
        partial_inputs: &mut Vec<Input>,
        all_inputs: &mut Vec<Vec<Input>>,
        max_moves: usize,
    ) {
        if all_inputs.len() >= max_moves || partial_inputs.len() > SMART_MAX_INPUT_CHAIN {
            return;
        }

        match game.process_input(partial_inputs.clone(), true, false) {
            Output::InvalidInput => {}
            Output::Events(_) => all_inputs.push(partial_inputs.clone()),
            Output::LocationsToStartFrom(locations) => {
                for location in locations {
                    if all_inputs.len() >= max_moves {
                        break;
                    }
                    partial_inputs.push(Input::Location(location));
                    Self::collect_legal_inputs(game, partial_inputs, all_inputs, max_moves);
                    partial_inputs.pop();
                }
            }
            Output::NextInputOptions(options) => {
                for option in options {
                    if all_inputs.len() >= max_moves {
                        break;
                    }
                    partial_inputs.push(option.input);
                    Self::collect_legal_inputs(game, partial_inputs, all_inputs, max_moves);
                    partial_inputs.pop();
                }
            }
        }
    }

    fn apply_inputs_for_search(game: &MonsGame, inputs: &[Input]) -> Option<MonsGame> {
        let mut simulated_game = game.clone_for_simulation();
        match simulated_game.process_input(inputs.to_vec(), false, false) {
            Output::Events(_) => Some(simulated_game),
            _ => None,
        }
    }

    fn score_state(game: &MonsGame, perspective: Color, depth: usize, search_depth: usize) -> i32 {
        if let Some(terminal_score) = Self::terminal_score(game, perspective, depth, search_depth) {
            terminal_score
        } else {
            evaluate_preferability(game, perspective)
        }
    }

    fn terminal_score(
        game: &MonsGame,
        perspective: Color,
        depth: usize,
        search_depth: usize,
    ) -> Option<i32> {
        game.winner_color().map(|winner| {
            let ply_count = (search_depth.saturating_sub(depth)) as i32;
            if winner == perspective {
                SMART_TERMINAL_SCORE - ply_count
            } else {
                -SMART_TERMINAL_SCORE + ply_count
            }
        })
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
            Event::SpiritTargetMove {
                item,
                from,
                to,
                by: _,
            } => EventModel {
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
            Event::UsePotion { from, to } => EventModel {
                kind: EventModelKind::UsePotion,
                item: None,
                mon: None,
                mana: None,
                loc1: Some(*from),
                loc2: Some(*to),
                color: None,
            },
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct VerboseTrackingEntityModel {
    fen: String,
    color: Color,
    events: Vec<Event>,
}

impl VerboseTrackingEntityModel {
    fn new(entity: &VerboseTrackingEntity) -> Self {
        Self {
            fen: entity.fen.clone(),
            color: entity.color,
            events: entity.events.clone(),
        }
    }
}

#[wasm_bindgen]
impl VerboseTrackingEntityModel {
    pub fn fen(&self) -> String {
        self.fen.clone()
    }
    pub fn color(&self) -> Color {
        self.color
    }
    pub fn events(&self) -> Vec<EventModel> {
        self.events.iter().map(|e| EventModel::new(e)).collect()
    }
    pub fn events_fen(&self) -> String {
        self.events
            .iter()
            .map(|e| e.fen())
            .collect::<Vec<_>>()
            .join(" ")
    }
}
