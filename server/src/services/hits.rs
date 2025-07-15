use crate::hits::get_all;
use hitster_core::Hit;

pub struct HitService {
    hits: Vec<&'static Hit>,
    finished_downloading: bool,
}

impl HitService {
    pub fn new() -> Self {
        Self {
            hits: vec![],
            finished_downloading: false,
        }
    }

    pub fn get_all(&self) -> Vec<&'static Hit> {
        self.hits.clone()
    }

    pub fn add(&mut self, hit: &'static Hit) {
        self.hits.push(hit);
    }

    pub fn get_progress(&self) -> (usize, usize, bool) {
        (self.hits.len(), get_all().len(), self.finished_downloading)
    }

    pub fn set_finished_downloading(&mut self) {
        self.finished_downloading = true
    }
}
