use serde::Serialize;
use serde::Deserialize;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Consumable {
    Potion,
    Bomb,
    BombOrPotion,
}