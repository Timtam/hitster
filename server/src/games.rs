use crate::{hits::HitPayload, users::User};
use hitster_core::Hit;
use rocket::serde::json::Json;
use rocket_okapi::okapi::{schemars, schemars::JsonSchema};
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, convert::From, default::Default};
use uuid::Uuid;

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
    pub remember_hits: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub packs: Option<Vec<Uuid>>,
}

#[derive(Deserialize, Serialize, JsonSchema, Clone, Eq, PartialEq, Debug)]
pub struct CreateGamePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<GameSettingsPayload>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<GameMode>,
}

impl From<Json<GameSettingsPayload>> for GameSettingsPayload {
    fn from(src: Json<GameSettingsPayload>) -> Self {
        Self {
            goal: src.goal,
            hit_duration: src.hit_duration,
            start_tokens: src.start_tokens,
            packs: src.packs.clone(),
            remember_hits: src.remember_hits,
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

#[derive(Deserialize, Serialize, JsonSchema, Clone, Eq, PartialEq, Debug, Copy)]
#[serde(rename_all_fields = "snake_case")]
pub enum GameMode {
    /// the game is available and visible to all users
    Public,
    /// the game is private, only the creator can see the game, but it can be joined by knowing the id or sharing the link
    Private,
    /// the game is private, only the creator can see and control the game
    Local,
}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, JsonSchema)]
pub struct Game {
    pub id: String,
    pub players: Vec<Player>,
    pub state: GameState,
    #[serde(skip)]
    pub hits_remaining: VecDeque<&'static Hit>,
    pub hit_duration: u8,
    pub start_tokens: u8,
    pub goal: u8,
    pub hit: Option<&'static Hit>,
    pub packs: Vec<Uuid>,
    pub mode: GameMode,
    pub remember_hits: bool,
    #[serde(skip)]
    pub remembered_hits: Vec<&'static Hit>,
    pub last_scored: Option<Player>,
}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, JsonSchema)]
pub struct GamePayload {
    pub id: String,
    pub players: Vec<Player>,
    pub state: GameState,
    pub hit_duration: u8,
    pub start_tokens: u8,
    pub goal: u8,
    pub hit: Option<HitPayload>,
    pub packs: Vec<Uuid>,
    pub mode: GameMode,
    pub remember_hits: bool,
    pub last_scored: Option<Player>,
}

impl From<&Game> for GamePayload {
    fn from(game: &Game) -> Self {
        Self {
            id: game.id.clone(),
            players: game.players.clone(),
            state: game.state,
            hit_duration: game.hit_duration,
            start_tokens: game.start_tokens,
            goal: game.goal,
            hit: game.hit.map(|h| h.into()),
            packs: game.packs.clone(),
            mode: game.mode,
            remember_hits: game.remember_hits,
            last_scored: game.last_scored.clone(),
        }
    }
}

#[derive(Deserialize, Serialize, JsonSchema, Clone, Eq, PartialEq, Debug)]
#[serde(rename_all_fields = "snake_case")]
pub enum PlayerState {
    Waiting,
    Guessing,
    Intercepting,
    Confirming,
}

#[derive(Serialize, JsonSchema, Clone, Eq, PartialEq, Debug)]
pub struct Player {
    pub id: Uuid,
    pub name: String,
    pub state: PlayerState,
    pub creator: bool,
    pub hits: Vec<&'static Hit>,
    pub tokens: u8,
    pub slots: Vec<Slot>,
    pub turn_player: bool,
    pub guess: Option<Slot>,
    pub r#virtual: bool,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Eq, PartialEq, Debug)]
pub struct Slot {
    pub from_year: u32,
    pub to_year: u32,
    pub id: u8,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "".into(),
            state: PlayerState::Waiting,
            creator: false,
            hits: vec![],
            tokens: 0,
            slots: vec![],
            turn_player: false,
            guess: None,
            r#virtual: true,
        }
    }
}

impl From<&User> for Player {
    fn from(u: &User) -> Self {
        Self {
            id: u.id,
            name: u.name.clone(),
            r#virtual: false,
            ..Default::default()
        }
    }
}

#[derive(Serialize, JsonSchema, Clone, Eq, PartialEq, Debug)]
pub struct GameEvent {
    #[serde(skip)]
    pub game_id: String,
    #[serde(skip)]
    pub event: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub players: Option<Vec<Player>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<GameState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hit: Option<&'static Hit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<GameSettingsPayload>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub winner: Option<Player>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_scored: Option<Player>,
}

impl Default for GameEvent {
    fn default() -> Self {
        Self {
            game_id: "".into(),
            event: "".into(),
            players: None,
            state: None,
            hit: None,
            settings: None,
            winner: None,
            last_scored: None,
        }
    }
}
