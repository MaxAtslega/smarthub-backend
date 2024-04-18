use std::sync::{Arc, Mutex};
use std::time::Instant;
use log::{error, info};
use tokio::sync::{broadcast, oneshot};
use tokio::sync::mpsc::{channel, Receiver, Sender};

use crate::{Config, hardware, websocket};
use crate::common::db;
use crate::enums::system_command::SystemCommand;
use crate::handlers::system_handler;
use crate::hardware::rfid;
use crate::models::websocket::WebSocketMessage;

#[tokio::main]
pub async fn launch(conf: &Config) {
    // Print welcome message
    info!("Starting App in {}", conf.app.environment);

    let db_connection = db::establish_connection_pool(&conf.database.connection_string);

    // Messaging setup for WebSocket and system handlers
    let (tx, rx1) = broadcast::channel::<WebSocketMessage>(10);
    let (tx_dbus, rx_dbus): (Sender<SystemCommand>, Receiver<SystemCommand>) = channel::<SystemCommand>(32);
   
    // Clone tx for multiple uses
    let tx1 = tx.clone();
    let tx2 = tx.clone();
    let tx3 = tx.clone();
    
    // Shared state across threads
    let last_event_time = Arc::new(Mutex::new(Instant::now()));
    let last_event_time_clone = Arc::clone(&last_event_time);

    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    // Launch hardware handlers in separate threads
    std::thread::spawn(|| {
        if let Err(e) = rfid::control_rfid(tx, shutdown_rx, last_event_time) {
            error!("Failed in control_rfid: {}", e);
        }
    });
    
    // Launch system handler
    std::thread::spawn(|| {
        if let Err(e) = hardware::display::display_handler_sleep(tx1, last_event_time_clone) {
            error!("Failed in systemd handler sleep: {}", e);
        }
    });
    
    // Initialize and run the WebSocket server
    std::thread::spawn(|| {
        if let Err(e) = system_handler::system_handler(tx2, rx_dbus) {
            error!("Failed in bluetooth_handler: {}", e);
        }
    });

    if let Err(e) = websocket::init(&conf.websocket, tx3, rx1, tx_dbus, db_connection).await {
        error!("WebSocket initialization failed: {}", e);
    }

    // Send shutdown signal
    let _ = shutdown_tx.send(());
}

