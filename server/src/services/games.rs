use crate::{
    games::{Game, GameMode, GameSettingsPayload, GameState, Player, PlayerState, Slot},
    responses::{
        ClaimHitError, ConfirmSlotError, GuessSlotError, HitError, JoinGameError, LeaveGameError,
        SkipHitError, StartGameError, StopGameError, UpdateGameError,
    },
    services::{HitService, ServiceHandle},
};
use hitster_core::{Hit, User};
use itertools::sorted;
use rand::{
    distr::{Alphanumeric, SampleString},
    prelude::SliceRandom,
    rng,
};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::Mutex,
};
use uuid::Uuid;

pub struct GameServiceData {
    games: HashMap<String, Game>,
}

pub struct GameService {
    data: Mutex<GameServiceData>,
    hit_service: ServiceHandle<HitService>,
}

impl GameService {
    pub fn new(hit_service: ServiceHandle<HitService>) -> Self {
        Self {
            hit_service,
            data: Mutex::new(GameServiceData {
                games: HashMap::new(),
            }),
        }
    }

    pub fn add(&self, creator: &User, mode: GameMode) -> Game {
        let mut data = self.data.lock().unwrap();
        let mut player: Player = creator.into();

        player.creator = true;

        let mut rng = rng();

        let id: String = loop {
            let id: String = Alphanumeric.sample_string(&mut rng, 8);

            if !data.games.contains_key(&id) {
                break id;
            }
        };

        let hs = self.hit_service.lock();

        let game = Game {
            id: id.clone(),
            players: vec![player],
            state: GameState::Open,
            hits_remaining: hs.copy_hits().into_iter().collect::<VecDeque<_>>(),
            hit_duration: 20,
            start_tokens: 2,
            goal: 10,
            hit: None,
            packs: hs.get_packs().into_iter().map(|p| p.id).collect::<Vec<_>>(),
            mode,
            remembered_hits: vec![],
            last_scored: None,
        };

        drop(hs);

        data.games.insert(id.clone(), game.clone());

        game
    }

    pub fn get_all(&self, user: Option<&User>) -> Vec<Game> {
        self.data
            .lock()
            .unwrap()
            .games
            .clone()
            .into_values()
            .filter(|g| {
                g.mode == GameMode::Public
                    || (user.is_some()
                        && (g.mode == GameMode::Private
                            && g.players.iter().any(|p| p.id == user.as_ref().unwrap().id)
                            || (g.mode == GameMode::Local
                                && g.players
                                    .iter()
                                    .any(|p| p.id == user.as_ref().unwrap().id && p.creator))))
            })
            .collect::<_>()
    }

    pub fn get(&self, id: &str, user: Option<&User>) -> Option<Game> {
        self.data
            .lock()
            .unwrap()
            .games
            .get(id)
            .cloned()
            .filter(|g| {
                g.mode != GameMode::Local
                    || (user.is_some()
                        && g.players
                            .iter()
                            .any(|p| p.id == user.as_ref().unwrap().id && p.creator))
            })
    }

    pub fn join(
        &self,
        game_id: &str,
        user: &User,
        player: Option<&str>,
    ) -> Result<Player, JoinGameError> {
        let mut data = self.data.lock().unwrap();

        if let Some(game) = data.games.get_mut(game_id) {
            if game.state != GameState::Open {
                Err(JoinGameError {
                    message: "the game is already running".into(),
                    http_status_code: 403,
                })
            } else if player.is_none() && game.players.iter().any(|p| p.id == user.id) {
                Err(JoinGameError {
                    message: "user is already part of this game".into(),
                    http_status_code: 409,
                })
            } else if player.is_some() && game.mode != GameMode::Local {
                Err(JoinGameError {
                    message: "virtual players can only be added to local games".into(),
                    http_status_code: 409,
                })
            } else if player.is_some()
                && game.mode == GameMode::Local
                && !game
                    .players
                    .iter()
                    .find(|p| p.id == user.id)
                    .map(|p| p.creator)
                    .unwrap_or(false)
            {
                Err(JoinGameError {
                    message: "only the creator can add virtual players to a game".into(),
                    http_status_code: 409,
                })
            } else {
                let plr: Player;

                if let Some(player) = player {
                    plr = Player {
                        id: Uuid::new_v4(),
                        name: player.into(),
                        ..Default::default()
                    };
                } else {
                    plr = user.into();
                }

                game.players.push(plr.clone());

                Ok(plr)
            }
        } else {
            Err(JoinGameError {
                message: "game not found".into(),
                http_status_code: 404,
            })
        }
    }

