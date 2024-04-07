use crate::{games::Game, hits::Pack, users::User};
use rocket::{
    http::{ContentType, Status},
    request::Request,
    response::{self, Responder, Response},
};
use rocket_okapi::{
    gen::OpenApiGenerator,
    okapi::{
        openapi3::{RefOr, Response as OpenApiResponse, Responses},
        schemars::{self, JsonSchema, Map},
    },
    response::OpenApiResponderInner,
    OpenApiError,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, JsonSchema)]
pub struct JoinGameError {
    pub message: String,
    #[serde(skip)]
    pub http_status_code: u16,
}

impl OpenApiResponderInner for JoinGameError {
    fn responses(_generator: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        let mut responses = Map::new();
        responses.insert(
            "403".to_string(),
            RefOr::Object(OpenApiResponse {
                description: "\
                # [403 Forbidden](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/403)\n\
                The game is already running.\
                "
                .to_string(),
                ..Default::default()
            }),
        );
        responses.insert(
            "404".to_string(),
            RefOr::Object(OpenApiResponse {
                description: "\
                # [404 Not Found](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/404)\n\
                A game with that ID doesn't exist.\
                "
                .to_string(),
                ..Default::default()
            }),
        );
        responses.insert(
            "409".to_string(),
            RefOr::Object(OpenApiResponse {
                description: "\
                # [409 Conflict](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/409)\n\
                That user is already part of this game.\
                "
                .to_string(),
                ..Default::default()
            }),
        );
        Ok(Responses {
            responses,
            ..Default::default()
        })
    }
}

impl std::fmt::Display for JoinGameError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Join game error `{}`", self.message,)
    }
}

impl std::error::Error for JoinGameError {}

impl<'r> Responder<'r, 'static> for JoinGameError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        // Convert object to json
        let body = serde_json::to_string(&self).unwrap();
        Response::build()
            .sized_body(body.len(), std::io::Cursor::new(body))
            .header(ContentType::JSON)
            .status(Status::new(self.http_status_code))
            .ok()
    }
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct LeaveGameError {
    pub message: String,
    #[serde(skip)]
    pub http_status_code: u16,
}

impl OpenApiResponderInner for LeaveGameError {
    fn responses(_generator: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        let mut responses = Map::new();
        responses.insert(
            "404".to_string(),
            RefOr::Object(OpenApiResponse {
                description: "\
                # [404 Not Found](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/404)\n\
                A game with that ID doesn't exist.\
                "
                .to_string(),
                ..Default::default()
            }),
        );
        responses.insert(
            "409".to_string(),
            RefOr::Object(OpenApiResponse {
                description: "\
                # [409 Conflict](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/409)\n\
                That user isn't part of this game.\
                "
                .to_string(),
                ..Default::default()
            }),
        );
        Ok(Responses {
            responses,
            ..Default::default()
        })
    }
}

impl std::fmt::Display for LeaveGameError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Leave game error `{}`", self.message,)
    }
}

impl std::error::Error for LeaveGameError {}

impl<'r> Responder<'r, 'static> for LeaveGameError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        // Convert object to json
        let body = serde_json::to_string(&self).unwrap();
        Response::build()
            .sized_body(body.len(), std::io::Cursor::new(body))
            .header(ContentType::JSON)
            .status(Status::new(self.http_status_code))
            .ok()
    }
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct StartGameError {
    pub message: String,
    #[serde(skip)]
    pub http_status_code: u16,
}

impl OpenApiResponderInner for StartGameError {
    fn responses(_generator: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        let mut responses = Map::new();
        responses.insert(
            "403".to_string(),
            RefOr::Object(OpenApiResponse {
                description: "\
                # [403 Forbidden](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/403)\n\
                You are not the creator of the selected game.\
                "
                .to_string(),
                ..Default::default()
            }),
        );
        responses.insert(
            "404".to_string(),
            RefOr::Object(OpenApiResponse {
                description: "\
                # [404 Not Found](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/404)\n\
                A game with that ID doesn't exist.\
                "
                .to_string(),
                ..Default::default()
            }),
        );
        responses.insert(
            "409".to_string(),
            RefOr::Object(OpenApiResponse {
                description: "\
                # [409 Conflict](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/409)\n\
                There are to few players in this game or the game is already running.\
                "
                .to_string(),
                ..Default::default()
            }),
        );
        Ok(Responses {
            responses,
            ..Default::default()
        })
    }
}

