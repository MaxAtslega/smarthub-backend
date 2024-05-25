use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use dbus::{Message, nonblock, Path};
use dbus::arg::{RefArg, Variant};
use dbus::nonblock::stdintf::org_freedesktop_dbus::{ObjectManager, Properties};
use dbus::nonblock::SyncConnection;
use log::{debug, error};
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::broadcast::Sender;
use tokio::task;

use crate::models::websocket::WebSocketMessage;

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

pub async fn get_bluetooth_device_properties(conn: &Arc<SyncConnection>, device_path: &str) -> Result<BluetoothDevice, Box<dyn Error>> {
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

pub async fn set_bluetooth_device_property(conn: &Arc<SyncConnection>, device_path: &str, property: &str, value: bool) {
    let proxy = nonblock::Proxy::new("org.bluez", device_path, Duration::from_secs(5), conn.clone());
    match proxy.method_call::<(), (&str, &str, dbus::arg::Variant<bool>), &str, &str>(
        "org.freedesktop.DBus.Properties",
        "Set",
        (
            "org.bluez.Device1",
            property,
            Variant(value),
        ),
    ).await {
        Ok(_) => debug!("Property {} set successfully", property),
        Err(e) => error!("Error setting property {}: {}", property, e),
    }
}

pub async fn handle_bluetooth_device_command(conn: &Arc<SyncConnection>, device_path: &str, method: &str) {
    match method {
        "Trust" => set_bluetooth_device_property(conn, device_path, "Trusted", true).await,
        "Untrust" => set_bluetooth_device_property(conn, device_path, "Trusted", false).await,
        _ => {
            let proxy = nonblock::Proxy::new("org.bluez", device_path, Duration::from_secs(5), conn.clone());
            match proxy.method_call::<(), (), _, _>("org.bluez.Device1", method, ()).await {
                Ok(_) => debug!("{} successfully", method),
                Err(e) => error!("Error in {}: {}", method, e),
            }
        }
    }
}

pub async fn handle_bluetooth_discovery_command(conn: &Arc<SyncConnection>, msg: String) {
    let proxy = nonblock::Proxy::new("org.bluez", "/org/bluez/hci0", Duration::from_secs(5), conn.clone());
    match proxy.method_call::<(), (), _, _>("org.bluez.Adapter1", msg, ()).await {
        Ok(_) => debug!("Discovery started successfully"),
        Err(e) => error!("Error starting discovery: {}", e),
    };
}

pub async fn handle_get_all_bluetooth_devices_command(conn: &Arc<SyncConnection>, tx: Sender<WebSocketMessage>) {
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

                    let notification = WebSocketMessage {
                        t: Some("DEVICE_INFO".to_string()),
                        op: 2,
                        d: Some(json!(bluetooth_device)),
                    };

                    tx.send(notification).unwrap();
                }
            }
        }
        Err(e) => error!("Error getting managed objects in bluetooth: {}", e),
    }
}

pub fn send_bluetooth_discover_event(tx: &Sender<WebSocketMessage>, variant: &Variant<Box<dyn RefArg>>) {
    let discovering = variant.0.as_u64().unwrap_or(0) != 0;

    let notif = if discovering {
        WebSocketMessage {
            t: Some("DISCOVERY_STARTED".to_string()),
            op: 2,
            d: Some(json!(discovering)),
        }
    } else {
        WebSocketMessage {
            t: Some("DISCOVERY_STOPPED".to_string()),
            op: 2,
            d: Some(json!(discovering)),
        }
    };

    tx.send(notif).unwrap();
}

pub fn send_bluetooth_device_boned_event(tx: &Sender<WebSocketMessage>, msg: &Message, conn: &Arc<SyncConnection>, variant: &Variant<Box<dyn RefArg>>) {
    let device_path = msg.path().unwrap().to_string();
    let bonded = variant.0.as_u64().unwrap_or(0) != 0;
    let tx = tx.clone();
    let conn = conn.clone();

    task::spawn(async move {
        match get_bluetooth_device_properties(&conn, &device_path).await {
            Ok(device) => {
                let notif = if bonded {
                    WebSocketMessage {
                        t: Some("DEVICE_BONDED".to_string()),
                        op: 2,
                        d: Some(json!(device)),
                    }
                } else {
                    WebSocketMessage {
                        t: Some("DEVICE_UNBONDED".to_string()),
                        op: 2,
                        d: Some(json!(device)),
                    }
                };

                tx.send(notif).unwrap();
            }
            Err(e) => error!("Error getting device name: {}", e),
        }
    });
}