    pub fn leave(
        &self,
        game_id: &str,
        user: &User,
        player_id: Option<Uuid>,
    ) -> Result<Player, LeaveGameError> {
        let mut data = self.data.lock().unwrap();

        if let Some(game) = data.games.get_mut(game_id) {
            if !game.players.iter().any(|p| p.id == user.id) {
                Err(LeaveGameError {
                    message: "user is not part of this game".into(),
                    http_status_code: 409,
                })
            } else if player_id.is_some()
                && !game
                    .players
                    .iter()
                    .find(|p| p.id == user.id)
                    .map(|p| p.creator)
                    .unwrap_or(false)
            {
                Err(LeaveGameError {
                    message: "only the creator can kick other players from a game".into(),
                    http_status_code: 409,
                })
            } else if player_id.is_some()
                && !game
                    .players
                    .iter()
                    .any(|p| p.id == *player_id.as_ref().unwrap())
            {
                Err(LeaveGameError {
                    message: "a player with this id isn't part of this game".into(),
                    http_status_code: 409,
                })
            } else {
                let id = player_id.unwrap_or(user.id);
                let pos = game.players.iter().position(|p| p.id == id).unwrap();
                let creator = game.players.get(pos).unwrap().creator;
                let turn_player = game.players.get(pos).unwrap().turn_player;

                let idx = (pos + 1) % game.players.len();

                if creator {
                    game.players.get_mut(idx).unwrap().creator = true;
                }

                if turn_player {
                    game.players.get_mut(idx).unwrap().turn_player = true;
                }

                if game.state != GameState::Open {
                    for i in 0..game.players.len() {
                        game.players.get_mut(i).unwrap().guess = None;
                        if game.players.get(i).unwrap().turn_player {
                            game.players.get_mut(i).unwrap().state = PlayerState::Guessing;
                        } else {
                            game.players.get_mut(i).unwrap().state = PlayerState::Waiting;
                        }
                    }

                    game.state = GameState::Guessing;
                    game.hits_remaining.pop_front();
                }

                let plr = game.players.remove(pos);

                if game.players.iter().filter(|p| !p.r#virtual).count() == 0 {
                    data.games.remove(game_id);
                } else if game.players.len() == 1 && game.state != GameState::Open {
                    drop(data);
                    let _ = self.stop(game_id, None);
                }

                Ok(plr)
            }
        } else {
            Err(LeaveGameError {
                message: "game not found".into(),
                http_status_code: 404,
            })
        }
    }

