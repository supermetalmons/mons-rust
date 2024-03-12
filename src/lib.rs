pub mod models;
pub use models::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::{self, Read};
    use std::path::Path;

    #[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Clone, Debug)]
    struct TestCase {
        fen_before: String,
        input: Vec<Input>,
        output: Output,
        fen_after: String,
    }

    #[test]
    fn test_from_test_data() -> io::Result<()> {
        let test_data_dir = Path::new("test-data");
        for entry in fs::read_dir(test_data_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                println!("Testing with file: {:?}", path);

                let mut file = File::open(path)?;
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;

                let test_case: TestCase = serde_json::from_str(&contents)
                    .expect("Failed to deserialize the test case");

                // Implement your test logic here using test_case
                // For example, assert_eq!(test_case.fen_before, "Expected FEN before state");
            }
        }
        Ok(())
    }
}
