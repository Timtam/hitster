use super::users::User;
use rocket_okapi::okapi::{schemars, schemars::JsonSchema};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, JsonSchema, Clone, Eq, PartialEq, Debug)]
pub struct Game {
    pub id: u64,
    pub creator: User,
    pub players: Vec<User>,
}
