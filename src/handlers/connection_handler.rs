use std::{io, net::SocketAddr};
use std::process::Command;

use futures_util::{SinkExt, StreamExt};
use log::{error, info};
use serde_derive::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc::Sender;
use tokio_tungstenite::{
    accept_async,
    tungstenite::{Message, Result}};

use crate::enums::dbus_command::DbusCommand;
use crate::enums::led_type::LEDType;
use crate::hardware::led;
use crate::models::notification_response::NotificationResponse;
use crate::models::websocket_message::WebSocketMessage;

#[derive(Serialize, Deserialize)]
struct LEDControlData {
    color: LEDType,
}

#[derive(Serialize, Deserialize)]
struct BluetoothDeviceData {
    address: String,
}

pub async fn handle_connection(peer: SocketAddr, stream: TcpStream, mut rx: Receiver<NotificationResponse>, tx_dbus: Sender<DbusCommand>) -> Result<()> {
    let ws_stream = accept_async(stream).await.expect("Failed to accept");
    info!("New WebSocket connection: {}", peer);
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    let tx_dbus2 = tx_dbus.clone();
    tx_dbus2.send(DbusCommand::GetAllBluetoothDevices).await.expect("Failed to send dbus command");
    tx_dbus2.send(DbusCommand::GetCurrentNetwork).await.expect("Failed to send dbus command");

    loop {
        tokio::select! {
            message = rx.recv() => {
                if let Ok(received_notification) = message {
                    let ws_message = WebSocketMessage {
                        op: received_notification.op,
                        t: Some(received_notification.title),
                        d: received_notification.data,
                    };
                    if let Ok(json_msg) = serde_json::to_string(&ws_message) {
                        ws_sender.send(Message::Text(json_msg)).await?;
                    }
                }
            }
            Some(msg) = ws_receiver.next() => {
                if let Ok(message) = msg {
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
                                    1 => {}, // Op code 1 for notifications
                                    2 => { // Op code 2 for bluetooth
                                        if let Some(event) = parsed_message.t {
                                            match event.as_str() {
                                                "START_DISCOVERING" => {
                                                        info!("Starting bluetooth scanning");
                                                        tx_dbus.send(DbusCommand::BluetoothDiscovering("StartDiscovery".to_string())).await.expect("Failed to send dbus command");
                                                },
                                                "STOP_DISCOVERING" => {
                                                        info!("Stopping bluetooth scanning");
                                                        tx_dbus.send(DbusCommand::BluetoothDiscovering("StopDiscovery".to_string())).await.expect("Failed to send dbus command");
                                                },
                                                "CONNECT" => {
                                                    if let Ok(device) = serde_json::from_value::<BluetoothDeviceData>(parsed_message.d) {
                                                            tx_dbus.send(DbusCommand::ConnectBluetoothDevice(device.address)).await.expect("Failed to send dbus command");
                                                    }
                                                },
                                                "DISCONNECT" => {
                                                    if let Ok(device) = serde_json::from_value::<BluetoothDeviceData>(parsed_message.d) {
                                                            tx_dbus.send(DbusCommand::DisconnectBluetoothDevice(device.address)).await.expect("Failed to send dbus command");
                                                    }
                                                },
                                                "PAIR" => {
                                                    if let Ok(device) = serde_json::from_value::<BluetoothDeviceData>(parsed_message.d) {
                                                            tx_dbus.send(DbusCommand::PairBluetoothDevice(device.address)).await.expect("Failed to send dbus command");
                                                    }
                                                },
                                                "UNPAIR" => {
                                                    if let Ok(device) = serde_json::from_value::<BluetoothDeviceData>(parsed_message.d) {
                                                            tx_dbus.send(DbusCommand::UnpairBluetoothDevice(device.address)).await.expect("Failed to send dbus command");
                                                    }
                                                },
                                                "TRUST" => {
                                                    if let Ok(device) = serde_json::from_value::<BluetoothDeviceData>(parsed_message.d) {
                                                            tx_dbus.send(DbusCommand::TrustBluetoothDevice(device.address)).await.expect("Failed to send dbus command");
                                                    }
                                                },
                                                "UNTRUST" => {
                                                    if let Ok(device) = serde_json::from_value::<BluetoothDeviceData>(parsed_message.d) {
                                                            tx_dbus.send(DbusCommand::UntrustBluetoothDevice(device.address)).await.expect("Failed to send dbus command");
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
