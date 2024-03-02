use crate::{games::Game, responses::GamesResponse, services::GameService, users::User};
use rocket::{response::status::Created, serde::json::Json, State};
use rocket_okapi::openapi;

/// Create a new game
///
/// Create a new game. The currently logged in user will be the creator of the game. The creator will be the only one who can change game properties later.
/// The API will return 401 if the user_id is invalid.

#[openapi(tag = "Games")]
#[post("/games")]
pub fn create_game(user: User, games: &State<GameService>) -> Created<Json<Game>> {
    let game = games.add(user);

    Created::new(format!("/games/{}", game.id)).body(Json(game))
}

/// Retrieve all currently known games
///
/// Get a flat overview over all current games.
/// The info returned by this call is currently identical to GET /games/game_id or POST /games/user_id calls, but that might change later.

#[openapi(tag = "Games")]
#[get("/games")]
pub fn get_all_games(games: &State<GameService>) -> Json<GamesResponse> {
    Json(GamesResponse {
        games: games.get_all(),
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        games::Game, routes::users::tests::create_test_users, test::mocked_client,
        users::UserLoginPayload,
    };
    use rocket::http::{ContentType, Status};
    use serde_json;

    #[sqlx::test]
    async fn can_create_game() {
        let client = mocked_client().await;

        create_test_users(&client).await;

        let response = client
            .post(uri!("/users/login"))
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

        let game = client
            .post(uri!("/games"))
            .private_cookie(response.cookies().get_private("login").unwrap())
            .dispatch()
            .await;

        assert_eq!(game.status(), Status::Created);
        assert!(game.into_json::<Game>().await.is_some());
    }

    #[sqlx::test]
    async fn each_game_gets_individual_ids() {
        let client = mocked_client().await;

        create_test_users(&client).await;

        let response = client
            .post(uri!("/users/login"))
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

        let game1 = client
            .post(uri!("/games"))
            .private_cookie(response.cookies().get_private("login").unwrap())
            .dispatch()
            .await;
        let game2 = client
            .post(uri!("/games"))
            .private_cookie(response.cookies().get_private("login").unwrap())
            .dispatch()
            .await;

        assert_ne!(
            game1.into_json::<Game>().await.unwrap().id,
            game2.into_json::<Game>().await.unwrap().id
        );
    }
}
