use crate::*;

#[derive(Debug, PartialEq)]
pub enum Square {
    Regular,
    ConsumableBase,
    SupermanaBase,
    ManaBase { color: Color },
    ManaPool { color: Color },
    MonBase { kind: MonKind, color: Color },
}