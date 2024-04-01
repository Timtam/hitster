use crate::{
    games::{Game, GameEvent, GameState},
    responses::{
        CurrentHitError, GamesResponse, JoinGameError, LeaveGameError, MessageResponse,
        StartGameError, StopGameError,
    },
    services::ServiceStore,
    users::User,
};
use rocket::{
    fs::NamedFile,
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
pub fn create_game(user: User, serv: &State<ServiceStore>) -> Created<Json<Game>> {
    let game = serv.game_service().lock().add(&user);

    Created::new(format!("/games/{}", game.id)).body(Json(game))
}

/// Retrieve all currently known games
///
/// Get a flat overview over all current games.
/// The info returned by this call is currently identical to GET /games/game_id or POST /games/user_id calls, but that might change later.

#[openapi(tag = "Games")]
#[get("/games")]
pub fn get_all_games(serv: &State<ServiceStore>) -> Json<GamesResponse> {
    Json(GamesResponse {
        games: serv.game_service().lock().get_all(),
    })
}

#[openapi(tag = "Games")]
#[patch("/games/<game_id>/join")]
pub async fn join_game(
    game_id: u32,
    user: User,
    serv: &State<ServiceStore>,
    queue: &State<Sender<GameEvent>>,
) -> Result<Json<MessageResponse>, JoinGameError> {
    let game_svc = serv.game_service();
    let games = game_svc.lock();

    games.join(game_id, &user).map(|_| {
        let _ = queue.send(GameEvent {
            game_id,
            event: "join".into(),
            players: Some(games.get(game_id).unwrap().players),
            ..Default::default()
        });
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
    serv: &State<ServiceStore>,
    queue: &State<Sender<GameEvent>>,
) -> Result<Json<MessageResponse>, LeaveGameError> {
    let game_svc = serv.game_service();
    let games = game_svc.lock();

    let state = games
        .get(game_id)
        .map(|g| g.state)
        .unwrap_or(GameState::Open);

    games.leave(game_id, &user).map(|_| {
        let _ = queue.send(GameEvent {
            game_id,
            event: "leave".into(),
            players: games.get(game_id).map(|g| g.players),
            ..Default::default()
        });

        let new_state = games
            .get(game_id)
            .map(|g| g.state)
            .unwrap_or(GameState::Open);

        if new_state != state {
            let _ = queue.send(GameEvent {
                game_id,
                event: "change_state".into(),
                state: Some(new_state),
                ..Default::default()
            });
        }

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
    serv: &State<ServiceStore>,
    queue: &State<Sender<GameEvent>>,
) -> Result<Json<MessageResponse>, StartGameError> {
    let game_svc = serv.game_service();
    let games = game_svc.lock();

    games.start(game_id, &user).map(|g| {
        let _ = queue.send(GameEvent {
            game_id,
            event: "change_state".into(),
            state: Some(g.state),
            players: Some(g.players),
            ..Default::default()
        });

        Json(MessageResponse {
            message: "started game".into(),
            r#type: "success".into(),
        })
    })
}

#[openapi(tag = "Games")]
#[patch("/games/<game_id>/stop")]
pub async fn stop_game(
    game_id: u32,
    user: User,
    serv: &State<ServiceStore>,
    queue: &State<Sender<GameEvent>>,
) -> Result<Json<MessageResponse>, StopGameError> {
    serv.game_service()
        .lock()
        .stop(game_id, Some(&user))
        .map(|_| {
            let _ = queue.send(GameEvent {
                game_id,
                event: "change_state".into(),
                state: Some(GameState::Open),
                ..Default::default()
            });

            Json(MessageResponse {
                message: "stopped game".into(),
                r#type: "success".into(),
            })
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
    serv: &State<ServiceStore>,
) -> Result<Json<Game>, NotFound<Json<MessageResponse>>> {
    let game_svc = serv.game_service();
    let games = game_svc.lock();

    match games.get(game_id) {
        Some(g) => Ok(Json(g)),
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
                yield Event::json(&msg).event(msg.event);
            }
        }
    }
}

#[openapi(tag = "Games")]
#[get("/games/<game_id>/hit")]
pub async fn hit(game_id: u32, serv: &State<ServiceStore>) -> Result<NamedFile, CurrentHitError> {
    let hit = serv.game_service().lock().get_current_hit(game_id);

    if hit.is_ok() {
        return NamedFile::open(&hit.unwrap().file().unwrap())
            .await
            .or(Err(CurrentHitError {
                message: "hit file couldn't be found".into(),
                http_status_code: 404,
            }));
    }

    Err(hit.err().unwrap())
}

#[cfg(test)]
mod tests {
    use crate::{
        games::{Game, GameEvent, GameState},
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
        assert!(game.into_json::<Game>().await.is_some());
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
            .into_json::<Game>()
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
            game1.into_json::<Game>().await.unwrap().id,
            game2.into_json::<Game>().await.unwrap().id
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
                    super::join_game(game_id = game.into_json::<Game>().await.unwrap().id)
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
                    super::join_game(game_id = game.into_json::<Game>().await.unwrap().id)
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
                    super::leave_game(game_id = game.into_json::<Game>().await.unwrap().id)
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
            .into_json::<Game>()
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
                    super::leave_game(game_id = game.into_json::<Game>().await.unwrap().id)
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
            .into_json::<Game>()
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
    async fn can_stop_game() {
        let client = mocked_client().await;

        let cookies = create_test_users(&client, 2).await;

        let game_id = client
            .post(uri!("/api", super::create_game))
            .private_cookie(cookies.get(0).cloned().unwrap())
            .dispatch()
            .await
            .into_json::<Game>()
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
                .patch(uri!("/api", super::stop_game(game_id = game_id)))
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
            .into_json::<Game>()
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
    async fn only_creators_can_stop_games() {
        let client = mocked_client().await;

        let cookies = create_test_users(&client, 2).await;

        let game_id = client
            .post(uri!("/api", super::create_game))
            .private_cookie(cookies.get(0).cloned().unwrap())
            .dispatch()
            .await
            .into_json::<Game>()
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
                .patch(uri!("/api", super::stop_game(game_id = game_id)))
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
            .into_json::<Game>()
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
            .into_json::<Game>()
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
            .into_json::<Game>()
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

            assert_eq!(data.state, Some(GameState::Guessing));
            break;
        }
    }

    #[sqlx::test]
    async fn can_read_hit_after_starting_game() {
        let client = mocked_client().await;

        let cookies = create_test_users(&client, 2).await;

        let game_id = client
            .post(uri!("/api", super::create_game))
            .private_cookie(cookies.get(0).cloned().unwrap())
            .dispatch()
            .await
            .into_json::<Game>()
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
                .get(uri!("/api", super::hit(game_id = game_id)))
                .dispatch()
                .await
                .status(),
            Status::Ok
        );
    }
}