    pub fn start(&self, game_id: &str, user: &User) -> Result<Game, StartGameError> {
        let mut data = self.data.lock().unwrap();

        if let Some(game) = data.games.get_mut(game_id) {
            if game.players.len() < 2 {
                Err(StartGameError {
                    message: "you need at least two players to start a game".into(),
                    http_status_code: 409,
                })
            } else if game.state != GameState::Open {
                Err(StartGameError {
                    http_status_code: 409,
                    message: "the game is already running".into(),
                })
            } else if !game
                .players
                .iter()
                .find(|p| p.id == user.id)
                .map(|p| p.creator)
                .unwrap_or(false)
            {
                Err(StartGameError {
                    http_status_code: 403,
                    message: "only the creator can start a game".into(),
                })
            } else {
                let mut rng = rng();
                let remembered_hits = game
                    .remembered_hits
                    .iter()
                    .filter(|h| h.packs.iter().any(|p| game.packs.contains(p)))
                    .cloned()
                    .collect::<Vec<_>>();
                let remembered_hits_count = remembered_hits.len();

                let mut hits_remaining: VecDeque<Hit> = self
                    .hit_service
                    .lock()
                    .get_hits_for_packs(&game.packs)
                    .into_iter()
                    .filter(|h| h.downloaded && !remembered_hits.contains(h))
                    .cloned()
                    .collect::<_>();

                if hits_remaining.len() + remembered_hits_count
                    < (game.players.len() * game.goal as usize * 2)
                {
                    return Err(StartGameError {
                        http_status_code: 409,
                        message: format!(
                            "There aren't enough hits available to start a game ({} individual hits in the currently selected packs, {} hits are required)",
                            hits_remaining.len() + remembered_hits_count,
                            game.players.len() * game.goal as usize * 2
                        ),
                    });
                }

                hits_remaining.make_contiguous().shuffle(&mut rng);

                game.hits_remaining = hits_remaining;

                game.remembered_hits = remembered_hits;

                game.state = GameState::Guessing;
                game.players.shuffle(&mut rng);
                game.players.get_mut(0).unwrap().state = PlayerState::Guessing;
                game.players.get_mut(0).unwrap().turn_player = true;

                for i in 0..game.players.len() {
                    let player = game.players.get_mut(i).unwrap();
                    let hit = game.hits_remaining.pop_front().unwrap();

                    player.hits.push(hit.clone());
                    game.remembered_hits.push(hit);
                    player.tokens = game.start_tokens;
                    player.slots = self.get_slots(&player.hits);
                }

                Ok(game.clone())
            }
        } else {
            Err(StartGameError {
                message: "game not found".into(),
                http_status_code: 404,
            })
        }
    }

    pub fn stop(&self, game_id: &str, user: Option<&User>) -> Result<Game, StopGameError> {
        let mut data = self.data.lock().unwrap();

        if let Some(game) = data.games.get_mut(game_id) {
            if let Some(u) = user
                && !game
                    .players
                    .iter()
                    .find(|p| p.id == u.id)
                    .map(|p| p.creator)
                    .unwrap_or(false)
            {
                return Err(StopGameError {
                    http_status_code: 403,
                    message: "you are not the creator of this game".into(),
                });
            }

            if game.state == GameState::Open {
                return Err(StopGameError {
                    http_status_code: 409,
                    message: "the game isn't running".into(),
                });
            }

            game.state = GameState::Open;
            game.last_scored = None;
            game.hits_remaining.clear();

            for p in game.players.iter_mut() {
                p.state = PlayerState::Waiting;
                p.tokens = 0;
                p.hits.clear();
                p.turn_player = false;
                p.guess = None;
            }

            Ok(game.clone())
        } else {
            Err(StopGameError {
                http_status_code: 404,
                message: "game with that id not found".into(),
            })
        }
    }

    pub fn get_hit(&self, game_id: &str, hit_id: Option<Uuid>) -> Result<Hit, HitError> {
        let mut data = self.data.lock().unwrap();

        if let Some(game) = data.games.get_mut(game_id) {
            if game.state == GameState::Open {
                Err(HitError {
                    message: "game currently isn't running".into(),
                    http_status_code: 409,
                })
            } else if let Some(hit_id) = hit_id {
                game.players
                    .iter()
                    .flat_map(|p| &p.hits)
                    .find(|h| h.id == hit_id)
                    .cloned()
                    .ok_or(HitError {
                        message: "the hit isn't currently revealed".into(),
                        http_status_code: 404,
                    })
            } else {
                game.hits_remaining.front().cloned().ok_or(HitError {
                    message: "no hit found".into(),
                    http_status_code: 500,
                })
            }
        } else {
            Err(HitError {
                message: "game not found".into(),
                http_status_code: 404,
            })
        }
    }

