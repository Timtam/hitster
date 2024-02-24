use crate::{hit::Hit, hits};

pub struct HitsService {
    hits: Vec<Hit>,
}

impl HitsService {
    pub fn new() -> Self {
        Self {
            hits: hits::get_all(),
        }
    }
}
