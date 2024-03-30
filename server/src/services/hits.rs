use crate::hits::{get_all, Hit};

pub struct HitService {
    hits: Vec<Hit>,
}

impl HitService {
    pub fn new() -> Self {
        Self {
            hits: get_all().into_iter().filter(|h| h.exists()).collect::<_>(),
        }
    }
}
