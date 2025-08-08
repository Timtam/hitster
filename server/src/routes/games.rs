use crate::{
    games::{
        ConfirmationPayload, CreateGamePayload, GameEvent, GameMode, GamePayload,
        GameSettingsPayload, GameState, SlotPayload,
    },
    responses::{
        ClaimHitError, ConfirmSlotError, GamesResponse, GetGameError, GuessSlotError, HitError,
        JoinGameError, LeaveGameError, MessageResponse, SkipHitError, StartGameError,
        StopGameError, UpdateGameError,
    },
    services::ServiceStore,
    users::User,
};
use rocket::{
    Shutdown, State,
    fs::NamedFile,
    response::{
        status::Created,
        stream::{Event, EventStream},
    },
    serde::json::Json,
    tokio::{
        select,
        sync::broadcast::{Sender, error::RecvError},
    },
};
use rocket_okapi::openapi;
use std::{default::Default, path::PathBuf};
use uuid::Uuid;

/// Create a new game
///
/// Create a new game. The currently logged in user will be the creator of the game. The creator will be the only one who can change game properties later.

#[openapi(tag = "Games")]
#[post("/games", format = "json", data = "<data>")]
pub fn create_game(
    user: User,
    data: Option<Json<CreateGamePayload>>,
    serv: &State<ServiceStore>,
) -> Created<Json<GamePayload>> {
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

    Created::new(format!("/games/{}", game.id)).body(Json((&game).into()))
}

/// Retrieve all currently known games
///
/// Get a flat overview over all current games.
/// The info returned by this call is currently identical to GET /games/game_id or POST /games/user_id calls, but that might change later.

