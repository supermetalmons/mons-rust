use crate::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MonsGame {
    pub board: Board,
    pub white_score: i32,
    pub black_score: i32,
    pub active_color: Color,
    pub actions_used_count: i32,
    pub mana_moves_count: i32,
    pub mons_moves_count: i32,
    pub white_potions_count: i32,
    pub black_potions_count: i32,
    pub turn_number: i32,
}

impl MonsGame {
    pub fn new() -> Self {
        Self {
            board: Board::new(),
            white_score: 0,
            black_score: 0,
            active_color: Color::White,
            actions_used_count: 0,
            mana_moves_count: 0,
            mons_moves_count: 0,
            white_potions_count: 0,
            black_potions_count: 0,
            turn_number: 1,
        }
    }

    pub fn with_params(
        board: Board,
        white_score: i32,
        black_score: i32,
        active_color: Color,
        actions_used_count: i32,
        mana_moves_count: i32,
        mons_moves_count: i32,
        white_potions_count: i32,
        black_potions_count: i32,
        turn_number: i32,
    ) -> Self {
        Self {
            board,
            white_score,
            black_score,
            active_color,
            actions_used_count,
            mana_moves_count,
            mons_moves_count,
            white_potions_count,
            black_potions_count,
            turn_number,
        }
    }

    pub fn update_with(&mut self, other_game: &MonsGame) {
        self.board = Board::new_with_items(other_game.board.items.clone());
        self.white_score = other_game.white_score;
        self.black_score = other_game.black_score;
        self.active_color = other_game.active_color;
        self.actions_used_count = other_game.actions_used_count;
        self.mana_moves_count = other_game.mana_moves_count;
        self.mons_moves_count = other_game.mons_moves_count;
        self.white_potions_count = other_game.white_potions_count;
        self.black_potions_count = other_game.black_potions_count;
        self.turn_number = other_game.turn_number;
    }

    // MARK: - process input

    pub fn process_input(&mut self, input: Vec<Input>, do_not_apply_events: bool, one_option_enough: bool) -> Output {
        if self.winner_color().is_some() {
            return Output::InvalidInput;
        }
        if input.is_empty() {
            return self.suggested_input_to_start_with();
        }
        let start_location = match input.get(0) {
            Some(Input::Location(location)) => *location,
            _ => return Output::InvalidInput,
        };
        let start_item = match self.board.item(start_location) {
            Some(item) => item.clone(),
            None => return Output::InvalidInput,
        };
        let specific_second_input = input.get(1).cloned();
        let second_input_options = self.second_input_options(start_location, &start_item, one_option_enough, specific_second_input);
    
        let second_input = match specific_second_input {
            None => {
                if second_input_options.is_empty() {
                    return Output::InvalidInput;
                } else {
                    return Output::NextInputOptions(second_input_options);
                }
            }
            Some(input) => input,
        };
    
        let target_location = match second_input {
            Input::Location(location) => location,
            _ => return Output::InvalidInput,
        };
        let second_input_kind = match second_input_options.iter().find(|option| option.input == second_input) {
            Some(option) => option.kind,
            None => return Output::InvalidInput,
        };
    
        let specific_third_input = input.get(2).cloned();
        let (mut events, third_input_options) = match self.process_second_input(second_input_kind, start_item.clone(), start_location, target_location, specific_third_input) {
            Some((events, options)) => (events, options),
            None => (vec![], vec![]),
        };
    
        if specific_third_input.is_none() {
            if !third_input_options.is_empty() {
                return Output::NextInputOptions(third_input_options);
            } else if !events.is_empty() {
                return Output::Events(if do_not_apply_events { events } else { self.apply_and_add_resulting_events(events) });
            } else {
                return Output::InvalidInput;
            }
        }
    
        let specific_third_input = specific_third_input.unwrap();
    
        let third_input = match third_input_options.iter().find(|option| option.input == specific_third_input) {
            Some(option) => option,
            None => return Output::InvalidInput,
        };
    
        let (forth_events, forth_input_options) = match self.process_third_input(*third_input, start_item, start_location, target_location) {
            Some((events, options)) => (events, options),
            None => (vec![], vec![]),
        };
        events.extend(forth_events);
    
        if forth_input_options.is_empty() && !events.is_empty() {
            return Output::Events(if do_not_apply_events { events } else { self.apply_and_add_resulting_events(events) });
        } else {
            return Output::InvalidInput;
        }
    }
    
    // MARK: - process step by step

    fn suggested_input_to_start_with(&self) -> Output {
        let locations_filter = |location: &Location| -> Option<Location> {
            let output = self.process_input(&[InputOption::Location(*location)], true, true);
            if let Output::NextInputOptions(options) = output {
                if !options.is_empty() {
                    Some(*location)
                } else {
                    None
                }
            } else {
                None
            }
        };

        let mut suggested_locations: Vec<Location> = self.board.all_mons_locations(self.active_color)
            .iter()
            .filter_map(locations_filter)
            .collect();
        
        if (!self.player_can_move_mon() && !self.player_can_use_action() || suggested_locations.is_empty()) && self.player_can_move_mana() {
            suggested_locations.append(
                &mut self.board.all_free_regular_mana_locations(self.active_color)
                    .iter()
                    .filter_map(locations_filter)
                    .collect(),
            );
        }

        if suggested_locations.is_empty() {
            Output::InvalidInput
        } else {
            Output::LocationsToStartFrom(suggested_locations)
        }
    }

