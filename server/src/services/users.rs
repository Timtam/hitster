use crate::users::User;
use names::Generator;
use std::sync::Mutex;

struct UserServiceData {
    users: Vec<User>,
    id: u32,
}

pub struct UserService {
    data: Mutex<UserServiceData>,
}

impl UserService {
    pub fn new() -> Self {
        Self {
            data: Mutex::new(UserServiceData {
                id: 0,
                users: vec![],
            }),
        }
    }

    pub fn add(&self) -> User {
        let mut gen = Generator::default();
        let mut data = self.data.lock().unwrap();
        data.id += 1;

        let user = User {
            id: data.id,
            name: gen.next().unwrap(),
        };

        data.users.push(user.clone());

        user
    }

    pub fn get_all(&self) -> Vec<User> {
        self.data.lock().unwrap().users.clone()
    }
}
