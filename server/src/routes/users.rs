use crate::services::UserService;
use crate::users::User;
use rocket::{serde::json::Json, State};
use rocket_okapi::openapi;

#[openapi(tag = "Users")]
#[post("/users")]
pub fn create_user(users: &State<UserService>) -> Json<User> {
    Json(users.add())
}

#[cfg(test)]
mod tests {
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
}
