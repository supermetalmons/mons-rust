use crate::*;

#[wasm_bindgen]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Consumable {
    Potion,
    Bomb,
    BombOrPotion,
}
