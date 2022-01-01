use crate::transaction::Transaction;
use serde::{Deserialize, Serialize};
use sha2::{digest::generic_array::GenericArray, Digest, Sha256};
use std::time::SystemTime;

#[derive(Serialize, Deserialize)]
pub struct Block {
    hash: String,
    prev_hash: String,
    transactions: Vec<Transaction>,
    time: SystemTime,
    index: u128,
}

impl Block {
    pub fn generate_hash(self) {}

    pub fn digest(self) -> String {
        let block_string = serde_json::to_string(&self);

        let mut hashed = Sha256::new().chain_update(block_string.unwrap()).finalize();

        println!("Hashed {:x}", hashed);

        format!("{:x}", hashed)
    }
}

#[cfg(test)]
mod tests {
    use crate::block::Block;
    use std::time::SystemTime;

    #[test]
    fn test_block() {
        let mut time_now = SystemTime::now();

        let new_block = Block {
            hash: String::from("123"),
            prev_hash: String::from("123"),
            transactions: vec![],
            time: time_now,
            index: 0,
        };

        let same_block = Block {
            hash: String::from("123"),
            prev_hash: String::from("123"),
            transactions: vec![],
            time: time_now,
            index: 0,
        };

        let first_block_digest = new_block.digest();
        assert_eq!(first_block_digest, same_block.digest());

        time_now = SystemTime::now();
        let second_block = Block {
            hash: String::from("123"),
            prev_hash: String::from("123"),
            transactions: vec![],
            time: time_now,
            index: 0,
        };

        assert_ne!(first_block_digest, second_block.digest());
    }
}
