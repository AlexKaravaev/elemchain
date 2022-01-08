mod block;
mod blockchain;
mod node;
mod p2p;
mod transaction;

use blockchain::Blockchain;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use p2p::AppBehaviour;
use rand::distributions::Alphanumeric;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use sha2::{digest::generic_array::GenericArray, Digest, Sha256};
use std::time::{Duration, SystemTime};
use std::{
    io::{stdout, Write},
    thread,
};
use transaction::Transaction;

use libp2p::{
    core::upgrade,
    futures::StreamExt,
    identity, mplex,
    noise::{Keypair, NoiseConfig, X25519Spec},
    swarm::{Swarm, SwarmBuilder},
    tcp::TokioTcpConfig,
    PeerId, Transport,
};
use tokio::{
    io::{stdin, AsyncBufReadExt, BufReader},
    select, spawn,
    sync::{mpsc, oneshot},
    time::sleep,
};

pub fn handle_print_chain(chain: &Blockchain) {
    println!("{}", chain);
}

pub async fn swarm_factory(node: node::Node) -> SwarmBuilder<AppBehaviour> {
    let id_keys = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(id_keys.public());

    let (response_sender, mut response_rcv) = mpsc::unbounded_channel();
    let (init_sender, mut init_rcv) = mpsc::unbounded_channel();

    let auth_keys = Keypair::<X25519Spec>::new()
        .into_authentic(&id_keys)
        .expect("can create auth keys");

    let transp = TokioTcpConfig::new()
        .upgrade(upgrade::Version::V1)
        .authenticate(NoiseConfig::xx(auth_keys).into_authenticated())
        .multiplex(mplex::MplexConfig::new())
        .boxed();

    let behaviour = p2p::AppBehaviour::new(peer_id, node, response_sender, init_sender.clone()).await;

    let mut swarm = SwarmBuilder::new(transp, behaviour, peer_id).executor(Box::new(|fut| {
        spawn(fut);
    }));

    swarm
}

#[tokio::main]
async fn main() {
    let selections = &[
        "Create block",
        "View local blockchain",
        "Generate transaction",
        "View nodes",
        "View pending txs",
    ];

    let blockchain = Blockchain::new(4, 3, 256);
    let node = node::Node { blockchain };
    let mut pending_txs: Vec<Transaction> = vec![];

    let (response_sender, mut response_rcv) = mpsc::unbounded_channel();
    let (init_sender, mut init_rcv) = mpsc::unbounded_channel();

    let (cli_sender, mut cli_rcv) = mpsc::unbounded_channel();

    let mut swarm = swarm_factory(node).await.build();

    Swarm::listen_on(
        &mut swarm,
        "/ip4/0.0.0.0/tcp/0"
            .parse()
            .expect("can get a local socket"),
    )
    .expect("swarm can be started");

    spawn(async move {
        sleep(Duration::from_secs(1)).await;
        init_sender.send(true).expect("can send init event");
    });

    // Wallet number will be just random generated string
    let wallet_rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect();

    let wallet = format!(
        "{:x}",
        Sha256::new().chain_update(wallet_rand_string).finalize()
    );

    let wallet_clone = wallet.clone();
    thread::spawn(move || loop {
        let selection = Select::with_theme(&ColorfulTheme::default())
            .clear(true)
            .with_prompt(format!("Your wallet num is {}\nPick option", wallet_clone))
            .default(0)
            .items(&selections[..])
            .interact()
            .unwrap();

        cli_sender.send(selection);
    });

    loop {
        let mut selection = 99;
        let evt = {
            select! {
                response = response_rcv.recv() => {
                    Some(p2p::EventType::LocalChainResponse(response.expect("response exists")))
                },
                _init = init_rcv.recv() => {
                    Some(p2p::EventType::Init)
                },
                _selection = cli_rcv.recv() => {
                    selection = _selection.unwrap();
                    Some(p2p::EventType::Cli)

                },
                event = swarm.select_next_some() => {
                    None
                },
            }
        };

        if let Some(event) = evt {
            match event {
                p2p::EventType::Init => {
                    let topic = swarm.behaviour_mut().blockchain_topic.clone();
                    let peers = p2p::get_list_peers(&swarm);
                    
                    if !peers.is_empty() {
                        let req = p2p::LocalChainRequest {
                            from_peer_id: peers
                                .iter()
                                .last()
                                .expect("at least one peer")
                                .to_string(),
                        };

                        let json = serde_json::to_string(&req).expect("can jsonify request");
                        swarm
                            .behaviour_mut()
                            .floodsub
                            .publish(topic, json.as_bytes());
                    }
                }
                p2p::EventType::LocalChainResponse(resp) => {
                    let topic = swarm.behaviour_mut().blockchain_topic.clone();
                    let json = serde_json::to_string(&resp).expect("can jsonify response");
                    swarm
                        .behaviour_mut()
                        .floodsub
                        .publish(topic, json.as_bytes());
                }
                p2p::EventType::Cli => {
                    // let selection = cli_rcv.recv().await.unwrap();
                    if selection == 0 {
                        clearscreen::clear().expect("failed to clear screen");
                        thread::sleep(Duration::from_millis(100));

                        let suc = swarm.behaviour_mut().node.blockchain.add_block(pending_txs.clone());
                        if suc {
                            // IF successfull mining, then we broadcast the block to the network
                            // https://www.oreilly.com/library/view/mastering-bitcoin/9781491902639/ch08.html
                            // However, there is no complex logic like orphans blocks or mempool here yet.
                            pending_txs.clear();
                            let topic = swarm.behaviour_mut().blockchain_topic.clone();
                            let json = serde_json::to_string(&swarm.behaviour_mut().node.blockchain.chain.last()).expect("can jsonify request");

                            swarm.behaviour_mut()
                                .floodsub
                                .publish(topic, json.as_bytes());
                        }
                    }
                    if selection == 1 {
                        clearscreen::clear().expect("failed to clear screen");
                        thread::sleep(Duration::from_millis(100));

                        handle_print_chain(&swarm.behaviour_mut().node.blockchain);
                        print!("\n");
                    }
                    if selection == 2 {
                        clearscreen::clear().expect("failed to clear screen");

                        // We will send 100 coins to random peer
                        let peers = p2p::get_list_peers(&swarm);
                        let to = peers.choose(&mut rand::thread_rng());

                        let transaction = Transaction {
                            from: wallet.clone(),
                            to: to.unwrap().to_string(),
                            amount: 100,
                            time: SystemTime::now(),
                        };
                        thread::sleep(Duration::from_millis(100));
                        println!("Generated tx \n {}", transaction);

                        pending_txs.push(transaction);
                    }
                    if selection == 3 {
                        clearscreen::clear().expect("failed to clear screen");
                        let peers = p2p::get_list_peers(&swarm);
                        thread::sleep(Duration::from_millis(100));
                        print!("Peers len {}. Peers list: \r\n", peers.len());
                        for mut peer in peers {
                            peer = peer.split_whitespace().collect();
                            print!("{}\r\n", peer);
                        }
                        print!("\n");
                    }
                    if selection == 4 {
                        clearscreen::clear().expect("failed to clear screen");
                        thread::sleep(Duration::from_millis(100));
                        print!("Total txs {}. Tx list: \r\n", pending_txs.len());
                        for i in 0..pending_txs.len() {
                            print!("{}. {} \r\n", i + 1, &pending_txs[i]);
                        }
                        print!("\n");
                    }
                }
            }
        }
    }
}
