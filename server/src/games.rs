use crate::users::User;
use rocket_okapi::okapi::{schemars, schemars::JsonSchema};
use serde::{Deserialize, Serialize};
use std::{convert::From, default::Default};

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

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Game {
    pub id: u32,
    pub creator: usize,
    pub players: Vec<Player>,
    pub state: GameState,
    /// the player who has to guess the next hit
    pub guessing_player: usize,
    /// the player who'll confirm the correctness of title and interpret
    pub confirming_player: usize,
}

#[derive(Deserialize, Serialize, JsonSchema, Clone, Eq, PartialEq, Debug)]
pub struct Player {
    pub id: u32,
    pub name: String,
}

impl From<&User> for Player {
    fn from(u: &User) -> Self {
        Self {
            id: u.id,
            name: u.username.clone(),
        }
    }
}

#[derive(Deserialize, Serialize, JsonSchema, Clone, Eq, PartialEq, Debug)]
pub struct GameEvent {
    #[serde(skip)]
    pub game_id: u32,
    pub event: String,
    pub player: Option<Player>,
    pub state: Option<GameState>,
}

impl Default for GameEvent {
    fn default() -> Self {
        Self {
            game_id: 0,
            event: "".into(),
            player: None,
            state: None,
        }
    }
}
