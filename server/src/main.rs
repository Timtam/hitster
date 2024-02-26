mod games;
mod hits;
mod responses;
mod routes;
mod services;
mod users;

use rocket::response::Redirect;
use rocket_okapi::{openapi_get_routes, rapidoc::*, settings::UrlObject, swagger_ui::*};
use routes::{games as games_routes, users as users_routes};
use services::{GameService, HitService, UserService};

#[macro_use]
extern crate rocket;

#[get("/")]
fn index() -> Redirect {
    Redirect::to("/swagger-ui")
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index,])
        .mount(
            "/",
            openapi_get_routes![
                users_routes::create_user,
                users_routes::get_all_users,
                users_routes::get_user,
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
