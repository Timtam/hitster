use crate::{
    responses::{MessageResponse, UsersResponse},
    services::ServiceStore,
    users::User,
    HitsterConfig,
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
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

/// Retrieve a list of all users
///
/// The object returned contains all users currently known by the server.

#[openapi(tag = "Users")]
#[get("/users")]
pub fn get_all_users(serv: &State<ServiceStore>) -> Json<UsersResponse> {
    Json(UsersResponse {
        users: serv.user_service().lock().get_all(),
    })
}

/*
/// Get all info about a certain user
///
/// Retrieve all known info about a specific user. user_id must be identical to a user's id, either returned by POST /users, or by GET /users.
/// The info here is currently identical with what you get with GET /users, but that might change later.
///
/// This call will return a 404 error if the user_id provided doesn't exist.

#[openapi(tag = "Users")]
#[get("/users/<user_id>")]
pub fn get_user(
    user_id: u32,
    serv: &State<ServiceStore>,
) -> Result<Json<User>, NotFound<Json<MessageResponse>>> {
    match serv.user_service().lock().get_by_id(user_id) {
        Some(u) => Ok(Json(u)),
        None => Err(NotFound(Json(MessageResponse {
            message: "user id not found".into(),
            r#type: "error".into(),
        }))),
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
    if let Some(user) = serv
        .user_service()
        .lock()
        .get_by_username(credentials.username.as_str())
    {
        let password_hash = PasswordHash::new(&user.password).unwrap();
        if Argon2::default()
            .verify_password(credentials.password.as_bytes(), &password_hash)
            .is_ok()
        {
            cookies.add_private(Cookie::new(
                "login",
                serde_json::to_string(&*credentials).unwrap(),
            ));

            cookies.add(
                Cookie::build(("logged_in", serde_json::to_string(&user).unwrap()))
                    .http_only(false)
                    .expires(OffsetDateTime::now_utc() + Duration::days(7)),
            );
            return Ok(Json(user));
        } else {
            return Err(NotFound(Json(MessageResponse {
                message: "incorrect user credentials".into(),
                r#type: "error".into(),
            })));
        }
    }

    sqlx::query("SELECT * FROM users where username = ?")
        .bind(credentials.username.as_str())
        .fetch_optional(&mut **db)
        .await
        .unwrap()
        .and_then(|user| {
            let u = User {
                id: user.get::<u32, &str>("id"),
                username: user.get::<String, &str>("username"),
                password: user.get::<String, &str>("password"),
            };

            let password_hash = PasswordHash::new(&u.password).unwrap();
            if Argon2::default()
                .verify_password(credentials.password.as_bytes(), &password_hash)
                .is_ok()
            {
                serv.user_service().lock().add(u.clone());

                cookies.add_private(Cookie::new(
                    "login",
                    serde_json::to_string(&*credentials).unwrap(),
                ));

                cookies.add(
                    Cookie::build(("logged_in", serde_json::to_string(&u).unwrap()))
                        .http_only(false)
                        .expires(OffsetDateTime::now_utc() + Duration::days(7)),
                );

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
#[post("/users/signup", format = "json", data = "<credentials>")]
pub async fn signup(
    mut credentials: Json<UserLoginPayload>,
    serv: &State<ServiceStore>,
    mut db: Connection<HitsterConfig>,
) -> Result<Json<MessageResponse>, NotFound<Json<MessageResponse>>> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    credentials.password = argon2
        .hash_password(credentials.password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    if sqlx::query("SELECT * FROM users WHERE username = ?")
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
    } else if let Ok(result) = sqlx::query("INSERT INTO users (username, password) VALUES (?1, ?2)")
        .bind(credentials.username.as_str())
        .bind(credentials.password.as_str())
        .execute(&mut **db)
        .await
    {
        serv.user_service().lock().add(User {
            id: result.last_insert_rowid() as u32,
            username: credentials.username.clone(),
            password: credentials.password.clone(),
        });

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

    cookies.remove_private("login");
    cookies.remove("logged_in");
    users.remove(user.id);

    Json(MessageResponse {
        message: "logged out".into(),
        r#type: "success".into(),
    })
}
*/
