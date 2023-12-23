#[macro_use]
extern crate rocket;

#[macro_use]
extern crate diesel;

mod app;
mod models;
mod config;
mod log;
mod routes;
mod api;
mod hardware;

use config::Config;

fn main() {
    let conf = Config::from_any().unwrap();

    // Setup simplelog
    log::setup(&conf.log);

    app::launch(&conf);
}
