use crate::{
    HitsterConfig,
    responses::{GetUserError, MessageResponse, RegisterUserError, UserLoginError, UsersResponse},
    routes::captcha::verify_captcha,
    services::ServiceStore,
    users::{
        UserAuthenticator, UserCookie, UserLoginPayload, UserPayload, UserRegistrationPayload,
    },
};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use hitster_core::{Permissions, Token, User};
use petname::{Generator, Petnames};
use rand::{RngCore, SeedableRng, rng};
use rand_chacha::ChaCha8Rng;
use rocket::{
    State,
    http::{Cookie, CookieJar},
    serde::json::Json,
};
use rocket_db_pools::{Connection, sqlx};
use rocket_okapi::openapi;
use serde_json;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

// internals

fn generate_token() -> String {
    let mut rng = rng();
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
        permissions: Permissions::default(),
    };

    let t = Token {
        token: generate_token(),
        expiration_time: OffsetDateTime::now_utc() + Duration::hours(1),
        refresh_time: OffsetDateTime::now_utc() + Duration::days(7),
    };

    u.tokens.push(t.clone());

    svc.user_service().lock().add(u.clone());

    (u, t)
}

fn newest_valid_token(user: &User) -> Option<Token> {
    user.tokens
        .iter()
        .filter(|t| t.expiration_time > OffsetDateTime::now_utc())
        .max_by_key(|t| t.expiration_time)
        .cloned()
}

fn set_cookies(user: &User, token: &Token, cookies: &CookieJar<'_>) {
    let mut uc = UserCookie::from(user);

    uc.valid_until = token.expiration_time;

    cookies.add_private(Cookie::build(("id", token.token.clone())).expires(token.refresh_time));
    cookies.add(
        Cookie::build(("user", serde_json::to_string(&uc).unwrap()))
            .http_only(false)
            .expires(token.refresh_time),
    );
}

