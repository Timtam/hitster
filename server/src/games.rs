use rocket_okapi::okapi::{schemars, schemars::JsonSchema};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, JsonSchema, Clone, Eq, PartialEq, Debug)]
#[serde(rename_all_fields = "snake_case")]
pub enum GameState {
    /// the game is currently accepting new players
    Open,
    /// the player has to guess, a song is currently available for playback
    Guessing,
    /// a different player has to confirm the choices
    Confirming,
}

#[derive(Deserialize, Serialize, JsonSchema, Clone, Eq, PartialEq, Debug)]
pub struct Game {
    pub id: u32,
    pub creator: u32,
    pub players: Vec<u32>,
    pub state: GameState,
    /// 0
    pub turn_player: u32,
}
