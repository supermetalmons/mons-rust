use crate::*;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Consumable {
    Potion,
    Bomb,
    BombOrPotion,
}