    fn second_input_options(&self, start_location: Location, start_item: &Item, only_one: bool, specific_next: Option<Input>) -> Vec<NextInput> {
        let specific_location = match specific_next {
            Some(Input::Location(location)) => Some(location),
            _ => None,
        };
    
        let start_square = self.board.square(start_location);
        let mut second_input_options = Vec::new();
        match start_item {
            Item::Mon { mon } if mon.color == self.active_color && !mon.is_fainted => {
                if self.player_can_move_mon() {
                    second_input_options.extend(
                        self.next_inputs(start_location.nearby_locations(), AvailableMoveKind::MonMove, only_one, specific_location, |location| {
                            let item = self.board.item(location);
                            let square = self.board.square(location);
    
                            match item {
                                Some(Item::Mon { .. }) | Some(Item::MonWithMana { .. }) | Some(Item::MonWithConsumable { .. }) => false,
                                Some(Item::Mana { mana }) => !(mon.kind != MonKind::Drainer && *mana != Mana::Regular(mon.color)),
                                Some(Item::Consumable { .. }) => true,
                                None => match square {
                                    Square::Regular | Square::ConsumableBase | Square::ManaBase | Square::ManaPool => true,
                                    Square::SupermanaBase => matches!(item, Some(Item::Mana { mana: Mana::Supermana }) && mon.kind == MonKind::Drainer),
                                    Square::MonBase { kind, color } => mon.kind == *kind && mon.color == *color,
                                },
                            }
                        }),
                    );
                }
    
                if !matches!(start_square, Square::MonBase { .. }) && self.player_can_use_action() {
                    match mon.kind {
                        MonKind::Angel | MonKind::Drainer => (),
                        MonKind::Mystic => {
                            second_input_options.extend(
                                self.next_inputs(start_location.reachable_by_mystic_action(), AvailableMoveKind::Action, only_one, specific_location, |location| {
                                    if let Some(item) = self.board.item(location) {
                                        if self.protected_by_opponents_angel.contains(&location) {
                                            return false;
                                        }
    
                                        match item {
                                            Item::Mon { mon: target_mon } | Item::MonWithMana { mon: target_mon, .. } | Item::MonWithConsumable { mon: target_mon, .. } => {
                                                mon.color != target_mon.color && !target_mon.is_fainted
                                            }
                                            _ => false,
                                        }
                                    } else {
                                        false
                                    }
                                }),
                            );
                        }
                        MonKind::Demon => {
                            second_input_options.extend(
                                self.next_inputs(start_location.reachable_by_demon_action(), AvailableMoveKind::Action, only_one, specific_location, |location| {
                                    if let Some(item) = self.board.item(location) {
                                        if self.protected_by_opponents_angel.contains(&location) || self.board.item(start_location.location_between(location)).is_some() {
                                            return false;
                                        }
    
                                        match item {
                                            Item::Mon { mon: target_mon } | Item::MonWithMana { mon: target_mon, .. } | Item::MonWithConsumable { mon: target_mon, .. } => {
                                                mon.color != target_mon.color && !target_mon.is_fainted
                                            }
                                            _ => false,
                                        }
                                    } else {
                                        false
                                    }
                                }),
                            );
                        }
                        MonKind::Spirit => {
                            second_input_options.extend(
                                self.next_inputs(start_location.reachable_by_spirit_action(), AvailableMoveKind::Action, only_one, specific_location, |location| {
                                    self.board.item(location).is_some()
                                }),
                            );
                        }
                        _ => (),
                    }
                }
            }
            Item::Mana { mana } if matches!(mana, Mana::Regular(color) if color == &self.active_color) && self.player_can_move_mana() => {
                second_input_options.extend(
                    self.next_inputs(start_location.nearby_locations(), AvailableMoveKind::ManaMove, only_one, specific_location, |location| {
                        let item = self.board.item(location);
                        let square = self.board.square(location);
    
                        match item {
                            Some(Item::Mon { mon }) => mon.kind == MonKind::Drainer,
                            Some(Item::MonWithConsumable { .. }) | Some(Item::Consumable { .. }) | Some(Item::MonWithMana { .. }) | Some(Item::Mana { .. }) => false,
                            None => matches!(square, Square::Regular | Square::ConsumableBase | Square::ManaBase | Square::ManaPool),
                        }
                    }),
                );
            }
            Item::MonWithMana { mon, mana } if mon.color == self.active_color && self.player_can_move_mon() => {
                second_input_options.extend(
                    self.next_inputs(start_location.nearby_locations(), AvailableMoveKind::MonMove, only_one, specific_location, |location| {
                        let item = self.board.item(location);
                        let square = self.board.square(location);
    
                        match item {
                            Some(Item::Mon { .. }) | Some(Item::MonWithMana { .. }) | Some(Item::MonWithConsumable { .. }) => false,
                            Some(Item::Consumable { .. }) | Some(Item::Mana { .. }) => true,
                            None => match square {
                                Square::Regular | Square::ConsumableBase | Square::ManaBase | Square::ManaPool => true,
                                Square::SupermanaBase => *mana == Mana::Supermana || matches!(item, Some(Item::Mana { mana: Mana::Supermana })),
                                Square::MonBase { .. } => false,
                            },
                        }
                    }),
                );
            }
            Item::MonWithConsumable { mon, consumable } if mon.color == self.active_color => {
                if self.player_can_move_mon() {
                    second_input_options.extend(
                        self.next_inputs(start_location.nearby_locations(), AvailableMoveKind::MonMove, only_one, specific_location, |location| {
                            let item = self.board.item(location);
                            let square = self.board.square(location);
    
                            match item {
                                Some(Item::Mon { .. }) | Some(Item::Mana { .. }) | Some(Item::MonWithMana { .. }) | Some(Item::MonWithConsumable { .. }) => false,
                                Some(Item::Consumable { .. }) => true,
                                None => matches!(square, Square::Regular | Square::ConsumableBase | Square::ManaBase | Square::ManaPool),
                            }
                        }),
                    );
                }
    
                if matches!(consumable, Consumable::Bomb) {
                    second_input_options.extend(
                        self.next_inputs(start_location.reachable_by_bomb(), AvailableMoveKind::Action, only_one, specific_location, |location| {
                            self.board.item(location).map_or(false, |item| {
                                match item {
                                    Item::Mon { mon: target_mon } | Item::MonWithMana { mon: target_mon, .. } | Item::MonWithConsumable { mon: target_mon, .. } => {
                                        mon.color != target_mon.color && !target_mon.is_fainted
                                    }
                                    _ => false,
                                }
                            })
                        }),
                    );
                }
            }
            _ => (),
        }
    
        second_input_options
    }
    
