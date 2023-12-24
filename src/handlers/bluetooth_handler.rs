extern crate dbus;

use std::collections::{HashSet};
use std::time::Duration;
use std::error::Error;
use std::sync::Arc;
use dbus::arg::RefArg;
use dbus::nonblock;
use dbus::nonblock::{Proxy, SyncConnection};
use dbus::nonblock::stdintf::org_freedesktop_dbus::ObjectManager;
use serde_derive::{Deserialize, Serialize};
use tokio::sync::{Mutex};
use tokio::sync::mpsc::Sender;
use dbus_tokio::connection;
use lazy_static::lazy_static;
use log::debug;
use tokio::task::JoinHandle;

const BLUETOOTH_DEVICE_PATH: &str = "/org/bluez/hci0";

#[derive(Serialize, Deserialize, Debug)]
pub struct BluetoothDevice {
    pub name: String,
    pub address: String,
}

struct ScanningState {
    is_scanning: bool,
    discovered_devices: HashSet<String>,
    scanning_task: Option<JoinHandle<()>>,
    dbus_resource: Option<JoinHandle<()>>,
    dbus_conn: Option<Arc<nonblock::SyncConnection>>,
}

lazy_static! {
    static ref SCANNING_STATE: Arc<Mutex<ScanningState>> = Arc::new(Mutex::new(ScanningState {
        is_scanning: false,
        discovered_devices: HashSet::new(),
        scanning_task: None,
        dbus_resource: None,
        dbus_conn: None,
    }));
}


pub async fn start_bluetooth_scanning(tx: Sender<BluetoothDevice>) -> Result<(), Box<dyn std::error::Error>> {
    let scanning_state = SCANNING_STATE.clone();
    let scanning_state_for_async = scanning_state.clone(); // Clone for use in the async block

    let mut state = scanning_state.lock().await;
    if state.is_scanning {
        println!("Bluetooth scanning is already in progress.");
        return Ok(());
    }

    let (resource, conn) = connection::new_system_sync()?;
    let dbus_handle = tokio::spawn(async move {
        let _err = resource.await;
        debug!("Lost connection to D-Bus");
    });

    let proxy = nonblock::Proxy::new("org.bluez", "/org/bluez/hci0", Duration::from_secs(5), conn.clone());
    proxy.method_call("org.bluez.Adapter1", "StartDiscovery", ()).await?;

    state.is_scanning = true;
    state.dbus_resource = Some(dbus_handle);
    state.dbus_conn = Some(conn.clone());

    drop(state); // Release the lock before entering the async block

    let scanning_task = tokio::spawn(async move {
        let proxy2 = nonblock::Proxy::new("org.bluez", "/", Duration::from_secs(5), conn);
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;

            let state_lock = scanning_state_for_async.lock().await;

            if !state_lock.is_scanning {
                break;
            }

            drop(state_lock); // Release the lock before sleeping

            let objects = proxy2.get_managed_objects().await;

            let objects = match objects {
                Ok(objects) => objects,
                Err(_) => {
                    break;
                }
            };

            for (path, interfaces) in objects {
                let device_id = path.to_string();

                if let Some(properties) = interfaces.get("org.bluez.Device1") {

                    // Clone state and tx to move into the async block
                    let scanning_state_clone = scanning_state_for_async.clone();
                    let tx_clone = tx.clone();

                    // Process in an async block
                    let mut state = scanning_state_clone.lock().await;

                    if state.discovered_devices.insert(device_id.clone()) {
                        let name = properties.get("Name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown Device")
                            .to_string();

                        let device = BluetoothDevice { address: device_id, name };
                        println!("Discovered device: {:?}", device);
                        tx_clone.send(device).await.unwrap();
                    }
                }
            }
        }
    });

    let mut state = scanning_state.lock().await;
    state.scanning_task = Some(scanning_task);

    Ok(())
}

pub async fn stop_bluetooth_scanning() -> Result<(), Box<dyn Error>> {
    let scanning_state = SCANNING_STATE.clone();
    let mut state = scanning_state.lock().await;

    if !state.is_scanning {
        println!("Bluetooth scanning is not in progress.");
        return Ok(());
    }

    // Stop the Bluetooth discovery process
    if let Some(conn) = state.dbus_conn.as_ref() {
        let proxy = nonblock::Proxy::new("org.bluez", "/org/bluez/hci0", Duration::from_secs(5), conn.clone());
        proxy.method_call("org.bluez.Adapter1", "StopDiscovery", ()).await?;
    }

    state.is_scanning = false;
    let task_handle = state.scanning_task.take();
    if let Some(dbus_handle) = state.dbus_resource.take() {
        dbus_handle.abort(); // Abort the D-Bus connection task
    }
    state.dbus_conn = None; // Drop the D-Bus connection
    drop(state); // Release the lock before awaiting the task

    if let Some(handle) = task_handle {
        let _ = handle.await;  // Wait for the task to complete
    }

    println!("Bluetooth scanning stopped");
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