use std::time::SystemTime;
use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Display)]
#[display(fmt = "from {} to {} amt {}", from, to, amount)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub time: SystemTime,
    pub amount: i32,
}
