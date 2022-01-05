use crate::transaction::Transaction;
use serde::{Deserialize, Serialize};
use sha2::{digest::generic_array::GenericArray, Digest, Sha256};
use std::time::SystemTime;

#[derive(Serialize, Deserialize, Clone)]
pub struct Block {
    hash: String,
    prev_hash: String,
    transactions: Vec<Transaction>,
    time: SystemTime,
    index: u128,
}

impl Block {
    pub fn generate_hash(self) {}

    pub fn digest(&self) -> String {
        let block_string = serde_json::to_string(&self);

        let mut hashed = Sha256::new().chain_update(block_string.unwrap()).finalize();

        println!("Hashed {:x}", hashed);

        format!("{:x}", hashed)
    }

    pub fn is_valid(&self, prev_block: &Block) -> bool {
        self.prev_hash == prev_block.hash
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{block::Block, transaction::Transaction};
    use std::time::SystemTime;

    pub fn generate_blocks() -> Vec<Block> {
        let mut time_now = SystemTime::now();

        let tx1 = Transaction{
            from: String::from("Alice"),
            to: String::from("Bob"),
            time: time_now,
            amount: 32
        };

        let new_block = Block {
            hash: String::from("123"),
            prev_hash: String::from("123"),
            transactions: vec![tx1.clone()],
            time: time_now,
            index: 0,
        };

        let same_block = Block {
            hash: String::from("123"),
            prev_hash: String::from("123"),
            transactions: vec![tx1.clone()],
            time: time_now,
            index: 0,
        };

        let next_block = Block{
            hash: String::from("new_hash"),
            prev_hash: String::from("123"),
            transactions: vec![tx1.clone()],
            time: time_now,
            index: 0,
        };

        vec![new_block, same_block, next_block]
    }

    #[test]
    fn test_block() {

        let mut time_now = SystemTime::now();

        let tx1 = Transaction{
            from: String::from("Alice"),
            to: String::from("Bob"),
            time: time_now,
            amount: 32
        };

        let blocks = generate_blocks();
        let new_block = &blocks[0];
        let next_block = &blocks[2];
        let same_block = &blocks[1];


        assert!(next_block.is_valid(&new_block));

        let first_block_digest = new_block.digest();
        assert_eq!(first_block_digest, same_block.digest());

        let time_now2 = SystemTime::now();
        let second_block_time_differ = Block {
            hash: String::from("123"),
            prev_hash: String::from("123"),
            transactions: vec![tx1.clone()],
            time: time_now2,
            index: 0,
        };

        assert_ne!(first_block_digest, second_block_time_differ.clone().digest());

        let tx2 = Transaction{
            from: String::from("Bob"),
            to: String::from("Alice"),
            time: time_now2,
            amount: 32
        };
        let second_block_txs_differ = Block {
            hash: String::from("123"),
            prev_hash: String::from("123"),
            transactions: vec![tx1,tx2],
            time: time_now2,
            index: 0,
        };

        assert_ne!(second_block_time_differ.digest(), second_block_txs_differ.digest());

    }
}