    pub fn get_slots(&self, hits: &[Hit]) -> Vec<Slot> {
        let mut slots = vec![];
        let years = sorted(hits.iter().map(|h| h.year).collect::<HashSet<_>>()).collect::<Vec<_>>();

        for i in 0..years.len() {
            if i == 0 {
                slots.push(Slot {
                    from_year: 0,
                    to_year: *years.get(i).unwrap(),
                    id: (slots.len() + 1) as u8,
                });
            }
            if i < years.len() - 1 {
                slots.push(Slot {
                    from_year: *years.get(i).unwrap(),
                    to_year: *years.get(i + 1).unwrap(),
                    id: (slots.len() + 1) as u8,
                });
            }
            if i == years.len() - 1 {
                slots.push(Slot {
                    from_year: *years.get(i).unwrap(),
                    to_year: 0,
                    id: (slots.len() + 1) as u8,
                });
            }
        }

        slots
    }

    pub fn guess(
        &self,
        game_id: &str,
        user: &User,
        slot_id: Option<u8>,
        player_id: Option<Uuid>,
    ) -> Result<Game, GuessSlotError> {
        let mut data = self.data.lock().unwrap();

        if let Some(game) = data.games.get_mut(game_id) {
            if !game.players.iter().any(|p| p.id == user.id) {
                return Err(GuessSlotError {
                    message: "user is not part of this game".into(),
                    http_status_code: 409,
                });
            } else if player_id.is_some() && game.mode != GameMode::Local {
                return Err(GuessSlotError {
                    message: "guesses for other players can only be submitted in local games"
                        .into(),
                    http_status_code: 409,
                });
            } else if player_id.is_some()
                && game.mode == GameMode::Local
                && !game
                    .players
                    .iter()
                    .find(|p| p.id == user.id)
                    .map(|p| p.creator)
                    .unwrap_or(false)
            {
                return Err(GuessSlotError {
                    message: "only the creator can submit guesses for virtual players".into(),
                    http_status_code: 409,
                });
            }

            let player_id = player_id.unwrap_or(user.id);
            let turn_player_pos = game.players.iter().position(|p| p.turn_player).unwrap();
            let pos = game.players.iter().position(|p| p.id == player_id).unwrap();

            if game.players.get(pos).unwrap().state != PlayerState::Guessing
                && game.players.get(pos).unwrap().state != PlayerState::Intercepting
            {
                return Err(GuessSlotError {
                    message: "this player cannot guess right now".into(),
                    http_status_code: 403,
                });
            } else if slot_id.is_some()
                && game.players.iter().any(|p| {
                    p.guess
                        .as_ref()
                        .map(|s| s.id == slot_id.unwrap())
                        .unwrap_or(false)
                })
            {
                return Err(GuessSlotError {
                    message: "this slot is already taken".into(),
                    http_status_code: 409,
                });
            } else if slot_id.is_some() && game.players.get(pos).unwrap().guess.is_some() {
                return Err(GuessSlotError {
                    message: "you already submitted a guess".into(),
                    http_status_code: 409,
                });
            } else if game.state == GameState::Intercepting
                && game.players.get(pos).unwrap().tokens == 0
            {
                return Err(GuessSlotError {
                    message: "this player doesn't have a token to intercept here".into(),
                    http_status_code: 403,
                });
            } else if slot_id.is_some()
                && !game
                    .players
                    .get(turn_player_pos)
                    .unwrap()
                    .slots
                    .iter()
                    .any(|s| s.id == slot_id.unwrap())
            {
                return Err(GuessSlotError {
                    message: "a guess with that id doesn't exist for the current turn player"
                        .into(),
                    http_status_code: 409,
                });
            } else if game.state == GameState::Guessing && slot_id.is_none() {
                return Err(GuessSlotError {
                    message: "this player needs to send a guess".into(),
                    http_status_code: 409,
                });
            }

            if let Some(slot) = slot_id {
                game.players.get_mut(pos).unwrap().guess = game
                    .players
                    .get(turn_player_pos)
                    .unwrap()
                    .slots
                    .iter()
                    .find(|s| s.id == slot)
                    .cloned();
            }
            game.players.get_mut(pos).unwrap().state = PlayerState::Waiting;

            if game.state == GameState::Guessing {
                game.state = GameState::Intercepting;

                for i in 0..game.players.len() {
                    if i != turn_player_pos && game.players.get(i).unwrap().tokens > 0 {
                        game.players.get_mut(i).unwrap().state = PlayerState::Intercepting;
                    }
                }
            }

            if game.state == GameState::Intercepting {
                if pos != turn_player_pos && slot_id.is_some() {
                    game.players.get_mut(pos).unwrap().tokens -= 1;
                }

                if !game
                    .players
                    .iter()
                    .any(|p| p.state == PlayerState::Intercepting)
                {
                    let len = game.players.len();

                    let winners = game
                        .players
                        .iter()
                        .enumerate()
                        .filter(|(_, p)| {
                            p.guess
                                .as_ref()
                                .map(|s| {
                                    (s.from_year == 0
                                        && game.hits_remaining.front().unwrap().year <= s.to_year)
                                        || (s.to_year == 0
                                            && game.hits_remaining.front().unwrap().year
                                                >= s.from_year)
                                        || (s.from_year
                                            <= game.hits_remaining.front().unwrap().year
                                            && game.hits_remaining.front().unwrap().year
                                                <= s.to_year)
                                })
                                .unwrap_or(false)
                        })
                        .collect::<Vec<_>>();

                    if let Some(i) = winners.iter().find(|(_, p)| p.turn_player).map(|(i, _)| *i) {
                        let player = game.players.get_mut(i).unwrap();

                        player
                            .hits
                            .push(game.hits_remaining.front().cloned().unwrap());
                        player.slots = self.get_slots(&player.hits);
                        game.last_scored = Some(player.clone());
                    } else if winners.len() == 1 {
                        let i = winners.first().map(|(i, _)| *i).unwrap();
                        let player = game.players.get_mut(i).unwrap();

                        player
                            .hits
                            .push(game.hits_remaining.front().cloned().unwrap());
                        player.slots = self.get_slots(&player.hits);
                        game.last_scored = Some(player.clone());
                    }

                    game.remembered_hits
                        .push(game.hits_remaining.front().cloned().unwrap());

                    game.state = GameState::Confirming;
                    game.hit = game.hits_remaining.front().cloned();
                    if game.mode == GameMode::Local {
                        let creator_pos = game.players.iter().position(|p| p.creator).unwrap();
                        game.players.get_mut(creator_pos).unwrap().state = PlayerState::Confirming;
                    } else {
                        game.players
                            .get_mut((turn_player_pos + 1) % len)
                            .unwrap()
                            .state = PlayerState::Confirming;
                    }
                }
            }

            Ok(game.clone())
        } else {
            Err(GuessSlotError {
                message: "game not found".into(),
                http_status_code: 404,
            })
        }
    }

