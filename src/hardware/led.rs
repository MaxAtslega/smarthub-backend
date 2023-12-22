use rppal::gpio::{Gpio, Error as GpioError};
use mfrc522::comm::{eh02::spi::SpiInterface, Interface};
use mfrc522::{Mfrc522};

use embedded_hal::blocking::spi::{Transfer as SpiTransfer, Write as SpiWrite};
use embedded_hal::digital::v2::OutputPin;
use linux_embedded_hal::spidev::{SpiModeFlags, SpidevOptions};
use linux_embedded_hal::sysfs_gpio::Direction;
use linux_embedded_hal::{Pin, Spidev};
use tokio::sync::broadcast;

pub async fn control_led(tx: broadcast::Sender<String>) -> Result<(), GpioError> {
    Gpio::new()?;

    let mut spi = Spidev::open("/dev/spidev0.0").unwrap();
    let options = SpidevOptions::new()
        .max_speed_hz(1_000_000)
        .mode(SpiModeFlags::SPI_MODE_0)
        .build();
    spi.configure(&options).unwrap();

    let pin = Pin::new(22);
    pin.export().unwrap();
    while !pin.is_exported() {}

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    pin.set_direction(Direction::Out).unwrap();
    pin.set_value(1).unwrap();


    let itf = SpiInterface::new(spi).with_nss(pin);
    let mut mfrc522 = Mfrc522::new(itf).init().unwrap();

    let vers = mfrc522.version().unwrap();

    log::debug!("VERSION: 0x{:x}", vers);


    loop {
        if let Ok(atqa) = mfrc522.reqa() {
            if let Ok(uid) = mfrc522.select(&atqa) {
                log::info!("UID: {:?}", uid.as_bytes());
                let uid_str = format!("UID: {:?}", uid.as_bytes());
                tx.send(uid_str).unwrap(); // Send UID over the channel
            }
        }


    }
}