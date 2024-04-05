mod games;
mod hits;
mod responses;
mod routes;
mod services;
mod users;

use dotenvy::dotenv;
use games::GameEvent;
use hits::HitsterDownloader;
use rocket::{
    fairing::{self, AdHoc},
    figment::{util::map, Figment},
    fs::NamedFile,
    response::Redirect,
    tokio::sync::broadcast::channel,
    Build, Config, Rocket,
};
use rocket_db_pools::{sqlx, Database};
use rocket_okapi::{openapi_get_routes, rapidoc::*, settings::UrlObject, swagger_ui::*};
use routes::{games as games_routes, users as users_routes};
use services::ServiceStore;
use std::{
    env,
    path::{Path, PathBuf},
};

#[macro_use]
extern crate rocket;

#[derive(Database)]
#[database("hitster_config")]
struct HitsterConfig(sqlx::SqlitePool);

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    match HitsterConfig::fetch(&rocket) {
        Some(db) => match sqlx::migrate!().run(&**db).await {
            Ok(_) => Ok(rocket),
            Err(e) => {
                error!("Failed to run database migrations: {}", e);
                Err(rocket)
            }
        },
        None => Err(rocket),
    }
}

#[get("/")]
async fn index() -> std::io::Result<NamedFile> {
    let page_directory_path = env::var("CLIENT_DIRECTORY").unwrap_or(format!(
        "{}/../client/dist",
        env::var("CARGO_MANIFEST_DIR").unwrap_or("./".to_string())
    ));
    NamedFile::open(Path::new(&page_directory_path).join("index.html")).await
}

#[get("/")]
async fn api_index() -> Redirect {
    Redirect::to("/swagger-ui")
}

#[get("/<file..>")]
async fn files(file: PathBuf) -> std::io::Result<NamedFile> {
    let page_directory_path = env::var("CLIENT_DIRECTORY").unwrap_or(format!(
        "{}/../client/dist",
        env::var("CARGO_MANIFEST_DIR").unwrap_or("./".to_string())
    ));
    NamedFile::open(Path::new(&page_directory_path).join(file))
        .await
        .or(NamedFile::open(Path::new(&page_directory_path).join("index.html")).await)
}

fn rocket_from_config(figment: Figment) -> Rocket<Build> {
    let migrations_fairing = AdHoc::try_on_ignite("SQLx Migrations", run_migrations);

    rocket::custom(figment)
        .attach(HitsterConfig::init())
        .attach(migrations_fairing)
        .attach(HitsterDownloader::default())
        .mount("/", routes![index, files,])
        .mount("/api/", routes![api_index, games_routes::events])
        .mount(
            "/api/",
            openapi_get_routes![
                users_routes::get_all_users,
                users_routes::get_user,
                users_routes::login,
                users_routes::logout,
                users_routes::signup,
                games_routes::confirm_slot,
                games_routes::create_game,
                games_routes::get_all_games,
                games_routes::get_game,
                games_routes::guess_slot,
                games_routes::hit,
                games_routes::join_game,
                games_routes::leave_game,
                games_routes::skip_hit,
                games_routes::start_game,
                games_routes::stop_game,
            ],
        )
        .mount(
            "/swagger-ui/",
            make_swagger_ui(&SwaggerUIConfig {
                url: "../api/openapi.json".to_owned(),
                ..Default::default()
            }),
        )
        .mount(
            "/rapidoc/",
            make_rapidoc(&RapiDocConfig {
                general: GeneralConfig {
                    spec_urls: vec![UrlObject::new("General", "../api/openapi.json")],
                    ..Default::default()
                },
                hide_show: HideShowConfig {
                    allow_spec_url_load: false,
                    allow_spec_file_load: false,
                    ..Default::default()
                },
                ..Default::default()
            }),
        )
        .manage(ServiceStore::default())
        .manage(channel::<GameEvent>(1024).0)
}

#[launch]
fn rocket() -> _ {
    let _ = dotenv();

    rocket_from_config(
        Config::figment()
            .merge((
                "databases",
                map![
                "hitster_config" => map![
                "url" => env::var("DATABASE_URL").expect("DATABASE_URL required"),
                ],
                    ],
            ))
            .merge((
                "secret_key",
                env::var("SECRET_KEY").expect("SECRET_KEY is required"),
            )),
    )
}

#[cfg(test)]
mod test {
    use super::rocket_from_config;
    use rocket::{
        figment::{util::map, value::Map},
        local::asynchronous::Client,
    };

    pub async fn mocked_client() -> Client {
        let db_config: Map<_, String> = map! {
          "url" => "sqlite::memory:".into(),
        };

        let figment =
            rocket::Config::figment().merge(("databases", map!["hitster_config" => db_config]));

        let client = Client::tracked(rocket_from_config(figment))
            .await
            .expect("valid rocket instance");

        return client;
    }
}
