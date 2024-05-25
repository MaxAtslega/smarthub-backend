use std::error::Error;
use std::fs;
use serde_derive::Serialize;

use serde_json::json;
use tokio::process::Command;
use tokio::sync::broadcast::Sender;

use crate::models::websocket::WebSocketMessage;
use crate::network::interfaces::get_interfaces;

#[derive(Serialize)]
pub struct ScanResult {
    bssid: String,
    frequency: String,
    signal_level: String,
    flags: String,
    ssid: String,
}

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

pub async fn get_current_network_status(tx: Sender<WebSocketMessage>) -> Result<(), Box<dyn Error>> {
    let output_result = Command::new("wpa_cli")
        .arg("status")
        .arg("-i")
        .arg("wlan0")
        .output().await;

    match output_result {
        Ok(output) => {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let mut ssid = String::new();
                let mut status = String::new();
                let mut ip_address = String::new();

                for line in output_str.lines() {
                    if line.starts_with("ssid=") {
                        ssid = line.split('=').nth(1).unwrap_or("").to_string();
                    }
                    if line.starts_with("wpa_state=") {
                        status = line.split('=').nth(1).unwrap_or("").to_string();
                    }
                    if line.starts_with("ip_address=") {
                        ip_address = line.split('=').nth(1).unwrap_or("").to_string();
                    }
                }

                let notification = WebSocketMessage {
                    op: 0,
                    t: Some("NETWORK_STATUS".to_string()),
                    d: Some(json!({
                        "ssid": ssid,
                        "status": status,
                        "ip_address": ip_address,
                    })),
                };

                tx.send(notification).expect("Failed to send notification");
                Ok(())
            } else {
                let notification = WebSocketMessage {
                    op: 0,
                    t: Some("NETWORK_STATUS".to_string()),
                    d: Some(json!({
                        "status": "DEACTIVATED",
                    })),
                };

                tx.send(notification)?;

                Ok(())
            }
        }
        Err(_) => {
            let notification = WebSocketMessage {
                op: 0,
                t: Some("NETWORK_STATUS".to_string()),
                d: Some(json!({
                        "status": "DEACTIVATED",
                    })),
            };

            tx.send(notification)?;
            
            Ok(())
        },
    }
}

pub async fn get_scan_results(tx: Sender<WebSocketMessage>) -> Result<(), Box<dyn Error>> {
    let output_result = Command::new("wpa_cli")
        .arg("scan_results")
        .arg("-i")
        .arg("wlan0")
        .output()
        .await;

    match output_result {
        Ok(output) => {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let mut results = Vec::new();

                for line in output_str.lines().skip(1) { // Skip the header line
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 5 {
                        results.push(ScanResult {
                            bssid: parts[0].to_string(),
                            frequency: parts[1].to_string(),
                            signal_level: parts[2].to_string(),
                            flags: parts[3].to_string(),
                            ssid: parts[4..].join(" "),
                        });
                    }
                }

                let notification = WebSocketMessage {
                    op: 0,
                    t: Some("SCAN_RESULTS".to_string()),
                    d: Some(json!(results)),
                };

                tx.send(notification)?;
                
                Ok(())
            } else {
                let notification = WebSocketMessage {
                    op: 0,
                    t: Some("SCAN_RESULTS".to_string()),
                    d: Some(json!([])),
                };

                tx.send(notification)?;

                Ok(())
            }
        }
        Err(_) => {
            let notification = WebSocketMessage {
                op: 0,
                t: Some("SCAN_RESULTS".to_string()),
                d: Some(json!([])),
            };

            tx.send(notification)?;

            Ok(())
        },
    }
}
