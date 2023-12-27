#![feature(ptr_metadata)]
extern crate dbus;

#[macro_use]
extern crate diesel;

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
mod network;
mod schema;

fn main() {
    let conf = Config::from_any().unwrap();

    // Setup simplelog
    log::setup(&conf.log);

    app::launch(&conf);
}
