use crate::{
    responses::{MessageResponse, UsersResponse},
    services::ServiceStore,
    users::{Time, Token, User, UserCookie, UserLoginPayload},
    HitsterConfig,
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use petname::{Generator, Petnames};
use rand::{prelude::thread_rng, RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rocket::{
    http::{Cookie, CookieJar},
    response::status::NotFound,
    serde::json::Json,
    State,
};
use rocket_db_pools::{
    sqlx::{self, Row},
    Connection,
};
use rocket_okapi::openapi;
use serde_json;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

/// internals

fn generate_token() -> String {
    let mut rng = thread_rng();
    let mut token_gen = ChaCha8Rng::seed_from_u64(rng.next_u64());
    let mut b = [0u8; 16];
    token_gen.fill_bytes(&mut b);

    u128::from_le_bytes(b).to_string()
}

fn generate_virtual_user(svc: &ServiceStore) -> (User, Token) {
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

    svc.user_service().lock().add(u.clone());

    (u, t)
}

fn set_cookies(user: &User, token: &Token, cookies: &CookieJar<'_>) {
    let mut uc = UserCookie::from(user);

    uc.valid_until = token.expiration_time.0;

    cookies.add_private(Cookie::build(("id", token.token.clone())).expires(token.refresh_time.0));
    cookies.add(
        Cookie::build(("user", serde_json::to_string(&uc).unwrap()))
            .http_only(false)
            .expires(token.refresh_time.0),
    );
}

async fn handle_existing_token(
    token: &str,
    user: &UserCookie,
    svc: &ServiceStore,
    cookies: &CookieJar<'_>,
    mut db: Connection<HitsterConfig>,
) -> User {
    if let Some(mut u) = svc
        .user_service()
        .lock()
        .get_by_username(user.name.as_str())
    {
        // the user already exists within the user service
        if let Some(t) = u.tokens.iter().find(|t| t.token == token) {
            // the token exists for the user
            if t.expiration_time.0 < OffsetDateTime::now_utc()
                && t.refresh_time.0 > OffsetDateTime::now_utc()
            {
                // token expired, but can still be refreshed
                let t = Token {
                    token: generate_token(),
                    expiration_time: Time(OffsetDateTime::now_utc() + Duration::hours(1)),
                    refresh_time: Time(OffsetDateTime::now_utc() + Duration::days(7)),
                };

                let pos = u.tokens.iter().position(|ti| ti.token == token).unwrap();

                std::mem::replace(&mut u.tokens[pos], t.clone());

                svc.user_service().lock().add(u.clone());

                if !u.r#virtual {
                    sqlx::query("UPDATE users SET tokens = ? WHERE id = ?")
                        .bind(serde_json::to_string(&u.tokens).unwrap())
                        .bind(u.id.to_string())
                        .execute(&mut **db)
                        .await;
                }

                set_cookies(&u, &t, cookies);
                return u;
            }

            if t.refresh_time.0 < OffsetDateTime::now_utc() {
                // token refresh time is up and you're not logged in anymore
                let (u, t) = generate_virtual_user(svc);

                set_cookies(&u, &t, cookies);
                return u;
            }

            // token didn't expire yet, so just return the user as-is
            return u;
        }

        // user exists, but token doesn't exist anymore
        // we'll generate a new virtual user for you

        let (u, t) = generate_virtual_user(svc);

        set_cookies(&u, &t, cookies);
        return u;
    }
    // user doesn't exist within the user service, but might still exist within the db
    if user.r#virtual {
        // nope, its a virtual one
        let (u, t) = generate_virtual_user(svc);

        set_cookies(&u, &t, cookies);
        return u;
    }

    if let Some(mut u) = sqlx::query("SELECT * FROM users where name = ?")
        .bind(user.name.as_str())
        .fetch_optional(&mut **db)
        .await
        .unwrap()
        .map(|user| User {
            id: Uuid::parse_str(&user.get::<String, &str>("id")).unwrap(),
            name: user.get::<String, &str>("name"),
            password: user.get::<String, &str>("password"),
            r#virtual: false,
            tokens: serde_json::from_str::<Vec<Token>>(&user.get::<String, &str>("tokens"))
                .unwrap(),
        })
    {
        // user exists within the db
        if let Some(t) = u.tokens.iter().find(|t| t.token == token) {
            // we found the token
            if t.expiration_time.0 < OffsetDateTime::now_utc()
                && t.refresh_time.0 > OffsetDateTime::now_utc()
            {
                // token expired, but can still be refreshed
                let t = Token {
                    token: generate_token(),
                    expiration_time: Time(OffsetDateTime::now_utc() + Duration::hours(1)),
                    refresh_time: Time(OffsetDateTime::now_utc() + Duration::days(7)),
                };

                let pos = u.tokens.iter().position(|ti| ti.token == token).unwrap();

                std::mem::replace(&mut u.tokens[pos], t.clone());

                svc.user_service().lock().add(u.clone());

                sqlx::query("UPDATE users SET tokens = ? WHERE id = ?")
                    .bind(serde_json::to_string(&u.tokens).unwrap())
                    .bind(u.id.to_string())
                    .execute(&mut **db)
                    .await;

                set_cookies(&u, &t, cookies);
                return u;
            }

            if t.refresh_time.0 < OffsetDateTime::now_utc() {
                // token refresh time is up and you're not logged in anymore
                let (u, t) = generate_virtual_user(svc);

                set_cookies(&u, &t, cookies);
                return u;
            }

            // token didn't expire yet, so just return the user as-is
            svc.user_service().lock().add(u.clone());

            return u;
        }

        // user exists, but token doesn't exist anymore
        // we'll generate a new virtual user for you

        let (u, t) = generate_virtual_user(svc);

        set_cookies(&u, &t, cookies);
        return u;
    }
    // user doesn't even exist within the db
    let (u, t) = generate_virtual_user(svc);

    set_cookies(&u, &t, cookies);
    u
}

