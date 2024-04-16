pub mod models;
pub use models::*;

use wasm_bindgen::prelude::*;

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

    return "x".to_string();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::{self, Read};
    use std::path::Path;

    #[derive(Hash, PartialEq, Eq, Clone, Debug)]
    struct TestCase {
        fen_before: String,
        input_fen: String,
        output_fen: String,
        fen_after: String,
    }

    #[test]
    fn test_from_test_data() -> io::Result<()> {
        let test_data_dir = Path::new("test-data");
        let mut count = 0;
        let mut oks = 0;
        for entry in fs::read_dir(test_data_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.file_name().and_then(|f| f.to_str()).map_or(false, |s| !s.starts_with('.')) {
                let mut file = File::open(&path)?;
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                let parts: Vec<&str> = contents.split("\"").collect();
                let mut test_case = TestCase {
                    fen_before: String::new(),
                    input_fen: String::new(),
                    output_fen: String::new(),
                    fen_after: String::new(),
                };

                for i in 0..parts.len() {
                    match parts[i] {
                        "fenBefore" => test_case.fen_before = parts[i + 2].replace('\\', ""),
                        "inputFen" => test_case.input_fen = parts[i + 2].replace('\\', ""),
                        "outputFen" => test_case.output_fen = parts[i + 2].replace('\\', ""),
                        "fenAfter" => test_case.fen_after = parts[i + 2].replace('\\', ""),
                        _ => {}
                    }
                }

                let inputs = Input::array_from_fen(&test_case.input_fen);
                let recreated_inputs_fen = Input::fen_from_array(&inputs);
                assert!(recreated_inputs_fen == test_case.input_fen);

                let game_after = MonsGame::from_fen(&test_case.fen_after);
                let recreated_game_after_fen = game_after.unwrap().fen();
                assert!(recreated_game_after_fen == test_case.fen_after);

                let mut game_before = MonsGame::from_fen(&test_case.fen_before).unwrap();
                let recreated_game_before_fen = game_before.fen();
                assert!(recreated_game_before_fen == test_case.fen_before);

                let output = Output::from_fen(&test_case.output_fen);
                let recreated_output_fen = output.unwrap().fen();
                assert!(recreated_output_fen == test_case.output_fen);

                assert!(!test_case.fen_before.is_empty() && !test_case.fen_after.is_empty() && !test_case.output_fen.is_empty(), "test data must not be empty");

                let actual_output = game_before.process_input(inputs, false, false);

                if game_before.fen() != test_case.fen_after || actual_output.fen() != test_case.output_fen {
                    println!("expected {}", test_case.output_fen);
                    println!("received {}", actual_output.fen());
                    println!("forinput {}", test_case.input_fen);
                    println!("forboard {}", test_case.fen_before);
                    println!("expected fen after {}", test_case.fen_after);
                    println!("received fen after {}\n\n\n\n\n", game_before.fen());
                    count += 1;
                } else {
                    oks += 1;
                    println!("ok {}", oks);
                }

                assert!(game_before.fen() == test_case.fen_after);
                assert!(actual_output.fen() == test_case.output_fen);
            }
        }
        println!("\n\n\n\n\n TOTAL ERRORS {}", count);
        println!("TOTAL OKS {}", oks);
        Ok(())
    }
}
