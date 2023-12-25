extern crate dbus;

use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use dbus::arg::{Dict, PropMap, RefArg, Variant};
use dbus::Message;
use dbus::message::MatchRule;
use dbus::nonblock::{NonblockReply, Proxy, SyncConnection};
use dbus::nonblock::stdintf::org_freedesktop_dbus::Properties;
use dbus_tokio::connection;
use futures::channel::mpsc::UnboundedReceiver;
use futures_util::stream::ForEach;
use log::{error, info};
use serde_json::json;
use tokio::sync::broadcast::Sender;
use tokio::sync::mpsc::Receiver;

use crate::enums::dbus_command::DbusCommand;
use crate::handlers::bluetooth_handler::{handle_bluetooth_device_command, handle_bluetooth_discovery_command, handle_get_all_bluetooth_devices_command, send_bluetooth_device_boned_event, send_bluetooth_device_connected_event, send_bluetooth_device_paired_event, send_bluetooth_device_trusted_event, send_bluetooth_discover_event, send_new_bluetooth_device_event};
use crate::models::notification_response::NotificationResponse;

#[tokio::main]
pub async fn dbus_handler(tx: tokio::sync::broadcast::Sender<NotificationResponse>, rx_dbus: Receiver<DbusCommand>) -> Result<(), Box<dyn std::error::Error>> {
    let (resource, conn) = connection::new_system_sync()?;

    tokio::spawn(async {
        let err = resource.await;
        error!("Lost connection to D-Bus: {}", err);
    });

    info!("Connected to D-Bus");

    // Process incoming messages
    let mr = MatchRule::new();

    let (incoming_signal, stream) = conn.add_match(mr).await?.stream();

    // Create a future calling D-Bus method each time the interval generates a tick
    futures::join!(handle_dbus_events(&tx, &conn, stream), handle_dbus_commands(rx_dbus, conn.clone(), tx.clone()));

    conn.remove_match(incoming_signal.token()).await?;

    Ok(())
}

async fn handle_dbus_commands(mut rx: Receiver<DbusCommand>, conn: Arc<SyncConnection>, tx: tokio::sync::broadcast::Sender<NotificationResponse>) {
    while let Some(command) = rx.recv().await {
        match command {
            DbusCommand::BluetoothDiscovering(msg) => {
                handle_bluetooth_discovery_command(&conn, msg).await;
            },
            DbusCommand::ConnectBluetoothDevice(device_path) => {
                handle_bluetooth_device_command(&conn, &device_path, "Connect").await;
            },
            DbusCommand::DisconnectBluetoothDevice(device_path) => {
                handle_bluetooth_device_command(&conn, &device_path, "Disconnect").await;
            },
            DbusCommand::PairBluetoothDevice(device_path) => {
                handle_bluetooth_device_command(&conn, &device_path, "Pair").await;
            },
            DbusCommand::UnpairBluetoothDevice(device_path) => {
                handle_bluetooth_device_command(&conn, &device_path, "Unpair").await;
            },
            DbusCommand::TrustBluetoothDevice(device_path) => {
                handle_bluetooth_device_command(&conn, &device_path, "Trust").await;
            },
            DbusCommand::UntrustBluetoothDevice(device_path) => {
                handle_bluetooth_device_command(&conn, &device_path, "Untrust").await;
            },
            DbusCommand::GetAllBluetoothDevices => {
                handle_get_all_bluetooth_devices_command(&conn, tx.clone()).await;
            },
            DbusCommand::GetCurrentNetwork => {
                handle_get_current_network_command(&conn, tx.clone()).await;
            },
        }
    }
}

async fn handle_get_current_network_command(conn: &Arc<SyncConnection>, tx: Sender<NotificationResponse>) {
    let proxy = Proxy::new("org.freedesktop.NetworkManager", "/org/freedesktop/NetworkManager", Duration::from_secs(5), conn.clone());

    let (variant,): (Variant<Box<dyn RefArg>>,) = match proxy.method_call(
        "org.freedesktop.DBus.Properties",
        "Get",
        ("org.freedesktop.NetworkManager", "PrimaryConnection")
    ).await {
        Ok(variant) => variant,
        Err(e) => {
            eprintln!("Failed to get active connection: {}", e);
            return;
        }
    };

    if let Some(active_connection_path) = variant.0.as_str() {
        if active_connection_path == "/" {
            println!("No active connection");
            return;
        }

        let active_connection_proxy = Proxy::new(
            "org.freedesktop.NetworkManager",
            active_connection_path,
            Duration::from_secs(5),
            conn.clone(),
        );

        // Get the IP address and additional properties
        let properties: HashMap<String, Variant<Box<dyn RefArg>>> = match active_connection_proxy.get_all("org.freedesktop.NetworkManager.Connection.Active").await {
            Ok(props) => props,
            Err(e) => {
                eprintln!("Failed to get properties: {}", e);
                return;
            }
        };


        let uuid = properties.get("Uuid").and_then(|v| v.0.as_str()).unwrap_or("Unknown");

        // Fetch the IP4Config and IP6Config paths
        let ip4_config_path = properties.get("Ip4Config").and_then(|v| v.0.as_str()).unwrap_or_default();
        let ip6_config_path = properties.get("Ip6Config").and_then(|v| v.0.as_str()).unwrap_or_default();

        // Fetch IPv4 and IPv6 addresses
        let ipv4_address = get_ip_address(conn, ip4_config_path).await;
        let ipv6_address = get_ip_address(conn, ip6_config_path).await;

        // Fetch SSID if it's a WLAN connection
        let ssid = if let Some(connection_path) = properties.get("Connection").and_then(|v| v.0.as_str()) {
            get_ssid(conn, connection_path).await
        } else {
            "Not a WLAN Connection".to_string()
        };

        // Send notification with the network details
        let notification = NotificationResponse {
            title: "CURRENT_NETWORK_INFO".to_string(),
            op: 1,
            data: json!({ "UUID": uuid, "IPv4 Address": ipv4_address, "IPv6 Address": ipv6_address, "SSID": ssid }),
        };

        tx.send(notification).unwrap();
    } else {
        println!("No active connection found");
    }
}

