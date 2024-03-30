use crate::{responses::MessageResponse, services::ServiceStore, HitsterConfig};
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

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct UserLoginPayload {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Serialize, JsonSchema, Clone, Eq, PartialEq, Debug, Hash)]
pub struct User {
    pub id: u32,
    pub username: String,
    #[serde(skip)]
    pub password: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = Json<MessageResponse>;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let cookies = req.guard::<&CookieJar>().await.unwrap();
        let serv = req.guard::<&State<ServiceStore>>().await.unwrap();
        let mut db = req.guard::<Connection<HitsterConfig>>().await.unwrap();

        if let Some(user) = cookies
            .get_private("login")
            .and_then(|cookie| serde_json::from_str::<UserLoginPayload>(cookie.value()).ok())
        {
            if let Some(u) = serv
                .user_service()
                .lock()
                .get_by_username(user.username.as_str())
            {
                return Outcome::Success(u);
            }

            sqlx::query("SELECT * FROM users where username = ?")
                .bind(user.username.as_str())
                .fetch_optional(&mut **db)
                .await
                .unwrap()
                .map(|user| {
                    let u = User {
                        id: user.get::<u32, &str>("id"),
                        username: user.get::<String, &str>("username"),
                        password: user.get::<String, &str>("password"),
                    };

                    serv.user_service().lock().add(u.clone());

                    Outcome::Success(u)
                })
                .unwrap_or(Outcome::Error((
                    Status::Unauthorized,
                    Json(MessageResponse {
                        message: "not logged in".into(),
                        r#type: "error".into(),
                    }),
                )))
        } else {
            Outcome::Error((
                Status::Unauthorized,
                Json(MessageResponse {
                    message: "not logged in".into(),
                    r#type: "error".into(),
                }),
            ))
        }
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