    pub fn confirm(
        &self,
        game_id: &str,
        user: &User,
        confirm: bool,
    ) -> Result<Game, ConfirmSlotError> {
        let mut data = self.data.lock().unwrap();

        if let Some(game) = data.games.get_mut(game_id) {
            if !game.players.iter().any(|p| p.id == user.id) {
                return Err(ConfirmSlotError {
                    message: "user is not part of this game".into(),
                    http_status_code: 409,
                });
            } else if self.get_winner(game).is_some() {
                return Err(ConfirmSlotError {
                    message: "the game already has a winner".into(),
                    http_status_code: 409,
                });
            }

            let turn_player_pos = game.players.iter().position(|p| p.turn_player).unwrap();
            let pos = game.players.iter().position(|p| p.id == user.id).unwrap();

            if game.players.get(pos).unwrap().state != PlayerState::Confirming {
                return Err(ConfirmSlotError {
                    message: "this player cannot confirm right now".into(),
                    http_status_code: 403,
                });
            }

            if confirm {
                game.players.get_mut(turn_player_pos).unwrap().tokens += 1;
            }

            game.hits_remaining.pop_front().unwrap();

            for p in game.players.iter_mut() {
                p.guess = None;
                p.state = PlayerState::Waiting;
            }

            game.state = GameState::Guessing;
            game.last_scored = None;
            game.players.get_mut(turn_player_pos).unwrap().turn_player = false;

            let len = game.players.len();

            if let Some(p) = game.players.get_mut((turn_player_pos + 1) % len) {
                p.turn_player = true;
                p.state = PlayerState::Guessing;
            }

            if game.hits_remaining.is_empty() {
                let mut rng = rng();
                game.hits_remaining = game
                    .remembered_hits
                    .iter()
                    .filter(|h| !game.players.iter().any(|p| p.hits.contains(h)))
                    .cloned()
                    .collect::<VecDeque<_>>();
                game.hits_remaining.make_contiguous().shuffle(&mut rng);
                game.remembered_hits = game
                    .players
                    .iter()
                    .flat_map(|p| &p.hits)
                    .cloned()
                    .collect::<Vec<_>>();
            }

            Ok(game.clone())
        } else {
            Err(ConfirmSlotError {
                message: "game not found".into(),
                http_status_code: 404,
            })
        }
    }

