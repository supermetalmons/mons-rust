#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum Kind {
    Demon,
    Drainer,
    Angel,
    Spirit,
    Mystic,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct Mon {
    kind: Kind,
    color: Color,
    cooldown: i32,
}

impl Mon {
    pub fn new(kind: Kind, color: Color, cooldown: i32) -> Self {
        Mon { kind, color, cooldown }
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
