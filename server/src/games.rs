use crate::{
    hits::{Hit, Pack},
    users::User,
};
use rocket::serde::json::Json;
use rocket_okapi::okapi::{schemars, schemars::JsonSchema};
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, convert::From, default::Default};

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct SlotPayload {
    pub id: Option<u8>,
}

#[derive(Deserialize, Serialize, JsonSchema, Clone, Eq, PartialEq, Debug)]
pub struct GameSettingsPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hit_duration: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_tokens: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub packs: Option<Vec<Pack>>,
}

impl From<Json<GameSettingsPayload>> for GameSettingsPayload {
    fn from(src: Json<GameSettingsPayload>) -> Self {
        Self {
            goal: src.goal,
            hit_duration: src.hit_duration,
            start_tokens: src.start_tokens,
            packs: src.packs.clone(),
        }
    }
}

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct ConfirmationPayload {
    pub confirm: bool,
}

#[derive(Deserialize, Serialize, JsonSchema, Clone, Eq, PartialEq, Debug, Copy)]
#[serde(rename_all_fields = "snake_case")]
pub enum GameState {
    /// the game is currently accepting new players
    Open,
    /// the player has to guess, a song is currently available for playback
    Guessing,
    Intercepting,
    /// a different player has to confirm the choices
    Confirming,
}

#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize, JsonSchema)]
pub struct Game {
    pub id: u32,
    pub players: Vec<Player>,
    pub state: GameState,
    #[serde(skip)]
    pub hits_remaining: VecDeque<Hit>,
    pub hit_duration: u8,
    pub start_tokens: u8,
    pub goal: u8,
    pub hit: Option<Hit>,
    pub packs: Vec<Pack>,
}

#[derive(Deserialize, Serialize, JsonSchema, Clone, Eq, PartialEq, Debug)]
#[serde(rename_all_fields = "snake_case")]
pub enum PlayerState {
    Waiting,
    Guessing,
    Intercepting,
    Confirming,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Eq, PartialEq, Debug)]
pub struct Player {
    pub id: u32,
    pub name: String,
    pub state: PlayerState,
    pub creator: bool,
    pub hits: Vec<Hit>,
    pub tokens: u8,
    pub slots: Vec<Slot>,
    pub turn_player: bool,
    pub guess: Option<Slot>,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Eq, PartialEq, Debug)]
pub struct Slot {
    pub from_year: u32,
    pub to_year: u32,
    pub id: u8,
}

impl From<&User> for Player {
    fn from(u: &User) -> Self {
        Self {
            id: u.id,
            name: u.username.clone(),
            state: PlayerState::Waiting,
            creator: false,
            hits: vec![],
            tokens: 0,
            slots: vec![],
            turn_player: false,
            guess: None,
        }
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Eq, PartialEq, Debug)]
pub struct GameEvent {
    #[serde(skip)]
    pub game_id: u32,
    #[serde(skip)]
    pub event: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub players: Option<Vec<Player>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<GameState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hit: Option<Hit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<GameSettingsPayload>,
}

impl Default for GameEvent {
    fn default() -> Self {
        Self {
            game_id: 0,
            event: "".into(),
            players: None,
            state: None,
            hit: None,
            settings: None,
        }
    }
}
