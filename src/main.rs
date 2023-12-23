#[macro_use]
extern crate diesel;
#[macro_use]
extern crate rocket;

use config::Config;

mod app;
mod models;
mod config;
mod log;
mod routes;
mod api;
mod hardware;

fn main() {
    let conf = Config::from_any().unwrap();

    // Setup simplelog
    log::setup(&conf.log);

    app::launch(&conf);
}
