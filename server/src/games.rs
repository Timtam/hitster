use super::users::User;
use rocket_okapi::okapi::{schemars, schemars::JsonSchema};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, JsonSchema, Clone, Eq, PartialEq, Debug)]
#[serde(rename_all_fields = "snake_case")]
pub enum GameState {
    /// the game is currently accepting new players
    Open,
}

#[derive(Deserialize, Serialize, JsonSchema, Clone, Eq, PartialEq, Debug)]
pub struct Game {
    pub id: u64,
    pub creator: User,
    pub players: Vec<User>,
    pub state: GameState,
}
