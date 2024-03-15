use crate::*;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Input {
    Location(Location),
    Modifier(Modifier),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Modifier {
    SelectPotion,
    SelectBomb,
    Cancel,
}
