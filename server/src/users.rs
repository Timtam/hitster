use crate::{
    GlobalEvent, HitsterConfig, games::GameMode, responses::MessageResponse, services::ServiceStore,
};
use hitster_core::{Permissions, User};
use rocket::{
    Data, State,
    fairing::{Fairing, Info, Kind},
    http::{CookieJar, Status},
    request::{self, FromRequest, Outcome, Request},
    serde::json::Json,
    tokio::sync::broadcast::Sender,
};
use rocket_db_pools::{Connection, sqlx};
use rocket_okapi::{
    r#gen::OpenApiGenerator,
    okapi::{schemars, schemars::JsonSchema},
    request::{OpenApiFromRequest, RequestHeaderInput},
};
use serde::{Deserialize, Serialize};
use std::convert::From;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct UserLoginPayload {
    pub username: String,
    pub password: String,
}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize, JsonSchema, Hash)]
pub struct PermissionsPayload {
    pub can_write_hits: bool,
    pub can_write_packs: bool,
}

impl From<&Permissions> for PermissionsPayload {
    fn from(p: &Permissions) -> Self {
        Self {
            can_write_hits: p.contains(Permissions::CAN_WRITE_HITS),
            can_write_packs: p.contains(Permissions::CAN_WRITE_PACKS),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, JsonSchema)]
pub struct UserPayload {
    pub id: Uuid,
    pub name: String,
    pub r#virtual: bool,
}

impl From<&User> for UserPayload {
    fn from(user: &User) -> Self {
        Self {
            id: user.id,
            name: user.name.clone(),
            r#virtual: user.r#virtual,
        }
    }
}

pub struct UserAuthenticator(pub User);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserAuthenticator {
    type Error = Json<MessageResponse>;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let cookies = req.guard::<&CookieJar>().await.unwrap();
        let serv = req.guard::<&State<ServiceStore>>().await.unwrap();
        let mut db = req.guard::<Connection<HitsterConfig>>().await.unwrap();

        let token = cookies
            .get_private("id")
            .map(|cookie| cookie.value().to_string());

        let user = cookies
            .get("user")
            .and_then(|cookie| serde_json::from_str::<UserCookie>(cookie.value()).ok());

        if user.is_some() && token.is_some() {
            let user = user.unwrap();

            if let Some(u) = serv
                .user_service()
                .lock()
                .get_by_username(user.name.as_str())
            {
                if let Some(t) = u
                    .tokens
                    .iter()
                    .find(|t| &t.token == token.as_ref().unwrap())
                {
                    if t.expiration_time >= OffsetDateTime::now_utc() {
                        return Outcome::Success(UserAuthenticator(u));
                    }
                }

                return Outcome::Error((
                    Status::Unauthorized,
                    Json(MessageResponse {
                        message: "token needs to be refreshed".into(),
                        r#type: "error".into(),
                    }),
                ));
            }

            if user.r#virtual {
                return Outcome::Error((
                    Status::Unauthorized,
                    Json(MessageResponse {
                        message: "token needs to be refreshed".into(),
                        r#type: "error".into(),
                    }),
                ));
            }

            let name = user.name.as_str();
            return sqlx::query_as::<_, User>("SELECT * FROM users WHERE name = ?")
                .bind(name)
                .fetch_optional(&mut **db)
                .await
                .unwrap()
                .and_then(|u| {
                    if let Some(t) = u
                        .tokens
                        .iter()
                        .find(|t| &t.token == token.as_ref().unwrap())
                    {
                        if t.expiration_time >= OffsetDateTime::now_utc() {
                            serv.user_service().lock().add(u.clone());

                            return Some(Outcome::Success(UserAuthenticator(u)));
                        }
                    }
                    None
                })
                .unwrap_or(Outcome::Error((
                    Status::Unauthorized,
                    Json(MessageResponse {
                        message: "token needs to be refreshed".into(),
                        r#type: "error".into(),
                    }),
                )));
        }

        Outcome::Error((
            Status::Unauthorized,
            Json(MessageResponse {
                message: "token needs to be refreshed".into(),
                r#type: "error".into(),
            }),
        ))
    }
}

impl OpenApiFromRequest<'_> for UserAuthenticator {
    fn from_request_input(
        _gen: &mut OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> rocket_okapi::Result<RequestHeaderInput> {
        Ok(RequestHeaderInput::None)
    }
}

#[derive(Deserialize, Serialize, Clone, Eq, PartialEq, Debug, Hash)]
pub struct UserCookie {
    pub id: Uuid,
    pub name: String,
    pub r#virtual: bool,
    #[serde(with = "time::serde::rfc3339")]
    pub valid_until: OffsetDateTime,
    pub permissions: PermissionsPayload,
}

impl From<&User> for UserCookie {
    fn from(src: &User) -> Self {
        Self {
            name: src.name.clone(),
            id: src.id,
            r#virtual: src.r#virtual,
            valid_until: OffsetDateTime::now_utc(),
            permissions: (&src.permissions).into(),
        }
    }
}

#[derive(Default)]
pub struct UserCleanupService {}

#[rocket::async_trait]
impl Fairing for UserCleanupService {
    fn info(&self) -> Info {
        Info {
            name: "User cleanup service",
            kind: Kind::Request,
        }
    }

    async fn on_request(&self, req: &mut Request<'_>, _: &mut Data<'_>) {
        let svc = req.guard::<&State<ServiceStore>>().await.unwrap();
        let queue = req.guard::<&State<Sender<GlobalEvent>>>().await.unwrap();
        let usvc = svc.user_service();
        let gsvc = svc.game_service();
        let games = gsvc.lock();
        let users = usvc.lock();

        for user in users.get_all().iter() {
            if users.cleanup_tokens(user.id) {
                for game in games.get_all(Some(user)).iter() {
                    let _ = games.leave(&game.id, user, None);
                    if games.get(&game.id, Some(user)).is_none() && game.mode == GameMode::Public {
                        let _ = queue.send(GlobalEvent::RemoveGame(game.id.clone()));
                    }
                }
                users.remove(user.id);
            }
        }
    }
}