async fn handle_existing_token(
    token: &str,
    user: &UserCookie,
    svc: &ServiceStore,
    cookies: &CookieJar<'_>,
    mut db: Connection<HitsterConfig>,
) -> User {
    let u = svc
        .user_service()
        .lock()
        .get_by_username(user.name.as_str());

    rocket::debug!(
        "user {} ({}) is trying to authorize with token {}",
        user.id,
        user.name,
        token
    );

    if let Some(mut u) = u {
        // the user already exists within the user service

        rocket::debug!("user found within the user service");

        if let Some(t) = u.tokens.iter().find(|t| t.token == token) {
            // the token exists for the user

            rocket::debug!("token found: {}", serde_json::to_string(&t).unwrap());

            if t.expiration_time < OffsetDateTime::now_utc()
                && t.refresh_time > OffsetDateTime::now_utc()
            {
                if let Some(newest) = newest_valid_token(&u) {
                    set_cookies(&u, &newest, cookies);
                    return u;
                }

                // token expired, but can still be refreshed
                let t = Token {
                    token: generate_token(),
                    expiration_time: OffsetDateTime::now_utc() + Duration::hours(1),
                    refresh_time: OffsetDateTime::now_utc() + Duration::days(7),
                };

                u.tokens = u
                    .tokens
                    .into_iter()
                    .filter(|t| t.refresh_time >= OffsetDateTime::now_utc() && t.token == token)
                    .collect::<_>();
                u.tokens.push(t.clone());

                svc.user_service().lock().add(u.clone());

                if !u.r#virtual {
                    let _ = sqlx::query("UPDATE users SET tokens = ? WHERE id = ?")
                        .bind(serde_json::to_string(&u.tokens).unwrap())
                        .bind(u.id.to_string())
                        .execute(&mut **db)
                        .await;
                }

                rocket::debug!("new token {} generated", t.token);

                set_cookies(&u, &t, cookies);
                return u;
            }

            if t.refresh_time < OffsetDateTime::now_utc() {
                // token refresh time is up and you're not logged in anymore
                let (u, t) = generate_virtual_user(svc);

                rocket::debug!(
                    "token cannot be refreshed anymore, new user {} ({}) with token {} generated",
                    u.id,
                    u.name,
                    t.token
                );

                set_cookies(&u, &t, cookies);
                return u;
            }

            // token didn't expire yet, so just return the user as-is
            rocket::debug!("token is still valid");

            return u;
        }

        // user exists, but token doesn't exist anymore
        // we'll generate a new virtual user for you

        let (u, t) = generate_virtual_user(svc);

        rocket::debug!("user exists, but token is unknown");
        rocket::debug!(
            "new user {} ({}) with token {} generated",
            u.id,
            u.name,
            t.token
        );

        set_cookies(&u, &t, cookies);
        return u;
    }
    // user doesn't exist within the user service, but might still exist within the db
    if user.r#virtual {
        // nope, its a virtual one
        let (u, t) = generate_virtual_user(svc);

        rocket::debug!(
            "new virtual user {} ({}) with token {} generated to replace unknown virtual user",
            u.id,
            u.name,
            t.token
        );

        set_cookies(&u, &t, cookies);
        return u;
    }

    let name = user.name.as_str();

    if let Some(mut u) = sqlx::query_as::<_, User>("SELECT * FROM users WHERE name = ?")
        .bind(name)
        .fetch_optional(&mut **db)
        .await
        .unwrap()
    {
        // user exists within the db

        rocket::debug!("user found within the database");

        if let Some(t) = u.tokens.iter().find(|t| t.token == token) {
            // we found the token

            rocket::debug!("token found: {}", serde_json::to_string(&t).unwrap());

            if t.expiration_time < OffsetDateTime::now_utc()
                && t.refresh_time > OffsetDateTime::now_utc()
            {
                if let Some(newest) = newest_valid_token(&u) {
                    set_cookies(&u, &newest, cookies);
                    return u;
                }

                // token expired, but can still be refreshed
                let t = Token {
                    token: generate_token(),
                    expiration_time: OffsetDateTime::now_utc() + Duration::hours(1),
                    refresh_time: OffsetDateTime::now_utc() + Duration::days(7),
                };

                u.tokens = u
                    .tokens
                    .into_iter()
                    .filter(|t| t.refresh_time >= OffsetDateTime::now_utc() && t.token == token)
                    .collect::<_>();
                u.tokens.push(t.clone());

                svc.user_service().lock().add(u.clone());

                let _ = sqlx::query("UPDATE users SET tokens = ? WHERE id = ?")
                    .bind(serde_json::to_string(&u.tokens).unwrap())
                    .bind(u.id.to_string())
                    .execute(&mut **db)
                    .await;

                rocket::debug!("new token {} generated", t.token);

                set_cookies(&u, &t, cookies);
                return u;
            }

            if t.refresh_time < OffsetDateTime::now_utc() {
                // token refresh time is up and you're not logged in anymore
                let (u, t) = generate_virtual_user(svc);

                rocket::debug!(
                    "token cannot be refreshed anymore, new user {} ({}) with token {} generated",
                    u.id,
                    u.name,
                    t.token
                );

                set_cookies(&u, &t, cookies);
                return u;
            }

            // token didn't expire yet, so just return the user as-is
            svc.user_service().lock().add(u.clone());

            rocket::debug!("token is still valid");

            return u;
        }

        // user exists, but token doesn't exist anymore
        // we'll generate a new virtual user for you

        let (u, t) = generate_virtual_user(svc);

        rocket::debug!("user exists, but token is unknown");
        rocket::debug!(
            "new user {} ({}) with token {} generated",
            u.id,
            u.name,
            t.token
        );

        set_cookies(&u, &t, cookies);
        return u;
    }
    // user doesn't even exist within the db
    let (u, t) = generate_virtual_user(svc);

    set_cookies(&u, &t, cookies);

    rocket::debug!("user doesn't exist");
    rocket::debug!(
        "new user {} ({}) with token {} generated",
        u.id,
        u.name,
        t.token
    );

    u
}

/// # Retrieve a list of all users
///
/// The object returned contains all users currently known by the server.

#[openapi(tag = "Users")]
#[get("/users")]
pub fn get_all(serv: &State<ServiceStore>) -> Json<UsersResponse> {
    Json(UsersResponse {
        users: serv
            .user_service()
            .lock()
            .get_all()
            .iter()
            .map(|u| u.into())
            .collect::<Vec<_>>(),
    })
}

