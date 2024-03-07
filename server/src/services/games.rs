use crate::{
    games::{Game, GameState},
    responses::{JoinGameError, LeaveGameError, StartGameError},
};
use rand::prelude::{thread_rng, SliceRandom};
use std::{collections::HashMap, sync::Mutex};

pub struct GameServiceData {
    games: HashMap<u32, Game>,
    id: u32,
}

pub struct GameService {
    data: Mutex<GameServiceData>,
}

impl GameService {
    pub fn new() -> Self {
        Self {
            data: Mutex::new(GameServiceData {
                id: 0,
                games: HashMap::new(),
            }),
        }
    }

    pub fn add(&self, creator: u32) -> Game {
        let mut data = self.data.lock().unwrap();
        data.id += 1;

        let game = Game {
            id: data.id,
            creator,
            players: vec![creator],
            state: GameState::Open,
            guessing_player: 0,
            confirming_player: 0,
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

    pub fn join(&self, game_id: u32, user_id: u32) -> Result<(), JoinGameError> {
        let mut data = self.data.lock().unwrap();

        if let Some(game) = data.games.get_mut(&game_id) {
            if game.players.contains(&user_id) {
                Err(JoinGameError {
                    message: "user is already part of this game".into(),
                    http_status_code: 409,
                })
            } else {
                game.players.push(user_id);
                Ok(())
            }
        } else {
            Err(JoinGameError {
                message: "game not found".into(),
                http_status_code: 404,
            })
        }
    }

    pub fn leave(&self, game_id: u32, user_id: u32) -> Result<(), LeaveGameError> {
        let mut data = self.data.lock().unwrap();

        if let Some(game) = data.games.get_mut(&game_id) {
            if !game.players.contains(&user_id) {
                Err(LeaveGameError {
                    message: "user is not part of this game".into(),
                    http_status_code: 409,
                })
            } else {
                let mut pos = game.players.iter().position(|u| *u == user_id).unwrap();
                game.players.remove(pos);

                if game.players.is_empty() {
                    data.games.remove(&game_id);
                } else {
                    if pos >= game.players.len() {
                        pos = 0;
                    }
                    if game.creator == user_id {
                        game.creator = *game.players.get(pos).unwrap();
                    }

                    if game.players.len() == 1 {
                        drop(data);
                        self.stop(game_id);
                    } else {
                        if game.guessing_player == user_id {
                            game.guessing_player = *game.players.get(pos).unwrap();
                        }
                        game.confirming_player = *game
                            .players
                            .iter()
                            .chain(game.players.iter())
                            .filter(|u| **u != game.guessing_player)
                            .nth(pos)
                            .unwrap();
                    }
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

    pub fn start(&self, game_id: u32, user_id: u32) -> Result<(), StartGameError> {
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
            } else if game.creator != user_id {
                Err(StartGameError {
                    http_status_code: 403,
                    message: "only the creator can start a game".into(),
                })
            } else {
                let mut rng = thread_rng();

                game.players.shuffle(&mut rng);

                game.guessing_player = *game.players.first().unwrap();
                game.confirming_player = *game.players.get(1).unwrap();
                game.state = GameState::Guessing;
                Ok(())
            }
        } else {
            Err(StartGameError {
                message: "game not found".into(),
                http_status_code: 404,
            })
        }
    }

    pub fn stop(&self, game_id: u32) {
        let mut data = self.data.lock().unwrap();

        if let Some(game) = data.games.get_mut(&game_id) {
            game.state = GameState::Open;
            game.guessing_player = 0;
            game.confirming_player = 0;
        }
    }
}
