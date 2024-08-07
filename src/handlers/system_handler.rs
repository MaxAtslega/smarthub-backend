use std::collections::HashMap;
use std::sync::Arc;

use dbus::arg::{RefArg, Variant};
use dbus::Message;
use dbus::message::MatchRule;
use dbus::nonblock::SyncConnection;
use dbus_tokio::connection;
use futures::channel::mpsc::UnboundedReceiver;
use log::{debug, error, info};
use tokio::sync::broadcast::Sender;
use tokio::sync::mpsc::Receiver;

use crate::enums::system_command::SystemCommand;
use crate::handlers::bluetooth_handler::{handle_bluetooth_device_command, handle_bluetooth_discovery_command, handle_get_all_bluetooth_devices_command, send_bluetooth_device_boned_event, send_bluetooth_device_connected_event, send_bluetooth_device_paired_event, send_bluetooth_device_trusted_event, send_bluetooth_discover_event, send_new_bluetooth_device_event};
use crate::handlers::network_handler::{get_current_network_status, get_network_interfaces, get_scan_results};
use crate::handlers::update_handler::{get_available_updates, perform_system_update};
use crate::models::websocket::WebSocketMessage;

#[tokio::main]
pub async fn system_handler(tx: Sender<WebSocketMessage>, rx_dbus: Receiver<SystemCommand>) -> Result<(), String> {
    let (resource, conn) = connection::new_system_sync().unwrap();

    tokio::spawn(async {
        let err = resource.await;
        error!("Lost connection to D-Bus: {}", err);
    });

    info!("Connected to D-Bus");

    // Process incoming messages
    let mr = MatchRule::new();

    let (incoming_signal, stream) = conn.add_match(mr).await.unwrap().stream();

    // Create a future calling D-Bus method each time the interval generates a tick
    let handle_dbus_events_future = handle_dbus_events(&tx, &conn, stream);
    let handle_dbus_commands_future = handle_dbus_commands(rx_dbus, conn.clone(), tx.clone());

    let print_to_console_future = async {
        let mut interval = tokio::time::interval( tokio::time::Duration::from_secs(5));

        loop {
            if let Err(e) = get_network_interfaces(tx.clone()).await {
                error!("Failed to get networtk interfaces: {}", e);
            }

            if let Err(e) = get_current_network_status(tx.clone()).await {
                error!("Failed to get current networtk status: {}", e);
            }
            
            if let Err(e) = get_scan_results(tx.clone()).await {
                error!("Failed to get current networtk status: {}", e);
            }

            interval.tick().await;
        }
    };

    futures::join!(
        handle_dbus_events_future,
        handle_dbus_commands_future,
        print_to_console_future
    );

    conn.remove_match(incoming_signal.token()).await.unwrap();

    Ok(())
}

async fn handle_dbus_commands(mut rx: Receiver<SystemCommand>, conn: Arc<SyncConnection>, tx: tokio::sync::broadcast::Sender<WebSocketMessage>) {
    while let Some(command) = rx.recv().await {
        match command {
            
            SystemCommand::BluetoothDiscovering(msg) => {
                handle_bluetooth_discovery_command(&conn, msg).await;
            },
            SystemCommand::ConnectBluetoothDevice(device_path) => {
                handle_bluetooth_device_command(&conn, &device_path, "Connect").await;
            },
            SystemCommand::DisconnectBluetoothDevice(device_path) => {
                handle_bluetooth_device_command(&conn, &device_path, "Disconnect").await;
            },
            SystemCommand::PairBluetoothDevice(device_path) => {
                handle_bluetooth_device_command(&conn, &device_path, "Pair").await;
            },
            SystemCommand::UnpairBluetoothDevice(device_path) => {
                handle_bluetooth_device_command(&conn, &device_path, "Unpair").await;
            },
            SystemCommand::TrustBluetoothDevice(device_path) => {
                handle_bluetooth_device_command(&conn, &device_path, "Trust").await;
            },
            SystemCommand::UntrustBluetoothDevice(device_path) => {
                handle_bluetooth_device_command(&conn, &device_path, "Untrust").await;
            },
            SystemCommand::GetAllBluetoothDevices => {
                handle_get_all_bluetooth_devices_command(&conn, tx.clone()).await;
            }
            SystemCommand::UpdateSystem => {
                if let Err(e) = perform_system_update(tx.clone()).await {
                    error!("Failed to perform system update: {}", e);
                }
            }
            SystemCommand::ListingSystemUpdates => {
                if let Err(e) = get_available_updates(tx.clone()).await {
                    error!("Failed to perform system update: {}", e);
                }
            }
        }
    }
}


async fn handle_dbus_events(tx: &Sender<WebSocketMessage>, conn: &Arc<SyncConnection>, stream: UnboundedReceiver<(Message, (String, ))>) {
    use futures_util::stream::StreamExt;

    let stream = stream.for_each(|(msg, (_source, )): (Message, (String, ))| {
        let conn_clone = conn.clone();
        if let Ok((interface, changed_properties)) = msg.read2::<String, HashMap<String, Variant<Box<dyn RefArg>>>>() {
            
            if interface.starts_with("org.bluez") { debug!("{}",interface) };
            
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