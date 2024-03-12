use crate::*;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Output {
    InvalidInput,
    LocationsToStartFrom(Vec<Location>),
    NextInputOptions(Vec<NextInput>),
    Events(Vec<Event>),
}