pub fn send_bluetooth_device_paired_event(tx: &Sender<WebSocketMessage>, msg: &Message, conn: &Arc<SyncConnection>, variant: &Variant<Box<dyn RefArg>>) {
    let device_path = msg.path().unwrap().to_string();
    let paired = variant.0.as_u64().unwrap_or(0) != 0;
    let tx = tx.clone();
    let conn = conn.clone();

    task::spawn(async move {
        match get_bluetooth_device_properties(&conn, &device_path).await {
            Ok(device) => {
                let notif = if paired {
                    WebSocketMessage {
                        t: Some("DEVICE_PAIRED".to_string()),
                        op: 2,
                        d: Some(json!(device)),
                    }
                } else {
                    WebSocketMessage {
                        t: Some("DEVICE_UNPAIRED".to_string()),
                        op: 2,
                        d: Some(json!(device)),
                    }
                };

                tx.send(notif).unwrap();
            }
            Err(e) => error!("Error getting device name: {}", e),
        }
    });
}

pub fn send_bluetooth_device_trusted_event(tx: &Sender<WebSocketMessage>, msg: &Message, conn: &Arc<SyncConnection>, variant: &Variant<Box<dyn RefArg>>) {
    let device_path = msg.path().unwrap().to_string();
    let trusted = variant.0.as_u64().unwrap_or(0) != 0;
    let tx = tx.clone();
    let conn = conn.clone();

    task::spawn(async move {
        match get_bluetooth_device_properties(&conn, &device_path).await {
            Ok(device) => {
                let notif = if trusted {
                    WebSocketMessage {
                        t: Some("DEVICE_TRUSTED".to_string()),
                        op: 2,
                        d: Some(json!(device)),
                    }
                } else {
                    WebSocketMessage {
                        t: Some("DEVICE_UNTRUSTED".to_string()),
                        op: 2,
                        d: Some(json!(device)),
                    }
                };

                tx.send(notif).unwrap();
            }
            Err(e) => error!("Error getting device name: {}", e),
        }
    });
}

pub fn send_new_bluetooth_device_event(tx: &Sender<WebSocketMessage>, msg: &Message, conn: &Arc<SyncConnection>) {
    let device_path = msg.path().unwrap().to_string();
    let conn = conn.clone();
    let tx = tx.clone();
    task::spawn(async move {
        match get_bluetooth_device_properties(&conn, &device_path).await {
            Ok(device) => {
                let notif = WebSocketMessage {
                    t: Some("DEVICE_FOUND".to_string()),
                    op: 2,
                    d: Some(json!(device)),
                };

                tx.send(notif).unwrap();
            }
            Err(e) => error!("Error getting device name: {}", e),
        }
    });
}

pub fn send_bluetooth_device_connected_event(tx: &Sender<WebSocketMessage>, msg: &Message, conn: &Arc<SyncConnection>, variant: &Variant<Box<dyn RefArg>>) {
    let device_path = msg.path().unwrap().to_string();
    let conn = conn.clone();
    let tx = tx.clone();
    let connected = variant.0.as_u64().unwrap_or(0) != 0;

    task::spawn(async move {
        match get_bluetooth_device_properties(&conn, &device_path).await {
            Ok(device) => {
                let notif = if connected {
                    WebSocketMessage {
                        t: Some("DEVICE_CONNECTED".to_string()),
                        op: 2,
                        d: Some(json!(device)),
                    }
                } else {
                    WebSocketMessage {
                        t: Some("DEVICE_DISCONNECTED".to_string()),
                        op: 2,
                        d: Some(json!(device)),
                    }
                };

                tx.send(notif).unwrap();
            }
            Err(e) => error!("Error getting device name: {}", e),
        }
    });
}