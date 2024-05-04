use crate::hits::{get_all, Hit};

pub struct HitService {
    hits: Vec<&'static Hit>,
}

impl HitService {
    pub fn new() -> Self {
        Self {
            hits: get_all().iter().filter(|h| h.exists()).collect::<Vec<_>>(),
        }
    }

    pub fn get_all(&self) -> Vec<&'static Hit> {
        self.hits.clone()
    }
}
