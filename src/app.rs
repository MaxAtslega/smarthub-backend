use std::sync::Arc;
use crate::{Config, routes};
use crate::hardware::led::control_led;
use log::{info, error};
use tokio::sync::broadcast;

pub fn launch(conf: &Config) {
    // Print welcome message
    info!("Starting App in {}", conf.app.environment);

    let ident = conf.webserver.ident.clone();
    let address = conf.webserver.address.clone();
    let port = conf.webserver.port.clone();

    let (tx, rx1) = broadcast::channel::<String>(10);

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .thread_name("rocket-worker-thread")
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime");

    // Run the async block within the Tokio runtime
    runtime.block_on(async {
        // Spawn the task for controlling the LED
        tokio::spawn(async move {
            if let Err(e) = control_led(tx).await {
                error!("Failed in control_led: {}", e);
            }
        });

        // Spawn the Rocket server as a separate async task
        let rocket = tokio::spawn(async move {
            routes::init(ident, address, port, rx1).await.unwrap()
        });

        // Wait for both tasks to complete
        let _ = tokio::join!(rocket);
    });
}

