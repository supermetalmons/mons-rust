use crate::*;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
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
