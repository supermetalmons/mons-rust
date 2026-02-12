#[cfg(target_arch = "wasm32")]
use crate::models::scoring::evaluate_preferability;
use crate::*;

#[wasm_bindgen]
#[derive(Debug)]
pub struct MonsGameModel {
    game: MonsGame,
    #[cfg(target_arch = "wasm32")]
    smart_search_in_progress: std::rc::Rc<std::cell::Cell<bool>>,
}

impl Clone for MonsGameModel {
    fn clone(&self) -> Self {
        Self::with_game(self.game.clone())
    }
}

#[cfg(target_arch = "wasm32")]
const DEFAULT_SMART_SEARCH_DEPTH: usize = 2;
#[cfg(target_arch = "wasm32")]
const DEFAULT_SMART_MAX_VISITED_NODES: usize = 320;
#[cfg(target_arch = "wasm32")]
const MIN_SMART_SEARCH_DEPTH: usize = 1;
#[cfg(target_arch = "wasm32")]
const MAX_SMART_SEARCH_DEPTH: usize = 4;
#[cfg(target_arch = "wasm32")]
const MIN_SMART_MAX_VISITED_NODES: usize = 32;
#[cfg(target_arch = "wasm32")]
const MAX_SMART_MAX_VISITED_NODES: usize = 20_000;
#[cfg(target_arch = "wasm32")]
const SMART_TERMINAL_SCORE: i32 = i32::MAX / 8;
#[cfg(target_arch = "wasm32")]
const SMART_MAX_INPUT_CHAIN: usize = 8;

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Copy)]
struct SmartSearchConfig {
    depth: usize,
    max_visited_nodes: usize,
    root_enum_limit: usize,
    root_branch_limit: usize,
    node_enum_limit: usize,
    node_branch_limit: usize,
}

#[cfg(target_arch = "wasm32")]
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

#[cfg(target_arch = "wasm32")]
struct ScoredRootMove {
    inputs: Vec<Input>,
    game: MonsGame,
    heuristic: i32,
}

#[cfg(target_arch = "wasm32")]
struct RootEvaluation {
    score: i32,
    inputs: Vec<Input>,
}

#[cfg(target_arch = "wasm32")]
struct AsyncSmartSearchState {
    game: MonsGame,
    perspective: Color,
    config: SmartSearchConfig,
    root_moves: Vec<ScoredRootMove>,
    next_index: usize,
    visited_nodes: usize,
    alpha: i32,
    scored_roots: Vec<RootEvaluation>,
}

#[wasm_bindgen]
impl MonsGameModel {
    fn with_game(game: MonsGame) -> Self {
        Self {
            game,
            #[cfg(target_arch = "wasm32")]
            smart_search_in_progress: std::rc::Rc::new(std::cell::Cell::new(false)),
        }
    }

    pub fn new() -> MonsGameModel {
        Self::with_game(MonsGame::new(true))
    }

    #[wasm_bindgen(js_name = newForSimulation)]
    pub fn new_for_simulation() -> MonsGameModel {
        Self::with_game(MonsGame::new(false))
    }

    pub fn from_fen(fen: &str) -> Option<MonsGameModel> {
        if let Some(game) = MonsGame::from_fen(fen, true) {
            Some(Self::with_game(game))
        } else {
            return None;
        }
    }

    #[wasm_bindgen(js_name = fromFenForSimulation)]
    pub fn from_fen_for_simulation(fen: &str) -> Option<MonsGameModel> {
        MonsGame::from_fen(fen, false).map(Self::with_game)
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
            return Some(Self::with_game(new_game));
        }

