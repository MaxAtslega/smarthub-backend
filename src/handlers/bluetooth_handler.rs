extern crate dbus;

use std::collections::{HashMap, HashSet};
use std::time::Duration;
use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use dbus::arg::{RefArg, Variant};
use dbus::channel::MatchingReceiver;
use dbus::message::MatchRule;
use dbus::{blocking, Message, nonblock};
use dbus::blocking::Connection;
use dbus::nonblock::stdintf::org_freedesktop_dbus::{ObjectManager, Properties};
use dbus::nonblock::{Proxy, SyncConnection};
use serde_derive::{Deserialize, Serialize};
use tokio::sync::{mpsc, Mutex};
use tokio::sync::mpsc::Sender;
use dbus_tokio::connection;
use futures_util::future;
use lazy_static::lazy_static;
use log::{debug, error, info};
use serde_json::json;
use tokio::task;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use crate::app::DbusCommand;
use crate::models::notification_response::NotificationResponse;
use tokio::sync::mpsc::{channel, Receiver};

const BLUETOOTH_DEVICE_PATH: &str = "/org/bluez/hci0";

#[derive(Serialize, Deserialize, Debug)]
pub struct BluetoothDevice {
    pub name: String,
    pub address: String,
}

static IS_SCANNING: AtomicBool = AtomicBool::new(false);

pub async fn start_bluetooth_scanning() -> Result<(), Box<dyn Error>> {


    Ok(())
}

pub async fn stop_bluetooth_scanning() -> Result<(), Box<dyn Error>> {


    Ok(())
}


pub async fn connect_to_bluetooth_device(device_address: String) -> Result<(), Box<dyn std::error::Error>> {
    // Implement Bluetooth device connect logic
    Ok(())
}

pub async fn disconnect_bluetooth_device(device_address: String) -> Result<(), Box<dyn std::error::Error>> {
    // Implement Bluetooth device disconnect logic
    Ok(())
}

pub async fn pair_with_bluetooth_device(device_address: String) -> Result<(), Box<dyn std::error::Error>> {
    // Implement Bluetooth device pair logic
    Ok(())
}

pub async fn unpair_bluetooth_device(device_address: String) -> Result<(), Box<dyn std::error::Error>> {
    // Implement Bluetooth device unpair logic
    Ok(())
}

async fn get_device_name(conn: &Arc<SyncConnection>, device_path: &str) -> Result<String, Box<dyn Error>> {
    let proxy = dbus::nonblock::Proxy::new("org.bluez", device_path, Duration::from_secs(2), conn.clone());

    let props: HashMap<String, Variant<Box<dyn RefArg>>> = proxy.get_all("org.bluez.Device1").await?;
    if let Some(name) = props.get("Name").and_then(|v| v.0.as_str()) {
        Ok(name.to_string())
    } else {
        Err("Name not found".into())
    }
}

async fn handle_dbus_commands(mut rx: Receiver<DbusCommand>, conn: Arc<SyncConnection> ) {
    while let Some(command) = rx.recv().await {
        match command {
            DbusCommand::BLUEZ(msg) => {
                let conn = conn.clone();

                let proxy = nonblock::Proxy::new("org.bluez", "/org/bluez/hci0", Duration::from_secs(5), conn);
                let (_,): (String,) = match proxy.method_call("org.bluez.Adapter1", msg, ()).await {
                    Ok(resp) => resp,
                    Err(_) => {
                        continue;
                    }
                };
            },
            // ... handle other commands
        }
    }
}

