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
    use super::UsersResponse;
    use crate::{rocket, users::User};
    use rocket::{http::Status, local::blocking::Client};

    #[test]
    fn can_create_user() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.post(uri!("/users")).dispatch();
        assert_eq!(response.status(), Status::Created);
        assert!(response.into_json::<User>().is_some());
    }

    #[test]
    fn each_user_gets_individual_ids() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let user1 = client.post(uri!("/users")).dispatch();
        let user2 = client.post(uri!("/users")).dispatch();
        assert_ne!(
            user1.into_json::<User>().unwrap().id,
            user2.into_json::<User>().unwrap().id
        );
    }

    #[test]
    fn can_read_all_users() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let user1 = client.post(uri!("/users")).dispatch();
        let user2 = client.post(uri!("/users")).dispatch();
        let users = client.get(uri!("/users")).dispatch();
        assert_eq!(users.status(), Status::Ok);
        assert_eq!(
            users.into_json::<UsersResponse>().unwrap().users,
            vec![
                user1.into_json::<User>().unwrap(),
                user2.into_json::<User>().unwrap()
            ]
        );
    }

    #[test]
    fn cause_error_when_retrieving_invalid_user() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get(uri!("/users/1")).dispatch();
        assert_eq!(response.status(), Status::NotFound);
    }

    #[test]
    fn can_get_single_user() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let user = client.post(uri!("/users")).dispatch();
        let response = client.get(uri!("/users/1")).dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            response.into_json::<User>().unwrap(),
            user.into_json::<User>().unwrap()
        );
    }
}
