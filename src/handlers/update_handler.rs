use std::error::Error;
use tokio::io::{AsyncBufReadExt, BufReader};
use std::process::{Stdio};
use log::debug;
use tokio::process::Command;

use serde_json::json;
use tokio::sync::broadcast::Sender;
use crate::models::websocket::WebSocketMessage;

pub async fn get_available_updates(tx: Sender<WebSocketMessage>) -> Result<(), Box<dyn Error>> {
    let notification = WebSocketMessage {
        op: 4,
        t: Some("START_LISTING_UPDATE".to_string()),
        d: Some(json!({"message": "Start listing updates"})),
    };

    tx.send(notification).expect("Failed to send notification");

    let update_status = Command::new("sh")
        .args(&["-c", "apt update"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status().await;

    debug!("apt update finished with status: {:?}", update_status);

    let mut child = Command::new("sh")
        .args(&["-c", "apt list --upgradable -a"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn().expect("Failed to spawn apt list command");

    let mut stdout = BufReader::new(child.stdout.take().unwrap()).lines();

    while let Some(line) = stdout.next_line().await? {
        if line.contains("Listing...") || line.is_empty() {
            continue;
        }

        let notification = WebSocketMessage {
            op: 4,
            t: Some("UPDATE_AVAILABLE".to_string()),
            d: Some(json!({"message": line})),
        };

        tx.send(notification).expect("Failed to send notification");
    }

    let notification = WebSocketMessage {
        op: 4,
        t: Some("FINISHED_LISTING_UPDATE".to_string()),
        d: Some(json!({"message": "Finished listing updates"})),
    };

    tx.send(notification).expect("Failed to send notification");

    Ok(())
}

pub async fn perform_system_update(tx: Sender<WebSocketMessage>) -> Result<(), Box<dyn Error>> {
    let notification = WebSocketMessage {
        op: 4,
        t: Some("START_UPDATE_PROCESS".to_string()),
        d: Some(json!({"message": "Starting system update process"})),
    };
    tx.send(notification).expect("Failed to send notification");


    // Execute the update command
    let update_status = Command::new("sh")
        .args(&["-c", "apt-get upgrade -y"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status().await;

    match update_status {
        Ok(status) if status.success() => {
            let success_notification = WebSocketMessage {
                op: 4,
                t: Some("UPDATE_SUCCESS".to_string()),
                d: Some(json!({"message": "System update completed successfully"})),
            };
            tx.send(success_notification).expect("Failed to send notification");
        }
        _ => {
            let fail_notification = WebSocketMessage {
                op: 4,
                t: Some("UPDATE_FAILURE".to_string()),
                d: Some(json!({"message": "System update failed"})),
            };
            tx.send(fail_notification).expect("Failed to send notification");
        }
    }

    Ok(())
}
