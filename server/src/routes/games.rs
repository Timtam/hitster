use crate::{
    responses::{
        GameResponse, GamesResponse, JoinGameError, LeaveGameError, MessageResponse, StartGameError,
    },
    services::{GameService, UserService},
    users::User,
};
use rocket::{response::status::Created, serde::json::Json, State};
use rocket_okapi::openapi;

/// Create a new game
///
/// Create a new game. The currently logged in user will be the creator of the game. The creator will be the only one who can change game properties later.
/// The API will return 401 if the user_id is invalid.

#[openapi(tag = "Games")]
#[post("/games")]
pub fn create_game(
    user: User,
    games: &State<GameService>,
    users: &State<UserService>,
) -> Created<Json<GameResponse>> {
    let game = games.add(user.id);

    Created::new(format!("/games/{}", game.id)).body(Json(GameResponse {
        id: game.id,
        creator: users.get_by_id(game.creator).unwrap(),
        players: game
            .players
            .into_iter()
            .map(|id| users.get_by_id(id).unwrap())
            .collect::<_>(),
    }))
}

/// Retrieve all currently known games
///
/// Get a flat overview over all current games.
/// The info returned by this call is currently identical to GET /games/game_id or POST /games/user_id calls, but that might change later.

#[openapi(tag = "Games")]
#[get("/games")]
pub fn get_all_games(
    games: &State<GameService>,
    users: &State<UserService>,
) -> Json<GamesResponse> {
    Json(GamesResponse {
        games: games
            .get_all()
            .into_iter()
            .map(|game| GameResponse {
                id: game.id,
                creator: users.get_by_id(game.creator).unwrap(),
                players: game
                    .players
                    .into_iter()
                    .map(|id| users.get_by_id(id).unwrap())
                    .collect::<_>(),
            })
            .collect::<_>(),
    })
}

#[openapi(tag = "Games")]
#[patch("/games/<game_id>/join")]
pub async fn join_game(
    game_id: u32,
    user: User,
    games: &State<GameService>,
) -> Result<Json<MessageResponse>, JoinGameError> {
    games.join(game_id, user.id).map(|_| {
        Json(MessageResponse {
            message: "joined the game successfully".into(),
            r#type: "success".into(),
        })
    })
}

#[openapi(tag = "Games")]
#[patch("/games/<game_id>/leave")]
pub async fn leave_game(
    game_id: u32,
    user: User,
    games: &State<GameService>,
) -> Result<Json<MessageResponse>, LeaveGameError> {
    games.leave(game_id, user.id).map(|_| {
        Json(MessageResponse {
            message: "left the game successfully".into(),
            r#type: "success".into(),
        })
    })
}

/// Start the game

