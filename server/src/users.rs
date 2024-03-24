use crate::{responses::MessageResponse, services::UserService};
use rocket::{
    http::{CookieJar, Status},
    request::{self, FromRequest, Outcome, Request},
    serde::json::Json,
    State,
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
        let users = req.guard::<&State<UserService>>().await.unwrap();

        cookies
            .get_private("login")
            .and_then(|cookie| serde_json::from_str::<UserLoginPayload>(cookie.value()).ok())
            .and_then(|user| users.get_by_username(user.username.as_str()))
            .and_then(|u| Some(Outcome::Success(u)))
            .or(Some(Outcome::Error((
                Status::Unauthorized,
                Json(MessageResponse {
                    message: "not logged in".into(),
                    r#type: "error".into(),
                }),
            ))))
            .unwrap()
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
