use std::fs;
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use diesel::serialize::ToSql;
use evdev::{Device, EventType};
use futures_util::StreamExt;
use log::{debug, error};
use serde_json::json;
use tokio::time::{interval, timeout};
use crate::models::websocket::WebSocketMessage;

#[tokio::main]
pub async fn display_handler_sleep(tx: tokio::sync::broadcast::Sender<WebSocketMessage>, last_event_time: Arc<Mutex<Instant>>) -> Result<(), Box<dyn std::error::Error>> {
    // Open the touchscreen device file
    let device_path = "/dev/input/event2";
    // Open the bl_power file for controlling the display power
    let mut bl_power_file = File::create("/sys/class/backlight/10-0045/bl_power")?;

    // Create a new Device from the file
    let device = Device::open(device_path)?;

    // Print device information
    debug!("Device: {}", device.name().unwrap_or("Unknown device"));

    // Track the last time an event occurred
    let mut last_event_time2 = last_event_time.lock().unwrap();

    let mut events = device.into_event_stream()?;
    let mut timer = interval(Duration::from_secs(10));

    loop {
        tokio::select! {
            Some(msg) = events.next() => {
                if let Ok(event) = msg {
                    match event.event_type() {
                        EventType::ABSOLUTE => {
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
                        _ => {}
                    }
                }
            }

            // Every second
            _ = timer.tick() => {
                let elapsed_time = Instant::now() - *last_event_time2;
                if elapsed_time >= Duration::from_secs(300) {
                    if get_display_power().contains("0") {
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