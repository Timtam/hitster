mod games;
mod hits;
mod stats;
mod users;

pub use games::GameService;
pub use hits::HitService;
use parking_lot::{MappedMutexGuard, Mutex, MutexGuard};
pub use stats::{StatsService, StatsServiceFairing};
use std::{default::Default, sync::Arc};
pub use users::UserService;

pub struct ServiceHandle<T>(Arc<Mutex<T>>);

impl<T> ServiceHandle<T> {
    pub fn new(t: T) -> Self {
        Self(Arc::new(Mutex::new(t)))
    }

    pub fn lock(&self) -> MappedMutexGuard<'_, T> {
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
    stats_service: Option<Arc<StatsService>>,
}

pub struct ServiceStore {
    data: Mutex<ServiceStoreData>,
}

impl ServiceStore {
    pub fn hit_service(&self) -> ServiceHandle<HitService> {
        let mut data = self.data.lock();

        if data.hit_service.is_none() {
            data.hit_service
                .replace(ServiceHandle::new(HitService::default()));
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
        let ss = self.stats_service();
        let mut data = self.data.lock();

        if data.game_service.is_none() {
            data.game_service
                .replace(ServiceHandle::new(GameService::new(hs, ss)));
        }

        data.game_service.as_ref().cloned().unwrap()
    }

    pub fn stats_service(&self) -> Arc<StatsService> {
        let mut data = self.data.lock();

        if data.stats_service.is_none() {
            data.stats_service
                .replace(Arc::new(StatsService::default()));
        }

        Arc::clone(data.stats_service.as_ref().unwrap())
    }
}

impl Default for ServiceStore {
    fn default() -> Self {
        Self {
            data: Mutex::new(ServiceStoreData::default()),
        }
    }
}
