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