impl std::fmt::Display for StartGameError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Start game error `{}`", self.message,)
    }
}

impl std::error::Error for StartGameError {}

impl<'r> Responder<'r, 'static> for StartGameError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        // Convert object to json
        let body = serde_json::to_string(&self).unwrap();
        Response::build()
            .sized_body(body.len(), std::io::Cursor::new(body))
            .header(ContentType::JSON)
            .status(Status::new(self.http_status_code))
            .ok()
    }
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct StopGameError {
    pub message: String,
    #[serde(skip)]
    pub http_status_code: u16,
}

impl OpenApiResponderInner for StopGameError {
    fn responses(_generator: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        let mut responses = Map::new();
        responses.insert(
            "403".to_string(),
            RefOr::Object(OpenApiResponse {
                description: "\
                # [403 Forbidden](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/403)\n\
                You are not the creator of the selected game.\
                "
                .to_string(),
                ..Default::default()
            }),
        );
        responses.insert(
            "404".to_string(),
            RefOr::Object(OpenApiResponse {
                description: "\
                # [404 Not Found](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/404)\n\
                A game with that ID doesn't exist.\
                "
                .to_string(),
                ..Default::default()
            }),
        );
        responses.insert(
            "409".to_string(),
            RefOr::Object(OpenApiResponse {
                description: "\
                # [409 Conflict](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/409)\n\
                The game isn't running.\
                "
                .to_string(),
                ..Default::default()
            }),
        );
        Ok(Responses {
            responses,
            ..Default::default()
        })
    }
}

impl std::fmt::Display for StopGameError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Start game error `{}`", self.message,)
    }
}

impl std::error::Error for StopGameError {}

impl<'r> Responder<'r, 'static> for StopGameError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        // Convert object to json
        let body = serde_json::to_string(&self).unwrap();
        Response::build()
            .sized_body(body.len(), std::io::Cursor::new(body))
            .header(ContentType::JSON)
            .status(Status::new(self.http_status_code))
            .ok()
    }
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct CurrentHitError {
    pub message: String,
    #[serde(skip)]
    pub http_status_code: u16,
}

impl OpenApiResponderInner for CurrentHitError {
    fn responses(_generator: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        let mut responses = Map::new();
        responses.insert(
            "404".to_string(),
            RefOr::Object(OpenApiResponse {
                description: "\
                # [404 Not Found](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/404)\n\
                A game with that ID doesn't exist or hit file couldn't be found.\
                "
                .to_string(),
                ..Default::default()
            }),
        );
        responses.insert(
            "409".to_string(),
            RefOr::Object(OpenApiResponse {
                description: "\
                # [409 Conflict](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/409)\n\
                The game needs to be running.\
                "
                .to_string(),
                ..Default::default()
            }),
        );
        responses.insert(
            "500".to_string(),
            RefOr::Object(OpenApiResponse {
                description: "\
                # [500 Internal Server Error](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/500)\n\
                A hit couldn't be found or a different error occurred.\
                "
                .to_string(),
                ..Default::default()
            }),
        );
        Ok(Responses {
            responses,
            ..Default::default()
        })
    }
}

impl std::fmt::Display for CurrentHitError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Current hit error `{}`", self.message,)
    }
}

impl std::error::Error for CurrentHitError {}

impl<'r> Responder<'r, 'static> for CurrentHitError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        // Convert object to json
        let body = serde_json::to_string(&self).unwrap();
        Response::build()
            .sized_body(body.len(), std::io::Cursor::new(body))
            .header(ContentType::JSON)
            .status(Status::new(self.http_status_code))
            .ok()
    }
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct GuessSlotError {
    pub message: String,
    #[serde(skip)]
    pub http_status_code: u16,
}

