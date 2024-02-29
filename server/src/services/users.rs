use crate::users::User;
use std::sync::Mutex;

struct UserServiceData {
    users: Vec<User>,
}

pub struct UserService {
    data: Mutex<UserServiceData>,
}

impl UserService {
    pub fn new() -> Self {
        Self {
            data: Mutex::new(UserServiceData { users: vec![] }),
        }
    }

    pub fn add(&self, user: User) {
        let mut data = self.data.lock().unwrap();
        data.users.push(user);
    }

    pub fn get_all(&self) -> Vec<User> {
        self.data.lock().unwrap().users.clone()
    }

    pub fn get_by_id(&self, id: u32) -> Option<User> {
        self.data
            .lock()
            .unwrap()
            .users
            .iter()
            .find(|u| u.id == id)
            .cloned()
    }

    pub fn get_by_username<'r>(&self, username: &'r str) -> Option<User> {
        self.data
            .lock()
            .unwrap()
            .users
            .iter()
            .find(|u| u.username == username)
            .cloned()
    }
}
