use crate::users::User;
use std::{collections::HashMap, sync::Mutex};

struct UserServiceData {
    users: HashMap<u32, User>,
}

pub struct UserService {
    data: Mutex<UserServiceData>,
}

impl UserService {
    pub fn new() -> Self {
        Self {
            data: Mutex::new(UserServiceData {
                users: HashMap::new(),
            }),
        }
    }

    pub fn add(&self, user: User) {
        let mut data = self.data.lock().unwrap();
        data.users.insert(user.id, user);
    }

    pub fn get_all(&self) -> Vec<User> {
        self.data
            .lock()
            .unwrap()
            .users
            .clone()
            .into_values()
            .collect::<_>()
    }

    pub fn get_by_id(&self, id: u32) -> Option<User> {
        self.data.lock().unwrap().users.get(&id).cloned()
    }

    pub fn get_by_username<'r>(&self, username: &'r str) -> Option<User> {
        self.data
            .lock()
            .unwrap()
            .users
            .values()
            .find(|u| u.username == username)
            .cloned()
    }

    pub fn remove(&self, id: u32) {
        self.data.lock().unwrap().users.remove(&id);
    }
}
