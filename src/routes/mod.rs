use rocket::{Error, Ignite, Rocket};
use crate::api;
use crate::config::Config;

mod catcher;

pub async fn init(conf: &Config) -> Result<Rocket<Ignite>, Error> {
    let conf = &conf.webserver;

    let figment = rocket::Config::figment()
        .merge(("port", &conf.port))
        .merge(("address", &conf.address))
        .merge(("ident", &conf.ident))
        .merge(("log_level", "Off"));

    let rocket = rocket::custom(figment)
        .register("/",catchers![
            catcher::bad_request,
            catcher::unauthorized,
            catcher::forbidden,
            catcher::not_found,
            catcher::not_implemented,
            catcher::internal_error,
            catcher::unprocessable_entity,
        ])
        .mount("/api/v3", routes![api::info::get_info, api::websocket::echo_stream])
        .ignite()
        .await?;

    let result = rocket.launch().await;
    println!("The server shutdown: {:?}", result);

    result
}