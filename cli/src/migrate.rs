use hitster_core::{Hit, HitId, HitsterData, Pack};
use regex_lite::Regex;
use std::{collections::HashMap, fs, path::PathBuf};
use terminal_menu::{button, label, list, menu, mut_menu, run};
use uuid::Uuid;

pub fn migrate(file: PathBuf) -> bool {
    let mut hits = HashMap::<String, Hit>::new();
    let mut packs = HashMap::<String, Pack>::new();
    let csv_data = fs::read_to_string(file).expect("Unable to read hits");
    let yt_id: Regex =
        Regex::new(r"^.*((youtu.be\/)|(v\/)|(\/u\/\w\/)|(embed\/)|(watch\?))\??v?=?([^#&?]*).*")
            .unwrap();
    let mut csv_reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .trim(csv::Trim::All)
        .from_reader(csv_data.as_bytes());

    let hit_ref = fs::read_to_string(PathBuf::from("etc/hits.yml"))
        .map(|s| serde_yml::from_str::<HitsterData>(&s).unwrap())
        .unwrap_or(HitsterData::new(vec![], vec![]));

    for result in csv_reader.records() {
        let record = result.unwrap();

        if record.get(5).unwrap() != "" {
            let my_yt_id = yt_id
                .captures(record.get(5).unwrap())
                .map(|caps| caps[7].to_string())
                .unwrap_or_else(|| {
                    panic!(
                        "no valid link found for {}: {}",
                        record.get(0).unwrap(),
                        record.get(2).unwrap()
                    )
                });

            let pack = record.get(3).unwrap().to_string();

            if !packs.contains_key(&pack) {
                packs.insert(
                    pack.clone(),
                    Pack {
                        id: hit_ref
                            .get_packs()
                            .into_iter()
                            .find(|p| p.name == pack)
                            .map(|p| p.id)
                            .unwrap_or_else(Uuid::new_v4),
                        name: pack.clone(),
                    },
                );
            }

            let artist = record.get(0).unwrap().to_string();
            let title = record.get(2).unwrap().to_string();
            let year = record.get(1).unwrap().to_string().parse::<u32>().unwrap();
            let playback_offset = record.get(6).unwrap().to_string().parse::<u16>().unwrap();
            let mut belongs_to = record.get(4).unwrap().to_string();

            if artist.is_empty() || title.is_empty() || year == 0 {
                continue;
            }

            if !hits.contains_key(&my_yt_id) {
                hits.insert(
                    my_yt_id.clone(),
                    Hit {
                        artist,
                        title,
                        year,
                        playback_offset,
                        packs: vec![packs.get(&pack).unwrap().id],
                        belongs_to,
                        id: hit_ref
                            .get_hit(&HitId::YtId(my_yt_id.clone()))
                            .map(|h| h.id)
                            .unwrap_or_else(Uuid::new_v4),
                        yt_id: my_yt_id,
                    },
                );
            } else if let Some(hit) = hits.get_mut(&my_yt_id) {
                let mut diff = false;

                if hit.belongs_to.is_empty() && !belongs_to.is_empty() {
                    hit.belongs_to = belongs_to.clone();
                } else if !hit.belongs_to.is_empty() && belongs_to.is_empty() {
                    belongs_to = hit.belongs_to.clone();
                }

                if title.to_lowercase() != hit.title.to_lowercase()
                    || artist.to_lowercase() != hit.artist.to_lowercase()
                    || playback_offset != hit.playback_offset
                    || year != hit.year
                    || belongs_to.to_lowercase() != hit.belongs_to.to_lowercase()
                {
                    diff = true;
                }

                if diff {
                    let mut m = vec![
                        label("--- Hit difference spotted ---"),
                        label(
                            "Please select the properties you want to keep for the different fields.",
                        ),
                        label("Press escape or q to cancel."),
                        button("Confirm"),
                    ];

                    if hit.title.to_lowercase() != title.to_lowercase() {
                        m.push(list("Title:", vec![&hit.title, &title]));
                    } else {
                        m.push(list("Title:", vec![&title]));
                    }

                    if hit.artist.to_lowercase() != artist.to_lowercase() {
                        m.push(list(
                            "Artist:",
                            vec![&hit.artist, &artist, "Concatenate both"],
                        ));
                    } else {
                        m.push(list("Artist:", vec![&artist]));
                    }

                    if hit.year != year {
                        m.push(list("Year:", vec![hit.year.to_string(), year.to_string()]));
                    } else {
                        m.push(list("Year:", vec![year.to_string()]));
                    }

                    if hit.belongs_to.to_lowercase() != belongs_to.to_lowercase() {
                        m.push(list(
                            "Belongs to:",
                            vec![&hit.belongs_to, &belongs_to, "Concatenate both"],
                        ));
                    } else {
                        m.push(list("Belongs to:", vec![&belongs_to]));
                    }

                    if hit.playback_offset != playback_offset {
                        m.push(list(
                            "Playback offset:",
                            vec![hit.playback_offset.to_string(), playback_offset.to_string()],
                        ));
                    } else {
                        m.push(list("Playback offset:", vec![playback_offset.to_string()]));
                    }

                    let menu = menu(m);
                    run(&menu);

                    let mm = mut_menu(&menu);

                    if mm.canceled() {
                        return false;
                    }

                    hit.title = mm.selection_value("Title:").to_string();

                    if mm.selection_value("Artist:") == "Concatenate both" {
                        hit.artist = format!("{}, {}", &hit.artist, &artist);
                    } else {
                        hit.artist = mm.selection_value("Artist:").to_string();
                    }

                    if mm.selection_value("Belongs to:") == "Concatenate both" {
                        hit.belongs_to = format!("{}, {}", &hit.belongs_to, &belongs_to);
                    } else {
                        hit.belongs_to = mm.selection_value("Belongs to:").to_string();
                    }

                    hit.year = mm.selection_value("Year:").parse::<u32>().unwrap();
                    hit.playback_offset = mm
                        .selection_value("Playback offset:")
                        .parse::<u16>()
                        .unwrap();
                }

                hit.packs.push(packs.get(&pack).unwrap().id);
            }
        }
    }

    let data = HitsterData::new(
        hits.into_values().collect::<Vec<_>>(),
        packs.into_values().collect::<Vec<_>>(),
    );

    let _ = fs::write(
        PathBuf::from("hits.yml"),
        serde_yml::to_string(&data).unwrap(),
    );
    true
}
