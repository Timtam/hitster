use ffmpeg_cli::{FfmpegBuilder, File, Parameter};
use regex::Regex;
use rocket::{
    fairing::{Fairing, Info, Kind},
    Orbit, Rocket,
};
use rusty_ytdl::{Video, VideoOptions, VideoQuality, VideoSearchOptions};
use std::{
    env,
    fs::{create_dir_all, remove_file},
    path::Path,
    process::Stdio,
    str::FromStr,
};
use strum::EnumString;

include!(concat!(env!("OUT_DIR"), "/hits.rs"));

#[derive(EnumString, Eq, PartialEq, Debug)]
pub enum Pack {
    Basic,
    Schlagerparty,
}

pub struct Hit {
    pub interpret: String,
    pub title: String,
    pub year: u32,
    pub pack: Pack,
    pub yt_url: String,
    pub playback_offset: u16,
}

#[derive(Default)]
pub struct HitsterDownloader {}

#[rocket::async_trait]
impl Fairing for HitsterDownloader {
    fn info(&self) -> Info {
        Info {
            kind: Kind::Liftoff,
            name: "Download hits",
        }
    }

    async fn on_liftoff(&self, _rocket: &Rocket<Orbit>) {
        let yt_id: Regex = Regex::new(
            r"^.*((youtu.be\/)|(v\/)|(\/u\/\w\/)|(embed\/)|(watch\?))\??v?=?([^#&?]*).*",
        )
        .unwrap();

        let download_dir = env::var("DOWNLOAD_DIRECTORY").unwrap_or("./hits".to_string());

        let _ = create_dir_all(download_dir.clone());

        println!("Starting download of missing hits. This may take a while...");

        for hit in get_all().iter() {
            if let Some(caps) = yt_id.captures(hit.yt_url.as_str()) {
                let id = &caps[7];

                if !Path::new(&download_dir)
                    .join(format!("{}.mp3", id))
                    .is_file()
                {
                    let options = VideoOptions {
                        quality: VideoQuality::HighestAudio,
                        filter: VideoSearchOptions::Audio,
                        ..Default::default()
                    };
                    let video = Video::new_with_options(id, options).unwrap();

                    println!(
                        "Download {}: {} to {}.opus",
                        hit.interpret.as_str(),
                        hit.title.as_str(),
                        id
                    );

                    video
                        .download(format!("{}/{}.opus", download_dir.as_str(), id))
                        .await
                        .unwrap();

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
                }
            }
        }

        println!("Download finished.");
    }
}
