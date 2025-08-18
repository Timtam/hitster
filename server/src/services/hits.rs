use crate::{
    hits::{DownloadHitData, HitSearchQuery, SortBy, SortDirection},
    responses::PaginatedResponse,
};
use fuse_rust::Fuse;
use hitster_core::{Hit, HitsterData, Pack};
use rocket::tokio::sync::broadcast::Sender;
use std::cmp::Ordering;
use uuid::Uuid;

pub struct HitService {
    hitster_data: HitsterData,
    downloading: bool,
    processing: bool,
    dl_sender: Option<Sender<Hit>>,
    process_sender: Option<Sender<DownloadHitData>>,
}

impl HitService {
    pub fn new(hitster_data: HitsterData) -> Self {
        Self {
            hitster_data,
            downloading: false,
            processing: false,
            dl_sender: None,
            process_sender: None,
        }
    }

    pub fn get_hits(&self) -> Vec<&Hit> {
        self.hitster_data.get_hits()
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

    pub fn downloading(&self) -> usize {
        if self.downloading {
            self.dl_sender.as_ref().map(|d| d.len() + 1).unwrap_or(0)
        } else {
            0
        }
    }

    pub fn set_downloading(&mut self, downloading: bool) {
        self.downloading = downloading
    }

    pub fn processing(&self) -> usize {
        if self.processing {
            self.process_sender
                .as_ref()
                .map(|p| p.len() + 1)
                .unwrap_or(0)
        } else {
            0
        }
    }

    pub fn set_processing(&mut self, processing: bool) {
        self.processing = processing
    }

    pub fn get_packs(&self) -> Vec<&Pack> {
        self.hitster_data.get_packs()
    }

    pub fn get_hits_for_packs(&self, packs: &[Uuid]) -> Vec<&Hit> {
        self.hitster_data.get_hits_for_packs(packs)
    }

    pub fn set_download_info(
        &mut self,
        dl_sender: Sender<Hit>,
        process_sender: Sender<DownloadHitData>,
    ) {
        self.dl_sender = Some(dl_sender);
        self.process_sender = Some(process_sender);
    }

    pub fn search_hits(&self, query: &HitSearchQuery) -> PaginatedResponse<Hit> {
        let def = HitSearchQuery::default();
        let start = query.start.or(def.start).unwrap();
        let mut amount = query.amount.or(def.amount).unwrap();
        let q = query.query.as_ref().or(def.query.as_ref()).unwrap();
        let mut sort_by = query
            .sort_by
            .as_ref()
            .or(def.sort_by.as_ref())
            .cloned()
            .unwrap();

        if sort_by.is_empty() {
            sort_by.push(SortBy::Title);
        }

        let mut hits = query
            .packs
            .as_ref()
            .or(def.packs.as_ref())
            .map(|p| {
                if p.is_empty() {
                    self.get_hits()
                } else {
                    self.get_hits_for_packs(p)
                }
            })
            .unwrap();

        if !q.is_empty() {
            let fs = Fuse::default();
            hits = fs
                .search_text_in_fuse_list(q, &hits)
                .into_iter()
                .filter(|r| r.score <= 0.05)
                .map(|r| *hits.get(r.index).unwrap())
                .collect::<Vec<_>>();
        }

        let total = hits.len();

        hits.sort_by(|a, b| {
            let mut res = Ordering::Equal;

            for s in sort_by.iter() {
                res = match s {
                    SortBy::Title => natord::compare_ignore_case(&a.title, &b.title),
                    SortBy::Artist => natord::compare_ignore_case(&a.artist, &b.artist),
                    SortBy::BelongsTo => natord::compare_ignore_case(&a.belongs_to, &b.belongs_to),
                    SortBy::Year => a.year.cmp(&b.year),
                };
                if res != Ordering::Equal {
                    break;
                }
            }
            res
        });

        if query.sort_direction.or(def.sort_direction).unwrap() == SortDirection::Descending {
            hits.reverse();
        }

        let results = hits
            .into_iter()
            .skip(start - 1)
            .take(amount)
            .cloned()
            .collect::<Vec<_>>();

        amount = results.len();

        PaginatedResponse {
            results,
            start,
            end: start + amount - 1,
            total,
        }
    }
}

impl Default for HitService {
    fn default() -> Self {
        HitService::new(HitsterData::new(vec![], vec![]))
    }
}
