use crate::*;

#[wasm_bindgen]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum MonKind {
    Demon,
    Drainer,
    Angel,
    Spirit,
    Mystic,
}

#[wasm_bindgen]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Mon {
    pub kind: MonKind,
    pub color: Color,
    pub cooldown: i32,
}

#[wasm_bindgen]
impl Mon {
    pub fn new(kind: MonKind, color: Color, cooldown: i32) -> Self {
        Mon {
            kind,
            color,
            cooldown,
        }
    }

    pub fn is_fainted(&self) -> bool {
        self.cooldown > 0
    }

    pub fn faint(&mut self) {
        self.cooldown = 2;
    }

    pub fn decrease_cooldown(&mut self) {
        if self.cooldown > 0 {
            self.cooldown -= 1;
        }
    }
}
