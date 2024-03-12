use crate::*;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Input {
    Location(Location),
    Modifier(Modifier),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Modifier {
    SelectPotion,
    SelectBomb,
    Cancel,
}
