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
    pub fn from_fen(fen: &str) -> Option<Self> {
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
        let item_fen = &fen[2..];

        match mon_fen {
            "xx" => match Mana::from_fen(item_fen) {
                Some(mana) => Some(Item::Mana { mana }),
                None => Consumable::from_fen(item_fen).map(|consumable| Item::Consumable { consumable }),
            },
            _ => {
                let mon = Mon::from_fen(mon_fen)?;
                if let Some(mana) = Mana::from_fen(item_fen) {
                    Some(Item::MonWithMana { mon, mana })
                } else if let Some(consumable) = Consumable::from_fen(item_fen) {
                    Some(Item::MonWithConsumable { mon, consumable })
                } else {
                    Some(Item::Mon { mon })
                }
            }
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
    pub fn from_fen(fen: &str) -> Option<Self> {
        let lines: Vec<&str> = fen.split('/').collect();
        if lines.len() != Config::BOARD_SIZE as usize {
            return None;
        }
        let mut items = std::collections::HashMap::new();
        for (i, line) in lines.iter().enumerate() {
            let mut j = 0;
            let mut chars = line.chars().peekable();

            while let Some(ch) = chars.peek() {
                match ch {
                    'n' => {
                        chars.next();
                        let num_chars: String = chars.by_ref().take(2).collect();
                        if let Ok(num) = num_chars.parse::<usize>() {
                            j += num;
                        }
                    },
                    _ => {
                        let item_fen: String = chars.by_ref().take(3).collect();
                        if let Some(item) = Item::from_fen(&item_fen) {
                            items.insert(Location { i: i as i32, j: j as i32 }, item);
                        }
                        j += 1;
                    }
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

impl FenRepresentable for Event {
    fn fen(&self) -> String {
        match self {
            Event::MonMove { item, from, to } => format!("mm {} {} {}", item.fen(), from.fen(), to.fen()),
            Event::ManaMove { mana, from, to } => format!("mma {} {} {}", mana.fen(), from.fen(), to.fen()),
            Event::ManaScored { mana, at } => format!("ms {} {}", mana.fen(), at.fen()),
            Event::MysticAction { mystic, from, to } => format!("ma {} {} {}", mystic.fen(), from.fen(), to.fen()),
            Event::DemonAction { demon, from, to } => format!("da {} {} {}", demon.fen(), from.fen(), to.fen()),
            Event::DemonAdditionalStep { demon, from, to } => format!("das {} {} {}", demon.fen(), from.fen(), to.fen()),
            Event::SpiritTargetMove { item, from, to } => format!("stm {} {} {}", item.fen(), from.fen(), to.fen()),
            Event::PickupBomb { by, at } => format!("pb {} {}", by.fen(), at.fen()),
            Event::PickupPotion { by, at } => format!("pp {} {}", by.fen(), at.fen()),
            Event::PickupMana { mana, by, at } => format!("pm {} {} {}", mana.fen(), by.fen(), at.fen()),
            Event::MonFainted { mon, from, to } => format!("mf {} {} {}", mon.fen(), from.fen(), to.fen()),
            Event::ManaDropped { mana, at } => format!("md {} {}", mana.fen(), at.fen()),
            Event::SupermanaBackToBase { from, to } => format!("sb {} {}", from.fen(), to.fen()),
            Event::BombAttack { by, from, to } => format!("ba {} {} {}", by.fen(), from.fen(), to.fen()),
            Event::MonAwake { mon, at } => format!("maw {} {}", mon.fen(), at.fen()),
            Event::BombExplosion { at } => format!("be {}", at.fen()),
            Event::NextTurn { color } => format!("nt {}", color.fen()),
            Event::GameOver { winner } => format!("go {}", winner.fen()),
            Event::Takeback => "z".to_string(),
        }
    }
}

impl Event {
    fn from_fen(fen: &str) -> Option<Self> {
        let parts: Vec<&str> = fen.split(' ').collect();
        match parts.as_slice() {
            ["mm", item_fen, from_fen, to_fen] => {
                Some(Event::MonMove {
                    item: Item::from_fen(item_fen)?,
                    from: Location::from_fen(from_fen)?,
                    to: Location::from_fen(to_fen)?,
                })
            }
            ["mma", mana_fen, from_fen, to_fen] => {
                Some(Event::ManaMove {
                    mana: Mana::from_fen(mana_fen)?,
                    from: Location::from_fen(from_fen)?,
                    to: Location::from_fen(to_fen)?,
                })
            }
            ["ms", mana_fen, at_fen] => {
                Some(Event::ManaScored {
                    mana: Mana::from_fen(mana_fen)?,
                    at: Location::from_fen(at_fen)?,
                })
            }
            ["ma", mystic_fen, from_fen, to_fen] => {
                Some(Event::MysticAction {
                    mystic: Mon::from_fen(mystic_fen)?,
                    from: Location::from_fen(from_fen)?,
                    to: Location::from_fen(to_fen)?,
                })
            }
            ["da", demon_fen, from_fen, to_fen] => {
                Some(Event::DemonAction {
                    demon: Mon::from_fen(demon_fen)?,
                    from: Location::from_fen(from_fen)?,
                    to: Location::from_fen(to_fen)?,
                })
            }
            ["das", demon_fen, from_fen, to_fen] => {
                Some(Event::DemonAdditionalStep {
                    demon: Mon::from_fen(demon_fen)?,
                    from: Location::from_fen(from_fen)?,
                    to: Location::from_fen(to_fen)?,
                })
            }
            ["stm", item_fen, from_fen, to_fen] => {
                Some(Event::SpiritTargetMove {
                    item: Item::from_fen(item_fen)?,
                    from: Location::from_fen(from_fen)?,
                    to: Location::from_fen(to_fen)?,
                })
            }
            ["pb", by_fen, at_fen] => {
                Some(Event::PickupBomb {
                    by: Mon::from_fen(by_fen)?,
                    at: Location::from_fen(at_fen)?,
                })
            }
            ["pp", by_fen, at_fen] => {
                Some(Event::PickupPotion {
                    by: Item::from_fen(by_fen)?,
                    at: Location::from_fen(at_fen)?,
                })
            }
            ["pm", mana_fen, by_fen, at_fen] => {
                Some(Event::PickupMana {
                    mana: Mana::from_fen(mana_fen)?,
                    by: Mon::from_fen(by_fen)?,
                    at: Location::from_fen(at_fen)?,
                })
            }
            ["mf", mon_fen, from_fen, to_fen] => {
                Some(Event::MonFainted {
                    mon: Mon::from_fen(mon_fen)?,
                    from: Location::from_fen(from_fen)?,
                    to: Location::from_fen(to_fen)?,
                })
            }
            ["md", mana_fen, at_fen] => {
                Some(Event::ManaDropped {
                    mana: Mana::from_fen(mana_fen)?,
                    at: Location::from_fen(at_fen)?,
                })
            }
            ["sb", from_fen, to_fen] => {
                Some(Event::SupermanaBackToBase {
                    from: Location::from_fen(from_fen)?,
                    to: Location::from_fen(to_fen)?,
                })
            }
            ["ba", by_fen, from_fen, to_fen] => {
                Some(Event::BombAttack {
                    by: Mon::from_fen(by_fen)?,
                    from: Location::from_fen(from_fen)?,
                    to: Location::from_fen(to_fen)?,
                })
            }
            ["maw", mon_fen, at_fen] => {
                Some(Event::MonAwake {
                    mon: Mon::from_fen(mon_fen)?,
                    at: Location::from_fen(at_fen)?,
                })
            }
            ["be", at_fen] => {
                Some(Event::BombExplosion {
                    at: Location::from_fen(at_fen)?,
                })
            }
            ["nt", color_fen] => {
                Some(Event::NextTurn {
                    color: Color::from_fen(color_fen)?,
                })
            }
            ["go", winner_fen] => {
                Some(Event::GameOver {
                    winner: Color::from_fen(winner_fen)?,
                })
            }
            _ => None,
        }
    }
}

impl FenRepresentable for NextInput {
    fn fen(&self) -> String {
        format!(
            "{} {} {}",
            self.input.fen(),
            self.kind.fen(),
            self.actor_mon_item.as_ref().map_or("o".to_string(), |item| item.fen())
        )
    }
}

impl NextInput {
    fn from_fen(fen: &str) -> Option<Self> {
        let components: Vec<&str> = fen.split_whitespace().collect();
        if components.len() != 3 {
            return None;
        }
        let input = Input::from_fen(components[0])?;
        let kind = NextInputKind::from_fen(components[1])?;
        let actor_mon_item = if components[2] != "o" {
            Some(Item::from_fen(components[2])?)
        } else {
            None
        };

        Some(Self::new(input, kind, actor_mon_item))
    }
}

impl FenRepresentable for NextInputKind {
    fn fen(&self) -> String {
        match self {
            NextInputKind::MonMove => "mm".to_string(),
            NextInputKind::ManaMove => "mma".to_string(),
            NextInputKind::MysticAction => "ma".to_string(),
            NextInputKind::DemonAction => "da".to_string(),
            NextInputKind::DemonAdditionalStep => "das".to_string(),
            NextInputKind::SpiritTargetCapture => "stc".to_string(),
            NextInputKind::SpiritTargetMove => "stm".to_string(),
            NextInputKind::SelectConsumable => "sc".to_string(),
            NextInputKind::BombAttack => "ba".to_string(),
        }
    }
}

impl NextInputKind {
    fn from_fen(fen: &str) -> Option<Self> {
        match fen {
            "mm" => Some(NextInputKind::MonMove),
            "mma" => Some(NextInputKind::ManaMove),
            "ma" => Some(NextInputKind::MysticAction),
            "da" => Some(NextInputKind::DemonAction),
            "das" => Some(NextInputKind::DemonAdditionalStep),
            "stc" => Some(NextInputKind::SpiritTargetCapture),
            "stm" => Some(NextInputKind::SpiritTargetMove),
            "sc" => Some(NextInputKind::SelectConsumable),
            "ba" => Some(NextInputKind::BombAttack),
            _ => None,
        }
    }
}

impl FenRepresentable for Location {
    fn fen(&self) -> String {
        format!("{},{}", self.i, self.j)
    }
}

impl Location {
    fn from_fen(fen: &str) -> Option<Self> {
        let parts: Vec<&str> = fen.split(',').collect();
        if parts.len() != 2 {
            return None;
        }
        let i = parts[0].parse().ok()?;
        let j = parts[1].parse().ok()?;
        Some(Self { i, j })
    }
}

impl FenRepresentable for Modifier {
    fn fen(&self) -> String {
        match self {
            Modifier::SelectPotion => "p",
            Modifier::SelectBomb => "b",
            Modifier::Cancel => "c",
        }.to_string()
    }
}

impl Modifier {
    fn from_fen(fen: &str) -> Option<Self> {
        match fen {
            "p" => Some(Modifier::SelectPotion),
            "b" => Some(Modifier::SelectBomb),
            "c" => Some(Modifier::Cancel),
            _ => None,
        }
    }
}

impl FenRepresentable for Input {
    fn fen(&self) -> String {
        match self {
            Input::Location(location) => format!("l{}", location.fen()),
            Input::Modifier(modifier) => format!("m{}", modifier.fen()),
            Input::Takeback => "z".to_string(),
        }
    }
}

impl Input {
    pub fn fen_from_array(inputs: &[Input]) -> String {
        inputs.iter()
            .map(|input| input.fen())
            .collect::<Vec<_>>()
            .join(";")
    }

    pub fn from_fen(fen: &str) -> Option<Self> {
        fen.chars().next().and_then(|prefix| match prefix {
            'l' => Location::from_fen(&fen[1..]).map(Input::Location),
            'm' => Modifier::from_fen(&fen[1..]).map(Input::Modifier),
            'z' => Some(Input::Takeback),
            _ => None,
        })
    }

    pub fn array_from_fen(fen: &str) -> Vec<Self> {
        if fen.is_empty() {
            vec![]
        } else {
            fen.split(';')
               .filter_map(|f| Input::from_fen(f))
               .collect()
        }
    }
}

impl FenRepresentable for Output {
    fn fen(&self) -> String {
        match self {
            Output::InvalidInput => "i".to_string(),
            Output::LocationsToStartFrom(locations) => {
                let mut sorted_locations: Vec<_> = locations.iter().map(|location| location.fen()).collect();
                sorted_locations.sort();
                "l".to_owned() + &sorted_locations.join("/")
            },
            Output::NextInputOptions(next_inputs) => {
                let mut sorted_next_inputs: Vec<_> = next_inputs.iter().map(|next_input| next_input.fen()).collect();
                sorted_next_inputs.sort();
                "n".to_owned() + &sorted_next_inputs.join("/")
            },
            Output::Events(events) => {
                let mut sorted_events: Vec<_> = events.iter().map(|event| event.fen()).collect();
                sorted_events.sort();
                "e".to_owned() + &sorted_events.join("/")
            },
        }
    }
}

impl Output {
    pub fn from_fen(fen: &str) -> Option<Self> {
        let (prefix, data) = fen.split_at(1);
        match prefix {
            "i" => Some(Output::InvalidInput),
            "l" => {
                let locations = data.split('/').filter_map(|f| Location::from_fen(f)).collect::<Vec<_>>();
                if locations.len() > 0 {
                    Some(Output::LocationsToStartFrom(locations))
                } else {
                    None
                }
            },
            "n" => {
                let next_inputs = data.split('/').filter_map(|f| NextInput::from_fen(f)).collect::<Vec<_>>();
                if next_inputs.len() > 0 {
                    Some(Output::NextInputOptions(next_inputs))
                } else {
                    None
                }
            },
            "e" => {
                let events = data.split('/').filter_map(|f| Event::from_fen(f)).collect::<Vec<_>>();
                if events.len() > 0 {
                    Some(Output::Events(events))
                } else {
                    None
                }
            },
            _ => None,
        }
    }
}