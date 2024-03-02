mod games;
mod hits;
mod responses;
mod routes;
mod services;
mod users;

use dotenvy::dotenv;
use rocket::{
    fairing::{self, AdHoc},
    figment::{util::map, Figment},
    response::Redirect,
    Build, Config, Rocket,
};
use rocket_db_pools::{sqlx, Database};
use rocket_okapi::{openapi_get_routes, rapidoc::*, settings::UrlObject, swagger_ui::*};
use routes::{games as games_routes, users as users_routes};
use services::{GameService, HitService, UserService};
use std::env;

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
fn index() -> Redirect {
    Redirect::to("/swagger-ui")
}

fn rocket_from_config(figment: Figment) -> Rocket<Build> {
    let migrations_fairing = AdHoc::try_on_ignite("SQLx Migrations", run_migrations);

    rocket::custom(figment)
        .attach(HitsterConfig::init())
        .attach(migrations_fairing)
        .mount("/", routes![index,])
        .mount(
            "/",
            openapi_get_routes![
                users_routes::get_all_users,
                users_routes::get_user,
                users_routes::user_login,
                users_routes::user_logout,
                users_routes::user_signup,
                games_routes::create_game,
                games_routes::get_all_games
            ],
        )
        .mount(
            "/swagger-ui/",
            make_swagger_ui(&SwaggerUIConfig {
                url: "../openapi.json".to_owned(),
                ..Default::default()
            }),
        )
        .mount(
            "/rapidoc/",
            make_rapidoc(&RapiDocConfig {
                general: GeneralConfig {
                    spec_urls: vec![UrlObject::new("General", "../openapi.json")],
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
        .manage(GameService::new())
        .manage(HitService::new())
        .manage(UserService::new())
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
