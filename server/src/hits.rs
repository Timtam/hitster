use ffmpeg_cli::{FfmpegBuilder, File, Parameter};
use regex::Regex;
use rocket::{
    fairing::{self, Fairing, Info, Kind},
    Build, Rocket,
};
use rocket_okapi::okapi::{schemars, schemars::JsonSchema};
use rusty_ytdl::{Video, VideoOptions, VideoQuality, VideoSearchOptions};
use serde::{Deserialize, Serialize};
use std::{
    env,
    fs::{create_dir_all, remove_file},
    path::{Path, PathBuf},
    process::Stdio,
    str::FromStr,
};
use strum::EnumString;

include!(concat!(env!("OUT_DIR"), "/hits.rs"));

#[derive(EnumString, Eq, PartialEq, Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum Pack {
    Basic,
    Schlagerparty,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Hit {
    pub artist: String,
    pub title: String,
    pub year: u32,
    pub pack: Pack,
    #[serde(skip)]
    pub yt_url: String,
    #[serde(skip)]
    pub playback_offset: u16,
}

impl Hit {
    pub fn yt_id(&self) -> Option<String> {
        let yt_id: Regex = Regex::new(
            r"^.*((youtu.be\/)|(v\/)|(\/u\/\w\/)|(embed\/)|(watch\?))\??v?=?([^#&?]*).*",
        )
        .unwrap();

        yt_id
            .captures(self.yt_url.as_str())
            .map(|caps| caps[7].to_string())
    }

    pub fn download_dir() -> String {
        env::var("DOWNLOAD_DIRECTORY").unwrap_or("./hits".to_string())
    }

    pub fn file(&self) -> Option<PathBuf> {
        self.yt_id()
            .map(|id| Path::new(&Hit::download_dir()).join(format!("{}.mp3", id)))
    }

    pub fn exists(&self) -> bool {
        self.file().map(|p| p.is_file()).unwrap_or(false)
    }
}

#[derive(Default)]
pub struct HitsterDownloader {}

#[rocket::async_trait]
impl Fairing for HitsterDownloader {
    fn info(&self) -> Info {
        Info {
            kind: Kind::Ignite,
            name: "Download hits",
        }
    }

    async fn on_ignite(&self, rocket: Rocket<Build>) -> fairing::Result {
        let download_dir = Hit::download_dir();

        let _ = create_dir_all(download_dir.clone());

        println!("Starting download of missing hits. This may take a while...");

        for hit in get_all().iter() {
            if let Some(id) = hit.yt_id() {
                if !hit.exists() {
                    let options = VideoOptions {
                        quality: VideoQuality::HighestAudio,
                        filter: VideoSearchOptions::Audio,
                        ..Default::default()
                    };
                    if let Ok(video) = Video::new_with_options(id.as_str(), options) {
                        println!(
                            "Download {}: {} to {}.opus",
                            hit.artist.as_str(),
                            hit.title.as_str(),
                            id.as_str()
                        );

                        if video
                            .download(format!("{}/{}.opus", download_dir.as_str(), id))
                            .await
                            .is_ok()
                        {
                            println!("Post-processing opus to mp3...");

                            let in_file = format!("{}/{}.opus", download_dir.as_str(), id);
                            let out_file = format!("{}/{}.mp3", download_dir.as_str(), id);
                            let offset = format!("{}", hit.playback_offset);

                            let builder = FfmpegBuilder::new()
                                .stderr(Stdio::piped())
                                .option(Parameter::Single("nostdin"))
                                .option(Parameter::Single("y"))
                                .input(File::new(in_file.as_str()))
                                .output(
                                    File::new(out_file.as_str())
                                        .option(Parameter::KeyValue("ss", offset.as_str()))
                                        .option(Parameter::Single("vn"))
                                        .option(Parameter::Single("sn"))
                                        .option(Parameter::Single("dn")),
                                );

                            let ffmpeg = builder.run().await.unwrap();

                            ffmpeg.process.wait_with_output().unwrap();

                            remove_file(in_file.as_str()).unwrap();
                        } else {
                            println!("Unable to download video.");
                        }
                    }
                }
            }
        }

        println!("Download finished.");

        Ok(rocket)
    }
}