impl OpenApiResponderInner for GuessSlotError {
    fn responses(_generator: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        let mut responses = Map::new();
        responses.insert(
            "404".to_string(),
            RefOr::Object(OpenApiResponse {
                description: "\
                # [403 Forbidden](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/403)\n\
                The player cannot guess right now.\
                "
                .to_string(),
                ..Default::default()
            }),
        );
        responses.insert(
            "404".to_string(),
            RefOr::Object(OpenApiResponse {
                description: "\
                # [404 Not Found](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/404)\n\
                A game with that ID doesn't exist or hit file couldn't be found.\
                "
                .to_string(),
                ..Default::default()
            }),
        );
        responses.insert(
            "409".to_string(),
            RefOr::Object(OpenApiResponse {
                description: "\
                # [409 Conflict](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/409)\n\
                The game needs to be running or the slot is already taken.\
                "
                .to_string(),
                ..Default::default()
            }),
        );
        Ok(Responses {
            responses,
            ..Default::default()
        })
    }
}

impl std::fmt::Display for GuessSlotError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Current hit error `{}`", self.message,)
    }
}

impl std::error::Error for GuessSlotError {}

impl<'r> Responder<'r, 'static> for GuessSlotError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        // Convert object to json
        let body = serde_json::to_string(&self).unwrap();
        Response::build()
            .sized_body(body.len(), std::io::Cursor::new(body))
            .header(ContentType::JSON)
            .status(Status::new(self.http_status_code))
            .ok()
    }
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ConfirmSlotError {
    pub message: String,
    #[serde(skip)]
    pub http_status_code: u16,
}

impl OpenApiResponderInner for ConfirmSlotError {
    fn responses(_generator: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        let mut responses = Map::new();
        responses.insert(
            "403".to_string(),
            RefOr::Object(OpenApiResponse {
                description: "\
                # [403 Forbidden](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/403)\n\
                The player cannot confirm right now.\
                "
                .to_string(),
                ..Default::default()
            }),
        );
        responses.insert(
            "404".to_string(),
            RefOr::Object(OpenApiResponse {
                description: "\
                # [404 Not Found](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/404)\n\
                A game with that ID doesn't exist.\
                "
                .to_string(),
                ..Default::default()
            }),
        );
        responses.insert(
            "409".to_string(),
            RefOr::Object(OpenApiResponse {
                description: "\
                # [409 Conflict](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/409)\n\
                The game needs to be running.\
                "
                .to_string(),
                ..Default::default()
            }),
        );
        Ok(Responses {
            responses,
            ..Default::default()
        })
    }
}

impl std::fmt::Display for ConfirmSlotError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Current hit error `{}`", self.message,)
    }
}

impl std::error::Error for ConfirmSlotError {}

impl<'r> Responder<'r, 'static> for ConfirmSlotError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        // Convert object to json
        let body = serde_json::to_string(&self).unwrap();
        Response::build()
            .sized_body(body.len(), std::io::Cursor::new(body))
            .header(ContentType::JSON)
            .status(Status::new(self.http_status_code))
            .ok()
    }
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct SkipHitError {
    pub message: String,
    #[serde(skip)]
    pub http_status_code: u16,
}

impl OpenApiResponderInner for SkipHitError {
    fn responses(_generator: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        let mut responses = Map::new();
        responses.insert(
            "403".to_string(),
            RefOr::Object(OpenApiResponse {
                description: "\
                # [403 Forbidden](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/403)\n\
                The player cannot skip the hit right now.\
                "
                .to_string(),
                ..Default::default()
            }),
        );
        responses.insert(
            "404".to_string(),
            RefOr::Object(OpenApiResponse {
                description: "\
                # [404 Not Found](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/404)\n\
                A game with that ID doesn't exist.\
                "
                .to_string(),
                ..Default::default()
            }),
        );
        responses.insert(
            "409".to_string(),
            RefOr::Object(OpenApiResponse {
                description: "\
                # [409 Conflict](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/409)\n\
                The game needs to be running.\
                "
                .to_string(),
                ..Default::default()
            }),
        );
        Ok(Responses {
            responses,
            ..Default::default()
        })
    }
}

impl std::fmt::Display for SkipHitError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Current hit error `{}`", self.message,)
    }
}

impl std::error::Error for SkipHitError {}

impl<'r> Responder<'r, 'static> for SkipHitError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        // Convert object to json
        let body = serde_json::to_string(&self).unwrap();
        Response::build()
            .sized_body(body.len(), std::io::Cursor::new(body))
            .header(ContentType::JSON)
            .status(Status::new(self.http_status_code))
            .ok()
    }
}

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

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct PacksResponse {
    pub packs: HashMap<Pack, usize>,
}
