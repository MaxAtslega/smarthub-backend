use log::{error, info};
use tokio::sync::{broadcast, oneshot};

use crate::{Config, routes};
use crate::hardware::rfid;

#[tokio::main]
pub async fn launch(conf: &Config) {
    // Print welcome message
    info!("Starting App in {}", conf.app.environment);

    let ident = conf.webserver.ident.clone();
    let address = conf.webserver.address.clone();
    let port = conf.webserver.port.clone();

    let (tx, rx1) = broadcast::channel::<String>(10);
    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    let control_rfid_handle = tokio::task::spawn_blocking(|| {
        if let Err(e) = rfid::control_rfid(tx, shutdown_rx) {
            error!("Failed in control_led: {}", e);
        }
    });

    routes::init(ident, address, port, rx1).await.expect("Failed to start Rocket server");

    // Send shutdown signal
    let _ = shutdown_tx.send(());

    let _ = control_rfid_handle.await;
}

