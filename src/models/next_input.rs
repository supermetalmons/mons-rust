use crate::*;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum NextInputKind {
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

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct NextInput {
    pub input: Input,
    pub kind: NextInputKind,
    pub actor_mon_item: Option<Item>,
}

impl NextInput {
    pub fn new(input: Input, kind: NextInputKind, actor_mon_item: Option<Item>) -> Self {
        Self {
            input,
            kind,
            actor_mon_item,
        }
    }
}
