use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct NextInput {
    input: Input,
    kind: Kind,
    actor_mon_item: Option<Item>,
}

impl NextInput {
    fn new(input: Input, kind: Kind, actor_mon_item: Option<Item>) -> Self {
        Self {
            input,
            kind,
            actor_mon_item,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
enum Kind {
    MonMove,
    ManaMove,
    MysticAction,
    DemonAction,
    DemonAdditionalStep,
    SpiritTargetCapture,
    SpiritTargetMove,
    SelectConsumable,
    BombAttack,
}
