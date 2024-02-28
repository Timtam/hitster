use crate::responses::{ErrorResponse, UsersResponse};
use crate::services::UserService;
use crate::users::User;
use rocket::{
    response::status::{Created, NotFound},
    serde::json::Json,
    State,
};
use rocket_okapi::openapi;

/// Create a new user
///
/// The username will be auto-generated for you. It is planned to be able to change it later.
/// For now the id returned by this API call will need to be stored to use this user later.

#[openapi(tag = "Users")]
#[post("/users")]
pub fn create_user(users: &State<UserService>) -> Created<Json<User>> {
    let user = users.add();

    Created::new(format!("/users/{}", user.id)).body(Json(user))
}

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
    user_id: u64,
    users: &State<UserService>,
) -> Result<Json<User>, NotFound<Json<ErrorResponse>>> {
    match users.get(user_id) {
        Some(u) => Ok(Json(u)),
        None => Err(NotFound(Json(ErrorResponse {
            error: "user id not found".into(),
        }))),
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
