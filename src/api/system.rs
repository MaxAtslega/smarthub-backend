use std::process::Command;

use axum::http::StatusCode;
use axum::Json;
use log::debug;
use serde_derive::Serialize;

use crate::api::ErrorMessage;

#[derive(Serialize)]
pub struct InfoResponse {
    health: String,
    version: String,
    app_name: String,
    app_description: String,
}

#[derive(Serialize)]
pub struct MessageResponse {
    health: String,
    version: String,
    app_name: String,
    app_description: String,
}

const VERSION: &str = env!("CARGO_PKG_VERSION");
const NAME: &str = env!("CARGO_PKG_NAME");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

pub async fn get_info() -> Result<Json<InfoResponse>, (StatusCode, String)> {
    Ok(Json(InfoResponse {
        version: VERSION.to_string(),
        health: "healthy".to_string(),
        app_description: DESCRIPTION.to_string(),
        app_name: NAME.to_string(),
    }))
}

pub async fn get_reboot() -> Result<(), (StatusCode, Json<ErrorMessage>)> {
    debug!("Rebooting system...");
    let status = Command::new("sudo")
        .arg("reboot")
        .status();

    match status {
        Ok(_) => Ok(()),
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorMessage { message: "Failed to reboot".to_string() }))),
    }
}

pub async fn get_shutdown() -> Result<Json<()>, (StatusCode, Json<ErrorMessage>)> {
    debug!("Shutting down system...");
    let status = Command::new("sudo")
        .arg("shutdown")
        .arg("now")
        .status();

    match status {
        Ok(_) => Ok(Json(())),
        Err(_) => {
            let error_message = Json(ErrorMessage {
                message: "Failed to shutdown".to_string(),
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, error_message))
        }
    }
}