/// Retrieve a list of all users
///
/// The object returned contains all users currently known by the server.

#[openapi(tag = "Users")]
#[get("/users")]
pub fn get_all(serv: &State<ServiceStore>) -> Json<UsersResponse> {
    Json(UsersResponse {
        users: serv.user_service().lock().get_all(),
    })
}

/// Get all info about a certain user
///
/// Retrieve all known info about a specific user. user_id must be identical to a user's id, either returned by POST /users, or by GET /users.
/// The info here is currently identical with what you get with GET /users, but that might change later.
///
/// This call will return a 404 error if the user_id provided doesn't exist.

#[openapi(tag = "Users")]
#[get("/users/<user_id>")]
pub fn get(
    user_id: &str,
    serv: &State<ServiceStore>,
) -> Result<Json<User>, NotFound<Json<MessageResponse>>> {
    if let Ok(u) = Uuid::parse_str(user_id) {
        match serv.user_service().lock().get_by_id(u) {
            Some(u) => Ok(Json(u)),
            None => Err(NotFound(Json(MessageResponse {
                message: "user id not found".into(),
                r#type: "error".into(),
            }))),
        }
    } else {
        Err(NotFound(Json(MessageResponse {
            message: "user id is not valid".into(),
            r#type: "error".into(),
        })))
    }
}

/// User login
///
/// The user will log in with the provided username and password

#[openapi(tag = "Users")]
#[post("/users/login", format = "json", data = "<credentials>")]
pub async fn login(
    credentials: Json<UserLoginPayload>,
    mut db: Connection<HitsterConfig>,
    cookies: &CookieJar<'_>,
    serv: &State<ServiceStore>,
) -> Result<Json<User>, NotFound<Json<MessageResponse>>> {
    if let Some(mut user) = serv
        .user_service()
        .lock()
        .get_by_username(credentials.username.as_str())
    {
        let password_hash = PasswordHash::new(&user.password).unwrap();
        if Argon2::default()
            .verify_password(credentials.password.as_bytes(), &password_hash)
            .is_ok()
            && !user.r#virtual
        {
            let t = Token {
                token: generate_token(),
                expiration_time: Time(OffsetDateTime::now_utc() + Duration::hours(1)),
                refresh_time: Time(OffsetDateTime::now_utc() + Duration::days(7)),
            };

            set_cookies(&user, &t, cookies);

            user.tokens.push(t);

            serv.user_service().lock().add(user.clone());

            return Ok(Json(user));
        } else {
            return Err(NotFound(Json(MessageResponse {
                message: "incorrect user credentials".into(),
                r#type: "error".into(),
            })));
        }
    }

    sqlx::query("SELECT * FROM users where name = ?")
        .bind(credentials.username.as_str())
        .fetch_optional(&mut **db)
        .await
        .unwrap()
        .and_then(|user| {
            let mut u = User {
                id: Uuid::parse_str(&user.get::<String, &str>("id")).unwrap(),
                name: user.get::<String, &str>("name"),
                password: user.get::<String, &str>("password"),
                r#virtual: false,
                tokens: serde_json::from_str::<Vec<Token>>(&user.get::<String, &str>("tokens"))
                    .unwrap(),
            };

            let password_hash = PasswordHash::new(&u.password).unwrap();
            if Argon2::default()
                .verify_password(credentials.password.as_bytes(), &password_hash)
                .is_ok()
            {
                let t = Token {
                    token: generate_token(),
                    expiration_time: Time(OffsetDateTime::now_utc() + Duration::hours(1)),
                    refresh_time: Time(OffsetDateTime::now_utc() + Duration::days(7)),
                };

                set_cookies(&u, &t, cookies);

                u.tokens.push(t);
                serv.user_service().lock().add(u.clone());

                Some(Json(u))
            } else {
                None
            }
        })
        .ok_or(NotFound(Json(MessageResponse {
            message: "incorrect user credentials".into(),
            r#type: "error".into(),
        })))
}

