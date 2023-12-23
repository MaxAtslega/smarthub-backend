use std::{net::SocketAddr, time::Duration};
use std::net::IpAddr;

use futures_util::{SinkExt, StreamExt};
use log::{error, info};
use serde_derive::{Deserialize, Serialize};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::Receiver;
use tokio_tungstenite::{
    accept_async,
    tungstenite::{Error, Message, Result},
};
use crate::app::Notification;

use crate::hardware::led;

pub async fn init(address: IpAddr, port: u16, rx: Receiver<Notification>) -> Result<(), Error> {
    let addr = format!("{}:{}", address, port);
    let try_socket = TcpListener::bind(&addr).await;

    let listener = try_socket.expect("Failed to bind");
    info!("Listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        let peer = stream.peer_addr().expect("connected streams should have a peer address");
        info!("Peer address: {}", peer);

        let rx_clone = rx.resubscribe(); // Cloning the receiver
        tokio::spawn(accept_connection(peer, stream, rx_clone)); // Passing the cloned receiver
    }

    Ok(())
}


async fn accept_connection(peer: SocketAddr, stream: TcpStream, rx: Receiver<Notification>) {
    if let Err(e) = handle_connection(peer, stream, rx).await {
        match e {
            Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
            err => error!("Error processing connection: {}", err),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct WebSocketMessage {
    t: Option<String>, // Event type
    op: u8,    // Operation code
    d: serde_json::Value, // Data
}

#[derive(Serialize, Deserialize)]
enum LEDType {
    RED,
    GREEN,
    BLUE,
}


#[derive(Serialize, Deserialize)]
struct LEDControlData {
    color: LEDType,
}

#[derive(Serialize, Deserialize)]
struct NotificationData {
    message: String,
    timestamp: String,
}

async fn handle_connection(peer: SocketAddr, stream: TcpStream, mut rx: Receiver<Notification>) -> Result<()> {
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
                                                        flash_led(led_data.color).await;
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


async fn flash_led(led_type: LEDType) {
    match led_type {
        LEDType::RED => {
            led::flash_led(led::LED_RED_PIN).await.expect("Failed to flash LED");
        },
        LEDType::GREEN => {
            led::flash_led(led::LED_GREEN_PIN).await.expect("Failed to flash LED");
        },
        LEDType::BLUE => {
            led::flash_led(led::LED_BLUE_PIN).await.expect("Failed to flash LED");
        },
    }
}