use log::{error, info};
use tokio::sync::{broadcast, oneshot};
use tokio::sync::mpsc::{channel, Receiver, Sender};

use crate::{Config, websocket};
use crate::enums::system_command::SystemCommand;
use crate::handlers::system_handler;
use crate::hardware::rfid;
use crate::models::notification_response::NotificationResponse;

#[tokio::main]
pub async fn launch(conf: &Config) {
    // Print welcome message
    info!("Starting App in {}", conf.app.environment);

    let (tx, rx1) = broadcast::channel::<NotificationResponse>(10);
    let (tx_dbus, rx_dbus): (Sender<SystemCommand>, Receiver<SystemCommand>) = channel::<SystemCommand>(32);

    let tx2 = tx.clone();
    let tx3 = tx.clone();

    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    let control_rfid_handle = tokio::task::spawn_blocking(|| {
        if let Err(e) = rfid::test(tx, shutdown_rx) {
            error!("Failed in control_rfid: {}", e);
        }
    });

    let control_bluetooth_handle = tokio::task::spawn_blocking(|| {
        if let Err(e) = system_handler::system_handler(tx2, rx_dbus) {
            error!("Failed in bluetooth_handler: {}", e);
        }
    });

    websocket::init(&conf.websocket, tx3, rx1, tx_dbus).await.expect("Failed to start websocket server");

    // Send shutdown signal
    let _ = shutdown_tx.send(());

    let _ = control_rfid_handle.await;
    let _ = control_bluetooth_handle.await;
}

