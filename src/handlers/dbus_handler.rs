extern crate dbus;

use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use dbus::{Message, nonblock};
use dbus::arg::{RefArg, Variant};
use dbus::message::MatchRule;
use dbus::nonblock::stdintf::org_freedesktop_dbus::{ObjectManager, Properties};
use dbus::nonblock::SyncConnection;
use dbus_tokio::connection;
use log::{debug, error, info};
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::mpsc::Receiver;
use tokio::task;
use crate::enums::dbus_command::DbusCommand;

use crate::models::notification_response::NotificationResponse;

#[derive(Serialize, Deserialize, Debug)]
pub struct BluetoothDevice {
    pub name: String,
    pub address: String,
    connected: Option<bool>,
    paired: Option<bool>,
    blocked: Option<bool>,
    trusted: Option<bool>,
    bonded: Option<bool>,
}

async fn get_device_properties(conn: &Arc<SyncConnection>, device_path: &str) -> Result<BluetoothDevice, Box<dyn Error>> {
    let proxy = dbus::nonblock::Proxy::new("org.bluez", device_path, Duration::from_secs(2), conn.clone());

    let device: HashMap<String, Variant<Box<dyn RefArg>>> = proxy.get_all("org.bluez.Device1").await?;
    let name = device.get("Name").and_then(|v| v.0.as_str()).unwrap_or("Unknown");
    let connected = device.get("Connected").and_then(|v| v.0.as_u64()).map(|v| v != 0);
    let paired = device.get("Paired").and_then(|v| v.0.as_u64()).map(|v| v != 0);
    let blocked = device.get("Blocked").and_then(|v| v.0.as_u64()).map(|v| v != 0);
    let trusted = device.get("Trusted").and_then(|v| v.0.as_u64()).map(|v| v != 0);
    let bonded = device.get("Bonded").and_then(|v| v.0.as_u64()).map(|v| v != 0);

    Ok(BluetoothDevice {
        name: name.to_string(),
        address: device_path.to_string(),
        connected,
        paired,
        blocked,
        trusted,
        bonded,
    })
}

async fn handle_device_command(conn: &Arc<SyncConnection>, device_path: &str, method: &str) {
    let proxy = nonblock::Proxy::new("org.bluez", device_path, Duration::from_secs(5), conn.clone());
    match proxy.method_call::<(), (), _, _>("org.bluez.Device1", method, ()).await {
        Ok(_) => debug!("{} successfully", method),
        Err(e) => debug!("Error in {}: {}", method, e),
    }
}

