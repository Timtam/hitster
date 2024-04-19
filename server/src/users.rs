use crate::{responses::MessageResponse, services::ServiceStore, HitsterConfig};
use deref_derive::{Deref, DerefMut};
use petname::{Generator, Petnames};
use rand::{prelude::thread_rng, RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rocket::{
    http::{Cookie, CookieJar, Status},
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
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

/*
#[derive(Deserialize, Serialize, JsonSchema)]
pub struct UserLoginPayload {
    pub username: String,
    pub password: String,
}
*/

#[derive(
    Deserialize, Serialize, JsonSchema, Clone, Eq, PartialEq, Debug, Hash, Deref, DerefMut,
)]
pub struct Time(#[schemars(with = "String")] OffsetDateTime);

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

        let generate_token = || -> String {
            let mut rng = thread_rng();
            let mut token_gen = ChaCha8Rng::seed_from_u64(rng.next_u64());
            let mut b = [0u8; 16];
            token_gen.fill_bytes(&mut b);

            u128::from_le_bytes(b).to_string()
        };

        let generate_virtual_user = || -> (User, Token) {
            let mut u = User {
                id: Uuid::new_v4(),
                name: Petnames::default().generate_one(3, "-").unwrap(),
                r#virtual: true,
                tokens: vec![],
                password: "".into(),
            };

            let t = Token {
                token: generate_token(),
                expiration_time: Time(OffsetDateTime::now_utc() + Duration::hours(1)),
                refresh_time: Time(OffsetDateTime::now_utc() + Duration::days(7)),
            };

            u.tokens.push(t.clone());

            serv.user_service().lock().add(u.clone());

            (u, t)
        };

        let set_cookies = |user: &User, token: &Token| {
            cookies.add_private(
                Cookie::build(("id", token.token.clone())).expires(token.refresh_time.0),
            );
            cookies.add(
                Cookie::build(("user", serde_json::to_string(user).unwrap()))
                    .http_only(false)
                    .expires(token.refresh_time.0),
            );
        };

        let handle_existing_token = |token: &str, user: &User| -> User {
            if let Some(mut u) = serv
                .user_service()
                .lock()
                .get_by_username(user.name.as_str())
            {
                if let Some(t) = u.tokens.iter().find(|t| t.token == token) {
                    if t.expiration_time.0 < OffsetDateTime::now_utc()
                        && t.refresh_time.0 > OffsetDateTime::now_utc()
                    {
                        let t = Token {
                            token: generate_token(),
                            expiration_time: Time(OffsetDateTime::now_utc() + Duration::hours(1)),
                            refresh_time: Time(OffsetDateTime::now_utc() + Duration::days(7)),
                        };

                        let pos = u.tokens.iter().position(|ti| ti.token == token).unwrap();

                        std::mem::replace(&mut u.tokens[pos], t.clone());

                        serv.user_service().lock().add(u.clone());

                        set_cookies(&u, &t);
                        return u;
                    } else if t.refresh_time.0 < OffsetDateTime::now_utc() {
                        let (u, t) = generate_virtual_user();

                        set_cookies(&u, &t);
                        return u;
                    } else {
                        return u;
                    }
                }

                let (u, t) = generate_virtual_user();

                set_cookies(&u, &t);
                u
            } else {
                // access DB
                let (u, t) = generate_virtual_user();

                set_cookies(&u, &t);
                u
            }
        };

        let token = cookies
            .get_private("id")
            .map(|cookie| cookie.value().to_string());

        let user = cookies
            .get_private("user")
            .and_then(|cookie| serde_json::from_str::<User>(cookie.value()).ok());

        if user.is_some() && token.is_some() {
            return Outcome::Success(handle_existing_token(
                token.as_ref().unwrap(),
                user.as_ref().unwrap(),
            ));
        } else {
            let (u, t) = generate_virtual_user();

            set_cookies(&u, &t);

            Outcome::Success(u)
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
