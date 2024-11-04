use crate::*;

pub fn evaluate_preferability(game: &MonsGame, color: Color) -> i32 {
    // TODO: better scoring function
    if color == Color::White {
        game.white_score
    } else {
        game.black_score
    }
}
