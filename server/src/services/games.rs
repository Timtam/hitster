use crate::games::{Game, GameState};
use crate::users::User;
use std::sync::Mutex;

pub struct GameServiceData {
    games: Vec<Game>,
    id: u64,
}

pub struct GameService {
    data: Mutex<GameServiceData>,
}

impl GameService {
    pub fn new() -> Self {
        Self {
            data: Mutex::new(GameServiceData {
                id: 0,
                games: vec![],
            }),
        }
    }

    pub fn add(&self, creator: User) -> Game {
        let mut data = self.data.lock().unwrap();
        data.id += 1;

        let game = Game {
            id: data.id,
            creator: creator.clone(),
            players: vec![creator.clone()],
            state: GameState::Open,
        };

        data.games.push(game.clone());

        game
    }

    pub fn get_all(&self) -> Vec<Game> {
        self.data.lock().unwrap().games.clone()
    }
}
