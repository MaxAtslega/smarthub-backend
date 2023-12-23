use log::{error, info};
use serde_derive::{Deserialize, Serialize};
use tokio::sync::{broadcast, oneshot};

use crate::{Config, websocket};
use crate::hardware::rfid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Notification {
    pub(crate) title: String,
    pub(crate) data: String,
}

#[tokio::main]
pub async fn launch(conf: &Config) {
    // Print welcome message
    info!("Starting App in {}", conf.app.environment);

    let address = conf.webserver.address.clone();
    let port = conf.webserver.port.clone();

    let (tx, rx1) = broadcast::channel::<Notification>(10);
    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    let control_rfid_handle = tokio::task::spawn_blocking(|| {
        if let Err(e) = rfid::control_rfid(tx, shutdown_rx) {
            error!("Failed in control_led: {}", e);
        }
    });

    websocket::init(address, port, rx1).await.expect("Failed to start websocket server");

    // Send shutdown signal
    let _ = shutdown_tx.send(());

    let _ = control_rfid_handle.await;
}