    pub fn skip(
        &self,
        game_id: &str,
        user: &User,
        player_id: Option<Uuid>,
    ) -> Result<(Game, Hit), SkipHitError> {
        let mut data = self.data.lock().unwrap();

        if let Some(game) = data.games.get_mut(game_id) {
            if !game.players.iter().any(|p| p.id == user.id) {
                return Err(SkipHitError {
                    message: "user is not part of this game".into(),
                    http_status_code: 409,
                });
            } else if player_id.is_some() && game.mode != GameMode::Local {
                return Err(SkipHitError {
                    message: "hits of other players can only be skipped in local games".into(),
                    http_status_code: 409,
                });
            } else if player_id.is_some()
                && game.mode == GameMode::Local
                && !game
                    .players
                    .iter()
                    .find(|p| p.id == user.id)
                    .map(|p| p.creator)
                    .unwrap_or(false)
            {
                return Err(SkipHitError {
                    message: "only the creator can skip hits of virtual players".into(),
                    http_status_code: 409,
                });
            }

            let player_id = player_id.unwrap_or(user.id);
            let pos = game.players.iter().position(|p| p.id == player_id).unwrap();

            if game.players.get(pos).unwrap().state != PlayerState::Guessing {
                return Err(SkipHitError {
                    message: "this player cannot skip right now".into(),
                    http_status_code: 403,
                });
            } else if game.players.get(pos).unwrap().tokens == 0 {
                return Err(SkipHitError {
                    message: "this player doesn't have a token to skip here".into(),
                    http_status_code: 403,
                });
            }

            game.players.get_mut(pos).unwrap().tokens -= 1;

            let hit = game.hits_remaining.pop_front().unwrap();

            game.remembered_hits.push(hit.clone());

            if game.hits_remaining.is_empty() {
                let mut rng = rng();
                game.hits_remaining = game
                    .remembered_hits
                    .iter()
                    .filter(|h| !game.players.iter().any(|p| p.hits.contains(h)))
                    .cloned()
                    .collect::<VecDeque<_>>();
                game.hits_remaining.make_contiguous().shuffle(&mut rng);
                game.remembered_hits = game
                    .players
                    .iter()
                    .flat_map(|p| &p.hits)
                    .cloned()
                    .collect::<Vec<_>>();
            }

            Ok((game.clone(), hit))
        } else {
            Err(SkipHitError {
                message: "game not found".into(),
                http_status_code: 404,
            })
        }
    }

