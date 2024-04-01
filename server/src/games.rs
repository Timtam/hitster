use crate::{hits::Hit, users::User};
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
    pub hits_remaining: Vec<Hit>,
    pub hit_duration: u8,
}

#[derive(Deserialize, Serialize, JsonSchema, Clone, Eq, PartialEq, Debug)]
#[serde(rename_all_fields = "snake_case")]
pub enum PlayerState {
    Waiting,
    Guessing,
    Intercepting,
    Confirming,
}

#[derive(Deserialize, Serialize, JsonSchema, Clone, Eq, PartialEq, Debug)]
pub struct Player {
    pub id: u32,
    pub name: String,
    pub state: PlayerState,
}

impl From<&User> for Player {
    fn from(u: &User) -> Self {
        Self {
            id: u.id,
            name: u.username.clone(),
            state: PlayerState::Waiting,
        }
    }
}

#[derive(Deserialize, Serialize, JsonSchema, Clone, Eq, PartialEq, Debug)]
pub struct GameEvent {
    #[serde(skip)]
    pub game_id: u32,
    #[serde(skip)]
    pub event: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub players: Option<Vec<Player>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<GameState>,
}

impl Default for GameEvent {
    fn default() -> Self {
        Self {
            game_id: 0,
            event: "".into(),
            players: None,
            state: None,
        }
    }
}