async fn handle_dbus_commands(mut rx: Receiver<DbusCommand>, conn: Arc<SyncConnection>, tx: tokio::sync::broadcast::Sender<NotificationResponse>) {
    while let Some(command) = rx.recv().await {
        match command {
            DbusCommand::BluetoothDiscovering(msg) => {
                let conn = conn.clone();
                debug!("test");
                let proxy = nonblock::Proxy::new("org.bluez", "/org/bluez/hci0", Duration::from_secs(5), conn);
                match proxy.method_call::<(), (), _, _>("org.bluez.Adapter1", msg, ()).await {
                    Ok(_) => debug!("Discovery started successfully"),
                    Err(e) => debug!("Error starting discovery: {}", e),
                };
            },
            DbusCommand::ConnectDevice(device_path) => {
                handle_device_command(&conn, &device_path, "Connect").await;
            },
            DbusCommand::DisconnectDevice(device_path) => {
                handle_device_command(&conn, &device_path, "Disconnect").await;
            },
            DbusCommand::PairDevice(device_path) => {
                handle_device_command(&conn, &device_path, "Pair").await;
            },
            DbusCommand::UnpairDevice(device_path) => {
                handle_device_command(&conn, &device_path, "Unpair").await;
            },
            DbusCommand::TrustDevice(device_path) => {
                handle_device_command(&conn, &device_path, "Trust").await;
            },
            DbusCommand::UntrustDevice(device_path) => {
                handle_device_command(&conn, &device_path, "Untrust").await;
            },
            DbusCommand::GetAllConnectedDevices => {
                // Logic to get all connected devices
                let proxy = nonblock::Proxy::new("org.bluez", "/", Duration::from_secs(5), conn.clone());

                match proxy.get_managed_objects().await {
                    Ok(objects) => {
                        for (path, interfaces) in objects {
                            if let Some(device) = interfaces.get("org.bluez.Device1") {
                                if let Some(connected) = device.get("Connected").and_then(|v| v.as_u64()) {
                                    if connected != 0 {
                                        let name = device.get("Name").and_then(|v| v.0.as_str()).unwrap_or("Unknown");
                                        let connected = device.get("Connected").and_then(|v| v.0.as_u64()).map(|v| v != 0);
                                        let paired = device.get("Paired").and_then(|v| v.0.as_u64()).map(|v| v != 0);
                                        let blocked = device.get("Blocked").and_then(|v| v.0.as_u64()).map(|v| v != 0);
                                        let trusted = device.get("Trusted").and_then(|v| v.0.as_u64()).map(|v| v != 0);
                                        let bonded = device.get("Bonded").and_then(|v| v.0.as_u64()).map(|v| v != 0);

                                        let bluetooth_device = BluetoothDevice {
                                            name: name.to_string(),
                                            address: path.to_string(),
                                            connected,
                                            paired,
                                            blocked,
                                            trusted,
                                            bonded,
                                        };

                                        let notification = NotificationResponse {
                                            title: "DEVICE_CONNECTED".to_string(),
                                            op: 2,
                                            data: json!(bluetooth_device),
                                        };

                                        tx.send(notification).unwrap();
                                    }
                                }
                            }
                        }
                    },
                    Err(e) => eprintln!("Error: {}", e),
                }
            },
            DbusCommand::GetAllPairedDevices => {
                // Logic to get all paired devices
                let proxy = nonblock::Proxy::new("org.bluez", "/", Duration::from_secs(5), conn.clone());
                match proxy.get_managed_objects().await {
                    Ok(objects) => {
                        for (path, interfaces) in objects {
                            if let Some(device) = interfaces.get("org.bluez.Device1") {
                                if let Some(paired) = device.get("Paired").and_then(|v| v.as_u64()) {
                                    if paired != 0 {
                                        let name = device.get("Name").and_then(|v| v.0.as_str()).unwrap_or("Unknown");
                                        let connected = device.get("Connected").and_then(|v| v.0.as_u64()).map(|v| v != 0);
                                        let paired = device.get("Paired").and_then(|v| v.0.as_u64()).map(|v| v != 0);
                                        let blocked = device.get("Blocked").and_then(|v| v.0.as_u64()).map(|v| v != 0);
                                        let trusted = device.get("Trusted").and_then(|v| v.0.as_u64()).map(|v| v != 0);
                                        let bonded = device.get("Bonded").and_then(|v| v.0.as_u64()).map(|v| v != 0);

                                        let bluetooth_device = BluetoothDevice {
                                            name: name.to_string(),
                                            address: path.to_string(),
                                            connected,
                                            paired,
                                            blocked,
                                            trusted,
                                            bonded,
                                        };

                                        let notification = NotificationResponse {
                                            title: "DEVICE_PAIRED".to_string(),
                                            op: 2,
                                            data: json!(bluetooth_device),
                                        };

                                        tx.send(notification).unwrap();
                                    }
                                }
                            }
                        }
                    },
                    Err(e) => eprintln!("Error: {}", e),
                }
            },
            DbusCommand::GetAllDevices => {
                let proxy = nonblock::Proxy::new("org.bluez", "/", Duration::from_secs(5), conn.clone());
                match proxy.get_managed_objects().await {
                    Ok(objects) => {
                        for (path, interfaces) in objects {
                            if let Some(device_properties) = interfaces.get("org.bluez.Device1") {
                                let name = device_properties.get("Name").and_then(|v| v.0.as_str()).unwrap_or("Unknown");
                                let connected = device_properties.get("Connected").and_then(|v| v.0.as_u64()).map(|v| v != 0);
                                let paired = device_properties.get("Paired").and_then(|v| v.0.as_u64()).map(|v| v != 0);
                                let blocked = device_properties.get("Blocked").and_then(|v| v.0.as_u64()).map(|v| v != 0);
                                let trusted = device_properties.get("Trusted").and_then(|v| v.0.as_u64()).map(|v| v != 0);
                                let bonded = device_properties.get("Bonded").and_then(|v| v.0.as_u64()).map(|v| v != 0);

                                let bluetooth_device = BluetoothDevice {
                                    name: name.to_string(),
                                    address: path.to_string(),
                                    connected,
                                    paired,
                                    blocked,
                                    trusted,
                                    bonded,
                                };

                                let notification = NotificationResponse {
                                    title: "DEVICE_INFO".to_string(),
                                    op: 2,
                                    data: json!(bluetooth_device),
                                };

                                tx.send(notification).unwrap();
                            }
                        }
                    },
                    Err(e) => eprintln!("Error: {}", e),
                }
            },
        }
    }
}

