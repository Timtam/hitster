use crate::games::{Game, GameState};
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
            creator: creator,
            players: vec![creator],
            state: GameState::Open,
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

    pub fn join(&self, game_id: u32, user_id: u32) -> Result<(), &'static str> {
        let mut data = self.data.lock().unwrap();

        if let Some(game) = data.games.get_mut(&game_id) {
            if game.players.contains(&user_id) {
                Err("user is already part of this game")
            } else {
                game.players.push(user_id);
                Ok(())
            }
        } else {
            Err("game not found")
        }
    }

    pub fn leave(&self, game_id: u32, user_id: u32) -> Result<(), &'static str> {
        let mut data = self.data.lock().unwrap();

        if let Some(game) = data.games.get_mut(&game_id) {
            if !game.players.contains(&user_id) {
                Err("user is not part of this game")
            } else {
                game.players
                    .swap_remove(game.players.iter().position(|u| *u == user_id).unwrap());

                if game.players.len() == 0 {
                    data.games.remove(&game_id);
                } else if game.creator == user_id {
                    game.creator = *game.players.first().unwrap();
                }

                Ok(())
            }
        } else {
            Err("game not found")
        }
    }
}
