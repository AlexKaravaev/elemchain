use std::time::SystemTime;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Transaction {
	pub from: String,
	pub to: String,
	pub time: SystemTime,
	pub amount: i32,
}