    fn process_second_input(&mut self, kind: NextInputKind, start_item: Item, start_location: Location, target_location: Location, specific_next: Option<Input>) -> Option<(Vec<Event>, Vec<NextInput>)> {
        let specific_location = match specific_next {
            Some(Input::Location(location)) => Some(location),
            _ => None,
        };
    
        let mut third_input_options = Vec::new();
        let mut events = Vec::new();
        let target_square = self.board.square(target_location);
        let target_item = self.board.item(target_location);
    
        match kind {
            NextInputKind::MonMove => {
                let start_mon = match start_item {
                    Item::Mon { mon } => mon,
                    _ => return None,
                };
                events.push(Event::MonMove {
                    item: start_item,
                    from: start_location,
                    to: target_location,
                });
    
                if let Some(target_item) = target_item {
                    match target_item {
                        Item::Mon { .. } | Item::MonWithMana { .. } | Item::MonWithConsumable { .. } => return None,
                        Item::Mana { mana } => {
                            if let Item::Mana { mana: start_mana } = start_item {
                                match start_mana {
                                    Mana::Supermana => events.push(Event::SupermanaBackToBase {
                                        from: start_location,
                                        to: self.board.supermana_base(),
                                    }),
                                    _ => events.push(Event::ManaDropped {
                                        mana: start_mana,
                                        at: start_location,
                                    }),
                                }
                            }
                            events.push(Event::PickupMana {
                                mana,
                                by: start_mon,
                                at: target_location,
                            });
                        },
                        Item::Consumable { consumable } => match consumable {
                            Consumable::Bomb | Consumable::Potion => return None,
                            Consumable::BombOrPotion => {
                                if start_item.consumable().is_some() || start_item.mana().is_some() {
                                    events.push(Event::PickupPotion {
                                        by: start_item,
                                        at: target_location,
                                    });
                                } else {
                                    third_input_options.push(NextInput {
                                        input: Input::Modifier(Modifier::SelectBomb),
                                        kind: NextInputKind::SelectConsumable,
                                        actor_mon_item: Some(start_item),
                                    });
                                    third_input_options.push(NextInput {
                                        input: Input::Modifier(Modifier::SelectPotion),
                                        kind: NextInputKind::SelectConsumable,
                                        actor_mon_item: Some(start_item),
                                    });
                                }
                            },
                        },
                    }
                }
    
                match target_square {
                    Square::Regular | Square::ConsumableBase | Square::SupermanaBase | Square::ManaBase | Square::MonBase { .. } => (),
                    Square::ManaPool { .. } => {
                        if let Item::Mana { mana } = start_item {
                            events.push(Event::ManaScored {
                                mana,
                                at: target_location,
                            });
                        }
                    },
                }
            },
            NextInputKind::ManaMove => {
                let mana = match start_item {
                    Item::Mana { mana } => mana,
                    _ => return None,
                };
                events.push(Event::ManaMove {
                    mana,
                    from: start_location,
                    to: target_location,
                });
    
                if let Some(target_item) = target_item {
                    match target_item {
                        Item::Mon { mon } => {
                            events.push(Event::PickupMana {
                                mana,
                                by: mon,
                                at: target_location,
                            });
                        },
                        Item::Mana { .. } | Item::Consumable { .. } | Item::MonWithMana { .. } | Item::MonWithConsumable { .. } => return None,
                    }
                }
    
                match target_square {
                    Square::ManaBase { .. } | Square::ConsumableBase | Square::Regular => (),
                    Square::ManaPool { .. } => {
                        events.push(Event::ManaScored {
                            mana,
                            at: target_location,
                        });
                    },
                    Square::MonBase { .. } | Square::SupermanaBase => return None,
                }
            },
            NextInputKind::MysticAction => {
                let start_mon = if let Item::Mon { mon } = start_item {
                    mon
                } else {
                    return None;
                };
                events.push(Event::MysticAction {
                    mystic: start_mon,
                    from: start_location,
                    to: target_location,
                });
    
                if let Some(target_item) = target_item {
                    match target_item {
                        Item::Mon { mon: target_mon } | Item::MonWithMana { mon: target_mon, .. } | Item::MonWithConsumable { mon: target_mon, .. } => {
                            events.push(Event::MonFainted {
                                mon: target_mon,
                                from: target_location,
                                to: self.board.base(mon: target_mon),
                            });
    
                            if let Item::MonWithMana { mana, .. } = target_item {
                                match mana {
                                    Mana::Regular => events.push(Event::ManaDropped { mana, at: target_location }),
                                    Mana::Supermana => events.push(Event::SupermanaBackToBase {
                                        from: target_location,
                                        to: self.board.supermana_base(),
                                    }),
                                }
                            }
    
                            if let Item::MonWithConsumable { consumable, .. } = target_item {
                                match consumable {
                                    Consumable::Bomb => {
                                        events.push(Event::BombExplosion { at: target_location });
                                    },
                                    Consumable::Potion | Consumable::BombOrPotion => return None,
                                }
                            }
                        },
                        Item::Consumable { .. } | Item::Mana { .. } => return None,
                    }
                }
            },
            NextInputKind::DemonAction => {
                let start_mon = if let Item::Mon { mon } = start_item {
                    mon
                } else {
                    return None;
                };
                events.push(Event::DemonAction {
                    demon: start_mon,
                    from: start_location,
                    to: target_location,
                });
                let mut requires_additional_step = false;
    
                if let Some(target_item) = target_item {
                    match target_item {
                        Item::Mana { .. } | Item::Consumable { .. } => return None,
                        Item::Mon { mon: target_mon } | Item::MonWithMana { mon: target_mon, .. } | Item::MonWithConsumable { mon: target_mon, .. } => {
                            events.push(Event::MonFainted {
                                mon: target_mon,
                                from: target_location,
                                to: self.board.base(mon: target_mon),
                            });
    
                            if let Item::MonWithMana { mana, .. } = target_item {
                                match mana {
                                    Mana::Regular => {
                                        requires_additional_step = true;
                                        events.push(Event::ManaDropped { mana, at: target_location });
                                    },
                                    Mana::Supermana => events.push(Event::SupermanaBackToBase {
                                        from: target_location,
                                        to: self.board.supermana_base(),
                                    }),
                                }
                            }
    
                            if let Item::MonWithConsumable { consumable, .. } = target_item {
                                match consumable {
                                    Consumable::Bomb => {
                                        events.push(Event::BombExplosion { at: target_location });
                                        events.push(Event::MonFainted {
                                            mon: start_mon,
                                            from: target_location,
                                            to: self.board.base(mon: start_mon),
                                        });
                                    },
                                    Consumable::Potion | Consumable::BombOrPotion => return None,
                                }
                            }
                        },
                    }
                }
    
                match target_square {
                    Square::Regular | Square::ConsumableBase | Square::ManaBase | Square::ManaPool => (),
                    Square::SupermanaBase | Square::MonBase { .. } => requires_additional_step = true,
                }
    
                if requires_additional_step {
                    let nearby_locations = target_location.nearby_locations(); // Assuming this is a method that returns a Vec<Location>
                    for location in nearby_locations.iter() {
                        let item = self.board.item(*location);
                        let square = self.board.square(*location);
                
                        let is_valid_location = match item {
                            Some(Item::Mon { .. }) | Some(Item::Mana { .. }) | Some(Item::MonWithMana { .. }) | Some(Item::MonWithConsumable { .. }) => false,
                            Some(Item::Consumable { .. }) | None => true,
                            _ => false,
                        };
                
                        if is_valid_location {
                            match square {
                                Square::Regular | Square::ConsumableBase | Square::ManaBase | Square::ManaPool => {
                                    third_input_options.push(NextInput {
                                        input: Input::Location(*location),
                                        kind: NextInputKind::DemonAdditionalStep,
                                        actor_mon_item: Some(start_item),
                                    });
                                },
                                Square::MonBase { kind, color } if kind == start_mon.kind && color == start_mon.color => {
                                    third_input_options.push(NextInput {
                                        input: Input::Location(*location),
                                        kind: NextInputKind::DemonAdditionalStep,
                                        actor_mon_item: Some(start_item),
                                    });
                                },
                                Square::SupermanaBase => (),
                                _ => (),
                            }
                        }
                    }
                }
                
            },
            NextInputKind::SpiritTargetCapture => {
                if target_item.is_none() { return None; }
    let target_mon = target_item.as_ref().and_then(|item| item.mon());
    let target_mana = target_item.as_ref().and_then(|item| item.mana());

    let nearby_locations = target_location.nearby_locations(); // This method needs to be defined to get nearby locations.
    for location in nearby_locations.iter() {
        let destination_item = self.board.item(*location);
        let destination_square = self.board.square(*location);

        let valid_destination = match &destination_item {
            Some(Item::Mon { mon: destination_mon }) => match &target_item {
                Some(Item::Mon { .. }) | Some(Item::MonWithMana { .. }) | Some(Item::MonWithConsumable { .. }) => false,
                Some(Item::Mana { .. }) => destination_mon.kind != MonKind::Drainer || destination_mon.is_fainted(),
                Some(Item::Consumable { consumable: target_consumable }) => *target_consumable != Consumable::BombOrPotion,
                None => false,
            },
            Some(Item::Mana { .. }) => matches!(target_item, Some(Item::Mon { mon: target_mon }) if target_mon.kind == MonKind::Drainer && !target_mon.is_fainted()),
            Some(Item::MonWithMana { .. }) | Some(Item::MonWithConsumable { .. }) => false,
            Some(Item::Consumable { consumable: destination_consumable }) => matches!(target_item, Some(Item::Mon { .. }) | Some(Item::MonWithMana { .. }) | Some(Item::MonWithConsumable { .. }) if *destination_consumable == Consumable::BombOrPotion),
            None => true,
        };

        if valid_destination {
            match destination_square {
                Square::Regular | Square::ConsumableBase | Square::ManaBase | Square::ManaPool => (),
                Square::SupermanaBase => {
                    if target_mana == Some(&Mana::Supermana) || (matches!(target_mon.map(|mon| mon.kind), Some(MonKind::Drainer)) && matches!(destination_item, Some(Item::Mana { mana: Mana::Supermana }))) {
                        third_input_options.push(NextInput {
                            input: Input::Location(*location),
                            kind: NextInputKind::SpiritTargetMove,
                            actor_mon_item: target_item.clone(),
                        });
                    }
                },
                Square::MonBase { kind, color } => {
                    if let Some(mon) = target_mon {
                        if mon.kind == *kind && mon.color == *color && target_mana.is_none() && target_item.as_ref().and_then(|item| item.consumable()).is_none() {
                            third_input_options.push(NextInput {
                                input: Input::Location(*location),
                                kind: NextInputKind::SpiritTargetMove,
                                actor_mon_item: target_item.clone(),
                            });
                        }
                    }
                },
                _ => (),
            }
        }
    }
            },
            NextInputKind::BombAttack => {
                let start_mon = if let Some(Item::Mon { mon }) = start_item {
                    mon
                } else {
                    return None;
                };
            
                events.push(Event::BombAttack {
                    by: start_mon.clone(),
                    from: start_location,
                    to: target_location,
                });
            
                match target_item {
                    Some(Item::Mon { mon }) | Some(Item::MonWithMana { mon, .. }) | Some(Item::MonWithConsumable { mon, .. }) => {
                        events.push(Event::MonFainted {
                            mon: mon.clone(),
                            from: target_location,
                            to: self.board.base(&mon),
                        });
            
                        if let Some(Item::MonWithMana { mana, .. }) = target_item {
                            match mana {
                                Mana::Regular => events.push(Event::ManaDropped {
                                    mana: *mana,
                                    at: target_location,
                                }),
                                Mana::Supermana => events.push(Event::SupermanaBackToBase {
                                    from: target_location,
                                    to: self.board.supermana_base(),
                                }),
                            }
                        }
            
                        if let Some(Item::MonWithConsumable { consumable, .. }) = target_item {
                            match consumable {
                                Consumable::Bomb => {
                                    events.push(Event::BombExplosion {
                                        at: target_location,
                                    });
                                },
                                Consumable::Potion | Consumable::BombOrPotion => return None,
                            }
                        }
                    },
                    Some(Item::Mana { .. }) | Some(Item::Consumable { .. }) => return None,
                    _ => (),
                }
            },
            _ => (),
        }
    
        Some((events, third_input_options))
    }
    
