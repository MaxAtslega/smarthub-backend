use std::{io, net::SocketAddr};
use std::fs::File;
use std::process::Command;

use futures_util::{SinkExt, StreamExt};
use futures_util::stream::{SplitSink, SplitStream};
use log::{debug, error, info};
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::net::TcpStream;
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc::Sender;
use tokio_tungstenite::{accept_async, tungstenite::{Message, Result}, WebSocketStream};
use crate::common::db::DatabasePool;

use crate::enums::system_command::SystemCommand;
use crate::enums::led_type::LEDType;
use crate::handlers::{database_handler, system_handler};
use crate::handlers::system_handler::system_handler;
use crate::hardware;
use crate::hardware::led;
use crate::models::constants::Constant;
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
struct UserDeleteData {
    id: i32,
}

#[derive(Serialize, Deserialize)]
struct ConstantData {
    name: String,
}

#[derive(Deserialize, Serialize)]
pub struct UserChangeData {
    pub id: i32,
    pub username: Option<String>,
    pub birthday: Option<chrono::NaiveDate>, // Nested Option to allow clearing the date
    pub theme: Option<i32>,
    pub language: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct BluetoothDeviceData {
    address: String,
}

pub async fn handle_connection(peer: SocketAddr, stream: TcpStream, _tx: tokio::sync::broadcast::Sender<WebSocketMessage>, mut rx: Receiver<WebSocketMessage>, tx_dbus: Sender<SystemCommand>, database_pool: DatabasePool) -> Result<()> {
    let ws_stream = accept_async(stream).await.expect("Failed to accept");
    info!("New WebSocket connection: {}", peer);
    let (mut ws_sender, mut ws_receiver): (SplitSink<WebSocketStream<TcpStream>, Message>, SplitStream<WebSocketStream<TcpStream>>) = ws_stream.split();

    let tx_dbus2 = tx_dbus.clone();
    tx_dbus2.send(SystemCommand::GetAllBluetoothDevices).await.expect("Failed to send dbus command");
    tx_dbus2.send(SystemCommand::GetNetworkInterfaces).await.expect("Failed to send dbus command");

    database_handler::get_users(&database_pool, &mut ws_sender).await;
    database_handler::get_contants(&database_pool, &mut ws_sender).await;

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
                                                    "DISPLAY" => {
                                                        let mut bl_power_file = File::create("/sys/class/backlight/10-0045/bl_power").unwrap();
                                                        hardware::display::set_display_power(&mut bl_power_file, false);

                                                        let notification = WebSocketMessage {
                                                            t: Some("DISPLAY_STATUS".to_string()),
                                                            op: 0,
                                                            d: Some(json!({"status": "off"})),
                                                        };
                                                        if let Ok(json_msg) = serde_json::to_string(&notification) {
                                                            ws_sender.send(Message::Text(json_msg)).await?;
                                                        }
                                                    },
                                                    "REBOOT" => {
                                                        if let Err(e) = reboot_system() {
                                                            error!("Failed to reboot: {}", e);
                                                        }
                                                    },
                                                     "SHUTDOWN" => {
                                                        if let Err(e) = shutdown_system() {
                                                            error!("Failed to shutdown: {}", e);
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
                                                    "CONSTANTS" => {
                                                        database_handler::get_contants(&database_pool, &mut ws_sender).await;
                                                    },
                                                    "USERS" => {
                                                        database_handler::get_users(&database_pool, &mut ws_sender).await;
                                                    },
                                                    "CREATE_USER" => {
                                                        if let Some(message) = parsed_message.d {
                                                            let user = serde_json::from_value::<crate::models::user::NewUser>(message);
                                                            if let Ok(user_data) = user {
                                                                database_handler::create_user(&database_pool, user_data, &mut ws_sender).await;
                                                            } else {
                                                                error!("Error: {:?}", user.unwrap_err())
                                                            }
                                                        }
                                                    },
                                                    "DELETE_USER" => {
                                                        if let Some(message) = parsed_message.d {
                                                            if let Ok(user_data) = serde_json::from_value::<UserDeleteData>(message) {
                                                                database_handler::delete_user(&database_pool, user_data.id, &mut ws_sender).await;
                                                            }
                                                        }
                                                    },
                                                    "UPDATE_USER" => {
                                                        if let Some(message) = parsed_message.d {
                                                            if let Ok(user_data) = serde_json::from_value::<UserChangeData>(message) {
                                                                database_handler::update_user(&database_pool,
                                                                    user_data.id,
                                                                    user_data.username,
                                                                    user_data.birthday,
                                                                    user_data.theme,
                                                                    user_data.language, &mut ws_sender).await;
                                                            }
                                                        }
                                                    },
                                                    "SET_CONSTANT" => {
                                                        if let Some(message) = parsed_message.d {
                                                            if let Ok(constant_data) = serde_json::from_value::<crate::models::constants::NewConstant>(message) {
                                                                database_handler::set_constant(&database_pool, constant_data.name, constant_data.value, &mut ws_sender).await;
                                                            }
                                                        }
                                                    },
                                                    "DELETE_CONSTANT" => {
                                                        if let Some(message) = parsed_message.d {
                                                            if let Ok(constant_data) = serde_json::from_value::<ConstantData>(message) {
                                                                database_handler::delete_constant(&database_pool, constant_data.name, &mut ws_sender).await;
                                                            }
                                                        }
                                                    },
                                                    "CREATE_CONSTANT" => {
                                                        if let Some(message) = parsed_message.d {
                                                            if let Ok(constant_data) = serde_json::from_value::<crate::models::constants::NewConstant>(message) {
                                                                database_handler::create_constant(&database_pool, constant_data.name, constant_data.value, &mut ws_sender).await;
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
    debug!("Rebooting system...");
    Command::new("sudo")
        .arg("reboot")
        .status()?;

    Ok(())
}

fn shutdown_system() -> io::Result<()> {
    debug!("Rebooting system...");
    Command::new("sudo")
        .arg("shutdown")
        .arg("now")
        .status()?;

    Ok(())
}
