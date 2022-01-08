mod block;
mod blockchain;
mod node;
mod transaction;

use blockchain::Blockchain;
use dialoguer::{theme::ColorfulTheme, Select};

pub fn handle_print_chain(chain: &Blockchain){
    println!("{}", chain);
}

fn main() {
    let selections = &[
        "Create block",
        "View local blockchain",
        "Generate transaction",
    ];

    let mut blockchain = Blockchain::new(0, 5, 256);
    let mut node = node::Node { blockchain };
    loop {
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Pick option")
            .default(0)
            .items(&selections[..])
            .interact()
            .unwrap();
        if selection == 0{
            let suc = node.blockchain.add_block(vec![]);
        }
        if selection == 1{
            handle_print_chain(&node.blockchain);
        }
    }
}
