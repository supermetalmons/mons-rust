pub mod models;
pub use models::*;

#[wasm_bindgen]
pub fn winner(fen_w: &str, fen_b: &str, flat_moves_string_w: &str, flat_moves_string_b: &str) -> String {
    let moves_w: Vec<&str> = flat_moves_string_w.split("-").collect();
    let moves_b: Vec<&str> = flat_moves_string_b.split("-").collect();

    let game_w = MonsGame::from_fen(&fen_w);
    let game_b = MonsGame::from_fen(&fen_b);

    if game_w.is_none() || game_b.is_none() {
        if game_w.is_none() && game_b.is_none() {
            return "x".to_string();
        } else if game_w.is_none() {
            return Color::Black.fen();
        } else {
            return Color::White.fen();
        }
    }

    let winner_color_game_w = game_w.unwrap().winner_color();
    let winner_color_game_b = game_b.unwrap().winner_color();

    if winner_color_game_w.is_none() && winner_color_game_b.is_none() {
        return "".to_string()
    } 

    let mut game = MonsGame::new();

    let mut w_index = 0;
    let mut b_index = 0;

    while w_index < moves_w.len() || b_index < moves_b.len() {
        if game.active_color == Color::White {
            if w_index >= moves_w.len() { return "x".to_string(); }
            let inputs = Input::array_from_fen(moves_w[w_index]);
            _ = game.process_input(inputs, false, false);
            w_index += 1;
        } else {
            if b_index >= moves_b.len() { return "x".to_string(); }
            let inputs = Input::array_from_fen(moves_b[b_index]);
            _ = game.process_input(inputs, false, false);
            b_index += 1;
        }

        if let Some(winner) = game.winner_color() {
            if winner == Color::White {
                if w_index == moves_w.len() && fen_w == game.fen() {
                    return winner.fen();
                } else {
                    return "x".to_string();
                }
            } else {
                if b_index == moves_b.len() && fen_b == game.fen() {
                    return winner.fen();
                } else {
                    return "x".to_string();
                }
            }
        }
    }

    // TODO: "x" stands for corrupted game data. see if there was cheating.
    return "x".to_string();
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::{self};

    #[test]
    fn test_from_test_data() -> io::Result<()> {
        let game = MonsGame::new();
        let fen = game.fen();
        assert!(fen == "0 0 w 0 0 0 0 0 1 n03y0xs0xd0xa0xe0xn03/n11/n11/n04xxmn01xxmn04/n03xxmn01xxmn01xxmn03/xxQn04xxUn04xxQ/n03xxMn01xxMn01xxMn03/n04xxMn01xxMn04/n11/n11/n03E0xA0xD0xS0xY0xn03");
        Ok(())
    }
}
