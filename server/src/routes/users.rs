use crate::{
    responses::{MessageResponse, UsersResponse},
    services::UserService,
    users::{User, UserLoginPayload},
    HitsterConfig,
};
use rocket::{
    http::{Cookie, CookieJar},
    response::status::{Created, NotFound},
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
    credentials.password = format!("{:x?}", hasher.finalize());

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

#[cfg(test)]
mod tests {
    use crate::responses::UsersResponse;
    use crate::{test::mocked_client, users::User};
    use rocket::http::Status;

    #[sqlx::test]
    async fn can_create_user() {
        let client = mocked_client().await;
        let response = client.post(uri!("/users")).dispatch().await;
        assert_eq!(response.status(), Status::Created);
        assert!(response.into_json::<User>().await.is_some());
    }

    #[sqlx::test]
    async fn each_user_gets_individual_ids() {
        let client = mocked_client().await;
        let user1 = client.post(uri!("/users")).dispatch().await;
        let user2 = client.post(uri!("/users")).dispatch().await;
        assert_ne!(
            user1.into_json::<User>().await.unwrap().id,
            user2.into_json::<User>().await.unwrap().id
        );
    }

    #[sqlx::test]
    async fn can_read_all_users() {
        let client = mocked_client().await;
        let user1 = client.post(uri!("/users")).dispatch().await;
        let user2 = client.post(uri!("/users")).dispatch().await;
        let users = client.get(uri!("/users")).dispatch().await;
        assert_eq!(users.status(), Status::Ok);
        assert_eq!(
            users.into_json::<UsersResponse>().await.unwrap().users,
            vec![
                user1.into_json::<User>().await.unwrap(),
                user2.into_json::<User>().await.unwrap()
            ]
        );
    }

    #[sqlx::test]
    async fn cause_error_when_retrieving_invalid_user() {
        let client = mocked_client().await;
        let response = client.get(uri!("/users/1")).dispatch().await;
        assert_eq!(response.status(), Status::NotFound);
    }

    #[sqlx::test]
    async fn can_get_single_user() {
        let client = mocked_client().await;
        let user = client.post(uri!("/users")).dispatch().await;
        let response = client.get(uri!("/users/1")).dispatch().await;

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.into_json::<User>().await.unwrap(),
            user.into_json::<User>().await.unwrap()
        );
    }
}
