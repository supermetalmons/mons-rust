use crate::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MonsGame {
    pub board: Board,
    pub white_score: i32,
    pub black_score: i32,
    pub active_color: Color,
    pub actions_used_count: i32,
    pub mana_moves_count: i32,
    pub mons_moves_count: i32,
    pub white_potions_count: i32,
    pub black_potions_count: i32,
    pub turn_number: i32,
}

impl MonsGame {
    pub fn new() -> Self {
        Self {
            board: Board::new(),
            white_score: 0,
            black_score: 0,
            active_color: Color::White,
            actions_used_count: 0,
            mana_moves_count: 0,
            mons_moves_count: 0,
            white_potions_count: 0,
            black_potions_count: 0,
            turn_number: 1,
        }
    }

    pub fn with_params(
        board: Board,
        white_score: i32,
        black_score: i32,
        active_color: Color,
        actions_used_count: i32,
        mana_moves_count: i32,
        mons_moves_count: i32,
        white_potions_count: i32,
        black_potions_count: i32,
        turn_number: i32,
    ) -> Self {
        Self {
            board,
            white_score,
            black_score,
            active_color,
            actions_used_count,
            mana_moves_count,
            mons_moves_count,
            white_potions_count,
            black_potions_count,
            turn_number,
        }
    }

    pub fn update_with(&mut self, other_game: &MonsGame) {
        self.board = Board::new_with_items(other_game.board.items.clone());
        self.white_score = other_game.white_score;
        self.black_score = other_game.black_score;
        self.active_color = other_game.active_color;
        self.actions_used_count = other_game.actions_used_count;
        self.mana_moves_count = other_game.mana_moves_count;
        self.mons_moves_count = other_game.mons_moves_count;
        self.white_potions_count = other_game.white_potions_count;
        self.black_potions_count = other_game.black_potions_count;
        self.turn_number = other_game.turn_number;
    }

    // MARK: - process input
    pub fn process_input(&self, input: Vec<Input>, do_not_apply_events: bool, one_option_enough: bool) -> Output {
        todo!();
    }

    // MARK: - process step by step
    fn suggested_input_to_start_with(&self) -> Output {
        todo!();
    }

    fn second_input_options(&self, start_location: Location, start_item: Item, only_one: bool, specific_next: Option<Input>) -> Vec<NextInput> {
        todo!();
    }

    fn process_second_input(&self, kind: NextInputKind, start_item: Item, start_location: Location, target_location: Location, specific_next: Option<Input>) -> Option<(Vec<Event>, Vec<NextInput>)> {
        todo!();
    }

    fn process_third_input(&self, third_input: NextInput, start_item: Item, start_location: Location, target_location: Location) -> Option<(Vec<Event>, Vec<NextInput>)> {
        todo!();
    }

    // MARK: - apply events
    fn apply_and_add_resulting_events(&self, events: Vec<Event>) -> Vec<Event> {
        todo!();
    }

    // MARK: - helpers
    pub fn next_inputs<F>(&self, locations: Vec<Location>, kind: NextInputKind, only_one: bool, specific: Option<Location>, filter: F) -> Vec<NextInput>
    where
        F: Fn(Location) -> bool,
    {
        if let Some(specific_location) = specific {
            if locations.contains(&specific_location) && filter(specific_location) {
                return vec![NextInput { input: Input::Location(specific_location), kind, actor_mon_item: None }];
            } else {
                return vec![];
            }
        } else if only_one {
            if let Some(one) = locations.into_iter().find(|&loc| filter(loc)) {
                return vec![NextInput { input: Input::Location(one), kind, actor_mon_item: None }];
            } else {
                return vec![];
            }
        } else {
            return locations.into_iter().filter_map(|loc| {
                if filter(loc) {
                    Some(NextInput { input: Input::Location(loc), kind, actor_mon_item: None })
                } else {
                    None
                }
            }).collect();
        }
    }

    pub fn available_move_kinds(&self) -> HashMap<AvailableMoveKind, i32> {
        let mut moves = HashMap::new();
        moves.insert(AvailableMoveKind::MonMove, Config::MONS_MOVES_PER_TURN - self.mons_moves_count);
        moves.insert(AvailableMoveKind::Action, 0);
        moves.insert(AvailableMoveKind::Potion, 0);
        moves.insert(AvailableMoveKind::ManaMove, 0);

        if self.turn_number == 1 {
            return moves;
        }

        moves.insert(AvailableMoveKind::Action, Config::ACTIONS_PER_TURN - self.actions_used_count);
        moves.insert(AvailableMoveKind::Potion, self.player_potions_count());
        moves.insert(AvailableMoveKind::ManaMove, Config::MANA_MOVES_PER_TURN - self.mana_moves_count);

        moves
    }

    pub fn winner_color(&self) -> Option<Color> {
        if self.white_score >= Config::TARGET_SCORE {
            Some(Color::White)
        } else if self.black_score >= Config::TARGET_SCORE {
            Some(Color::Black)
        } else {
            None
        }
    }

    pub fn is_later_than(&self, game: &MonsGame) -> bool {
        if self.turn_number > game.turn_number {
            true
        } else if self.turn_number == game.turn_number {
            self.player_potions_count() < game.player_potions_count() ||
            self.actions_used_count > game.actions_used_count ||
            self.mana_moves_count > game.mana_moves_count ||
            self.mons_moves_count > game.mons_moves_count ||
            self.board.fainted_mons_locations(self.active_color.other()).len() > game.board.fainted_mons_locations(game.active_color.other()).len()
        } else {
            false
        }
    }

    pub fn is_first_turn(&self) -> bool { 
        self.turn_number == 1 
    }

    pub fn player_potions_count(&self) -> i32 {
        match self.active_color {
            Color::White => self.white_potions_count,
            Color::Black => self.black_potions_count,
        }
    }

    pub fn player_can_move_mon(&self) -> bool { 
        self.mons_moves_count < Config::MONS_MOVES_PER_TURN 
    }

    pub fn player_can_move_mana(&self) -> bool { 
        !self.is_first_turn() && self.mana_moves_count < Config::MANA_MOVES_PER_TURN 
    }

    pub fn player_can_use_action(&self) -> bool { 
        !self.is_first_turn() && (self.player_potions_count() > 0 || self.actions_used_count < Config::ACTIONS_PER_TURN) 
    }

    pub fn protected_by_opponents_angel(&self) -> std::collections::HashSet<Location> {
        if let Some(location) = self.board.find_awake_angel(self.active_color.other()) {
            let protected: Vec<Location> = location.nearby_locations(1);
            protected.into_iter().collect()
        } else {
            std::collections::HashSet::new()
        }
    }
}
