#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub enum AvailableMoveKind {
    MonMove,
    ManaMove,
    Action,
    Potion,
}
