use crate::{
    hits::{DownloadHitData, HitSearchQuery, SortBy, SortDirection},
    responses::PaginatedResponse,
};
use hitster_core::{Hit, HitId, HitsterData, Pack};
use rocket::tokio::sync::broadcast::Sender;
use std::{cmp::Ordering, collections::HashMap};
use uuid::Uuid;

pub struct HitService {
    hitster_data: HitsterData,
    downloading: bool,
    processing: bool,
    dl_sender: Option<Sender<Hit>>,
    process_sender: Option<Sender<DownloadHitData>>,
    availability_sender: Option<Sender<Hit>>,
}

impl HitService {
    pub fn new(hitster_data: HitsterData) -> Self {
        Self {
            hitster_data,
            downloading: false,
            processing: false,
            dl_sender: None,
            process_sender: None,
            availability_sender: None,
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

    pub fn get_pack(&self, pack: Uuid) -> Option<&Pack> {
        self.hitster_data.get_pack(pack)
    }

    pub fn remove_pack(&mut self, pack: Uuid) -> bool {
        self.hitster_data.remove_pack(pack)
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

    pub fn set_availability_sender(&mut self, availability_sender: Sender<Hit>) {
        self.availability_sender = Some(availability_sender);
    }

    pub fn get_hit(&self, hit_id: &HitId) -> Option<&Hit> {
        self.hitster_data.get_hit(hit_id)
    }

    pub fn remove_hit(&mut self, hit: &HitId) -> bool {
        self.hitster_data.remove_hit(hit)
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

        let mut hits = if !q.is_empty() {
            self.hitster_data.search_hits(q)
        } else {
            self.get_hits()
        };

        if let Some(packs) = query.packs.as_ref()
            && !packs.is_empty()
        {
            hits = packs
                .iter()
                .fold(
                    hits.into_iter()
                        .map(|h| (HitId::Id(h.id), h))
                        .collect::<HashMap<HitId, &Hit>>(),
                    |mut hits, p| {
                        hits.retain(|_, hit| hit.packs.contains(p));
                        hits
                    },
                )
                .into_values()
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

    pub fn download_hit(&self, hit: Hit) {
        let _ = self.dl_sender.as_ref().unwrap().send(hit);
    }

    pub fn queue_availability_check(&self, hit: Hit) {
        if let Some(sender) = &self.availability_sender {
            let _ = sender.send(hit);
        }
    }
}

impl Default for HitService {
    fn default() -> Self {
        HitService::new(HitsterData::new(vec![], vec![]))
    }
}
