use std::time::SystemTime;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub time: SystemTime,
    pub amount: i32,
}