/// # Get all info about a certain user
///
/// Retrieve all known info about a specific user. user_id must be identical to a user's id, either returned by POST /users, or by GET /users.
/// The info here is currently identical with what you get with GET /users, but that might change later.

#[openapi(tag = "Users")]
#[get("/users/<user_id>")]
pub fn get(user_id: &str, serv: &State<ServiceStore>) -> Result<Json<UserPayload>, GetUserError> {
    if let Ok(u) = Uuid::parse_str(user_id) {
        match serv.user_service().lock().get_by_id(u) {
            Some(u) => Ok(Json((&u).into())),
            None => Err(GetUserError {
                message: "user id not found".into(),
                http_status_code: 404,
            }),
        }
    } else {
        Err(GetUserError {
            message: "user id is not valid".into(),
            http_status_code: 404,
        })
    }
}

/// # User login
///
/// The user will log in with the provided username and password
/// If successful, two cookies will be created/refreshed:
///
/// * a private one that contains the token used to authenticate against the server
///
/// * a public one that contains user id, validity information and the user's permissions

#[openapi(tag = "Users")]
#[post("/users/login", format = "json", data = "<credentials>")]
pub async fn login(
    credentials: Json<UserLoginPayload>,
    user: Option<UserAuthenticator>,
    mut db: Connection<HitsterConfig>,
    cookies: &CookieJar<'_>,
    serv: &State<ServiceStore>,
) -> Result<Json<UserPayload>, UserLoginError> {
    let u = serv
        .user_service()
        .lock()
        .get_by_username(credentials.username.as_str());

    if let Some(mut u) = u {
        let password_hash = PasswordHash::new(&u.password).unwrap();
        if Argon2::default()
            .verify_password(credentials.password.as_bytes(), &password_hash)
            .is_ok()
            && !u.r#virtual
        {
            let t = Token {
                token: generate_token(),
                expiration_time: OffsetDateTime::now_utc() + Duration::hours(1),
                refresh_time: OffsetDateTime::now_utc() + Duration::days(7),
            };

            set_cookies(&u, &t, cookies);

            if let Some(cu) = user {
                serv.user_service().lock().remove(cu.0.id);
            }

            u.tokens.push(t);

            u.tokens = u
                .tokens
                .into_iter()
                .filter(|t| t.refresh_time >= OffsetDateTime::now_utc())
                .collect::<_>();

            let _ = sqlx::query("UPDATE users SET tokens = ? WHERE id = ?")
                .bind(serde_json::to_string(&u.tokens).unwrap())
                .bind(u.id.to_string())
                .execute(&mut **db)
                .await;

            serv.user_service().lock().add(u.clone());

            return Ok(Json((&u).into()));
        } else {
            return Err(UserLoginError {
                message: "incorrect user credentials".into(),
                http_status_code: 401,
            });
        }
    }

    let name = credentials.username.as_str();

    if let Some(mut u) = sqlx::query_as::<_, User>("SELECT * FROM users WHERE name = ?")
        .bind(name)
        .fetch_optional(&mut **db)
        .await
        .unwrap()
    {
        let password_hash = PasswordHash::new(&u.password).unwrap();
        if Argon2::default()
            .verify_password(credentials.password.as_bytes(), &password_hash)
            .is_ok()
        {
            let t = Token {
                token: generate_token(),
                expiration_time: OffsetDateTime::now_utc() + Duration::hours(1),
                refresh_time: OffsetDateTime::now_utc() + Duration::days(7),
            };

            set_cookies(&u, &t, cookies);

            u.tokens.push(t);

            u.tokens = u
                .tokens
                .into_iter()
                .filter(|t| t.refresh_time >= OffsetDateTime::now_utc())
                .collect::<_>();

            let _ = sqlx::query("UPDATE users SET tokens = ? WHERE id = ?")
                .bind(serde_json::to_string(&u.tokens).unwrap())
                .bind(u.id.to_string())
                .execute(&mut **db)
                .await;

            serv.user_service().lock().add(u.clone());

            return Ok(Json((&u).into()));
        } else {
            return Err(UserLoginError {
                message: "incorrect user credentials".into(),
                http_status_code: 401,
            });
        }
    }

    Err(UserLoginError {
        message: "incorrect user credentials".into(),
        http_status_code: 401,
    })
}