    fn process_third_input(&mut self, third_input: NextInput, start_item: Item, start_location: Location, target_location: Location) -> Option<(Vec<Event>, Vec<NextInput>)> {
        let target_item = self.board.item(target_location);
        let mut forth_input_options = Vec::new();
        let mut events = Vec::new();
    
        match third_input.kind {
            AvailableMoveKind::MonMove | AvailableMoveKind::ManaMove | AvailableMoveKind::Action | AvailableMoveKind::Potion => return None,
            AvailableMoveKind::SpiritTargetMove => {
                if let Input::Location(destination_location) = third_input.input {
                    if let Some(target_item) = target_item {
                        let destination_item = self.board.item(destination_location);
                        let destination_square = self.board.square(destination_location);
    
                        events.push(Event::SpiritTargetMove { item: target_item, from: target_location, to: destination_location });
    
                        if let Some(destination_item) = destination_item {
                            match target_item {
                                Item::Mon { mon: travelling_mon } => match destination_item {
                                    Item::Mon { .. } | Item::MonWithMana { .. } | Item::MonWithConsumable { .. } => return None,
                                    Item::Mana { mana: destination_mana } => {
                                        events.push(Event::PickupMana { mana: destination_mana, by: travelling_mon, at: destination_location });
                                    },
                                    Item::Consumable { consumable: destination_consumable } => match destination_consumable {
                                        Consumable::Potion | Consumable::Bomb => return None,
                                        Consumable::BombOrPotion => {
                                            forth_input_options.push(NextInput { input: Input::Modifier(Modifier::SelectBomb), kind: AvailableMoveKind::SelectConsumable, actor_mon_item: Some(target_item) });
                                            forth_input_options.push(NextInput { input: Input::Modifier(Modifier::SelectPotion), kind: AvailableMoveKind::SelectConsumable, actor_mon_item: Some(target_item) });
                                        },
                                    },
                                },
                                Item::Mana { mana: travelling_mana } => match destination_item {
                                    Item::Mana { .. } | Item::MonWithMana { .. } | Item::MonWithConsumable { .. } | Item::Consumable { .. } => return None,
                                    Item::Mon { mon: destination_mon } => {
                                        events.push(Event::PickupMana { mana: travelling_mana, by: destination_mon, at: destination_location });
                                    },
                                },
                                Item::MonWithMana { .. } => match destination_item {
                                    Item::Mon { .. } | Item::Mana { .. } | Item::MonWithMana { .. } | Item::MonWithConsumable { .. } => return None,
                                    Item::Consumable { consumable: destination_consumable } => match destination_consumable {
                                        Consumable::Potion | Consumable::Bomb => return None,
                                        Consumable::BombOrPotion => {
                                            events.push(Event::PickupPotion { by: target_item, at: destination_location });
                                        },
                                    },
                                },
                                Item::MonWithConsumable { .. } => match destination_item {
                                    Item::Mon { .. } | Item::Mana { .. } | Item::MonWithMana { .. } | Item::MonWithConsumable { .. } => return None,
                                    Item::Consumable { consumable: destination_consumable } => match destination_consumable {
                                        Consumable::Potion | Consumable::Bomb => return None,
                                        Consumable::BombOrPotion => {
                                            events.push(Event::PickupPotion { by: target_item, at: destination_location });
                                        },
                                    },
                                },
                                Item::Consumable { consumable: travelling_consumable } => match destination_item {
                                    Item::Mana { .. } | Item::Consumable { .. } => return None,
                                    Item::Mon { .. } => {
                                        forth_input_options.push(NextInput { input: Input::Modifier(Modifier::SelectBomb), kind: AvailableMoveKind::SelectConsumable, actor_mon_item: Some(destination_item) });
                                        forth_input_options.push(NextInput { input: Input::Modifier(Modifier::SelectPotion), kind: AvailableMoveKind::SelectConsumable, actor_mon_item: Some(destination_item) });
                                    },
                                    Item::MonWithMana { .. } | Item::MonWithConsumable { .. } => match travelling_consumable {
                                        Consumable::Potion | Consumable::Bomb => return None,
                                        Consumable::BombOrPotion => {
                                            events.push(Event::PickupPotion { by: destination_item, at: destination_location });
                                        },
                                    },
                                },
                            }
                        }
    
                        if matches!(destination_square, Square::ManaPool { .. }) {
                            if let Some(mana) = target_item.mana() {
                                events.push(Event::ManaScored { mana, at: destination_location });
                            }
                        }
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            },
            AvailableMoveKind::DemonAdditionalStep => {
                if let Input::Location(destination_location) = third_input.input {
                    if let Some(demon) = start_item.mon() {
                        events.push(Event::DemonAdditionalStep { demon, from: target_location, to: destination_location });
    
                        if let Some(item) = self.board.item(destination_location) {
                            if let Item::Consumable { consumable } = item {
                                match consumable {
                                    Consumable::Potion | Consumable::Bomb => return None,
                                    Consumable::BombOrPotion => {
                                        forth_input_options.push(NextInput { input: Input::Modifier(Modifier::SelectBomb), kind: AvailableMoveKind::SelectConsumable, actor_mon_item: Some(start_item) });
                                        forth_input_options.push(NextInput { input: Input::Modifier(Modifier::SelectPotion), kind: AvailableMoveKind::SelectConsumable, actor_mon_item: Some(start_item) });
                                    },
                                }
                            }
                        }
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            },
            AvailableMoveKind::SelectConsumable => {
                if let Input::Modifier(modifier) = third_input.input {
                    if let Some(mon) = start_item.mon() {
                        match modifier {
                            Modifier::SelectBomb => {
                                events.push(Event::PickupBomb { by: mon, at: target_location });
                            },
                            Modifier::SelectPotion => {
                                events.push(Event::PickupPotion { by: start_item, at: target_location });
                            },
                            Modifier::Cancel => return None,
                        }
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            },
        }
    
        Some((events, forth_input_options))
    }

    // MARK: - apply events

    pub fn apply_and_add_resulting_events(&mut self, events: Vec<Event>) -> Vec<Event> {
        let mut extra_events = Vec::new();
        for event in &events {
            match event {
                Event::MonMove { item, from, to } => {
                    self.mons_moves_count += 1;
                    self.board.remove_item(*from);
                    self.board.put(item.clone(), *to);
                }
                Event::ManaMove { mana, from, to } => {
                    self.mana_moves_count += 1;
                    self.board.remove_item(*from);
                    self.board.put(Item::Mana { mana: *mana }, *to);
                }
                Event::ManaScored { mana, at } => {
                    let score = mana.score(self.active_color);
                    if self.active_color == Color::White {
                        self.white_score += score;
                    } else {
                        self.black_score += score;
                    }
                    if let Some(item) = self.board.item(*at) {
                        match item {
                            Item::Mon { mon } => {
                                self.board.put(Item::Mon { mon: mon.clone() }, *at);
                            },
                            _ => {
                                self.board.remove_item(*at);
                            }
                        }
                    }
                }
                Event::MysticAction { mystic, from, to } => {
                    if self.actions_used_count >= Config::ACTIONS_PER_TURN {
                        if self.active_color == Color::White {
                            self.white_potions_count -= 1;
                        } else {
                            self.black_potions_count -= 1;
                        }
                    } else {
                        self.actions_used_count += 1;
                    }
                    self.board.remove_item(*to);
                }
                Event::DemonAction { demon, from, to } => {
                    if self.actions_used_count >= Config::ACTIONS_PER_TURN {
                        if self.active_color == Color::White {
                            self.white_potions_count -= 1;
                        } else {
                            self.black_potions_count -= 1;
                        }
                    } else {
                        self.actions_used_count += 1;
                    }
                    self.board.remove_item(*from);
                    self.board.put(Item::Mon { mon: demon.clone() }, *to);
                }
                Event::DemonAdditionalStep { demon, from: _, to } => {
                    self.board.put(Item::Mon { mon: demon.clone() }, *to);
                }
                Event::SpiritTargetMove { item, from, to } => {
                    if self.actions_used_count >= Config::ACTIONS_PER_TURN {
                        if self.active_color == Color::White {
                            self.white_potions_count -= 1;
                        } else {
                            self.black_potions_count -= 1;
                        }
                    } else {
                        self.actions_used_count += 1;
                    }
                    self.board.remove_item(*from);
                    self.board.put(item.clone(), *to);
                }
                Event::PickupBomb { by, at } => {
                    self.board.put(Item::MonWithConsumable { mon: by.clone(), consumable: Consumable::Bomb }, *at);
                }
                Event::PickupPotion { by, at } => {
                    let mon_color = if let Item::Mon { mon } = by {
                        mon.color
                    } else {
                        continue;
                    };
                    if mon_color == Color::White {
                        self.white_potions_count += 1;
                    } else {
                        self.black_potions_count += 1;
                    }
                    self.board.put(by.clone(), *at);
                }
                Event::PickupMana { mana, by, at } => {
                    self.board.put(Item::MonWithMana { mon: by.clone(), mana: *mana }, *at);
                }
                Event::MonFainted { mon, from: _, to } => {
                    let mut fainted_mon = mon.clone();
                    fainted_mon.faint();
                    self.board.put(Item::Mon { mon: fainted_mon }, *to);
                }
                Event::ManaDropped { mana, at } => {
                    self.board.put(Item::Mana { mana: *mana }, *at);
                }
                Event::SupermanaBackToBase { from: _, to } => {
                    self.board.put(Item::Mana { mana: Mana::Supermana }, *to);
                }
                Event::BombAttack { by, from, to } => {
                    self.board.remove_item(*to);
                    self.board.put(Item::Mon { mon: by.clone() }, *from);
                }
                Event::BombExplosion { at } => {
                    self.board.remove_item(*at);
                }
                Event::MonAwake { mon, at } => {
                    self.board.put(Item::Mon { mon: mon.clone() }, *at);
                }
                Event::GameOver { winner } => extra_events.push(Event::GameOver { winner: winner.clone() }),
                Event::NextTurn { color } => {
                    self.active_color = *color;
                    self.reset_turn_state();
                    for mon_location in self.board.fainted_mons_locations(self.active_color) {
                        if let Some(Item::Mon { mon }) = self.board.item(mon_location) {
                            let mut awake_mon = mon.clone();
                            awake_mon.decrease_cooldown();
                            self.board.put(Item::Mon { mon: awake_mon.clone() }, mon_location);
                            if !awake_mon.is_fainted() {
                                extra_events.push(Event::MonAwake { mon: awake_mon, at: mon_location });
                            }
                        }
                    }
                }
            }
        }
    
        if let Some(winner) = self.winner_color() {
            extra_events.push(Event::GameOver { winner });
        } else if self.is_first_turn() && !self.player_can_move_mon() ||
                  !self.is_first_turn() && (!self.player_can_move_mana() || !self.player_can_move_mon() && self.board.find_mana(self.active_color).is_none()) {
            self.active_color = self.active_color.other();
            self.turn_number += 1;
            self.reset_turn_state();
            extra_events.push(Event::NextTurn { color: self.active_color });
        }
    
        events.into_iter().chain(extra_events.into_iter()).collect()
    }
    
    fn reset_turn_state(&mut self) {
        self.actions_used_count = 0;
        self.mana_moves_count = 0;
        self.mons_moves_count = 0;
    }

    // MARK: - helpers
    pub fn next_inputs<F>(&self, locations: Vec<Location>, kind: NextInputKind, only_one: bool, specific: Option<Location>, filter: F) -> Vec<NextInput> where F: Fn(Location) -> bool {
        if let Some(specific_location) = specific {
            if locations.contains(&specific_location) && filter(specific_location) {
                return vec![NextInput { input: Input::Location(specific_location), kind, actor_mon_item: None }];
            } else {
                return vec![];
            }
        } else if only_one {
            if let Some(one) = locations.into_iter().find(|&loc| filter(loc)) {
                return vec![NextInput { input: Input::Location(one), kind, actor_mon_item: None }];
            } else {
                return vec![];
            }
        } else {
            return locations.into_iter().filter_map(|loc| {
                if filter(loc) {
                    Some(NextInput { input: Input::Location(loc), kind, actor_mon_item: None })
                } else {
                    None
                }
            }).collect();
        }
    }

    pub fn available_move_kinds(&self) -> HashMap<AvailableMoveKind, i32> {
        let mut moves = HashMap::new();
        moves.insert(AvailableMoveKind::MonMove, Config::MONS_MOVES_PER_TURN - self.mons_moves_count);
        moves.insert(AvailableMoveKind::Action, 0);
        moves.insert(AvailableMoveKind::Potion, 0);
        moves.insert(AvailableMoveKind::ManaMove, 0);

        if self.turn_number == 1 {
            return moves;
        }

        moves.insert(AvailableMoveKind::Action, Config::ACTIONS_PER_TURN - self.actions_used_count);
        moves.insert(AvailableMoveKind::Potion, self.player_potions_count());
        moves.insert(AvailableMoveKind::ManaMove, Config::MANA_MOVES_PER_TURN - self.mana_moves_count);

        moves
    }

    pub fn winner_color(&self) -> Option<Color> {
        if self.white_score >= Config::TARGET_SCORE {
            Some(Color::White)
        } else if self.black_score >= Config::TARGET_SCORE {
            Some(Color::Black)
        } else {
            None
        }
    }

    pub fn is_later_than(&self, game: &MonsGame) -> bool {
        if self.turn_number > game.turn_number {
            true
        } else if self.turn_number == game.turn_number {
            self.player_potions_count() < game.player_potions_count() ||
            self.actions_used_count > game.actions_used_count ||
            self.mana_moves_count > game.mana_moves_count ||
            self.mons_moves_count > game.mons_moves_count ||
            self.board.fainted_mons_locations(self.active_color.other()).len() > game.board.fainted_mons_locations(game.active_color.other()).len()
        } else {
            false
        }
    }

    pub fn is_first_turn(&self) -> bool {
        self.turn_number == 1 
    }

    pub fn player_potions_count(&self) -> i32 {
        match self.active_color {
            Color::White => self.white_potions_count,
            Color::Black => self.black_potions_count,
        }
    }

    pub fn player_can_move_mon(&self) -> bool {
        self.mons_moves_count < Config::MONS_MOVES_PER_TURN 
    }

    pub fn player_can_move_mana(&self) -> bool {
        !self.is_first_turn() && self.mana_moves_count < Config::MANA_MOVES_PER_TURN 
    }

    pub fn player_can_use_action(&self) -> bool {
        !self.is_first_turn() && (self.player_potions_count() > 0 || self.actions_used_count < Config::ACTIONS_PER_TURN) 
    }

    pub fn protected_by_opponents_angel(&self) -> std::collections::HashSet<Location> {
        if let Some(location) = self.board.find_awake_angel(self.active_color.other()) {
            let protected: Vec<Location> = location.nearby_locations(1);
            protected.into_iter().collect()
        } else {
            std::collections::HashSet::new()
        }
    }
}
