#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Output {
    InvalidInput,
    LocationsToStartFrom(Vec<Location>),
    NextInputOptions(Vec<NextInput>),
    Events(Vec<Event>),
}

impl PartialEq for Output {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Output::InvalidInput, Output::InvalidInput) => true,
            (Output::LocationsToStartFrom(a), Output::LocationsToStartFrom(b)) => {
                let mut sa = a.clone();
                let mut sb = b.clone();
                sa.sort();
                sb.sort();
                sa == sb
            },
            (Output::NextInputOptions(a), Output::NextInputOptions(b)) => {
                let mut sa = a.clone();
                let mut sb = b.clone();
                sa.sort();
                sb.sort();
                sa == sb
            },
            (Output::Events(a), Output::Events(b)) => {
                let mut sa = a.clone();
                let mut sb = b.clone();
                sa.sort();
                sb.sort();
                sa == sb
            },
            _ => false,
        }
    }
}
