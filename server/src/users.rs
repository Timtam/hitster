use crate::{responses::MessageResponse, services::ServiceStore, HitsterConfig};
use deref_derive::{Deref, DerefMut};
use rocket::{
    http::{CookieJar, Status},
    request::{self, FromRequest, Outcome, Request},
    serde::json::Json,
    State,
};
use rocket_db_pools::{
    sqlx::{self, Row},
    Connection,
};
use rocket_okapi::{
    gen::OpenApiGenerator,
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

#[derive(
    Deserialize, Serialize, JsonSchema, Clone, Eq, PartialEq, Debug, Hash, Deref, DerefMut,
)]
pub struct Time(#[schemars(with = "String")] pub OffsetDateTime);

impl Default for Time {
    fn default() -> Self {
        Self(OffsetDateTime::now_utc())
    }
}

#[derive(Deserialize, Serialize, JsonSchema, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Token {
    pub token: String,
    #[serde(skip)]
    #[schemars(with = "String")]
    pub expiration_time: Time,
    #[serde(skip)]
    #[schemars(with = "String")]
    pub refresh_time: Time,
}

impl Default for Token {
    fn default() -> Self {
        Self {
            token: "".into(),
            expiration_time: Time(OffsetDateTime::now_utc()),
            refresh_time: Time(OffsetDateTime::now_utc()),
        }
    }
}

#[derive(Deserialize, Serialize, JsonSchema, Clone, Eq, PartialEq, Debug, Hash)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    #[serde(skip)]
    pub password: String,
    #[serde(skip)]
    pub tokens: Vec<Token>,
    pub r#virtual: bool,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
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
                    if t.expiration_time.0 >= OffsetDateTime::now_utc() {
                        return Outcome::Success(u);
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

            return sqlx::query("SELECT * FROM users where name = ?")
                .bind(user.name.as_str())
                .fetch_optional(&mut **db)
                .await
                .unwrap()
                .and_then(|user| {
                    let u = User {
                        id: Uuid::parse_str(&user.get::<String, &str>("id")).unwrap(),
                        name: user.get::<String, &str>("name"),
                        password: user.get::<String, &str>("password"),
                        r#virtual: false,
                        tokens: serde_json::from_str::<Vec<Token>>(
                            &user.get::<String, &str>("tokens"),
                        )
                        .unwrap(),
                    };

                    if let Some(t) = u
                        .tokens
                        .iter()
                        .find(|t| &t.token == token.as_ref().unwrap())
                    {
                        if t.expiration_time.0 >= OffsetDateTime::now_utc() {
                            serv.user_service().lock().add(u.clone());

                            return Some(Outcome::Success(u));
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

impl<'r> OpenApiFromRequest<'r> for User {
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
}

impl From<&User> for UserCookie {
    fn from(src: &User) -> Self {
        Self {
            name: src.name.clone(),
            id: src.id,
            r#virtual: src.r#virtual,
            valid_until: OffsetDateTime::now_utc(),
        }
    }
}
