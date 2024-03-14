pub mod models;
pub use models::*;

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
        for entry in fs::read_dir(test_data_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
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
                        "fenBefore" => test_case.fen_before = parts[i + 2].to_string(),
                        "inputFen" => test_case.input_fen = parts[i + 2].to_string(),
                        "outputFen" => test_case.output_fen = parts[i + 2].to_string(),
                        "fenAfter" => test_case.fen_after = parts[i + 2].to_string(),
                        _ => {}
                    }
                }

                let inputs = Input::array_from_fen(&test_case.input_fen);
                let recreated_inputs_fen = Input::fen_from_array(&inputs);

                assert!(recreated_inputs_fen == test_case.input_fen);
                assert!(!test_case.fen_before.is_empty() && !test_case.fen_after.is_empty() && !test_case.output_fen.is_empty(), "test data must not be empty");
            }
        }
        Ok(())
    }
}
