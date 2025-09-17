use crate::hits::HitPayload;
use hitster_core::{Hit, User};
use rocket::serde::json::Json;
use rocket_okapi::okapi::{schemars, schemars::JsonSchema};
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, convert::From, default::Default};
use uuid::Uuid;

/// A payload to address a slot

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct SlotPayload {
    /// The slot ID for a certain player, or no slot at all
    pub id: Option<u8>,
}

/// Game settings

#[derive(Deserialize, Serialize, JsonSchema, Clone, Eq, PartialEq, Debug)]
pub struct GameSettingsPayload {
    /// the duration of a hit that gets played when guessing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hit_duration: Option<u8>,
    /// the amount of tokens every player starts with
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_tokens: Option<u8>,
    /// the amount of hits it needs to have to win
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal: Option<u8>,
    /// packs to draw hits from
    #[serde(skip_serializing_if = "Option::is_none")]
    pub packs: Option<Vec<Uuid>>,
}

/// options when creating a game

#[derive(Deserialize, Serialize, JsonSchema, Clone, Eq, PartialEq, Debug)]
pub struct CreateGamePayload {
    /// game settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<GameSettingsPayload>,
    /// game mode
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
        }
    }
}

/// confirmation

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct ConfirmationPayload {
    /// wether the player should receive a token or not
    pub confirm: bool,
}

/// the possible states a game can be in

#[derive(Deserialize, Serialize, JsonSchema, Clone, Eq, PartialEq, Debug, Copy)]
#[serde(rename_all_fields = "snake_case")]
pub enum GameState {
    /// the game is currently accepting new players
    Open,
    /// the player has to guess, a song is currently available for playback
    Guessing,
    /// other players can pick different slots (first come first serve)
    Intercepting,
    /// a different player has to confirm the choices
    Confirming,
}

/// visibility scope of a game

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

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Game {
    pub id: String,
    pub players: Vec<Player>,
    pub state: GameState,
    pub hits_remaining: VecDeque<Hit>,
    pub hit_duration: u8,
    pub start_tokens: u8,
    pub goal: u8,
    pub hit: Option<Hit>,
    pub packs: Vec<Uuid>,
    pub mode: GameMode,
    pub remembered_hits: Vec<Hit>,
    pub last_scored: Option<Player>,
}

/// all information related to a game

#[derive(Clone, Eq, PartialEq, Debug, Serialize, JsonSchema)]
pub struct GamePayload {
    /// the unique game ID
    pub id: String,
    /// all players who are part of a game
    pub players: Vec<PlayerPayload>,
    /// current state of a game
    pub state: GameState,
    /// the configured playback length of a hit when guessing
    pub hit_duration: u8,
    /// the amount of tokens every player starts with
    pub start_tokens: u8,
    /// the amount of hits one needs to have to win
    pub goal: u8,
    /// the currently revealed hit
    pub hit: Option<HitPayload>,
    /// the pack IDs of packs hits get drawn from
    pub packs: Vec<Uuid>,
    /// the visibility scope of the game
    pub mode: GameMode,
    /// the player who last scored a hit
    pub last_scored: Option<PlayerPayload>,
}

impl From<&Game> for GamePayload {
    fn from(game: &Game) -> Self {
        Self {
            id: game.id.clone(),
            players: game.players.iter().map(|p| p.into()).collect::<Vec<_>>(),
            state: game.state,
            hit_duration: game.hit_duration,
            start_tokens: game.start_tokens,
            goal: game.goal,
            hit: game.hit.as_ref().map(|h| h.into()),
            packs: game.packs.clone(),
            mode: game.mode,
            last_scored: game.last_scored.as_ref().map(|p| p.into()),
        }
    }
}

/// information relating to a pack

#[derive(Clone, Eq, PartialEq, Debug, Serialize, JsonSchema)]
pub struct PackPayload {
    /// the unique ID of a pack
    pub id: Uuid,
    /// the name of the pack
    pub name: String,
    /// the amount of hits in this pack
    pub hits: usize,
}

/// possible player states

#[derive(Deserialize, Serialize, JsonSchema, Clone, Eq, PartialEq, Debug, Copy)]
#[serde(rename_all_fields = "snake_case")]
pub enum PlayerState {
    /// the player doesn't need to react
    Waiting,
    /// the player needs to guess a slot
    Guessing,
    /// the player needs to intercept, either by providing a slot id where they want to guess or by not providing a slot ID
    Intercepting,
    /// this player needs to confirm wether the guessing player gets a token or not
    Confirming,
}

#[derive(Serialize, Clone, Eq, PartialEq, Debug)]
pub struct Player {
    pub id: Uuid,
    pub name: String,
    pub state: PlayerState,
    pub creator: bool,
    pub hits: Vec<Hit>,
    pub tokens: u8,
    pub slots: Vec<Slot>,
    pub turn_player: bool,
    pub guess: Option<Slot>,
    pub r#virtual: bool,
}

/// a player who is part of a game

#[derive(Serialize, JsonSchema, Clone, Eq, PartialEq, Debug)]
pub struct PlayerPayload {
    /// the unique id of a player
    pub id: Uuid,
    /// the name that is shown to other users
    pub name: String,
    /// the state a player is in, e.g. what they need to do next
    pub state: PlayerState,
    /// wether the player is the creator of the game, can change its settings, kick players etc
    pub creator: bool,
    /// the hits this player collected
    pub hits: Vec<HitPayload>,
    /// the amount of tokens this player has
    pub tokens: u8,
    /// the slots which get generated by the hits the player collected
    pub slots: Vec<Slot>,
    /// wether its this player's turn or not
    pub turn_player: bool,
    /// this is the slot the player guessed if they are within the Guessing or Intercepting state
    pub guess: Option<Slot>,
    /// wether the player is virtual (in a local game) or an actual user
    pub r#virtual: bool,
}

impl From<&Player> for PlayerPayload {
    fn from(p: &Player) -> Self {
        Self {
            id: p.id,
            name: p.name.clone(),
            state: p.state,
            creator: p.creator,
            hits: p.hits.iter().map(|h| h.into()).collect::<Vec<_>>(),
            tokens: p.tokens,
            slots: p.slots.clone(),
            turn_player: p.turn_player,
            guess: p.guess.clone(),
            r#virtual: p.r#virtual,
        }
    }
}

/// a slot which gets auto-generated from the hits of a player

#[derive(Serialize, Deserialize, JsonSchema, Clone, Eq, PartialEq, Debug)]
pub struct Slot {
    /// start (either 0 or a year)
    pub from_year: u32,
    /// end (either 0 or a year)
    pub to_year: u32,
    /// the id of a slot. This isn't unique and always relates to the player the slot originates from
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
    pub players: Option<Vec<PlayerPayload>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<GameState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hit: Option<HitPayload>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<GameSettingsPayload>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub winner: Option<PlayerPayload>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_scored: Option<PlayerPayload>,
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
