use crate::*;

pub trait FenRepresentable {
    fn fen(&self) -> String;
}

impl FenRepresentable for MonsGame {
    fn fen(&self) -> String {
        let fields = vec![
            self.white_score.to_string(),
            self.black_score.to_string(),
            self.active_color.fen(),
            self.actions_used_count.to_string(),
            self.mana_moves_count.to_string(),
            self.mons_moves_count.to_string(),
            self.white_potions_count.to_string(),
            self.black_potions_count.to_string(),
            self.turn_number.to_string(),
            self.board.fen(),
        ];
        fields.join(" ")
    }
}

impl MonsGame {
    fn from_fen(fen: &str) -> Option<Self> {
        let fields: Vec<&str> = fen.split_whitespace().collect();
        if fields.len() != 10 {
            return None;
        }
        Some(Self {
            board: Board::from_fen(fields[9])?,
            white_score: fields[0].parse().ok()?,
            black_score: fields[1].parse().ok()?,
            active_color: Color::from_fen(fields[2])?,
            actions_used_count: fields[3].parse().ok()?,
            mana_moves_count: fields[4].parse().ok()?,
            mons_moves_count: fields[5].parse().ok()?,
            white_potions_count: fields[6].parse().ok()?,
            black_potions_count: fields[7].parse().ok()?,
            turn_number: fields[8].parse().ok()?,
        })
    }
}

impl FenRepresentable for Item {
    fn fen(&self) -> String {
        match self {
            Item::Mon { mon } => mon.fen() + "x",
            Item::Mana { mana } => "xx".to_string() + &mana.fen(),
            Item::MonWithMana { mon, mana } => mon.fen() + &mana.fen(),
            Item::MonWithConsumable { mon, consumable } => mon.fen() + &consumable.fen(),
            Item::Consumable { consumable } => "xx".to_string() + &consumable.fen(),
        }
    }
}

impl Item {
    fn from_fen(fen: &str) -> Option<Self> {
        // Implementation similar to the Swift version, constructing an Item based on the FEN string
        todo!();
    }
}

impl FenRepresentable for Board {
    fn fen(&self) -> String {
        // Implementation similar to the Swift version, representing the board state as a FEN string
        todo!();
    }
}

impl Board {
    fn from_fen(fen: &str) -> Option<Self> {
        // Implementation similar to the Swift version, constructing a Board based on the FEN string
        todo!();
    }
}

impl FenRepresentable for Mon {
    fn fen(&self) -> String {
        // Implementation similar to the Swift version, representing a Mon as a FEN string
        todo!();
    }
}

impl Mon {
    fn from_fen(fen: &str) -> Option<Self> {
        // Implementation similar to the Swift version, constructing a Mon based on the FEN string
        todo!();
    }
}

impl FenRepresentable for Mana {
    fn fen(&self) -> String {
        // Implementation similar to the Swift version, representing Mana as a FEN string
        todo!();
    }
}

impl Mana {
    fn from_fen(fen: &str) -> Option<Self> {
        // Implementation similar to the Swift version, constructing Mana based on the FEN string
        todo!();
    }
}

impl FenRepresentable for Color {
    fn fen(&self) -> String {
        // Implementation similar to the Swift version, representing a Color as a FEN string
        todo!();
    }
}

impl Color {
    fn from_fen(fen: &str) -> Option<Self> {
        // Implementation similar to the Swift version, constructing a Color based on the FEN string
        todo!();
    }
}

impl FenRepresentable for Consumable {
    fn fen(&self) -> String {
        // Implementation similar to the Swift version, representing a Consumable as a FEN string
        todo!();
    }
}

impl Consumable {
    fn from_fen(fen: &str) -> Option<Self> {
        // Implementation similar to the Swift version, constructing a Consumable based on the FEN string
        todo!();
    }
}
