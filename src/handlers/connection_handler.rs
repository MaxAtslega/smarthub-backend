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

use crate::enums::system_command::SystemCommand;
use crate::enums::led_type::LEDType;
use crate::hardware::led;
use crate::models::websocket::WebSocketMessage;

#[derive(Serialize, Deserialize)]
struct LEDControlData {
    color: LEDType,
}

#[derive(Serialize, Deserialize)]
struct WlanData {
    ssid: String,
    password: String
}

#[derive(Serialize, Deserialize)]
struct BluetoothDeviceData {
    address: String,
}

pub async fn handle_connection(peer: SocketAddr, stream: TcpStream, tx: tokio::sync::broadcast::Sender<WebSocketMessage>, mut rx: Receiver<WebSocketMessage>, tx_dbus: Sender<SystemCommand>) -> Result<()> {
    let ws_stream = accept_async(stream).await.expect("Failed to accept");
    info!("New WebSocket connection: {}", peer);
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    let tx_dbus2 = tx_dbus.clone();
    tx_dbus2.send(SystemCommand::GetAllBluetoothDevices).await.expect("Failed to send dbus command");
    tx_dbus2.send(SystemCommand::GetNetworkInterfaces).await.expect("Failed to send dbus command");

    loop {
        tokio::select! {
            message = rx.recv() => {
                if let Ok(received_notification) = message {
                    if let Ok(json_msg) = serde_json::to_string(&received_notification) {
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
                                                        if let Some(message) = parsed_message.d {
                                                            if let Ok(led_data) = serde_json::from_value::<LEDControlData>(message) {
                                                                led::flash_led(led_data.color).await.expect("Failed to flash LED");
                                                            }
                                                        }
                                                    },
                                                    "REBOOT" => {
                                                        if let Err(e) = reboot_system() {
                                                            error!("Failed to reboot: {}", e);
                                                        }
                                                    },
                                                    "LISTING_UPDATE" => {
                                                        tx_dbus.send(SystemCommand::ListingSystemUpdates).await.expect("Failed to send dbus command");
                                                    },
                                                    "UPDATE" => {
                                                        tx_dbus.send(SystemCommand::UpdateSystem).await.expect("Failed to send dbus command");
                                                    }
                                                    "NETWORK_INTERFACES" => {
                                                        tx_dbus.send(SystemCommand::GetNetworkInterfaces).await.expect("Failed to send dbus command");
                                                    },
                                                    "SCAN_WIFI" => {
                                                        tx_dbus.send(SystemCommand::WlanScan).await.expect("Failed to send dbus command");
                                                    },
                                                    "CONNECT_WIFI" => {
                                                        if let Some(message) = parsed_message.d {
                                                            if let Ok(wifi_data) = serde_json::from_value::<WlanData>(message) {
                                                                tx_dbus.send(SystemCommand::ConnectWifi(wifi_data.ssid, wifi_data.password )).await.expect("Failed to send dbus command");
                                                            }
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
                                                            tx_dbus.send(SystemCommand::BluetoothDiscovering("StartDiscovery".to_string())).await.expect("Failed to send dbus command");
                                                    },
                                                    "STOP_DISCOVERING" => {
                                                            info!("Stopping bluetooth scanning");
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
                            Err(_) => {}
                        }
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
