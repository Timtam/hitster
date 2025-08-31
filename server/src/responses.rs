use crate::{
    games::{GamePayload, PackPayload},
    users::UserPayload,
};
use rocket::{
    http::{ContentType, Status},
    request::Request,
    response::{self, Responder, Response},
};
use rocket_okapi::{
    OpenApiError,
    r#gen::OpenApiGenerator,
    okapi::{
        openapi3::{RefOr, Response as OpenApiResponse, Responses},
        schemars::{self, JsonSchema, Map},
    },
    response::OpenApiResponderInner,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, JsonSchema)]
pub struct PaginatedResponse<T> {
    pub results: Vec<T>,
    pub total: usize,
    pub start: usize,
    pub end: usize,
}

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
            "401".to_string(),
            RefOr::Object(OpenApiResponse {
                description: "\
                # [401 Unauthorized](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/401)\n\
                The API call requires a valid token, but the token needs to be refreshed by calling the /users/auth endpoint.\
                "
                .to_string(),
                ..Default::default()
            }),
        );
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
            "401".to_string(),
            RefOr::Object(OpenApiResponse {
                description: "\
                # [401 Unauthorized](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/401)\n\
                The API call requires a valid token, but the token needs to be refreshed by calling the /users/auth endpoint.\
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
            "401".to_string(),
            RefOr::Object(OpenApiResponse {
                description: "\
                # [401 Unauthorized](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/401)\n\
                The API call requires a valid token, but the token needs to be refreshed by calling the /users/auth endpoint.\
                "
                .to_string(),
                ..Default::default()
            }),
        );
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
            "401".to_string(),
            RefOr::Object(OpenApiResponse {
                description: "\
                # [401 Unauthorized](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/401)\n\
                The API call requires a valid token, but the token needs to be refreshed by calling the /users/auth endpoint.\
                "
                .to_string(),
                ..Default::default()
            }),
        );
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
        write!(formatter, "Stop game error `{}`", self.message,)
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
pub struct HitError {
    pub message: String,
    #[serde(skip)]
    pub http_status_code: u16,
}

impl OpenApiResponderInner for HitError {
    fn responses(_generator: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        let mut responses = Map::new();
        responses.insert(
            "404".to_string(),
            RefOr::Object(OpenApiResponse {
                description: "\
                # [404 Not Found](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/404)\n\
                A game with that ID doesn't exist, hit or file couldn't be found.\
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

impl std::fmt::Display for HitError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Current hit error `{}`", self.message,)
    }
}

impl std::error::Error for HitError {}

impl<'r> Responder<'r, 'static> for HitError {
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
            "401".to_string(),
            RefOr::Object(OpenApiResponse {
                description: "\
                # [401 Unauthorized](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/401)\n\
                The API call requires a valid token, but the token needs to be refreshed by calling the /users/auth endpoint.\
                "
                .to_string(),
                ..Default::default()
            }),
        );
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
        write!(formatter, "Guess slot error `{}`", self.message,)
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
            "401".to_string(),
            RefOr::Object(OpenApiResponse {
                description: "\
                # [401 Unauthorized](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/401)\n\
                The API call requires a valid token, but the token needs to be refreshed by calling the /users/auth endpoint.\
                "
                .to_string(),
                ..Default::default()
            }),
        );
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
        write!(formatter, "Confirm slot error `{}`", self.message,)
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
            "401".to_string(),
            RefOr::Object(OpenApiResponse {
                description: "\
                # [401 Unauthorized](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/401)\n\
                The API call requires a valid token, but the token needs to be refreshed by calling the /users/auth endpoint.\
                "
                .to_string(),
                ..Default::default()
            }),
        );
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
        write!(formatter, "Skip hit error `{}`", self.message,)
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

#[derive(Debug, Serialize, JsonSchema)]
pub struct UpdateGameError {
    pub message: String,
    #[serde(skip)]
    pub http_status_code: u16,
}

