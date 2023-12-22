use rppal::gpio::{Gpio, Error as GpioError};

const LED_PIN: u8 = 17;

pub async fn control_led() -> Result<(), GpioError> {
    let gpio = Gpio::new()?;
    let mut led = gpio.get(LED_PIN)?.into_output();

    loop {
        led.set_high();
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        led.set_low();
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    }
}