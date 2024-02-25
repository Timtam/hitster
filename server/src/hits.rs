include!(concat!(env!("OUT_DIR"), "/hits.rs"));

pub struct Hit {
    pub interpret: String,
    pub title: String,
    pub year: u32,
}
