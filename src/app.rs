use log::{error, info};
use tokio::sync::{broadcast, oneshot};

use crate::{Config, websocket};
use crate::hardware::rfid;
use crate::models::notification_response::NotificationResponse;

#[tokio::main]
pub async fn launch(conf: &Config) {
    // Print welcome message
    info!("Starting App in {}", conf.app.environment);

    let (tx, rx1) = broadcast::channel::<NotificationResponse>(10);
    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    let control_rfid_handle = tokio::task::spawn_blocking(|| {
        if let Err(e) = rfid::control_rfid(tx, shutdown_rx) {
            error!("Failed in control_led: {}", e);
        }
    });

    websocket::init(&conf.websocket, rx1).await.expect("Failed to start websocket server");

    // Send shutdown signal
    let _ = shutdown_tx.send(());

    let _ = control_rfid_handle.await;
}

