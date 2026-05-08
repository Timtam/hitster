use hitster_core::User;
use parking_lot::Mutex;
use std::collections::HashMap;
use time::{Duration, OffsetDateTime};
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
        let mut data = self.data.lock();
        data.users.insert(user.id, user);
    }

    pub fn get_all(&self) -> Vec<User> {
        self.data
            .lock()
            .users
            .clone()
            .into_values()
            .collect::<_>()
    }

    pub fn get_by_id(&self, id: Uuid) -> Option<User> {
        self.data.lock().users.get(&id).cloned()
    }

    pub fn get_by_username(&self, username: &str) -> Option<User> {
        self.data
            .lock()
            .users
            .values()
            .find(|u| u.name == username)
            .cloned()
    }

    pub fn remove(&self, id: Uuid) {
        self.data.lock().users.remove(&id);
    }

    pub fn cleanup_tokens(&self, user: Uuid) -> bool {
        let mut data = self.data.lock();

        if let Some(u) = data.users.get_mut(&user) {
            u.tokens = u
                .tokens
                .clone()
                .into_iter()
                .filter(|t| t.refresh_time > OffsetDateTime::now_utc())
                .collect::<_>();
            // we'll consider all users who didn't refresh their expired token in an hour to be logged off
            !u.tokens
                .iter()
                .any(|t| t.expiration_time + Duration::hours(1) > OffsetDateTime::now_utc())
        } else {
            false
        }
    }
}
