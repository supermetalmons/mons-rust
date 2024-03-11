use serde::{Serialize, Deserialize};
use std::collections::HashSet;

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Clone, Debug)]
struct TestCase {
    fen_before: String,
    input: Vec<Input>,
    output: Output,
    fen_after: String,
}