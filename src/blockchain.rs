use crate::{block::Block, transaction::Transaction};
use rand::Rng;
use rayon::prelude::*;
use std::time::SystemTime;

pub struct Blockchain {
    chain: Vec<Block>,
    difficulty: usize,
    concurrent_hashes: u64,
    min_tx_per_block: u8,
}

impl Blockchain {
    pub fn new(min_tx_per_block: u8, difficulty: usize, concurrent_hashes: u64) -> Self {
        Blockchain {
            chain: vec![],
            difficulty,
            concurrent_hashes,
            min_tx_per_block,
        }
    }

    pub fn is_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            if !(self
                .chain
                .get(i)
                .unwrap()
                .is_valid(self.chain.get(i - 1).unwrap()))
            {
                return false;
            }
        }

        true
    }

    fn mine_block(
        &self,
        nonce: u64,
        time: SystemTime,
        txs: Vec<Transaction>,
        idx: u128,
    ) -> Option<Block> {
        const CHARSET: &[u8] = b"abcdef\
                            0123456789";
        let mut rng = rand::thread_rng();

 		let mut mine_target: String = (0..self.difficulty)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();

        mine_target = mine_target.to_lowercase();

        println!("Mine target {}", mine_target);

        let mut nonces: Vec<u64> = (0..self.concurrent_hashes).map(|x| x + nonce).collect();

        let prev = match self.chain.len() {
            0 => String::new(),
            _ => self.chain.get(self.chain.len() - 1).unwrap().hash.clone(),
        };

        nonces.par_iter().find_map_any(move |&nonce| {
            let mut block = Block::new(prev.clone(), txs.clone(), nonce, time, idx);

            let hash = block.generate_hash();

            if (hash.starts_with(&mine_target)) {
                println!("\nMined! {}\n", block.hash.clone());
                return Some(block);
            }

            None
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::block::tests;
    use crate::{block::Block, blockchain::Blockchain, transaction::Transaction};
    use std::time::SystemTime;

    #[test]
    fn test_chain_validity() {
        let blocks = crate::block::tests::generate_blocks();

        let chain = Blockchain {
            chain: blocks,
            difficulty: 1,
            min_tx_per_block: 1,
            concurrent_hashes: 256,
        };

        assert!(chain.is_valid());
    }

    #[test]
    fn test_mining() {
        let mut txs: Vec<Transaction> = vec![];
        for i in 0..10 {
            txs.push(Transaction {
                from: String::from("test"),
                to: String::from(i.to_string()),
                amount: i,
                time: SystemTime::now(),
            });
        }

		let concurrent_hashes = 256;
        let chain = Blockchain::new(5, 3, concurrent_hashes);
        let mut nonce = 0;
        let time = SystemTime::now();

		loop {
  
			chain.mine_block(1, time, txs.clone(), 3);
			nonce += concurrent_hashes;
		}
	}
}
