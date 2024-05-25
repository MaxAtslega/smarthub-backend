use std::process::Command;

use axum::http::StatusCode;
use axum::Json;
use log::debug;
use serde_derive::{Deserialize, Serialize};

use crate::api::ErrorMessage;

#[derive(Serialize)]
pub struct InfoResponse {
    health: String,
    version: String,
    app_name: String,
    app_description: String,
}

#[derive(Serialize)]
pub struct MessageResponse {
    message: String,
}

#[derive(Serialize)]
pub struct NetworkStatusResponse {
    pub ssid: String,
    pub status: String,
    pub ip_address: String,
}

const VERSION: &str = env!("CARGO_PKG_VERSION");
const NAME: &str = env!("CARGO_PKG_NAME");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

pub async fn get_info() -> Result<Json<InfoResponse>, (StatusCode, String)> {
    Ok(Json(InfoResponse {
        version: VERSION.to_string(),
        health: "healthy".to_string(),
        app_description: DESCRIPTION.to_string(),
        app_name: NAME.to_string(),
    }))
}

pub async fn post_reboot() -> Result<(), (StatusCode, Json<ErrorMessage>)> {
    debug!("Rebooting system...");
    let status = Command::new("sudo")
        .arg("reboot")
        .status();

    match status {
        Ok(_) => Ok(()),
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorMessage { message: "Failed to reboot".to_string() }))),
    }
}

pub async fn post_shutdown() -> Result<Json<()>, (StatusCode, Json<ErrorMessage>)> {
    debug!("Shutting down system...");
    let status = Command::new("sudo")
        .arg("shutdown")
        .arg("now")
        .status();

    match status {
        Ok(_) => Ok(Json(())),
        Err(_) => {
            let error_message = Json(ErrorMessage {
                message: "Failed to shutdown".to_string(),
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, error_message))
        }
    }
}

pub async fn start_wpa_supplicant() -> Result<Json<MessageResponse>, (StatusCode, Json<ErrorMessage>)> {
    debug!("Starting wpa_supplicant...");
    let status = Command::new("sudo")
        .arg("systemctl")
        .arg("start")
        .arg("wpa_supplicant")
        .status();

    match status {
        Ok(_) => Ok(Json(MessageResponse { message: "wpa_supplicant started".to_string() })),
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorMessage { message: "Failed to start wpa_supplicant".to_string() }))),
    }
}

pub async fn stop_wpa_supplicant() -> Result<Json<MessageResponse>, (StatusCode, Json<ErrorMessage>)> {
    debug!("Stopping wpa_supplicant...");
    let status = Command::new("sudo")
        .arg("systemctl")
        .arg("stop")
        .arg("wpa_supplicant")
        .status();

    match status {
        Ok(_) => Ok(Json(MessageResponse { message: "wpa_supplicant stopped".to_string() })),
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorMessage { message: "Failed to stop wpa_supplicant".to_string() }))),
    }
}

pub async fn start_scan() -> Result<Json<MessageResponse>, (StatusCode, Json<ErrorMessage>)> {
    debug!("Starting Wi-Fi scan...");
    let status = Command::new("wpa_cli")
        .arg("scan")
        .arg("-i")
        .arg("wlan0")
        .status();

    match status {
        Ok(_) => Ok(Json(MessageResponse { message: "Scan started".to_string() })),
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorMessage { message: "Failed to start scan".to_string() }))),
    }
}

#[derive(Serialize)]
pub struct ScanResult {
    bssid: String,
    frequency: String,
    signal_level: String,
    flags: String,
    ssid: String,
}

pub async fn get_scan_results() -> Result<Json<Vec<ScanResult>>, (StatusCode, Json<ErrorMessage>)> {
    debug!("Retrieving scan results...");
    let output = Command::new("wpa_cli")
        .arg("scan_results")
        .arg("-i")
        .arg("wlan0")
        .output();

    match output {
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

                Ok(Json(results))
            } else {
                Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorMessage { message: "Failed to get scan results".to_string() })))
            }
        }
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorMessage { message: "Failed to execute scan results command".to_string() }))),
    }
}

pub async fn get_current_network_status() -> Result<Json<NetworkStatusResponse>, (StatusCode, Json<ErrorMessage>)> {
    let output = Command::new("wpa_cli")
        .arg("status")
        .arg("-i")
        .arg("wlan0")
        .output();

    match output {
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

                Ok(Json(NetworkStatusResponse {
                    ssid,
                    status,
                    ip_address,
                }))
            } else {
                Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorMessage { message: "Failed to get network status".to_string() })))
            }
        }
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorMessage { message: "Failed to execute status command".to_string() }))),
    }
}

#[derive(Deserialize)]
pub struct WifiCredentials {
    ssid: String,
    psk: Option<String>, // PSK is optional for open networks
}

pub async fn connect_wifi(Json(credentials): Json<WifiCredentials>) -> Result<Json<MessageResponse>, (StatusCode, Json<ErrorMessage>)> {
    debug!("Connecting to Wi-Fi...");

    // Create the wpa_supplicant configuration content
    let config_content = if let Some(psk) = credentials.psk {
        format!(
            "network={{\n\tssid=\"{}\"\n\tpsk=\"{}\"\n}}",
            credentials.ssid, psk
        )
    } else {
        format!(
            "network={{\n\tssid=\"{}\"\n\tkey_mgmt=NONE\n}}",
            credentials.ssid
        )
    };

    // Write the configuration to the wpa_supplicant.conf file
    let write_status = std::fs::write("/etc/wpa_supplicant/wpa_supplicant.conf", config_content);

    if write_status.is_err() {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorMessage {
                message: "Failed to write wpa_supplicant configuration".to_string(),
            }),
        ));
    }

    // Restart wpa_supplicant to apply the new configuration
    let reconfig_status = tokio::process::Command::new("systemctl")
        .args(&["restart", "wpa_supplicant"])
        .status()
        .await;

    match reconfig_status {
        Ok(_) => Ok(Json(MessageResponse {
            message: "Connected to Wi-Fi".to_string(),
        })),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorMessage {
                message: "Failed to reconfigure wpa_supplicant".to_string(),
            }),
        )),
    }
}

pub async fn disconnect_wifi() -> Result<Json<MessageResponse>, (StatusCode, Json<ErrorMessage>)> {
    debug!("Disconnecting from Wi-Fi...");

    let status = Command::new("sudo")
        .arg("wpa_cli")
        .arg("-i")
        .arg("wlan0")
        .arg("disconnect")
        .status();

    match status {
        Ok(_) => Ok(Json(MessageResponse {
            message: "Disconnected from Wi-Fi".to_string(),
        })),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorMessage {
                message: "Failed to disconnect from Wi-Fi".to_string(),
            }),
        )),
    }
}