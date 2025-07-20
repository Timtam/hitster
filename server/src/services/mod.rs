mod games;
mod hits;
mod users;

use crate::hits::get_hitster_data;
pub use games::GameService;
pub use hits::HitService;
use parking_lot::{MappedMutexGuard, Mutex, MutexGuard};
use std::{default::Default, sync::Arc};
pub use users::UserService;

pub struct ServiceHandle<T>(Arc<Mutex<T>>);

impl<T> ServiceHandle<T> {
    pub fn new(t: T) -> Self {
        Self(Arc::new(Mutex::new(t)))
    }

    pub fn lock(&self) -> MappedMutexGuard<T> {
        MutexGuard::map(self.0.lock(), |s| s)
    }
}

impl<T> Clone for ServiceHandle<T> {
    fn clone(&self) -> Self {
        ServiceHandle(Arc::clone(&self.0))
    }
}

#[derive(Default)]
pub struct ServiceStoreData {
    game_service: Option<ServiceHandle<GameService>>,
    hit_service: Option<ServiceHandle<HitService>>,
    user_service: Option<ServiceHandle<UserService>>,
}

pub struct ServiceStore {
    data: Mutex<ServiceStoreData>,
}

impl ServiceStore {
    pub fn hit_service(&self) -> ServiceHandle<HitService> {
        let mut data = self.data.lock();

        if data.hit_service.is_none() {
            data.hit_service.replace(ServiceHandle::new(HitService::new(
                get_hitster_data().clone(),
            )));
        }

        data.hit_service.as_ref().cloned().unwrap()
    }

    pub fn user_service(&self) -> ServiceHandle<UserService> {
        let mut data = self.data.lock();

        if data.user_service.is_none() {
            data.user_service
                .replace(ServiceHandle::new(UserService::new()));
        }

        data.user_service.as_ref().cloned().unwrap()
    }

    pub fn game_service(&self) -> ServiceHandle<GameService> {
        let hs = self.hit_service();
        let mut data = self.data.lock();

        if data.game_service.is_none() {
            data.game_service
                .replace(ServiceHandle::new(GameService::new(hs)));
        }

        data.game_service.as_ref().cloned().unwrap()
    }
}

impl Default for ServiceStore {
    fn default() -> Self {
        Self {
            data: Mutex::new(ServiceStoreData::default()),
        }
    }
}
