use crate::{
    GlobalEvent,
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
    users::UserAuthenticator,
};
use rocket::{
    Shutdown, State,
    fs::NamedFile,
    futures::stream::Stream,
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

/// # Create a new game
///
/// Create a new game. The currently logged in user will be the creator of the game. The creator will be the only one who can change game properties later.
/// You can create the game with pre-defined settings via the data parameter. If supplied, it must be of type `CreateGamePayload`.

#[openapi(tag = "Games")]
#[post("/games", format = "json", data = "<data>")]
pub fn create_game(
    data: Option<Json<CreateGamePayload>>,
    user: UserAuthenticator,
    serv: &State<ServiceStore>,
    queue: &State<Sender<GlobalEvent>>,
) -> Created<Json<GamePayload>> {
    let game_svc = serv.game_service();
    let games = game_svc.lock();
    let mode = if let Some(data) = data.as_ref() {
        data.mode.unwrap_or(GameMode::Public)
    } else {
        GameMode::Public
    };

    let mut game = games.add(&user.0, mode);

    if let Some(data) = data.as_ref()
        && let Some(settings) = data.settings.as_ref()
    {
        game = games
            .update(&game.id, &user.0, settings)
            .ok()
            .unwrap_or(game);
    }

    if mode == GameMode::Public {
        let _ = queue.send(GlobalEvent::CreateGame((&game).into()));
    }

    Created::new(format!("/games/{}", game.id)).body(Json((&game).into()))
}

/// # Retrieve all currently known games
///
/// Get a flat overview over all current games.
/// The info returned by this call is currently identical to GET /games/game_id or POST /games/user_id calls, but that might change later.
/// The games returned by this call depend on the scope of the user cookie:
///
///   * If no user or an invalid user is provided, only public games are fetched
///
///   * If a valid user is provided, only public games and games created by this user are returned

#[openapi(tag = "Games")]
#[get("/games")]
pub fn get_all_games(
    user: Option<UserAuthenticator>,
    serv: &State<ServiceStore>,
) -> Json<GamesResponse> {
    Json(GamesResponse {
        games: serv
            .game_service()
            .lock()
            .get_all(user.map(|u| u.0).as_ref())
            .into_iter()
            .map(|g| (&g).into())
            .collect::<Vec<_>>(),
    })
}

/// # Join a game
///
/// Any authenticated user can join a game.
/// Local games will usually be joined automatically and cannot be joined by users other than the creator. The player parameter however can be used to create virtual players who join the local game instead.
/// Private games can be joined if the id is known, e.g. by sharing the game link.
/// Public games can be joined by everyone.

#[openapi(tag = "Games")]
#[patch("/games/<game_id>/join/<player..>")]
pub async fn join_game(
    game_id: &str,
    player: PathBuf,
    user: UserAuthenticator,
    serv: &State<ServiceStore>,
    queue: &State<Sender<GameEvent>>,
) -> Result<Json<MessageResponse>, JoinGameError> {
    let game_svc = serv.game_service();
    let games = game_svc.lock();

    games
        .join(
            game_id,
            &user.0,
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

/// # (forcefully) leave a game
///
/// If called without any additional parameter, the authenticated user will leave the provided game.
/// If the authenticated user is the creator of said game, you can provide a player id to kick them from the game instead.

#[openapi(tag = "Games")]
#[patch("/games/<game_id>/leave/<player_id..>")]
pub async fn leave_game(
    game_id: &str,
    player_id: PathBuf,
    user: UserAuthenticator,
    serv: &State<ServiceStore>,
    game_event_queue: &State<Sender<GameEvent>>,
    global_event_queue: &State<Sender<GlobalEvent>>,
) -> Result<Json<MessageResponse>, LeaveGameError> {
    let game_svc = serv.game_service();
    let games = game_svc.lock();
    let old_mode = games
        .get(game_id, Some(&user.0))
        .map(|g| g.mode)
        .unwrap_or(GameMode::Public);

    games
        .leave(
            game_id,
            &user.0,
            player_id.to_str().and_then(|p| Uuid::parse_str(p).ok()),
        )
        .map(|p| {
            let _ = game_event_queue.send(GameEvent {
                game_id: game_id.into(),
                event: "leave".into(),
                players: Some(vec![(&p).into()]),
                ..Default::default()
            });

            let new_state = games
                .get(game_id, Some(&user.0))
                .map(|g| g.state)
                .unwrap_or_else(|| {
                    if old_mode == GameMode::Public {
                        let _ =
                            global_event_queue.send(GlobalEvent::RemoveGame(game_id.to_string()));
                    }

                    GameState::Open
                });

            let _ = game_event_queue.send(GameEvent {
                game_id: game_id.into(),
                event: "change_state".into(),
                state: Some(new_state),
                players: games
                    .get(game_id, Some(&user.0))
                    .map(|g| g.players.iter().map(|p| p.into()).collect::<Vec<_>>()),
                ..Default::default()
            });

            Json(MessageResponse {
                message: "left the game successfully".into(),
                r#type: "success".into(),
            })
        })
}

/// # Start a game
///
/// Only the creator of a game can start it.

#[openapi(tag = "Games")]
#[patch("/games/<game_id>/start")]
pub async fn start_game(
    game_id: &str,
    user: UserAuthenticator,
    serv: &State<ServiceStore>,
    queue: &State<Sender<GameEvent>>,
) -> Result<Json<MessageResponse>, StartGameError> {
    let game_svc = serv.game_service();
    let games = game_svc.lock();

    games.start(game_id, &user.0).map(|g| {
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

/// # Stop a game
///
/// Only the creator of a game can stop it.

#[openapi(tag = "Games")]
#[patch("/games/<game_id>/stop")]
pub async fn stop_game(
    game_id: &str,
    user: UserAuthenticator,
    serv: &State<ServiceStore>,
    queue: &State<Sender<GameEvent>>,
) -> Result<Json<MessageResponse>, StopGameError> {
    serv.game_service()
        .lock()
        .stop(game_id, Some(&user.0))
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

/// # Get all info about a certain game
///
/// Retrieve all known info about a specific game.
/// The info here is currently identical with what you get with GET /games, but that might change later.

#[openapi(tag = "Games")]
#[get("/games/<game_id>")]
pub fn get_game(
    game_id: &str,
    user: Option<UserAuthenticator>,
    serv: &State<ServiceStore>,
) -> Result<Json<GamePayload>, GetGameError> {
    let game_svc = serv.game_service();
    let games = game_svc.lock();

    match games.get(game_id, user.map(|u| u.0).as_ref()) {
        Some(g) => Ok(Json((&g).into())),
        None => Err(GetGameError {
            message: "game id not found".into(),
            http_status_code: 404,
        }),
    }
}

/// # Subscribe to game events
///
/// All events that affect the game will be distributed via this event stream (Server-Side Events) in real-time.
/// The following table lists all the possible payloads that are provided as JSON:
///
/// <table>
///   <thead>
///     <th>Event Name</th>
///     <th>Key</th>
///     <th>Description</th>
///   </thead>
///   <tbody>
///     <tr>
///       <th rowSpan="6">change_state</th>
///     </tr>
///     <tr>
///       <td>hit</td>
///       <td>this field is set if the hit is flipped up, mostly when entering GameState.Confirming state</td>
///     </tr>
///     <tr>
///       <td>last_scored</td>
///       <td>this field is set when someone wins a hit, mostly when entering GameState.Confirming state</td>
///     </tr>
///     <tr>
///       <td>players</td>
///       <td>Array of player objects in the game</td>
///     </tr>
///     <tr>
///       <td>state</td>
///       <td>a state as specified within the GameState enum</td>
///     </tr>
///     <tr>
///       <td>winner</td>
///       <td>this field is set when a player wins the game, e.g. after someone claims a hit or guesses correctly</td>
///     </tr>
///     <tr>
///       <th rowSpan="3">claim</th>
///     </tr>
///     <tr>
///       <td>hit</td>
///       <td>the HitPayload that was just claimed</td>
///     </tr>
///     <tr>
///       <td>players</td>
///       <td>Array of players who claimed a hit</td>
///     </tr>
///     <tr>
///       <td>guess</td>
///       <td>players</td>
///       <td>Array of players who last updated their guess</td>
///     </tr>
///     <tr>
///       <td>join</td>
///       <td>players</td>
///       <td>Array of player objects who joined the game</td>
///     </tr>
///     <tr>
///       <td>leave</td>
///       <td>players</td>
///       <td>Array of player objects who left the game</td>
///     </tr>
///     <tr>
///       <th rowSpan="3">skip</th>
///     </tr>
///     <tr>
///       <td>hit</td>
///       <td>the HitPayload that was just skipped</td>
///     </tr>
///     <tr>
///       <td>players</td>
///       <td>Array of players who skipped a hit</td>
///     </tr>
///     <tr>
///       <td>update</td>
///       <td>settings</td>
///       <td>GameSettingsPayload with the updated game settings</td>
///     </tr>
///   </tbody>
/// </table>

#[openapi(tag = "Games")]
#[get("/games/<game_id>/events")]
pub async fn events(
    game_id: String,
    queue: &State<Sender<GameEvent>>,
    mut end: Shutdown,
) -> EventStream<impl Stream<Item = Event>> {
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

/// # Get the audio file for a revealed hit
///
/// Retrieve the audio file for a specific revealed hit in a game.
/// The audio files are trimmed (see FullHitPayload.playback_offset) and come in MP3 format.
/// If no hit_id is specified, the last revealed hit will be fetched.
/// You can provide any hit_id of a hit that is currently in a player's possession to fetch that one instead.

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

    if let Ok(hit) = hit {
        return NamedFile::open(&hit.file()).await.or(Err(HitError {
            message: "hit file couldn't be found".into(),
            http_status_code: 404,
        }));
    }

    Err(hit.err().unwrap())
}

/// # Guess a slot
///
/// When in GameState.Guessing or GameState.Intercepting, guess a slot that the player wants to place their bet in.
/// If its not the player's turn, you can leave the slot's id empty to don't step in.
/// By default the authenticated user will guess their slot.
/// When in a local game, the creator can guess for any virtual player by specifying the player id here.

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
    user: UserAuthenticator,
    serv: &State<ServiceStore>,
    queue: &State<Sender<GameEvent>>,
) -> Result<Json<MessageResponse>, GuessSlotError> {
    let player_id = player_id.to_str().and_then(|p| Uuid::parse_str(p).ok());
    let state = serv
        .game_service()
        .lock()
        .get(game_id, Some(&user.0))
        .map(|g| g.state)
        .unwrap_or(GameState::Guessing);
    let game = serv
        .game_service()
        .lock()
        .guess(game_id, &user.0, slot.id, player_id);

    game.map(|mut game| {
        let _ = queue.send(GameEvent {
            game_id: game_id.into(),
            event: "guess".into(),
            players: game
                .players
                .iter()
                .find(|p| p.id == player_id.unwrap_or(user.0.id))
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

/// # Confirm a guess
///
/// After guessing a song, confirm wether the guessing player needs to get a token for their guess.

#[openapi(tag = "Games")]
#[post("/games/<game_id>/confirm", format = "json", data = "<confirmation>")]
pub fn confirm_slot(
    game_id: &str,
    confirmation: Json<ConfirmationPayload>,
    user: UserAuthenticator,
    serv: &State<ServiceStore>,
    queue: &State<Sender<GameEvent>>,
) -> Result<Json<MessageResponse>, ConfirmSlotError> {
    serv.game_service()
        .lock()
        .confirm(game_id, &user.0, confirmation.confirm)
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

/// # Skip a hit
///
/// Skip the current hit for the authenticated user.
/// When in a local game, the creator can skip a hit for any virtual player by specifying the player id.

#[openapi(tag = "Games")]
#[post("/games/<game_id>/skip/<player_id..>")]
pub fn skip_hit(
    game_id: &str,
    player_id: PathBuf,
    user: UserAuthenticator,
    serv: &State<ServiceStore>,
    queue: &State<Sender<GameEvent>>,
) -> Result<Json<MessageResponse>, SkipHitError> {
    let player_id = player_id.to_str().and_then(|p| Uuid::parse_str(p).ok());
    serv.game_service()
        .lock()
        .skip(game_id, &user.0, player_id)
        .map(|(game, hit)| {
            let _ = queue.send(GameEvent {
                game_id: game_id.into(),
                event: "skip".into(),
                players: game
                    .players
                    .iter()
                    .find(|p| p.id == player_id.unwrap_or(user.0.id))
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

/// # Claim a hit
///
/// Claim a hit for the authenticated user.
/// When in a local game, the crator can claim a hit for a virtual player by specifying the player id.

#[openapi(tag = "Games")]
#[post("/games/<game_id>/claim/<player_id..>")]
pub fn claim_hit(
    game_id: &str,
    player_id: PathBuf,
    user: UserAuthenticator,
    serv: &State<ServiceStore>,
    queue: &State<Sender<GameEvent>>,
) -> Result<Json<MessageResponse>, ClaimHitError> {
    let player_id = player_id.to_str().and_then(|p| Uuid::parse_str(p).ok());
    let res = serv
        .game_service()
        .lock()
        .claim(game_id, &user.0, player_id);

    res.map(|(mut game, hit)| {
        let _ = queue.send(GameEvent {
            game_id: game_id.into(),
            event: "claim".into(),
            players: game
                .players
                .iter()
                .find(|p| p.id == player_id.unwrap_or(user.0.id))
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

/// # Update a game
///
/// A game's settings can be updated by the creator while it isn't running.

#[openapi(tag = "Games")]
#[patch("/games/<game_id>/update", format = "json", data = "<settings>")]
pub fn update_game(
    game_id: &str,
    settings: Json<GameSettingsPayload>,
    user: UserAuthenticator,
    serv: &State<ServiceStore>,
    queue: &State<Sender<GameEvent>>,
) -> Result<Json<MessageResponse>, UpdateGameError> {
    serv.game_service()
        .lock()
        .update(game_id, &user.0, &settings)
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
