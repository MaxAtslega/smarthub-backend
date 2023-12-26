use std::error::Error;
use libc::printf;
use serde_json::json;
use tokio::sync::broadcast::Sender;
use crate::models::notification_response::NotificationResponse;
use crate::network::interfaces::get_interfaces;
use crate::network::wifi_scan;

pub async fn get_network_interfaces(tx: Sender<NotificationResponse>) -> Result<(), Box<dyn Error>>{
    let interfaces = get_interfaces();

    if interfaces.is_err() {
        return Err(Box::new(interfaces.err().unwrap()));
    }

    let notification = NotificationResponse {
        op: 0,
        title: "NETWORK_INTERFACES".to_string(),
        data: json!(interfaces.unwrap()),
    };

    tx.send(notification).expect("Failed to send notification");

    Ok(())
}


pub async fn scan_wifi(tx: Sender<NotificationResponse>) -> Result<(), Box<dyn Error>>{
    let wifi_networks = wifi_scan::scan().await;

    if wifi_networks.is_err() {
        return Err(Box::new(wifi_networks.err().unwrap()));
    }

    for network in wifi_networks.unwrap() {
        let notification = NotificationResponse {
            op: 0,
            title: "WIFI_NETWORK_FOUND".to_string(),
            data: json!({
                "mac": network.mac,
                "ssid": network.ssid,
                "channel": network.channel,
                "signal_level": network.signal_level,
                "security": network.security
            }),
        };

        tx.send(notification).expect("Failed to send notification");
    }

    Ok(())
}