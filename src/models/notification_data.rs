use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct NotificationData {
    pub message: String,
    pub timestamp: String,
}