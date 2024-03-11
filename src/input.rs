#[derive(Debug, PartialEq, Eq, Hash, Clone)]
enum Input {
    Location(Location),
    Modifier(Modifier),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
enum Modifier {
    SelectPotion,
    SelectBomb,
    Cancel,
}
