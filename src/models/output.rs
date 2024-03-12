use crate::*;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Output {
    InvalidInput,
    LocationsToStartFrom(Vec<Location>),
    NextInputOptions(Vec<NextInput>),
    Events(Vec<Event>),
}