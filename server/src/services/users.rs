use crate::users::User;
use std::{collections::HashMap, sync::Mutex};
use uuid::Uuid;

struct UserServiceData {
    users: HashMap<Uuid, User>,
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

    pub fn get_by_id(&self, id: Uuid) -> Option<User> {
        self.data.lock().unwrap().users.get(&id).cloned()
    }

    pub fn get_by_username(&self, username: &str) -> Option<User> {
        self.data
            .lock()
            .unwrap()
            .users
            .values()
            .find(|u| u.name == username)
            .cloned()
    }

    pub fn remove(&self, id: Uuid) {
        self.data.lock().unwrap().users.remove(&id);
    }
}
