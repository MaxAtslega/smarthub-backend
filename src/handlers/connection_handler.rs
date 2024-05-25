use std::fs::File;
use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use log::error;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;

use crate::api::AppState;
use crate::enums::system_command::SystemCommand;
use crate::hardware;
use crate::models::websocket::WebSocketMessage;

#[derive(Serialize, Deserialize)]
struct WlanData {
    ssid: String,
    password: String
}

#[derive(Serialize, Deserialize)]
struct UserDeleteData {
    id: i32,
}

#[derive(Serialize, Deserialize)]
struct ConstantData {
    name: String,
}

#[derive(Serialize, Deserialize)]
struct BluetoothDeviceData {
    address: String,
}

pub async fn handle_connection(stream: WebSocket, state: Arc<AppState>) {
    let (mut ws_sender, mut ws_receiver) = stream.split();

    let tx_dbus = state.tx_dbus.clone();
    let tx_dbus2 = tx_dbus.clone();

    tx_dbus2.send(SystemCommand::GetAllBluetoothDevices).await.expect("Failed to send dbus command");

    let mut rx = state.tx.subscribe();

    loop {
        tokio::select! {
            message = rx.recv() => {
                if let Ok(received_notification) = message {
                    if let Ok(json_msg) = serde_json::to_string(&received_notification) {
                        ws_sender.send(Message::Text(json_msg)).await.unwrap();
                    }
                }
            }
            Some(msg) = ws_receiver.next() => {
                if let Ok(message) = msg {
                        if let Ok(text) = message.to_text() {
                            let parsed = serde_json::from_str::<WebSocketMessage>(text);

                            if let Ok(parsed_message) = parsed {
                                    match parsed_message.op {
                                        0 => { // Op code 0 for control commands
                                            if let Some(event) = parsed_message.t {
                                                match event.as_str() {
                                                    "DISPLAY" => {
                                                        let mut bl_power_file = File::create("/sys/class/backlight/10-0045/bl_power").unwrap();
                                                        hardware::display::set_display_power(&mut bl_power_file, false);

                                                        let notification = WebSocketMessage {
                                                            t: Some("DISPLAY_STATUS".to_string()),
                                                            op: 0,
                                                            d: Some(json!({"status": "off"})),
                                                        };
                                                        if let Ok(json_msg) = serde_json::to_string(&notification) {
                                                            ws_sender.send(Message::Text(json_msg)).await.expect("Failed send websocket message");
                                                        }
                                                    },
                                                    "LISTING_UPDATE" => {
                                                        tx_dbus.send(SystemCommand::ListingSystemUpdates).await.expect("Failed to send dbus command");
                                                    },
                                                    "UPDATE" => {
                                                        tx_dbus.send(SystemCommand::UpdateSystem).await.expect("Failed to send dbus command");
                                                    },
                                                    _ => {}
                                                }
                                            }
                                        },
                                        2 => { // Op code 2 for bluetooth
                                            if let Some(event) = parsed_message.t {
                                                match event.as_str() {
                                                    "DEVICES" => {
                                                        tx_dbus.send(SystemCommand::GetAllBluetoothDevices).await.expect("Failed to send dbus command");
                                                    },
                                                    "START_DISCOVERING" => {
                                                            tx_dbus.send(SystemCommand::BluetoothDiscovering("StartDiscovery".to_string())).await.expect("Failed to send dbus command");
                                                    },
                                                    "STOP_DISCOVERING" => {
                                                            tx_dbus.send(SystemCommand::BluetoothDiscovering("StopDiscovery".to_string())).await.expect("Failed to send dbus command");
                                                    },
                                                    "CONNECT" => {
                                                        if let Some(message) = parsed_message.d {
                                                            if let Ok(device) = serde_json::from_value::<BluetoothDeviceData>(message) {
                                                                tx_dbus.send(SystemCommand::ConnectBluetoothDevice(device.address)).await.expect("Failed to send dbus command");
                                                            }
                                                        }
                                                    },
                                                    "DISCONNECT" => {
                                                        if let Some(message) = parsed_message.d {
                                                             if let Ok(device) = serde_json::from_value::<BluetoothDeviceData>(message) {
                                                                tx_dbus.send(SystemCommand::DisconnectBluetoothDevice(device.address)).await.expect("Failed to send dbus command");
                                                            }
                                                        }
                                                    },
                                                    "PAIR" => {
                                                        if let Some(message) = parsed_message.d {
                                                            if let Ok(device) = serde_json::from_value::<BluetoothDeviceData>(message) {
                                                                tx_dbus.send(SystemCommand::PairBluetoothDevice(device.address)).await.expect("Failed to send dbus command");
                                                            }
                                                        }
                                                    },
                                                    "UNPAIR" => {
                                                        if let Some(message) = parsed_message.d {
                                                            if let Ok(device) = serde_json::from_value::<BluetoothDeviceData>(message) {
                                                                tx_dbus.send(SystemCommand::UnpairBluetoothDevice(device.address)).await.expect("Failed to send dbus command");
                                                            }
                                                        }
                                                    },
                                                    "TRUST" => {
                                                        if let Some(message) = parsed_message.d {
                                                            if let Ok(device) = serde_json::from_value::<BluetoothDeviceData>(message) {
                                                                tx_dbus.send(SystemCommand::TrustBluetoothDevice(device.address)).await.expect("Failed to send dbus command");
                                                            }
                                                        }
                                                    },
                                                    "UNTRUST" => {
                                                        if let Some(message) = parsed_message.d {
                                                            if let Ok(device) = serde_json::from_value::<BluetoothDeviceData>(message) {
                                                                tx_dbus.send(SystemCommand::UntrustBluetoothDevice(device.address)).await.expect("Failed to send dbus command");
                                                            }
                                                        }
                                                    },
                                                    _ => {}
                                                }
                                            }
                                        },
                                    _ => {}
                                }
                            }
                    }
                }
            }
        }
    }
}
