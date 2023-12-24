use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NotificationResponse {
    pub title: String,
    pub data: String,
}