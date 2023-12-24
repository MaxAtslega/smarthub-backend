use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum LEDType {
    RED = 21,
    GREEN = 20,
    BLUE = 16,
}