#[openapi(tag = "Games")]
#[patch("/games/<game_id>/start")]
pub async fn start_game(
    game_id: u32,
    user: User,
    games: &State<GameService>,
) -> Result<Json<MessageResponse>, StartGameError> {
    games.start(game_id, user.id).map(|_| {
        Json(MessageResponse {
            message: "started game".into(),
            r#type: "success".into(),
        })
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        responses::GameResponse, routes::users::tests::create_test_users, test::mocked_client,
    };
    use rocket::http::Status;

    #[sqlx::test]
    async fn can_create_game() {
        let client = mocked_client().await;

        let cookie = create_test_users(&client, 1).await.get(0).cloned().unwrap();

        let game = client
            .post(uri!("/api", super::create_game))
            .private_cookie(cookie.clone())
            .dispatch()
            .await;

        assert_eq!(game.status(), Status::Created);
        assert!(game.into_json::<GameResponse>().await.is_some());
    }

    #[sqlx::test]
    async fn each_game_gets_individual_ids() {
        let client = mocked_client().await;

        let cookie = create_test_users(&client, 1).await.get(0).cloned().unwrap();

        let game1 = client
            .post(uri!("/api", super::create_game))
            .private_cookie(cookie.clone())
            .dispatch()
            .await;
        let game2 = client
            .post(uri!("/api", super::create_game))
            .private_cookie(cookie.clone())
            .dispatch()
            .await;

        assert_ne!(
            game1.into_json::<GameResponse>().await.unwrap().id,
            game2.into_json::<GameResponse>().await.unwrap().id
        );
    }

    #[sqlx::test]
    async fn can_join_game() {
        let client = mocked_client().await;

        let cookies = create_test_users(&client, 2).await;

        let game = client
            .post(uri!("/api", super::create_game))
            .private_cookie(cookies.get(0).cloned().unwrap())
            .dispatch()
            .await;

        assert_eq!(
            client
                .patch(uri!(
                    "/api",
                    super::join_game(game_id = game.into_json::<GameResponse>().await.unwrap().id)
                ))
                .private_cookie(cookies.get(1).cloned().unwrap())
                .dispatch()
                .await
                .status(),
            Status::Ok
        );
    }

    #[sqlx::test]
    async fn cannot_join_game_twice() {
        let client = mocked_client().await;

        let cookie = create_test_users(&client, 1).await.get(0).cloned().unwrap();

        let game = client
            .post(uri!("/api", super::create_game))
            .private_cookie(cookie.clone())
            .dispatch()
            .await;

        assert_eq!(
            client
                .patch(uri!(
                    "/api",
                    super::join_game(game_id = game.into_json::<GameResponse>().await.unwrap().id)
                ))
                .private_cookie(cookie)
                .dispatch()
                .await
                .status(),
            Status::Conflict
        );
    }

    #[sqlx::test]
    async fn can_leave_game() {
        let client = mocked_client().await;

        let cookie = create_test_users(&client, 1).await.get(0).cloned().unwrap();

        let game = client
            .post(uri!("/api", super::create_game))
            .private_cookie(cookie.clone())
            .dispatch()
            .await;

        assert_eq!(
            client
                .patch(uri!(
                    "/api",
                    super::leave_game(game_id = game.into_json::<GameResponse>().await.unwrap().id)
                ))
                .private_cookie(cookie)
                .dispatch()
                .await
                .status(),
            Status::Ok
        );
    }

    #[sqlx::test]
    async fn cannot_leave_game_twice() {
        let client = mocked_client().await;

        let cookie = create_test_users(&client, 1).await.get(0).cloned().unwrap();

        let game_id = client
            .post(uri!("/api", super::create_game))
            .private_cookie(cookie.clone())
            .dispatch()
            .await
            .into_json::<GameResponse>()
            .await
            .unwrap()
            .id;

        assert_eq!(
            client
                .patch(uri!("/api", super::leave_game(game_id = game_id)))
                .private_cookie(cookie.clone())
                .dispatch()
                .await
                .status(),
            Status::Ok
        );

        assert_eq!(
            client
                .patch(uri!("/api", super::leave_game(game_id = game_id)))
                .private_cookie(cookie)
                .dispatch()
                .await
                .status(),
            Status::NotFound
        );
    }

    #[sqlx::test]
    async fn cannot_leave_game_without_joining() {
        let client = mocked_client().await;

        let cookies = create_test_users(&client, 2).await;

        let game = client
            .post(uri!("/api", super::create_game))
            .private_cookie(cookies.get(0).cloned().unwrap())
            .dispatch()
            .await;

        assert_eq!(
            client
                .patch(uri!(
                    "/api",
                    super::leave_game(game_id = game.into_json::<GameResponse>().await.unwrap().id)
                ))
                .private_cookie(cookies.get(1).cloned().unwrap())
                .dispatch()
                .await
                .status(),
            Status::Conflict
        );
    }

    #[sqlx::test]
    async fn can_start_game() {
        let client = mocked_client().await;

        let cookies = create_test_users(&client, 2).await;

        let game_id = client
            .post(uri!("/api", super::create_game))
            .private_cookie(cookies.get(0).cloned().unwrap())
            .dispatch()
            .await
            .into_json::<GameResponse>()
            .await
            .unwrap()
            .id;

        client
            .patch(uri!("/api", super::join_game(game_id = game_id)))
            .private_cookie(cookies.get(1).cloned().unwrap())
            .dispatch()
            .await;

        assert_eq!(
            client
                .patch(uri!("/api", super::start_game(game_id = game_id)))
                .private_cookie(cookies.get(0).cloned().unwrap())
                .dispatch()
                .await
                .status(),
            Status::Ok
        );
    }

    #[sqlx::test]
    async fn only_creators_can_start_games() {
        let client = mocked_client().await;

        let cookies = create_test_users(&client, 2).await;

        let game_id = client
            .post(uri!("/api", super::create_game))
            .private_cookie(cookies.get(0).cloned().unwrap())
            .dispatch()
            .await
            .into_json::<GameResponse>()
            .await
            .unwrap()
            .id;

        client
            .patch(uri!("/api", super::join_game(game_id = game_id)))
            .private_cookie(cookies.get(1).cloned().unwrap())
            .dispatch()
            .await;

        assert_eq!(
            client
                .patch(uri!("/api", super::start_game(game_id = game_id)))
                .private_cookie(cookies.get(1).cloned().unwrap())
                .dispatch()
                .await
                .status(),
            Status::Forbidden
        );
    }

    #[sqlx::test]
    async fn cannot_start_game_with_too_few_players() {
        let client = mocked_client().await;

        let cookie = create_test_users(&client, 1).await.get(0).cloned().unwrap();

        let game_id = client
            .post(uri!("/api", super::create_game))
            .private_cookie(cookie.clone())
            .dispatch()
            .await
            .into_json::<GameResponse>()
            .await
            .unwrap()
            .id;

        assert_eq!(
            client
                .patch(uri!("/api", super::start_game(game_id = game_id)))
                .private_cookie(cookie)
                .dispatch()
                .await
                .status(),
            Status::Conflict
        );
    }

    #[sqlx::test]
    async fn cannot_start_game_that_is_already_running() {
        let client = mocked_client().await;

        let cookies = create_test_users(&client, 2).await;

        let game_id = client
            .post(uri!("/api", super::create_game))
            .private_cookie(cookies.get(0).cloned().unwrap())
            .dispatch()
            .await
            .into_json::<GameResponse>()
            .await
            .unwrap()
            .id;

        client
            .patch(uri!("/api", super::join_game(game_id = game_id)))
            .private_cookie(cookies.get(1).cloned().unwrap())
            .dispatch()
            .await;

        client
            .patch(uri!("/api", super::start_game(game_id = game_id)))
            .private_cookie(cookies.get(0).cloned().unwrap())
            .dispatch()
            .await;

        assert_eq!(
            client
                .patch(uri!("/api", super::start_game(game_id = game_id)))
                .private_cookie(cookies.get(0).cloned().unwrap())
                .dispatch()
                .await
                .status(),
            Status::Conflict
        );
    }
}
