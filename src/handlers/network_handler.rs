use std::error::Error;
use std::fs;

use serde_json::json;
use tokio::process::Command;
use tokio::sync::broadcast::Sender;

use crate::models::websocket::WebSocketMessage;
use crate::network::interfaces::get_interfaces;
use crate::network::wifi_scan;
use std::fmt::Write as FmtWrite;

pub async fn get_network_interfaces(tx: Sender<WebSocketMessage>) -> Result<(), Box<dyn Error>>{
    let interfaces = get_interfaces();

    if interfaces.is_err() {
        return Err(Box::new(interfaces.err().unwrap()));
    }

    let notification = WebSocketMessage {
        op: 0,
        t: Some("NETWORK_INTERFACES".to_string()),
        d: Some(json!(interfaces.unwrap())),
    };

    tx.send(notification).expect("Failed to send notification");

    Ok(())
}


pub async fn scan_wifi(tx: Sender<WebSocketMessage>) -> Result<(), Box<dyn Error>>{
    let wifi_networks = wifi_scan::scan().await;

    if wifi_networks.is_err() {
        return Err(Box::new(wifi_networks.err().unwrap()));
    }

    for network in wifi_networks.unwrap() {
        let notification = WebSocketMessage {
            op: 0,
            t: Some("WIFI_NETWORK_FOUND".to_string()),
            d: Some(json!({
                "mac": network.mac,
                "ssid": network.ssid,
                "channel": network.channel,
                "signal_level": network.signal_level,
                "capability": network.capability

            })),
        };

        tx.send(notification).expect("Failed to send notification");
    }

    Ok(())
}

/// Creates a wpa_supplicant configuration file for a given SSID and PSK.
///
/// # Arguments
///
/// * `ssid` - The SSID of the Wi-Fi network.
/// * `psk` - The Pre-Shared Key (password) for the Wi-Fi network. If empty, it denotes an open network.
///
/// # Returns
///
/// A result indicating success or failure. On success, returns `Ok(())`. On failure, returns an error.
async fn create_wpa_supplicant_conf(ssid: String, psk: String) -> Result<(), Box<dyn Error>> {
    let mut config = String::new();

    // Start constructing the network configuration
    writeln!(&mut config, "network={{")?;
    writeln!(&mut config, "\tssid=\"{}\"", ssid)?;

    // Conditionally add the PSK or mark as open network
    if !psk.is_empty() {
        writeln!(&mut config, "\tpsk=\"{}\"", psk)?;
    } else {
        writeln!(&mut config, "\tkey_mgmt=NONE")?;
    }

    // Close the network configuration block
    writeln!(&mut config, "}}")?;

    // Write the configuration to the wpa_supplicant file
    fs::write("/etc/wpa_supplicant/wpa_supplicant.conf", config)?;
    Ok(())
}

async fn restart_wpa_supplicant() -> Result<(), Box<dyn Error>> {
    Command::new("systemctl")
        .args(&["restart", "wpa_supplicant"])
        .status()
        .await?;

    Ok(())
}


pub async fn connect_to_wifi(ssid: String, psk: String) -> Result<(), Box<dyn Error>> {
    create_wpa_supplicant_conf(ssid, psk).await?;
    restart_wpa_supplicant().await?;

    Ok(())
}

pub async fn disconnect_wifi() -> Result<(), Box<dyn Error>> {
    create_wpa_supplicant_conf(String::from(""), String::from("")).await?;
    restart_wpa_supplicant().await?;

    Ok(())
}