    pub fn claim(
        &self,
        game_id: &str,
        user: &User,
        player_id: Option<Uuid>,
    ) -> Result<(Game, Hit), ClaimHitError> {
        let mut data = self.data.lock().unwrap();

        if let Some(game) = data.games.get_mut(game_id) {
            if !game.players.iter().any(|p| p.id == user.id) {
                return Err(ClaimHitError {
                    message: "user is not part of this game".into(),
                    http_status_code: 409,
                });
            } else if player_id.is_some() && game.mode != GameMode::Local {
                return Err(ClaimHitError {
                    message: "hits of other players can only be claimed in local games".into(),
                    http_status_code: 409,
                });
            } else if player_id.is_some()
                && game.mode == GameMode::Local
                && !game
                    .players
                    .iter()
                    .find(|p| p.id == user.id)
                    .map(|p| p.creator)
                    .unwrap_or(false)
            {
                return Err(ClaimHitError {
                    message: "only the creator can claim hits of virtual players".into(),
                    http_status_code: 409,
                });
            }

            let player_id = player_id.unwrap_or(user.id);
            let pos = game.players.iter().position(|p| p.id == player_id).unwrap();

            if game.players.get(pos).unwrap().tokens < 3 {
                return Err(ClaimHitError {
                    message: "this player doesn't have enough tokens to claim a hit".into(),
                    http_status_code: 403,
                });
            }

            game.players.get_mut(pos).unwrap().tokens -= 3;

            if game.hits_remaining.len() == 1 {
                let current_hit = game.hits_remaining.pop_front().unwrap();
                let mut rng = rng();
                game.hits_remaining = game
                    .remembered_hits
                    .iter()
                    .filter(|h| !game.players.iter().any(|p| p.hits.contains(h)))
                    .cloned()
                    .collect::<VecDeque<_>>();
                game.hits_remaining.make_contiguous().shuffle(&mut rng);
                game.remembered_hits = game
                    .players
                    .iter()
                    .flat_map(|p| &p.hits)
                    .cloned()
                    .collect::<Vec<_>>();
                game.hits_remaining.push_front(current_hit);
            }

            let hit = game.hits_remaining.remove(1).unwrap();

            let player = game.players.get_mut(pos).unwrap();
            player.hits.push(hit.clone());
            player.slots = self.get_slots(&player.hits);
            game.remembered_hits.push(hit.clone());

            Ok((game.clone(), hit))
        } else {
            Err(ClaimHitError {
                message: "game not found".into(),
                http_status_code: 404,
            })
        }
    }

    pub fn update(
        &self,
        game_id: &str,
        user: &User,
        settings: &GameSettingsPayload,
    ) -> Result<Game, UpdateGameError> {
        let mut data = self.data.lock().unwrap();

        if let Some(game) = data.games.get_mut(game_id) {
            if !game.players.iter().any(|p| p.id == user.id) {
                return Err(UpdateGameError {
                    message: "user is not part of this game".into(),
                    http_status_code: 409,
                });
            } else if game.state != GameState::Open {
                return Err(UpdateGameError {
                    message: "game is currently running".into(),
                    http_status_code: 403,
                });
            } else if !game
                .players
                .iter()
                .find(|p| p.id == user.id)
                .unwrap()
                .creator
            {
                return Err(UpdateGameError {
                    message: "user must be creator of the game".into(),
                    http_status_code: 409,
                });
            }

            game.packs = if let Some(packs) = &settings.packs {
                if packs.is_empty() {
                    self.hit_service
                        .lock()
                        .get_packs()
                        .into_iter()
                        .map(|p| p.id)
                        .collect::<Vec<_>>()
                } else {
                    packs.clone()
                }
            } else {
                game.packs.clone()
            };

            game.start_tokens = settings.start_tokens.unwrap_or(game.start_tokens);
            game.goal = settings.goal.unwrap_or(game.goal);
            game.hit_duration = settings.hit_duration.unwrap_or(game.hit_duration);

            Ok(game.clone())
        } else {
            Err(UpdateGameError {
                message: "game not found".into(),
                http_status_code: 404,
            })
        }
    }

    pub fn get_winner(&self, game: &Game) -> Option<Player> {
        game.players
            .iter()
            .find(|p| p.hits.len() >= game.goal as usize)
            .cloned()
    }
}