        None
    }

    pub fn fen(&self) -> String {
        return self.game.fen();
    }

    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen(js_name = smartAutomoveAsync)]
    pub fn smart_automove_async(&self) -> js_sys::Promise {
        let (depth, max_visited_nodes) = Self::default_smart_budget_for_game(&self.game);
        self.smart_automove_with_budget_async(depth as i32, max_visited_nodes as i32)
    }

    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen(js_name = smartAutomoveWithBudgetAsync)]
    pub fn smart_automove_with_budget_async(
        &self,
        depth: i32,
        max_visited_nodes: i32,
    ) -> js_sys::Promise {
        use std::cell::RefCell;
        use std::rc::Rc;
        use wasm_bindgen::closure::Closure;
        use wasm_bindgen::JsCast;

        if self.smart_search_in_progress.get() {
            return js_sys::Promise::reject(&JsValue::from_str(
                "smart automove already in progress",
            ));
        }
        self.smart_search_in_progress.set(true);
        let in_progress = self.smart_search_in_progress.clone();

        let config = SmartSearchConfig::from_budget(depth, max_visited_nodes);
        let perspective = self.game.active_color;
        let game = self.game.clone_for_simulation();
        let root_moves = Self::ranked_root_moves(&game, perspective, config);

        let state = Rc::new(RefCell::new(AsyncSmartSearchState {
            game,
            perspective,
            config,
            root_moves,
            next_index: 0,
            visited_nodes: 0,
            alpha: i32::MIN,
            scored_roots: Vec::new(),
        }));

        js_sys::Promise::new(&mut move |resolve, reject| {
            let global = js_sys::global();
            let set_timeout = match js_sys::Reflect::get(&global, &JsValue::from_str("setTimeout"))
                .ok()
                .and_then(|value| value.dyn_into::<js_sys::Function>().ok())
            {
                Some(function) => function,
                None => {
                    in_progress.set(false);
                    let _ = reject.call1(
                        &JsValue::NULL,
                        &JsValue::from_str("setTimeout is not available"),
                    );
                    return;
                }
            };

            let tick: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
            let tick_for_closure = tick.clone();
            let state_inner = state.clone();
            let resolve_inner = resolve.clone();
            let reject_inner = reject.clone();
            let set_timeout_inner = set_timeout.clone();
            let in_progress_inner = in_progress.clone();

            *tick.borrow_mut() = Some(Closure::wrap(Box::new(move || {
                let done = {
                    let mut borrowed = state_inner.borrow_mut();
                    Self::advance_async_search(&mut borrowed)
                };

                if done {
                    let output = {
                        let mut borrowed = state_inner.borrow_mut();
                        Self::finalize_async_search(&mut borrowed)
                    };
                    in_progress_inner.set(false);
                    let _ = resolve_inner.call1(&JsValue::NULL, &JsValue::from(output));
                    tick_for_closure.borrow_mut().take();
                    return;
                }

                let callback = {
                    let borrowed = tick_for_closure.borrow();
                    borrowed.as_ref().map(|cb| cb.as_ref().clone())
                };

                if let Some(cb) = callback {
                    if let Err(err) = set_timeout_inner.call2(
                        &JsValue::NULL,
                        cb.unchecked_ref(),
                        &JsValue::from_f64(0.0),
                    ) {
                        in_progress_inner.set(false);
                        let _ = reject_inner.call1(&JsValue::NULL, &err);
                        tick_for_closure.borrow_mut().take();
                    }
                }
            }) as Box<dyn FnMut()>));

            let initial_callback = {
                let borrowed = tick.borrow();
                borrowed.as_ref().map(|cb| cb.as_ref().clone())
            };
            if let Some(cb) = initial_callback {
                let schedule_result =
                    set_timeout.call2(&JsValue::NULL, cb.unchecked_ref(), &JsValue::from_f64(0.0));
                if let Err(err) = schedule_result {
                    in_progress.set(false);
                    let _ = reject.call1(&JsValue::NULL, &err);
                    tick.borrow_mut().take();
                }
            }
        })
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

#[cfg(target_arch = "wasm32")]
impl MonsGameModel {
    fn default_smart_budget_for_game(game: &MonsGame) -> (usize, usize) {
        let _ = game;
        (DEFAULT_SMART_SEARCH_DEPTH, DEFAULT_SMART_MAX_VISITED_NODES)
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

    #[cfg(target_arch = "wasm32")]
    fn advance_async_search(state: &mut AsyncSmartSearchState) -> bool {
        if state.next_index >= state.root_moves.len()
            || state.visited_nodes >= state.config.max_visited_nodes
        {
            return true;
        }

        let candidate = &state.root_moves[state.next_index];
        state.visited_nodes += 1;
        let candidate_score = if state.config.depth > 1 {
            Self::search_score(
                &candidate.game,
                state.perspective,
                state.config.depth - 1,
                state.alpha,
                i32::MAX,
                &mut state.visited_nodes,
                state.config,
            )
        } else {
            candidate.heuristic
        };

        state.scored_roots.push(RootEvaluation {
            score: candidate_score,
            inputs: candidate.inputs.clone(),
        });

        if candidate_score > state.alpha {
            state.alpha = candidate_score;
        }

        state.next_index += 1;
        state.next_index >= state.root_moves.len()
            || state.visited_nodes >= state.config.max_visited_nodes
    }

    #[cfg(target_arch = "wasm32")]
    fn finalize_async_search(state: &mut AsyncSmartSearchState) -> OutputModel {
        if state.scored_roots.is_empty() {
            return Self::automove_game(&mut state.game);
        }

        let best_inputs = Self::pick_root_move_with_exploration(&state.scored_roots);
        let input_fen = Input::fen_from_array(&best_inputs);
        let output = state.game.process_input(best_inputs, false, false);
        OutputModel::new(output, input_fen.as_str())
    }

    fn pick_root_move_with_exploration(scored_roots: &[RootEvaluation]) -> Vec<Input> {
        let mut best_index = 0usize;
        let mut best_score = i32::MIN;
        for (index, evaluation) in scored_roots.iter().enumerate() {
            if evaluation.score > best_score {
                best_score = evaluation.score;
                best_index = index;
            }
        }
        scored_roots[best_index].inputs.clone()
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
