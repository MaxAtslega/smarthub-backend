use std::fs::File;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use linux_embedded_hal::spidev::{SpidevOptions, SpiModeFlags};
use linux_embedded_hal::{Delay, SpidevBus, SysfsPin};
use linux_embedded_hal::sysfs_gpio::{Direction};
use mfrc522::comm::blocking::spi::SpiInterface;
use mfrc522::Mfrc522;
use serde_json::json;
use tokio::sync::broadcast::Sender;
use tokio::sync::oneshot;
use embedded_hal::delay::DelayNs;
use embedded_hal_bus::spi::ExclusiveDevice;

use crate::common::utils;
use crate::hardware::display::{get_display_power, set_display_power};
use crate::models::websocket::WebSocketMessage;

#[tokio::main]
pub async fn control_rfid(tx: Sender<WebSocketMessage>, mut shutdown_rx: oneshot::Receiver<()>, last_event_time: Arc<Mutex<Instant>>) -> Result<(), String> {
    if !utils::is_raspberry_pi_4b() {
        return Err("This app is only compatible with Raspberry Pi 4 Model B".to_string());
    }
    let mut bl_power_file = File::create("/sys/class/backlight/10-0045/bl_power").unwrap();

    let mut delay = Delay;
    
    let mut spi = SpidevBus::open("/dev/spidev0.0").unwrap();
    let options = SpidevOptions::new()
        .max_speed_hz(1_000_000)
        .mode(SpiModeFlags::SPI_MODE_0 | SpiModeFlags::SPI_NO_CS)
        .build();
    spi.configure(&options).unwrap();

    let pin = SysfsPin::new(22);
    pin.export().unwrap();
    while !pin.is_exported() {}
    delay.delay_ms(500u32);
    
    let pin = pin.into_output_pin(embedded_hal::digital::PinState::High).unwrap();
    
    pin.set_direction(Direction::Out).unwrap();
    pin.set_value(1).unwrap();

    let spi = ExclusiveDevice::new(spi, pin, Delay);
    let itf = SpiInterface::new(spi);
    let mut mfrc522 = Mfrc522::new(itf).init().unwrap();

    let vers = mfrc522.version().unwrap();

    log::debug!("VERSION: 0x{:x}", vers);

    let mut last_sent = Instant::now();
    let mut last_uid = None;

    loop {
        if let Ok(atqa) = mfrc522.reqa() {
            if let Ok(uid) = mfrc522.select(&atqa) {
                let uid_str = format!("{:?}", uid.as_bytes());

                // Check if the UID is different from the last sent or if 5 seconds have passed
                if last_uid.as_ref() != Some(&uid_str) || last_sent.elapsed() >= Duration::from_secs(5) {
                    let notif = WebSocketMessage {
                        t: Some("RFID_DETECT".to_string()),
                        op: 1,
                        d: Some(json!(uid.as_bytes())),
                    };

                    tx.send(notif).unwrap();

                    if get_display_power().contains("1") {
                        set_display_power(&mut bl_power_file, true);

                        let notification = WebSocketMessage {
                            t: Some("DISPLAY_STATUS".to_string()),
                            op: 0,
                            d: Some(json!({"status": "on"})),
                        };

                        tx.send(notification).unwrap();

                        let mut guard = last_event_time.lock().unwrap();
                        *guard = Instant::now();
                    }

                    last_uid = Some(uid_str);
                    last_sent = Instant::now();
                }
            }
        }

        if shutdown_rx.try_recv().is_ok() {
            break;
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    Ok(())
}