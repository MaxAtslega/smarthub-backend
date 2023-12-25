use std::time::{Duration, Instant};

use linux_embedded_hal::{Pin, Spidev};
use linux_embedded_hal::spidev::{SpidevOptions, SpiModeFlags};
use linux_embedded_hal::sysfs_gpio::Direction;
use mfrc522::comm::eh02::spi::SpiInterface;
use mfrc522::Mfrc522;
use serde_json::json;
use tokio::sync::broadcast::Sender;
use tokio::sync::oneshot;

use crate::common::utils;
use crate::models::notification_response::NotificationResponse;

#[tokio::main]
pub async fn control_rfid(tx: Sender<NotificationResponse>, mut shutdown_rx: oneshot::Receiver<()>) -> Result<(), String> {
    if !utils::is_raspberry_pi_4b() {
        return Err("This app is only compatible with Raspberry Pi 4 Model B".to_string());
    }

    let mut spi = Spidev::open("/dev/spidev0.0").unwrap();
    let options = SpidevOptions::new()
        .max_speed_hz(1_000_000)
        .mode(SpiModeFlags::SPI_MODE_0)
        .build();
    spi.configure(&options).unwrap();

    let pin = Pin::new(22);
    pin.export().unwrap();
    while !pin.is_exported() {}

    pin.set_direction(Direction::Out).unwrap();
    pin.set_value(1).unwrap();


    let itf = SpiInterface::new(spi).with_nss(pin);
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
                    let notif = NotificationResponse {
                        title: "RFID_DETECT".to_string(),
                        op: 1,
                        data: json!(uid.as_bytes()),
                    };

                    tx.send(notif).unwrap();
                    last_uid = Some(uid_str);
                    last_sent = Instant::now();
                }
            }
        }

        if shutdown_rx.try_recv().is_ok() {
            break;
        }

        std::thread::sleep(Duration::from_millis(100));
    }

    Ok(())
}

#[tokio::main]
#[warn(dead_code)]
pub async fn test(tx: Sender<NotificationResponse>, mut shutdown_rx: oneshot::Receiver<()>) -> Result<(), ()> {
    loop {
        if shutdown_rx.try_recv().is_ok() {
            break;
        }

        std::thread::sleep(Duration::from_millis(10000));

        let notif = NotificationResponse {
            title: "RFID_DETECT".to_string(),
            op: 1,
            data: json!("Test"),
        };

        tx.send(notif).unwrap();
    }

    Ok(())
}