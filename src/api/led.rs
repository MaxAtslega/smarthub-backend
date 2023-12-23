use rocket::serde::json::Json;
use rppal::gpio::{Gpio, Error as GpioError};
use rocket::response::status;

const LED_RED_PIN: u8 = 21;
const LED_GREEN_PIN: u8 = 20;
const LED_BLUE_PIN: u8 = 16;

#[derive(serde::Serialize)]
pub struct InfoResponse {
    message: String,
}

// Function to flash an LED
async fn flash_led(pin: u8) -> Result<(), GpioError> {
    let gpio = Gpio::new()?;
    let mut led = gpio.get(pin)?.into_output();

    for _ in 0..3 {
        led.set_high();
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        led.set_low();
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    Ok(())
}

#[post("/led/red")]
pub async fn flash_red() -> Result<Json<InfoResponse>, status::Custom<Json<InfoResponse>>> {
    tokio::spawn(async {
        if let Err(e) = flash_led(LED_RED_PIN).await {
            error!("Error flashing red LED: {:?}", e);
        }
    });

    Ok(Json(InfoResponse { message: "Flashing red LED initiated.".into() }))
}

#[post("/led/green")]
pub async fn flash_green() -> Result<Json<InfoResponse>, status::Custom<Json<InfoResponse>>> {
    tokio::spawn(async {
        if let Err(e) = flash_led(LED_GREEN_PIN).await {
            error!("Error flashing green LED: {:?}", e);
        }
    });

    Ok(Json(InfoResponse { message: "Flashing green LED initiated.".into() }))
}

#[post("/led/blue")]
pub async fn flash_blue() -> Result<Json<InfoResponse>, status::Custom<Json<InfoResponse>>> {
    tokio::spawn(async {
        if let Err(e) = flash_led(LED_BLUE_PIN).await {
            error!("Error flashing green LED: {:?}", e);
        }
    });

    Ok(Json(InfoResponse { message: "Flashing green LED initiated.".into() }))
}