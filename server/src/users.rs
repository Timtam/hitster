use rocket::{
    http::{CookieJar, Status},
    request::{self, FromRequest, Outcome, Request},
};
use rocket_okapi::okapi::{schemars, schemars::JsonSchema};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct UserLoginPayload {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Serialize, JsonSchema, Clone, Eq, PartialEq, Debug)]
pub struct User {
    pub id: u32,
    pub username: String,
    #[serde(skip_serializing)]
    pub password: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = &'static str;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        Outcome::Success(User {
            id: 0,
            username: "".into(),
            password: "".into(),
        })
    }
}
