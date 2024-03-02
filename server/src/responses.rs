use crate::games::Game;
use crate::users::User;
use rocket_okapi::okapi::{schemars, schemars::JsonSchema};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct GamesResponse {
    pub games: Vec<Game>,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct MessageResponse {
    pub r#type: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct UsersResponse {
    pub users: Vec<User>,
}