async fn get_ip_address(conn: &Arc<SyncConnection>, config_path: &str) -> (String, u32) {
    if config_path.is_empty() {
        return ("Not Available".to_string(), 0);
    }

    let proxy = Proxy::new(
        "org.freedesktop.NetworkManager",
        config_path,
        Duration::from_secs(5),
        conn.clone(),
    );
    let properties: HashMap<String, Variant<Box<dyn RefArg>>> = match proxy.get_all("org.freedesktop.NetworkManager.IP4Config").await {
        Ok(props) => props,
        Err(_) => return ("Error Fetching IP".to_string(), 0),
    };

    if let Some(address_data) = properties.get("AddressData") {
        if let Some(iter) = address_data.0.as_iter() {
            let mut ip_address = String::new();
            let mut prefix = 0;

            for refarg in iter {
                if let Some(data) = refarg.as_iter() {
                    let data_vec = data.collect::<Vec<_>>();
                    if data_vec.len() >= 4 {
                        if let Some(ip) = data_vec[1].as_str() {
                            ip_address = ip.to_string();
                        }
                        if let Some(pr) = data_vec[3].as_u64() {
                            prefix = pr as u32;
                        }
                        if let Some(ip) = data_vec[3].as_str() {
                            ip_address = ip.to_string();
                        }
                        if let Some(pr) = data_vec[1].as_u64() {
                            prefix = pr as u32;
                        }
                    }
                }
            }

            return (ip_address, prefix);
        }
    }

    ("No IP Found".to_string(), 0)
}

async fn get_ssid(conn: &Arc<SyncConnection>, connection_path: &str) -> String {
    info!("Connected to D-Bus");

    let address: String = connection_path.to_string();

    let proxy = Proxy::new(
        "org.freedesktop.NetworkManager",
        address.as_str(),
        Duration::from_secs(5),
        conn.clone(),
    );

    let result: Result<(PropMap,), dbus::Error> = proxy.method_call(
        "org.freedesktop.NetworkManager.Settings.Connection",
        "GetSettings",
        ()
    ).await;

    match result {
        Ok((variant,)) => {
            // Diagnostic: Print the variant to understand its structure
            println!("Variant response: {:?}", variant);

            // TODO: Process the variant to extract the SSID
            // Adjust this part based on the actual structure of the variant
        },
        Err(e) => {
            eprintln!("Error Fetching SSID: {}", e);
        },
    }
    "".to_string()
}



async fn handle_dbus_events(tx: &Sender<NotificationResponse>, conn: &Arc<SyncConnection>, stream: UnboundedReceiver<(Message, (String, ))>) {
    use futures_util::stream::StreamExt;

    let stream = stream.for_each(|(msg, (source, )): (Message, (String, ))| {
        let conn_clone = conn.clone();
        if let Ok((interface, changed_properties)) = msg.read2::<String, HashMap<String, Variant<Box<dyn RefArg>>>>() {
            if interface == "org.bluez.Device1" {
                for (key, variant) in changed_properties {
                    match key.as_str() {
                        "Connected" => {
                            send_bluetooth_device_connected_event(&tx, &msg, &conn_clone, &variant);
                        }
                        "UUIDs" => {
                            send_new_bluetooth_device_event(&tx, &msg, &conn_clone);
                        }
                        "Trusted" => {
                            send_bluetooth_device_trusted_event(&tx, &msg, &conn_clone, &variant);
                        }
                        "Paired" => {
                            send_bluetooth_device_paired_event(&tx, &msg, &conn_clone, &variant);
                        }
                        "Boned" => {
                            send_bluetooth_device_boned_event(&tx, &msg, &conn_clone, &variant);
                        }
                        _ => {}
                    }
                }
            } else if interface == "org.bluez.Adapter1" {
                for (key, variant) in changed_properties {
                    match key.as_str() {
                        "Discovering" => {
                            send_bluetooth_discover_event(&tx, &variant);
                        }
                        _ => {}
                    }
                }
            }
        };

        async {}
    });

    stream.await
}