use crate::hits::DownloadHitData;
use hitster_core::{Hit, HitsterData, Pack};
use rocket::tokio::sync::broadcast::Sender;
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
}
impl Default for HitService {
    fn default() -> Self {
        HitService::new(HitsterData::new(vec![], vec![]))
    }
}
