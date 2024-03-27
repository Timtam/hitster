use crate::{
    games::{GameEvent, GameState},
    responses::{
        GameResponse, GamesResponse, JoinGameError, LeaveGameError, MessageResponse, StartGameError,
    },
    services::GameService,
    users::User,
};
use rocket::{
    response::{
        status::{Created, NotFound},
        stream::{Event, EventStream},
    },
    serde::json::Json,
    tokio::{
        select,
        sync::broadcast::{error::RecvError, Sender},
    },
    Shutdown, State,
};
use rocket_okapi::openapi;
use std::default::Default;

/// Create a new game
///
/// Create a new game. The currently logged in user will be the creator of the game. The creator will be the only one who can change game properties later.
/// The API will return 401 if the user_id is invalid.

#[openapi(tag = "Games")]
#[post("/games")]
pub fn create_game(user: User, games: &State<GameService>) -> Created<Json<GameResponse>> {
    let game = games.add(&user);

    Created::new(format!("/games/{}", game.id)).body(Json(GameResponse {
        id: game.id,
        creator: game.players.get(game.creator).cloned().unwrap(),
        players: game.players,
        state: game.state,
    }))
}

/// Retrieve all currently known games
///
/// Get a flat overview over all current games.
/// The info returned by this call is currently identical to GET /games/game_id or POST /games/user_id calls, but that might change later.

#[openapi(tag = "Games")]
#[get("/games")]
pub fn get_all_games(games: &State<GameService>) -> Json<GamesResponse> {
    Json(GamesResponse {
        games: games
            .get_all()
            .into_iter()
            .map(|game| GameResponse {
                id: game.id,
                creator: game.players.get(game.creator).cloned().unwrap(),
                players: game.players,
                state: game.state,
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
    games.join(game_id, &user).map(|_| {
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
    games.leave(game_id, &user).map(|_| {
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
    queue: &State<Sender<GameEvent>>,
) -> Result<Json<MessageResponse>, StartGameError> {
    games.start(game_id, &user).and_then(|_| {
        let _ = queue.send(GameEvent {
            game_id,
            event: "change_state".into(),
            state: Some(GameState::Guessing),
            ..Default::default()
        });

        Ok(Json(MessageResponse {
            message: "started game".into(),
            r#type: "success".into(),
        }))
    })
}

/// Get all info about a certain game
///
/// Retrieve all known info about a specific game. game_id must be identical to a game's id, either returned by POST /games, or by GET /games.
/// The info here is currently identical with what you get with GET /games, but that might change later.
///
/// This call will return a 404 error if the game_id provided doesn't exist.

#[openapi(tag = "Games")]
#[get("/games/<game_id>")]
pub fn get_game(
    game_id: u32,
    games: &State<GameService>,
) -> Result<Json<GameResponse>, NotFound<Json<MessageResponse>>> {
    match games.get(game_id) {
        Some(g) => Ok(Json(GameResponse {
            id: g.id,
            creator: g.players.get(g.creator).cloned().unwrap(),
            players: g.players,
            state: g.state,
        })),
        None => Err(NotFound(Json(MessageResponse {
            message: "game id not found".into(),
            r#type: "error".into(),
        }))),
    }
}

#[get("/games/<game_id>/events")]
pub async fn events(
    game_id: u32,
    queue: &State<Sender<GameEvent>>,
    mut end: Shutdown,
) -> EventStream![] {
    let mut rx = queue.subscribe();
    EventStream! {
        loop {
            let msg = select! {
                msg = rx.recv() => match msg {
                    Ok(msg) => msg,
                    Err(RecvError::Closed) => break,
                    Err(RecvError::Lagged(_)) => continue,
                },
                _ = &mut end => break,
            };

            if msg.game_id == game_id {
                yield Event::json(&msg);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        games::{GameEvent, GameState},
        responses::GameResponse,
        routes::users::tests::create_test_users,
        test::mocked_client,
    };
    use rocket::{
        http::Status,
        tokio::io::{AsyncBufReadExt, BufReader},
    };

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
    async fn can_read_single_game() {
        let client = mocked_client().await;

        let cookie = create_test_users(&client, 1).await.get(0).cloned().unwrap();

        let game = client
            .post(uri!("/api", super::create_game))
            .private_cookie(cookie.clone())
            .dispatch()
            .await
            .into_json::<GameResponse>()
            .await
            .unwrap();

        let response = client
            .get(uri!("/api", super::get_game(game_id = game.id)))
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
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

    #[sqlx::test]
    async fn can_read_event_when_starting_game() {
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

        let response = client
            .get(uri!("/api", super::events(game_id = game_id)))
            .dispatch()
            .await;

        client
            .patch(uri!("/api", super::start_game(game_id = game_id)))
            .private_cookie(cookies.get(0).cloned().unwrap())
            .dispatch()
            .await;

        let mut reader = BufReader::new(response).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            if !line.starts_with("data:") {
                continue;
            }

            let data: GameEvent = serde_json::from_str(&line[5..]).expect("message JSON");

            assert_eq!(data.event, "change_state");
            assert_eq!(data.state, Some(GameState::Guessing));
            break;
        }
    }
}
