mod core {
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};
    use std::{
        cmp::PartialEq,
        collections::HashMap,
        convert::From,
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
        pub packs: Vec<Uuid>,
        pub playback_offset: u16,
        pub id: Uuid,
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
    #[serde(from = "HitsterFileFormat")]
    #[serde(into = "HitsterFileFormat")]
    pub struct HitsterData {
        hits: HashMap<Uuid, Hit>,
        packs: HashMap<Uuid, Pack>,
    }

    impl HitsterData {
        pub fn new(hits: Vec<Hit>, packs: Vec<Pack>) -> Self {
            HitsterData {
                hits: hits
                    .into_iter()
                    .map(|h| (h.id, h))
                    .collect::<HashMap<Uuid, Hit>>(),
                packs: packs
                    .into_iter()
                    .map(|p| (p.id, p))
                    .collect::<HashMap<Uuid, Pack>>(),
            }
        }

        pub fn get_hits(&self) -> Vec<&Hit> {
            self.hits.values().collect::<Vec<_>>()
        }

        pub fn get_packs(&self) -> Vec<&Pack> {
            self.packs.values().collect::<Vec<_>>()
        }

        pub fn get_hits_for_pack(&self, pack: Uuid) -> Vec<&Hit> {
            self.hits
                .values()
                .filter(|h| h.packs.contains(&pack))
                .collect::<Vec<_>>()
        }

        pub fn get_hit(&self, hit_id: Uuid) -> Option<&Hit> {
            self.hits.get(&hit_id)
        }

        pub fn get_pack(&self, pack_id: Uuid) -> Option<&Pack> {
            self.packs.get(&pack_id)
        }
    }

    #[derive(Serialize, Deserialize)]
    pub struct HitsterFileFormat {
        hits: Vec<Hit>,
        packs: Vec<Pack>,
    }

    impl From<HitsterData> for HitsterFileFormat {
        fn from(data: HitsterData) -> Self {
            let mut hits = data.hits.into_values().collect::<Vec<_>>();

            hits.sort_by(|a, b| {
                natord::compare(
                    &format!("{} {}", &a.artist, &a.title),
                    &format!("{} {}", &b.artist, &b.title),
                )
            });

            let mut packs = data.packs.into_values().collect::<Vec<_>>();

            packs.sort_by(|a, b| natord::compare(&a.name, &b.name));

            HitsterFileFormat { hits, packs }
        }
    }

    impl From<HitsterFileFormat> for HitsterData {
        fn from(file: HitsterFileFormat) -> Self {
            HitsterData {
                hits: file
                    .hits
                    .into_iter()
                    .map(|h| (h.id, h))
                    .collect::<HashMap<Uuid, Hit>>(),
                packs: file
                    .packs
                    .into_iter()
                    .map(|p| (p.id, p))
                    .collect::<HashMap<Uuid, Pack>>(),
            }
        }
    }
}

pub use core::{Hit, HitsterData, HitsterFileFormat, Pack};
