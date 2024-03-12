#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Mana {
    Regular(Color),
    Supermana,
}

impl Mana {
    fn score(&self, player: Color) -> i32 {
        match self {
            Mana::Regular(color) => if *color == player { 1 } else { 2 },
            Mana::Supermana => 2,
        }
    }
}