#[openapi(tag = "Games")]
#[get("/games")]
pub fn get_all_games(user: Option<User>, serv: &State<ServiceStore>) -> Json<GamesResponse> {
    Json(GamesResponse {
        games: serv
            .game_service()
            .lock()
            .get_all(user.as_ref())
            .into_iter()
            .map(|g| (&g).into())
            .collect::<Vec<_>>(),
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
        .map(|p| {
            let _ = queue.send(GameEvent {
                game_id: game_id.into(),
                event: "join".into(),
                players: Some(vec![(&p).into()]),
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

    games
        .leave(
            game_id,
            &user,
            player_id.to_str().and_then(|p| Uuid::parse_str(p).ok()),
        )
        .map(|p| {
            let _ = queue.send(GameEvent {
                game_id: game_id.into(),
                event: "leave".into(),
                players: Some(vec![(&p).into()]),
                ..Default::default()
            });

            let new_state = games
                .get(game_id, Some(&user))
                .map(|g| g.state)
                .unwrap_or(GameState::Open);

            let _ = queue.send(GameEvent {
                game_id: game_id.into(),
                event: "change_state".into(),
                state: Some(new_state),
                players: games
                    .get(game_id, Some(&user))
                    .map(|g| g.players.iter().map(|p| p.into()).collect::<Vec<_>>()),
                ..Default::default()
            });

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
            players: Some(g.players.iter().map(|p| p.into()).collect::<Vec<_>>()),
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
        .map(|g| {
            let _ = queue.send(GameEvent {
                game_id: game_id.into(),
                event: "change_state".into(),
                state: Some(GameState::Open),
                players: Some(g.players.iter().map(|p| p.into()).collect::<Vec<_>>()),
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

#[openapi(tag = "Games")]
#[get("/games/<game_id>")]
pub fn get_game(
    game_id: &str,
    user: Option<User>,
    serv: &State<ServiceStore>,
) -> Result<Json<GamePayload>, GetGameError> {
    let game_svc = serv.game_service();
    let games = game_svc.lock();

    match games.get(game_id, user.as_ref()) {
        Some(g) => Ok(Json((&g).into())),
        None => Err(GetGameError {
            message: "game id not found".into(),
            http_status_code: 404,
        }),
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
#[get("/games/<game_id>/hit/<hit_id..>")]
pub async fn hit(
    game_id: &str,
    hit_id: PathBuf,
    serv: &State<ServiceStore>,
) -> Result<NamedFile, HitError> {
    let hit = serv.game_service().lock().get_hit(
        game_id,
        hit_id.to_str().and_then(|h| Uuid::parse_str(h).ok()),
    );

    if hit.is_ok() {
        return NamedFile::open(&hit.unwrap().file())
            .await
            .or(Err(HitError {
                message: "hit file couldn't be found".into(),
                http_status_code: 404,
            }));
    }

    Err(hit.err().unwrap())
}

#[openapi(tag = "Games")]
#[post(
    "/games/<game_id>/guess/<player_id..>",
    format = "json",
    data = "<slot>"
)]
pub fn guess_slot(
    game_id: &str,
    player_id: PathBuf,
    slot: Json<SlotPayload>,
    user: User,
    serv: &State<ServiceStore>,
    queue: &State<Sender<GameEvent>>,
) -> Result<Json<MessageResponse>, GuessSlotError> {
    let player_id = player_id.to_str().and_then(|p| Uuid::parse_str(p).ok());
    let state = serv
        .game_service()
        .lock()
        .get(game_id, Some(&user))
        .map(|g| g.state)
        .unwrap_or(GameState::Guessing);
    let game = serv
        .game_service()
        .lock()
        .guess(game_id, &user, slot.id, player_id);

    game.map(|mut game| {
        let _ = queue.send(GameEvent {
            game_id: game_id.into(),
            event: "guess".into(),
            players: game
                .players
                .iter()
                .find(|p| p.id == player_id.unwrap_or(user.id))
                .map(|p| vec![p.into()]),
            ..Default::default()
        });

        if state != game.state {
            let last_scored = game.last_scored.clone();
            let hit = game.hits_remaining.front().cloned();
            let winner = serv.game_service().lock().get_winner(&game);

            if winner.is_some() {
                game = serv.game_service().lock().stop(&game.id, None).unwrap();
            }

            let _ = queue.send(GameEvent {
                game_id: game_id.into(),
                event: "change_state".into(),
                state: Some(game.state),
                players: Some(game.players.iter().map(|p| p.into()).collect::<Vec<_>>()),
                hit: hit.and_then(|h| {
                    if game.state == GameState::Intercepting {
                        None
                    } else {
                        Some((&h).into())
                    }
                }),
                last_scored: last_scored.map(|p| (&p).into()),
                winner: winner.as_ref().map(|p| p.into()),
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
                players: Some(game.players.iter().map(|p| p.into()).collect::<Vec<_>>()),
                ..Default::default()
            });

            Json(MessageResponse {
                message: "confirmation received".into(),
                r#type: "success".into(),
            })
        })
}

#[openapi(tag = "Games")]
#[post("/games/<game_id>/skip/<player_id..>")]
pub fn skip_hit(
    game_id: &str,
    player_id: PathBuf,
    user: User,
    serv: &State<ServiceStore>,
    queue: &State<Sender<GameEvent>>,
) -> Result<Json<MessageResponse>, SkipHitError> {
    let player_id = player_id.to_str().and_then(|p| Uuid::parse_str(p).ok());
    serv.game_service()
        .lock()
        .skip(game_id, &user, player_id)
        .map(|(game, hit)| {
            let _ = queue.send(GameEvent {
                game_id: game_id.into(),
                event: "skip".into(),
                players: game
                    .players
                    .iter()
                    .find(|p| p.id == player_id.unwrap_or(user.id))
                    .map(|p| vec![p.into()]),
                hit: Some((&hit).into()),
                ..Default::default()
            });

            Json(MessageResponse {
                message: "skipped successfully".into(),
                r#type: "success".into(),
            })
        })
}

#[openapi(tag = "Games")]
#[post("/games/<game_id>/claim/<player_id..>")]
pub fn claim_hit(
    game_id: &str,
    player_id: PathBuf,
    user: User,
    serv: &State<ServiceStore>,
    queue: &State<Sender<GameEvent>>,
) -> Result<Json<MessageResponse>, ClaimHitError> {
    let player_id = player_id.to_str().and_then(|p| Uuid::parse_str(p).ok());
    let res = serv.game_service().lock().claim(game_id, &user, player_id);

    res.map(|(mut game, hit)| {
        let _ = queue.send(GameEvent {
            game_id: game_id.into(),
            event: "claim".into(),
            players: game
                .players
                .iter()
                .find(|p| p.id == player_id.unwrap_or(user.id))
                .map(|p| vec![p.into()]),
            hit: Some((&hit).into()),
            ..Default::default()
        });

        let winner = serv.game_service().lock().get_winner(&game);

        if winner.is_some() {
            game = serv.game_service().lock().stop(game_id, None).unwrap();

            let _ = queue.send(GameEvent {
                game_id: game_id.into(),
                event: "change_state".into(),
                state: Some(game.state),
                players: Some(game.players.iter().map(|p| p.into()).collect::<Vec<_>>()),
                winner: winner.as_ref().map(|p| p.into()),
                ..Default::default()
            });
        }

        Json(MessageResponse {
            message: "claimed hit successfully".into(),
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