/// Register a new user
///
/// Register a new user with a given username and password

#[openapi(tag = "Users")]
#[post("/users/register", format = "json", data = "<credentials>")]
pub async fn register(
    mut credentials: Json<UserLoginPayload>,
    mut db: Connection<HitsterConfig>,
) -> Result<Json<MessageResponse>, NotFound<Json<MessageResponse>>> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    credentials.password = argon2
        .hash_password(credentials.password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    if sqlx::query("SELECT * FROM users WHERE name = ?")
        .bind(credentials.username.as_str())
        .fetch_optional(&mut **db)
        .await
        .unwrap()
        .is_some()
    {
        Err(NotFound(Json(MessageResponse {
            message: "username is already in use".into(),
            r#type: "error".into(),
        })))
    } else if sqlx::query("INSERT INTO users (id, name, password, tokens) VALUES (?1, ?2, ?3, ?4)")
        .bind(Uuid::new_v4().to_string())
        .bind(credentials.username.as_str())
        .bind(credentials.password.as_str())
        .bind("")
        .execute(&mut **db)
        .await
        .is_ok()
    {
        Ok(Json(MessageResponse {
            message: "user created".into(),
            r#type: "success".into(),
        }))
    } else {
        Err(NotFound(Json(MessageResponse {
            message: "error while creating a database entry".into(),
            r#type: "error".into(),
        })))
    }
}

/// Logout user
///
/// Logout user and clear cookies.

#[openapi(tag = "Users")]
#[post("/users/logout")]
pub async fn logout(
    user: User,
    serv: &State<ServiceStore>,
    cookies: &CookieJar<'_>,
) -> Json<MessageResponse> {
    let game_srv = serv.game_service();
    let games = game_srv.lock();

    for game in games.get_all(Some(&user)).iter() {
        let _ = games.leave(&game.id, &user, None);
    }

    let user_srv = serv.user_service();
    let users = user_srv.lock();

    cookies.remove_private("id");
    cookies.remove("user");
    users.remove(user.id);

    Json(MessageResponse {
        message: "logged out".into(),
        r#type: "success".into(),
    })
}

#[openapi(tag = "Users")]
#[get("/users/auth")]
pub async fn authorize(
    db: Connection<HitsterConfig>,
    cookies: &CookieJar<'_>,
    serv: &State<ServiceStore>,
) -> Json<MessageResponse> {
    let token = cookies
        .get_private("id")
        .map(|cookie| cookie.value().to_string());

    let user = cookies
        .get("user")
        .and_then(|cookie| serde_json::from_str::<UserCookie>(cookie.value()).ok());

    if user.is_some() && token.is_some() {
        handle_existing_token(
            token.as_ref().unwrap(),
            user.as_ref().unwrap(),
            serv,
            cookies,
            db,
        )
        .await;

        Json(MessageResponse {
            message: "success".into(),
            r#type: "success".into(),
        })
    } else {
        let (u, t) = generate_virtual_user(serv);

        set_cookies(&u, &t, cookies);

        Json(MessageResponse {
            message: "success".into(),
            r#type: "success".into(),
        })
    }
}
