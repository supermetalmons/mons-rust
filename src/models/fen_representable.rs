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
            Item::Mon { mon } => format!("{}x", mon.fen()),
            Item::Mana { mana } => format!("xx{}", mana.fen()),
            Item::MonWithMana { mon, mana } => format!("{}{}", mon.fen(), mana.fen()),
            Item::MonWithConsumable { mon, consumable } => format!("{}{}", mon.fen(), consumable.fen()),
            Item::Consumable { consumable } => format!("xx{}", consumable.fen()),
        }
    }
}

impl Item {
    fn from_fen(fen: &str) -> Option<Self> {
        if fen.len() != 3 {
            return None;
        }

        let mon_fen = &fen[0..2];
        let item_fen = &fen[2..3];

        let mon = Mon::from_fen(mon_fen);
        let mana = Mana::from_fen(item_fen);
        let consumable = Consumable::from_fen(item_fen);

        match (mon, mana, consumable) {
            (Some(mon), Some(mana), _) => Some(Item::MonWithMana { mon, mana }),
            (Some(mon), _, Some(consumable)) => Some(Item::MonWithConsumable { mon, consumable }),
            (Some(mon), None, None) => Some(Item::Mon { mon }),
            (None, Some(mana), _) => Some(Item::Mana { mana }),
            (None, _, Some(consumable)) => Some(Item::Consumable { consumable }),
            _ => None,
        }
    }
}

impl FenRepresentable for Board {
    fn fen(&self) -> String {
        let mut lines: Vec<String> = Vec::new();
        for i in 0..Config::BOARD_SIZE {
            let mut line = String::new();
            let mut empty_space_count = 0;
            for j in 0..Config::BOARD_SIZE {
                match self.items.get(&Location { i, j }) {
                    Some(item) => {
                        if empty_space_count > 0 {
                            line += &format!("n{:02}", empty_space_count);
                            empty_space_count = 0;
                        }
                        line += &item.fen();
                    }
                    None => {
                        empty_space_count += 1;
                    }
                }
            }
            if empty_space_count > 0 {
                line += &format!("n{:02}", empty_space_count);
            }
            lines.push(line);
        }
        lines.join("/")
    }
}

impl Board {
    fn from_fen(fen: &str) -> Option<Self> {
        let lines: Vec<&str> = fen.split('/').collect();
        if lines.len() as i32 != Config::BOARD_SIZE {
            return None;
        }
        let mut items = std::collections::HashMap::new();

        for (i, line) in lines.iter().enumerate() {
            let mut j = 0;
            let mut chars = line.chars().peekable();
            while let Some(ch) = chars.next() {
                if ch == 'n' {
                    if let Some(next_ch) = chars.next() {
                        if let Some(digit) = next_ch.to_digit(10) {
                            j += digit as usize;
                            if chars.peek().is_some() {
                                if let Some(second_digit) = chars.next().unwrap().to_digit(10) {
                                    j += (second_digit + 10 * digit - digit) as usize;
                                }
                            }
                        }
                    }
                } else {
                    let item_fen = ch.to_string() + &chars.next().unwrap().to_string() + &chars.next().unwrap().to_string();
                    if let Some(item) = Item::from_fen(&item_fen) {
                        items.insert(Location { i: i as i32, j: j as i32 }, item);
                    }
                    j += 1;
                }
            }
        }
        Some(Self { items })
    }
}

impl FenRepresentable for Mon {
    fn fen(&self) -> String {
        let kind_char = match self.kind {
            MonKind::Demon => 'e',
            MonKind::Drainer => 'd',
            MonKind::Angel => 'a',
            MonKind::Spirit => 's',
            MonKind::Mystic => 'y',
        };
        let kind_char = if self.color == Color::White { kind_char.to_uppercase().to_string() } else { kind_char.to_string() };
        format!("{}{}", kind_char, self.cooldown % 10)
    }
}

impl Mon {
    fn from_fen(fen: &str) -> Option<Self> {
        if fen.len() != 2 {
            return None;
        }
        let chars: Vec<char> = fen.chars().collect();
        let kind = match chars[0].to_ascii_lowercase() {
            'e' => MonKind::Demon,
            'd' => MonKind::Drainer,
            'a' => MonKind::Angel,
            's' => MonKind::Spirit,
            'y' => MonKind::Mystic,
            _ => return None,
        };
        let color = if chars[0].is_uppercase() { Color::White } else { Color::Black };
        let cooldown = chars[1].to_digit(10)?;
        Some(Mon { kind, color, cooldown: cooldown as i32 })
    }
}

impl FenRepresentable for Mana {
    fn fen(&self) -> String {
        match *self {
            Mana::Regular(Color::White) => "M".to_string(),
            Mana::Regular(Color::Black) => "m".to_string(),
            Mana::Supermana => "U".to_string(),
        }
    }
}

impl Mana {
    fn from_fen(fen: &str) -> Option<Self> {
        match fen {
            "U" => Some(Mana::Supermana),
            "M" => Some(Mana::Regular(Color::White)),
            "m" => Some(Mana::Regular(Color::Black)),
            _ => None,
        }
    }
}

impl FenRepresentable for Color {
    fn fen(&self) -> String {
        match self {
            Color::White => "w".to_string(),
            Color::Black => "b".to_string(),
        }
    }
}

impl Color {
    fn from_fen(fen: &str) -> Option<Self> {
        match fen {
            "w" => Some(Color::White),
            "b" => Some(Color::Black),
            _ => None,
        }
    }
}

impl FenRepresentable for Consumable {
    fn fen(&self) -> String {
        match self {
            Consumable::Potion => "P".to_string(),
            Consumable::Bomb => "B".to_string(),
            Consumable::BombOrPotion => "Q".to_string(),
        }
    }
}

impl Consumable {
    fn from_fen(fen: &str) -> Option<Self> {
        match fen {
            "P" => Some(Consumable::Potion),
            "B" => Some(Consumable::Bomb),
            "Q" => Some(Consumable::BombOrPotion),
            _ => None,
        }
    }
}