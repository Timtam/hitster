use hitster_core::{Hit, HitsterData, Pack};
use uuid::Uuid;

pub struct HitService {
    hitster_data: HitsterData,
    finished_downloading: bool,
}

impl HitService {
    pub fn new(hitster_data: HitsterData) -> Self {
        Self {
            hitster_data,
            finished_downloading: false,
        }
    }

    pub fn copy_hits(&self) -> Vec<Hit> {
        self.hitster_data
            .get_hits()
            .into_iter()
            .cloned()
            .collect::<Vec<_>>()
    }

    pub fn insert_hit(&mut self, hit: Hit) {
        self.hitster_data.insert_hit(hit);
    }

    pub fn insert_pack(&mut self, pack: Pack) {
        self.hitster_data.insert_pack(pack);
    }

    pub fn get_progress(&self) -> (usize, usize, bool) {
        (
            self.hitster_data.get_hits().len(),
            self.hitster_data.get_hits().len(),
            self.finished_downloading,
        )
    }

    pub fn set_finished_downloading(&mut self) {
        self.finished_downloading = true
    }

    pub fn get_packs(&self) -> Vec<&Pack> {
        self.hitster_data.get_packs()
    }

    pub fn get_hits_for_packs(&self, packs: &[Uuid]) -> Vec<&Hit> {
        self.hitster_data.get_hits_for_packs(packs)
    }
}
impl Default for HitService {
    fn default() -> Self {
        HitService::new(HitsterData::new(vec![], vec![]))
    }
}
