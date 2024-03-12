use crate::*;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub enum Input {
    Location(Location),
    Modifier(Modifier),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
pub enum Modifier {
    SelectPotion,
    SelectBomb,
    Cancel,
}
