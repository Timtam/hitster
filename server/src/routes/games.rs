use crate::games::Game;
use crate::responses::{ErrorResponse, GamesResponse};
use crate::services::{GameService, UserService};
use rocket::{
    response::status::{Created, NotFound},
    serde::json::Json,
    State,
};
use rocket_okapi::openapi;

/// Create a new game
///
/// Create a new game by specifying the user_id of the user who will be the creator. The creator will be the only one who can change game properties later.
/// The API will return 404 if the user_id is invalid.

#[openapi(tag = "Games")]
#[post("/games/<user_id>")]
pub fn create_game(
    user_id: u64,
    users: &State<UserService>,
    games: &State<GameService>,
) -> Result<Created<Json<Game>>, NotFound<Json<ErrorResponse>>> {
    match users.get(user_id) {
        Some(u) => {
            let game = games.add(u);

            Ok(Created::new(format!("/games/{}", game.id)).body(Json(game)))
        }
        None => Err(NotFound(Json(ErrorResponse {
            error: "user id not found".into(),
        }))),
    }
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
    use crate::{games::Game, responses::GamesResponse, test::mocked_client};
    use rocket::http::Status;

    #[sqlx::test]
    async fn can_create_game() {
        let client = mocked_client().await;
        client.post(uri!("/users")).dispatch().await;
        let game = client.post(uri!("/games/1")).dispatch().await;

        assert_eq!(game.status(), Status::Created);
        assert!(game.into_json::<Game>().await.is_some());
    }

    #[sqlx::test]
    async fn each_game_gets_individual_ids() {
        let client = mocked_client().await;
        client.post(uri!("/users")).dispatch().await;
        let game1 = client.post(uri!("/games/1")).dispatch().await;
        let game2 = client.post(uri!("/games/1")).dispatch().await;
        assert_ne!(
            game1.into_json::<Game>().await.unwrap().id,
            game2.into_json::<Game>().await.unwrap().id
        );
    }

    #[sqlx::test]
    async fn creating_a_game_with_invalid_user_causes_errors() {
        let client = mocked_client().await;
        let game = client.post(uri!("/games/1")).dispatch().await;
        assert_eq!(game.status(), Status::NotFound);
    }

    #[sqlx::test]
    async fn can_read_all_games() {
        let client = mocked_client().await;
        client.post(uri!("/users")).dispatch().await;
        let game1 = client.post(uri!("/games/1")).dispatch().await;
        let game2 = client.post(uri!("/games/1")).dispatch().await;
        let games = client.get(uri!("/games")).dispatch().await;
        assert_eq!(games.status(), Status::Ok);
        assert_eq!(
            games.into_json::<GamesResponse>().await.unwrap().games,
            vec![
                game1.into_json::<Game>().await.unwrap(),
                game2.into_json::<Game>().await.unwrap()
            ]
        );
    }
}
