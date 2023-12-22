use crate::{Config, routes};
use crate::hardware::led::control_led;
use log::{info, error};
use rocket::{Error, Ignite, Rocket};

pub fn launch(conf: &Config) -> Result<Rocket<Ignite>, Error> {
    // Print welcome message
    info!("Starting App in {}", conf.app.environment);

    // Spawn the task for controlling the LED
    tokio::spawn(async {
        if let Err(e) = control_led().await {
            error!("Failed in control_led: {}", e);
        }
    });

    // Build a multi-threaded Tokio runtime
    tokio::runtime::Builder::new_multi_thread()
        .thread_name("rocket-worker-thread")
        .enable_all()
        .build()
        .expect("create tokio runtime")
        .block_on(routes::init(conf))
}

