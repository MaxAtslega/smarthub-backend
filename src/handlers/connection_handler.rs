use std::{io, net::SocketAddr, time::Duration};
use std::net::IpAddr;
use std::process::Command;

use futures_util::{SinkExt, StreamExt};
use log::{error, info};
use serde_derive::{Deserialize, Serialize};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::Receiver;
use tokio_tungstenite::{
    accept_async,
    tungstenite::{Error, Message, Result}};

use crate::enums::led_type::LEDType;
use crate::models::websocket_message::WebSocketMessage;
use crate::models::notification_data::NotificationData;
use crate::hardware;
use crate::hardware::led;
use crate::models::notification_response::NotificationResponse;

#[derive(Serialize, Deserialize)]
struct LEDControlData {
    color: LEDType,
}


pub(crate) async fn handle_connection(peer: SocketAddr, stream: TcpStream, mut rx: Receiver<NotificationResponse>) -> Result<()> {
    let ws_stream = accept_async(stream).await.expect("Failed to accept");
    info!("New WebSocket connection: {}", peer);

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    loop {
        tokio::select! {
            message = rx.recv() => {
                if let Ok(received_notification) = message {
                    let notification = NotificationData {
                        message: received_notification.data,
                        timestamp: format!("{}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")),
                    };
                    let ws_message = WebSocketMessage {
                        op: 1, // Op code 1 for notifications
                        t: Some(received_notification.title),
                        d: serde_json::to_value(notification).unwrap(),
                    };
                    if let Ok(json_msg) = serde_json::to_string(&ws_message) {
                        ws_sender.send(Message::Text(json_msg)).await?;
                    }
                }
            }
            Some(msg) = ws_receiver.next() => {
                let message = msg?;

                if message.is_text() {
                    if let Ok(text) = message.to_text() {
                        let parsed = serde_json::from_str::<WebSocketMessage>(text);

                        match parsed {
                            Ok(parsed_message) => {
                                match parsed_message.op {
                                    0 => { // Op code 0 for control commands
                                        if let Some(event) = parsed_message.t {
                                            match event.as_str() {
                                                "FLASH_LED" => {
                                                    if let Ok(led_data) = serde_json::from_value::<LEDControlData>(parsed_message.d) {
                                                        led::flash_led(led_data.color).await.expect("Failed to flash LED");
                                                    }
                                                },
                                                "REBOOT" => {
                                                    if let Err(e) = reboot_system() {
                                                        error!("Failed to reboot: {}", e);
                                                    }
                                                },
                                                _ => {}
                                            }
                                        }
                                    },
                                    _ => {}
                                }
                            }
                        Err(_) => {}}
                    }
                } else if message.is_close() {
                    break;
                }
            }
        }
    }

    Ok(())
}


fn reboot_system() -> io::Result<()> {
    println!("Rebooting system...");
    Command::new("sudo")
        .arg("reboot")
        .status()?;

    Ok(())
}
