use crate::services::UserService;
use crate::users::User;
use rocket::{serde::json::Json, State};
use rocket_okapi::{
    okapi::{schemars, schemars::JsonSchema},
    openapi,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct UsersResponse {
    pub users: Vec<User>,
}

#[openapi(tag = "Users")]
#[post("/users")]
pub fn create_user(users: &State<UserService>) -> Json<User> {
    Json(users.add())
}

#[openapi(tag = "Users")]
#[get("/users")]
pub fn get_all_users(users: &State<UserService>) -> Json<UsersResponse> {
    Json(UsersResponse {
        users: users.get_all(),
    })
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
        assert_eq!(response.status(), Status::Ok);
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
}
