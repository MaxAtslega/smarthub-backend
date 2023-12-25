use log::{error, info};
use tokio::sync::{broadcast, oneshot};
use tokio::sync::mpsc::{channel, Receiver, Sender};

use crate::{Config, websocket};
use crate::handlers::bluetooth_handler;
use crate::hardware::rfid;
use crate::models::notification_response::NotificationResponse;

pub(crate) enum DbusCommand {
    BLUEZ(String),
}

#[tokio::main]
pub async fn launch(conf: &Config) {
    // Print welcome message
    info!("Starting App in {}", conf.app.environment);

    let (tx, rx1) = broadcast::channel::<NotificationResponse>(10);
    let (tx_dbus, mut rx_dbus): (Sender<DbusCommand>, Receiver<DbusCommand>) = channel::<DbusCommand>(32);
    let tx2 = tx.clone();

    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    let control_rfid_handle = tokio::task::spawn_blocking(|| {
        if let Err(e) = rfid::control_rfid(tx, shutdown_rx) {
            error!("Failed in control_rfid: {}", e);
        }
    });

    let control_bluetooth_handle = tokio::task::spawn_blocking(|| {
        if let Err(e) = bluetooth_handler::listening(tx2, rx_dbus) {
            error!("Failed in bluetooth_handler: {}", e);
        }
    });

    websocket::init(&conf.websocket, rx1, tx_dbus).await.expect("Failed to start websocket server");

    // Send shutdown signal
    let _ = shutdown_tx.send(());

    let _ = control_rfid_handle.await;
    let _ = control_bluetooth_handle.await;
}

