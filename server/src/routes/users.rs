use crate::{
    responses::{MessageResponse, UsersResponse},
    services::UserService,
    users::{User, UserLoginPayload},
    HitsterConfig,
};
use hex;
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
use sha3::{Digest, Sha3_512};

/// Create a new user
///
/// The username will be auto-generated for you. It is planned to be able to change it later.
/// For now the id returned by this API call will need to be stored to use this user later.

/// Retrieve a list of all users
///
/// The object returned contains all users currently known by the server.

#[openapi(tag = "Users")]
#[get("/users")]
pub fn get_all_users(users: &State<UserService>) -> Json<UsersResponse> {
    Json(UsersResponse {
        users: users.get_all(),
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
    users: &State<UserService>,
) -> Result<Json<User>, NotFound<Json<MessageResponse>>> {
    match users.get_by_id(user_id) {
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
pub async fn user_login(
    mut credentials: Json<UserLoginPayload>,
    mut db: Connection<HitsterConfig>,
    cookies: &CookieJar<'_>,
    users: &State<UserService>,
) -> Result<Json<MessageResponse>, NotFound<Json<MessageResponse>>> {
    let mut hasher = Sha3_512::new();
    hasher.update(credentials.password.as_bytes());
    credentials.password = format!("{:?}", hex::encode(hasher.finalize()));

    if let Some(user) = users.get_by_username(credentials.username.as_str()) {
        if user.password == credentials.password {
            cookies.add_private(Cookie::new(
                "login",
                serde_json::to_string(&*credentials).unwrap(),
            ));

            Ok(Json(MessageResponse {
                message: "logged in successfully".into(),
                r#type: "success".into(),
            }))
        } else {
            Err(NotFound(Json(MessageResponse {
                message: "incorrect user credentials".into(),
                r#type: "error".into(),
            })))
        }
    } else {
        let user = sqlx::query("SELECT * FROM users where username = ?1 AND password = ?2")
            .bind(credentials.username.as_str())
            .bind(credentials.password.as_str())
            .fetch_optional(&mut **db)
            .await
            .unwrap();

        match user {
            Some(user) => {
                if users.get_by_id(user.get("id")).is_none() {
                    users.add(User {
                        id: user.get::<u32, &str>("id"),
                        username: user.get::<String, &str>("username"),
                        password: user.get::<String, &str>("password"),
                    });
                }

                cookies.add_private(Cookie::new(
                    "login",
                    serde_json::to_string(&*credentials).unwrap(),
                ));

                Ok(Json(MessageResponse {
                    message: "logged in successfully".into(),
                    r#type: "success".into(),
                }))
            }
            None => Err(NotFound(Json(MessageResponse {
                message: "incorrect user credentials".into(),
                r#type: "error".into(),
            }))),
        }
    }
}

/// Register a new user
///
/// Register a new user with a given username and password

#[openapi(tag = "Users")]
#[post("/users/signup", format = "json", data = "<credentials>")]
pub async fn user_signup(
    mut credentials: Json<UserLoginPayload>,
    users: &State<UserService>,
    mut db: Connection<HitsterConfig>,
) -> Result<Json<MessageResponse>, NotFound<Json<MessageResponse>>> {
    let mut hasher = Sha3_512::new();
    hasher.update(credentials.password.as_bytes());
    credentials.password = format!("{:?}", hex::encode(hasher.finalize()));

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
    } else {
        if let Ok(result) = sqlx::query("INSERT INTO users (username, password) VALUES (?1, ?2)")
            .bind(credentials.username.as_str())
            .bind(credentials.password.as_str())
            .execute(&mut **db)
            .await
        {
            users.add(User {
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
}

/// Logout user
///
/// Logout user and clear cookies.

#[openapi(tag = "Users")]
#[post("/users/logout")]
pub async fn user_logout(
    user: User,
    users: &State<UserService>,
    cookies: &CookieJar<'_>,
) -> Json<MessageResponse> {
    cookies.remove_private("login");
    users.remove(user.id);

    Json(MessageResponse {
        message: "logged out".into(),
        r#type: "success".into(),
    })
}

#[cfg(test)]
pub mod tests {
    use crate::{
        responses::UsersResponse,
        test::mocked_client,
        users::{User, UserLoginPayload},
    };
    use rocket::{
        http::{ContentType, Cookie, Status},
        local::asynchronous::Client,
    };
    use serde_json;
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

    pub async fn create_test_users(client: &Client) {
        client
            .post(uri!(super::user_signup))
            .header(ContentType::JSON)
            .body(
                serde_json::to_string(&UserLoginPayload {
                    username: "testuser1".into(),
                    password: "abc1234".into(), // don't do this in practice!
                })
                .unwrap(),
            )
            .dispatch()
            .await;
        client
            .post(uri!(super::user_signup))
            .header(ContentType::JSON)
            .body(
                serde_json::to_string(&UserLoginPayload {
                    username: "testuser2".into(),
                    password: "abc1234".into(), // don't do this in practice!
                })
                .unwrap(),
            )
            .dispatch()
            .await;
    }

    #[sqlx::test]
    async fn can_create_user() {
        let client = mocked_client().await;
        let response = client
            .post(uri!(super::user_signup))
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

        create_test_users(&client).await;

        client
            .post(uri!(super::user_login))
            .header(ContentType::JSON)
            .body(
                serde_json::to_string(&UserLoginPayload {
                    username: "testuser1".into(),
                    password: "abc1234".into(), // don't do this in practice!
                })
                .unwrap(),
            )
            .dispatch()
            .await;
        client
            .post(uri!(super::user_login))
            .header(ContentType::JSON)
            .body(
                serde_json::to_string(&UserLoginPayload {
                    username: "testuser2".into(),
                    password: "abc1234".into(), // don't do this in practice!
                })
                .unwrap(),
            )
            .dispatch()
            .await;

        let users = client
            .get(uri!(super::get_all_users))
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

        create_test_users(&client).await;

        let response = client.get(uri!(super::get_all_users)).dispatch().await;

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
            .get(uri!(super::get_user(user_id = 1)))
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::NotFound);
    }

    #[sqlx::test]
    async fn can_get_single_user() {
        let client = mocked_client().await;

        create_test_users(&client).await;

        let response = client
            .get(uri!(super::get_user(user_id = 1)))
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);

        let user = response.into_json::<User>().await.unwrap();

        assert_eq!(user.username, "testuser1".to_string(),);
    }

    #[sqlx::test]
    async fn can_login() {
        let client = mocked_client().await;

        create_test_users(&client).await;

        let response = client
            .post(uri!(super::user_login))
            .header(ContentType::JSON)
            .body(
                serde_json::to_string(&UserLoginPayload {
                    username: "testuser1".into(),
                    password: "abc1234".into(), // don't do this in practice!
                })
                .unwrap(),
            )
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
    }

    #[sqlx::test]
    async fn wrong_credentials_on_login_cause_an_error() {
        let client = mocked_client().await;

        let response = client
            .post(uri!(super::user_login))
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

        create_test_users(&client).await;

        let response = client
            .post(uri!(super::user_login))
            .header(ContentType::JSON)
            .body(
                serde_json::to_string(&UserLoginPayload {
                    username: "testuser1".into(),
                    password: "abc1234".into(),
                })
                .unwrap(),
            )
            .dispatch()
            .await;

        assert_eq!(
            client
                .post(uri!(super::user_logout))
                .private_cookie(response.cookies().get_private("login").unwrap())
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
                .post(uri!(super::user_logout))
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
                .post(uri!(super::user_logout))
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
}
