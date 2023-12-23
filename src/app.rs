use std::sync::Arc;
use crate::{Config, routes};
use crate::hardware::led;
use log::{info, error};
use rocket::{Ignite, Rocket, Error};
use tokio::sync::broadcast;

pub fn launch(conf: &Config) -> Result<Rocket<Ignite>, Error> {
    // Print welcome message
    info!("Starting App in {}", conf.app.environment);

    let ident = conf.webserver.ident.clone();
    let address = conf.webserver.address.clone();
    let port = conf.webserver.port.clone();

    let (tx, rx1) = broadcast::channel::<String>(10);

    // Build a multi-threaded Tokio runtime
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .thread_name("rocket-worker-thread")
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime");

    runtime.block_on(async {
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            if let Err(e) = led::control_led(tx_clone).await {
                error!("Failed in control_led: {}", e);
            }
        });

        routes::init(ident, address, port, rx1).await
    })
}