/// # Register a new user
///
/// Register a new user with a given username and password

#[openapi(tag = "Users")]
#[post("/users/register", format = "json", data = "<credentials>")]
pub async fn register(
    mut credentials: Json<UserRegistrationPayload>,
    mut user: UserAuthenticator,
    mut db: Connection<HitsterConfig>,
    cookies: &CookieJar<'_>,
    svc: &State<ServiceStore>,
) -> Result<Json<MessageResponse>, RegisterUserError> {
    if !verify_captcha(&credentials.altcha_token) {
        return Err(RegisterUserError {
            message: "altcha solution incorrect".into(),
            http_status_code: 403,
        });
    }

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    credentials.password = argon2
        .hash_password(credentials.password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    let name = credentials.username.as_str();

    if sqlx::query_as::<_, User>("SELECT * FROM users WHERE name = ?")
        .bind(name)
        .fetch_optional(&mut **db)
        .await
        .unwrap()
        .is_some()
    {
        Err(RegisterUserError {
            message: "username is already in use".into(),
            http_status_code: 409,
        })
    } else if !user.0.r#virtual {
        Err(RegisterUserError {
            message: "A user is already authenticated and registered.".into(),
            http_status_code: 405,
        })
    } else if sqlx::query(
        "INSERT INTO users (id, name, password, tokens, permissions) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(user.0.id.to_string())
    .bind(credentials.username.as_str())
    .bind(credentials.password.as_str())
    .bind(serde_json::to_string(&user.0.tokens).unwrap())
    .bind(0)
    .execute(&mut **db)
    .await
    .is_ok()
    {
        user.0.name.clone_from(&credentials.username);
        user.0.password.clone_from(&credentials.password);
        user.0.r#virtual = false;
        user.0.permissions = Permissions::from_bits(0).unwrap();

        let token = cookies
            .get_private("id")
            .map(|c| c.value().to_string())
            .and_then(|t| user.0.tokens.iter().find(|ti| ti.token == t))
            .unwrap();

        set_cookies(&user.0, token, cookies);

        svc.user_service().lock().add(user.0);

        Ok(Json(MessageResponse {
            message: "user registered".into(),
            r#type: "success".into(),
        }))
    } else {
        Err(RegisterUserError {
            message: "error while creating a database entry.".into(),
            http_status_code: 500,
        })
    }
}

/// # Logout user
///
/// Logout user and clear cookies.

#[openapi(tag = "Users")]
#[post("/users/logout")]
pub async fn logout(
    user: UserAuthenticator,
    serv: &State<ServiceStore>,
    cookies: &CookieJar<'_>,
) -> Json<MessageResponse> {
    let game_srv = serv.game_service();
    let games = game_srv.lock();

    for game in games.get_all(Some(&user.0)).iter() {
        let _ = games.leave(&game.id, &user.0, None);
    }

    let user_srv = serv.user_service();
    let users = user_srv.lock();

    cookies.remove_private("id");
    cookies.remove("user");
    users.remove(user.0.id);

    Json(MessageResponse {
        message: "logged out".into(),
        r#type: "success".into(),
    })
}

/// # (re)authorize user
///
/// This endpoint is necessary to update a user's token.
/// The current token of the user is always just valid for a certain amount of time.
/// The user however also got a second token that can be used to refresh the first token.
/// This endpoint will check that the first token is either valid, or refreshes the first token with the refresh token.
/// Cookies will be updated accordingly.
/// If tokens are invalid or no token exists at all, a virtual user will be created and authorized instead.

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

    if let Some(user) = user.as_ref()
        && let Some(token) = token.as_ref()
    {
        handle_existing_token(token, user, serv, cookies, db).await;

        Json(MessageResponse {
            message: "success".into(),
            r#type: "success".into(),
        })
    } else {
        let (u, t) = generate_virtual_user(serv);

        rocket::debug!(
            "new user {} ({}) authorized with token {}",
            u.id,
            u.name,
            t.token
        );

        set_cookies(&u, &t, cookies);

        Json(MessageResponse {
            message: "success".into(),
            r#type: "success".into(),
        })
    }
}
