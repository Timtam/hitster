use crate::games::Game;
use crate::responses::{ErrorResponse, GamesResponse};
use crate::services::{GameService, UserService};
use rocket::{response::status::NotFound, serde::json::Json, State};
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
) -> Result<Json<Game>, NotFound<Json<ErrorResponse>>> {
    match users.get(user_id) {
        Some(u) => Ok(Json(games.add(u))),
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
    use super::GamesResponse;
    use crate::{games::Game, rocket, users::User};
    use rocket::{http::Status, local::blocking::Client};

    #[test]
    fn can_create_game() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.post(uri!("/users")).dispatch();
        let game = client.post(uri!("/games/1")).dispatch();

        assert_eq!(game.status(), Status::Ok);
        assert!(game.into_json::<Game>().is_some());
    }

    #[test]
    fn each_game_gets_individual_ids() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let user = client.post(uri!("/users")).dispatch();
        let game1 = client.post(uri!("/games/1")).dispatch();
        let game2 = client.post(uri!("/games/1")).dispatch();
        assert_ne!(
            game1.into_json::<Game>().unwrap().id,
            game2.into_json::<Game>().unwrap().id
        );
    }

    #[test]
    fn creating_a_game_with_invalid_user_causes_errors() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let game = client.post(uri!("/games/1")).dispatch();
        assert_eq!(game.status(), Status::NotFound);
    }

    #[test]
    fn can_read_all_games() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let user = client.post(uri!("/users")).dispatch();
        let game1 = client.post(uri!("/games/1")).dispatch();
        let game2 = client.post(uri!("/games/1")).dispatch();
        let games = client.get(uri!("/games")).dispatch();
        assert_eq!(games.status(), Status::Ok);
        assert_eq!(
            games.into_json::<GamesResponse>().unwrap().games,
            vec![
                game1.into_json::<Game>().unwrap(),
                game2.into_json::<Game>().unwrap()
            ]
        );
    }
}
