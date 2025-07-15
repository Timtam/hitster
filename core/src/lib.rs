mod core {
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};
    use std::{
        cmp::PartialEq,
        env,
        hash::{Hash, Hasher},
        path::{Path, PathBuf},
    };
    use uuid::Uuid;

    #[derive(Clone, Eq, Debug, Serialize, Deserialize, JsonSchema)]
    pub struct Hit {
        pub artist: String,
        pub title: String,
        pub belongs_to: String,
        pub year: u32,
        pub packs: Vec<String>,
        #[serde(skip)]
        pub playback_offset: u16,
        pub id: Uuid,
        #[serde(skip)]
        pub yt_id: String,
    }

    impl Hit {
        pub fn download_dir() -> String {
            env::var("DOWNLOAD_DIRECTORY").unwrap_or("./hits".to_string())
        }

        pub fn file(&self) -> PathBuf {
            Path::new(&Hit::download_dir()).join(format!(
                "{}_{}.mp3",
                self.yt_id.as_str(),
                self.playback_offset
            ))
        }

        pub fn exists(&self) -> bool {
            self.file().is_file()
        }
    }

    impl PartialEq for Hit {
        fn eq(&self, h: &Self) -> bool {
            self.yt_id == h.yt_id
        }
    }

    impl Hash for Hit {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.yt_id.hash(state);
        }
    }

    #[derive(Clone, Eq, Debug, Serialize, Deserialize, JsonSchema)]
    pub struct Pack {
        pub id: Uuid,
        pub name: String,
    }

    impl PartialEq for Pack {
        fn eq(&self, p: &Self) -> bool {
            self.id == p.id
        }
    }

    impl Hash for Pack {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.id.hash(state);
        }
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct HitsterData {
        pub version: u8,
        pub hits: Vec<Hit>,
        pub packs: Vec<Pack>,
    }

    impl HitsterData {
        pub fn new() -> Self {
            HitsterData {
                version: 2,
                // v1 was csv
                hits: vec![],
                packs: vec![],
            }
        }
    }
}

pub use core::{Hit, HitsterData, Pack};
