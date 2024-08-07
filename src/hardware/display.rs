use std::fs;
use std::fs::File;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use evdev::{Device, EventType};
use futures_util::StreamExt;
use log::{debug, error};
use serde_json::json;
use tokio::time::interval;

use crate::common::utils;
use crate::models::websocket::WebSocketMessage;

#[tokio::main]
pub async fn display_handler_sleep(tx: tokio::sync::broadcast::Sender<WebSocketMessage>, last_event_time: Arc<Mutex<Instant>>) -> Result<(), String> {
    if !utils::is_raspberry_pi_4b() {
        return Err("It is only compatible with Raspberry Pi 4 Model B".to_string());
    }
    // Open the bl_power file for controlling the display power
    let mut bl_power_file = File::create("/sys/class/backlight/10-0045/bl_power").unwrap();

    // Create a new Device from the file
    let device_path = "/dev/input/by-path/platform-fe205000.i2c-event";

    while !std::path::Path::new(device_path).exists() {
        debug!("Waiting for {} to become available...", device_path);
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    
    let device = Device::open(device_path).unwrap();

    // Print device information
    debug!("Device: {}", device.name().unwrap_or("Unknown device"));

    // Track the last time an event occurred
    let mut last_event_time2 = last_event_time.lock().unwrap();

    let mut events = device.into_event_stream().unwrap();
    let mut timer = interval(Duration::from_secs(10));

    loop {
        tokio::select! {
            Some(msg) = events.next() => {
                if let Ok(event) = msg {
                    if event.event_type() == EventType::ABSOLUTE {
                        if get_display_power().contains("1") {
                            set_display_power(&mut bl_power_file, true);

                            let notification = WebSocketMessage {
                                t: Some("DISPLAY_STATUS".to_string()),
                                op: 0,
                                d: Some(json!({"status": "on"})),
                            };

                            tx.send(notification).unwrap();
                        }

                        *last_event_time2 = Instant::now();
                    }
                }
            }

            // Every second
            _ = timer.tick() => {
                let elapsed_time = Instant::now() - *last_event_time2;
                if elapsed_time >= Duration::from_secs(300) && get_display_power().contains("0") {
                    let notification = WebSocketMessage {
                        t: Some("DISPLAY_STATUS".to_string()),
                        op: 0,
                        d: Some(json!({"status": "off"})),
                    };

                    tx.send(notification).unwrap();
                    set_display_power(&mut bl_power_file, false);
                }
            }
        }
    }
}


pub fn set_display_power(bl_power_file: &mut File, activate: bool) {
    let power_value = if activate { "0" } else { "1" };
    if let Err(err) = bl_power_file.write_all(power_value.as_bytes()) {
        eprintln!("Error setting display power: {}", err);
    }
}


pub fn get_display_power() -> String {
    let read = fs::read_to_string("/sys/class/backlight/10-0045/bl_power");

    return match read {
        Err(err) => {
            error!("Error reading bl_power file: {}", err);
            return "0".to_string()
        }
        Ok(content) => {
            content.trim().to_string()
        }
    }
}