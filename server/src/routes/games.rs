use crate::{
    games::{
        ConfirmationPayload, CreateGamePayload, Game, GameEvent, GameMode, GameSettingsPayload,
        GameState, SlotPayload,
    },
    responses::{
        ConfirmSlotError, CurrentHitError, GamesResponse, GuessSlotError, JoinGameError,
        LeaveGameError, MessageResponse, SkipHitError, StartGameError, StopGameError,
        UpdateGameError,
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
use std::{default::Default, path::PathBuf};
use uuid::Uuid;

/// Create a new game
///
/// Create a new game. The currently logged in user will be the creator of the game. The creator will be the only one who can change game properties later.
/// The API will return 401 if the user_id is invalid.

#[openapi(tag = "Games")]
#[post("/games", format = "json", data = "<data>")]
pub fn create_game(
    user: User,
    data: Option<Json<CreateGamePayload>>,
    serv: &State<ServiceStore>,
) -> Created<Json<Game>> {
    let game_svc = serv.game_service();
    let games = game_svc.lock();
    let mode = if let Some(data) = data.as_ref() {
        data.mode.unwrap_or(GameMode::Public)
    } else {
        GameMode::Public
    };

    let mut game = games.add(&user, mode);

    if data.is_some() && data.as_ref().unwrap().settings.is_some() {
        game = games
            .update(&game.id, &user, data.unwrap().settings.as_ref().unwrap())
            .ok()
            .unwrap_or(game);
    }

    Created::new(format!("/games/{}", game.id)).body(Json(game))
}

/// Retrieve all currently known games
///
/// Get a flat overview over all current games.
/// The info returned by this call is currently identical to GET /games/game_id or POST /games/user_id calls, but that might change later.

#[openapi(tag = "Games")]
#[get("/games")]
pub fn get_all_games(user: Option<User>, serv: &State<ServiceStore>) -> Json<GamesResponse> {
    Json(GamesResponse {
        games: serv.game_service().lock().get_all(user.as_ref()),
    })
}

#[openapi(tag = "Games")]
#[patch("/games/<game_id>/join/<player..>")]
pub async fn join_game(
    game_id: &str,
    player: PathBuf,
    user: User,
    serv: &State<ServiceStore>,
    queue: &State<Sender<GameEvent>>,
) -> Result<Json<MessageResponse>, JoinGameError> {
    let game_svc = serv.game_service();
    let games = game_svc.lock();

    games
        .join(
            game_id,
            &user,
            player
                .to_str()
                .and_then(|p| if !p.is_empty() { Some(p) } else { None }),
        )
        .map(|_| {
            let _ = queue.send(GameEvent {
                game_id: game_id.into(),
                event: "join".into(),
                players: Some(games.get(game_id, Some(&user)).unwrap().players),
                ..Default::default()
            });
            Json(MessageResponse {
                message: "joined the game successfully".into(),
                r#type: "success".into(),
            })
        })
}

#[openapi(tag = "Games")]
#[patch("/games/<game_id>/leave/<player_id..>")]
pub async fn leave_game(
    game_id: &str,
    player_id: PathBuf,
    user: User,
    serv: &State<ServiceStore>,
    queue: &State<Sender<GameEvent>>,
) -> Result<Json<MessageResponse>, LeaveGameError> {
    let game_svc = serv.game_service();
    let games = game_svc.lock();

    let state = games
        .get(game_id, Some(&user))
        .map(|g| g.state)
        .unwrap_or(GameState::Open);

    games
        .leave(
            game_id,
            &user,
            player_id.to_str().and_then(|p| Uuid::parse_str(p).ok()),
        )
        .map(|_| {
            let _ = queue.send(GameEvent {
                game_id: game_id.into(),
                event: "leave".into(),
                players: games.get(game_id, Some(&user)).map(|g| g.players),
                ..Default::default()
            });

            let new_state = games
                .get(game_id, Some(&user))
                .map(|g| g.state)
                .unwrap_or(GameState::Open);

            if new_state != state {
                let _ = queue.send(GameEvent {
                    game_id: game_id.into(),
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
    game_id: &str,
    user: User,
    serv: &State<ServiceStore>,
    queue: &State<Sender<GameEvent>>,
) -> Result<Json<MessageResponse>, StartGameError> {
    let game_svc = serv.game_service();
    let games = game_svc.lock();

    games.start(game_id, &user).map(|g| {
        let _ = queue.send(GameEvent {
            game_id: game_id.into(),
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
    game_id: &str,
    user: User,
    serv: &State<ServiceStore>,
    queue: &State<Sender<GameEvent>>,
) -> Result<Json<MessageResponse>, StopGameError> {
    serv.game_service()
        .lock()
        .stop(game_id, Some(&user))
        .map(|_| {
            let _ = queue.send(GameEvent {
                game_id: game_id.into(),
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
    game_id: &str,
    user: Option<User>,
    serv: &State<ServiceStore>,
) -> Result<Json<Game>, NotFound<Json<MessageResponse>>> {
    let game_svc = serv.game_service();
    let games = game_svc.lock();

    match games.get(game_id, user.as_ref()) {
        Some(g) => Ok(Json(g)),
        None => Err(NotFound(Json(MessageResponse {
            message: "game id not found".into(),
            r#type: "error".into(),
        }))),
    }
}

#[get("/games/<game_id>/events")]
pub async fn events(
    game_id: String,
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
pub async fn hit(game_id: &str, serv: &State<ServiceStore>) -> Result<NamedFile, CurrentHitError> {
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

#[openapi(tag = "Games")]
#[post("/games/<game_id>/guess", format = "json", data = "<slot>")]
pub fn guess_slot(
    game_id: &str,
    slot: Json<SlotPayload>,
    user: User,
    serv: &State<ServiceStore>,
    queue: &State<Sender<GameEvent>>,
) -> Result<Json<MessageResponse>, GuessSlotError> {
    let state = serv
        .game_service()
        .lock()
        .get(game_id, Some(&user))
        .map(|g| g.state)
        .unwrap_or(GameState::Guessing);
    serv.game_service()
        .lock()
        .guess(game_id, &user, slot.id)
        .map(|game| {
            let _ = queue.send(GameEvent {
                game_id: game_id.into(),
                event: "guess".into(),
                players: game
                    .players
                    .iter()
                    .find(|p| p.id == user.id)
                    .cloned()
                    .map(|p| vec![p]),
                ..Default::default()
            });

            if state != game.state {
                let _ = queue.send(GameEvent {
                    game_id: game_id.into(),
                    event: "change_state".into(),
                    state: Some(game.state),
                    players: Some(game.players),
                    hit: Some(game.state).and_then(|s| {
                        if s == GameState::Confirming {
                            game.hits_remaining.front().cloned()
                        } else {
                            None
                        }
                    }),
                    ..Default::default()
                });
            }

            Json(MessageResponse {
                message: "guess submitted successfully".into(),
                r#type: "success".into(),
            })
        })
}

#[openapi(tag = "Games")]
#[post("/games/<game_id>/confirm", format = "json", data = "<confirmation>")]
pub fn confirm_slot(
    game_id: &str,
    confirmation: Json<ConfirmationPayload>,
    user: User,
    serv: &State<ServiceStore>,
    queue: &State<Sender<GameEvent>>,
) -> Result<Json<MessageResponse>, ConfirmSlotError> {
    serv.game_service()
        .lock()
        .confirm(game_id, &user, confirmation.confirm)
        .map(|game| {
            let _ = queue.send(GameEvent {
                game_id: game_id.into(),
                event: "change_state".into(),
                state: Some(game.state),
                players: Some(game.players),
                ..Default::default()
            });

            Json(MessageResponse {
                message: "confirmation received".into(),
                r#type: "success".into(),
            })
        })
}

#[openapi(tag = "Games")]
#[post("/games/<game_id>/skip")]
pub fn skip_hit(
    game_id: &str,
    user: User,
    serv: &State<ServiceStore>,
    queue: &State<Sender<GameEvent>>,
) -> Result<Json<MessageResponse>, SkipHitError> {
    serv.game_service().lock().skip(game_id, &user).map(|game| {
        let _ = queue.send(GameEvent {
            game_id: game_id.into(),
            event: "skip".into(),
            players: game
                .players
                .iter()
                .find(|p| p.id == user.id)
                .cloned()
                .map(|p| vec![p]),
            ..Default::default()
        });

        Json(MessageResponse {
            message: "skipped successfully".into(),
            r#type: "success".into(),
        })
    })
}

#[openapi(tag = "Games")]
#[patch("/games/<game_id>/update", format = "json", data = "<settings>")]
pub fn update_game(
    game_id: &str,
    settings: Json<GameSettingsPayload>,
    user: User,
    serv: &State<ServiceStore>,
    queue: &State<Sender<GameEvent>>,
) -> Result<Json<MessageResponse>, UpdateGameError> {
    serv.game_service()
        .lock()
        .update(game_id, &user, &settings)
        .map(|_| {
            let _ = queue.send(GameEvent {
                game_id: game_id.into(),
                event: "update".into(),
                settings: Some(settings.into()),
                ..Default::default()
            });

            Json(MessageResponse {
                message: "game updated successfully".into(),
                r#type: "success".into(),
            })
        })
}

#[cfg(test)]
mod tests {
    use crate::{
        games::{Game, GameEvent, GameState, Slot},
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

    #[sqlx::test]
    async fn can_read_slots_after_starting_game() {
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

        let player = client
            .get(uri!("/api", super::get_game(game_id = game_id)))
            .dispatch()
            .await
            .into_json::<Game>()
            .await
            .unwrap()
            .players
            .into_iter()
            .find(|p| p.id == 1)
            .unwrap();

        assert_eq!(
            player.slots,
            vec![
                Slot {
                    from_year: 0,
                    to_year: player.hits.get(0).unwrap().year,
                    id: 1,
                },
                Slot {
                    from_year: player.hits.get(0).unwrap().year,
                    to_year: 0,
                    id: 2,
                }
            ]
        );
    }
}
