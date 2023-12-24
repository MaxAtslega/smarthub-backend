use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub t: Option<String>, // Event type
    pub op: u8, // Operation code
    pub d: serde_json::Value, // Data
}