impl OpenApiResponderInner for UpdateGameError {
    fn responses(_generator: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        let mut responses = Map::new();
        responses.insert(
            "401".to_string(),
            RefOr::Object(OpenApiResponse {
                description: "\
                # [401 Unauthorized](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/401)\n\
                The API call requires a valid token, but the token needs to be refreshed by calling the /users/auth endpoint.\
                "
                .to_string(),
                ..Default::default()
            }),
        );
        responses.insert(
            "403".to_string(),
            RefOr::Object(OpenApiResponse {
                description: "\
                # [403 Forbidden](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/403)\n\
                The game can only be updated while not running.\
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
                The game can only be updated by the creator or a specified value is invalid.\
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

impl std::fmt::Display for UpdateGameError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Update game error `{}`", self.message,)
    }
}

impl std::error::Error for UpdateGameError {}

impl<'r> Responder<'r, 'static> for UpdateGameError {
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
pub struct UpdateHitError {
    pub message: String,
    #[serde(skip)]
    pub http_status_code: u16,
}

impl OpenApiResponderInner for UpdateHitError {
    fn responses(_generator: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        let mut responses = Map::new();
        responses.insert(
            "401".to_string(),
            RefOr::Object(OpenApiResponse {
                description: "\
                # [401 Unauthorized](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/401)\n\
                This endpoint is only usable by an authenticated user who has write permissions for hits.\
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
                A hit with that ID doesn't exist.\
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

impl std::fmt::Display for UpdateHitError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Update hit error `{}`", self.message,)
    }
}

impl std::error::Error for UpdateHitError {}

impl<'r> Responder<'r, 'static> for UpdateHitError {
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

#[derive(Serialize, JsonSchema)]
pub struct GamesResponse {
    pub games: Vec<GamePayload>,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct MessageResponse {
    pub r#type: String,
    pub message: String,
}

#[derive(Serialize, JsonSchema)]
pub struct UsersResponse {
    pub users: Vec<UserPayload>,
}

#[derive(Serialize, JsonSchema)]
pub struct PacksResponse {
    pub packs: Vec<PackPayload>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ClaimHitError {
    pub message: String,
    #[serde(skip)]
    pub http_status_code: u16,
}

impl OpenApiResponderInner for ClaimHitError {
    fn responses(_generator: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        let mut responses = Map::new();
        responses.insert(
            "401".to_string(),
            RefOr::Object(OpenApiResponse {
                description: "\
                # [401 Unauthorized](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/401)\n\
                The API call requires a valid token, but the token needs to be refreshed by calling the /users/auth endpoint.\
                "
                .to_string(),
                ..Default::default()
            }),
        );
        responses.insert(
            "403".to_string(),
            RefOr::Object(OpenApiResponse {
                description: "\
                # [403 Forbidden](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/403)\n\
                The player cannot claim a hit right now.\
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

impl std::fmt::Display for ClaimHitError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Claim hit error `{}`", self.message,)
    }
}

impl std::error::Error for ClaimHitError {}

impl<'r> Responder<'r, 'static> for ClaimHitError {
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
pub struct GetGameError {
    pub message: String,
    #[serde(skip)]
    pub http_status_code: u16,
}

impl OpenApiResponderInner for GetGameError {
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
        Ok(Responses {
            responses,
            ..Default::default()
        })
    }
}

impl std::fmt::Display for GetGameError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Get game error `{}`", self.message,)
    }
}

impl std::error::Error for GetGameError {}

impl<'r> Responder<'r, 'static> for GetGameError {
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
pub struct GetHitError {
    pub message: String,
    #[serde(skip)]
    pub http_status_code: u16,
}

impl OpenApiResponderInner for GetHitError {
    fn responses(_generator: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        let mut responses = Map::new();
        responses.insert(
            "404".to_string(),
            RefOr::Object(OpenApiResponse {
                description: "\
                # [404 Not Found](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/404)\n\
                A hit with that ID doesn't exist.\
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

impl std::fmt::Display for GetHitError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Get hit error `{}`", self.message,)
    }
}

impl std::error::Error for GetHitError {}

impl<'r> Responder<'r, 'static> for GetHitError {
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
pub struct GetUserError {
    pub message: String,
    #[serde(skip)]
    pub http_status_code: u16,
}

impl OpenApiResponderInner for GetUserError {
    fn responses(_generator: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        let mut responses = Map::new();
        responses.insert(
            "404".to_string(),
            RefOr::Object(OpenApiResponse {
                description: "\
                # [404 Not Found](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/404)\n\
                A user with that ID doesn't exist.\
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

impl std::fmt::Display for GetUserError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Get user error `{}`", self.message,)
    }
}

impl std::error::Error for GetUserError {}

impl<'r> Responder<'r, 'static> for GetUserError {
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
pub struct UserLoginError {
    pub message: String,
    #[serde(skip)]
    pub http_status_code: u16,
}

impl OpenApiResponderInner for UserLoginError {
    fn responses(_generator: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        let mut responses = Map::new();
        responses.insert(
            "401".to_string(),
            RefOr::Object(OpenApiResponse {
                description: "\
                # [401 Unauthorized](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/401)\n\
                The user credentials are invalid.\
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

impl std::fmt::Display for UserLoginError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "User login error `{}`", self.message,)
    }
}

impl std::error::Error for UserLoginError {}

impl<'r> Responder<'r, 'static> for UserLoginError {
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
pub struct RegisterUserError {
    pub message: String,
    #[serde(skip)]
    pub http_status_code: u16,
}

impl OpenApiResponderInner for RegisterUserError {
    fn responses(_generator: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        let mut responses = Map::new();
        responses.insert(
            "405".to_string(),
            RefOr::Object(OpenApiResponse {
                description: "\
                # [405 Method Not Allowed](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/405)\n\
                A user is already authenticated and registered.\
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
                Username is already in use.\
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
                error while creating a database entry.\
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

impl std::fmt::Display for RegisterUserError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Register user error `{}`", self.message,)
    }
}

impl std::error::Error for RegisterUserError {}

impl<'r> Responder<'r, 'static> for RegisterUserError {
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
