use rppal::gpio::{Gpio, Error as GpioError};

pub const LED_RED_PIN: u8 = 21;
pub const LED_GREEN_PIN: u8 = 20;
pub const LED_BLUE_PIN: u8 = 16;

pub async fn flash_led(pin: u8) -> Result<(), GpioError> {
    let gpio = Gpio::new()?;
    let mut led = gpio.get(pin)?.into_output();

    for _ in 0..6 {
        led.set_high();
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        led.set_low();
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    Ok(())
}