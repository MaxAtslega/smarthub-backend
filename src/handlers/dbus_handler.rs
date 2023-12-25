extern crate dbus;

use std::collections::HashMap;
use std::sync::Arc;

use dbus::arg::{RefArg, Variant};
use dbus::Message;
use dbus::message::MatchRule;
use dbus::nonblock::SyncConnection;
use dbus_tokio::connection;
use log::{error, info};
use tokio::sync::mpsc::Receiver;

use crate::enums::dbus_command::DbusCommand;
use crate::handlers::bluetooth_handler::{handle_bluetooth_device_command, handle_bluetooth_discovery_command, handle_get_all_bluetooth_devices_command, send_bluetooth_device_boned_event, send_bluetooth_device_connected_event, send_bluetooth_device_paired_event, send_bluetooth_device_trusted_event, send_bluetooth_discover_event, send_new_bluetooth_device_event};
use crate::models::notification_response::NotificationResponse;

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
            }
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
    let mr = MatchRule::new();

    let (incoming_signal, stream) = conn.add_match(mr).await?.stream();


    // Create a future calling D-Bus method each time the interval generates a tick
    let conn2: Arc<SyncConnection> = conn.clone();


    use futures_util::stream::StreamExt;
    let stream = stream.for_each(|(msg, (source, )): (Message, (String, ))| {
        let conn_clone = conn.clone();
        println!("{}: {:?}", source, msg);
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

    futures::join!(stream, handle_dbus_commands(rx_dbus, conn2, tx.clone()));

    conn.remove_match(incoming_signal.token()).await?;

    Ok(())
}