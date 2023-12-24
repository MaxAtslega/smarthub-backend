#[macro_use]
extern crate diesel;
extern crate dbus;

use config::Config;

mod app;
mod models;
mod config;
mod log;
mod websocket;
mod hardware;
mod common;
mod enums;
mod handlers;

fn main() {
    let conf = Config::from_any().unwrap();

    // Setup simplelog
    log::setup(&conf.log);

    app::launch(&conf);
}
