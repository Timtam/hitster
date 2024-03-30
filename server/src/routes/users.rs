use crate::{
    responses::{MessageResponse, UsersResponse},
    services::ServiceStore,
    users::{User, UserLoginPayload},
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

/// Create a new user
///
/// The username will be auto-generated for you. It is planned to be able to change it later.
/// For now the id returned by this API call will need to be stored to use this user later.

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

    for game in games.get_all().iter() {
        let _ = games.leave(game.id, &user);
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

#[cfg(test)]
pub mod tests {
    use crate::{
        responses::{GameResponse, GamesResponse, UsersResponse},
        routes::games as games_routes,
        test::mocked_client,
        users::{User, UserLoginPayload},
    };
    use futures::future::join_all;
    use rocket::{
        http::{ContentType, Cookie, Status},
        local::asynchronous::Client,
    };
    use serde_json;
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

    pub async fn create_test_users<'a, 'b>(client: &'a Client, amount: u8) -> Vec<Cookie<'b>> {
        join_all((1..=amount).into_iter().map(|i| async move {
            client
                .post(uri!("/api", super::signup))
                .header(ContentType::JSON)
                .body(
                    serde_json::to_string(&UserLoginPayload {
                        username: format!("testuser{}", i),
                        password: "abc1234".into(), // don't do this in practice!
                    })
                    .unwrap(),
                )
                .dispatch()
                .await;

            client
                .post(uri!("/api", super::login))
                .header(ContentType::JSON)
                .body(
                    serde_json::to_string(&UserLoginPayload {
                        username: format!("testuser{}", i),
                        password: "abc1234".into(), // don't do this in practice!
                    })
                    .unwrap(),
                )
                .dispatch()
                .await
                .cookies()
                .get_private("login")
                .unwrap()
        }))
        .await
    }

    #[sqlx::test]
    async fn can_create_user() {
        let client = mocked_client().await;
        let response = client
            .post(uri!("/api", super::signup))
            .header(ContentType::JSON)
            .body(
                serde_json::to_string(&UserLoginPayload {
                    username: "testuser".into(),
                    password: "123abcd".into(), // don't do this in practice!
                })
                .unwrap(),
            )
            .dispatch()
            .await;
        assert_eq!(
            response.status(),
            Status::Ok,
            "returned {}",
            response.into_string().await.unwrap()
        );
    }

    #[sqlx::test]
    async fn each_user_gets_individual_ids(
        _pool_opt: SqlitePoolOptions,
        _conn_opt: SqliteConnectOptions,
    ) -> sqlx::Result<()> {
        let client = mocked_client().await;

        create_test_users(&client, 2).await;

        let users = client
            .get(uri!("/api", super::get_all_users))
            .dispatch()
            .await
            .into_json::<UsersResponse>()
            .await
            .unwrap();

        assert_ne!(
            users.users.get(0).unwrap().id,
            users.users.get(1).unwrap().id
        );

        Ok(())
    }

    #[sqlx::test]
    async fn can_read_all_users() {
        let client = mocked_client().await;

        create_test_users(&client, 2).await;

        let response = client
            .get(uri!("/api", super::get_all_users))
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response
                .into_json::<UsersResponse>()
                .await
                .unwrap()
                .users
                .len(),
            2
        );
    }

    #[sqlx::test]
    async fn cause_error_when_retrieving_invalid_user() {
        let client = mocked_client().await;
        let response = client
            .get(uri!("/api", super::get_user(user_id = 1)))
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::NotFound);
    }

    #[sqlx::test]
    async fn can_get_single_user() {
        let client = mocked_client().await;

        create_test_users(&client, 1).await;

        let response = client
            .get(uri!("/api", super::get_user(user_id = 1)))
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);

        let user = response.into_json::<User>().await.unwrap();

        assert_eq!(user.username, "testuser1".to_string(),);
    }

    #[sqlx::test]
    async fn can_login() {
        let client = mocked_client().await;

        client
            .post(uri!("/api", super::signup))
            .header(ContentType::JSON)
            .body(
                serde_json::to_string(&UserLoginPayload {
                    username: "testuser".into(),
                    password: "123abcd".into(), // don't do this in practice!
                })
                .unwrap(),
            )
            .dispatch()
            .await;
        let response = client
            .post(uri!("/api", super::login))
            .header(ContentType::JSON)
            .body(
                serde_json::to_string(&UserLoginPayload {
                    username: "testuser".into(),
                    password: "123abcd".into(), // don't do this in practice!
                })
                .unwrap(),
            )
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
    }

    #[sqlx::test]
    async fn can_access_non_private_logged_in_cookie() {
        let client = mocked_client().await;

        client
            .post(uri!("/api", super::signup))
            .header(ContentType::JSON)
            .body(
                serde_json::to_string(&UserLoginPayload {
                    username: "testuser".into(),
                    password: "123abcd".into(), // don't do this in practice!
                })
                .unwrap(),
            )
            .dispatch()
            .await;
        assert_eq!(
            serde_json::from_str::<User>(
                client
                    .post(uri!("/api", super::login))
                    .header(ContentType::JSON)
                    .body(
                        serde_json::to_string(&UserLoginPayload {
                            username: "testuser".into(),
                            password: "123abcd".into(), // don't do this in practice!
                        })
                        .unwrap(),
                    )
                    .dispatch()
                    .await
                    .cookies()
                    .get("logged_in")
                    .unwrap()
                    .value()
            )
            .unwrap()
            .username,
            "testuser"
        );
    }

    #[sqlx::test]
    async fn wrong_credentials_on_login_cause_an_error() {
        let client = mocked_client().await;

        let response = client
            .post(uri!("/api", super::login))
            .header(ContentType::JSON)
            .body(
                serde_json::to_string(&UserLoginPayload {
                    username: "testuser1".into(),
                    password: "".into(),
                })
                .unwrap(),
            )
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::NotFound);
    }

    #[sqlx::test]
    async fn can_logout() {
        let client = mocked_client().await;

        assert_eq!(
            client
                .post(uri!("/api", super::logout))
                .private_cookie(create_test_users(&client, 1).await.get(0).cloned().unwrap())
                .dispatch()
                .await
                .status(),
            Status::Ok
        );
    }

    #[sqlx::test]
    async fn cannot_logout_if_not_logged_in() {
        let client = mocked_client().await;

        assert_eq!(
            client
                .post(uri!("/api", super::logout))
                .dispatch()
                .await
                .status(),
            Status::Unauthorized
        );
    }

    #[sqlx::test]
    async fn wrong_cookie_causes_authorization_errors() {
        let client = mocked_client().await;

        assert_eq!(
            client
                .post(uri!("/api", super::logout))
                .private_cookie(Cookie::new(
                    "login",
                    "this is totally not the expected payload"
                ))
                .dispatch()
                .await
                .status(),
            Status::Unauthorized
        );
    }

    #[sqlx::test]
    async fn leave_game_when_logging_out() {
        let client = mocked_client().await;

        let cookies = create_test_users(&client, 2).await;

        let game = client
            .post(uri!("/api", games_routes::create_game))
            .private_cookie(cookies.get(0).cloned().unwrap())
            .dispatch()
            .await;

        client
            .patch(uri!(
                "/api",
                games_routes::join_game(
                    game_id = game.into_json::<GameResponse>().await.unwrap().id
                )
            ))
            .private_cookie(cookies.get(1).cloned().unwrap())
            .dispatch()
            .await;

        assert_eq!(
            client
                .get(uri!("/api", games_routes::get_all_games))
                .dispatch()
                .await
                .into_json::<GamesResponse>()
                .await
                .unwrap()
                .games
                .get(0)
                .unwrap()
                .players
                .len(),
            2
        );

        client
            .post(uri!("/api", super::logout))
            .private_cookie(cookies.get(1).cloned().unwrap())
            .dispatch()
            .await;

        assert_eq!(
            client
                .get(uri!("/api", games_routes::get_all_games))
                .dispatch()
                .await
                .into_json::<GamesResponse>()
                .await
                .unwrap()
                .games
                .get(0)
                .unwrap()
                .players
                .len(),
            1
        );
    }

    #[sqlx::test]
    async fn select_new_game_creator_after_logging_out() {
        let client = mocked_client().await;

        let cookies = create_test_users(&client, 2).await;

        let game = client
            .post(uri!("/api", games_routes::create_game))
            .private_cookie(cookies.get(0).cloned().unwrap())
            .dispatch()
            .await;

        client
            .patch(uri!(
                "/api",
                games_routes::join_game(
                    game_id = game.into_json::<GameResponse>().await.unwrap().id
                )
            ))
            .private_cookie(cookies.get(1).cloned().unwrap())
            .dispatch()
            .await;

        client
            .post(uri!("/api", super::logout))
            .private_cookie(cookies.get(0).cloned().unwrap())
            .dispatch()
            .await;

        assert_eq!(
            client
                .get(uri!("/api", games_routes::get_all_games))
                .dispatch()
                .await
                .into_json::<GamesResponse>()
                .await
                .unwrap()
                .games
                .get(0)
                .unwrap()
                .creator
                .id,
            2
        );
    }

    #[sqlx::test]
    async fn delete_game_after_last_player_logs_out() {
        let client = mocked_client().await;

        let cookie = create_test_users(&client, 1).await.get(0).cloned().unwrap();

        client
            .post(uri!("/api", games_routes::create_game))
            .private_cookie(cookie.clone())
            .dispatch()
            .await;

        client
            .post(uri!("/api", super::logout))
            .private_cookie(cookie)
            .dispatch()
            .await;

        assert_eq!(
            client
                .get(uri!("/api", games_routes::get_all_games))
                .dispatch()
                .await
                .into_json::<GamesResponse>()
                .await
                .unwrap()
                .games
                .len(),
            0
        );
    }
}