#[tokio::main]
pub async fn listening(tx: tokio::sync::broadcast::Sender<NotificationResponse>, mut rx_dbus: Receiver<DbusCommand>) -> Result<(), Box<dyn std::error::Error>>{
    let (resource, conn) = connection::new_system_sync()?;

    tokio::spawn(async {
        let err = resource.await;
        error!("Lost connection to D-Bus: {}", err);
    });

    info!("Connected to D-Bus");

    // Process incoming messages
    let mr = MatchRule::new()
        .with_sender("org.bluez")
        .with_interface("org.freedesktop.DBus.Properties")
        .with_member("PropertiesChanged");

    let (incoming_signal, stream) = conn.add_match(mr).await?.stream();


    // Create a future calling D-Bus method each time the interval generates a tick
    let conn2: Arc<SyncConnection> = conn.clone();


    use futures_util::stream::StreamExt;
    let stream = stream.for_each(|(msg, (source,)): (Message, (String,))| {
        let conn_clone = conn.clone();

        if let Ok((interface, changed_properties)) = msg.read2::<String, HashMap<String, Variant<Box<dyn RefArg>>>>() {
            if interface == "org.bluez.Device1" {
                for (key, variant) in changed_properties {
                    match key.as_str() {
                        "Connected" => {
                            let device_path = msg.path().unwrap().to_string();
                            let conn_clone = conn_clone.clone();
                            let tx = tx.clone();
                            let connected = variant.0.as_u64().unwrap_or(0) != 0;

                            task::spawn(async move {
                                match get_device_name(&conn_clone, &device_path).await {
                                    Ok(name) => {
                                        let notif = if connected {
                                            NotificationResponse {
                                                title: "DEVICE_CONNECTED".to_string(),
                                                op: 2,
                                                data: json!(BluetoothDevice { name, address: device_path }),
                                            }
                                        } else {
                                            NotificationResponse {
                                                title: "DEVICE_DISCONNECTED".to_string(),
                                                op: 2,
                                                data: json!(BluetoothDevice { name, address: device_path }),
                                            }
                                        };

                                        tx.send(notif).unwrap();
                                    },
                                    Err(e) => error!("Error getting device name: {}", e),
                                }
                            });
                        },
                        "UUIDs" => {
                            let device_path = msg.path().unwrap().to_string();
                            let conn_clone = conn_clone.clone();
                            let tx = tx.clone();
                            task::spawn(async move {
                                match get_device_name(&conn_clone, &device_path).await {
                                    Ok(name) => {
                                        let notif = NotificationResponse {
                                            title: "DEVICE_FOUND".to_string(),
                                            op: 2,
                                            data: json!(BluetoothDevice { name, address: device_path }),
                                        };

                                        tx.send(notif).unwrap();
                                    }
                                    Err(e) => eprintln!("Error getting device name: {}", e),
                                }
                            });
                        },
                        "Trusted" => {
                            let device_path = msg.path().unwrap().to_string();
                            let trusted = variant.0.as_u64().unwrap_or(0) != 0;
                            let tx = tx.clone();
                            let conn_clone = conn_clone.clone();

                            task::spawn(async move {
                                match get_device_name(&conn_clone, &device_path).await {
                                    Ok(name) => {
                                        let notif = if trusted {
                                            NotificationResponse {
                                                title: "DEVICE_TRUSTED".to_string(),
                                                op: 2,
                                                data: json!(BluetoothDevice { name, address: device_path }),
                                            }
                                        } else {
                                            NotificationResponse {
                                                title: "DEVICE_UNTRUSTED".to_string(),
                                                op: 2,
                                                data: json!(BluetoothDevice { name, address: device_path }),
                                            }
                                        };

                                        tx.send(notif).unwrap();
                                    }
                                    Err(e) => eprintln!("Error getting device name: {}", e),
                                }
                            });
                        },
                        "Paired" => {
                            let device_path = msg.path().unwrap().to_string();
                            let paired = variant.0.as_u64().unwrap_or(0) != 0;
                            let tx = tx.clone();
                            let conn_clone = conn_clone.clone();

                            task::spawn(async move {
                                match get_device_name(&conn_clone, &device_path).await {
                                    Ok(name) => {
                                        let notif = if paired {
                                            NotificationResponse {
                                                title: "DEVICE_PAIRED".to_string(),
                                                op: 2,
                                                data: json!(BluetoothDevice { name, address: device_path }),
                                            }
                                        } else {
                                            NotificationResponse {
                                                title: "DEVICE_UNPAIRED".to_string(),
                                                op: 2,
                                                data: json!(BluetoothDevice { name, address: device_path }),
                                            }
                                        };

                                        tx.send(notif).unwrap();
                                    }
                                    Err(e) => eprintln!("Error getting device name: {}", e),
                                }
                            });
                        },
                        "Boned" => {
                            let device_path = msg.path().unwrap().to_string();
                            let bonded = variant.0.as_u64().unwrap_or(0) != 0;
                            let tx = tx.clone();
                            let conn_clone = conn_clone.clone();

                            task::spawn(async move {
                                match get_device_name(&conn_clone, &device_path).await {
                                    Ok(name) => {
                                        let notif = if bonded {
                                            NotificationResponse {
                                                title: "DEVICE_BONDED".to_string(),
                                                op: 2,
                                                data: json!(BluetoothDevice { name, address: device_path }),
                                            }
                                        } else {
                                            NotificationResponse {
                                                title: "DEVICE_UNBONDED".to_string(),
                                                op: 2,
                                                data: json!(BluetoothDevice { name, address: device_path }),
                                            }
                                        };

                                        tx.send(notif).unwrap();
                                    }
                                    Err(e) => eprintln!("Error getting device name: {}", e),
                                }
                            });
                        },
                        _ => {},
                    }
                }
            } else if interface == "org.bluez.Adapter1" {
                for (key, variant) in changed_properties {
                    match key.as_str() {
                        "Discovering" => {
                            let discovering = variant.0.as_u64().unwrap_or(0) != 0;

                            let notif = if discovering {
                                NotificationResponse {
                                    title: "DISCOVERY_STARTED".to_string(),
                                    op: 2,
                                    data: json!(discovering),
                                }
                            } else {
                                NotificationResponse {
                                    title: "DISCOVERY_STOPPED".to_string(),
                                    op: 2,
                                    data: json!(discovering),
                                }
                            };
                        },
                        _ => {},
                    }
                }
            }
        };

        async {}
    });

    futures::join!(stream, handle_dbus_commands(rx_dbus, conn2));

    conn.remove_match(incoming_signal.token()).await?;

    Ok(())
}