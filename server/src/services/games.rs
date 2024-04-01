use crate::{
    games::{Game, GameState, PlayerState},
    hits::Hit,
    responses::{CurrentHitError, JoinGameError, LeaveGameError, StartGameError, StopGameError},
    services::{HitService, ServiceHandle},
    users::User,
};
use rand::prelude::{thread_rng, SliceRandom};
use std::{collections::HashMap, sync::Mutex};

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

        let game = Game {
            id: data.id,
            creator: 0,
            players: vec![creator.into()],
            state: GameState::Open,
            hits_remaining: vec![],
            hit_duration: 20,
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
            if game.players.iter().any(|p| p.id == user.id) {
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

                if game.players.iter().find(|p| p.id == user.id).unwrap().state
                    != PlayerState::Waiting
                {
                    for i in 0..game.players.len() {
                        if (pos == game.players.len() - 1 && i == 0) || (i == pos + 1) {
                            game.players.get_mut(i).unwrap().state = PlayerState::Guessing;
                        } else {
                            game.players.get_mut(i).unwrap().state = PlayerState::Waiting;
                        }
                    }
                }

                game.players.remove(pos);

                if game.players.is_empty() {
                    data.games.remove(&game_id);
                } else if game.players.len() == 1 {
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

    pub fn start(&self, game_id: u32, user: &User) -> Result<(), StartGameError> {
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
            } else if game.players.get(game.creator).unwrap().id != user.id {
                Err(StartGameError {
                    http_status_code: 403,
                    message: "only the creator can start a game".into(),
                })
            } else {
                let mut rng = thread_rng();

                game.players.shuffle(&mut rng);

                game.state = GameState::Guessing;
                game.players.get_mut(0).unwrap().state = PlayerState::Guessing;
                game.hits_remaining = self.hit_service.lock().get_all();
                game.hits_remaining.shuffle(&mut rng);
                Ok(())
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
                if game
                    .players
                    .iter()
                    .enumerate()
                    .find(|(_, p)| p.id == u.id)
                    .unwrap()
                    .0
                    != game.creator
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
            game.hits_remaining = vec![];

            for p in game.players.iter_mut() {
                p.state = PlayerState::Waiting;
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
                Ok(game.hits_remaining.first().cloned().unwrap())
            }
        } else {
            Err(CurrentHitError {
                message: "game not found".into(),
                http_status_code: 404,
            })
        }
    }
}
