use crate::{Config, routes};
use crate::hardware::led::control_led;
use log::{info, error};
use rocket::{Error, Ignite, Rocket};

pub fn launch(conf: &Config) -> Result<Rocket<Ignite>, Error> {
    // Print welcome message
    info!("Starting App in {}", conf.app.environment);

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .thread_name("rocket-worker-thread")
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime");

    // Run the async block within the Tokio runtime
    runtime.block_on(async {
        // Spawn the task for controlling the LED
        tokio::spawn(async {
            if let Err(e) = control_led().await {
                error!("Failed in control_led: {}", e);
            }
        });

        // Initialize routes and return the Rocket instance
        match routes::init(conf).await {
            Ok(rocket) => Ok(rocket),
            Err(e) => Err(e),
        }
    })
}

