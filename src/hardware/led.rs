use rppal::gpio::{Error as GpioError, Gpio};

use crate::enums::led_type::LEDType;

pub async fn flash_led(led_type: LEDType) -> Result<(), GpioError> {
    let gpio = Gpio::new()?;
    let mut led = gpio.get(led_type as u8)?.into_output();

    for _ in 0..6 {
        led.set_high();
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        led.set_low();
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    Ok(())
}