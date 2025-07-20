use hitster_core::{Hit, HitsterData, Pack};
use uuid::Uuid;

pub struct HitService {
    hitster_data: HitsterData,
    hits: Vec<&'static Hit>,
    finished_downloading: bool,
}

impl HitService {
    pub fn new(hitster_data: HitsterData) -> Self {
        Self {
            hitster_data,
            hits: vec![],
            finished_downloading: false,
        }
    }

    pub fn get_available_hits(&self) -> Vec<&'static Hit> {
        self.hits.clone()
    }

    pub fn add(&mut self, hit: &'static Hit) {
        self.hits.push(hit);
    }

    pub fn get_progress(&self) -> (usize, usize, bool) {
        (
            self.hits.len(),
            self.hitster_data.get_hits().len(),
            self.finished_downloading,
        )
    }

    pub fn set_finished_downloading(&mut self) {
        self.finished_downloading = true
    }

    pub fn get_hit(&self, hit_id: Uuid) -> Option<&Hit> {
        self.hitster_data.get_hit(hit_id)
    }

    pub fn get_pack(&self, pack_id: Uuid) -> Option<&Pack> {
        self.hitster_data.get_pack(pack_id)
    }
}
