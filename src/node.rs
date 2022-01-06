use crate::blockchain::Blockchain;

struct Node {
    blockchain: Blockchain,
}

impl Node {
    fn resolve_chain_conflict(&mut self, other: Blockchain) {
        let own_valid = self.blockchain.is_valid();
        let other_valid = other.is_valid();
        let mut correct_chain;

        if own_valid && other_valid {
            if self.blockchain.len() >= other.len() {
                correct_chain = &self.blockchain;
            } else {
                correct_chain = &other
            }
        } else if other_valid {
            correct_chain = &other;
        } else if own_valid {
            correct_chain = &self.blockchain;
        } else {
            panic!("All chains are invalid");
        }
		

		self.blockchain.chain = correct_chain.chain.clone();
    }
}

#[cfg(test)]
mod tests {
	use crate::{blockchain::Blockchain, node::Node, block::Block};

}