#[tokio::main]
pub async fn dbus_handler(tx: tokio::sync::broadcast::Sender<NotificationResponse>, mut rx_dbus: Receiver<DbusCommand>) -> Result<(), Box<dyn std::error::Error>> {
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
    let stream = stream.for_each(|(msg, (source, )): (Message, (String, ))| {
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
                                match get_device_properties(&conn_clone, &device_path).await {
                                    Ok(device) => {
                                        let notif = if connected {
                                            NotificationResponse {
                                                title: "DEVICE_CONNECTED".to_string(),
                                                op: 2,
                                                data: json!(device),
                                            }
                                        } else {
                                            NotificationResponse {
                                                title: "DEVICE_DISCONNECTED".to_string(),
                                                op: 2,
                                                data: json!(device),
                                            }
                                        };

                                        tx.send(notif).unwrap();
                                    }
                                    Err(e) => error!("Error getting device name: {}", e),
                                }
                            });
                        }
                        "UUIDs" => {
                            let device_path = msg.path().unwrap().to_string();
                            let conn_clone = conn_clone.clone();
                            let tx = tx.clone();
                            task::spawn(async move {
                                match get_device_properties(&conn_clone, &device_path).await {
                                    Ok(device) => {
                                        let notif = NotificationResponse {
                                            title: "DEVICE_FOUND".to_string(),
                                            op: 2,
                                            data: json!(device),
                                        };

                                        tx.send(notif).unwrap();
                                    }
                                    Err(e) => eprintln!("Error getting device name: {}", e),
                                }
                            });
                        }
                        "Trusted" => {
                            let device_path = msg.path().unwrap().to_string();
                            let trusted = variant.0.as_u64().unwrap_or(0) != 0;
                            let tx = tx.clone();
                            let conn_clone = conn_clone.clone();

                            task::spawn(async move {
                                match get_device_properties(&conn_clone, &device_path).await {
                                    Ok(device) => {
                                        let notif = if trusted {
                                            NotificationResponse {
                                                title: "DEVICE_TRUSTED".to_string(),
                                                op: 2,
                                                data: json!(device),
                                            }
                                        } else {
                                            NotificationResponse {
                                                title: "DEVICE_UNTRUSTED".to_string(),
                                                op: 2,
                                                data: json!(device),
                                            }
                                        };

                                        tx.send(notif).unwrap();
                                    }
                                    Err(e) => eprintln!("Error getting device name: {}", e),
                                }
                            });
                        }
                        "Paired" => {
                            let device_path = msg.path().unwrap().to_string();
                            let paired = variant.0.as_u64().unwrap_or(0) != 0;
                            let tx = tx.clone();
                            let conn_clone = conn_clone.clone();

                            task::spawn(async move {
                                match get_device_properties(&conn_clone, &device_path).await {
                                    Ok(device) => {
                                        let notif = if paired {
                                            NotificationResponse {
                                                title: "DEVICE_PAIRED".to_string(),
                                                op: 2,
                                                data: json!(device),
                                            }
                                        } else {
                                            NotificationResponse {
                                                title: "DEVICE_UNPAIRED".to_string(),
                                                op: 2,
                                                data: json!(device),
                                            }
                                        };

                                        tx.send(notif).unwrap();
                                    }
                                    Err(e) => eprintln!("Error getting device name: {}", e),
                                }
                            });
                        }
                        "Boned" => {
                            let device_path = msg.path().unwrap().to_string();
                            let bonded = variant.0.as_u64().unwrap_or(0) != 0;
                            let tx = tx.clone();
                            let conn_clone = conn_clone.clone();

                            task::spawn(async move {
                                match get_device_properties(&conn_clone, &device_path).await {
                                    Ok(device) => {
                                        let notif = if bonded {
                                            NotificationResponse {
                                                title: "DEVICE_BONDED".to_string(),
                                                op: 2,
                                                data: json!(device),
                                            }
                                        } else {
                                            NotificationResponse {
                                                title: "DEVICE_UNBONDED".to_string(),
                                                op: 2,
                                                data: json!(device),
                                            }
                                        };

                                        tx.send(notif).unwrap();
                                    }
                                    Err(e) => eprintln!("Error getting device name: {}", e),
                                }
                            });
                        }
                        _ => {}
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

                            tx.send(notif).unwrap();
                        }
                        _ => {}
                    }
                }
            }
        };

        async {}
    });

    futures::join!(stream, handle_dbus_commands(rx_dbus, conn2, tx.clone()));

    conn.remove_match(incoming_signal.token()).await?;

    Ok(())
}