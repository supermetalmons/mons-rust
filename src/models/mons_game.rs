use crate::*;

#[derive(Debug, Clone)]
pub struct MonsGame {
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
}

impl MonsGame {
    fn new() -> Self {
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

    fn with_params(
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

    fn update_with(&mut self, other_game: &MonsGame) {
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
    fn process_input(&self, input: Vec<Input>, do_not_apply_events: bool, one_option_enough: bool) -> Output {
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
    fn next_inputs(&self, locations: Vec<Location>, kind: NextInputKind, only_one: bool, specific: Option<Location>, filter: impl Fn(Location) -> bool) -> Vec<NextInput> {
        todo!();
    }

    fn available_move_kinds(&self) -> std::collections::HashMap<AvailableMoveKind, i32> {
        todo!();
    }

    fn winner_color(&self) -> Option<Color> {
        todo!();
    }

    fn is_later_than(&self, game: &MonsGame) -> bool {
        todo!();
    }

    fn is_first_turn(&self) -> bool {
        self.turn_number == 1
    }

    fn player_potions_count(&self) -> i32 {
        match self.active_color {
            Color::White => self.white_potions_count,
            Color::Black => self.black_potions_count,
        }
    }

    fn player_can_move_mon(&self) -> bool {
        self.mons_moves_count < Config::mons_moves_per_turn()
    }

    fn player_can_move_mana(&self) -> bool {
        !self.is_first_turn() && self.mana_moves_count < Config::mana_moves_per_turn()
    }

    fn player_can_use_action(&self) -> bool {
        !self.is_first_turn() && (self.player_potions_count() > 0 || self.actions_used_count < Config::actions_per_turn())
    }

    fn protected_by_opponents_angel(&self) -> std::collections::HashSet<Location> {
        todo!();
    }
}
