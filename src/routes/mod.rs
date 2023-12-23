use std::net::IpAddr;

use rocket::{Error, Ignite, Rocket};
use tokio::sync::broadcast::Receiver;

use crate::api;

mod catcher;

pub struct SharedChannel {
    pub receiver: Receiver<String>,
}

pub async fn init(ident: String, address: IpAddr, port: u16, rx: Receiver<String>) -> Result<Rocket<Ignite>, Error> {
    let figment = rocket::Config::figment()
        .merge(("port", port))
        .merge(("address", address))
        .merge(("ident", ident))
        .merge(("log_level", "Off"));

    let rocket = rocket::custom(figment)
        .register("/", catchers![
            catcher::bad_request,
            catcher::unauthorized,
            catcher::forbidden,
            catcher::not_found,
            catcher::not_implemented,
            catcher::internal_error,
            catcher::unprocessable_entity,
        ])
        .manage(SharedChannel { receiver: rx })
        .mount("/", routes![api::info::get_info, api::led::flash_blue, api::led::flash_green, api::led::flash_red, api::websocket::echo_stream])
        .ignite()
        .await?;

    let result = rocket.launch().await;
    println!("The server shutdown: {:?}", result);

    result
}