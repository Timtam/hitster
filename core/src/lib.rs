mod hitster_core {
    use multi_key_map::MultiKeyMap;
    use serde::{Deserialize, Serialize};
    use std::{
        cmp::PartialEq,
        collections::HashMap,
        convert::From,
        env,
        hash::{Hash, Hasher},
        path::{Path, PathBuf},
    };
    use time::OffsetDateTime;
    use uuid::Uuid;

    #[derive(Clone, Eq, Debug, Serialize, Deserialize)]
    pub struct Hit {
        pub artist: String,
        pub title: String,
        pub belongs_to: String,
        pub year: u32,
        pub packs: Vec<Uuid>,
        pub playback_offset: u16,
        pub id: Uuid,
        pub yt_id: String,
        #[serde(with = "time::serde::rfc3339")]
        #[serde(default = "OffsetDateTime::now_utc")]
        pub last_modified: OffsetDateTime,
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

    #[derive(Clone, Eq, Debug, Serialize, Deserialize)]
    pub struct Pack {
        pub id: Uuid,
        pub name: String,
        #[serde(with = "time::serde::rfc3339")]
        #[serde(default = "OffsetDateTime::now_utc")]
        pub last_modified: OffsetDateTime,
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

    #[derive(Clone, Eq, PartialEq, Hash, Debug)]
    pub enum HitId {
        Id(Uuid),
        YtId(String),
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[serde(from = "HitsterFileFormat")]
    #[serde(into = "HitsterFileFormat")]
    pub struct HitsterData {
        hits: MultiKeyMap<HitId, Hit>,
        packs: HashMap<Uuid, Pack>,
    }

    impl HitsterData {
        pub fn new(hits: Vec<Hit>, packs: Vec<Pack>) -> Self {
            HitsterData {
                hits: hits
                    .into_iter()
                    .map(|h| (vec![HitId::Id(h.id), HitId::YtId(h.yt_id.clone())], h))
                    .collect::<MultiKeyMap<HitId, Hit>>(),
                packs: packs
                    .into_iter()
                    .map(|p| (p.id, p))
                    .collect::<HashMap<Uuid, Pack>>(),
            }
        }

        pub fn get_hits(&self) -> Vec<&Hit> {
            self.hits.values().collect::<Vec<_>>()
        }

        pub fn insert_pack(&mut self, pack: Pack) {
            self.packs.insert(pack.id, pack);
        }

        pub fn insert_hit(&mut self, hit: Hit) {
            self.hits
                .insert_many(vec![HitId::Id(hit.id), HitId::YtId(hit.yt_id.clone())], hit);
        }

        pub fn get_packs(&self) -> Vec<&Pack> {
            self.packs.values().collect::<Vec<_>>()
        }

        pub fn get_hits_for_packs(&self, packs: &[Uuid]) -> Vec<&Hit> {
            self.hits
                .values()
                .filter(|h| packs.iter().any(|p| h.packs.contains(p)))
                .collect::<Vec<_>>()
        }

        pub fn get_hit(&self, hit_id: &HitId) -> Option<&Hit> {
            self.hits.get(hit_id)
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
                    &format!("{} {} {} {}", &a.artist, &a.title, a.year, &a.belongs_to),
                    &format!("{} {} {} {}", &b.artist, &b.title, b.year, &b.belongs_to),
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
                    .map(|h| (vec![HitId::Id(h.id), HitId::YtId(h.yt_id.clone())], h))
                    .collect::<MultiKeyMap<HitId, Hit>>(),
                packs: file
                    .packs
                    .into_iter()
                    .map(|p| (p.id, p))
                    .collect::<HashMap<Uuid, Pack>>(),
            }
        }
    }
}

pub use hitster_core::{Hit, HitId, HitsterData, HitsterFileFormat, Pack};
