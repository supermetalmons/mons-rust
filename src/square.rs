#[derive(Debug, PartialEq)]
enum Square {
    Regular,
    ConsumableBase,
    SupermanaBase,
    ManaBase(Color),
    ManaPool(Color),
    MonBase { kind: MonKind, color: Color },
}