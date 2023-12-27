use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WebSocketMessage {
    pub t: Option<String>,
    // Event type
    pub op: u8,
    // Operation code
    pub d: Option<serde_json::Value>, // Data
}