use serde::Serialize;
use serde::Deserialize;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn other(&self) -> Self {
        match self {
            Color::Black => Color::White,
            Color::White => Color::Black,
        }
    }
}