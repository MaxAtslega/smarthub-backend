use std::sync::Arc;
use crate::{Config, routes};
use crate::hardware::rfid;
use log::{info, error};
use rocket::{Ignite, Rocket, Error};
use tokio::sync::{broadcast, oneshot};

pub fn launch(conf: &Config) {
    // Print welcome message
    info!("Starting App in {}", conf.app.environment);

    let ident = conf.webserver.ident.clone();
    let address = conf.webserver.address.clone();
    let port = conf.webserver.port.clone();

    let (tx, rx1) = broadcast::channel::<String>(10);
    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    // Build a multi-threaded Tokio runtime
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .thread_name("rocket-worker-thread")
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime");

    runtime.block_on(async {
        let control_rfid_handle = tokio::spawn(async move {
            if let Err(e) = rfid::control_rfid(tx, shutdown_rx).await {
                error!("Failed in control_led: {}", e);
            }
        });

        routes::init(ident, address, port, rx1).await;

        // Wait for CTRL+C
        tokio::signal::ctrl_c().await.expect("Failed to listen for CTRL+C");

        // Send shutdown signal
        let _ = shutdown_tx.send(());

        // Optionally, wait for the `control_rfid` task to finish
        let _ = control_rfid_handle.await;
    })
}

