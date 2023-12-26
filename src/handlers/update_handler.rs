use std::error::Error;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::json;
use tokio::sync::broadcast::Sender;

use crate::models::notification_response::NotificationResponse;

pub async fn get_available_updates(tx: Sender<NotificationResponse>) -> Result<(), Box<dyn Error>> {
    let notification = NotificationResponse {
        op: 4,
        title: "START_LISTING_UPDATE".to_string(),
        data: json!({"message": "Start listing updates"}),
    };

    tx.send(notification).expect("Failed to send notification");

    let update_status = Command::new("sh")
        .args(&["-c", "apt update"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    println!("apt update finished with status: {:?}", update_status);

    let child = Command::new("sh")
        .args(&["-c", "apt list --upgradable -a"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn command");

    let stdout = BufReader::new(child.stdout.unwrap());
    let start = Instant::now();

    for line in stdout.lines() {
        match line {
            Ok(line) => {
                if line.contains("Listing...") || line.is_empty() {
                    continue;
                }

                let notification = NotificationResponse {
                    op: 4,
                    title: "UPDATE_AVAILABLE".to_string(),
                    data: json!({"message": line}),
                };

                tx.send(notification).expect("Failed to send notification");
            }
            Err(e) => {}
        }
    }

    let notification = NotificationResponse {
        op: 4,
        title: "FINISHED_LISTING_UPDATE".to_string(),
        data: json!({"message": "Finished listing updates"}),
    };

    tx.send(notification).expect("Failed to send notification");

    Ok(())
}

pub async fn perform_system_update(tx: Sender<NotificationResponse>) -> Result<(), Box<dyn Error>> {
    let notification = NotificationResponse {
        op: 4,
        title: "START_UPDATE_PROCESS".to_string(),
        data: json!({"message": "Starting system update process"}),
    };
    tx.send(notification).expect("Failed to send notification");


    std::thread::sleep(Duration::from_millis(10000));

    // Execute the update command
    let update_status = Command::new("sh")
        .args(&["-c", "apt-get upgrade -y"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    match update_status {
        Ok(status) if status.success() => {
            let success_notification = NotificationResponse {
                op: 4,
                title: "UPDATE_SUCCESS".to_string(),
                data: json!({"message": "System update completed successfully"}),
            };
            tx.send(success_notification).expect("Failed to send notification");
        }
        _ => {
            let fail_notification = NotificationResponse {
                op: 4,
                title: "UPDATE_FAILURE".to_string(),
                data: json!({"message": "System update failed"}),
            };
            tx.send(fail_notification).expect("Failed to send notification");
        }
    }

    Ok(())
}
