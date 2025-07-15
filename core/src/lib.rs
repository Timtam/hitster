mod core {
    use schemars::JsonSchema;
    use serde::Serialize;
    use std::{
        cmp::PartialEq,
        env,
        hash::{Hash, Hasher},
        path::{Path, PathBuf},
    };
    use uuid::Uuid;

    #[derive(Clone, Eq, Debug, Serialize, JsonSchema)]
    pub struct Hit {
        pub artist: &'static str,
        pub title: &'static str,
        pub belongs_to: &'static str,
        pub year: u32,
        pub pack: &'static str,
        #[serde(skip)]
        pub playback_offset: u16,
        pub id: Uuid,
        #[serde(skip)]
        pub yt_id: &'static str,
    }

    impl Hit {
        pub fn download_dir() -> String {
            env::var("DOWNLOAD_DIRECTORY").unwrap_or("./hits".to_string())
        }

        pub fn file(&self) -> PathBuf {
            Path::new(&Hit::download_dir())
                .join(format!("{}_{}.mp3", self.yt_id, self.playback_offset))
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
}

pub use core::Hit;
