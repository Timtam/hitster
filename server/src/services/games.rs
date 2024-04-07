use crate::{
    games::{Game, GameState, Player, PlayerState, Slot},
    hits::Hit,
    responses::{
        ConfirmSlotError, CurrentHitError, GuessSlotError, JoinGameError, LeaveGameError,
        SkipHitError, StartGameError, StopGameError,
    },
    services::{HitService, ServiceHandle},
    users::User,
};
use itertools::sorted;
use rand::prelude::{thread_rng, SliceRandom};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::Mutex,
};

pub struct GameServiceData {
    games: HashMap<u32, Game>,
    id: u32,
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
                id: 0,
                games: HashMap::new(),
            }),
        }
    }

    pub fn add(&self, creator: &User) -> Game {
        let mut data = self.data.lock().unwrap();
        data.id += 1;
        let mut player: Player = creator.into();

        player.creator = true;

        let game = Game {
            id: data.id,
            players: vec![player],
            state: GameState::Open,
            hits_remaining: VecDeque::new(),
            hit_duration: 20,
            start_tokens: 2,
            goal: 10,
            hit: None,
        };

        data.games.insert(game.id, game.clone());

        game
    }

    pub fn get_all(&self) -> Vec<Game> {
        self.data
            .lock()
            .unwrap()
            .games
            .clone()
            .into_values()
            .collect::<_>()
    }

    pub fn get(&self, id: u32) -> Option<Game> {
        self.data.lock().unwrap().games.get(&id).cloned()
    }

    pub fn join(&self, game_id: u32, user: &User) -> Result<(), JoinGameError> {
        let mut data = self.data.lock().unwrap();

        if let Some(game) = data.games.get_mut(&game_id) {
            if game.state != GameState::Open {
                Err(JoinGameError {
                    message: "the game is already running".into(),
                    http_status_code: 403,
                })
            } else if game.players.iter().any(|p| p.id == user.id) {
                Err(JoinGameError {
                    message: "user is already part of this game".into(),
                    http_status_code: 409,
                })
            } else {
                game.players.push(user.into());
                Ok(())
            }
        } else {
            Err(JoinGameError {
                message: "game not found".into(),
                http_status_code: 404,
            })
        }
    }

    pub fn leave(&self, game_id: u32, user: &User) -> Result<(), LeaveGameError> {
        let mut data = self.data.lock().unwrap();

        if let Some(game) = data.games.get_mut(&game_id) {
            if !game.players.iter().any(|p| p.id == user.id) {
                Err(LeaveGameError {
                    message: "user is not part of this game".into(),
                    http_status_code: 409,
                })
            } else {
                let pos = game.players.iter().position(|p| p.id == user.id).unwrap();
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

                game.players.remove(pos);

                if game.players.is_empty() {
                    data.games.remove(&game_id);
                } else if game.players.len() == 1 && game.state != GameState::Open {
                    drop(data);
                    let _ = self.stop(game_id, None);
                }

                Ok(())
            }
        } else {
            Err(LeaveGameError {
                message: "game not found".into(),
                http_status_code: 404,
            })
        }
    }

    pub fn start(&self, game_id: u32, user: &User) -> Result<Game, StartGameError> {
        let mut data = self.data.lock().unwrap();

        if let Some(game) = data.games.get_mut(&game_id) {
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
                let mut rng = thread_rng();

                game.state = GameState::Guessing;
                game.players.get_mut(0).unwrap().state = PlayerState::Guessing;
                game.players.get_mut(0).unwrap().turn_player = true;
                game.hits_remaining = self
                    .hit_service
                    .lock()
                    .get_all()
                    .into_iter()
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect::<_>();
                game.hits_remaining.make_contiguous().shuffle(&mut rng);

                for i in 0..game.players.len() {
                    let player = game.players.get_mut(i).unwrap();

                    player.hits.push(game.hits_remaining.pop_front().unwrap());
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

    pub fn stop(&self, game_id: u32, user: Option<&User>) -> Result<(), StopGameError> {
        let mut data = self.data.lock().unwrap();

        if let Some(game) = data.games.get_mut(&game_id) {
            if let Some(u) = user {
                if !game
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
            }

            if game.state == GameState::Open {
                return Err(StopGameError {
                    http_status_code: 409,
                    message: "the game isn't running".into(),
                });
            }

            game.state = GameState::Open;
            game.hits_remaining.clear();

            for p in game.players.iter_mut() {
                p.state = PlayerState::Waiting;
                p.tokens = 0;
                p.hits.clear();
                p.turn_player = false;
                p.guess = None;
            }

            Ok(())
        } else {
            Err(StopGameError {
                http_status_code: 404,
                message: "game with that id not found".into(),
            })
        }
    }

    pub fn get_current_hit(&self, game_id: u32) -> Result<Hit, CurrentHitError> {
        let mut data = self.data.lock().unwrap();

        if let Some(game) = data.games.get_mut(&game_id) {
            if game.state == GameState::Open {
                Err(CurrentHitError {
                    message: "game currently isn't running".into(),
                    http_status_code: 409,
                })
            } else {
                game.hits_remaining.front().cloned().ok_or(CurrentHitError {
                    message: "no hit found".into(),
                    http_status_code: 500,
                })
            }
        } else {
            Err(CurrentHitError {
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
        game_id: u32,
        user: &User,
        slot_id: Option<u8>,
    ) -> Result<Game, GuessSlotError> {
        let mut data = self.data.lock().unwrap();

        if let Some(game) = data.games.get_mut(&game_id) {
            if !game.players.iter().any(|p| p.id == user.id) {
                return Err(GuessSlotError {
                    message: "user is not part of this game".into(),
                    http_status_code: 409,
                });
            }

            let turn_player_pos = game.players.iter().position(|p| p.turn_player).unwrap();
            let pos = game.players.iter().position(|p| p.id == user.id).unwrap();

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

            if game.state == GameState::Intercepting
                && !game
                    .players
                    .iter()
                    .any(|p| p.state == PlayerState::Intercepting)
            {
                let len = game.players.len();
                game.state = GameState::Confirming;
                game.hit = game.hits_remaining.front().cloned();
                game.players
                    .get_mut((turn_player_pos + 1) % len)
                    .unwrap()
                    .state = PlayerState::Confirming;
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
        game_id: u32,
        user: &User,
        confirm: bool,
    ) -> Result<Game, ConfirmSlotError> {
        let mut data = self.data.lock().unwrap();

        if let Some(mut game) = data.games.get_mut(&game_id) {
            if !game.players.iter().any(|p| p.id == user.id) {
                return Err(ConfirmSlotError {
                    message: "user is not part of this game".into(),
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
                                    && game.hits_remaining.front().unwrap().year >= s.from_year)
                                || (s.from_year <= game.hits_remaining.front().unwrap().year
                                    && game.hits_remaining.front().unwrap().year <= s.to_year)
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
            } else if winners.len() == 1 {
                let i = winners.get(0).map(|(i, _)| *i).unwrap();
                let player = game.players.get_mut(i).unwrap();

                player
                    .hits
                    .push(game.hits_remaining.front().cloned().unwrap());
                player.slots = self.get_slots(&player.hits);
            }

            game.hits_remaining.pop_front();

            if game
                .players
                .iter()
                .any(|p| p.hits.len() >= game.goal as usize)
            {
                drop(data);
                let _ = self.stop(game_id, None);
                data = self.data.lock().unwrap();
                game = data.games.get_mut(&game_id).unwrap();
            } else {
                for p in game.players.iter_mut() {
                    if p.guess.is_some() && !p.turn_player {
                        p.tokens -= 1;
                    }
                    p.guess = None;
                    p.state = PlayerState::Waiting;
                }

                game.state = GameState::Guessing;
                game.players.get_mut(turn_player_pos).unwrap().turn_player = false;

                let len = game.players.len();

                if let Some(p) = game.players.get_mut((turn_player_pos + 1) % len) {
                    p.turn_player = true;
                    p.state = PlayerState::Guessing;
                }

                if game.hits_remaining.len() == 0 {
                    let mut rng = thread_rng();
                    game.hits_remaining = self
                        .hit_service
                        .lock()
                        .get_all()
                        .into_iter()
                        .filter(|h| !game.players.iter().any(|p| p.hits.contains(h)))
                        .collect::<HashSet<_>>()
                        .into_iter()
                        .collect::<VecDeque<_>>();
                    game.hits_remaining.make_contiguous().shuffle(&mut rng);
                }
            }

            Ok(game.clone())
        } else {
            Err(ConfirmSlotError {
                message: "game not found".into(),
                http_status_code: 404,
            })
        }
    }

    pub fn skip(&self, game_id: u32, user: &User) -> Result<Game, SkipHitError> {
        let mut data = self.data.lock().unwrap();

        if let Some(game) = data.games.get_mut(&game_id) {
            if !game.players.iter().any(|p| p.id == user.id) {
                return Err(SkipHitError {
                    message: "user is not part of this game".into(),
                    http_status_code: 409,
                });
            }

            let pos = game.players.iter().position(|p| p.id == user.id).unwrap();

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

            game.hits_remaining.pop_front();

            if game.hits_remaining.len() == 0 {
                let mut rng = thread_rng();
                game.hits_remaining = self
                    .hit_service
                    .lock()
                    .get_all()
                    .into_iter()
                    .filter(|h| !game.players.iter().any(|p| p.hits.contains(h)))
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect::<VecDeque<_>>();
                game.hits_remaining.make_contiguous().shuffle(&mut rng);
            }

            Ok(game.clone())
        } else {
            Err(SkipHitError {
                message: "game not found".into(),
                http_status_code: 404,
            })
        }
    }
}
